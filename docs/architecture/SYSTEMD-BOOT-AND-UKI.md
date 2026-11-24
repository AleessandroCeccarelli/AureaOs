<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# systemd-boot and Unified Kernel Images

## Status

**Approved platform decision:** systemd-boot is the primary boot manager for AureaOS Roma 1.0.

**Release constraints:** UEFI only; legacy BIOS/CSM is unsupported. Secure Boot is disabled for the first release. GRUB 2 is not part of the default installation and may be reconsidered only if a future, demonstrated requirement justifies it.

UKI is the selected architectural direction for kernel boot artifacts, subject to the implementation and compatibility checks listed below. This document describes architecture and requirements; it does not claim that the installer, boot pipeline or recovery environment is implemented.

## Why systemd-boot

AureaOS targets modern UEFI computers and Debian GNU/Linux 13 Trixie. systemd-boot is a small UEFI boot manager maintained within the systemd ecosystem and is packaged by Debian 13. Its focused scope is a better fit than carrying GRUB 2's broader legacy and multi-platform capabilities when AureaOS does not target legacy BIOS.

The decision is based on platform fit, not on novelty. GRUB 2 remains a technically valid and mature bootloader, but it is not the default architecture for AureaOS.

## Firmware and hardware target

- UEFI firmware is mandatory.
- Legacy BIOS and UEFI CSM boot are not supported.
- The initial target is modern amd64 PCs, approximately from the last decade, provided they implement a usable UEFI boot environment.
- Calendar age alone is not a compatibility guarantee. Firmware quality, UEFI implementation, storage controller, graphics and other hardware requirements must be validated separately.
- The installer must detect and clearly report when it has itself been started in Legacy/CSM mode; it must not silently install a system that cannot use the selected boot path.

The exact minimum UEFI revision and hardware support matrix remain subject to the Roma 1.0 hardware qualification process.

## Secure Boot policy

Secure Boot is **disabled for Roma 1.0**. The project will not make Microsoft signing, Microsoft third-party UEFI CA enrollment or a commercial certification/signing arrangement a release prerequisite while those costs are not sustainable.

This is an explicit first-release scope decision, not an accidental omission. The installer and documentation must explain that Secure Boot needs to be disabled in firmware for the initial release. The system must not claim Secure Boot protection or signed-chain verification while it is disabled.

The architecture should avoid choices that make future Secure Boot support unnecessarily difficult. UKI generation, artifact provenance, reproducible build considerations and a future signing boundary may be prepared, but unsigned UKIs are not trusted merely because they are unified images. Before enabling Secure Boot in a later release, AureaOS must define key ownership, signing authority, revocation and recovery procedures, firmware compatibility, and the sustainable route for distributing trusted boot binaries. Owner-managed keys and a Microsoft-trusted signing path are distinct options to evaluate; neither is approved by this document.

## Boot artifacts: UKI direction

A Unified Kernel Image combines the Linux kernel, initramfs and selected boot metadata (such as the kernel command line and OS identification) into one EFI executable. Debian 13 provides the `systemd-ukify` package, while `systemd-boot` and its tools are available in Trixie.

The intended direction is:
- Generate a versioned UKI for each supported kernel.
- Keep the kernel command line controlled by AureaOS configuration, not by ad-hoc installer or user shell fragments.
- Include only required initramfs content and hardware microcode where applicable.
- Keep at least one known-good previous boot artifact during a kernel/update transition, subject to storage and retention policy.
- Validate UKI contents and metadata before making a new entry the default.
- Do not embed secrets, private keys, user data or mutable application state in a UKI.

The exact integration between Debian kernel packages, initramfs-tools, `kernel-install`, `ukify` and Aurea Update must be prototyped on Trixie before implementation is considered complete.

## Disk layout and ESP

The installed OS layout remains deliberately simple:
- One EFI System Partition (ESP), formatted with a UEFI-compatible FAT filesystem.
- One Btrfs partition containing the entire installed AureaOS system.
- No XBOOTLDR partition.
- No separate `/home`, `/var`, cache, snapshot or backup partition.

systemd-boot and the UKIs will reside on the ESP. Btrfs remains the system filesystem; the firmware and systemd-boot must not be expected to read Linux kernel artifacts directly from Btrfs.

