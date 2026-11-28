<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS ISO Factory

**Status:** operating procedure and target specification  
**Last reviewed:** 2026-11-27  
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
- uses a persistent, source-commit-specific build-parent directory by default and a workspace lock;
- refuses to build when the repository contains uncommitted or untracked changes, because the source snapshot is created from `git archive HEAD`;
- delegates the actual image generation to `tools/iso/build-live-iso.sh`;
- publishes a uniquely named, validated historical candidate plus JSON manifest and SHA-256 sidecar in `~/Documenti` by default;
- updates the stable `AureaOS.iso` convenience alias and checksum only after the isolated assembly tree and the exact ISO both pass validation.

For an interrupted build, rerun from the same clean checkout and commit:

```bash
RESUME=1 bash tools/iso/build-live-iso-podman.sh
```

The default build parent includes the source commit ID, so a new commit receives a fresh workspace instead of accidentally reusing a completed chroot that contains an older `aurea-desktop` binary. If `BUILD_PARENT` was explicitly overridden, pass that exact same value when resuming. Do not start a second build against the same workspace. Do not delete a failed workspace before its logs and cause have been examined.

The inner script stages source using `git archive HEAD`. It now refuses dirty working trees and writes the full source commit to `.aureaos-source-commit` in each workspace. Resume is allowed only when that marker matches the current commit; legacy workspaces without a marker are rejected rather than trusted. Therefore, it builds the local checked-out commit, not an automatic live fetch of GitHub. The Fedora synchronization and quality gates above ensure that this local commit is the intended current `main` state. The script must not silently pull or change branches during a build.

### Phase E — Automatic image validation

The current source tree contains a chroot preflight, a UKI/ISO assembly helper, and a focused final-image validator:

- `tools/iso/tests/validate-chroot-boot.sh` — checks required boot packages and live-boot initramfs integration in the target chroot, regenerates initramfs and checks for `scripts/live`.
- `tools/iso/scripts/build-systemd-boot-live-iso.sh` — assembles an unsigned UKI from the exact Debian kernel/initramfs pair, builds a FAT EFI boot image containing systemd-boot and the UKI entries, removes intermediate GRUB payloads, and remasters the UEFI-only ISO.
- `tools/iso/tests/validate-live-image.sh` — checks the actual SquashFS rootfs is Debian 13 Trixie with systemd PID 1 and the required Debian Live packages; checks the desktop session integration and artwork; inspects every UKI and extracts its embedded initramfs to prove that the exact UKI payload contains `scripts/live`; verifies the EFI fallback loader, systemd-boot entries and El Torito EFI catalog.

The coordinator explicitly stages the chroot boot validator under `/usr/local/libexec/aureaos/` before live-build runs the hook. The hook regenerates and inspects initramfs before purging build-only dependencies. The UKI assembler works on a same-filesystem hard-link snapshot of the live-build binary tree: kernel/initramfs and GRUB staging files are removed only from that isolated snapshot, never from the resumable live-build workspace. It validates the snapshot before mastering the ISO; the coordinator then validates the exact ISO and the copied candidate.

The validation layers should cover, at minimum:

1. Required package state in the target chroot: `live-boot`, `live-boot-initramfs-tools`, `live-config`, the explicit `live-config-systemd` backend, `systemd-sysv`, `initramfs-tools` and the intended kernel package.
2. Presence and execution of the live-boot initramfs hook and early-boot script.
3. Successful initramfs generation for the installed kernel.
4. Presence of `scripts/live` in every selected initramfs and, independently, in the `.initrd` section extracted from every final UKI.
5. A non-empty Live filesystem image at `/live/filesystem.squashfs`, with the real desktop binary and session integration present inside it.
6. A FAT EFI image containing `EFI/BOOT/BOOTX64.EFI`, systemd-boot and loader configuration.
7. One or more UKIs containing the kernel, initramfs, AureaOS os-release metadata and embedded `boot=live` command line.
8. Matching systemd-boot entries and a valid default entry.
9. No GRUB or split kernel/initramfs boot payload in the final image.
10. ISO readability and an El Torito EFI boot catalog referencing the systemd-boot image.
11. The actual SquashFS rootfs contains the executable desktop, session launcher, systemd/PAM wiring and enabled service.
12. Both context-menu RGBA8 assets exist beneath `/usr/share/aureaos/human-interfaces` and have exact expected lengths.
13. SHA-256 checksum and a machine-readable validation report.

