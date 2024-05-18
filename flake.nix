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
        upload-image = pkgs.writeShellScriptBin "upload-image" ''
          set -eu
          OCI_ARCHIVE=$(nix build .#docker --no-link --print-out-paths)
          DOCKER_REPOSITORY="docker://nappairam/minstack-echoipserver"
          ${pkgs.skopeo}/bin/skopeo copy --dest-creds="nappairam:$DOCKERHUB_TOKEN" "docker-archive:$OCI_ARCHIVE" "$DOCKER_REPOSITORY"
        '';
      in
      {
        defaultPackage = bin;
        packages.docker = pkgs.dockerTools.buildImage {
            name = "echoipserver";
            tag = "latest";
            copyToRoot = [ bin ];
            config = {
              Entrypoint = [ "${bin}/bin/echoipserver" ];
            };
          };
        apps.upload-image = utils.lib.mkApp { drv = upload-image; };
        devShell = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt rustPackages.clippy ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      }
    );
}
