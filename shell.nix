{ sources ? import ./nix/sources.nix , pkgs ? import sources.nixpkgs {} }:
pkgs.stdenv.mkDerivation {
  name = "rust-env";
  buildInputs = [
    pkgs.rustup pkgs.pkgconfig pkgs.glibc pkgs.zlib pkgs.openssl pkgs.libgit2
    pkgs.exa
    pkgs.ripgrep
    pkgs.watchexec
    pkgs.tokei
    pkgs.bat
    pkgs.fd
    pkgs.yarn
  ];

  shellHook = ''
    export NIX_ENFORCE_PURITY=0
    alias ls=exa
    alias find=fd
    cargo update 
    rustup component add clippy
    rustup component add rustfmt
    cargo install cargo-watch
    cargo install cargo-edit
    cargo install cargo-tarpaulin
    cargo install cargo-audit
    cargo install cargo-outdated
    cargo install cargo-release
    cargo install cargo-udeps
    cargo install cargo-geiger
    mkdir -p .nix-node
    export NODE_PATH=$PWD/.nix-node
    export NPM_CONFIG_PREFIX=$PWD/.nix-node
    export PATH=$NODE_PATH/bin:$PATH
    set -o vi
  '';

  # Set Environment Variables
  RUST_BACKTRACE = 1;
}
