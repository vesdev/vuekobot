{ config, lib, pkgs, ... }: {
  options = with lib; {
    services.vueko-backend = {
      enable = mkEnableOption ''
        Backend for vueko chat bot
      '';

      package = mkOption {
        type = lib.types.package;
        default = pkgs.vueko-frontend;
      };

      configFile = mkOption {
        type = lib.types.path;
        default = pkgs.vuekobot;
      };
    };

    services.vueko-frontend = {
      enable = mkEnableOption ''
        Frontend for vueko chat bot
      '';

      package = mkOption {
        type = lib.types.package;
        default = pkgs.vueko-backend;
      };
    };
  };

  config = {
    systemd.services.vueko-backend =
      lib.mkIf config.services.vueko-backend.enable {
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" "postgresql.service" ];
        wants = [ "network-online.target" ];

        serviceConfig = {
          user = "vuekobot";
          group = "vuekobot";
          restart = "always";
          WorkingDirectory = "${config.services.vueko-backend.package}";
          ExecStart =
            "${config.services.vueko-backend.package}/bin/vuekobot ${config.services.vueko-backend.configFile}";
        };
      };

    systemd.services.vueko-frontend =
      lib.mkIf config.services.vueko-frontend.enable {
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" ];
        wants = [ "network-online.target" ];

        serviceConfig = {
          user = "vuekobot";
          group = "vuekobot";
          restart = "always";
          WorkingDirectory = "${config.services.vueko-frontend.package}";
          ExecStart =
            "${pkgs.bash}/bin/bash ${config.services.vueko-frontend.package}/bin/vueko-frontend";
        };
      };
  };

}
