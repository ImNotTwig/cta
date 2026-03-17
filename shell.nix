{
  pkgs ?
    import <nixpkgs> {
      system = "x86_64-linux";
      overlays = [
        (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/2c7510a559416d07242621d036847152d970612b.tar.gz"))
      ];
    },
}: let
  toolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;
in
  pkgs.mkShell rec {
    RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
    packages = [
      toolchain
      pkgs.pkg-config
      pkgs.openssl
      pkgs.udev
      pkgs.lavalink
    ];
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
  }
