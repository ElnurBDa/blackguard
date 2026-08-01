{ pkgs ? import <nixpkgs> { } }:

# Dev shell for blackguard.
#
#   nix-shell            # drops you into a shell with the Rust toolchain
#   nix-shell --run cmd  # run a single command inside it
#
# No Rust is installed on the host; everything comes from Nix. All crates are
# pure-Rust (no C system deps), which keeps static-musl release builds clean.
pkgs.mkShell {
  name = "blackguard-dev";

  packages = with pkgs; [
    rustc
    cargo
    clippy
    rustfmt
    rust-analyzer
  ];

  RUST_BACKTRACE = "1";

  shellHook = ''
    echo "blackguard dev shell — $(rustc --version)"
  '';
}
