<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Debian Base

## Purpose

Debian provides the foundational operating system layer for AureaOS Roma 1.0.

AureaOS is not intended to be a repackaged Debian desktop. Debian supplies the underlying Linux system, packages, kernel, drivers, and system infrastructure. AureaOS develops and maintains its own desktop experience, system integration, visual identity, and first-party applications.

The Debian base and the AureaOS product layer have separate responsibilities and update lifecycles.

This document describes the selected Debian foundation, the current bootstrap implementation, and the work still required to produce a complete installable system.

## Base Distribution

- Distribution: Debian GNU/Linux 13 (Trixie).
- Release channel: Stable.
- Target architecture: `amd64` (x86-64).
- Default installed filesystem: Btrfs.
- Initial root filesystem tool: `debootstrap`.
- Bootstrap mirror: `https://deb.debian.org/debian`.
- Security mirror: `https://security.debian.org/debian-security`.
- Initial debootstrap profile: `minbase`.
- Bootstrap and target APT components: `main`, `contrib`, `non-free`, `non-free-firmware`.

Btrfs is the selected default filesystem for installed AureaOS systems. The current bootstrap does not format disks or configure the final filesystem layout.

## Initial System Profile

The current build stage creates a minimal, terminal-only Debian root filesystem.

This is a development artifact, not a complete AureaOS installation image.

The initial profile does not include:

- Aurea Desktop.
- A graphical desktop environment.
- Graphical system settings.
- AureaOS first-party desktop applications.
- An installer or a bootable installation image.

The package selection is maintained under `debian/packages/`.

The bootstrap script reads the common package manifests and installs their packages into the target root filesystem. Optional manifests provide RAR extraction support or a selected BIOS/UEFI GRUB package.

The profile is integrated into the build script. However, package resolution and a complete root filesystem build still need to be tested against the live Debian Trixie repositories before the profile can be considered fully validated.

A successful script syntax check, help invocation, or partial build does not prove that every package resolves and installs correctly.

## Build Implementation

The bootstrap script is:

`tools/build/bootstrap-debian.sh`

Default destination:

`~/Documenti/AureaOS-Build/rootfs`

A different absolute path can be supplied with `--target`.

Generated root filesystems are kept outside the Git repository.

The script:

- Resolves the destination path.
- Refuses to overwrite an existing file, directory, or symbolic link.
- Refuses destinations inside the repository.
- Does not install packages on the host system.
- Reports when a failed bootstrap may have left an incomplete root filesystem.
- Does not automatically delete an incomplete destination.

The script requires `debootstrap` on the host system.

### Debian Bootstrap Configuration

The script runs `debootstrap` with:

- Suite: `trixie`.
- Architecture: `amd64`.
- Variant: `minbase`.
- Components: `main`, `contrib`, `non-free`, `non-free-firmware`.
- Mirror: `https://deb.debian.org/debian`.

After creating the base root filesystem, it writes:

`/etc/apt/sources.list.d/debian.sources`

The file uses the deb822 format.

The configured suites are:

- `trixie`
- `trixie-updates`
- `trixie-security`

The security repository uses:

`https://security.debian.org/debian-security`

The script preserves an existing legacy `/etc/apt/sources.list` as `sources.list.bootstrap-backup` rather than silently replacing it.

If that backup already exists, the script stops.

### Chroot Preparation

Before installing packages, the script mounts the required virtual filesystems inside the target root filesystem:

- `/dev`
- `/proc`
- `/sys`

These mounts allow package maintainer scripts and system-related package configuration to operate in the chroot.

A temporary `policy-rc.d` prevents services from being started automatically during package installation.

The script removes this temporary policy and attempts to unmount the virtual filesystems during cleanup, including when an error interrupts the build.

If the bootstrap fails, the script reports that the destination may contain an incomplete root filesystem. It deliberately does not delete that destination automatically.

### Package Installation Options

The script reads the common manifests:

- `base.list`
- `networking.list`
- `bluetooth.list`
- `security.list`
- `storage.list`
- `archives.list`
- `firmware.list`
- `boot.list`

The optional `--with-rar` flag adds:

`rar-extraction-optional.list`

The `--boot-mode` option accepts:

- `none` — default; no GRUB package is selected.
- `bios` — adds the legacy BIOS GRUB manifest.
- `uefi` — adds the x86-64 UEFI GRUB manifest.

The BIOS and UEFI manifests are alternatives and are not installed together by default.

These options select packages only. They do not configure a complete bootable system.

### Final Validation

After package installation, the script performs checks for:

- The presence of `/etc/os-release`.
- The Debian codename being `trixie`.
- The target architecture being `amd64`.
- The expected Debian components in the generated APT sources.
- The package status output for the fundamental `systemd` and `linux-image-amd64` packages.

These checks validate important properties of the generated root filesystem. They do not establish that the system can boot, install successfully on hardware, or operate as a complete AureaOS desktop.

The package-status check is an initial validation step and must not be treated as a substitute for a complete package audit.

## Package Manifest

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

This makes packages from those components available to APT. It does not automatically install every package from those components.

