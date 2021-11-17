{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nmattia/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, fenix, naersk, utils, nixpkgs }:
    utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in rec {
        packages.bloodbath = (naersk.lib.${system}.override {
          inherit (fenix.packages.${system}.stable) cargo rustc;
        }).buildPackage {
          name = "bloodbath";
          src = ./.;
          nativeBuildInputs = with pkgs;
            [ openssl pkgconfig ] ++ nixpkgs.lib.optional pkgs.stdenv.isDarwin [
              # needed by curl-sys
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];
        };

        defaultPackage = packages.bloodbath;
      }) // {
        nixosModule = { config, lib, pkgs, ... }:
          with lib;
          let cfg = config.services.bloodbath;
          in {
            options.services.bloodbath = {
              enable = mkEnableOption "bloodbath";

              config = mkOption {
                type = types.str;
                default = "";
                example = "";
                description = "The configuration (written in TOML).";
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

              services.bloodbath = {
                serviceConfig.Type = "oneshot";
                after = [ "network-online.target" ];
                wantedBy = [ "network-online.target" ];
                script = "${self.defaultPackage.${pkgs.system}}/bin/bloodbath ${
                    pkgs.writeText "config.toml" cfg.config
                  }";
              };
            };
          };
      };
}
