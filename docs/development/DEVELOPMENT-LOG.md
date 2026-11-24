<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Development Log

This is the chronological engineering diary for the AureaOS repository. It complements, rather than replaces, the architecture documents, security audits, roadmap, and Git history.

## 2026-11-18 — Repository baseline and display-scale foundation

### Repository state

- The active development branch is `main`.
- The repository currently has no other branches and no open pull requests.
- The Rust workspace contains four crates: `aurea-desktop`, `aurea-shell`, `aurea-desktop-api`, and `aurea-compositor`.
- The repository already has dedicated architecture, product, security, testing, and human-interface documentation. This entry adds a chronological record instead of duplicating those documents.
- No source files or branches were removed as part of this review: the inspection did not establish that any tracked item was obsolete or safe to delete.

### Engineering work recorded

Recent commits establish the initial display-coordinate and fractional-scale foundation:

- Exact rational scale representation and checked coordinate conversion.
- Separation of global logical coordinates from output-local physical pixels.
- Logical output placement based on scale-aware widths.
- Carrying the rational scale into render targets and pointer geometry.
- Registration of the Wayland fractional-scale manager global.
- Conservative initial preferred scale of 1.0 for newly created fractional-scale objects.

Relevant commits include:

- `9796c023` — document display coordinate foundation and migration status.
- `26f47d60` — define the Linux desktop interoperability contract.
- `6bebd8f1` and `d77db70d` — logical output placement and scale-aware placement tests.
- `8a0df018`, `9cce2778`, `2ae83868`, `d9ff4e36`, and `91f2b214` — fractional-scale protocol and manager-state work.
- `12cddaa8` — fix compositor fractional-scale state access and formatting.

### Verification

