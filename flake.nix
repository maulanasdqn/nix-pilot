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

        # API-only package (UI will be added later when trunk workspace build is fixed)
        nix-pilot = pkgs.runCommand "nix-pilot" {
          buildInputs = [ np-api ];
        } ''
          mkdir -p $out/bin
          mkdir -p $out/share/nix-pilot/static

          # Copy the API binary
          cp ${np-api}/bin/np-api $out/bin/nix-pilot-api

          # Create login page
          cat > $out/share/nix-pilot/static/index.html <<'HTMLEOF'
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Nix Pilot</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
      color: #fff;
    }
    .container { text-align: center; padding: 2rem; }
    .logo { font-size: 3rem; margin-bottom: 0.5rem; }
    h1 { font-size: 2rem; margin-bottom: 2rem; font-weight: 300; }
    .login-form {
      background: rgba(255,255,255,0.1);
      padding: 2rem;
      border-radius: 12px;
      backdrop-filter: blur(10px);
      max-width: 400px;
      margin: 0 auto;
    }
    .form-group { margin-bottom: 1rem; text-align: left; }
    label { display: block; margin-bottom: 0.5rem; font-size: 0.9rem; opacity: 0.8; }
    input {
      width: 100%;
      padding: 0.75rem 1rem;
      border: 1px solid rgba(255,255,255,0.2);
      border-radius: 8px;
      background: rgba(0,0,0,0.2);
      color: #fff;
      font-size: 1rem;
    }
    input:focus { outline: none; border-color: #4f8cff; }
    button {
      width: 100%;
      padding: 0.75rem 1rem;
      border: none;
      border-radius: 8px;
      background: #4f8cff;
      color: #fff;
      font-size: 1rem;
      cursor: pointer;
      margin-top: 1rem;
      transition: background 0.2s;
    }
    button:hover { background: #3d7be8; }
    button:disabled { background: #666; cursor: not-allowed; }
    .error { color: #ff6b6b; margin-top: 1rem; font-size: 0.9rem; }
    .dashboard { display: none; }
    .dashboard.active { display: block; }
    .login-form.hidden { display: none; }
    .card {
      background: rgba(255,255,255,0.1);
      padding: 1.5rem;
      border-radius: 12px;
      margin: 1rem 0;
    }
    .status { color: #4ade80; }
    .logout-btn {
      background: rgba(255,255,255,0.1);
      margin-top: 2rem;
    }
    .logout-btn:hover { background: rgba(255,255,255,0.2); }
  </style>
</head>
<body>
  <div class="container">
    <div class="logo">🚀</div>
    <h1>Nix Pilot</h1>

    <div id="loginForm" class="login-form">
      <div class="form-group">
        <label for="username">Username</label>
        <input type="text" id="username" placeholder="admin" autocomplete="username">
      </div>
      <div class="form-group">
        <label for="password">Password</label>
        <input type="password" id="password" placeholder="Password" autocomplete="current-password">
      </div>
      <button id="loginBtn" onclick="login()">Login</button>
      <div id="error" class="error"></div>
    </div>

    <div id="dashboard" class="dashboard">
      <div class="card">
        <h3>Status: <span class="status">Online</span></h3>
        <p style="margin-top: 0.5rem; opacity: 0.7;">API is running</p>
      </div>
      <div class="card">
        <h3>Quick Links</h3>
        <p style="margin-top: 0.5rem;">
          <a href="/api/health" style="color: #4f8cff;">Health Check</a> |
          <a href="/api/machines" style="color: #4f8cff;">Machines</a> |
          <a href="/api/flakes" style="color: #4f8cff;">Flakes</a>
        </p>
      </div>
      <p style="margin-top: 1rem; opacity: 0.6;">Full UI coming soon...</p>
      <button class="logout-btn" onclick="logout()">Logout</button>
    </div>
  </div>

  <script>
    const TOKEN_KEY = 'np_token';

    async function checkAuth() {
      const token = localStorage.getItem(TOKEN_KEY);
      if (!token) return false;
      try {
        const res = await fetch('/api/auth/check', {
          headers: { 'Authorization': 'Bearer ' + token }
        });
        const data = await res.json();
        return data.authenticated || !data.auth_enabled;
      } catch { return false; }
    }

    async function login() {
      const btn = document.getElementById('loginBtn');
      const error = document.getElementById('error');
      btn.disabled = true;
      error.textContent = '';

      try {
        const res = await fetch('/api/auth/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            username: document.getElementById('username').value,
            password: document.getElementById('password').value
          })
        });
        const data = await res.json();
        if (data.success && data.token) {
          localStorage.setItem(TOKEN_KEY, data.token);
          showDashboard();
        } else if (data.success && !data.token) {
          showDashboard(); // Auth disabled
        } else {
          error.textContent = data.message || 'Login failed';
        }
      } catch (e) {
        error.textContent = 'Connection error';
      }
      btn.disabled = false;
    }

    async function logout() {
      const token = localStorage.getItem(TOKEN_KEY);
      if (token) {
        await fetch('/api/auth/logout', {
          method: 'POST',
          headers: { 'Authorization': 'Bearer ' + token }
        });
      }
      localStorage.removeItem(TOKEN_KEY);
      showLogin();
    }

    function showDashboard() {
      document.getElementById('loginForm').classList.add('hidden');
      document.getElementById('dashboard').classList.add('active');
    }

    function showLogin() {
      document.getElementById('loginForm').classList.remove('hidden');
      document.getElementById('dashboard').classList.remove('active');
    }

    // Check auth on load
    checkAuth().then(ok => ok ? showDashboard() : showLogin());

    // Enter key to login
    document.getElementById('password').addEventListener('keyup', e => {
      if (e.key === 'Enter') login();
    });
  </script>
</body>
</html>
HTMLEOF

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
          inherit np-api nix-pilot;
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

            auth = {
              enable = lib.mkOption {
                type = lib.types.bool;
                default = true;
                description = "Enable authentication";
              };

              username = lib.mkOption {
                type = lib.types.str;
                default = "admin";
                description = "Admin username";
              };

              password = lib.mkOption {
                type = lib.types.str;
                default = "changeme";
                description = "Admin password (set via environment or secrets in production)";
              };
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
                NP_AUTH_ENABLED = if cfg.auth.enable then "true" else "false";
                NP_AUTH_USERNAME = cfg.auth.username;
                NP_AUTH_PASSWORD = cfg.auth.password;
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
