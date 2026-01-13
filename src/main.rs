use std::io::{self, BufRead, IsTerminal, Write};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use et::{apply_duration, format_iso, is_duration, now, parse_epoch, parse_iso, Duration, EtError};

#[derive(Parser, Debug)]
#[command(
    name = "et",
    version,
    about = "A CLI tool to print and manipulate Unix epoch timestamps.",
    long_about = "A CLI tool to print and manipulate Unix epoch timestamps.\n\n\
                  DURATION UNITS\n  \
                    s    seconds\n  \
                    m    minutes (60s)\n  \
                    h    hours (3600s)\n  \
                    d    days (86400s)\n  \
                    w    weeks (604800s)\n  \
                    M    months (calendar)\n  \
                    Y    years (calendar)\n\n\
                  Calendar units handle variable-length months and leap years.\n\
                  When adding months, days are clamped to valid range\n\
                  (e.g., Jan 31 + 1M = Feb 28/29).",
    after_help = "EXAMPLES\n  \
                  et                  Print current epoch\n  \
                  et -7d              Subtract 7 days\n  \
                  et +3h              Add 3 hours\n  \
                  et +1M              Add 1 month\n  \
                  et -1Y              Subtract 1 year\n  \
                  et 1704912345 +1h   Add 1 hour to given epoch\n  \
                  et parse 2026-01-05T12:00:00Z\n  \
                  et format 1704912345\n  \
                  echo 1704912345 | et -1d"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Epoch, duration, or 'now'
    #[arg(value_name = "ARG", allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print current epoch timestamp
    Now {
        /// Duration offset (e.g., +3h, -7d)
        #[arg(value_name = "DURATION", allow_hyphen_values = true)]
        duration: Option<String>,
    },

    /// Convert ISO-8601 timestamp to epoch
    Parse {
        /// ISO-8601 timestamp with timezone (e.g., 2026-01-05T12:00:00Z)
        #[arg(value_name = "TIMESTAMP")]
        timestamp: String,
    },

    /// Convert epoch timestamp to ISO-8601
    Format {
        /// Epoch timestamp in seconds
        #[arg(value_name = "EPOCH")]
        epoch: String,
    },
}

fn run() -> et::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Now { duration }) => {
            let epoch = now();
            let result = match duration {
                Some(d) => apply_duration(epoch, Duration::parse(&d)?)?,
                None => epoch,
            };
            println!("{result}");
        }

        Some(Command::Parse { timestamp }) => {
            let epoch = parse_iso(&timestamp)?;
            println!("{epoch}");
        }

        Some(Command::Format { epoch }) => {
            let epoch_val = parse_epoch(&epoch)?;
            let iso = format_iso(epoch_val)?;
            println!("{iso}");
        }

        None => {
            // Handle positional arguments or stdin
            handle_args_or_stdin(&cli.args)?;
        }
    }

    Ok(())
}

fn handle_args_or_stdin(args: &[String]) -> et::Result<()> {
    match args.len() {
        0 => {
            // No args - try stdin, fall back to now
            if try_process_stdin(None)? == 0 {
                println!("{}", now());
            }
        }
        1 => {
            let arg = &args[0];

            if arg == "now" {
                // `et now` - print current time
                println!("{}", now());
            } else if is_duration(arg) {
                let duration = Duration::parse(arg)?;
                // Try stdin first; if no data, apply to now
                if try_process_stdin(Some(duration))? == 0 {
                    let result = apply_duration(now(), duration)?;
                    println!("{result}");
                }
            } else {
                // `et 1704912345` - just echo the epoch
                let epoch = parse_epoch(arg)?;
                println!("{epoch}");
            }
        }
        2 => {
            // et EPOCH DURATION or et now DURATION
            let epoch = if args[0] == "now" {
                now()
            } else {
                parse_epoch(&args[0])?
            };
            let duration = Duration::parse(&args[1])?;
            let result = apply_duration(epoch, duration)?;
            println!("{result}");
        }
        _ => {
            return Err(EtError::InvalidDuration("too many arguments".to_string()));
        }
    }

    Ok(())
}

/// Try to process timestamps from stdin. Returns the number of lines processed.
/// Returns 0 if stdin is a terminal or has no data (allowing caller to fall back).
fn try_process_stdin(duration: Option<Duration>) -> et::Result<usize> {
    let stdin = io::stdin();

    // If stdin is a terminal, no data to read
    if stdin.is_terminal() {
        return Ok(0);
    }

    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    let mut count = 0;

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        count += 1;

        let epoch = parse_epoch(trimmed)?;
        let result = match duration {
            Some(d) => apply_duration(epoch, d)?,
            None => epoch,
        };

        writeln!(stdout_lock, "{result}")?;
    }

    Ok(count)
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
