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

        # nixosModules.default = { config, lib, pkgs, ... }:
        #   with lib;
        #   let cfg = config.hochreiner.services.rusthello;
        #   in {
        #     options.hochreiner.services.rusthello = {
        #       enable = mkEnableOption "Enables the rust hello service";
        #     };

        #     config = mkIf cfg.enable {
        #       systemd.services."hochreiner.rusthello" = {
        #         description = "rust hello test service";
        #         wantedBy = [ "multi-user.target" ];

        #         serviceConfig = let pkg = self.packages.${system}.default;
        #         in {
        #           # Restart = "on-failure";
        #           Type = "oneshot";
        #           ExecStart = "${pkg}/bin/event-extractor";
        #           DynamicUser = "yes";
        #           RuntimeDirectory = "hochreiner.rusthello";
        #           RuntimeDirectoryMode = "0755";
        #           StateDirectory = "hochreiner.rusthello";
        #           StateDirectoryMode = "0700";
        #           CacheDirectory = "hochreiner.rusthello";
        #           CacheDirectoryMode = "0750";
        #         };
        #       };
        #       systemd.timers."hochreiner.rusthello" = {
        #         description = "timer for the rust hello test service";
        #         wantedBy = [ "multi-user.target" ];
        #         timerConfig = {
        #           OnBootSec="5min";
        #           OnUnitInactiveSec="5min";
        #           Unit="hochreiner.rusthello.service";
        #         };
        #       };
        #     };
        #   };

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