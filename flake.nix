{
  description = "Dice Roller - A comprehensive CLI dice roller supporting Roll20-style notation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      let
        mkDiceRoller = buildType: pkgs.rustPlatform.buildRustPackage {
          pname = "dice_roller";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl ];
          inherit buildType;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
          ];

          shellHook = ''
            echo "Dice Roller Development Environment"
            echo "Rust version: $(rustc --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build         - Build the project (debug)"
            echo "  cargo build --release - Build optimized release"
            echo "  cargo run           - Run the CLI (debug)"
            echo "  cargo test          - Run tests"
            echo "  cargo clippy        - Run linter"
            echo "  cargo fmt           - Format code"
            echo ""
            echo "Nix build targets:"
            echo "  nix build           - Release build (default)"
            echo "  nix build .#debug   - Debug build"
            echo "  nix build .#release - Release build (explicit)"
            echo ""
          '';

          RUST_BACKTRACE = "1";
        };

        packages = {
          # Default package is release build
          default = mkDiceRoller "release";

          # Explicit release build
          release = mkDiceRoller "release";

          # Debug build for development
          debug = mkDiceRoller "debug";
        };
      }
    );
}