The project is not limited to free-software-only packages. Each third-party package retains its own license and distribution conditions.

The AureaOS project license does not replace or override the licenses of Debian packages or other third-party software.

## AureaOS System Architecture

The Debian base is the lower-level foundation of the operating system.

AureaOS owns the product layer built on top of it.

The intended separation is:

### Debian Base

Provides:

- Linux kernel and hardware driver infrastructure.
- Systemd and system services.
- APT and Debian package management.
- Networking and Bluetooth infrastructure.
- Storage and filesystem utilities.
- Firmware and CPU microcode packages.
- Security and authorization foundations.

### AureaOS System Layer

Provides the AureaOS-specific desktop and system experience, including:

- Aurea Desktop and its compositor integration.
- AureaOS Human Interfaces.
- System settings and configuration.
- Aurea Store.
- Aurea Update.
- Aurea Time.
- Aurea Wallet.
- Other first-party system applications and services.

Rust is the primary language for AureaOS first-party system development where appropriate.

The desktop architecture is Wayland-first and uses Smithay for the compositor. Vulkan is the selected and exclusive graphics rendering API.

These choices belong to the AureaOS product architecture and do not change the responsibilities of the Debian base.

### Third-Party Applications

Aurea Store is intended to integrate Flathub as an application source.

Flatpak provides the installation, execution, sandboxing, and update mechanisms for third-party applications.

Third-party Flatpak applications remain separate from AureaOS first-party system components.

The use of Flatpak does not replace Debian package management for the underlying operating system.

## Filesystem, Boot and Recovery

### Filesystem

Btrfs is the selected default filesystem for installed AureaOS systems.

The current bootstrap installs filesystem tools but does not format or configure the final system filesystem.

The following decisions and implementation work remain necessary:

- Partition layout.
- EFI System Partition configuration.
- Root and user-data layout.
- Btrfs subvolume structure.
- Mount configuration.
- Initramfs integration.
- Installation and recovery behavior.

The final partitioning and filesystem configuration must be validated before an installable release can be produced.

### Backup and Restore

AureaOS will not use Btrfs native snapshots as its backup and restore workflow.

Aurea Time is the planned first-party system for backup and restore, with a scope extending from individual files to full-system recovery.

Its backup format, supported storage destinations, USB-disk and network-storage compatibility, and recovery process remain to be designed and validated.

### Boot Process

The kernel and initramfs packages do not make a root filesystem bootable by themselves.

The image build must select a target boot mode and install and configure the matching bootloader.

`boot-bios.list` and `boot-uefi.list` are alternatives, not lists to install together by default.

The final release boot configuration, including UEFI requirements and Secure Boot support, remains to be defined and tested.

## Update Policy

AureaOS separates updates into three independent domains.

### 1. Debian Base Updates

Debian packages and security updates are delivered through the configured Debian repositories.

APT remains responsible for the Debian base package lifecycle.

### 2. AureaOS System Updates

AureaOS-specific system components are updated through the planned Aurea Update mechanism.

This domain covers the AureaOS product layer and must remain distinct from Debian package updates.

The project-managed release and update mechanism, signing, compatibility rules, rollback, and failure recovery remain to be designed.

### 3. Third-Party Application Updates

Third-party applications installed through Aurea Store using Flatpak/Flathub are updated through their own distribution mechanisms.

These applications do not become Debian base packages or AureaOS system components merely because they are installed through Aurea Store.

This separation is an architectural requirement, not an already implemented update system.

## Build Objectives and Status

### Completed Foundations

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
- [x] Add initial final checks for Debian codename, target architecture, APT components, and fundamental package status.

### Remaining Work

- [ ] Run a complete fresh build and validate package resolution and installation against live Trixie repositories.
- [ ] Review the installed package set and its total size.
- [ ] Audit package versions, dependencies, licenses, and firmware coverage.
- [ ] Define system accounts, hostname, locale, timezone, and service defaults.
- [ ] Define the partition scheme and Btrfs subvolume/mount layout.
- [ ] Configure firewall defaults and test networking, DNS, and Bluetooth behavior.
- [ ] Select the release image's boot mode and configure the appropriate bootloader.
- [ ] Define the final UEFI and Secure Boot requirements.
- [ ] Produce and test a bootable x86-64 image.
- [ ] Design and validate Aurea Time backup and restore.
- [ ] Implement the AureaOS system update mechanism.
- [ ] Test installation, updates, recovery, and representative hardware.
- [ ] Document supported hardware and known limitations.

## Reproducibility and Validation

The bootstrap procedure has been executed successfully, but this does not establish bit-for-bit reproducibility.

Repository package versions can change over time.

A reproducible release process will need:

- Documented package snapshots or version constraints.
- Build-environment control.
- Artifact checksums.
- Automated package and filesystem validation.
- Repeatable image-generation procedures.

The current root filesystem is not a bootable AureaOS image.

No final installation ISO has been produced.

AureaOS Roma 1.0 must not be considered ready for installation or distribution until the complete build, boot, installation, update, and recovery paths have been validated.
