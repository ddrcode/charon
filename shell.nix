# Compatibility shim for `nix-shell`
# Prefer using `nix develop` with the flake instead
(import (
  fetchTarball {
    url = "https://github.com/edolstra/flake-compat/archive/refs/tags/v1.1.0.tar.gz";
    sha256 = "sha256:1vp9gv5rqdjqdvpzcs0jb7spx02dxdbvjravy3dajs5lhhcih3cb";
  }
) { src = ./.; }).shellNix