GitHub Actions run [44290834407](https://github.com/AleessandroCeccarelli/AureaOs/actions/runs/44290834407), for commit `12cddaa8fe35dff6d8b5e1eece5c5f70ce267424`, completed successfully on 2026-11-18.

All three jobs passed:

- Rust formatting.
- Workspace build and tests.
- Rust dependency security audit.

This is CI evidence for the checks configured in that workflow. It is not evidence of a successful boot on physical hardware, complete DRM/KMS scanout, a finished graphical desktop, or a bootable installation ISO.

### Known limitations and open work

- Fractional scaling is not end-to-end. Output-aware preferred-scale updates, surface buffer/viewport handling, cursor hotspots, clipping, hit-testing, and full renderer integration remain to be connected and validated.
- Physical DRM/KMS output and scanout still require testing on supported hardware.
- The Debian bootstrap and rootfs publication flow still require a complete end-to-end validation on the supported Fedora build host.
- The security audit dated 2026-11-13 records open hardening work. Its findings must not be treated as closed merely because the Rust dependency-audit CI job passes.
- The RustSec advisory affecting transitive `cgmath 0.18.0` remains documented in the security audit and needs an upstream-aware resolution; a successful audit job is not the same as a warning-free dependency report.

### Next engineering actions

1. Continue the fractional-scale integration by inspecting surface/output association and the relevant Smithay 0.7 APIs before changing behavior.
2. Define and test a consistent output-enter/leave and preferred-scale policy for toplevels, popups, and child surfaces.
3. Complete the buffer, viewport, cursor, clipping, and hit-testing integration in small, independently testable modules.
4. Preserve the explicit distinction between CI validation and physical-hardware validation in project documentation.
5. Revisit the open security-audit findings and schedule the Fedora rootfs/bootstrap validation when the build machine is available.

---

## Entry template

Use this structure for future entries:

- **Date and scope**
- **Repository/commit references**
- **Changes actually committed**
- **Verification performed and exact result**
- **Known limitations or regressions**
- **Next actions**

Do not record intentions as completed work.


## 2026-11-19 — Shared input and appearance API foundations

### Repository references

- Active branch: `main`.
- Input API grouping and compatibility export: `872229a`, `47dc3ea`.
- Shared pointer settings consumed by the compositor: `0cfeab6`.
- Appearance token roles and provider contract: `8ee9ff3`.
- Observable device, pointer and keyboard contracts: `aa2de73`, `0250c5c`, `b187afa`, `9446aca`.
- Updated keyboard/pointer policy and Human Interfaces status: `6033a4b`, `6aae467`.

### Changes committed

- Grouped technology-independent keyboard, pointer, device and vendor-extension contracts under `aurea_desktop_api::input`; retained the original keyboard import path as a compatibility re-export.
- Added device descriptors, verified capability snapshots and lifecycle notifications without exposing serial numbers or unrestricted raw input streams.
- Added shared pointer settings and semantic button identifiers; the compositor now consumes the API's pointer-settings type instead of maintaining a duplicate definition.
- Added observable contracts for keyboard-layout selection and pointer settings.
- Added an appearance contract for system/light/dark preference, resolved color scheme, accessibility appearance flags and semantic color roles mapped to the canonical Human Interfaces token names.
- Extended the vendor extension descriptor to match both vendor and product identifiers.
- Expanded the input policy to cover standard mouse behavior and the separation between normal Wayland application input and privileged global input services.

### Verification and limitations

- Repository files and commit contents were inspected after publication.
- No local Cargo build or test result is claimed for these commits. GitHub Actions is configured to run on source changes pushed to `main`; the result for this change set has not been independently confirmed in this entry.
- Fedora hardware validation has not been performed.
- These are public Rust contracts only. Device discovery/provider implementations, runtime layout switching, persisted pointer settings, application-facing IPC/portal transport, native Human Interfaces widgets, and the out-of-process vendor extension manager remain future implementation work.
- The appearance API identifies canonical semantic tokens; actual palette values remain owned by `aurea-human-interfaces/design-system/tokens/index.css`.


### Follow-up — API contract review

- Added observable device lifecycle, keyboard-layout selection and pointer-preference contracts, plus verified vertical-scroll capability.
- Added a focused test for capability deduplication and clarified that additional pointer buttons use normalized identifiers rather than raw kernel event codes.
- Updated the desktop interoperability contract to distinguish internal Rust contracts, the future native Human Interfaces library and the still-unimplemented application-facing IPC/portal layer.
- Updated the roadmap to reflect the new input and appearance API foundations.
- Latest API-focused code commits: `4022516` and `9d2bc14`. GitHub Actions is configured to trigger for source changes pushed to `main`; the run result is not yet confirmed here.


## 2026-11-19 — Shared display API foundation

### Scope and repository alignment

The existing architecture decisions were reviewed before implementation:

- [Multi-Monitor Policy](../architecture/MULTI-MONITOR-POLICY.md) remains authoritative: a maximum of four active outputs, while additional connected displays may remain discovered but disabled.
- [Display Intelligence](../architecture/DISPLAY-INTELLIGENCE.md) remains authoritative for automatic/manual management, fixed refresh, VRR capability checks, safe fallback, and the boundary between policy and compositor/DRM ownership.
- Existing compositor output identity, mode discovery, registration, logical layout, rational scaling, and four-output enforcement remain the implementation foundations. No duplicate compositor output manager or second DRM owner was introduced.

### API files added

Under `components/aurea-desktop-api/src/display/`:

- `modes.rs`: validated pixel dimensions and millihertz refresh values, preserving fractional rates without floating-point identity comparisons.
- `capabilities.rs`: stable display IDs, connector type, verified VRR range, HDR/bit-depth and physical-size capability fields, and public display descriptors.
- `layout.rs`: logical positions, rational per-display scale, enablement and primary-display layout.
- `policy.rs`: Automatic/Manual management, Automatic/Fixed/PreferVariable refresh preference, active refresh strategy and compositor policy context.
- `events.rs`: connected/disconnected/capability/configuration notifications.
- `manager.rs`: snapshot and transactional configuration contracts, validation of unique displays/layout placements, primary-display validity, and the four-active-display ceiling.

The existing `display.rs` remains the public entry point and retains the shared Night Light color controller. It now exports the display-management contracts without changing existing call sites.

### Safety and correctness rules

- Refresh rates are represented in millihertz; no universal 60/120/144 Hz assumptions are embedded.
- VRR ranges are optional verified capability data. Unknown means unavailable for policy decisions, not guessed from EDID or marketing labels.
- Four active displays are allowed; a fifth connected display can remain represented but disabled.
- Layout validation rejects duplicate IDs, missing/extra placements, and a primary display that is not enabled.
- Display configuration is specified as a single transaction. Implementations must restore the last known working configuration if applying a request fails and report rollback failure separately.
- Automatic policy receives explicit compositor/system context; it must not inspect application pixels or private content.
- These are API contracts only. The concrete provider, DRM/KMS mode-application path, VRR execution, persistence, Settings UI and application-facing IPC remain future work.

### Verification

- Source changes were committed directly to `main`.
- Focused Rust unit tests were added for mode validation, fractional refresh representation, VRR range validation, common scale values, automatic defaults, four-display acceptance, fifth-active-display rejection, duplicate IDs/layouts, and invalid primary-display selection.
- No local Cargo test/build is claimed. The Fedora hardware environment has not been used for validation in this change.


### Display API refinement

- Centralized the four-active-display product limit in `aurea-desktop-api::display::MAX_ACTIVE_DISPLAYS`. The compositor's existing `MAX_ACTIVE_OUTPUTS` symbol is now a compatibility re-export, so policy and enforcement cannot drift.
- Kept layout state (enabled/position/scale) in one `DisplayLayout` source rather than duplicating it in each mode request.
- Made the requested mode optional in Automatic mode so the policy engine can choose a discovered mode. Manual mode and Fixed refresh requests require an explicit mode for each enabled display.
- Represented unknown HDR capability explicitly instead of treating missing information as unsupported.
- Added validation that a non-empty active layout has exactly one enabled primary display, and that output IDs and layout placements are unique and complete.


## 2026-11-19 — Aurea Appearance, Aurea Palette, Aurea Form and Aurea Lumen

### Naming and ownership

- **Aurea Appearance** is the system-wide theme/accessibility appearance contract.
- **Aurea Palette** names the canonical semantic color system.
- **Aurea Form** names the canonical geometry and surface language.
- **Aurea Lumen** names environmental display color adaptation, distinct from Light/Dark theme selection.
- AureaOS Human Interfaces remains the single design system and owner of visual values. No parallel palette or token source was created.

### API changes

- Extended `appearance.rs` with the canonical palette identity, form profile and semantic geometry roles. The appearance snapshot now carries palette/form identity alongside the resolved color scheme and accessibility preferences.
- Added `appearance/lumen.rs` with Manual, Schedule, Solar, Ambient and Automatic modes; validated local-clock intervals; ambient-light thresholds with hysteresis; and clamped warmth preferences.
- Automatic Lumen policy prefers reliable ambient sensing and falls back to an available solar or local schedule. Sensor/time providers and policy execution remain separate implementation responsibilities.
- Existing `NightLightState` and the compositor color adjustment path remain in place as migration foundations; no unverified sensor or solar service is claimed.

### Documentation changes

- Added `docs/architecture/AUREA-APPEARANCE-AND-LUMEN.md`.
- Named Aurea Palette and Aurea Form in Human Interfaces documentation and the color-system specification.
- Reaffirmed `aurea-human-interfaces/design-system/tokens/index.css` as the sole source of visual values.

### Verification status

- Changes committed directly to `main`.
- No local Cargo build/test or completed CI result is claimed in this entry.
- Concrete Appearance provider, ambient sensor discovery, solar/time provider, Lumen coordinator, persistence, native Settings UI and application-facing IPC remain future implementation work.


## 2026-11-19 — Clarify early-stage and provisional design status

AureaOS is explicitly an embryonic project. The recent names and contracts for Aurea Appearance, Aurea Lumen, Aurea Palette and Aurea Form describe the current direction, not a final or immutable product specification.

Updated the architecture document, Human Interfaces overview, design-system README, color-system specification and roadmap to state that:

- Product names, palette values, shapes, visual treatments, functions, APIs, architecture and technology choices may change during development.
- Current visual tokens and API identities are provisional and must not be presented as frozen standards.
- Documentation and API contracts are not evidence of completed runtime services.
- Design proposals, experimental code, implemented API contracts, hardware-validated behavior and release-ready capabilities must be reported separately.
- A compatibility/versioning policy must be established before promising long-term stability to third-party developers.

No runtime code or visual token values were changed in this documentation clarification.


## 2026-11-19 — Lumen preference contract hardening and shell adapter

### Changes committed

- Made `LumenPreferences` fields private so callers cannot bypass the warmth range or replace validated schedule/ambient-threshold values through direct field mutation.
- Added focused getters and setters for enabled state, mode, warmth, schedule and ambient thresholds.
- Added a unit test covering preference updates through the public API.
- Replaced the shell's duplicate Night Light schedule/preference storage with a compatibility adapter over the shared `aurea-desktop-api::appearance::lumen` types.
- Preserved the existing shell-facing Night Light mode names and runtime call pattern during migration. The adapter maps Manual, CustomSchedule and SunsetToSunrise to the corresponding shared Lumen modes.
- The existing shell scheduler remains the source of the current schedule decision. No ambient sensor, solar provider, policy coordinator or new runtime behavior is claimed.

### Verification status

- Changes were committed directly to `main`.
- No local Cargo formatting, build or test result is claimed. The latest GitHub combined-status query did not return individual status checks for the API change.
- The next integration step is to replace the temporary compatibility mode/scheduler interface with an explicit Lumen policy decision once its provider contract is designed and tested.


## 2026-11-19 — Initial system time and time-zone API foundation

### Direction

Defined system time as a shared operating-system concern rather than a shell-only clock widget. Wall-clock time (UTC), monotonic time and local civil-time presentation are explicitly separate concepts. A time-zone identifier is not interchangeable with a fixed UTC offset.

### API changes

- Added `components/aurea-desktop-api/src/time.rs` and exported it from the crate.
- Added an IANA-style `TimeZoneId` shape validator; actual zone existence and rules remain the responsibility of the installed tz database provider.
- Added bounded `UtcOffset` and instant-specific `TimeZoneSnapshot` contracts with optional daylight-saving status.
- Added normalized Unix timestamp representation, synchronization policy/status/source snapshots, and clock/date display preferences.
- Added unit tests for identifier validation, UTC offset boundaries, zone snapshots, normalized timestamps, synchronization state and display defaults.

### Architecture and roadmap

- Added `docs/architecture/SYSTEM-TIME-AND-TIMEZONES.md`.
- Added roadmap milestones for Debian synchronization backend selection, provider integration, privileged configuration, tzdata, shell settings and hardware validation.
- The architecture proposes reusing one supported Debian time synchronization service (systemd-timesyncd or chrony after evaluation) rather than starting a competing NTP daemon.
- The IANA tz database must remain updateable; daylight-saving rules must not be hard-coded.
- Manual zone selection and offline clock operation remain required. Automatic zone detection must respect privacy and must not silently assume precise location.

### Verification status

These are experimental API contracts and documentation only. No daemon, NTP synchronization, tzdb resolver, privileged setter, persistence, shell clock widget or IPC implementation has been added. Cargo formatting, build and tests have not been run in this environment.


### Debian backend decision update

After reviewing Debian 13 documentation, selected `systemd-timesyncd` as the initial network time synchronization backend and added it to `debian/packages/base.list`. Debian documents it as the default client-side synchronization service; chrony remains a possible future alternative only if AureaOS develops requirements that justify it. The architecture specifies using `systemd-timedated` over D-Bus for privileged time/time-zone/RTC configuration rather than launching privileged shell commands from the UI. The installed `tzdata` package remains the source of updateable time-zone rules.

This package-manifest change has not yet been validated by a fresh Debian bootstrap.


## 2026-11-22 — First ISO generation and first real boot

### Context and operational decision

The day was spent producing the first complete Debian Trixie Live image and testing it on real hardware through Ventoy. The user confirmed the established workflow: copy the ISO into Ventoy's data partition and select it from the boot menu. Do not ask the user to prepare or reformat another USB drive.

The ISO build completed, but a successful image-generation process did not prove that the Live system could boot.

### Changes and Git references

Commits published to `main` during the session:
- `47fd5ed7` — synchronized local repository changes.
- `6bfc72c7` — integration tests for touchpad gestures reaching the desktop shell.
- `ad524e94` — formatting cleanup for gesture integration tests.
- `f38bd6ac` — serialize access to the persistent live-build workspace.
- `63c4bed8` — recover a stale live-build lock when resuming an interrupted build.

Relevant ISO files:
- `tools/iso/build-live-iso.sh`
- `tools/iso/build-live-iso-podman.sh`
- `tools/iso/hooks/010-build-aureaos.hook.chroot`
- `tools/iso/hooks/020-brand-boot-menu.hook.binary`
- `tools/iso/package-lists/aureaos.list.chroot`

The previous workspace used a nested, non-canonical `config/config` layout. The build script was updated to cautiously migrate the old workspace to Debian's standard layout while preserving chroot, stage markers, and caches when possible. The Podman wrapper now serializes workspace access with `flock`; the inner script only removes a stale lock during resume and only when it is a regular non-symlink file.

### Build result

The user's build log dated 22 November reports:
- recovery from the `lb chroot_devpts` issue;
- kernel and initramfs generation;
- successful execution of the GRUB branding hook;
- ISO creation through `xorriso`;
- final message `P: Build completed successfully`.

Reported artifacts:
- `~/Documenti/AureaOS.iso`
- `~/Documenti/AureaOS.iso.sha256`
- SHA-256: `1d929c0f99b665f1307191d9502f5c094a25452e32dd28615309feb8350c0704`

Fedora checks:
- ISO size approximately 350 MB.
- `sha256sum -c` returned `AureaOS.iso: OK`.
- `file` recognized ISO 9660 with a DOS/MBR boot sector and volume `AUREAOS_BLUE_TEST`.
- After installing `xorriso`, El Torito reporting detected GRUB BIOS (`/boot/grub/grub_eltorito`), EFI image (`/boot/grub/efi.img`), protective MBR, and GPT.

These checks establish image creation and boot structures, not a successful Live boot.

### First Ventoy boot: blocking failure

On the Lenovo laptop, Ventoy loaded GRUB and the Debian kernel `6.12.117+deb13-amd64`. Boot then stopped before the desktop started.

Messages captured in the user's photograph:
```
init: line 249: can't open '/scripts/live': No such file or directory
Kernel panic - not syncing: Attempted to kill init: exitcode=0x00000200
```

Verified conclusion: when init runs, `/scripts/live` is missing from the initramfs environment. The kernel panic follows the init failure. No GPU, Vulkan, or Smithay issue has been established; the system has not reached those stages. The exact cause must be found by inspecting the initramfs and chroot contents.

### Build-script findings

Reviewing the current files on `main` revealed that `build-live-iso.sh` automatically removes the entire `BUILD_ROOT` when `lb build` exits successfully. This also removes the workspace and caches after a successful build, defeating incremental reuse. Cleanup must become explicit and optional, not automatic.

The current final validation mainly checks that a non-empty ISO exists and that live-build exits successfully. It does not verify that the initramfs actually contains `/scripts/live`, the live-boot hooks, the expected kernel, or coherent GRUB parameters. A build must not be considered valid solely because an ISO file exists.

### Agreed decisions and priorities

- Starting Monday, 23 November, work returns exclusively to the AureaOS desktop on GitHub.
- No ISO builds and no ISO-script work during desktop sessions unless explicitly requested.
- Resume ISO investigation on Fedora on Friday, 27 November, starting with inspection of the existing image and initramfs; do not rebuild blindly.
- Continue using Ventoy by copying the ISO directly into its data partition.
- Keep changes on `main`; do not run remote Rust builds/tests. The user will run `cargo fmt`, checks, and tests on Fedora when agreed.
- Preserve strict modularity: one file, one responsibility; split build-script responsibilities into small modules with a small coordinator.

### Next actions

**Desktop work (from Monday):** continue desktop implementation and features; do not reopen the ISO workstream.

**Friday ISO session:**
1. Mount the existing ISO read-only and locate its kernel/initramfs.
2. Inspect the initramfs for `scripts/live`, live-boot hooks, and modules.
3. Inspect the chroot/workspace if needed; do not assume the workspace survived, because current cleanup removes it after success.
4. Verify `live-boot`, `live-config`, initramfs-tools hooks, kernel `boot=live` parameters, and GRUB entries.
5. Identify the cause before making a targeted pipeline change.
6. Preserve the workspace after success and make cleanup explicit.
7. Add pre-image and post-image validation, including actual initramfs inspection, while retaining complete diagnostic logs.

Do not report the desktop as complete or the ISO as bootable: the first real boot disproved the latter.


## 23 November 2026 — First operational desktop core

Implementation has started after the desktop-first architecture decision. The immediate goal is a real desktop base surface; Dock, Launcher, and pointer work remain deferred.

### DesktopManager and base surface

A modular compositor subsystem has been added under `components/aurea-compositor/src/desktop/`:
- `manager.rs`: coordinates theme/background policy and produces an immutable per-frame snapshot;
- `theme.rs`: Light/Dark desktop themes using AureaOS desktop tokens;
- `background.rs`: theme-derived or validated custom solid background;
- `surface.rs`: immutable desktop surface description;
- `snapshot.rs`: desktop data captured for one frame;
- `mod.rs`: module exports.

The compositor now owns the DesktopManager. Each RenderScene receives its snapshot, and the Vulkan surface compositor consumes the selected color for both empty and populated scenes, including readback and KMS release paths. The Vulkan clear pass remains generic and contains no theme policy.

The default surface uses the Light token `#edf4ff`; Dark uses `#0d1320`. Custom solid colors are validated and forced opaque.

### Verification status

Unit tests were added for themes, background validation, and DesktopManager behavior. No `cargo fmt`, `cargo check`, tests, or builds were run; Fedora validation remains with the user. The code is published but is not yet claimed to compile or work on hardware.

Wallpaper decoding, Vulkan texture upload, scaling/cropping, and persistence are not implemented yet. Pointer, Dock, and Launcher work remains deferred.

Main commits: `542f918`, `2826f2d`, `e0fb92e`.


## 23 November 2026 — First wallpaper asset-management layer

Implementation has begun for the selected `aureaos-midnight-flow.jpg` asset.

### Code added

Created `components/aurea-compositor/src/desktop/wallpaper/` with small, single-purpose modules:
- `asset.rs`: embeds the selected JPEG in the compositor binary and validates its JPEG frame header/dimensions without decoding pixels.
- `manager.rs`: owns the selected wallpaper, supports a safe no-wallpaper fallback, and produces immutable snapshots.
- `snapshot.rs`: carries the selected asset into a render-frame snapshot.
- `mod.rs`: exports the wallpaper subsystem.

Integrated the manager into `DesktopManager` and `DesktopSnapshot`; `RenderScene` now carries the selected asset reference supplied by the compositor.

### Explicit implementation boundary

The compressed JPEG bytes are embedded and available to the render scene, but the image is **not yet displayed**. JPEG-to-RGBA decoding, Vulkan texture upload, wallpaper draw pass, output fitting/cropping, and GPU cache remain outstanding. The Vulkan renderer still paints the solid fallback background.

No decoder dependency was added because the compositor's current Cargo dependency/lock state does not include one and the lockfile must be regenerated and checked on Fedora rather than edited speculatively.

### Verification

No Cargo formatting, build, tests, ISO work, or hardware validation were run. Fedora validation remains pending.


## 23 November 2026 — Per-output wallpaper layout and bounded RGBA pixels

Added two focused modules under `components/aurea-compositor/src/desktop/wallpaper/`:

- `layout.rs`: Fill, Fit, Stretch, and Center placement modes. Source crop and output-local destination rectangles are kept separate. Center may extend beyond output bounds and therefore requires output scissoring. Crop dimensions are clamped to the source image to handle extreme aspect ratios safely. Unit tests cover all modes and zero-sized inputs.
- `pixels.rs`: owned RGBA8 pixel buffer with exact byte-length validation, zero-dimension and arithmetic-overflow checks, and a 256 MiB decoded-image budget. Unit tests cover valid and invalid buffers.

The wallpaper module exports both APIs. These changes prepare the CPU-side image contract and per-output placement; they do not decode or display the JPEG yet.

The Vulkan wallpaper draw pass remains intentionally pending. The current scene still carries compressed JPEG bytes, not a sampled GPU texture. The empty-scene branch cannot draw the image until a decoder, validated RGBA buffer, dedicated wallpaper upload/cache, and texture binding exist. Do not add a placeholder draw call or reuse the Wayland SHM cache with fabricated surface IDs.

Commits: `7b15482`, `84b3f00`, `e1a1fdb`, `0be01da`, `2120f43`, `ce0509f`.

No Cargo formatting, compilation, or tests were run remotely. Fedora remains the validation environment.


## 23 November 2026 — Animated and dynamic wallpaper product decision

The wallpaper system now has a dedicated architecture specification at `docs/design/WALLPAPER-SYSTEM.md`.

The selected `aureaos-midnight-flow.jpg` remains the default wallpaper. Animated support is an expansion of the wallpaper engine, not a replacement of the chosen visual identity.

Planned capabilities:
- looping video or animated-image wallpapers;
- time-of-day dynamic collections (dawn/day/sunset/night);
- per-monitor assignment, shared wallpaper, and panoramic Span mode;
- muted audio by default;
- decoding outside the compositor render thread;
- pause when fully obscured, output disabled, session locked, or system suspended;
- bounded resolution, frame rate, queue depth, and memory;
- static fallback, reduced-motion preference, and power-saving policy.

The media backend is intentionally undecided. GStreamer and FFmpeg are candidates for a later comparison covering maintenance, Fedora/target distribution packaging, codec licensing, Rust APIs, hardware decode, and software fallback. Do not add a dependency or edit Cargo.lock before this review.

Implementation order remains: finish the static JPEG Vulkan path first; add PNG/BMP and Tile/Span; then define timestamped animation frames and deterministic playback state; finally integrate a media backend and power/session policy.

This is a product decision and documented plan, not an implemented feature. No Cargo build or tests were run.


## 23 November 2026 — Animated wallpaper playback contracts

The first backend-independent Rust contracts for animated wallpaper have been added and integrated into the existing wallpaper manager.

- `wallpaper/animation.rs` defines a bounded playback policy, desktop playback conditions, explicit pause reasons, timestamped RGBA frames, provider errors, and the `WallpaperFrameProvider` trait.
- `WallpaperManager` owns the playback policy and includes it in immutable wallpaper snapshots.
- The default policy caps animation at 30 fps, keeps audio muted, and pauses for obscured wallpapers, disabled outputs, locked sessions, system suspend, and power-saving mode. Reduced-motion forces a paused decision. The configurable frame-rate range is 1–60 fps.

This is a real API and policy integration, not a working GIF/video decoder. No animated frames are decoded or rendered yet. Provider implementations must run on a dedicated worker and must never block the Vulkan render thread.

Next: Fedora validation, finish the static JPEG decode/upload/cache/draw path, then add GIF/video providers and a bounded worker-to-renderer frame queue.

No Cargo formatting, build, tests, or hardware validation were performed.


## 23 November 2026 — Wallpaper output clipping geometry

### Changes committed

- Extended `WallpaperPixelRect` with an intersection operation that returns the visible portion of a rectangle inside an output clip rectangle.
- The operation uses saturating edge calculations and returns `None` when rectangles do not overlap.
- Added focused unit tests for clipping a centered 2560×1440 wallpaper to a 1920×1080 output and for disjoint rectangles.
- Kept the change inside the existing wallpaper layout module. No renderer, Vulkan resource, decoder, dependency, or Cargo.lock changes were made.

Commit: `99714264f53e280c1ad820cac92de9106f5a9172`.

### Verification status

- Source and tests were committed to `main`.
- Tests have **not** been executed yet. No build or formatting result is claimed.
- The new helper prepares output-local clipping geometry for the future Vulkan wallpaper draw path; it does not itself render or display the wallpaper.

### Next step

Continue a coherent block of desktop work before the next CI run, as agreed. On the next validation pass, run Rust formatting, workspace checks, and tests together. Fedora hardware validation remains necessary for actual monitor clipping and Vulkan presentation.


## 2026-11-23 — Desktop item model and optional visual stacks

### Scope and implementation

Added the first technology-independent desktop item model under components/aurea-desktop-api/src/desktop/:

- item.rs defines stable item identifiers, file/folder/application/Trash kinds, logical positions and file-extension extraction.
- preferences.rs defines the Trash visibility preference, opt-in stack grouping and per-item exclusions.
- stacks.rs derives deterministic visual groups by file category or extension. Only groups with at least two eligible files become stacks.
- manager.rs owns the in-memory desktop entries, rejects duplicate IDs, protects the system Trash entry and exposes derived stacks.
- mod.rs exports the desktop contracts; the API crate remains independent from compositor, shell and graphics backends.

The default model contains only the Trash entry. Stack grouping is disabled by default. Creating or disabling a stack does not mutate source entries or their file URIs.

Architecture rules and explicit implementation boundaries are documented in docs/architecture/DESKTOP-ITEMS-AND-STACKS.md.

### Verification and limitations

Focused unit tests were added for identifier validation, extension extraction, default Trash-only state, duplicate IDs, Trash protection, optional grouping, category/extension grouping and item exclusions. No Cargo command was executed in this session; these tests are added but not yet run. Persistence, filesystem discovery/watchers, MIME database integration, shell presentation, drag-and-drop, context menus, stack expansion and compositor rendering remain future work.

The latest CI run inspected before this change (45256450916, commit 99714264) had successful workspace build/tests and dependency audit, but Rust formatting failed in existing wallpaper/Vulkan files. Those formatting-only differences are corrected in this commit; the new Rust files must be checked by the next CI run.


## 2026-11-23 — Desktop Trash icon artwork

### Scope and visual decision

The existing `light-trash.svg` and `dark-trash.svg` under the Fides v5 folder family are folder/location illustrations, not standalone trash-can artwork. They remain unchanged. A separate desktop Trash symbol family has been added, following the established AureaOS dimensional style, blue-grey palette, soft highlights and shadow treatment.

### Source assets added

Under `aurea-human-interfaces/icons/source/system/trash/`:

- `light/trash-empty.svg`
- `light/trash-half.svg`
- `light/trash-full.svg`
- `dark/trash-empty.svg`
- `dark/trash-half.svg`
- `dark/trash-full.svg`

All six use a transparent 512 × 512 SVG canvas and share the same can geometry. Light/Dark changes the palette for theme contrast; Empty/Half/Full changes only the visible contents.

Added a dedicated README for the desktop Trash artwork and updated the icon-library inventory. The Fides folder sprite and its existing trash-folder symbols were not modified.

### Verification and limitations

- Source SVG markup was authored for all six variants and stored as editable masters.
- No local SVG renderer, export-size visual inspection, shell integration or runtime state selection was run in this change.
- The icons do not implement Trash behavior, file counting, restore, permanent deletion or desktop rendering.
- The state-to-icon mapping must be driven by the actual Trash contents model, independently of the active theme.

### Next actions

1. Inspect the six assets at the documented logical sizes and refine optical alignment if needed.
2. Connect theme selection to the Light/Dark artwork.
3. Connect Empty/Half/Full selection to the actual Trash state.
4. Add the desktop Trash item to the shell scene and wire open/context-menu behavior separately.

## 2026-11-23 — Desktop Trash artwork revised to an open-bin design

### User-approved visual direction

The first standalone Trash drafts incorrectly included a lid and did not make the contents states sufficiently clear. They have been replaced with an open, lidless waste bin inspired by the familiar Windows desktop metaphor.

- Empty: clearly open and visibly empty.
- Half: crumpled paper is visible inside the opening.
- Full: discarded paper protrudes above the rim.
- Light and Dark retain identical geometry and use theme-appropriate palettes.
- All six SVGs have transparent backgrounds.

### Files revised

All six files under `aurea-human-interfaces/icons/source/system/trash/{light,dark}/` were replaced. The artwork README now documents the lidless visual contract.

### Verification and limitations

The SVG source was updated and committed. No graphical rendering or multi-size visual inspection was run, and no runtime desktop integration is claimed. The contents-state selection still needs to be connected to the actual Trash model.

 
## 2026-11-23 — Desktop Trash redesigned as a translucent wire-mesh basket

At the founder's request and using the supplied photograph as visual reference, the desktop Trash artwork now depicts a realistic cylindrical wire-mesh wastebasket instead of the previous stylized solid bin.

- Slightly tapered body, rounded upper rim and permanently open top; no lid.
- Crossed metallic mesh with intentional transparency so the interior and contents remain visible, as in the Windows/macOS visual references.
- Metallic highlights, side shading and reinforced lower ring.
- Identical geometry for Empty, Half and Full; only the contents change.
- Separate Light/Dark metallic palettes and transparent canvas.

Updated all six SVG masters under aurea-human-interfaces/icons/source/system/trash/, the family README and the general icon inventory. Fides v5 folder/location Trash artwork remains untouched.

**Limitations:** no SVG renderer or desktop-size visual inspection was run. Shell integration and automatic theme/state selection are not implemented. No Cargo command was run.

**Next:** render at 24, 32, 48, 64 and 128 px; inspect mesh legibility and paper visibility; connect theme and actual Trash state; integrate the symbol into the desktop scene.

 
## 2026-11-23 — Desktop Trash approved direction: softly frosted translucent material

The founder supplied a more precise visual reference showing an empty and a full white wastebasket against a desktop wallpaper. This corrects the previous wire-mesh interpretation: **wire mesh is rejected**.

### Approved specification
- White satin/opal translucent material with soft diffusion.
- Gently tapered cylindrical body, rounded base and understated upper rim.
- Permanently open top; no lid.
- Interior and discarded objects remain perceptible through the walls, softly diffused by the material.
- When full, paper and small objects clearly protrude above the opening.
- Identical silhouette, proportions and lighting across Empty, Half and Full.
- Light/Dark changes material contrast only; state changes contents only.
- Transparent SVG canvas: wallpaper is not part of the icon.

Updated all six SVGs, the family README and the icon inventory. Fides v5 folder artwork remains separate and unchanged.

**Verification:** no SVG renderer or real-size visual inspection was run. The new direction has been translated into vector source, but still requires rendering and optical refinement before final approval. No runtime integration or automatic state selection has been implemented.

**Next:** render all six SVGs over light and dark wallpapers, verify translucent readability, refine the opening and protruding contents, then connect theme and actual Trash state in the shell.


## 2026-11-23 — Desktop Trash SVG structure fix

After the founder checked the GitHub preview, a malformed XML structure was found in all six Trash SVGs: an SVG group was left unclosed and the clipping group was duplicated. GitHub consequently displayed “Error rendering embedded code — Invalid image source”.

Regenerated all six Light/Dark × Empty/Half/Full assets with a simplified, balanced SVG group structure. Contents sit behind the translucent front wall; protruding pieces remain above the rim. Fix published in commit cf515c2.

**Still required:** reopen GitHub Preview and visually inspect all six assets on a real device. Do not claim visual approval until that preview has been checked.

 
## 2026-11-23 — Embed desktop Trash SVG assets in Aurea Shell

The six approved assets (Light/Dark × Empty/Half/Full) are now embedded at compile time in Aurea Shell via `include_bytes!`, without new dependencies and without maintaining a second copy of the SVG masters.

### New modules
- `components/aurea-shell/src/desktop/trash.rs`: semantic `TrashContentsState` enum (Empty, PartiallyFull, Full), defaulting to Empty.
- `components/aurea-shell/src/desktop/trash_artwork.rs`: resolves the SVG resource solely from `DesktopTheme` and `TrashContentsState`.
- `desktop/mod.rs`: `DesktopState` now owns the visual Trash state and exposes `trash_artwork_svg()` and `set_trash_contents()`.

Unit tests were added for all six mappings, UTF-8/root SVG structure, Light/Dark distinction and Full-state selection.

### Explicit boundaries
This change embeds the assets in the shell binary and connects them to its presentation model, but **does not yet draw the icon on the desktop**: the inspected repository does not yet contain a desktop widget/renderer consuming these bytes. No filesystem Trash service is connected yet; therefore the state starts Empty and must be updated by the future service. Fill thresholds, monitoring, restore and permanent deletion remain unimplemented.

**Verification:** no Cargo command, build or test was run in this phase. Tests have been added but remain to be executed on Fedora in the agreed test cycle.

### Corrected visual specification
An earlier historical entry describing wire mesh was an incorrect interpretation and is superseded: the approved design is an open, white satin/opal translucent basket, with contents visible through its walls. The six current SVGs, including the fuller Full state approved by the founder, are the canonical masters.


## 2026-11-23 — First filesystem Trash service

Added `components/aurea-desktop/src/trash/` with no new dependencies:
- `metadata.rs`: original-path encoding/decoding and .trashinfo format.
- `service.rs`: home Trash service at `~/.local/share/Trash`, supporting move, list, restore, explicit permanent deletion, empty, and snapshots.
- `mod.rs`: public service API.

Restore never overwrites an existing destination. Entries require both the file under `files` and matching .trashinfo metadata. Relative paths and paths already inside Trash are rejected. The visual Full threshold is configurable and defaults to 10 entries.

Added `ShellRuntime::set_trash_contents` as the shell-side update point for semantic state supplied by the service.

**Not wired yet:** the service is not instantiated by the composition root, no filesystem watcher or confirmation UI exists, and the Vulkan renderer does not yet draw the icon. This implementation currently targets the home Trash only: cross-filesystem moves return an error and remove the just-written metadata. Tests were added but not run; no Cargo command was launched.


## 2026-11-24 — Desktop rendering and media pipeline audit

### Repository baseline

- Audited `main` at commit `d9089aafaaa0b6470aa231f946aad1d8a7c5aede`.
- Confirmed the wallpaper metadata/pixel/layout/manager contracts, desktop item snapshot, Vulkan SHM texture upload/cache, textured-region draw path, native cursor composition, and home Trash service.
- Added [Desktop Rendering and Media Pipeline](../architecture/DESKTOP-RENDERING-AND-MEDIA-PIPELINE.md) as the detailed audit and implementation boundary.

### Findings

- Wallpaper selection and RGBA8 pixel storage exist, but the committed JPEG is not decoded and no wallpaper texture is uploaded or drawn.
- Vulkan currently clears to the desktop fallback color, then draws mapped Wayland surfaces and the native cursor. Desktop item snapshots are present in `RenderScene` but are not yet converted into native drawable elements.
- The existing SHM texture cache is keyed to Wayland surface IDs and should not be misused for wallpaper or desktop assets.
- Pointer/input foundations already exist; the first visible desktop should layer wallpaper underneath windows and cursor.
- Trash SVG artwork is embedded by the shell, but there is no compositor SVG rasterizer or desktop-icon rendering path.
- Debian 13 provides libjpeg-turbo/TurboJPEG packages. librsvg is available too, but its runtime dependency chain includes Cairo, GLib and Pango; this is not justified for the current static icon requirement.

### Decisions and direction

- Keep decoding off the compositor render thread.
- Evaluate a small Aurea-owned safe adapter over Debian's TurboJPEG system library for the committed JPEG wallpaper; verify ABI, Fedora package availability, packaging and error handling before adding code.
- Keep the Vulkan renderer as the exclusive GPU API and reuse its existing image/upload/draw primitives with a dedicated bounded desktop-asset cache.
- Preserve approved SVG masters as canonical editable artwork; evaluate generated raster variants for static desktop assets instead of adding a heavyweight runtime SVG stack.
- Treat Aurea Background, Aurea Media, Desktop Items, Aurea Files and Aurea Shell as clear service boundaries. Begin with in-process/on-demand workers where possible; do not create always-running daemons without measured need.
- Continue using FreeDesktop/XDG and Wayland conventions rather than replacing Linux facilities.

### Verification and limitations

- Source files and Debian package metadata were inspected.
- No source code was changed in this audit entry.
- No Cargo formatting, build, check, test, CI, ISO generation or physical-hardware validation was performed.
- TurboJPEG is a candidate pending packaging and ABI review, not yet an implemented dependency.

### Next actions

1. Inspect the existing Vulkan image/upload abstractions and output-coordinate conversion before designing the wallpaper texture cache.
2. Verify TurboJPEG development/runtime packages on both Debian Trixie and the Fedora development host.
3. Implement the isolated decoder and worker boundary with strict memory limits.
4. Connect wallpaper texture upload and draw before Wayland surfaces and cursor.
5. Add raster desktop artwork and native desktop item draw elements.
6. Only after a visible wallpaper and pointer are ready, request the agreed Fedora build/test cycle.


## 2026-11-24 — File Intelligence and metadata architecture

### Scope and repository reference

- Active branch: main.
- Architecture decision: `docs/architecture/FILE-INTELLIGENCE-AND-METADATA.md`.
- The design is inspired by the useful ideas found in modern desktop search/indexing systems, but does not reproduce their UI, private databases or service implementations.

### Decisions recorded

- Separate file cataloguing, metadata extraction, folder presentation state and desktop icon placement.
- Define Aurea File Intelligence as the shared catalog/query capability for Fides, Quaero and other authorized clients.
- Keep Fides responsible for file operations/navigation, Quaero responsible for search experience/ranking, Desktop responsible for desktop-specific arrangement, Aurea Media responsible for bounded media extraction, and the compositor responsible only for Wayland/Vulkan composition.
- Do not create a hidden sidecar file in every folder as a clone of macOS Finder metadata.
- Keep filesystem facts, format-native metadata, extracted metadata, user-authored metadata, presentation state and thumbnail cache as separate classes.
- Use XDG/FreeDesktop conventions where applicable, without depending on GNOME Tracker, KDE Baloo, GTK or Qt.
- Prefer a local embedded database; SQLite is the leading candidate, but its FTS5 availability, Rust binding, MSRV, packaging, WAL and recovery behavior must be verified on Debian 13 and Fedora before adding a dependency.
- Use filesystem notifications as low-latency hints, not as the source of truth. A bounded reconciliation scanner remains authoritative.
- Do not start with privileged fanotify, a mandatory daemon, vector database, cloud index or AI runtime.
- Preserve future extension points for semantic search and other content providers without burdening the initial desktop.

### Verification and limitations

- Repository main was checked at `fae8d8ba3f2f0be6d3ae34e91adb6974e10054e5` before the documentation change.
- The architecture document was committed directly to main as `f232abc0bb4f720b1238fff0da5bfd9c1582524f`.
- No source code, Cargo dependency or runtime behavior was changed.
- No build, formatting command, test, or CI workflow was run.
- The database, watcher and metadata extraction technology choices remain subject to compatibility and resource-budget validation.

### Next actions

1. Audit existing file, desktop, search and media modules for overlapping ownership before defining or implementing File Intelligence APIs.
2. Verify Debian 13 and Fedora SQLite/FTS5 packaging and Rust binding options.
3. Compare stable notify releases with a small native inotify adapter, including overflow, watch-limit and reconciliation behavior.
4. Define stable file identity, metadata provenance, freshness and permission-aware query contracts.
5. Keep implementation modular and defer expensive extraction until it is justified by user-visible features.


## 2026-11-24 — Desktop module audit and File Intelligence follow-up

### Scope and repository references

- Active branch: `main`.
- File Intelligence architecture: `docs/architecture/FILE-INTELLIGENCE-AND-METADATA.md`.
- Italian counterpart: `docs/it/ARCHITETTURA-INTELLIGENZA-FILE-METADATI.md`.

### Findings from source inspection

- `aurea-desktop-api::desktop` already provides technology-independent contracts: DesktopItem, DesktopItemId, DesktopItemKind, DesktopPosition, DesktopPreferences, DesktopManager, visual grouping and hit-testing.
- The Desktop model is currently in-memory. Position persistence and a service that discovers and refreshes desktop file entries are not implemented yet.
- `DesktopItemKind` distinguishes File, Folder, Application and Trash; file/folder references are URI strings. `file_extension()` derives an extension from a path, but this is not MIME identification and must not become the authoritative file-type source.
- `DesktopItemsSnapshot` copies owned entries into the compositor. The native Vulkan path does not yet draw desktop icons.
- `components/aurea-desktop/src/trash` implements a FreeDesktop-compatible home Trash service, but it is not yet connected to the composition root, a filesystem watcher or confirmation UI.
- The current workspace has the `aurea-desktop` composition-root/services crate, but no independent Fides or Quaero application crates yet.
- No general file catalog, global search engine or metadata/thumbnail extraction service was found in the inspected paths. Those responsibilities must not be duplicated inside existing Desktop modules.

### Decisions and direction

- Aurea File Intelligence owns the shared local catalog; Fides owns file navigation/operations, Quaero owns search experience, Desktop owns desktop arrangement, Aurea Media owns bounded extraction, and the compositor owns rendering/input only.
- No per-directory hidden metadata file in the style of Finder's `.DS_Store` as AureaOS's canonical format.
- Keep filesystem facts, format-native metadata, extracted metadata, user-authored metadata, presentation state and thumbnail cache separate.
- SQLite is the leading initial catalog candidate, not an approved dependency. Verify FTS5, Rust binding, MSRV, Debian/Fedora packaging, WAL and recovery behavior first.
- Filesystem notifications are low-latency hints; scanning and reconciliation remain authoritative.
- No permanent daemon, privileged fanotify, full-disk indexing, vector database or AI runtime in the first phase.
- Every new Markdown architecture document must be published in English and in its corresponding Italian version under `docs/it/`.

### Verification and limitations

- Inspected Desktop API contracts, DesktopManager, DesktopItemsSnapshot, Trash service, composition root and related architecture documents.
- No Rust source files were modified.
- No Cargo command, formatting, build, test or CI workflow was run.
- This is a static repository audit; it does not establish runtime behavior or Fedora performance.

### Next actions

1. Verify SQLite/FTS5 availability and Debian 13/Fedora packages.
2. Compare notify with native inotify, including overflow, watch limits, network filesystems and reconciliation.
3. Define stable identity, metadata provenance, freshness and permission contracts.
4. Design public contracts without adding dependencies or duplicating existing Desktop API.
5. Continue with small modules and bilingual documentation (English/Italian).


## 2026-11-24 — AureaOS platform foundations

### Direction confirmed
- Platform baseline: Debian 13 Trixie, Btrfs, Rust, Wayland, Smithay and Vulkan.
- Quaero is the system-wide search and conversational entry point, with a ChatGPT-like experience integrated through Aurea Intelligence.
- Aurea Intelligence owns orchestration, local context policy, provider boundaries and authorization. ChatGPT is an optional explicit remote provider; essential search and desktop behavior remain local/offline-capable.
- Aurea Passkey is a system authentication capability spanning login/session unlock, privilege authorization and documented application APIs, with FIDO2/CTAP, PAM integration and optional TPM 2.0.
- Btrfs layout, update transactions, snapshots and rollback must be designed together. Snapshots are not backups; nested subvolumes are snapshot boundaries.
- Keep credentials outside the compositor and ordinary application configuration. Preserve recovery and fallback; no proprietary cryptographic protocol.
- Preserve File Intelligence ownership and modular boundaries.

### Technology research
- Debian 13 packages SQLite runtime/development files; Fedora package metadata reports FTS5 enabled. Debian target builds still need an explicit FTS5 capability check.
- notify 9 documents Linux inotify support and CC0 licensing, but current MSRV is Rust 1.91. Inspected workspace manifests use edition 2024 and do not declare rust-version. Do not add notify 9 until the project's MSRV policy is set; evaluate native inotify too.
- Rust passkey crates differ in protocol scope. passkey-rs offers WebAuthn client and CTAP2 authenticator components; Linux FIDO libraries have distinct transport/licensing trade-offs. No dependency selected.
- Btrfs snapshots are subvolume-based, do not recurse across nested subvolumes and are not backups.

### Repository and verification
- Branch: main. HEAD before this documentation update: 9332e26cdb936695c7dc85e6dee3f347820eb129.
- Added bilingual platform architecture documents.
- No Rust source, dependencies, Cargo manifests or runtime behavior changed.
- No Cargo fmt/check/test/build or CI was run.

### Next actions
1. Design Btrfs subvolume and rollback model.
2. Define Aurea Passkey threat model and system broker/PAM boundaries.
3. Define Aurea Intelligence provider and action-authorization contracts.
4. Complete SQLite FTS5 and watcher compatibility checks before adding dependencies.


## 2026-11-24 — Btrfs layout and recovery design
- Added bilingual proposed Btrfs layout/recovery documents.
- Proposed flat sibling subvolumes to make snapshot boundaries explicit and avoid nested-subvolume omissions.
- Kept /var/lib within the rollback unit so dpkg and system service state remain aligned with installed system files; proposed preserving /home and diagnostic logs.
- Defined target update transaction lifecycle and recovery requirements. Bootloader integration, encryption defaults, retention, cache policy and swap/hibernation remain open.
- No installer, snapshot manager, update transaction or rollback code was implemented. No Cargo commands or tests were run.


## 2026-11-24 — Boot architecture decision: systemd-boot

### Approved decisions
- systemd-boot is the primary boot manager for AureaOS Roma 1.0.
- UEFI is mandatory; Legacy BIOS/CSM is unsupported. The initial target is modern amd64 PCs, approximately from the last decade, subject to actual firmware and hardware qualification.
- Secure Boot is disabled for Roma 1.0. Microsoft signing/certification costs are not a first-release prerequisite. Future Secure Boot support remains a later objective, not a current guarantee.
- GRUB 2 is retained only as a possible future alternative if a demonstrated requirement emerges; it is not installed or configured as the default boot manager.
- UKI is the selected boot-artifact direction, subject to validation of Debian 13 kernel/initramfs integration.
- The ESP stores systemd-boot and UKIs. The disk layout remains one ESP plus one Btrfs partition for the entire installed OS; no XBOOTLDR partition is planned.
- A 1 GiB ESP is a provisional sizing target only. Final sizing requires measurements including current, previous and Aurea Recovery UKIs.

### Technical research
- Debian 13 Trixie provides systemd-boot 257, systemd-boot tools and the systemd-ukify package.
- systemd-boot supports UEFI only, loads UKIs from ESP or XBOOTLDR, and implements boot counting with fallback behavior.
- The boot manager cannot itself determine whether the complete desktop/session is healthy. AureaOS needs an explicit userspace success signal and update lifecycle.

### Documentation and scope
- Added bilingual systemd-boot/UKI architecture documents.
- Replaced the earlier speculative Btrfs snapshot/subvolume proposal with the approved single-Btrfs-partition and Aurea Time-only backup direction.
- Updated platform foundations, README, Technology and Vision references in English and Italian where applicable.
- No Rust source, package manifest, installer, boot configuration or runtime behavior was changed.
- No Cargo command, build, test or CI was run.

### Next engineering work
1. Audit the existing Debian bootstrap and package manifests for boot-related assumptions.
2. Prototype UKI creation from Debian kernel packages and initramfs-tools.
3. Measure ESP consumption and validate the provisional capacity target.
4. Specify boot-success signaling, boot counting, rollback-to-previous-artifact and cleanup ownership.
5. Design the independent Aurea Recovery boot image and external recovery-media path.
6. Keep Secure Boot disabled in Roma 1.0 acceptance criteria; evaluate signing and key management separately for a future release.

### Superseded Btrfs proposal
The earlier 24 November draft proposing sibling subvolumes, snapshots, snapshot-based update transactions and rollback was not an approved project decision. It is superseded. AureaOS uses Btrfs as its selected filesystem, with one Btrfs partition for the installed OS. Aurea Time is the sole planned backup/restore system; native Btrfs snapshots are not part of the planned workflow.


## 2026-11-24 — Secure Boot readiness strategy

- Confirmed that Roma 1.0 keeps Secure Boot disabled, but the UKI pipeline should be structured for a future signed profile.
- Research found Debian 13 Trixie packages a Microsoft-UEFI-CA-signed shim, Debian-signed shim helpers and systemd-boot, plus systemd-ukify for UKI generation/signing.
- A future owner-enrollment path using firmware DB or shim MOK may avoid AureaOS needing its own Microsoft signing relationship, but the exact shim → systemd-boot → UKI trust path must be proven on real firmware. Do not promise universal compatibility.
- Documented separate build/signing stages, public/private key separation, protected production key custody, signature verification, rotation/revocation, SBAT/DBX awareness and recovery requirements.
- No signing keys were generated; no firmware state or package manifests were changed. No build, test or CI was run.
- Updated the English and Italian systemd-boot/UKI architecture documents.



## 2026-11-24 — Debian-first dependency governance

### Decision

Established a mandatory dependency register at `docs/development/DEPENDENCY-REGISTER.md`, with its Italian counterpart at `docs/it/REGISTRO-DIPENDENZE.md`.

Debian GNU/Linux 13 (Trixie), amd64, is explicitly the target platform and compatibility authority. Fedora remains the development host only. A dependency must not be accepted because it is convenient or available on Fedora.

### Register contents

- Defined dependency states: Adopted, Approved, Candidate, Conditional, Rejected and Needs review.
- Required evidence includes exact Debian package/source package and versions, Fedora development packages, Rust crate/MSRV, ABI/API and linking, amd64 support, SPDX licensing, upstream maintenance, security advisories, transitive/runtime footprint, evidence links and actual validation.
- Recorded current architectural technologies without claiming that package integration or hardware validation is complete.
- Added focused records for TurboJPEG, librsvg, SQLite, filesystem notifications, passkey/FIDO2 and the Smithay-to-cgmath RustSec concern.
- Kept TurboJPEG as a Candidate pending exact Debian API/ABI, Rust binding and Fedora package verification. librsvg is rejected for the current core runtime scope; SVG masters remain canonical and raster assets are the planned direction.
- Recorded SQLite and filesystem notifications as not yet approved, and passkey dependencies as unselected.
- Added a mandatory approval checklist and a change procedure requiring both language versions and relevant diary/architecture updates.

### Scope and limitations

This is a governance/documentation change. It does not add, remove or upgrade software dependencies, modify Cargo manifests or Cargo.lock, or claim that all listed packages have been tested. Package/version fields not yet evidenced are explicitly left unverified.

No Cargo formatting, build, check, tests or CI were run as part of this change.

### Ongoing rule

Before proposing or adding any dependency, consult and update the register. Debian compatibility wins over development-host convenience. Never infer Debian compatibility from Fedora compilation or Ubuntu CI.



## 2026-11-24 — Debian JPEG decoder compatibility review

### Findings

- Verified Debian 13 Trixie amd64 packages `libturbojpeg0` and `libturbojpeg0-dev` at version `1:2.1.5-4`.
- Verified Fedora development package names `libjpeg-turbo-devel` and `turbojpeg-devel`; Fedora package streams surfaced during review use libjpeg-turbo 3.1.x, so Fedora's native headers do not represent Debian's 2.1 target.
- Verified that `turbojpeg-sys` 1.1.1 and 1.2.0 expose TurboJPEG 3.0 `tj3*` APIs. The current high-level `turbojpeg` crate depends on this binding family. This is not a valid assumed match for Debian 13's TurboJPEG 2.1 library.
- Evaluated `zune-jpeg` 0.5.15 as an alternative candidate: pure Rust, one declared normal dependency (`zune-core ^0.5.1`), MSRV 1.75, license `MIT OR Apache-2.0 OR Zlib`.

### Register updates

- Updated both language versions of the dependency register.
- TurboJPEG Rust bindings are not accepted for the Debian system-library path unless a verified 2.1 binding or maintained Aurea-owned FFI adapter is established.
- `zune-jpeg` is recorded as Candidate only. Its full dependency/security review, limits and failure behavior remain open.

### Scope

No dependency was added to Cargo manifests or lockfiles. No decoder implementation was written. No build, format, check, tests or CI were run.


### Additional decoder review — 2026-11-24

- Found upstream issue #496: zune-jpeg 0.5.15 can decode valid baseline JPEG images with incorrect colors; upstream identifies a fix in 0.5.16-rc3.
- 0.5.16-rc3 is a pre-release and is not accepted as a production dependency.
- Updated both dependency registers: zune-jpeg 0.5.15 is now explicitly blocked; wait for a stable fixed release and repeat security, compatibility and decoder-boundary review.
- No decoder dependency or source code was added. No builds or tests were run.



## 2026-11-24 — Native libjpeg-turbo adapter evaluation

### Findings
- Confirmed Debian 13 Trixie amd64 packages `libturbojpeg0` and `libturbojpeg0-dev` at version `1:2.1.5-4`.
- Confirmed Fedora development package names `libjpeg-turbo-devel` and `turbojpeg-devel`; reviewed Fedora streams report the 3.1.x series.
- Upstream 3.x release notes describe a TurboJPEG 2.x compatibility wrapper. A small Aurea-owned FFI boundary around the shared legacy API is therefore plausible, without relying on Rust bindings that expose the 3.0 `tj3*` API.
- Exact compatibility is not yet demonstrated: symbols, headers, ABI and linking must be checked against the actual Debian 2.1 and Fedora 3.1 installations.
- `jpeg-decoder` 0.3.2 is a pure-Rust alternative, but upstream describes it as maintenance-mode; Debian Trixie packages 0.3.0.
- `zune-jpeg` 0.5.15 remains blocked due to its color regression; the identified fix is only in the 0.5.16-rc3 pre-release.

### Provisional decision
- Record libjpeg-turbo through an Aurea-owned FFI adapter as the leading Candidate, not Approved.
- Confine unsafe FFI to one small module; the rest of the project receives only a safe `WallpaperPixels` result.
- Decode on a bounded worker, never on the render thread.
- Before implementation, verify ABI/symbols on both distributions, the runtime package in the Debian base, license/advisories, error handling and memory limits.

### Status and verification
- Updated the EN/IT dependency registers with this evaluation and the remaining approval checks.
- No dependency was added to Cargo.toml or Cargo.lock; no Rust source was modified.
- No package was installed; no build, format, check, test or CI was run.
- Session starting HEAD: `66a0b6ce0657a8be35baa169ae1cb6d3fa5420eb` on `main`.

### Next action
Verify the actual legacy TurboJPEG headers and exported symbols supplied by Debian 13 and Fedora packages, then define the adapter contract without connecting it to the Vulkan renderer yet.