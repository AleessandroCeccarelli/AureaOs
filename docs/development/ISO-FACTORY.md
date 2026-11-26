<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS ISO Factory

**Status:** operating procedure and target specification  
**Last reviewed:** 2026-11-26  
**Repository:** `AleessandroCeccarelli/AureaOs`  
**Primary branch:** `main`  
**Target platform:** Debian GNU/Linux 13 (Trixie), amd64

This document defines the recurring development-to-ISO workflow for AureaOS. It is deliberately explicit about what is already implemented, what remains to be improved, and what must be verified on real hardware. It is the shared project reference; the more detailed chronological record remains in the [development log](../development/DEVELOPMENT-LOG.md).

## 1. Purpose

ISO Factory is the stable, repeatable release workflow used to turn the current, verified state of the AureaOS repository into a new Live ISO.

The weekly cycle is:

1. Develop and commit changes on GitHub during the week.
2. On Friday or Saturday, synchronize the Fedora workstation with `origin/main`.
3. Run Rust formatting, compilation and tests locally on Fedora.
4. Build the ISO from that exact local repository state using the maintained ISO scripts.
5. Validate the boot payload and ISO structure automatically.
6. Boot the image on physical hardware and record the outcome.
7. Preserve the ISO, checksum, build manifest, validation report and logs for that build.

The goal is to update the operating system every week without rewriting the build procedure or losing the history of previous test images.

## 2. Non-negotiable operating rules

- **One source of truth:** the intended release source is the exact commit on `main` that has been synchronized to Fedora and passed the agreed local checks.
- **No weekly script rewrites:** improve the maintained scripts when a verified defect or requirement demands it; do not create a new one-off build procedure every week.
- **No unverified success claims:** a successful `live-build` process or an ISO file on disk does not prove that the Live system boots.
- **Preserve evidence:** failed builds must retain their workspace and diagnostics. Successful builds must retain the workspace/cache unless an explicit cleanup operation is requested.
- **Do not overwrite evidence:** previous release candidates and their reports should remain identifiable by commit and build ID.
- **No remote substitute for Fedora validation:** the agreed Rust formatting, build and test checks are run on the user's Fedora workstation. A GitHub source update is not proof that those checks passed.
- **Keep the project modular:** one script or module has one responsibility; keep the orchestration script small and delegate validation to focused scripts.
- **Respect AureaOS architecture:** Debian Trixie amd64; Wayland-only, Vulkan-only, Smithay-based desktop. The ISO pipeline must not introduce X11, XWayland, OpenGL, GNOME, KDE or XFCE as AureaOS product dependencies.
- Work on `main` unless the user explicitly changes this policy. Do not discard uncommitted local work to force synchronization.

## 3. Weekly workflow

### Phase A — GitHub development (Monday to Friday)

- Work on the repository's `main` branch.
- Keep commits focused and messages descriptive.
- Update the relevant technical documentation and the development log when decisions, behavior, limitations or verification results change.
- Do not generate a weekly ISO during ordinary desktop-development sessions.
- Do not claim a feature is complete merely because its API, files, design or tests have been added; distinguish prototype, implementation, automated verification and hardware verification.

### Phase B — Synchronize Fedora (Friday or Saturday)

From the AureaOS repository on Fedora:

```bash
git status --short --branch
git switch main
git pull --ff-only origin main
git status --short --branch
git rev-parse HEAD
git log -1 --oneline --decorate
```

Expected conditions before validation:

- The current branch is `main`.
- The pull completed as a fast-forward or reported that the branch is already current.
- There are no unexpected uncommitted changes.
- The full commit ID is recorded for the weekly validation.

If the working tree contains changes, stop and inspect them. Do not reset, clean or overwrite local work automatically. Resolve or preserve the changes deliberately before continuing.

### Phase C — Rust quality gates on Fedora

Run the checks in this order:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

Then run additional feature-specific checks required by the code being validated, for example the compositor DRM feature when applicable:

```bash
cargo check -p aurea-compositor --features drm
```

The exact additional commands depend on the current implementation and supported hardware. Do not claim a command passed unless its actual exit status and output were observed. Record failures, ignored tests and hardware-dependent skips accurately.

**Gate:** do not proceed to ISO production while a required formatting, compilation or test gate is failing. A deliberately documented exception requires an explicit decision and must be recorded in the manifest/log.

### Phase D — Build the ISO

The maintained Fedora entry point is the Podman wrapper:

```bash
bash tools/iso/build-live-iso-podman.sh
```

The wrapper currently:
- uses a Debian Trixie slim container for the live-build environment;
- mounts the repository read-only;
- uses a persistent build-parent directory and a workspace lock;
- delegates the actual image generation to `tools/iso/build-live-iso.sh`;
- writes the current candidate to `~/Documenti/AureaOS.iso` by default and creates a SHA-256 sidecar.

