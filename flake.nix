{
  description = "Charon - USB keyboard pass-through device";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Rust toolchain with cross-compilation target for RP5
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          targets = [ "aarch64-unknown-linux-gnu" ];
        };

        # Common packages for all platforms
        commonPackages = with pkgs; [
          rustToolchain
          bacon
          cargo-machete
          treefmt
          just
          pkg-config
          openssl
          perl  # needed for vendored openssl build
        ];

        # Linux-specific: cross-compilation toolchain for RP5
        linuxCrossPackages = with pkgs; [
          pkgsCross.aarch64-multiplatform.stdenv.cc
        ];

        # Darwin-specific: use Zig as cross-linker (no Docker needed)
        darwinPackages = with pkgs; [
          zig
          cargo-zigbuild
        ];

      in {
        devShells.default = pkgs.mkShell {
          packages = commonPackages
            ++ pkgs.lib.optionals pkgs.stdenv.isLinux linuxCrossPackages
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin darwinPackages;

          shellHook = ''
            export DEBUG=1

            # Zig cache directories (avoid Nix store conflicts)
            export ZIG_LOCAL_CACHE_DIR="$HOME/.cache/zig"
            export ZIG_GLOBAL_CACHE_DIR="$HOME/.cache/zig-global"

            # For Linux cross-compilation to RP5
            ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
              export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="${pkgs.pkgsCross.aarch64-multiplatform.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc"
            ''}

            echo "Charon development environment"
            echo "  Local target: ${system}"
            echo "  Cross target: aarch64-unknown-linux-gnu (RP5)"
            ${pkgs.lib.optionalString pkgs.stdenv.isDarwin ''
              echo "  Cross-compile: cargo zigbuild --target aarch64-unknown-linux-gnu --release"
            ''}
          '';

          # For openssl-sys
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        };
      }
    );
}
