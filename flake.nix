{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        bin = naersk-lib.buildPackage {
          pname = "echoipserver";
          src = ./.;
        };
      in
      {
        defaultPackage = bin;
        packages.docker = pkgs.dockerTools.buildImage {
            name = "echoipserver";
            tag = "latest";
            copyToRoot = [ bin ];
            config = {
              Cmd = [ "${bin}/bin/echoipserver" ];
            };
          };
        devShell = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt rustPackages.clippy ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      }
    );
}