**Scope boundary:** the base-system gate ends when Debian 13 Trixie has mounted its Live SquashFS root, systemd PID 1 is running, and the required Live boot packages and initramfs payload have been verified. The subsequent desktop service, PAM session, compositor, Vulkan and visible desktop are AureaOS integration work; they are not evidence for whether Debian Live itself booted.

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

### Follow-up audit — 2026-11-27

A full source-snapshot and target-runtime dependency pass found and corrected two gaps:

- The `git archive HEAD` source snapshot included the wallpaper and context-menu masters, but omitted `aurea-human-interfaces/icons/source/system/trash`. Aurea Shell embeds six Trash SVG files with `include_bytes!`, so the release build could fail at compile time even though the assets existed in the repository. The ISO coordinator now archives the Trash source directory and checks all six files immediately after staging.
- The final SquashFS validator previously checked the compositor/session and cursor payloads, but did not prove that the actual image contained the AMD kernel module, AMD firmware or RADV Vulkan ICD. It now extracts and checks those payloads and verifies the corresponding installed Debian packages, including DRM/GBM/libinput/libseat/logind, XKB keyboard data and HEIF/TurboJPEG runtime packages. The chroot hook also checks shared-library resolution with `ldd` before removing build headers. The ISO-mode validator now extracts the complete `/boot` directory, so its no-GRUB check inspects the actual mastered image rather than an unpopulated temporary path.

The validator additions are static image-content checks; they do not prove that a physical GPU initializes, that Vulkan presentation succeeds on a particular machine, or that a physical mouse moves. Those remain hardware tests. No ISO build, Rust build or hardware test was run as part of this repository edit.

The ISO source archive is intentionally created from `git archive HEAD`, not from arbitrary uncommitted working-tree files. Therefore all intended changes must be committed on the checked-out commit before starting the Fedora/Podman build. The wrapper also refuses a dirty working tree and pins a workspace to that exact commit.

### Implemented in the repository as inspected on 2026-11-27

- Debian Trixie amd64 live-build configuration.
- Production desktop build enables the opt-in `heif` feature, while ordinary development builds remain free of a mandatory native libheif dependency.
- The target image installs `libheif1` and only the HEVC/AV1 still-image decoder plugins; `libheif-dev` is removed after compilation.
- Podman wrapper using a Debian Trixie container.
- Persistent live-build workspace and workspace serialization with `flock`.
- Resume path for an interrupted build.
- Chroot hook that builds and installs the AureaOS desktop binary and session integration, checks shared-library resolution and systemd unit validity, and generates/installs only the raw RGBA8 runtime artwork.
- Explicit boot package list including `live-boot-initramfs-tools`.
- Chroot boot validation script.
- Binary-tree/ISO boot payload validation script, including inspection of the actual SquashFS rootfs.
- systemd-boot + UKI EFI image assembly and final UEFI-only ISO mastering.
- UKI inspection for embedded Live boot parameters and final image validation.
- Workspace preservation after success or failure in the current inner build script.
- Source-commit marker and strict same-commit resume guard in the inner coordinator.
- Debian target dependencies for the always-built TurboJPEG C shim and the enabled HEIF feature; build-only headers are purged after use; SVG rasterization tools remain confined to the disposable builder container.
- SHA-256 sidecars for both the uniquely named candidate and the stable convenience alias.
- JSON provenance manifest for each validated candidate, including exact source commit, kernel version, image size/hash and hardware-validation state.
- Atomic promotion of the stable `AureaOS.iso` alias only after validation of the exact temporary ISO; uniquely named historical candidates are preserved.

### Remaining work before calling ISO Factory release-ready

