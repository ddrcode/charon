{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  packages = with pkgs; [
    cargo
    clippy
    rust-analyzer
    rustc
    rustfmt
    treefmt
  ];

  # inputsFrom = [ pkgs.hello pkgs.gnutar ];

  shellHook = ''
    export DEBUG=1
  '';
}
