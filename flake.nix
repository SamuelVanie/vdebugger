{
  description = "A nix file for my homemade debugger";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = pkgs.rust-bin.stable.latest.default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        vdebuggerBuild = rustPlatform.buildRustPackage {
          pname = "vdebugger";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };

      in
      {
        defaultPackage = vdebuggerBuild;

        devShell = pkgs.mkShell {
          buildInputs = [ (rustVersion.override { extensions = [ "rust-src" ]; }) pkgs.dwarfdump ];
        };

      });
}