- Inspect the existing failed-boot ISO's initramfs on Fedora before another rebuild.
- Establish why the real boot reported that `/scripts/live` could not be opened. The exact root cause remains unconfirmed.
- Verify the validator's assumptions against the actual generated UEFI image and El Torito EFI path.
- Persist complete timestamped logs and a concise final report for successful and failed runs.
- Add explicit, safe cleanup controls; never automatically delete a workspace or historical artifact needed for diagnosis or rollback.
- Consider caching/versioning the builder container so package installation does not needlessly repeat, without compromising updates or reproducibility.
- Verify UEFI systemd-boot/UKI boot and the real Live session on physical hardware.

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
- [ ] If the previous failed ISO is still available, inspect its initramfs and retain the old diagnosis.
- [ ] Run the maintained Podman ISO entry point after grouped Fedora Rust checks.
- [ ] Confirm chroot and image validation reports pass.
- [ ] Verify checksum and UEFI El Torito/systemd-boot/UKI image structure.
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

### Second-pass verification — 2026-11-27

- Debian Trixie `mesa-vulkan-drivers` installs the RADV manifest as `/usr/share/vulkan/icd.d/radeon_icd.json` (not `radeon_icd.x86_64.json`) and the driver as `/usr/lib/x86_64-linux-gnu/libvulkan_radeon.so`. The image validator now extracts these exact paths and verifies both the manifest reference and driver library. This corrects a false-negative path assumption in the previous validator revision.
- The source snapshot was checked against compile-time inclusions: wallpaper JPEG, six Trash SVGs and Vulkan SPIR-V generated by `components/aurea-compositor/build.rs`. The shader compiler is `glslangValidator`, provided by Debian `glslang-tools` in the chroot build dependency set and purged after compilation.
- Cursor generation uses Python standard-library modules plus `rsvg-convert` and `xcursorgen`. Both are installed only in the disposable Debian builder (`librsvg2-bin` and `x11-apps`); the coordinator generates both Xcursor themes before `live-build` and stages only the finished theme files into `includes.chroot`. Neither package nor any X11 utility is installed in the AureaOS target chroot.
- The compositor's current Vulkan feature plan explicitly rejects physical devices below Vulkan 1.3 and requires `dynamicRendering` plus `synchronization2`; shader compilation also targets `vulkan1.3`. Therefore the repository does **not** currently implement the previously discussed Vulkan 1.0–1.4 compatibility range. A lower-version fallback is a separate renderer change and must not be claimed as supported. The physical GPU's own `apiVersion` (not the Vulkan loader/instance version) must be checked on the test laptop.
- No complete ISO build or physical hardware boot has been executed in this audit.

### Terza passata tecnica — 2026-11-27
- Verificato il percorso di rendering di riserva: se la GPU associata al DRM non soddisfa i requisiti, AureaOS tenta un dispositivo Vulkan CPU (Lavapipe) e continua a presentare tramite GBM/KMS. La riserva richiede comunque Vulkan 1.3, quindi non risolve una mancanza di feature del dispositivo CPU; non va descritta come fallback universale.
- La factory installa già `mesa-vulkan-drivers`, che in Debian Trixie amd64 fornisce anche `/usr/share/vulkan/icd.d/lvp_icd.json` e `/usr/lib/x86_64-linux-gnu/libvulkan_lvp.so`. Il validatore controllava RADV ma non il payload Lavapipe: ora estrae e verifica anche manifest, riferimento JSON e libreria del driver CPU. Evidenza: file list ufficiale Debian Trixie amd64.
- La verifica riguarda la presenza dei componenti, non dimostra che Lavapipe riesca a creare il dispositivo o che il fallback sia selezionato correttamente sul portatile. Restano da provare runtime Vulkan e boot fisico.


