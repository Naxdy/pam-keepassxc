# pam-keepassxc

pam-keepassxc is a PAM module that can automatically unlock one or more KeePassXC databases upon successful user
authentication. It works with password-only databases, but also those that require a password in combination with a
hardware key (e.g. YubiKey), provided the password **is the same as the one for your system user account in both
cases**.

## IMPORTANT: Read before continuing

This module unlocks your KeePassXC databases by communicating with KeePassXC's dbus interface
(`org.keepassxc.KeePassXC.MainWindow`) and providing it with your user account password. This means if a malicious
application on your machine has the necessary permissions to impersonate this service on the dbus, it may be able to
sniff your password.

This means for Flatpak or other sandboxed applications you don't trust, make sure they don't have permissions to provide
that dbus service.

> [!NOTE]
>
> - If you're running a malicious application outside of a sandbox, it can get your credentials in a myriad of
>   alternative ways, so this point doesn't really apply
> - You should never run suspicious applications, even in a sandbox, so ideally you won't have to heed this warning
>   anyway :)

## Installation

### Prerequisites

- Ensure all databases you want to unlock share the same password, which must be the same as your user account, i.e. the
  password you use to unlock your machine must be the same as the one you use to unlock your databases.

- You can use databases that require a hardware key, provided the above password requirement is fulfilled. You may need
  to manually unlock the database once in order for KeePassXC to register that it requires a hardware key.

- Your version of KeePassXC must be built with dbus support enabled. This should be the default on most distros, but
  certain maintainers may opt to compile with `-DQT_NO_DBUS=1`, in which case unlocking via dbus will not work.

### NixOS

pam-keepassxc provides a flake with a NixOS module. Including it in your configuration is as simple as the following
example:

```nix
{
  inputs = {
    pam-keepassxc.url = "https://flakehub.com/f/Naxdy/pam-keepassxc/*";

    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, pam-keepassxc }: {
    nixosConfigurations.mysystem = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./my-configuration.nix
        pam-keepassxc.nixosModules.default
        ({ ... }: {
          security.pam.keepassxc = {
            enable = true;
            # List of absolute paths to databases that you want to unlock
            databases = [
              "/home/alice/passwords.kbdx"
            ];
            # PAM services (as they appear in /etc/pam.d) that should trigger the unlock
            services = [
              "kde"
            ];
          };
        })
      ];
    };
  };
}
```

### Other distros / manual

1. Build the project using the Rust toolchain, as required for your distro:

   ```bash
   cargo build --release
   ```

2. Copy the resulting file from `./target/release/libpam_keepassxc.so` to your library directory under
   `security/pam_keepassxc.so` for your distro, e.g. the full path is probably something like
   `/usr/lib/security/pam_keepassxc.so`.

3. For any PAM service you want to work with pam-keepassxc, add an appropriate `auth` entry to its config residing in
   `/etc/pam.d`. For example, for `hyprlock`, you would add the following entry:

   ```
   # auth = unlock whenever hyprlock is dismissed
   # optional = failing to unlock will not block login
   # pam_keepassxc.so = module name, can also be an absolute path
   # /home/alice/passwords.kbdx = absolute path to the database that should be unlocked
   #    multiple space-separated databases are supported, provided they all share
   #    the same password. if the path itself contains spaces, it must be wrapped
   #    in brackets, e.g.: [/home/alice/my db.kbdx]
   auth optional pam_keepassxc.so /home/alice/passwords.kbdx
   ```

   **Note:** The entry must be listed _before_ one with the `sufficient` control value (if such an entry exists).
   Usually placing it before `pam_unix` is good.

4. You may add a `session` entry with the same syntax, except using `session` instead of `auth` (useful for certain
   display managers).
