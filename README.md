# epoch-time

Print or manipulate Unix epoch timestamps.

## Description

`et` (epoch time) is a CLI tool that prints and manipulates Unix epoch timestamps. All operations use
UTC. Timestamps are integers representing seconds since 1970-01-01T00:00:00Z.

With no arguments, print the current epoch. With a duration argument,
apply the offset to the current epoch. With an epoch and duration,
apply the offset to the given epoch.

When standard input is not a terminal, read epoch timestamps from
stdin (one per line) and apply any duration offset to each.

## Motivation

I needed a simple tool to quickly generate epoch timestamps for URL queries. The date command can do this, but its syntax varies across implementations (GNU coreutils on Linux, BSD on macOS, and others), making it inconvenient for quick, portable use. This tool provides a fast and consistent way to generate epoch timestamps. Here's an example of how it can be used:

```
curl "https://api.example.com/data?start=$(et now)&end=$(et now +7d)"
```

## Usage

- `et now [OFFSET]`         Print current epoch timestamp, optionally applying an offset
- `et <EPOCH> [OFFSET]`     Print an epoch timestamp with an optional offset
- `et parse <TIMESTAMP>`    Convert ISO-8601 timestamp (UTC required) to epoch
- `et format <EPOCH>`      Convert epoch to ISO-8601 UTC


## Duration Units

| Unit | Value            |
|------|------------------|
| s    | 1 second         |
| m    | 60 seconds       |
| h    | 3600 seconds     |
| d    | 86400 seconds    |
| w    | 604800 seconds   |
| M    | 1 month          |
| Y    | 1 year           |

Months and years use calendar arithmetic. Days are clamped to valid
range for the target month (e.g., Jan 31 + 1M = Feb 28).

## Examples

Print current epoch:

    et

Subtract 7 days:

    et -7d

Add 3 hours:

    et +3h

Add 1 month:

    et +1M

Subtract 1 year:

    et -1Y

Add 1 hour to a specific epoch:

    et 1704912345 +1h

Convert ISO-8601 to epoch:

    et parse 2026-01-05T12:00:00Z

Convert epoch to ISO-8601:

    et format 1704912345

Apply offset to timestamps from stdin:

    cat timestamps.txt | et -1d

## License

MIT