For an interrupted build, the current wrapper supports resuming the same workspace:

```bash
RESUME=1 BUILD_PARENT=/absolute/path/to/aureaos-podman-build \
  bash tools/iso/build-live-iso-podman.sh
```

Use the same build parent that contains the interrupted workspace. Do not start a second build against the same workspace. Do not delete a failed workspace before its logs and cause have been examined.

The inner script stages source using `git archive HEAD`. Therefore, it builds the local checked-out commit, not an automatic live fetch of GitHub. The Fedora synchronization and quality gates above are what ensure that this local commit is the intended current `main` state. The script must not silently pull or change branches during a build.

### Phase E — Automatic image validation

The current source tree contains two focused validation scripts:

- `tools/iso/tests/validate-chroot-boot.sh` — checks required boot packages and live-boot initramfs integration in the target chroot, regenerates initramfs and checks for `scripts/live`.
- `tools/iso/scripts/build-systemd-boot-live-iso.sh` — assembles an unsigned UKI from the exact Debian kernel/initramfs pair, builds a FAT EFI boot image containing systemd-boot and the UKI entries, removes intermediate GRUB payloads, and remasters the UEFI-only ISO.
- `tools/iso/tests/validate-live-image.sh` — checks the final EFI image, systemd-boot fallback path, loader entry, UKI sections and embedded `boot=live` command line, plus the Debian Live filesystem and El Torito EFI catalog.

The build hook invokes the chroot boot validator after package installation and cleanup. The build coordinator validates the staged binary payload and then validates the copied ISO.

The validation layers should cover, at minimum:

1. Required package state in the target chroot: `live-boot`, `live-boot-initramfs-tools`, `live-config`, `initramfs-tools` and the intended kernel package.
2. Presence and execution of the live-boot initramfs hook and early-boot script.
3. Successful initramfs generation for the installed kernel.
4. Presence of `scripts/live` in every selected initramfs.
5. Matching, non-empty kernel and initramfs files under the Live image payload.
6. A FAT EFI image containing `EFI/BOOT/BOOTX64.EFI`, systemd-boot and loader configuration.
7. One or more UKIs containing the kernel, initramfs, AureaOS os-release metadata and embedded `boot=live` command line.
8. Matching systemd-boot entries and a valid default entry.
9. No GRUB or split kernel/initramfs boot payload in the final image.
10. ISO readability and an El Torito EFI boot catalog referencing the systemd-boot image.
11. SHA-256 checksum and a machine-readable validation report.

A test that is not implemented or not executed must be marked as pending, not assumed to pass.

### Phase F — Physical hardware validation

Automatic inspection cannot replace a real boot. Copy the candidate ISO to the existing Ventoy data partition and select it from Ventoy. Do not reformat or recreate the USB drive as part of the routine.

Record the following separately:

- Did Ventoy load the image?
- Did systemd-boot display the AureaOS Live entry?
- Did the kernel load?
- Did initramfs complete and locate the Live filesystem?
- Did systemd reach the expected target?
- Did the AureaOS session service start?
- Did the compositor obtain the DRM device and seat?
- Was a physical display output detected and presented?
- Did the shell appear and remain responsive?
- Did keyboard, mouse, wheel, touchpad and other available input devices work?
- Were there errors, hangs, kernel panics or missing firmware?

Record the exact machine, firmware/boot mode, GPU and display configuration when known. Do not attribute an early initramfs failure to Vulkan, Smithay or the GPU unless the logs show that execution reached those layers.

### Phase G — Archive and record

Every candidate must be traceable to its source and its tests. The target artifact set is:

- ISO image.
- SHA-256 checksum.
- Build manifest (for example, `manifest.json`).
- Automated validation report.
- Full build log and failure diagnostics.
- Hardware test record.
- Exact Git commit ID and branch.

Recommended identity:

`AureaOS-YYYYMMDD-<short-commit>.iso`

Keep a stable `AureaOS.iso` convenience copy only after the candidate has passed all required automatic checks. Preserve the uniquely named candidate and its records even when it is not promoted.

The manifest should record at least:

- project and image name;
- build ID and timestamp, including timezone or UTC;
- Git remote, branch and full commit ID;
- whether the working tree was clean;
- Debian release and architecture;
- build method and builder/container image identity;
- kernel version;
- ISO size and SHA-256;
- each validation check and its result;
- hardware test status, machine description and notes;
- known failures and deviations.

**Reproducibility terminology:** every image must be reproducibly traceable to a specific source commit and build inputs. Bit-for-bit identical ISO output is a stronger property and must not be promised until timestamps, package repository snapshots, toolchain/container versions and all other variable inputs are pinned and tested.

## 4. Current implementation versus target

