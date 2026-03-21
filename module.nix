{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.security.pam.keepassxc;
in
{
  options.security.pam.keepassxc = {
    enable = lib.mkEnableOption "KeePassXC PAM module";

    package = lib.mkPackageOption pkgs "pam-keepassxc" { };

    databases = lib.mkOption {
      type = lib.types.listOf lib.types.path;
      description = "A list of paths to database files to be opened by pam-keepassxc";
      default = [ ];
      example = [
        "/home/alice/passwords.kbdx"
      ];
    };

    services = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      description = "List of PAM service names to enable this module for";
      default = [ ];
      example = [
        "hyprlock"
        "kde"
      ];
    };
  };

  config = lib.mkIf cfg.enable {
    security.pam.services = builtins.listToAttrs (
      map (e: {
        name = e;
        value = {
          rules = {
            session = {
              keepassxc = {
                enable = true;
                modulePath = "${cfg.package}/lib/security/pam_keepassxc.so";
                order = config.security.pam.services.${e}.rules.session.unix.order - 10;
                args = cfg.databases;
                control = "optional";
              };
            };
            auth = {
              keepassxc = {
                enable = true;
                modulePath = "${cfg.package}/lib/security/pam_keepassxc.so";
                order = config.security.pam.services.${e}.rules.auth.unix.order - 10;
                args = cfg.databases;
                control = "optional";
              };
            };
          };
        };
      }) cfg.services
    );
  };
}