The ESP must have enough capacity for the boot manager, the current UKI, at least one fallback/previous UKI, and the independent on-disk Aurea Recovery boot artifact if that recovery design is retained. **The final ESP size is not yet frozen.** A 1 GiB ESP is a working sizing target to validate by measuring real kernel, initramfs, UKI and recovery artifact sizes across supported hardware and update scenarios. Do not silently shrink this to a conventional small ESP without evidence.

The installer must validate free space before writing a new UKI and must never delete the only known-bootable entry to make room for an update.

## Boot entries, updates and fallback

systemd-boot supports Boot Loader Specification entries, UKIs and boot counting with fallback to older entries when an entry repeatedly fails to complete booting. AureaOS must define the userspace success signal and lifecycle that marks a boot successful; the boot manager cannot independently prove that the desktop, services or user session are healthy.

Aurea Update and the boot-artifact manager must coordinate:
1. Stage the new kernel/initramfs/UKI without replacing the current known-good artifact.
2. Validate the artifact and its metadata.
3. Publish a boot entry with a bounded number of attempts.
4. Start the new entry.
5. Confirm successful system startup through an explicit system service after required health checks.
6. Retain or select the previous known-good entry if the new boot repeatedly fails.
7. Clean up older artifacts only under a defined retention and free-space policy.

This is a target lifecycle, not implemented functionality. Package-manager hooks must not independently race with Aurea Update or delete artifacts still referenced by boot entries.

## Aurea Recovery

Aurea Recovery is a separate recovery environment, not a Btrfs snapshot and not a second backup system.

The intended design is an independently bootable recovery image, selectable from systemd-boot and capable of restoring the installed OS through Aurea Time even when the installed root filesystem does not boot. The on-disk recovery artifact must not be the only recovery route: damaged or unavailable internal storage requires external recovery media.

The recovery environment must be designed and validated separately for:
- booting without relying on the installed root filesystem;
- detecting the target disk and its single Btrfs OS partition;
- discovering an authorized Aurea Time backup on USB or NAS;
- restoring system files, settings, metadata and boot artifacts safely;
- reinstalling or repairing systemd-boot and its EFI entries;
- reporting failures without destroying the only available backup.

## Ownership boundaries

- **UEFI firmware:** initializes hardware and launches the selected EFI executable.
- **systemd-boot:** presents boot choices and launches the selected EFI image.
- **UKI build/update pipeline:** creates, validates, stages and retires boot artifacts.
- **Initramfs:** discovers and prepares the installed system root.
- **Aurea Recovery:** provides an independent maintenance and restore environment.
- **Aurea Time:** owns backup history and restore data, from individual files to the full OS.
- **Aurea Update:** coordinates OS updates and boot-artifact transitions.

The boot manager must not implement backup policy, manage Passkey credentials, or orchestrate desktop services.

## Secure Boot readiness and future activation

Secure Boot remains disabled by default and is not a Roma 1.0 acceptance requirement. The boot architecture must nevertheless be designed so that a later release can enable it without replacing the boot manager, UKI format, installer disk layout or update lifecycle.

### Trust-chain direction

Debian 13 Trixie already packages a Secure Boot path for systemd-boot:
- `shim-signed` supplies a Microsoft-UEFI-CA-signed first-stage loader.
- Debian supplies signed shim helper binaries and a signed systemd-boot package for supported architectures.
- `systemd-ukify` can build UKIs and sign the final PE image with a Secure Boot certificate.

The intended future chain is:

1. UEFI firmware validates the Microsoft-signed shim, when the machine trusts the relevant Microsoft UEFI CA.
2. Shim validates and launches the Debian-signed systemd-boot using its trusted distribution certificate.
3. The next-stage boot policy validates the AureaOS UKI using an explicitly trusted AureaOS signing certificate, enrolled through a supported trust path.
4. The UKI contains the kernel, initramfs and controlled command line, so the signature covers the complete boot payload rather than a kernel and a separately mutable initramfs.

This is the target trust model, not a claim that the chain has already been integrated or tested. In particular, the exact interaction between Trixie's systemd-boot, shim's MOK database, UEFI image verification and UKI signatures must be demonstrated on real firmware before selecting the final enrollment path.

### Trust enrollment options

Two routes must be evaluated rather than conflated:

