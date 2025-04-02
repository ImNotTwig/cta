{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    crane,
    ...
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    };
    toolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;
    craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
    
    moldDevShell = craneLib.devShell.override {
      # For example, use the mold linker
      mkShell = pkgs.mkShell.override {
        stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;
      };
    };
    
  in {
    devShells.${system}.default = moldDevShell {
      RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
    };
    packages = {
      cta = craneLib.buildPackage {
        src = craneLib.cleanCargoSource ./.;
      };
    };
  };
}
