

<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Debian Base

## Base distribution

- Distribution: Debian GNU/Linux 13 (Trixie)
- Release channel: Stable
- Target architecture: `amd64` (x86-64)
- Default installed filesystem: Btrfs (selected; not configured by the current bootstrap)
- Initial root filesystem tool: `debootstrap`
- Bootstrap mirror: `https://deb.debian.org/debian`
- Security mirror: `https://security.debian.org/debian-security`
- Initial debootstrap profile: `minbase`
- Bootstrap and target APT components: `main`, `contrib`, `non-free`, `non-free-firmware`

## Initial system profile

The first build stage creates a minimal, terminal-only Debian root filesystem. It is a development artifact, not a complete AureaOS installation image. No graphical desktop or desktop applications are included in this stage.

The package selection is maintained under `debian/packages/`. The bootstrap script reads the common manifests and installs their packages into the target root filesystem. Optional manifests add RAR extraction support or a selected BIOS/UEFI bootloader.

The profile is integrated into the build script, but package resolution and a complete rootfs build still need to be tested against the live Debian Trixie repositories before the profile can be considered fully validated.

A successful script syntax check or help invocation does not prove that every package resolves and installs correctly.

## Build implementation

The bootstrap script is:

`tools/build/bootstrap-debian.sh`

Default destination:

`~/Documenti/AureaOS-Build/rootfs`

A different absolute path can be supplied with `--target`.

Generated root filesystems are kept outside the Git repository. The script resolves the destination path and refuses to overwrite an existing file, directory, or symbolic link. It also refuses destinations inside the repository.

The script requires `debootstrap` on the host system. It does not install packages on the host.

The script runs `debootstrap` with:

- Suite: `trixie`
- Architecture: `amd64`
- Variant: `minbase`
- Components: `main`, `contrib`, `non-free`, `non-free-firmware`
- Mirror: `https://deb.debian.org/debian`

After creating the base root filesystem, it writes `/etc/apt/sources.list.d/debian.sources` using the deb822 format.

The configured suites are:

- `trixie`
- `trixie-updates`
- `trixie-security`

The script preserves an existing legacy `/etc/apt/sources.list` as `sources.list.bootstrap-backup` rather than silently replacing it. If that backup already exists, the script stops.

### Chroot preparation

Before installing packages, the script mounts the required virtual filesystems inside the target root filesystem:

- `/dev`
- `/proc`
- `/sys`

These mounts allow package maintainer scripts and system-related package configuration to operate in the chroot.

A temporary `policy-rc.d` prevents services from being started automatically during package installation.

The script removes this temporary policy and attempts to unmount the virtual filesystems during cleanup, including when an error interrupts the build.

If the bootstrap fails, the script reports that the destination may contain an incomplete root filesystem. It deliberately does not delete that destination automatically.

### Package installation options

The script reads the common manifests:

- `base.list`
- `networking.list`
- `bluetooth.list`
- `security.list`
- `storage.list`
- `archives.list`
- `firmware.list`
- `boot.list`

The optional `--with-rar` flag adds `rar-extraction-optional.list`.

The `--boot-mode` option accepts:

- `none` — default; no GRUB package is selected.
- `bios` — adds the legacy BIOS GRUB manifest.
- `uefi` — adds the x86-64 UEFI GRUB manifest.

The BIOS and UEFI manifests are alternatives and are not installed together by default.

These options select packages only. They do not configure a complete bootable system.

### Final validation

After package installation, the script verifies:

- The presence of `/etc/os-release`.
- The Debian codename is `trixie`.
- The target architecture is `amd64`.
- The generated APT sources contain all four expected Debian components.
- The fundamental `systemd` and `linux-image-amd64` packages are installed.

These checks validate important properties of the generated root filesystem. They do not establish that the system can boot, install successfully on hardware, or operate as a complete AureaOS desktop.

## Package manifest

`debian/packages/README.md` documents the proposed package groups and their limitations.

The package lists separate:

- The terminal-only base system.
- Networking and Bluetooth.
- Security and authorization infrastructure.
- Storage and filesystem utilities.
- Archive and compression support.
- Firmware and CPU microcode.
- Kernel and initramfs packages.
- Alternative BIOS and UEFI bootloader packages.

All four Debian components are enabled consistently for the main archive and security updates.

This makes packages from those components available to APT; it does not automatically install every package from those components.

The project is not limited to free-software-only packages. Each third-party package retains its own license and distribution conditions.

## Build objectives and status

- [x] Select Debian 13 Trixie and `amd64` as the initial target.
- [x] Select Btrfs as the default installed filesystem.
- [x] Select `debootstrap` and create a minimal root filesystem outside the repository.
- [x] Add a bootstrap script with destination safety checks.
- [x] Validate the script syntax and its help behavior.
- [x] Test refusal to overwrite an existing destination.
- [x] Build and verify a second minimal root filesystem using the script.
- [x] Add an initial package-manifest proposal under `debian/packages/`.
- [x] Enable all four Debian archive components for bootstrap and configure Trixie, updates, and security APT sources.
- [x] Integrate the common package manifests into the bootstrap script with optional RAR and BIOS/UEFI GRUB selections.
- [x] Add chroot preparation and cleanup for `/dev`, `/proc`, and `/sys`.
- [x] Add final checks for Debian codename, target architecture, APT components, and fundamental installed packages.
- [ ] Run a complete fresh build and validate package resolution and installation against live Trixie repositories.
- [ ] Review the installed package set and its total size.
- [ ] Define system accounts, hostname, locale, timezone, and service defaults.
- [ ] Define the partition scheme and Btrfs subvolume/mount layout.
- [ ] Configure firewall defaults and test networking, DNS, and Bluetooth behavior.
- [ ] Select the release image's boot mode and configure the appropriate bootloader.
- [ ] Produce and test a bootable x86-64 image.
- [ ] Test installation, updates, recovery, and representative hardware.
- [ ] Document supported hardware and known limitations.

## Filesystem, boot and recovery

The default installed filesystem is Btrfs. The current bootstrap installs filesystem tools but does not format or configure the final system filesystem.

Partitioning, subvolumes, mounts, initramfs configuration, and boot configuration still need to be designed and tested.

AureaOS will not use Btrfs's native snapshot or rollback workflow for backup and restore. Aurea Time is the planned system for restoring data from an individual file through the full system.

Its backup format, USB-disk and network-storage compatibility, and recovery process remain to be designed and validated.

The kernel and initramfs packages do not make a root filesystem bootable by themselves. The image build must select a target boot mode and install and configure the matching bootloader.

`boot-bios.list` and `boot-uefi.list` are alternatives, not lists to install together by default.

## Update policy

The intended policy separates three update domains:

1. Debian base packages and security updates from explicitly configured Debian repositories.
2. AureaOS-specific system components from a project-managed release/update mechanism, which remains to be designed.
3. Third-party applications delivered through Flatpak/Flathub, updated through their own distribution mechanisms.

This separation is an architectural intention, not an implemented update system.

Compatibility, rollback, signing, repository trust, and failure recovery must be designed before release.

## Reproducibility and validation

The bootstrap procedure has been executed successfully, but this does not establish bit-for-bit reproducibility.

Repository package versions can change over time. A reproducible release process will need documented package snapshots or version constraints, build-environment control, artifact checksums, and automated validation.

The current root filesystem is not a bootable AureaOS image, and no final installation ISO has been produced.

