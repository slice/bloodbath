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
        defaultPackage = (naersk.lib.${system}.override {
          inherit (fenix.packages.${system}.minimal) cargo rustc;
        }).buildPackage {
          name = "bloodbath";
          src = ./.;
          nativeBuildInputs = nixpkgs.lib.optional pkgs.stdenv.isDarwin [
            # needed by curl-sys
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];
        };

        nixosModule = { config, lib, pkgs }:
          with lib; {
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
                serviceConfig.type = "oneshot";
                after = [ "network-online.target" ];
                wantedBy = [ "network-online.target" ];
                script = "${defaultPackage}/bin/bloodbath ${
                    pkgs.writeText "config.toml" cfg.config
                  }";
              };
            };
          };
      });
}