- **Owner enrollment:** the user explicitly enrolls the AureaOS public certificate in firmware Secure Boot DB, or through shim's Machine Owner Key (MOK) flow where the selected chain supports it. This can avoid AureaOS obtaining its own Microsoft signing relationship, but requires a clear, safe, per-device enrollment step and compatible firmware.
- **Broad pre-trusted distribution:** AureaOS obtains a signing/distribution arrangement that makes its first-stage boot component trusted by a broad range of stock PCs without a per-device custom-key enrollment. This is a separate future business and release-engineering decision; do not assume it is free, guaranteed or required for every Secure Boot deployment.

Debian's Microsoft-signed shim is a possible foundation, not a guarantee of universal compatibility. Some firmware disables the Microsoft third-party UEFI CA, contains different CA generations, or has revocation/firmware defects. The installer must detect and explain trust-chain failures rather than silently promise Secure Boot support.

### Signing architecture to prepare now

The boot-artifact pipeline must expose a signing boundary independent of the UKI build:
- Build and validate the UKI first; signing is a distinct finalization step.
- Roma 1.0 uses an explicit unsigned development/release profile while Secure Boot is disabled.
- A future Secure Boot profile signs the final UKI after every content change. Never modify the PE image after signing.
- Keep development, test and production signing identities separate.
- Keep production private signing keys out of installed systems, user home directories and ordinary CI logs/artifacts. Use controlled offline or hardware-backed custody and restricted signing access.
- Distribute only public certificates and enrollment material; never distribute private keys.
- Verify the signature and artifact digest before publishing or installing a signed UKI.
- Define certificate expiry, renewal, rotation, emergency revocation, compromised-key response and recovery before enabling the feature.
- Include and maintain SBAT metadata where applicable, and track firmware DBX and shim revocation updates.
- Treat TPM PCR policy signing/sealing as a separate optional feature; it is not required merely to enable Secure Boot.

Aurea Update must preserve the last known-bootable signed artifact while staging and validating a replacement. A failed signature check must prevent promotion of the new artifact and must not delete the previous known-good entry. Aurea Recovery must itself be bootable under the selected trust policy, or provide a separately documented external recovery path.

### Roma 1.0 behavior

- Secure Boot is not required and remains disabled in the supported installation profile.
- The UKI build pipeline should already be structured around the same UKI artifact format and validation steps that the future signed profile will use.
- No production Secure Boot key is generated or embedded on user machines for Roma 1.0.
- Do not enroll AureaOS keys, alter firmware DB/KEK/PK variables, or change Secure Boot state automatically.
- Do not describe an unsigned UKI as Secure Boot protected.

### Validation gates before future activation

1. Verify the exact Trixie amd64 versions and package behavior for `shim-signed`, `shim-helpers-amd64-signed`, `systemd-boot`, `systemd-boot-efi-signed`, `systemd-ukify`, `mokutil` and signing tools.
2. Demonstrate the complete shim → systemd-boot → Aurea-signed UKI chain with Secure Boot enabled on representative firmware.
3. Confirm whether MOK enrollment is honored by the exact UKI loading path; otherwise use and test explicit firmware DB enrollment or another supported trust path.
4. Test key enrollment, cancellation, removal, rotation and recovery, including machines with Microsoft third-party CA disabled and newer CA databases.
5. Test revoked/expired signatures, DBX and SBAT update behavior, interrupted UKI updates and fallback.
6. Validate the independent Aurea Recovery path under the same Secure Boot policy.
7. Decide whether per-device owner enrollment is acceptable or whether a broad pre-trusted distribution route is needed.
8. Only then enable Secure Boot as an optional or default installation profile in a later release.

No signing key, firmware variable, Secure Boot setting or boot artifact is changed by this documentation decision.


## Validation required before implementation is frozen

1. Verify the exact Debian 13 package set and supported systemd 257 interfaces on the target amd64 image.
2. Prototype UKI generation from Debian kernel packages and initramfs-tools, including microcode handling and kernel command-line ownership.
3. Measure ESP usage for current, previous and recovery UKIs; validate the provisional 1 GiB target.
4. Test install, upgrade, interrupted update, failed boot counting, fallback and cleanup.
5. Test firmware fallback-path behavior and NVRAM entry creation/removal.
6. Test recovery boot when the installed Btrfs root is damaged or unavailable.
7. Test restore of the OS and boot chain from an external Aurea Time target.
8. Document Secure Boot as disabled in Roma 1.0 and keep future signing work explicitly out of first-release acceptance criteria.

No build, package installation, firmware modification or hardware test is implied by this document.
