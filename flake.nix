{
  description = "Event-Extractor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        craneLib = crane.lib.${system};
        event-extractor = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;

          buildInputs = [
            # Add additional build inputs here
          ];
        };
      in
      {
        checks = {
          inherit event-extractor;
        };

        packages.default = event-extractor;

        apps.default = flake-utils.lib.mkApp {
          drv = event-extractor;
        };

        nixosModules.default = { config, lib, pkgs, ... }:
          with lib;
          let cfg = config.hochreiner.services.event-extractor;
          in {
            options.hochreiner.services.event-extractor = {
              enable = mkEnableOption "Enables the event-extractor service";
              config_path = mkOption {
                type = lib.types.str;
                description = "Sets the path of the event-extractor config file";
              };
              user = mkOption {
                type = lib.types.str;
                description = "Sets the user for the service";
              };
              group = mkOption {
                type = lib.types.str;
                description = "Sets the group for the service";
              };
            };

            config = mkIf cfg.enable {
              systemd.services."hochreiner.event-extractor" = {
                description = "event-extractor service";
                wantedBy = [ "multi-user.target" ];

                serviceConfig = let pkg = self.packages.${system}.default;
                in {
                  Type = "oneshot";
                  ExecStart = "${pkg}/bin/event-extractor --config ${cfg.config_path}";
                  User = cfg.user;
                  Group = cfg.group;
                };
              };
              systemd.timers."hochreiner.event-extractor" = {
                description = "timer for the event-extractor service";
                wantedBy = [ "multi-user.target" ];
                timerConfig = {
                  OnBootSec="30min";
                  OnUnitInactiveSec="30min";
                  Unit="hochreiner.event-extractor.service";
                };
              };
            };
          };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          # Extra inputs can be added here
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
          ];
        };
      }
    );
  
  nixConfig = {
    substituters = [
      "https://cache.nixos.org"
      "https://hannes-hochreiner.cachix.org"
    ];
    trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "hannes-hochreiner.cachix.org-1:+ljzSuDIM6I+FbA0mdBTSGHcKOcEZSECEtYIEcDA4Hg="
    ];
  };
}