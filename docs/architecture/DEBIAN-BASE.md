# AureaOS Debian Base

## Base distribution

- Distribution: Debian GNU/Linux 13 (Trixie)
- Release channel: Stable
- Target architecture: `amd64` (x86-64)
- Initial root filesystem tool: `debootstrap`
- Bootstrap mirror: `https://deb.debian.org/debian`
- Initial debootstrap profile: `minbase`
- Bootstrap and target APT components: `main`, `contrib`, `non-free`, `non-free-firmware`

## Initial system profile

The first build stage creates a minimal, terminal-only Debian root filesystem. It is a development artifact, not a complete AureaOS installation image. No graphical desktop or desktop applications are included at this stage.

The package selection is maintained under `debian/packages/`. The bootstrap script reads and installs the common manifests. A successful syntax check does not prove all packages resolve; the integrated profile still needs a full test against the live Debian Trixie repositories.

## Build implementation

The bootstrap script is `tools/build/bootstrap-debian.sh`.

Default destination:

`~/Documenti/AureaOS-Build/rootfs`

A different absolute path can be supplied with `--target`. Generated root filesystems are kept outside the Git repository. The script refuses to overwrite an existing destination and verifies the Debian codename and target architecture after bootstrap.

The script runs `debootstrap` with suite `trixie`, architecture `amd64`, variant `minbase`, and all four components: `main`, `contrib`, `non-free`, and `non-free-firmware`. It then writes `/etc/apt/sources.list.d/debian.sources` for `trixie`, `trixie-updates`, and `trixie-security`, updates package indexes, and installs the common manifests.

The optional `--with-rar` flag adds RAR extraction support. `--boot-mode bios` and `--boot-mode uefi` select the corresponding GRUB manifest; the default is `none`, so no bootloader is selected automatically. The script does not configure Btrfs, firewall policy, user accounts, or create an installable image.

## Package manifest

`debian/packages/README.md` documents the proposed groups and their limitations. The package lists separate the terminal-only base, networking, Bluetooth, security, storage, archives, firmware, kernel/initramfs, and alternative BIOS/UEFI bootloader packages.

All four Debian components are enabled consistently for the main archive and security updates. This makes the selected packages available to APT; it does not automatically install every package from those components. The project is not limited to free-software-only packages.

## Build objectives and status

- [x] Select Debian 13 Trixie and `amd64` as the initial target.
- [x] Select `debootstrap` and create a minimal root filesystem outside the repository.
- [x] Add a bootstrap script with destination safety checks.
- [x] Validate the script syntax and its help behavior.
- [x] Test refusal to overwrite an existing destination.
- [x] Build and verify a second minimal root filesystem using the script.
- [x] Add an initial package-manifest proposal under `debian/packages/`.
- [x] Enable all four Debian archive components for bootstrap and configure Trixie, updates, and security APT sources.
- [x] Integrate the common package manifests into the bootstrap script with optional RAR and BIOS/UEFI GRUB selections.
- [ ] Run a complete fresh build and validate package resolution/installation against live Trixie repositories.
- [ ] Define system accounts, hostname, locale, timezone and service defaults.
- [ ] Define the partition scheme and Btrfs subvolume/mount layout.
- [ ] Configure firewall defaults and test networking, DNS and Bluetooth behavior.
- [ ] Select the release image's boot mode and configure the appropriate bootloader.
- [ ] Produce and test a bootable x86-64 image.
- [ ] Test installation, updates, recovery and representative hardware.
- [ ] Document supported hardware and known limitations.

## Filesystem, boot and recovery

Installing `btrfs-progs` only supplies Btrfs administration tools. A Btrfs root requires an explicit partitioning plan, subvolumes, mount configuration and compatible initramfs/boot configuration. Snapshot and recovery behavior must be designed and tested; installing Btrfs tools alone does not provide automatic snapshots or rollback.

The kernel and initramfs packages do not make a root filesystem bootable by themselves. The image build must select a target boot mode and install/configure the matching bootloader. `boot-bios.list` and `boot-uefi.list` are alternatives, not lists to install together by default.

## Update policy

The intended policy separates three update domains:

1. Debian base packages and security updates from explicitly configured Debian repositories.
2. AureaOS-specific system components from a project-managed release/update mechanism, which remains to be designed.
3. Third-party applications delivered through Flatpak/Flathub, updated through their own distribution mechanisms.

This separation is an architectural intention, not an implemented update system. Compatibility, rollback, signing, repository trust and failure recovery must be designed before release.

## Reproducibility and validation

The bootstrap procedure has been executed successfully, but this does not establish bit-for-bit reproducibility. Repository package versions can change over time. A reproducible release process will need documented package snapshots or version constraints, build-environment control, artifact checksums and automated validation.

The current root filesystem is not a bootable AureaOS image and no final installation ISO has been produced.