The repository already has a working foundation, but ISO Factory is not yet a fully finished release-management system.

### Implemented in the repository as inspected on 2026-11-23

- Debian Trixie amd64 live-build configuration.
- Production desktop build enables the opt-in `heif` feature, while ordinary development builds remain free of a mandatory native libheif dependency.
- The target image installs `libheif1` and only the HEVC/AV1 still-image decoder plugins; `libheif-dev` is removed after compilation.
- Podman wrapper using a Debian Trixie container.
- Persistent live-build workspace and workspace serialization with `flock`.
- Resume path for an interrupted build.
- Chroot hook that builds and installs the AureaOS desktop binary and session integration.
- Explicit boot package list including `live-boot-initramfs-tools`.
- Chroot boot validation script.
- Binary-tree/ISO boot payload validation script.
- systemd-boot + UKI EFI image assembly and final UEFI-only ISO mastering.
- UKI inspection for embedded Live boot parameters and final image validation.
- Workspace preservation after success or failure in the current inner build script.
- SHA-256 sidecar generation.

### Remaining work before calling ISO Factory release-ready

- Inspect the existing failed-boot ISO's initramfs on Fedora before another rebuild.
- Establish why the real boot reported that `/scripts/live` could not be opened. The exact root cause remains unconfirmed.
- Verify the validator's assumptions against the actual generated image and both BIOS/UEFI paths.
- Make publication atomic: validate a temporary candidate before replacing the stable `AureaOS.iso`. A failed validation must never overwrite the last known candidate.
- Preserve uniquely named historical ISO artifacts, manifests, reports and logs rather than retaining only one mutable filename.
- Generate a manifest containing the exact source commit and actual validation results.
- Persist complete timestamped logs and a concise final report for successful and failed runs.
- Add explicit, safe cleanup controls; never automatically delete a workspace or historical artifact needed for diagnosis or rollback.
- Consider caching/versioning the builder container so package installation does not needlessly repeat, without compromising updates or reproducibility.
- Verify BIOS and UEFI boot and the real Live session on physical hardware.

These are requirements and follow-up tasks, not claims that they are already implemented.

## 5. Boot failure currently under investigation

The first GRUB-based test ISO was readable and had a valid SHA-256. On the first real Ventoy test, GRUB loaded the Debian kernel, but init failed before the desktop:

```text
init: line 249: can't open '/scripts/live': No such file or directory
Kernel panic - not syncing: Attempted to kill init: exitcode=0x00000200
```

The established fact is that `/scripts/live` was unavailable when init attempted to execute it. The kernel panic followed that init failure. The cause has not yet been isolated. The new factory retains the chroot preflight, regenerates the initramfs after package installation, validates `scripts/live`, and embeds that exact initramfs in the UKI. This is a stronger automated guard, not proof that the previous root cause is fixed until the new image is booted.

No evidence from this failure establishes a GPU, Vulkan or Smithay fault: the system did not reach compositor startup.

## 6. Friday/Saturday run checklist

- [ ] Confirm repository is on `main`.
- [ ] Fast-forward pull from `origin/main`.
- [ ] Confirm clean working tree and record full commit ID.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo check --workspace`.
- [ ] Run `cargo test --workspace`.
- [ ] Run relevant feature-specific checks.
- [ ] Record command results and exceptions.
- [ ] Inspect the existing initramfs first while the boot issue remains open.
- [ ] Run the maintained Podman ISO entry point only after the diagnosis/decision.
- [ ] Confirm chroot and image validation reports pass.
- [ ] Verify checksum and BIOS/UEFI image structure.
- [ ] Boot via the existing Ventoy setup on real hardware.
- [ ] Record boot, desktop and input-device results.
- [ ] Preserve ISO, manifest, logs and validation reports.
- [ ] Update both language development logs and this document when facts or decisions change.

## 7. Related files

- `tools/iso/build-live-iso-podman.sh` — Fedora/Podman entry point.
- `tools/iso/build-live-iso.sh` — Debian live-build coordinator.
- `tools/iso/package-lists/aureaos.list.chroot` — target package manifest.
- `tools/iso/hooks/010-build-aureaos.hook.chroot` — desktop build and chroot integration.
- `tools/iso/scripts/build-systemd-boot-live-iso.sh` — UKI, EFI system partition image and final ISO assembly.
- `tools/iso/os-release` — AureaOS identity embedded in the UKI.
- `tools/iso/tests/validate-chroot-boot.sh` — chroot boot preflight.
- `tools/iso/tests/validate-live-image.sh` — staged-image and ISO validation.
- [Development log](../development/DEVELOPMENT-LOG.md).
- [Internal assistant handoff](../../tools/iso/INTERNAL-ASSISTANT-HANDOFF.md) — continuity notes; not a user-facing product specification.
