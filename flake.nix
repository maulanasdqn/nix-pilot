{
  description = "Nix Pilot - NixOS Management Web UI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common arguments for all builds
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          buildInputs = with pkgs; [
            openssl
            pkg-config
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
        };

        # Build dependencies only (for caching)
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the np-api binary
        np-api = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          cargoExtraArgs = "-p np-api";
        });

        # Build the np-ui WASM
        np-ui = craneLib.buildTrunkPackage (commonArgs // {
          inherit cargoArtifacts;
          cargoExtraArgs = "-p np-ui";
          trunkIndexPath = "np-ui/index.html";

          # Trunk needs wasm-bindgen-cli
          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [
            pkgs.trunk
            pkgs.wasm-bindgen-cli
          ];
        });

        # Combined package with both API and UI
        nix-pilot = pkgs.runCommand "nix-pilot" {
          buildInputs = [ np-api ];
        } ''
          mkdir -p $out/bin
          mkdir -p $out/share/nix-pilot/static

          # Copy the API binary
          cp ${np-api}/bin/np-api $out/bin/nix-pilot-api

          # Copy the UI static files
          cp -r ${np-ui}/* $out/share/nix-pilot/static/

          # Create a wrapper script
          cat > $out/bin/nix-pilot <<'EOF'
          #!/bin/sh
          export NP_STATIC_DIR="${placeholder "out"}/share/nix-pilot/static"
          exec "${placeholder "out"}/bin/nix-pilot-api" "$@"
          EOF
          chmod +x $out/bin/nix-pilot
        '';

      in
      {
        packages = {
          inherit np-api np-ui nix-pilot;
          default = nix-pilot;
        };

        apps = {
          default = flake-utils.lib.mkApp {
            drv = nix-pilot;
            exePath = "/bin/nix-pilot";
          };

          api = flake-utils.lib.mkApp {
            drv = np-api;
            exePath = "/bin/np-api";
          };
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from commonArgs
          inputsFrom = [ np-api ];

          # Additional dev tools
          packages = with pkgs; [
            # Rust tools
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-edit
            cargo-audit

            # WASM tools
            trunk
            wasm-bindgen-cli
            wasm-pack

            # Formatting
            nixpkgs-fmt
            taplo # TOML formatter

            # Other tools
            just
            jq
          ];

          # Environment variables
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            echo "nix-pilot development shell"
            echo "Run 'cargo build' to build the project"
            echo "Run 'trunk serve --open' in np-ui/ to start the UI dev server"
          '';
        };

        # Checks (run with `nix flake check`)
        checks = {
          # Build checks
          inherit np-api;

          # Clippy
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          # Formatting
          fmt = craneLib.cargoFmt {
            src = commonArgs.src;
          };

          # Tests
          tests = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
          });
        };
      }
    ) // {
      # NixOS module
      nixosModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.services.nix-pilot;
        in
        {
          options.services.nix-pilot = {
            enable = lib.mkEnableOption "Nix Pilot web UI";

            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.system}.default;
              description = "The nix-pilot package to use";
            };

            port = lib.mkOption {
              type = lib.types.port;
              default = 3000;
              description = "Port to listen on";
            };

            address = lib.mkOption {
              type = lib.types.str;
              default = "127.0.0.1";
              description = "Address to bind to";
            };

            dataDir = lib.mkOption {
              type = lib.types.path;
              default = "/var/lib/nix-pilot";
              description = "Directory for nix-pilot data";
            };

            user = lib.mkOption {
              type = lib.types.str;
              default = "nix-pilot";
              description = "User to run nix-pilot as";
            };

            group = lib.mkOption {
              type = lib.types.str;
              default = "nix-pilot";
              description = "Group to run nix-pilot as";
            };

            openFirewall = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = "Whether to open the firewall for nix-pilot";
            };
          };

          config = lib.mkIf cfg.enable {
            users.users.${cfg.user} = {
              isSystemUser = true;
              group = cfg.group;
              home = cfg.dataDir;
              createHome = true;
            };

            users.groups.${cfg.group} = {};

            systemd.services.nix-pilot = {
              description = "Nix Pilot Web UI";
              after = [ "network.target" ];
              wantedBy = [ "multi-user.target" ];

              environment = {
                NP_PORT = toString cfg.port;
                NP_ADDRESS = cfg.address;
                NP_DATA_DIR = cfg.dataDir;
              };

              serviceConfig = {
                Type = "simple";
                User = cfg.user;
                Group = cfg.group;
                ExecStart = "${cfg.package}/bin/nix-pilot";
                Restart = "on-failure";
                RestartSec = "5s";

                # Hardening
                NoNewPrivileges = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                PrivateTmp = true;
                ReadWritePaths = [ cfg.dataDir ];
              };
            };

            networking.firewall.allowedTCPPorts =
              lib.mkIf cfg.openFirewall [ cfg.port ];
          };
        };
    };
}
