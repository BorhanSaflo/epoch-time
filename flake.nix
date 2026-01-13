{
  description = "A CLI tool to print and manipulate Unix epoch timestamps";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages = {
          epoch-time = pkgs.rustPlatform.buildRustPackage {
            pname = "epoch-time";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            meta = with pkgs.lib; {
              description = "A CLI tool to print and manipulate Unix epoch timestamps";
              homepage = "https://github.com/BorhanSaflo/epoch-time";
              license = licenses.mit;
              maintainers = [ ];
              mainProgram = "et";
            };
          };

          default = self.packages.${system}.epoch-time;
        };

        apps = {
          epoch-time = flake-utils.lib.mkApp {
            drv = self.packages.${system}.epoch-time;
            name = "et";
          };

          default = self.apps.${system}.epoch-time;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
          ];
        };
      }
    );
}

