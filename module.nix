{ config, lib, pkgs, ... }: {
  options = with lib; {
    services.vuekobot = {
      enable = mkEnableOption ''
        Some twitch chat bot
      '';

      package = mkOption {
        type = lib.types.package;
        default = pkgs.vuekobot;
      };

      configFile = mkOption {
        type = lib.types.path;
        default = pkgs.vuekobot;
      };
    };
  };

  config = lib.mkIf config.services.vuekobot.enable {
    systemd.services.vuekobot = {
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" "postgresql.service" ];
      wants = [ "network-online.target" ];

      serviceConfig = {
        user = "vuekobot";
        group = "vuekobot";
        restart = "always";
        WorkingDirectory = "${config.services.vuekobot.package}";
        ExecStart =
          "${config.services.vuekobot.package}/bin/vuekobot ${config.services.vuekobot.configFile}";
      };
    };
  };
}
