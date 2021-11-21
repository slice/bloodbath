{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, fenix, flake-utils, nixpkgs }:
    let
      packages = flake-utils.lib.eachDefaultSystem (system:
        let pkgs = nixpkgs.legacyPackages.${system};
        in rec {
          packages.bloodbath = (pkgs.makeRustPlatform {
            inherit (fenix.packages.${system}.stable) cargo rustc;
          }).buildRustPackage {
            pname = "bloodbath";
            version = (nixpkgs.lib.importTOML ./Cargo.toml).package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.openssl ]
              ++ nixpkgs.lib.optional pkgs.stdenv.isDarwin [
                # needed by curl-sys
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              ];
          };

          defaultPackage = packages.bloodbath;
        });
    in packages // {
      nixosModule = { config, lib, pkgs, ... }:
        with lib;
        let
          cfg = config.services.bloodbath;
          pkg = self.defaultPackage.${pkgs.system};
          tomlConfigPath = if cfg.configFile != null then
            cfg.configFile
          else
            ((pkgs.formats.toml { }).generate "config.toml" (cfg.config // {
              # systemd StateDirectory
              database_path = "/var/lib/bloodbath";
            }));
        in {
          options.services.bloodbath = {
            enable = mkEnableOption "bloodbath";

            config = mkOption {
              type = types.attrsOf types.anything;
              default = "";
              example = "";
              description = "The configuration.";
            };

            configFile = mkOption {
              type = types.path;
              default = null;
              description = ''
                A path to the a TOML configuration. Takes priority over the config option.
                Make sure to set `database_path` to `/var/lib/bloodbath`.
              '';
            };

            timer = mkOption {
              type = types.str;
              default = "*:0/3";
              example = "hourly";
              description =
                "How often to run bloodbath (uses systemd calendar event syntax).";
            };
          };

          config.systemd = mkIf cfg.enable {
            timers.bloodbath = {
              wantedBy = [ "timers.target" ];
              partOf = [ "bloodbath.service" ];
              timerConfig.OnCalendar = cfg.timer;
            };

            services.bloodbath = rec {
              serviceConfig = {
                Type = "oneshot";
                User = "bloodbath";
                Group = "bloodbath";
                DynamicUser = true;
                StateDirectory = "bloodbath";
              };
              after = [ "network-online.target" ];
              wantedBy = [ "network-online.target" ];
              script = "${pkg}/bin/bloodbath ${tomlConfigPath}";
            };
          };
        };
    };
}
