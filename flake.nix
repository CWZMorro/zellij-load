{
  description = "A system monitoring plugin for Zellij terminal multiplexer that displays real-time CPU, memory, and GPU usage";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        daemon = pkgs.rustPlatform.buildRustPackage {
          pname = "zellij-system-monitor";
          version = "0.1.3";
          src = pkgs.lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildFeatures = [ "native" ];
          cargoBuildFlags = [
            "--bin"
            "zellij_system_monitor"
          ];
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.libdrm ];
          doCheck = false; # skip cargo test
        };

        plugin = pkgs.pkgsCross.wasi32.callPackage (
          { rustPlatform, lld }:
          rustPlatform.buildRustPackage {
            pname = "zellij-load-plugin";
            version = "0.1.3";
            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;
            buildFeatures = [ "plugin" ];
            cargoBuildFlags = [
              "--bin"
              "zellij-load-plugin"
            ];
            doCheck = false;

            env.RUSTFLAGS = "-C linker=wasm-ld";
            nativeBuildInputs = [ lld ];

            installPhase = ''
              mkdir -p $out/share/zellij/plugins
              cp target/wasm32-wasip1/release/zellij-load-plugin.wasm $out/share/zellij/plugins/zellij-load.wasm
            '';
          }
        ) { };

      in
      {
        packages = {
          inherit daemon plugin;

          default = pkgs.symlinkJoin {
            name = "zellij-load";
            paths = [
              daemon
              plugin
            ];
          };
        };

        # dev shell: `nix develop` provides cargo, rustc, just without global install
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            pkg-config
            libdrm
            just
          ];
        };
      }
    );
}
