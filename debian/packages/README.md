# AureaOS Roma 1.0 — Debian Package Manifest

## Purpose and status

These files are the version-controlled package selection for the initial AureaOS Roma 1.0 terminal-only profile. `tools/build/bootstrap-debian.sh` reads these lists and installs the common manifests into the target root filesystem. The profile is integrated into the build script, but package resolution and a complete rootfs build still need to be tested against the live Debian Trixie repositories before calling it validated.

## Target

- Base: Debian GNU/Linux 13 (Trixie), Stable
- Architecture: `amd64` (x86-64)
- Initial profile: terminal-only, with no graphical desktop or desktop applications
- Intended audience: general-purpose users who should not need a terminal for ordinary tasks once the desktop layer exists

## Package groups

- `base.list`: essential command-line utilities and service infrastructure
- `networking.list`: wired and Wi-Fi networking
- `bluetooth.list`: Bluetooth service and command-line tools
- `security.list`: firewall and authorization infrastructure; policies still need implementation
- `storage.list`: filesystem and disk utilities; does not configure the partition layout or root filesystem
- `archives.list`: common archive and compression formats
- `rar-extraction-optional.list`: optional RAR extraction support through 7-Zip
- `firmware.list`: candidate firmware and CPU microcode packages for broad hardware compatibility
- `boot.list`: kernel and initramfs packages shared by the target image
- `boot-bios.list`: GRUB for legacy BIOS/CSM images only
- `boot-uefi.list`: GRUB for x86-64 UEFI images only

The BIOS and UEFI bootloader lists are alternatives, not a single list to install wholesale. The final image target and secure-boot requirements must be decided before either list is integrated into the build.

## Debian repository components

The bootstrap and target APT configuration enable all four official Debian archive components: `main`, `contrib`, `non-free` and `non-free-firmware`. The generated deb822 source file covers `trixie`, `trixie-updates` and `trixie-security`. This makes packages from those components available to APT; it does not mean every package in every component is installed.

The project is not limited to free-software-only packages. The original AureaOS code remains licensed under GPL-3.0, while packages installed from Debian retain their own upstream packaging and license metadata.

## Important limitations

- The script now consumes the package lists, but a complete build must still confirm that every package resolves and installs successfully against the live Debian 13 repositories.
- Package lists do not configure services, firewall rules, permissions, DNS behavior, firmware loading, or automatic updates.
- `btrfs-progs` does not make the installed root filesystem Btrfs. Partitioning, subvolumes, mount configuration, initramfs and recovery behavior remain to be designed.
- `linux-image-amd64` and `initramfs-tools` do not by themselves make the root filesystem bootable. A matching bootloader, partition layout, configuration, kernel command line and image-generation process are required.
- RAR support is optional. Use `--with-rar` to install `7zip-rar` from `non-free`; it provides RAR extraction through 7-Zip, not RAR archive creation.
- Firmware packages improve compatibility for supported devices but cannot guarantee that every Wi-Fi, Bluetooth, graphics or other device will work. Hardware testing is required.
- No desktop environment, graphical settings application, graphical firewall manager, graphical archive manager, installer, or AureaOS-specific application is included in this terminal-only package profile.

## Build options

- Default: install the common manifests, including kernel/initramfs, with no GRUB package selected.
- `--boot-mode bios`: add the legacy BIOS GRUB package.
- `--boot-mode uefi`: add the x86-64 UEFI GRUB package.
- `--with-rar`: add optional RAR extraction support.

The BIOS and UEFI options are mutually exclusive. Neither option alone creates a bootable image.

## Next steps

1. Run the updated script against a fresh test destination and resolve any package/dependency errors.
2. Review installed package size and trim or retain the broad firmware set based on hardware coverage.
3. Design the disk layout, Btrfs subvolumes, boot mode and recovery model.
4. Add explicit service defaults, network/firewall policy and user-account setup.
5. Build and test a bootable image in a virtual machine before testing representative hardware.
