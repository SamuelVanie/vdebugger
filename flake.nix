{
  description = "A nix file for my homemade debugger";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "vdebugger";
          version = "0.1.0";

          src = ./.;

          buildInputs = [ pkgs.rustc pkgs.cargo ];

          buildPhase = ''
            cargo build --release
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/vdebugger $out/bin/
          '';

          meta = with pkgs.lib; {
            description = "A Rust debugger";
            license = licenses.mit;
            maintainers = [ maintainers.SamuelVanie ];
          };
        };

        devShell = pkgs.mkShell {
          buildInputs = [ pkgs.rustc pkgs.cargo ];
        };
      });
}
