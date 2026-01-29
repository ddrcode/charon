{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  packages = with pkgs; [
    bacon
    cargo
    clippy
    rust-analyzer
    rustc
    rustfmt
    treefmt
    openssl
    pkg-config
    cargo-machete
    nodejs_20
    just
  ];

  # inputsFrom = [ pkgs.hello pkgs.gnutar ];

  shellHook = ''
    export DEBUG=1
  '';
}