### Boot-order and native-session audit — 2026-11-27
- Reconstructed the actual startup chain from the installed systemd unit through the PAM session, libseat/logind, udev DRM discovery, GBM, Vulkan, KMS outputs, Wayland output publication, pointer centering and Calloop rendering/input dispatch.
- The compositor deliberately does not sleep for an arbitrary duration or assume that device enumeration is complete at process launch: libseat owns access to the active seat; the Udev monitor supplies the DRM inventory; libinput is created against the same session handle and assigned to that seat. This is preferable to a fixed delay or a blanket `udevadm settle` dependency. systemd startup is parallelized, so `After=` expresses ordering only; hardware readiness remains an explicit runtime responsibility. Debian's systemd boot documentation describes this ordering model.
- Confirmed the compositor publishes outputs only after GBM/KMS surface initialization and validates that every enabled output has a retained native surface. The first frame is requested when entering the event loop; rendering fails explicitly if no GPU has an initialized Vulkan backend or if no compatible presentation path exists. Pointer centering occurs after outputs are published, and startup aborts if active outputs exist but the pointer cannot be placed.
- Strengthened the final SquashFS validator to check the complete systemd/logind/libseat/TTY contract: logind ordering and Wants, dedicated PAM registration, unprivileged `aurea` user, `LIBSEAT_BACKEND=logind`, `XDG_SEAT=seat0`, tty1 attachment, minimal live mode, session launcher and multi-user enablement link.
- This is static/source and image-validator coverage only. It does not prove the kernel's real device timing, logind session acquisition, DRM modeset, Vulkan device creation, physical pointer movement or visible wallpaper. Those remain tomorrow's Fedora/USB checks.


### ISO dependency correction — 2026-11-28

- The failed build log identified an invalid Debian Trixie package name: `xcursorgen` is an executable, not an installable package. Debian Trixie ships it in the `x11-apps` package.
- The ISO coordinator now builds the Light and Dark Xcursor themes in the disposable Debian Trixie Podman builder, where `librsvg2-bin` and `x11-apps` are installed as builder tools. Only the generated Xcursor theme files are copied into `config/includes.chroot/usr/share/icons` before `live-build`.
- The target package list no longer installs `librsvg2-bin` or `xcursorgen`, and the chroot hook no longer builds cursors or purges those tools. This keeps X11 utilities and their build dependencies out of the AureaOS Live filesystem while preserving the native Xcursor asset format used by the Wayland compositor.
- The builder image may contain X11-related build utilities temporarily; they are confined to the disposable Podman container and are not part of the shipped OS.
- This source correction has not yet been validated by a Fedora ISO build or physical boot. The previously preserved failed workspace belongs to the older source commit; do not resume it with the new source snapshot. Use the fresh commit-specific workspace created by the wrapper.


### Kernel, UKI size, and temporary bootloader staging — 2026-11-28

The Fedora build report confirmed that Debian Live installs the explicit
`linux-image-amd64` package and produced kernel/initramfs
`6.12.118+deb13-amd64`. The UKI builder had relied on version autodetection;
it now passes the exact version extracted from the staged kernel filename via
`ukify --uname`.

The UKI assembly stopped because the combined UKI payload did not fit the
31 MiB El Torito EFI image. The chroot now stages
`/etc/initramfs-tools/conf.d/aureaos.conf` with `COMPRESS=xz` before kernel
installation, reducing the embedded initramfs size. The assembler reports
exact UKI, bootloader, and ESP byte totals if the capacity check still fails.

Debian Live Build's intermediate hybrid-ISO stage is not AureaOS's final
bootloader. Its configuration now uses temporary Syslinux staging instead of
GRUB EFI, avoiding the unnecessary GRUB/shim package set. The final image is
still assembled separately as UEFI-only systemd-boot + UKI, and its validator
continues to reject GRUB payloads.

Obsolete failed Podman workspaces can be listed safely with
`bash tools/iso/cleanup-failed-builds.sh`. After reviewing the list, remove
only recognized old commit-specific workspaces with
`sudo bash tools/iso/cleanup-failed-builds.sh --apply`. The current commit's
workspace is always preserved, active workspaces are skipped, and unknown
directories are not touched.

## UKI inspection CLI correction — 2026-11-28

- The first full live-build binary stage completed and produced its intermediate hybrid ISO, but final systemd-boot/UKI assembly stopped while inspecting the generated UKI. The log ended with `ukify: error: unrecognized arguments: <UKI path>`.
- Debian Trixie's `ukify` CLI expects inspect options before the `inspect` verb: `ukify --all inspect FILE`. Both the UKI assembler and the final image validator now use that argument order. The earlier `ukify inspect --all FILE` invocation was rejected by the parser before image validation or final ISO promotion.
- This correction is committed to `main`; it has not yet been re-run in the Fedora/Podman build. The preserved failed workspace is from the previous source commit and must not be resumed under the new commit. Start a fresh commit-specific build as documented above.
