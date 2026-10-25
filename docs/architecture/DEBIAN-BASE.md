# AureaOS Debian Base

## Base Distribution

- Distribution: Debian GNU/Linux
- Release: Debian 13 (Trixie)
- Release channel: Stable
- Target architecture: amd64 (x86-64)
- Initial build method: `debootstrap`
- Debian mirror: `https://deb.debian.org/debian`

## Initial System Profile

The initial AureaOS base is a minimal, terminal-only system.

It must not include a graphical desktop environment or desktop applications.

Only the packages and services required for a functional, maintainable base system should be included.

The initial root filesystem is created using Debian's `minbase` profile and the `main` repository component.

## Build Objectives

- [x] Define the initial Debian release and target architecture.
- [x] Select `debootstrap` for creating the initial root filesystem.
- [x] Create a minimal Debian root filesystem outside the Git repository.
- [x] Implement a bootstrap script in `tools/build/bootstrap-debian.sh`.
- [x] Validate the bootstrap script's Bash syntax.
- [x] Verify that the script refuses to overwrite an existing destination.
- [x] Build and verify a second root filesystem using the bootstrap script.
- [ ] Define the initial package selection and system configuration in the repository.
- [ ] Configure and document the system's APT repositories and update policy.
- [ ] Establish a reproducible Linux build environment targeting amd64.
- [ ] Validate the complete filesystem configuration and boot behavior.
- [ ] Produce a bootable x86-64 test image.
- [ ] Document dependencies, licenses, and known limitations.

## Build Process

The initial root filesystem is created with `debootstrap` using the following configuration:

- Distribution: `trixie`
- Architecture: `amd64`
- Variant: `minbase`
- Components: `main`
- Mirror: `https://deb.debian.org/debian`

The bootstrap script is located at:

`tools/build/bootstrap-debian.sh`

By default, the script creates the root filesystem at:

`~/Documenti/AureaOS-Build/rootfs`

A different destination can be specified with the `--target` option.

The build output is kept outside the Git repository to prevent generated system files from being mixed with source code, documentation, and build tooling.

The script checks the destination before starting and refuses to overwrite an existing path.

## Validation Status

The following initial checks have been completed:

- The bootstrap script passes Bash syntax validation.
- The script's help option works.
- The script refuses to overwrite an existing root filesystem.
- A minimal Debian root filesystem has been created successfully.
- A second root filesystem has been created using the bootstrap script.
- The resulting system identifies itself as Debian GNU/Linux 13 (Trixie).
- The resulting root filesystem reports the `amd64` architecture.

These checks validate the initial bootstrap process. They do not establish that AureaOS is bootable or ready for installation.

The following validations remain outstanding:

- Verification of the complete system configuration.
- Validation of APT repository and security update configuration.
- Boot testing.
- Testing on supported hardware.
- Production and validation of a bootable x86-64 image.

The current build process is operationally repeatable, but bit-for-bit reproducibility has not yet been established. Package versions may change as Debian repositories are updated.

## Update Policy

The Debian base is intended to follow the configured official Debian repositories for the selected Stable release and its security updates.

The final APT repository configuration and update policy must be defined and verified before the system is considered ready for distribution.

AureaOS-specific components will have a separate update and release channel managed by the AureaOS project. Their distribution mechanism and integration with system updates remain to be defined.

Third-party applications distributed through Flatpak and Flathub will follow their respective distribution and update mechanisms.

The separation between Debian updates and AureaOS component updates must preserve compatibility between the underlying system and the AureaOS desktop experience.

## Scope

This document defines the initial Debian base and its bootstrap process.

The graphical desktop, AureaOS applications, installer, final release image, and complete system update implementation are outside the scope of the initial base implementation.

## Implementation Status

The initial Debian root filesystem has been built and the bootstrap script has been tested successfully.

The build script is version-controlled in the AureaOS repository. Generated root filesystems remain outside the repository.

The project has not yet produced a bootable AureaOS image or a final installation ISO.

Further work is required to define the system configuration, package selection, repository policy, boot process, and validation procedures before the Debian base can be considered ready for integration into AureaOS Roma 1.0.