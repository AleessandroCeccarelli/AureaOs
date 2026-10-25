# AureaOS Roma 1.0 — Debian Package Manifest

## Purpose and status

These files are the proposed, version-controlled package selection for AureaOS Roma 1.0. They are inputs for a future build implementation; they do not install packages. The lists have not yet been integrated into `tools/build/bootstrap-debian.sh` and must not be described as a finished or fully dependency-validated system profile.

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

## Debian repository components and licensing

The base configuration currently uses only `main` for the debootstrap bootstrap stage. Several proposed packages require additional Debian archive components:

- `non-free-firmware`: firmware and CPU microcode packages in `firmware.list`
- `non-free`: `7zip-rar` in `rar-extraction-optional.list`

The final APT source configuration must explicitly enable the required components for both the main archive and Debian security updates. Enabling a component is a project distribution decision, not an automatic consequence of listing a package.

AureaOS uses a pragmatic selection policy. Components are evaluated for hardware compatibility, reliability, security, performance, usability, maintenance, licensing and redistribution terms. Open-source-only selection is not a project requirement. Proprietary components may be used when they materially improve the user experience and their terms permit the intended distribution. The GPL-3.0 license for AureaOS's original project code does not relicense third-party packages; each component retains its own license and terms.

## Important limitations

- Package names and dependencies must be checked against the target Debian 13 repositories before the lists are consumed by automation.
- Package lists do not configure services, firewall rules, permissions, DNS behavior, firmware loading, or automatic updates.
- `btrfs-progs` does not make the installed root filesystem Btrfs. Partitioning, subvolumes, mount configuration, initramfs and recovery behavior remain to be designed.
- `linux-image-amd64` and `initramfs-tools` do not by themselves make the root filesystem bootable. A matching bootloader, partition layout, configuration, kernel command line and image-generation process are required.
- RAR support is optional. `7zip-rar` is in `non-free` and provides RAR extraction through 7-Zip; it does not provide RAR archive creation. Review its license and redistribution terms before including it.
- Firmware packages improve compatibility for supported devices but cannot guarantee that every Wi-Fi, Bluetooth, graphics or other device will work. Hardware testing is required.
- No desktop environment, graphical settings application, graphical firewall manager, graphical archive manager, installer, or AureaOS-specific application is included in this terminal-only package profile.

## Next steps

1. Verify package availability, dependencies and archive components against Debian 13 Trixie.
2. Decide whether optional RAR support and the broad firmware set belong in the default image.
3. Define APT sources and security-update policy.
4. Design the disk layout, Btrfs subvolumes, boot mode and recovery model.
5. Integrate these lists into a build script only after the choices above are reviewed.
6. Test the generated image in a virtual machine and on representative hardware.
