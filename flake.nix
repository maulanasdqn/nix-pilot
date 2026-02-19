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
    sops-nix = {
      url = "github:Mic92/sops-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane, sops-nix }:
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

        # Source filter including Rust sources, templates, and frontend assets
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            let
              baseName = builtins.baseNameOf path;
              isWebAsset = builtins.match ".*\\.(html|css|js|ico|svg|png|jpg|wasm)$" path != null;
              isTrunkConfig = baseName == "Trunk.toml";
              isStylesDir = builtins.match ".*styles.*" path != null;
              isTemplatesDir = builtins.match ".*templates.*" path != null;
            in
            (craneLib.filterCargoSources path type) ||
            isWebAsset ||
            isTrunkConfig ||
            isStylesDir ||
            isTemplatesDir;
        };

        # Common arguments for all builds
        commonArgs = {
          inherit src;
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

        devShells.default = pkgs.mkShell {
          # Build inputs
          buildInputs = with pkgs; [
            # Rust toolchain with WASM target
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-edit

            # Build dependencies
            openssl
            pkg-config

            # WASM tools
            trunk
            wasm-bindgen-cli

            # Formatting
            nixpkgs-fmt
            taplo

            # SOPS/Age for secret management
            sops
            age

            # Other tools
            just
            jq
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.libiconv
          ];

          # Environment variables
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";

          shellHook = ''
            echo ""
            echo "  Nix Pilot Development Shell"
            echo "  ==========================="
            echo ""
            echo "  Commands:"
            echo "    just dev     - Run API + UI (recommended)"
            echo "    just api     - Run only API server"
            echo "    just ui      - Run only UI server"
            echo "    just build   - Build all packages"
            echo "    just test    - Run tests"
            echo "    just check   - Run all checks"
            echo ""
            echo "  URLs (when running):"
            echo "    API:  http://localhost:3000"
            echo "    UI:   http://localhost:8080"
            echo ""
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
          imports = [ sops-nix.nixosModules.sops ];

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

            secrets = {
              enable = lib.mkEnableOption "SOPS secret management for nix-pilot";

              ageKeyFile = lib.mkOption {
                type = lib.types.nullOr lib.types.path;
                default = null;
                description = "Path to the age key file for decrypting secrets";
                example = "/var/lib/nix-pilot/age-key.txt";
              };

              sopsFile = lib.mkOption {
                type = lib.types.nullOr lib.types.path;
                default = null;
                description = "Path to the SOPS encrypted secrets file";
              };

              apiKey = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "Name of the API key secret in the SOPS file";
              };

              extraSecrets = lib.mkOption {
                type = lib.types.attrsOf (lib.types.submodule {
                  options = {
                    key = lib.mkOption {
                      type = lib.types.str;
                      description = "Key path in the SOPS file";
                    };
                    owner = lib.mkOption {
                      type = lib.types.str;
                      default = cfg.user;
                      description = "Owner of the secret file";
                    };
                    group = lib.mkOption {
                      type = lib.types.str;
                      default = cfg.group;
                      description = "Group of the secret file";
                    };
                    mode = lib.mkOption {
                      type = lib.types.str;
                      default = "0400";
                      description = "File permissions";
                    };
                  };
                });
                default = {};
                description = "Additional secrets to provision";
              };
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

            # SOPS configuration
            sops = lib.mkIf cfg.secrets.enable {
              defaultSopsFile = lib.mkIf (cfg.secrets.sopsFile != null) cfg.secrets.sopsFile;
              age.keyFile = lib.mkIf (cfg.secrets.ageKeyFile != null) cfg.secrets.ageKeyFile;

              secrets = lib.mkMerge [
                # API key secret
                (lib.mkIf (cfg.secrets.apiKey != null) {
                  "nix-pilot-api-key" = {
                    key = cfg.secrets.apiKey;
                    owner = cfg.user;
                    group = cfg.group;
                    mode = "0400";
                    restartUnits = [ "nix-pilot.service" ];
                  };
                })
                # Extra secrets
                (lib.mapAttrs (name: secret: {
                  inherit (secret) key owner group mode;
                  restartUnits = [ "nix-pilot.service" ];
                }) cfg.secrets.extraSecrets)
              ];
            };

            systemd.services.nix-pilot = {
              description = "Nix Pilot Web UI";
              after = [ "network.target" ] ++ lib.optionals cfg.secrets.enable [ "sops-nix.service" ];
              wantedBy = [ "multi-user.target" ];

              environment = {
                NP_PORT = toString cfg.port;
                NP_ADDRESS = cfg.address;
                NP_DATA_DIR = cfg.dataDir;
                NP_SECRETS_DIR = "${cfg.dataDir}/secrets";
                NP_AGE_KEY_PATH = lib.mkIf (cfg.secrets.enable && cfg.secrets.ageKeyFile != null)
                  cfg.secrets.ageKeyFile;
              } // lib.optionalAttrs (cfg.secrets.enable && cfg.secrets.apiKey != null) {
                NP_API_KEY_FILE = config.sops.secrets."nix-pilot-api-key".path;
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
