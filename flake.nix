{
    description = "A system monitoring plugin for Zellij terminal multiplexer that displays real-time CPU, memory, and GPU usage";

    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
      };

    outputs = { self, nixpkgs }:
      let
        forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ];
      in {
          packages = forAllSystems (system:
          let pkgs = nixpkgs.legacyPackages.${system};
          in {
              default = pkgs.rustPlatform.buildRustPackage {
                  pname = "zellij-load";
                  version = "0.1.3";
                  src = self;
                  cargoLock.lockFile = ./Cargo.lock;
                  buildFeatures = [ "native" ];
                  cargoBuildFlags = [ "--bin" "zellij_system_monitor"];
                  nativeBuildInputs = [pkgs.pkg-config ];
                  buildInputs =  [ pkgs.libdrm ];
                  doCheck = false;
                };
            }
          );
        };
  }
