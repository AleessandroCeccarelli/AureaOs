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


### Local ABI probe prepared

- Added `tools/dependencies/jpeg/turbojpeg-abi-probe.c`, a small C diagnostic using the legacy TurboJPEG API: initialization, header read, RGBA decode and handle cleanup.
- Added `tools/dependencies/jpeg/README.md` with compile/run instructions for Fedora and Debian 13.
- The probe caps compressed input at 64 MiB, each dimension at 16384 pixels and RGBA8 output at 256 MiB; it explicitly rejects CMYK/YCCK in the initial scope and verifies opaque alpha.
- The source received a static review and was corrected to initialize the colorspace with the legacy `TJCS_DEFAULT` value.
- The probe has NOT been compiled or executed and therefore does not prove ABI compatibility. No package was installed and no Rust dependency was added.
- Next: when the user is back on Fedora, compile and run the probe against the committed wallpaper; then repeat the same check in a Debian 13 Trixie environment before approving the adapter.


## 2026-11-24 — Aurea Media Service architecture decision

### Decision
- AureaOS should own its media service: API, limits, validation, error handling, scheduling, dispatch and integration.
- Do not reimplement the JPEG codec from scratch. Initially use libjpeg-turbo/TurboJPEG behind a small private FFI boundary, if the Fedora and Debian compatibility checks pass.
- First phase: an in-process Rust `aurea-media` crate, not a separate daemon. Revisit IPC/helper-process isolation when multiple clients exist or untrusted-image exposure justifies it.
- The service returns CPU-owned RGBA8 pixels and has no dependency on Vulkan, textures or desktop scene composition.
- Decode on a background worker, never on the render thread.
- Full proposal: `docs/architecture/AUREA-MEDIA-SERVICE.md`.

### Probe correction
- Direct inspection of Debian TurboJPEG 2.1.5 headers showed that `TJCS_DEFAULT` and `TJCS_UNKNOWN` are not defined in the legacy 2.1 colorspace enum.
- Corrected the C probe to initialize the output colorspace integer to `-1`; `tjDecompressHeader3()` writes the detected value on success.
- Confirmed `TJFLAG_LIMITSCANS` exists in 2.1.5.
- The probe has not been compiled or executed. ABI compatibility is not yet claimed.

## 2026-11-24 — Cross-device sharing direction: Aurea Link and Aurea Share

### Product decision
- Add cross-device continuity as a product direction: AureaOS should exchange files, photos, audio, video, documents, links and text with Android, iOS/iPadOS, macOS and other desktops.
- Separate **Aurea Link** (pairing, peer discovery, secure local transfer and receive lifecycle) from **Aurea Share** (system Share action and destination registry).
- Distinguish local device handoff from sending content to a person through Telegram/WhatsApp. AureaOS may open an installed supported app with user-selected content; the user confirms the final recipient/action there. Do not automate private messaging APIs.
- Initial target is local-network file/link/text transfer, not live media streaming, cloud relay or message/notification synchronization.

### Technology conclusion
- LocalSend Protocol v2.x (current v2.2 spec reviewed) is the first interoperability candidate; do not invent a proprietary wire protocol for the first milestone.
- Prefer authenticated HTTPS app-to-app transfers, explicit receiver approval, integrity checks and bounded streaming. Discovery is not authentication. Provide manual IP/QR fallback.
- Existing LocalSend clients provide a cross-platform reference for Linux, macOS, Windows, Android and iOS. They are a compatibility baseline, not an AureaOS UI or mandatory bundled application.
- KDE Connect is an optional secondary ecosystem. AirDrop/AWDL/OWL remains experimental research and must not be promised or required.
- Future mobile companion: Android Sharesheet and iOS Share Extensions. macOS companion can be considered later.

### Security and architecture
- Explicit pairing and consent; unknown senders require approval.
- Show sender, files, size and destination; no silent overwrite or auto-open.
- Validate paths/metadata, enforce size/session/rate/disk limits, stream to temporary files, verify hashes and atomically publish.
- No silent downgrade to unauthenticated HTTP; no unrestricted always-on LAN server.
- Start with small Rust libraries and session-scoped listeners, not a monolithic daemon.
- Keep networking/transfer outside compositor, render thread, Aurea Media decoder and GStreamer pipeline.

### Documentation
- Added docs/architecture/AUREA-LINK-SHARE.md and docs/it/ARCHITETTURA-AUREA-LINK-SHARE.md.
- Updated universal technology strategy, EN/IT dependency registers, product catalog and README entry points.
- No Cargo dependency, manifest, lockfile or Rust source changed. No build, format, check, test or CI run.

## 2026-11-24 — Aurea Share Online: direct service integrations

- Clarified product intent: the user wants to share a currently viewed local photo/PDF/video/document from Fides or a first-party viewer to online services without manually opening the destination app and re-selecting/attaching the file.
- This is distinct from Aurea Link device-to-device transfer. Aurea Share now has an explicit online-service adapter path.
- Researched official API surfaces: Telegram Bot API vs Telegram API/TDLib; WhatsApp Cloud API business-only scope; Facebook Page publishing; Instagram Professional publishing; Reddit OAuth/access and commercial-use approval; YouTube authenticated channel upload.
- Decision: use official APIs only for actions/account types they permit; otherwise official share/deep link or registered desktop app handoff. Never use private endpoints, scraping or silent automation.
- Defined provider-neutral capability descriptors (content kinds, actions, account class, OAuth scopes, MIME/size limits, quotas, app review, commercial restrictions, confirmation requirement).
- Added online integration architecture in EN/IT and linked it from strategy, Aurea Link/Share docs and README. Dependency registers updated.
- No Rust/Cargo code, dependency, OAuth registration or build/test changed. API access approval remains unresolved and must be reviewed before implementation.

## 2026-11-24 — Aurea Share account model

- Clarified that AureaOS serves everyday users, children, older adults, creators, professionals and businesses without assuming a user persona.
- Account linking is optional and user-initiated; no online account is required during OS setup.
- Added an account-manager requirement with multiple accounts where provider rules allow, account class, permissions, available capabilities, disconnect and revocation.
- Available actions depend on account, scopes, provider review, region, content and current limits.
- Share chooser remains usable without linked accounts through supported desktop handoff.
- Updated EN/IT Aurea Share online architecture. No code or dependency added.


## 2026-11-24 — Pointer motion allocation reduction

- Audited the existing mouse/touchpad input path before changing behavior.
- Removed the intermediate output-geometry vector from pointer motion and changed output-bound constraint evaluation to a single pass over enabled outputs.
- The constraint helper no longer collects its iterator into a temporary vector; it returns immediately when the requested point is already inside an output and otherwise tracks only the nearest valid point.
- Added tests rejecting non-finite requested pointer coordinates. Existing gap, edge, negative-origin and mixed-scale coverage remains in place.
- No pointer semantics, input routing, output layout, dependencies or public API were intentionally changed.
- Changes are committed on main. No Cargo fmt/check/test/build or CI was run; local Fedora verification remains required. No ISO was generated.


## 2026-11-24 — JPEG SOF metadata validation

- Hardened the embedded wallpaper JPEG metadata parser: a Start Of Frame segment must now declare a non-zero component count and its segment length must exactly match the JPEG SOF structure (8 bytes plus 3 bytes per component).
- Added unit coverage for a minimal valid one-component frame, a truncated component table and a zero component count.
- This is metadata validation only; it does not decode pixels or approve the TurboJPEG FFI backend. The Fedora and Debian 13 ABI probe remains a required gate before adding the Rust decoder.
- No dependency or manifest changed. No build, format, check, test, CI or ISO was run.


## 2026-11-24 — First Vulkan wallpaper texture resource

- Added `VulkanWallpaperTexture` in `components/aurea-compositor/src/vulkan/wallpaper_texture.rs`. It consumes the validated `WallpaperPixels` RGBA8 contract, allocates an optimal-tiled sampled image, uploads through the existing host-visible staging batch, and transitions the image from UNDEFINED → TRANSFER_DST → SHADER_READ.
- The texture owns a shared Vulkan image and an image view, so its resource lifetime is independent of a stack frame. It is exported by the Vulkan module.
- Exposed the existing validated `WallpaperPixels` type from `desktop` so the GPU resource can consume it without coupling the media decoder to Vulkan.
- This is the GPU upload/resource milestone only. It is not yet cached by wallpaper identity, inserted into the render scene, or drawn below desktop surfaces. The current constructor waits for its one-time upload; it must be called from a resource-preparation/background path, not a latency-sensitive render callback.
- No JPEG decoder or dependency was added. No Cargo fmt/check/test/build or CI was run; no ISO was generated.


## 2026-11-24 — Asynchronous Vulkan wallpaper upload

- Refactored `VulkanWallpaperTexture` so the upload path is available as `submit_pixels`: it submits the staging transfer and returns a fence-backed `VulkanPendingUpload<Arc<VulkanWallpaperTexture>>` without waiting for GPU completion.
- Kept `from_pixels` as a blocking convenience wrapper for controlled resource-preparation paths; it now delegates to the asynchronous implementation and waits only there.
- The pending upload retains the staging buffer and texture until the fence completes, matching the ownership model already used by SHM surface uploads. No render-scene integration or cache has been added yet.
- No dependencies/manifests changed. No Cargo fmt/check/test/build or CI was run; no ISO was generated.


## 2026-11-24 — Bounded Vulkan wallpaper texture cache

- Added `VulkanWallpaperTextureCache`, keyed by a wallpaper asset identifier plus an explicit revision number, never by Wayland surface IDs.
- The cache retains one ready selected texture and up to two pending fence-backed uploads, with a 256 MiB aggregate pending-upload budget. Uploads are polled without blocking; only a completed upload matching the currently selected key becomes visible.
- Integrated the cache into the per-device `VulkanSurfacePipelineCache` lifecycle and poll it during frame preparation. Added preparation/access/clear entry points for the decoder and upcoming wallpaper draw pass.
- The decoder does not yet produce `WallpaperPixels` for the selected asset, so the cache is infrastructure only: no texture is populated and the background is not drawn yet. The draw pass and scene integration remain outstanding.
- No dependencies/manifests changed. No Cargo fmt/check/test/build or CI was run; no ISO was generated.


## WALLPAPER OUTPUT DRAW GEOMETRY — 2026-11-24

- Added `desktop/wallpaper/draw_geometry.rs` with `WallpaperDrawGeometry` and `WallpaperUvRect`.
- It converts the existing Fill/Fit/Stretch/Center layout into an output-clipped destination and normalized source UV range. Center clipping adjusts UVs rather than producing negative Vulkan scissor offsets; Fill preserves the centered source crop.
- Added unit tests for all four modes and a zero-sized output/non-intersection case.
- Added a validated `WallpaperPixelRect::new` constructor and exported the new geometry/layout types through `desktop`.
- Commits: `a5b33ccd` (geometry), `93284f76` (rectangle constructor), `804f9e67` (wallpaper module exports), `96c49b75` (desktop exports).
- This is CPU-side geometry only. No Vulkan shader/draw integration, decoder, or wallpaper display has been added. No Cargo fmt/check/test/build/CI or ISO was run; local Fedora validation remains required.


## 2026-11-24 — Dedicated Vulkan wallpaper shader and pipeline

- Added dedicated `wallpaper_texture.vert` / `.frag` shaders. The vertex shader maps a normalized UV rectangle supplied as four float push constants; the fragment shader samples the wallpaper texture.
- Updated `build.rs` to track and compile both shaders for DRM builds and expose their generated SPIR-V paths.
- Added a format-specific wallpaper pipeline cache alongside the existing surface pipeline cache. It uses the existing sampled-texture descriptor layout but has its own four-float UV push-constant range; the surface shader and its `u32` transform ABI remain unchanged.
- This is pipeline infrastructure only. No wallpaper descriptor is bound and no wallpaper draw is recorded yet. The decoder and selected texture population remain outstanding.
- Commits: `e0823ebe` (vertex shader), `fa3d28e4` (fragment shader), `41b24c1c` (shader build integration), `7a742fe6` (dedicated pipeline cache).
- No Cargo fmt/check/test/build, CI or ISO was run. Fedora local validation remains required; Debian 13 compatibility remains the target.

## 2026-11-24 — Generic Vulkan textured push constants

## 2026-11-24 — Wallpaper UV push-constant encoding

## 2026-11-24 — Wallpaper draw integration in Vulkan frame composition

## 2026-11-24 — Wallpaper cache asset identity guard

- Added `VulkanWallpaperTextureCache::texture_for_asset` and a matching surface-cache accessor.
- The compositor now compares the selected asset ID in `DesktopSnapshot` with the ready texture key before binding or drawing. A stale texture from a previously selected asset can no longer flash while a replacement upload is pending.
- Commits: `e5faff5a`, `337b403a`, `4bbde807`, `2c2e0e3e`.
- No decoder or pixel producer is present yet. No Cargo fmt/check/test/build, CI or ISO was run.

- Integrated the selected, upload-complete wallpaper texture into `surface_compositor::render_frame`.
- The frame creates a descriptor binding for the wallpaper, uses the dedicated format-specific wallpaper pipeline, computes output-local `Fill` geometry from the uploaded texture dimensions, and records the draw immediately after clear and before SHM surfaces/cursor.
- A ready wallpaper is rendered even when the scene has no windows or cursor; the old clear-only fast path is retained when no ready wallpaper texture exists.
- Clearing wallpaper selection clears the texture cache, preventing a previously selected image from persisting.
- Commit: `30582074` (frame integration), `a347fde8` (selection-clear behavior).
- Important limitation: there is still no JPEG decoder or producer feeding decoded pixels to `prepare_wallpaper_texture`. This is the Vulkan consumption path only; no visible wallpaper is claimed. No Cargo fmt/check/test/build, CI or ISO was run.

- Added `WallpaperUvRect::to_push_constant_bytes`, encoding the UV rectangle as four native-endian `f32` values in shader ABI order: min X, min Y, max X, max Y.
- Added a unit test that decodes the 16-byte payload and checks the exact component order.
- Commit: `f474ddfd`.
- This is CPU-side payload preparation only. No wallpaper descriptor binding or draw integration has been added. Test authored, not run; no Cargo format/check/test/build or CI run.

- Extended `VulkanDrawPass` with `record_textured_region_with_push_constants`, allowing dedicated textured pipelines to provide their own push-constant bytes without changing the existing surface transform ABI.
- Existing textured surface calls retain the four-byte `u32` transform. The wallpaper shader can use its separate 16-byte UV rectangle range.
- Added pre-record validation for non-empty, four-byte-aligned payloads up to 128 bytes, plus unit tests for the 16-byte wallpaper payload and invalid sizes.
- Commit: `7b9b4b37` (draw-pass API), `d8a31412` (validation and tests).
- This is draw-pass infrastructure only: wallpaper binding and draw integration are still pending. Tests were authored but not run. No Cargo fmt/check/test/build, CI or ISO was run.


## 2026-11-24 — SHM texture device-loss propagation

- The Vulkan call-chain audit found a stale `from_vk_result` reference in the `clear_and_readback` path after the converter had been removed. Replaced it with `submit_and_wait_with` and centralized `VulkanQueueSubmitError` classification.
- Commit: `76af94e5`.
- Extended fatal device-loss classification to Wayland SHM texture upload submission and completion. `ERROR_DEVICE_LOST` is no longer flattened into a generic texture error in `prepare_surface_textures` and `complete_surface_texture_uploads`.
- Added a unit test proving that device loss from the SHM texture cache becomes `VulkanSurfaceCompositionError::DeviceLost`.
- Commits: `8feb7509`, `822e1da3`.
- No new package or crate; no Cargo.toml/Cargo.lock changes.
- Tests were added but not executed. No build, rustfmt, cargo check or CI was started; verification remains scheduled on Fedora under the agreed workflow.
- Next audit: inspect remaining Vulkan entry points that may flatten `ERROR_DEVICE_LOST`, especially native cursor texture creation and diagnostic/offscreen paths. Keep the fail-stop policy; do not add device recovery before a complete device-recreation lifecycle exists.


### Follow-up — typed native cursor texture errors

- Extended the same classification to initial native cursor texture setup. The cache no longer converts `VulkanShmTextureError` to `String` before the compositor can recognize `ERROR_DEVICE_LOST`.
- Added a unit test for direct fatal SHM texture error conversion; the cache-level test remains in place.
- Commits: `c6e2737c`, `3d5fba86`, `d3ff57c4`.
- No verification was run. Initial cursor setup still uses synchronous `from_snapshot`; evaluate a future asynchronous upload so the frame callback does not wait for GPU completion.


### Follow-up — legacy Vulkan result conversion

- Audited callers that still use `submit_and_wait()` and receive a `vk::Result` instead of the full `VulkanQueueSubmitError` type.
- The legacy converter always returned `wait_error` for `CompletionUncertain`. When fence waiting returned `TIMEOUT` and the `device_wait_idle` fallback returned `ERROR_DEVICE_LOST`, the fatal error was hidden.
- Updated `VulkanQueueSubmitError::result()` to prioritize `ERROR_DEVICE_LOST` in either field while retaining `wait_error` for ordinary failures. Added two focused unit tests.
- Commit: `4a5ac3d2`.
- No tests, build, formatting or CI were run.

- Static follow-up: kept `result()` const-compatible by using pattern matching instead of `PartialEq` comparisons. Commit: `2acb358a`.


## 2026-11-24 — Asynchronous native cursor texture upload

- Removed the synchronous first-time upload of the compositor-owned native arrow from the Vulkan compositor.
- The pipeline cache now starts the upload through `VulkanShmTexture::submit_snapshots` and retains a `VulkanPendingUpload` (staging allocation, command pool, command buffer and fence) until completion.
- Each frame polls the fence without blocking. The texture becomes available to rendering only after completion; during intermediate frames the cursor is temporarily omitted, avoiding both a blocked frame callback and sampling an incomplete texture.
- The ready texture retains shared ownership through `Arc`, consistent with the cache resource model.
- Vulkan errors remain typed and `ERROR_DEVICE_LOST` still reaches the compositor's fatal classification.
- No dependency added and no Cargo.toml/Cargo.lock changes.
- Commits: `bb8ae90e`, `fd1f8a80`, `0e35bd1f`.
- No build, rustfmt, cargo check, tests or CI were run. Verification remains scheduled for the agreed Fedora session.


### Follow-up — static cursor audit and pending-upload initialization fix

- Static review of `VulkanSurfacePipelineCache::new` found that the newly added `pending_cursor_upload` field was not initialized. Added `None` to the constructor; without this correction the workspace would fail to compile.
- Fix commit: `45757f16`.
- Initial pointer-path review: the native cursor uses an internal RGBA8 bitmap, an explicit logical hotspot, logical coordinates, and output-aware scale/transform geometry. Physical input is routed through libinput via Smithay.
- Touchpad preferences are applied only when libinput advertises the corresponding device capabilities; unsupported settings are reported rather than forcing drivers or broader permissions.
- No new dependency, root access, or Debian-base modification was introduced. No build, tests, rustfmt or CI were run.
- Standing compatibility gate reaffirmed: assess every new dependency and hardware/codec/service integration against Debian 13 Trixie, least-privilege requirements, licensing, and repository availability before adoption.



### Follow-up — Debian compatibility for shader build tools

- The build audit found a Fedora-specific installation hint in `components/aurea-compositor/build.rs`, which was inappropriate for the target platform. Replaced it with a distribution-neutral message identifying `glslang-tools` for Debian and `glslang` for Fedora. Commit: `6be56e11`.
- Updated `tools/shaders/compile-vulkan-shaders.sh` to show both installation commands. Commit: `4e30598b`.
- Verified `glslang-tools` availability on Debian 13 Trixie amd64 (version `15.1.0+1.4.309.0-1`). It is a build-time tool and must not be added automatically to the installed runtime image.
- Recorded candidate native development dependencies for Smithay/DRM/input in both language versions of the dependency register. The package set is not approved wholesale: exact direct requirements must be confirmed from the locked Cargo graph and a future Debian build.
- English dependency register commit: `39a20444`; Italian register commit: `f0542b28`.
- No package was installed or added to runtime manifests. No build, tests, rustfmt or CI were run.


### Follow-up — pointer input path status

- Static review confirms that libinput relative motion, button, axis/scroll and touchpad gesture events are routed into the compositor through Smithay.
- Motion updates global logical pointer coordinates, constrains them to the union of enabled outputs, recomputes the Wayland surface under the pointer and requests frames on all outputs.
- Primary-button press already focuses/raises the hit toplevel; the event is then delivered to the Wayland surface. Double-click semantics and desktop-icon interactions are not yet implemented in this path.
- Client Wayland cursor surfaces and the AureaOS native arrow use separate paths: the former uses the client surface and its hotspot; the latter uses the internal bitmap and output-aware geometry.
- No additional broad device permissions or proprietary vendor driver are required by the reviewed input path; libinput access is mediated by the libseat session.
- Static audit only; no build/tests/formatting were run. Next: inspect cursor-state lifecycle across outputs, hotplug and session pause/resume, then connect desktop hit testing/interactions without duplicating Wayland input logic.


## 24 November 2026 — Desktop pipeline alignment and pointer lifecycle audit

### Actual wallpaper pipeline status

A source review on `main` showed that the implementation is further ahead than the previous architecture record described:

- `aurea-media` is already a separate crate with a TurboJPEG C shim, Rust API and bounded worker.
- `WallpaperDecodeService` is connected to the Vulkan cache and handles revisions, stale results, failures and back-pressure without blocking the frame.
- `VulkanWallpaperTextureCache` retains asynchronous uploads and pending resources until their fences complete.
- `surface_compositor::render_frame` draws a ready wallpaper texture after the clear and before Wayland surfaces; the cursor remains above the scene.
- Fill geometry is computed per output. Wallpaper resources are separate from the Wayland SHM surface cache.

This describes source integration, not visual validation. The Fedora laptop test has not been run, and wallpaper presentation on physical hardware is not claimed as working.

### Static output/session/pointer audit

- Output reconciliation refreshes pointer focus and requests new frames.
- When an entire DRM device is removed, its outputs are withdrawn and the compositor attempts to recenter the pointer on the first remaining enabled output.
- During `PauseSession`, queued input events are ignored; KMS surfaces are disabled before GBM/Vulkan resources are released.
- During `ActivateSession`, GBM, Vulkan, KMS surfaces and deferred outputs are restored; the pointer is then centered on an enabled output.
- Recentering is skipped while a client owns a pointer grab. This intentionally avoids interrupting a Wayland interaction. The end-of-grab transition still needs review when the output underneath the pointer is removed during the grab.
- Native cursor geometry is resolved per output and accounts for scale, fractional scale and target transform.

This was a static audit; no physical hotplug or VT-switch test was performed.

### Desktop items: existing starting point

- The shared API already contains `DesktopRect` and `DesktopHitTest`, using logical coordinates, inclusive left/top edges, exclusive right/bottom edges and deterministic overlap resolution.
- `DesktopItemsSnapshot` is already carried by `RenderScene`.
- Vulkan composition does not yet draw native desktop icons, and primary-click handling does not yet consult `DesktopHitTest`: current click handling focuses/raises Wayland windows.
- `icons/generated` currently contains only its README; SVG source artwork remains canonical.

### Documentation

- Updated `docs/architecture/DESKTOP-RENDERING-AND-MEDIA-PIPELINE.md` to match the implementation, distinguish source integration from hardware validation, and correct the next implementation sequence.
- Commit: `33e8ebae`.

### Next block

1. Define a reproducible SVG-to-raster pipeline with versioned outputs and no runtime SVG dependency.
2. Add dedicated desktop assets and a Vulkan texture cache without reusing the SHM cache.
3. Draw icons between wallpaper and Wayland surfaces, with consistent logical/output geometry and clipping.
4. Only then connect hit testing, selection, double-click and context menus; file opening and application launching remain owned by Files/Shell services.
5. Resume the pointer-grab completion audit for hot-unplug.

No build, tests, rustfmt or CI were run. No runtime dependency was added. Debian 13 Trixie remains the target platform; Fedora remains the development host.


## 24 November 2026 — First reproducible Trash icon export pipeline

- Added `tools/assets/render-trash-icons.sh`, a working-directory-independent Bash script that exports the six approved Trash SVG masters (Light/Dark × Empty/Half/Full) to PNG.
- Eight square sizes are defined: 16, 24, 32, 48, 64, 96, 128 and 256 pixels; 48 output files in total.
- The renderer is pinned to `resvg 0.45.1`. The script rejects other versions and checks file presence, PNG signature, exact dimensions and 8-bit RGBA format.
- Verified Debian 13 Trixie package availability for `resvg 0.45.1-1` (amd64). Upstream license is Apache-2.0 OR MIT. No official Fedora package was confirmed; installing the same version through Cargo is documented but its compilation cost on the laptop remains to be assessed.
- `resvg` is a host-only static-asset tool. It is not added to AureaOS runtime, Cargo.toml or the root filesystem.
- Updated icon READMEs and both dependency registers. The tool remains a candidate until exports are generated and visually inspected.
- Main commits: `be04164b` (script), `70409c7d` (PNG validation), `8c17ffc1`, `cdac2ab4`, `33ed98bd`, `67e2278b` (documentation).
- The script, builds, tests, rustfmt and CI were not run. No PNG was generated and no runtime dependency was added.


### Follow-up — reuse the existing SVG renderer

- The first pipeline draft introduced `resvg` as a separate tool. Before treating that choice as final, inspection of `tools/assets/build_cursor_theme.py` and `cursor_rasterizer.py` showed that AureaOS already uses `rsvg-convert` to generate its cursor theme.
- To avoid a second renderer and a new installation path, `render-trash-icons.sh` was changed to reuse `rsvg-convert`.
- Verified packages: Debian 13 `librsvg2-bin 2.60.0+dfsg-1+deb13u1`; Fedora 45 `librsvg2-tools 2.63.2-2.fc45`. Both provide the `rsvg-convert` command.
- The actual renderer version is recorded in `EXPORT-MANIFEST.txt`; PNG validation checks signature, square dimensions and RGBA8 format.
- The resvg entry was replaced in the dependency register. No additional renderer or runtime package is required beyond the tool already used by the cursor asset flow.
- The Trash pipeline has not yet been run; no PNGs have been generated or visually validated.
- Correction commit: `e0228650`; documentation updated in commits `65705341`, `eb28e253`, `4aa8a60e`, `e9819c41`, `bc95da4b`.


## 24 November 2026 — Per-output desktop icon geometry and RGBA8 asset pipeline

### Icon geometry

- Added `components/aurea-compositor/src/desktop/item_geometry.rs`.
- Persisted positions remain global logical coordinates; the module resolves physical destination geometry independently for each output.
- Handles output origin, integer and fractional scale, display transform and clipping to physical output bounds.
- Initial icon destination is 64 × 64 logical pixels; the renderer will use raster variants for 1x, 1.5x, 2x and 3x.
- Items outside an output or with invalid geometry are skipped individually so one bad saved position cannot suppress other icons.
- Added four source tests: secondary-output global origin, off-output item, 2x scale and partial clipping.
- This module is preparatory: it is not connected to Vulkan drawing yet, and visible desktop icons are not claimed. No tests were run.

### Static assets without a runtime PNG decoder

- Added `tools/assets/png_to_rgba.py`, using only the Python standard library.
- The converter accepts only non-interlaced RGBA8 PNGs up to 256 × 256; it validates chunk CRCs, decompression limits, exact pixel length and PNG filters 0–4.
- `render-trash-icons.sh` now generates 24 raw RGBA8 files (64, 96, 128 and 192 px) from the six canonical SVGs, reusing `rsvg-convert` already required by the cursor asset flow.
- Raw files are tightly packed, row-major, without padding: exact size = width × height × 4. Temporary PNGs are removed at the end.
- No Rust PNG decoder, Cargo crate or runtime package was added. Python 3 and rsvg-convert remain host-side asset-generation tools.
- The new script has not been run; the 24 outputs have not been generated or visually inspected.

### Status and next steps

1. Run the script on Fedora and inspect transparency, alpha edges, Light/Dark contrast and optical alignment.
2. Implement a dedicated desktop asset cache and bounded asynchronous Vulkan upload; do not reuse the SHM cache.
3. Connect per-output geometry to the renderer and draw icons between wallpaper and Wayland surfaces.
4. Only then implement hit testing, selection and activation.

No builds, tests, rustfmt or CI were started. No runtime dependency was added. Debian 13 Trixie remains the target.


## 2026-11-24 — Desktop geometry review and module cleanup

### References
- Branch: `main`.
- Commit `10f7e7ce607118555e09666cab23c33fb0b48de5`: removed duplicate `item_geometry` module declarations and duplicate re-exports from `components/aurea-compositor/src/desktop/mod.rs`.

### Engineering review
- Confirmed that `ShmBufferSnapshot::from_rgba8` accepts compositor-generated pixel data and `VulkanShmTexture::submit_snapshots` provides fence-backed asynchronous uploads.
- Confirmed that the SHM cache is keyed by Wayland `ObjectId` and must not be reused for native desktop icons.
- Confirmed that per-output desktop geometry handles global coordinates, integer/fractional scale, output transforms and clipping; it remains preparatory and is not connected to Vulkan drawing yet.
- The current render pass draws wallpaper, Wayland surfaces and the cursor. Native desktop icons are not yet part of the pass.

### Verification and limitations
- Source inspection on `main`; no build, tests, rustfmt or CI were run.
- RGBA8 assets have not yet been generated or visually inspected on Fedora.
- No compilation claim is made because the agreed Cargo verification has not been run.

### Next steps
1. Generate and visually inspect the 24 Trash RGBA8 variants on Fedora before integrating them as runtime assets.
2. Implement a native Vulkan cache with asset keys, memory budgets and asynchronous uploads, separate from SHM and wallpaper caches.
3. Connect ready textures to per-output geometry and draw them between wallpaper and Wayland surfaces.
4. Connect desktop hit-testing and interactions afterward, preserving Wayland focus precedence.


## 2026-11-24 — Dedicated Vulkan cache for native desktop icons

### Changes committed
- Added `components/aurea-compositor/src/vulkan/desktop_icon_texture_cache.rs`.
- Native cache identity consists of an approved asset ID, revision and raster pixel size; it does not use Wayland `ObjectId` values.
- The cache is independent from both client SHM surface textures and wallpaper textures.
- Explicit limits: 32 resident textures / 16 MiB; two in-flight upload batches / 8 MiB staging budget.
- Uploads use `VulkanShmTexture::submit_snapshots`; fence polling is non-blocking in the normal frame path, and completion is explicit before renderer access.
- Square dimensions are validated against the key and capped at 256 pixels. Pending uploads retain their resources until completion.
- Registered the module in `vulkan.rs` and exposed prepare, poll, completion and texture lookup through `VulkanSurfacePipelineCache`.

### References
- `02feb39e` — initial cache implementation.
- `80b8cc3f` — request deduplication.
- `4fed642e` — identity/dimension validation and pending-upload lifetime.
- `574fece0` — Vulkan module registration.
- `750e9d6d` — pipeline-cache integration.

### Status and limitations
- This is preparatory infrastructure: no generated assets feed it yet, and it is not connected to the draw pass or icon geometry.
- The 24 RGBA8 variants have not been generated or inspected on Fedora; no unverified binary assets were added.
- No build, tests, rustfmt or CI were run. Compilation is not claimed.
- No Cargo or runtime dependency was added.

### Next steps
1. Generate and inspect RGBA8 assets on the Fedora host.
2. Add the mapping from desktop item and theme/state to approved assets.
3. Connect cache, per-output geometry and Vulkan drawing while preserving wallpaper → icons → Wayland surfaces → cursor order.
4. Proceed to hit testing, selection and activation only afterward.


## 2026-11-24 — Desktop icon asset resolution

### Changes committed

- Added `components/aurea-compositor/src/desktop/icon_assets.rs`, a single-purpose resolver for approved desktop artwork.
- The current desktop model's Trash item maps to the six approved variants: Light/Dark × Empty/PartiallyFull/Full.
- Four raster densities are represented: 64, 96, 128 and 192 pixels for a 64-logical-pixel icon.
- Each descriptor carries a stable asset ID, content revision, raster size and repository-relative generated RGBA8 path.
- The descriptor validates RGBA8 bytes through the existing `ShmBufferSnapshot::from_rgba8` constructor before creating a cache request.
- `DesktopIconTextureKey` now belongs to the Desktop module rather than the optional Vulkan module. The asset contract therefore remains available when the Vulkan feature is disabled.
- File, Folder and Application entries intentionally receive no fallback artwork yet; their canonical assets and MIME/application lookup are not approved or integrated.

### Explicit limitations

- Empty/PartiallyFull/Full is supplied to the resolver; it is not inferred from an item name or URI and is not yet connected to the Trash service.
- The 24 RGBA8 files have not been generated, loaded or visually inspected on Fedora.
- The resolver is not connected to the frame renderer. The Vulkan cache remains preparatory infrastructure and no desktop icon is drawn yet.
- No Cargo or runtime dependency was added.

### References

- `0f06f637` — initial desktop asset descriptor module.
- `fbd82001` — move texture identity out of the Vulkan cache.
- `38d5749b` — register the resolver in the Desktop module.
- `0d1e135e` — temporary Vulkan key export, removed in the architecture correction.
- `9c0b9a26`, `6ca60a59`, `f0353135` — final correction: key defined in Desktop, independent of the optional Vulkan feature.
- Architecture documentation updated in the same work block.

### Verification

Static source/contract review only. No build, tests, rustfmt, cargo check or CI were run under the agreed workflow. Compilation is not claimed.

### Next actions

1. Run the 24-asset export on Fedora and inspect dimensions, transparency, alpha edges and appearance over light/dark backgrounds.
2. Connect the Trash service to `DesktopTrashContents` without fabricating a fallback state.
3. Load the generated bytes and build requests for the Vulkan texture cache.
4. Connect cache and per-output geometry to the draw pass, preserving wallpaper → icons → Wayland surfaces → pointer ordering.


### Follow-up — Shared desktop Trash state

- Added `DesktopTrashState` to `aurea-desktop-api`. Empty, PartiallyFull and Full now have one technology-independent semantic type shared by the shell, Trash service and compositor-facing desktop model.
- `DesktopManager` owns the state with explicit getter/setter methods and an Empty default.
- `DesktopItemsSnapshot` copies the state with the desktop entries, providing an immutable value to each render frame.
- Aurea Shell reuses the shared enum under its existing `TrashContentsState` name. The `aurea-desktop::TrashVisualState` service type has an explicit conversion into the shared contract.
- The icon resolver now consumes the shared state; no duplicate compositor-local state enum remains.
- Filesystem access, item counting and threshold policy remain in the Trash service. The compositor does not scan directories, calculate fullness or run a watcher.

References: `1807c23d`, `544036fb`, `5c35a382`, `17e7062e`, `5a08ca66`, `6048a1cc`, `5a3c05bf`, `e776dfba`.

**Remaining limitation:** the conversion is available, but the application composition root does not yet forward the service snapshot into the compositor's desktop model. No watcher or automatic refresh is implemented.

No dependency was added. No build, tests, rustfmt, cargo check or CI were run.


### Follow-up — Scale-aware raster selection

- The resolver selects the smallest approved raster that covers the output's effective scale: 64 px through 1x, 96 px through 1.5x, 128 px through 2x, and 192 px above 2x.
- Non-finite scale values conservatively fall back to the 1x asset.
- The caller must use `RenderTarget::effective_scale()`, not only the legacy integer output scale.
- The resolver remains disconnected from the frame renderer until RGBA8 assets have been generated and inspected.

Commits: `4bb33193`, `1ca67908`. No Cargo or CI verification was run.


## 2026-11-24 — Trash service modularization

- Reduced `components/aurea-desktop/src/trash/service.rs` from 399 to approximately 280 lines.
- Moved freedesktop deletion-date formatting into `trash/timestamp.rs`, with focused tests for the Unix epoch, leap day and year boundary.
- Moved public Trash service behavior tests into `components/aurea-desktop/tests/trash_service.rs`; production service code no longer embeds a large test module.
- Metadata parsing/serialization remains in `trash/metadata.rs`; filesystem entry lifecycle operations remain in `trash/service.rs`.
- No new dependency and no intended public behavior change.

Commits: `4769658c`, `f00e1d9e`, `78e05c62`, `40a95532`, `a3f6e9b2`.

Static path and dependency review only. No build, tests, rustfmt, cargo check or CI were run.


### Follow-up — Native cursor Vulkan module extraction

- Reduced `vulkan/surface_pipeline_cache.rs` from 398 to 353 lines.
- Moved native arrow texture creation, pending upload fence ownership, ready-texture promotion and cursor-specific errors into `vulkan/native_cursor_texture.rs` (99 lines).
- The general surface pipeline coordinator now delegates cursor resource lifecycle rather than owning its pending upload state.
- Intended behavior is unchanged: initial upload remains asynchronous and the cursor texture is exposed only after it is ready.
- No dependency added.

Commits: `d0ea103b`, `bd5a6e28`, `bf415144`. Static review only; no build, tests, rustfmt, cargo check or CI.


## 2026-11-24 — Native cursor Vulkan error-path corrections

- Replaced the synthetic Vulkan `ERROR_UNKNOWN` used for an empty completed upload with an explicit `EmptyUpload` error. An empty result violates the upload contract; it is not a driver-reported Vulkan failure.
- Aligned the composition layer with the extracted `NativeCursor` cache-error variant.
- Made the pipeline-cache error type visible to the sibling Vulkan composition module so it can preserve the specific SHM texture error mapping, including `ERROR_DEVICE_LOST`.
- No dependency added and no intended change to normal frame behavior.

Commits: `30397783`, `04c3aed5`, `552a342b`.

Static source review only. No build, tests, rustfmt, cargo check or CI were run. Compilation remains unverified until the agreed Fedora verification block.


## 2026-11-24 — Initial Trash state composition

- Added `trash/session_state.rs` to Aurea Desktop. It reads the home Trash service snapshot and converts it to the shared `DesktopTrashState` contract.
- The `aurea-desktop` composition root obtains this semantic value before creating the compositor. If Trash initialization or snapshot retrieval fails, it logs a warning and allows the desktop session to continue.
- `WaylandCompositorFactory` accepts the initial state, and `WaylandServer` applies it to the shared desktop item model before entering the event loop.
- The compositor still has no filesystem access: it receives only the service-derived Empty/PartiallyFull/Full value.
- No dependency added. The service uses the standard home Trash location under `$HOME/.local/share/Trash`.

**Explicit limitation:** this is a startup snapshot only. No watcher or in-session refresh mechanism exists yet, so the displayed state can become stale after delete/restore operations.

Commits: `46d2c1d6`, `1d3f8fa4`, `611720ee`, `cc7b860e`, `5930c025`.

Static interface and data-flow review only. No build, tests, rustfmt, cargo check or CI were run.


## 2026-11-24 — Desktop canvas item selection

- Added semantic `DesktopItemSelected { id }` and `DesktopSelectionCleared` events to the shared `aurea-desktop-api` contract. The interaction enum is no longer `Copy` because selection carries an owned identifier.
- Added `compositor/desktop_selection.rs`: converts global pointer coordinates to logical integer coordinates, applies `DesktopHitTest` to desktop model entries using a 64×64 logical-pixel hit region, and forwards the result to the shell.
- Primary-click handling still prioritizes Wayland surfaces. A click is interpreted as a desktop selection only when no window surface is hit.
- `ShellRuntime` updates `DesktopState.selected_item_id` and handles selection clearing. Added runtime and desktop-state tests for both transitions.
- Kept hit-testing policy separate from the general pointer-button handler.

**Explicit limitation:** selection is computed and retained in Shell state, but the renderer does not yet draw a selection outline/highlight. Double-click and item activation/opening remain future work. No desktop icon is visible yet: generated RGBA8 assets still need to be produced and connected to the Vulkan draw pass.

Main commits: `edb084ba`, `57174278`, `281eeb56`, `66259fdc`, `141bb3d2`, `23eefbc5`, `52eb356e`, `1aab344d`, `738a7dca`.

Also fixed the duplicated invalid visibility declaration on `VulkanSurfacePipelineCacheError` (`2169ed40`). No build, tests, rustfmt, cargo check or CI were run.


## 2026-11-24 — Desktop double-click recognition

- Added the focused `compositor/desktop_click_tracker.rs` module. It recognizes the same item clicked twice within 400 ms and within 4 logical pixels.
- Uses monotonic `Instant` timestamps without timers, worker threads or polling. Clicking a window or empty desktop resets the pending sequence; a completed double-click resets the tracker so a third click starts a new sequence.
- Added the shared `DesktopItemActivated { id }` semantic event. The compositor emits selection first, then activation.
- The shell retains the activation intent in its bounded pending-action queue. It deliberately does not launch a file or application; entry resolution and launch policy belong to the Files/Shell service layer.
- Added tracker source tests for valid double-click, different item, timeout, pointer distance, reset and triple-click, plus a shell test for activation retention.

No icon drawing, selection highlight or actual file/application launch is claimed. Static review only; no build, tests, rustfmt, cargo check or CI were run.


## 2026-11-24 — Shell selection propagated into Vulkan frame preparation

- Added a shared `DesktopSelectionController` to `aurea-desktop-api`, backed by `Arc<RwLock<Option<String>>>`. It returns owned snapshots and never exposes a lock guard to rendering code.
- The interaction sender publishes selection only after successful event delivery. This makes the next click-triggered frame observe the new selection without racing the Shell event-loop poll.
- `ShellRuntime` attaches the receiver's controller and mirrors selection/clear events into both `DesktopState` and the shared projection.
- `AureaCompositor` retains the controller; `render_scene` samples the selected identifier into the immutable frame.
- `DesktopItemsSnapshot` captures that identifier, and per-output `DesktopItemDrawGeometry` carries an `is_selected` flag for the matching item.
- Added source tests for shared controller state, immediate publication, immutable frame snapshot and selected/unselected geometry.

The flag currently reaches per-output geometry preparation only. The Vulkan draw pass does not consume it yet, so no visible selection highlight or native desktop icon rendering is claimed. No build, tests, rustfmt, cargo check or CI were run.

## 24 November 2026 — Desktop item ownership contract

- Clarified ownership: Desktop Shell owns the user-facing experience and presentation policy; the Compositor owns output geometry, GPU resources, Vulkan drawing and presentation only.
- aurea-desktop-api remains the technology-neutral boundary for items, identifiers, positions and interaction intents.
- Filesystem/Fides provide discovery, authoritative metadata, MIME information, thumbnails and operations; Fides does not draw desktop icons.
- Trash Service owns Trash state and operations and publishes semantic data; it does not draw its own icon.
- Icon/Theme resolves approved assets; it does not choose positions or perform file operations.
- The common visual pipeline must support File, Folder, Application and Trash. File artwork must use authoritative MIME metadata rather than filename extension alone; application artwork is resolved from Desktop Entry identity.
- The renderer must not inspect URIs to infer types, scan directories, count Trash entries, launch applications or contain filesystem policy.
- Identified a decision required before rendering integration: the installed runtime asset-root contract. Repository-relative paths are only suitable for source asset generation and must not depend on process working directory.
- Updated docs/architecture/DESKTOP-RENDERING-AND-MEDIA-PIPELINE.md with the ownership contract and implementation sequence.
- No runtime source code changed in this block. No build, tests, rustfmt or CI were run.

### Next block

1. Define runtime asset root/packaging and an RGBA8 loader outside the frame loop.
2. Complete asset mapping for all item categories.
3. Connect geometry, Vulkan cache and draw pass through focused modules.
4. Implement visible selection treatment, then connect Shell/Files actions.


## 2026-11-24 — Scope of the first usable desktop milestone

The immediate target is deliberately narrower than the long-term desktop architecture. The first usable session must show only the desktop background, a working pointer and one Trash icon at the upper-left, with visible selection feedback on click.

- The desktop model already contains exactly one default Trash item, ID `system:trash`, at global logical position `(32, 32)`. Do not duplicate or reposition it.
- Pointer movement, buttons, double-click recognition and wheel input already have code paths; real laptop validation is still pending.
- The selection flag reaches per-output icon geometry, but no visible selection treatment is drawn yet.
- The asset resolver, geometry and dedicated Vulkan cache are preparatory and are not connected to the draw pass.
- The 24 generated Trash RGBA8 files have not yet been exported or inspected. Generate and inspect them on Fedora before renderer integration.
- File, Folder and Application icons, context menus, drag-and-drop and item opening are explicitly deferred.
- Runtime loading must use an explicit installed asset root and happen outside the frame loop. Do not rely on the process working directory or read assets on every frame.
- Insert the Trash icon once in the shared composition path used by direct Vulkan and CPU-upload compatibility, between wallpaper and Wayland surfaces.

No build, tests, rustfmt or CI were run. Verification remains grouped on Fedora after integration and asset generation. The Trash is not yet claimed to be visible or hardware-validated.


### Desktop asset loader — groundwork

- Added `desktop/icon_asset_loader.rs` with one responsibility: read an already-generated RGBA8 asset beneath an explicitly supplied root directory.
- The loader rejects absolute paths and non-normal path components (including parent traversal), does not consult the process working directory, and reuses `ShmBufferSnapshot` RGBA8 validation.
- The runtime root has not yet been selected by the application composition root, and the renderer does not call the loader yet; the module is intentionally staged.
- No disk reads were added to the frame loop. No dependency was added.
- No build, tests, rustfmt or CI were run.


## 2026-11-24 — Static wallpaper policy: GIF removed

AureaOS wallpaper support is now explicitly still-image-only. GIF is excluded entirely, including first-frame import; no animated wallpaper or video playback path is part of the product.

Source changes on `main`:
- Removed GIF from `aurea-media::ImageFormat`, decoder dispatch and the `image` crate feature list.
- Removed GIF from `WallpaperFormat`, signature detection and decode-service mapping.
- Removed the animated wallpaper provider/playback contracts and the playback state previously carried by `WallpaperManager` and `WallpaperSnapshot`.
- Kept TIFF as a still-image format. Multi-page TIFF uses its first page as a static source; there is no page animation.

Updated `docs/design/WALLPAPER-SYSTEM.md` and `docs/architecture/AUREA-MEDIA-SERVICE.md`. This supersedes earlier development notes proposing GIF/video wallpaper providers.

No build, tests, rustfmt, Cargo check or CI were run. Cargo.lock has not been regenerated; Fedora validation remains pending.


## 2026-11-24 — Split libinput event routing from backend lifecycle

### Scope

The native input backend module mixed Calloop/libinput registration, touchpad preference application, and the dispatch of every input event family. The event router was extracted to keep the backend lifecycle module focused.

### Changes committed to main

- Reduced `components/aurea-compositor/src/server/input.rs` to 135 lines, covering backend registration, timestamp conversion, and touchpad preference application.
- Added `components/aurea-compositor/src/server/input/dispatch.rs` (335 lines) for libinput event routing.
- Preserved the existing routing paths for keyboard, pointer, pointer axes, swipe/pinch/hold gestures, touchscreen, and tablet-tool events.
- This is a structural refactor; it does not claim new input features or complete tablet-pad support.

### Verification and limitations

- No build, Cargo check, rustfmt, or tests were run, per the agreed Fedora validation workflow.
- No Cargo.toml or Cargo.lock changes were made.
- A grouped build and test pass remains pending after pulling on Fedora.
- Tablet-pad events and complete libinput event coverage remain an audit item; do not claim full support for additional tablet controls without checking the actual Smithay 0.7 API and event paths.

### Next action

Continue auditing input and interaction modules for correct event routing, lifecycle handling, and single-responsibility file boundaries.



## 2026-11-24 — Smithay 0.7 input event coverage audit

- Added explicit diagnostic routing for `PointerMotionAbsolute`; it records device identity and coordinates but does not move the pointer because no absolute-device-to-output mapping contract exists yet.
- Added explicit `SwitchToggle` diagnostics for lid/tablet-mode switch changes. Power/session policy remains outside the compositor input router.
- Confirmed from Smithay 0.7's public `InputEvent` API that `DeviceCapability::TabletPad` exists but there is no TabletPad event variant or pad event trait in this event enum. Existing tablet-tool buttons are not tablet-pad controls.
- Added `docs/architecture/INPUT-EVENT-COVERAGE.md` with the supported event surface, AureaOS routing status, limitations and follow-up sequence.
- No new dependencies and no Cargo manifest/lockfile changes.
- Static source/API review only. No build, Cargo check, rustfmt, tests or hardware validation. Grouped Fedora validation remains scheduled for Friday; Debian 13 remains the compatibility baseline.


### 2026-11-24 — Include pad-only devices in tablet-seat cleanup

- Device removal previously called `remove_tablet_device` only when `TabletTool` capability was present.
- Expanded the guard to include `TabletPad`, ensuring pad-only tablet devices participate in tablet-seat lifecycle cleanup even though Smithay 0.7 does not route pad input events.
- Per-device mapping/calibration cleanup remains handled by the generic pointing-device removal path.
- No build, Cargo check, rustfmt or tests run; grouped Fedora verification remains pending.

### 2026-11-24 — Prevent touchscreen auto-rerouting after output disconnect

- Found a routing edge case: removing an output erased a touchscreen's explicit assignment.
- If only one output remained, the normal single-output fallback could then route that touchscreen to a different display without user intent.
- The map now retains an unavailable-assignment state for affected devices. Routing stays blocked until the device is explicitly reassigned or unassigned to restore automatic mode.
- Device removal clears the retained state; a new explicit assignment also restores routing.
- No build, Cargo check, rustfmt or tests run. Grouped Fedora validation remains pending.


### 2026-11-24 — Separate pen-tablet and tablet-pad inventory kinds

- Found that pad-only devices were classified as `InputDeviceKind::Tablet`, allowing them to appear eligible for pen calibration and tablet output assignment despite having no TabletTool event path.
- Added `InputDeviceKind::TabletPad`; `Tablet` now requires `TabletTool`, while pad-only hardware receives the distinct inventory kind. Devices exposing both capabilities remain `Tablet` and keep their pad capability metadata.
- Pad-only devices remain discoverable in the pointing-device registry, but pen-specific calibration and output mapping APIs continue to accept only actual pen tablets.
- This does not add tablet-pad event support; the Smithay 0.7 limitation remains.
- No build, Cargo check, rustfmt or tests run. Grouped Fedora validation remains pending.

### 2026-11-24 — Prevent pen-tablet rerouting after output disconnect

- Found an inconsistency with touchscreen routing: tablet output mappings were deleted when their assigned monitor disconnected.
- With one display remaining, a pen tablet could then fall back to controlling the full desktop without an explicit user decision.
- The map now retains the stale assignment as unavailable and blocks resolution until the user explicitly assigns another output or unassigns the tablet.
- Physical tablet removal clears both assignment and unavailable state. Other devices' mappings remain independent.
- Added source tests for blocked rerouting, reassignment, unassignment and per-device isolation.
- No build, Cargo check, rustfmt or tests were run; grouped Fedora validation remains pending.

### 2026-11-24 — Constrain pen-tablet coordinates to actual monitor rectangles

- Reviewed pen coordinates on multi-monitor layouts with gaps or offset displays.
- Tablet calibration uses a bounding rectangle; on a non-contiguous layout that rectangle can include points that do not belong to any output.
- After calibration, tablet locations are now constrained to the actual enabled logical output rectangles. A point in a gap is moved to the nearest valid output edge.
- When a tablet is explicitly mapped to one monitor, only that monitor's geometry participates in the constraint.
- Reuses the geometry policy already used for relative pointer motion; touchscreen transforms and generic absolute-pointer events are unchanged.
- No build, Cargo check, rustfmt or tests were run. Grouped Fedora validation remains pending.


### 2026-11-24 — Initial absolute-pointer routing

- Connected PointerMotionAbsolute to the compositor's pointer seat.
- On a desktop with exactly one enabled output, absolute coordinates are transformed into that output's global logical rectangle, then routed through normal pointer focus, frame notification and redraw scheduling.
- Multi-output layouts are deliberately rejected until an explicit per-device output assignment exists. The compositor does not guess which monitor owns an absolute device.
- Touchscreen and tablet-tool events remain on their dedicated paths.
- This is partial support, not complete multi-monitor absolute-device support.
- No build, Cargo check, rustfmt or tests were run. Grouped Fedora verification remains pending; no dependency or Cargo manifest changes were made.

### 2026-11-24 — Add Debian 13 HEIF/AVIF native dependency gate

- Added a dedicated GitHub Actions job using the Debian Trixie container; the existing Ubuntu workspace build remains independent of libheif.
- The job installs only the libheif development package and the selected HEVC/AV1 decoder plugins (libheif-plugin-libde265 and libheif-plugin-dav1d), plus pkg-config.
- The job verifies the discovered libheif version is at least 1.20 and checks that both decoder plugin packages install shared-library files.
- This is a native package/API-baseline check only. It does not yet compile the Rust wrapper or enable HEIF/AVIF decoding in Aurea Media.
- No Cargo manifest or lockfile changes; no Fedora build/tests performed.
- The CI result must be inspected before selecting the wrapper integration and changing the media decoder.


### 2026-11-24 — Implement opt-in HEIC/AVIF still-image decoding

- Added an ISO-BMFF `ftyp` detector for AVIF (`avif`) and HEIC (`heic`/`heix`) still-image brands. Sequence brands (`avis`, `msf1`) and generic `mif1` alone are rejected; extended-size and malformed/truncated boxes are covered by unit tests.
- Added optional `libheif-rs 3.0.0` integration with the explicit `v1_20` API feature. The default build remains independent of native libheif; the compositor forwards an opt-in `heif` feature.
- The adapter applies libheif security limits before parsing, caps decoding threads, rejects sequences/multiple top-level images, validates dimensions/stride/buffer length and returns owned packed RGBA8 bytes. Format selection must match the detected container brand.
- Wallpaper asset detection and decoding dispatch now recognize HEIC/AVIF while keeping codec APIs out of the compositor renderer. No video or animated-image support was introduced.
- Added a Debian 13 CI job to install selected decoder-only plugins and compile/test the real Aurea Media feature. First CI attempt failed due to a private re-export and Debian Rust 1.85 being too old for `image 0.25.10` (requires Rust 1.88). Both issues were corrected; the next grouped CI run must confirm the result.
- Rust formatting remains deferred to the grouped Fedora session. No local Fedora build/test was run. Security review, real HEIC/AVIF fixtures, Fedora package parity and runtime manifest work remain pending.


### 2026-11-25 — Enable HEIC/AVIF in the production desktop image

- Source review confirmed the wallpaper decode worker, wallpaper decode service, Vulkan texture cache and frame renderer are already connected. The remaining integration gap was the production ISO build: it compiled `aurea-desktop` without enabling the optional HEIF feature, and the target image did not include libheif runtime plugins.
- Added an opt-in `heif` feature to `aurea-desktop`, forwarding to `aurea-compositor/heif`. The default workspace build remains independent of native libheif.
- The Debian Live hook now builds the desktop with `--features heif`, installs only `libheif1`, `libheif-plugin-libde265` and `libheif-plugin-dav1d` for HEVC/AV1 still images, and removes `libheif-dev` after compilation.
- Added a Debian 13 CI step to compile the complete `aurea-desktop` dependency graph with HEIF enabled, in addition to the media crate tests.
- No local Fedora build, Cargo check, test or rustfmt was run. The next CI run must verify the production feature graph and package integration. Security/licence review and real HEIC/AVIF fixtures remain open gates.


### 2026-11-25 — Audit TouchCancel semantics

- Reviewed the complete libinput input dispatcher and the touchscreen event path.
- Found that `InputEvent::TouchCancel` currently calls `handle_touch_cancel`, which releases that device's slot IDs and emits ordinary `TouchHandle::up` events. This is not equivalent to the Wayland `wl_touch.cancel` protocol event.
- Verified Smithay 0.7 exposes `TouchHandle::cancel`, but it cancels the seat's active touch session rather than accepting a device or slot filter. Using it for one device can therefore cancel contacts belonging to another touchscreen.
- No source behavior changed: a safe correction needs an explicit seat-wide cancellation policy and slot-map reset, rather than silently changing multi-touch behavior. No build or tests run.


### 2026-11-25 — Implement seat-wide TouchCancel semantics

- Replaced the previous synthetic per-contact `TouchHandle::up` sequence with Smithay 0.7's protocol-level `TouchHandle::cancel`.
- Smithay cancellation is seat-wide, so the policy is explicit: only a cancel from a device with at least one active contact triggers cancellation; then all physical-device-to-seat slot mappings are cleared.
- A late/spurious cancel from a device without active contacts is ignored and cannot interrupt another touchscreen's sequence.
- Added `TouchscreenSlotMap::clear_all` and a unit test proving that contacts from every device are cleared after seat cancellation.
- Updated `docs/architecture/INPUT-EVENT-COVERAGE.md` with the exact semantics and remaining validation requirements.
- This deliberately means a cancel on one active touchscreen interrupts concurrent touch contacts from other devices. Their later motion/up events are ignored until a new down event; no stale Wayland slot remains.
- No local build, Cargo check, rustfmt or tests were run, per the grouped-validation rule. Runtime behavior and device hot-unplug/reconnect still require validation.


### 2026-11-25 — Introduce modular pointer manager and macOS-inspired touchpad gesture

- Added `input/pointing/manager.rs` as the owner of pointing-device inventory, per-device pointer/touchpad preferences and physical pressed-button bookkeeping; compositor protocol delivery remains in its focused modules.
- Routed compositor device lookups, per-device pointer/touchpad settings and button-release bookkeeping through `PointerManager`.
- Added the three-finger upward swipe as an Overview shortcut, preserving three-finger horizontal workspace switching and four-finger pinch as an additional Overview shortcut.
- Added focused profile tests for upward direction and minimum distance. Existing two-finger scroll, natural-scrolling preference and Wayland pinch/swipe forwarding remain the basis of the touchpad experience.
- Recorded the macOS-inspired behavior as a design direction, not a claim of complete macOS parity. Downward App Exposé, Launchpad-style gestures and user-facing gesture customization remain future work.
- No build, Cargo check, rustfmt or tests were run; all changes are queued for grouped validation.


### 2026-11-25 — Add keyboard navigation fallback for touchpad gestures

- Extended the existing global shortcut resolver so a mouse-and-keyboard setup can use the same desktop navigation actions as the touchpad.
- Default bindings: `Super + Ctrl + Left/Right` for previous/next virtual workspace, `Super + Ctrl + Up` for Overview and `Super + Ctrl + Down` for App Exposé.
- Workspace changes continue through the compositor's workspace-switch operation; Overview and App Exposé use the existing compositor-to-shell interaction channel.
- Expanded shortcut resolver unit coverage for all four directions. The matching key press/repeat/release interception remains handled by the existing keyboard event filter.
- Updated the input coverage audit. The shell panel state is wired, but finished visual Overview/App Exposé surfaces remain separate work.
- No build, Cargo check, rustfmt or tests were run. Grouped validation remains pending.


### 2026-11-25 — Harden pointing-device disconnect bookkeeping

- Made synthesized pointer-button releases deterministic by sorting the outstanding Linux button codes before dispatch.
- Added focused PointerManager unit tests for per-device button isolation, stable multi-button release ordering, individual button release, per-device preference isolation and cleanup, and rejection of touchpad-only preferences on a mouse.
- No build, Cargo check, rustfmt or test execution was run. These tests are queued for the agreed grouped validation pass.


### 2026-11-25 — Order pointing-device hot-unplug cleanup

- Reordered libinput `DeviceRemoved` handling so gesture ownership, active touch contacts, held pointer buttons, interaction state, routing/calibration state and per-device preferences are cleared before the pointing-device inventory entry is removed.
- Removed touchpad apply-cache and retained libinput handle after compositor state cleanup, then reconciled touchscreen capability.
- Added source tests for deterministic current-sysname ID derivation and clarified that `InputDeviceId` is session-scoped, not a persistent physical-device identity. Per-device settings remain runtime overrides.
- No build, Cargo check, rustfmt, test execution or hardware validation was run; grouped validation remains pending.


### 2026-11-25 — Centralize pointing-device lifecycle and reclassification

- Routed device discovery and disconnect through compositor methods backed by `PointerManager`; registry mutation is now crate-private.
- Reconciled per-device settings when a device is re-announced with a changed kind, preserving only compatible pointer settings and clearing touchpad-only state/report data when appropriate.
- Unified disconnect ordering: clear compositor/protocol state first, then remove manager preferences, pressed-button bookkeeping and inventory; the libinput backend then drops retained handles and applied-setting cache.
- Added source-level tests for reclassification, stale touchpad report cleanup and disconnect state removal. No build, Cargo check, rustfmt, tests or hardware validation were run; grouped validation remains pending.

### 2026-11-25 — Clean up unsupported pointing-device re-announcements

Audited the `DeviceAdded` path for capability changes. If a previously registered pointing device is re-announced under the same sysname but no longer yields a supported pointing descriptor, AureaOS now disconnects it through the compositor lifecycle boundary, clears touchpad runtime caches, and synchronizes touchscreen capability. Added a PointerManager regression test proving that a same-ID reconnect does not inherit the previous connection's pressed buttons or per-device preferences. Source-level tests added but not executed; no build or rustfmt run, in accordance with the grouped validation plan.

### 2026-11-25 — Reject stale pointing events after hot-unplug

Added an inventory guard to pointer motion, buttons, axes, touchpad swipe/pinch/hold gestures, and touchscreen contacts. Late queued events for a removed device are ignored rather than forwarded into compositor state. Keyboard and tablet-tool handling remain separate. Validation is deferred to the planned grouped session; no build, test execution, or rustfmt run.


- Reworked touchscreen seat-slot allocation to recycle released IDs after touch-up, device removal and seat-wide cancellation. This prevents a long-lived session from exhausting the monotonically increasing slot counter during normal use. Added source tests for slot reuse and isolation between simultaneous devices. Commit: `505e284`.
- Expanded touchpad discovery to recognize devices that expose scroll methods even when tap-to-click, disable-while-typing and gesture capabilities are absent. The configuration path now shares the same discovery predicate. Added focused source tests. Commits: `f6c9cf8`, `ff722e3`, `908a00c`.
- No build, Cargo check, rustfmt, test execution or hardware validation was run. Validation remains grouped until the pointing-device block is complete.


- Added `docs/architecture/TOUCHPAD-COMPATIBILITY.md`: research baseline and feature matrix for Synaptics, ELAN, ALPS, Apple, generic HID/Precision Touchpad, Goodix, Cypress and external USB/Bluetooth touchpads. It separates kernel transport drivers, libinput normalization/gesture recognition, and AureaOS settings/action mapping; records hardware-dependent feature limits and the staged implementation plan. Commit: `12886b9`.
- Research references include the Linux Touchpad Dev Guide and current libinput documentation for gestures, configuration, features, and device quirks. No build, Cargo check, rustfmt, test execution or hardware validation was performed.

### 2026-11-25 — Expand libinput touchpad configuration

- Expanded the shared Desktop API while preserving the existing five-argument `TouchpadSettings::new` constructor. Added clickpad method, tap-button mapping, tap drag-lock, edge scrolling, left-handed mode, middle-button emulation, pointer speed and send-events preferences.
- The compositor applies each setting through the Smithay 0.7/libinput API only when that capability is advertised; per-setting reports distinguish Applied, Unsupported and Failed.
- Added regression coverage for builder defaults, bounded pointer speed, mutually exclusive edge/two-finger selection and the expanded report shape. Updated the compatibility matrix and report fixtures.
- Audited the relevant Smithay/input 0.7 API signatures against the versioned source documentation. This is source-level API verification, not a successful compile.
- No build, Cargo check, rustfmt, test execution or hardware validation was run. Continue batching validation until the pointing-device block is complete.

- Follow-up audit against the exact locked `input 0.9.1` source corrected the send-events integration to use its bitflags API (the later 0.10 API has different signatures). The settings now preserve each device's default clickpad method and acceleration profile unless the user explicitly chooses another one. Touchpad discovery also recognizes clickpad-only capability sets. Commit: `d3c0903`, `2f3dbe5`, `565bc5b`, `edf3a1a`, `4ef0a6f`, `4e91ef7`, `9cea047`, `106ad02`.
- Identified a real version boundary: Smithay 0.7.0 activates input's `libinput_1_19` feature, so DWTP (available in input 0.9.1 only behind `libinput_1_21`) is not wired yet. This remains explicit in the compatibility document rather than being falsely advertised.

- Audited the existing gesture path end to end: swipe, pinch and hold phases are dispatched to the Wayland pointer-gestures protocol; the AureaOS interaction engine accumulates swipe movement, tracks pinch scale/rotation and resolves completed gestures; cancellation suppresses shell actions; device removal clears active gestures and queued actions. The seat-level Wayland gesture stream is deliberately single-owner, while the internal engine remains device-scoped. No additional gesture patch was necessary in this pass.

### 2026-11-25 — Persistent touchpad profiles and DWTP support

- Added a privacy-preserving persistent identity to input descriptors. It hashes vendor/product IDs and udev ID_PATH, falling back to the normalized physical sysfs path; it deliberately excludes eventN numbering, display names and serial numbers. If neither physical path can be established, the profile is not persisted.
- Added an atomic TOML profile store at `$XDG_CONFIG_HOME/aureaos/input/touchpads.toml` (fallback: `~/.config/aureaos/input/touchpads.toml`). Writes use a temporary file, fsync/rename and 0600 permissions. Profiles are restored for matching touchpads on connect and saved when settings change.
- Added serializable touchpad preference types and an explicit disable-while-trackpointing setting.
- Smithay 0.7.0 remains the compositor framework and is pinned exactly to the audited crates.io release. Cargo already downloads it; no source vendoring or fork is needed. Optional udev 0.9 metadata access is activated only with the native DRM backend.
- Enabled the `input 0.9.1` `libinput_1_21` feature through an optional direct dependency activated by the native `drm` feature. Cargo feature unification makes the newer DWTP API available on Smithay's re-exported input crate. The compositor now applies DWTP when advertised by the device.
- Debian package metadata checked: Debian 13 ships libinput 1.28.1 and Debian 12 ships 1.22.1; both satisfy the 1.21 minimum for DWTP. Devices/build environments with older libinput will need a compatibility build profile or an explicit minimum-version check.
- No build, Cargo check, rustfmt, test execution or hardware validation was run. All validation remains grouped as requested.

- Dependency bookkeeping note: the new direct `serde`, `toml` and optional `udev` manifest entries are not yet reflected in `Cargo.lock`. Do not use `--locked` until the grouped Fedora Cargo run refreshes the lockfile. This was intentionally left for the planned validation batch; no GitHub build was started.


### 2026-11-25 — Protect persistent touchpad settings

- The touchpad profile store now opens malformed, unreadable or unsupported-version files in read-only mode. A later settings change returns an error instead of replacing the original file with an empty or downgraded profile set.
- The store now accepts only the explicitly supported format version, rather than treating a missing/zero version as compatible.
- Added focused source tests for malformed TOML and future format versions, including assertions that the original bytes remain unchanged after a rejected write.
- Runtime settings can still be applied for the current session; persistence failure is reported through the existing warning path. The user's original file remains intact for recovery or a future migration.
- No build, Cargo check, rustfmt, test execution or hardware validation was run. Grouped validation remains pending.


- Follow-up hardening: profile loading now treats only an actual filesystem `NotFound` result as a first-run empty store. Permission and other I/O errors remain read-only failures; they cannot be mistaken for an absent file and then replaced.


### 2026-11-25 — Per-device touchpad capability snapshots

- Added the public Desktop API type `TouchpadCapabilities`, attached optionally to each `InputDeviceCapabilities` snapshot.
- Discovery now captures maximum tap-finger count; two-finger and edge scrolling; natural-scroll availability; button-area/clickfinger methods; DWT/DWTP; gesture support; left-handed mode; middle-button emulation; acceleration availability and flat/adaptive profiles.
- The compositor reports what the active libinput device actually exposes. It does not infer capabilities from Synaptics, ELAN, ALPS, Goodix or other vendor names.
- Updated the compatibility matrix: kernel transport drivers and upstream libinput quirks remain responsible for hardware protocol differences. AureaOS consumes the normalized capabilities and does not add vendor-specific protocol drivers.
- USB and Bluetooth HID touchpads remain in scope; persistent profiles use available physical udev/sysfs identity and are deliberately not guessed when no stable path exists.
- Static repository search found no external struct-literal construction of `InputDeviceCapabilities`; callers use its constructors/default. This was a source search, not a compiler check.
- No build, Cargo check, rustfmt, tests or physical-device validation was run. Grouped Fedora validation remains pending.


## 2026-11-25 — Expanded touchpad capability snapshot

- Extended the shared desktop API with tap-to-click, tap-and-drag, drag-lock,
  tap-button-map, disable-events and disable-on-external-mouse capability flags.
- The compositor derives these flags from the pinned Rust `input 0.9.1`
  libinput API and the device's reported send-events modes.
- Deliberately did not expose three-finger drag: it is absent from the pinned
  `input 0.9.1` Device API, so newer upstream libinput documentation is not
  sufficient evidence that the pinned dependency can support it.
- No Cargo build, formatting, test suite or hardware validation was run; these
  changes remain queued for the grouped Fedora validation pass.


## 2026-11-25 — Restore touchpad preferences after session activation

- Added an explicit invalidation operation for the runtime cache of applied
  touchpad profiles.
- On successful libseat session activation, AureaOS now reapplies the saved
  profile to each retained touchpad libinput handle after graphics/device
  restoration. This covers VT pause/resume without treating it as a physical
  disconnect and preserves persistent hardware identity.
- Failed or unsupported settings remain represented in the per-setting report;
  incomplete profiles are not marked fully applied and can be retried later.
- No build, rustfmt, tests or physical-device validation performed.


### 2026-11-25 — Enforce non-exclusive multi-device pointing

- Corrected the session-resume path to invalidate and reapply each touchpad
  profile independently. A touchpad needing recovery no longer clears the
  successful apply-cache entries of other connected touchpads.
- Added a PointerManager regression test registering mouse, touchpad,
  touchscreen, tablet and tablet pad simultaneously, then disconnecting only
  the touchpad and asserting all other devices remain registered.
- Documented the shared Wayland seat model: events from all registered pointing
  devices feed the same desktop pointer/seat while preferences, button state,
  touch contacts, tablet state and cleanup remain device-scoped where the
  protocol supports it. Multi-monitor direct-touch mapping remains explicit.
- No build, rustfmt, tests or physical-device validation performed; grouped
  Fedora validation remains pending.


### 2026-11-25 — Isolate touchscreen cancellation by physical device

- Fixed a cross-device isolation issue: a TouchCancel from one touchscreen
  previously invoked Smithay's seat-wide touch cancel and cleared every active
  contact, including contacts originating from other screens.
- The compositor now removes only the cancelling device's physical-to-seat
  slot mappings, emits up events for those contacts and frames the shared seat.
  Other touchscreens keep their active contacts and slot mappings.
- Documented the Wayland limitation: wl_touch.cancel is seat-wide, so per-device
  cancellation is represented as per-contact completion to avoid cancelling
  unrelated users' active touches.
- No build, rustfmt, tests or physical-device validation performed.


### 2026-11-25 — Aggregate simultaneous pointer-button holds

- Changed physical button bookkeeping to report only shared seat transitions.
  Multiple mice/touchpads holding the same button now produce one Wayland press
  and one final release, rather than one press/release per physical device.
- Device removal releases only buttons no other connected device continues to
  hold. Added regression tests for overlapping holds and disconnect isolation.
- No build, rustfmt, tests or physical-device validation performed.


### 2026-11-25 — Isolate Wacom pen source and hot-unplug lifecycle

- Added a dedicated TabletToolRegistry separating physical tablet ownership
  from the currently active proximity source of each unique pen/tool.
- Axis, tip and tool-button events are now accepted only from the tablet that
  owns that tool's active proximity session. A uniquely identified pen can
  move between compatible tablets; the previous source is closed before the
  new one becomes active.
- Tablet removal now ends active pen proximity first (allowing Smithay to
  synthesize releases for held pen buttons and tip contact), then removes the
  tablet and retires tools only after their last owning tablet disconnects.
- Added registry tests for independent pens, shared pen identity across
  tablets, source handoff, and device-specific removal.
- Confirmed from Smithay 0.7 API documentation that TabletToolDescriptor
  exposes pen/eraser type and pressure, distance, tilt, rotation, slider and
  wheel capabilities. Smithay 0.7 explicitly does not implement tablet-pad
  input; pad buttons, rings and strips remain documented as unsupported.
- No build, Cargo check, rustfmt, test execution or physical Wacom validation
  was run. Grouped Fedora validation remains pending.


### 2026-11-25 — Reconcile touchpad gesture documentation

- Audited `GestureProfile` and its tests against the input coverage document. Three-finger swipe up already resolves to Overview; three-finger swipe down already resolves to App Exposé, with dominant-axis and minimum-distance checks.
- Corrected the architecture document so it no longer describes the downward gesture as unimplemented. The shell interaction state is wired; the finished visual panels remain separate presentation work.
- Documentation-only correction. No build, Cargo check, rustfmt or tests were run.


### 2026-11-25 — Filter stale touchscreen frame notifications

- Added the same live-device inventory guard to `TouchFrame` that already protects touch down, motion, up and cancel events.
- A queued frame notification arriving after physical touchscreen removal is now ignored instead of being forwarded to the shared Wayland touch seat.
- Updated the input coverage audit. Source change only; no build, Cargo check, rustfmt or tests were run.


### 2026-11-25 — Mouse input lifecycle hardening

- Re-audited the native libinput → Calloop → compositor path for relative mouse motion, button transitions, vertical/horizontal scroll and device hot-unplug. The seat pointer is initialized during server construction and centered after enabled outputs are published.
- Added a session-pause cleanup path that releases all logically held pointer buttons before libseat marks the session inactive. This prevents a client from retaining a stuck button when a physical release is lost during a VT switch.
- Releases are aggregated across physical devices: if two mice hold the same button, pausing the session emits one final release, not duplicate releases.
- This is a source-level correction only. No Cargo build, rustfmt, tests or physical mouse validation were run; grouped Fedora validation and a real mouse test remain mandatory before calling the mouse complete.

- Pointer acceleration is now applied at device discovery as well as during motion dispatch, so the selected profile is active before the first movement rather than waiting for the first motion event.


### 2026-11-25 — Mouse right-click on desktop

- Added a dedicated secondary-button path: right-clicking an empty desktop selects the item under the pointer (or clears selection) and sends the existing ShowContextMenu semantic interaction to Aurea Shell.
- Right-clicks over application windows remain normal Wayland pointer-button events, preserving application-owned context menus.
- Right-click does not participate in the desktop double-click activation tracker.
- No build, Cargo check, rustfmt, tests or physical mouse validation were run.


- Added a manager regression test proving duplicate physical press/release notifications from one mouse cannot generate duplicate Wayland seat transitions. Test source added but not executed.

### 2026-11-25 — Connect desktop pointer clicks to selection

- Wired primary clicks on the empty desktop to `select_desktop_item_at`: the item under the pointer is selected, the existing double-click tracker can emit activation, and empty space clears selection.
- Wired secondary clicks on the empty desktop to update/clear selection before sending `ShowContextMenu` with the pointer's logical position.
- Mapped Wayland surfaces retain precedence; right-clicks over application windows are still delivered to the client.
- Commit: `80483c23`. Shell already stores the requested panel and position, but no graphical menu surface is implemented yet; the menu is not visually displayed at this stage.
- No build, Cargo check, rustfmt, tests or physical mouse validation were run. Grouped Fedora verification remains pending.


### 2026-11-25 — Initial XDG popup configure and reposition

- Implemented the initial XDG popup configure path in `aurea-compositor/src/xdg_shell.rs`: preserve the supplied `PositionerState`, derive the protocol geometry with `get_geometry()`, and send `send_configure()`.
- Implemented the reposition response: update pending positioner/geometry state and acknowledge with `send_repositioned(token)`. Both paths request output frames.
- This is protocol lifecycle progress only. Output-aware constraint adjustment, per-seat PopupManager and coordinated pointer/keyboard grabs, popup rendering, popup hit testing and dismissal behavior remain incomplete.
- `grab` intentionally remains unimplemented rather than emulating an incorrect partial grab.
- No build, Cargo check, rustfmt, tests or hardware validation were run. Grouped Fedora verification remains pending.


### 2026-11-25 — Track nested XDG popup lifecycle

- Added Smithay 0.7 `PopupManager` to compositor state.
- Newly created XDG popups are registered as `PopupKind::Xdg`; surface commits update the parent/child popup tree and clean up dead resources.
- This establishes popup ownership and lifecycle tracking, but does not yet implement output-constrained placement, coordinated pointer/keyboard grabs, scene rendering, hit testing or dismissal.
- No build, Cargo check, rustfmt or tests were run; grouped Fedora validation remains pending.


### 2026-11-25 — Include XDG popups in Vulkan scene and hit testing

- `RenderScene` now collects popup surface trees registered with Smithay's `PopupManager`, computes their global logical origins from Smithay's popup offsets and geometry, and includes them in SHM snapshots consumed by the Vulkan compositor.
- Parent surfaces are emitted before popup surfaces because AureaOS Vulkan composition draws snapshots in sequence; this keeps popups visually above their owning toplevel. Popup object IDs are included in active surface tracking for texture-cache lifecycle.
- Pointer hit testing checks popup surfaces before their parent toplevel, so existing pointer motion/focus paths can target popup content.
- Output-constrained placement, coordinated pointer/keyboard grabs, dismissal and multi-monitor visual validation remain open. No build, Cargo check, rustfmt or tests were run.

### 2026-11-25 — Coordinate XDG popup pointer and keyboard grabs

- Implemented the `xdg_popup.grab` path using the actual `WlSeat` resource to recover its Smithay `Seat`. Requests are rejected if the resource is not managed or either pointer/keyboard capability is unavailable.
- Resolve the popup's toplevel root and let `PopupManager::grab_popup` validate the serial and nested-popup hierarchy.
- Install Smithay's `PopupPointerGrab` and `PopupKeyboardGrab` with the same serial. Their standard behavior confines interaction to the popup tree, dismisses grabs on outside pointer input, and restores root focus when the grab ends.
- Source/API integration only; no Cargo build, `cargo fmt`, check, tests or hardware validation were run. Keep verification grouped on Fedora as agreed.
- Remaining work: output-aware popup constraints, explicit keyboard-dismiss coverage, initial focus/runtime validation and multi-monitor testing.

### 2026-11-25 — Output-aware XDG popup constraint placement

- Added a dedicated `compositor/popup_placement.rs` module. The protocol handler now delegates placement rather than embedding geometry policy in `xdg_shell.rs`.
- Resolve the toplevel root, mapped window geometry and actual popup parent. Nested popup parents use Smithay's tracked popup-tree offsets, so the target rectangle is expressed in the correct parent window-geometry coordinate space.
- Select the enabled output containing the XDG anchor point in global logical coordinates. If the anchor lies in a monitor gap, fall back to the output containing the parent window center; do not silently choose an arbitrary first output.
- Translate the selected output's global logical bounds into parent-local coordinates and pass them to Smithay 0.7 `PositionerState::get_unconstrained_geometry`. Smithay applies the client's requested XDG flip, slide and resize adjustments.
- Reuse the same path for initial popup configure and `xdg_popup.reposition`; acknowledge the reposition token only after updating pending geometry.
- Invalid/dead parent chains, unavailable output targets and unrepresentable coordinate conversions preserve the protocol's unconstrained geometry instead of overflowing or guessing an output.
- Added source tests for global-to-parent target conversion and overflow rejection. Tests were not executed.
- Reactive reconfiguration after parent movement/output topology changes and physical multi-monitor validation remain open. No Cargo build, `cargo fmt`, check, test run or hardware validation was performed; grouped Fedora validation remains the plan.

### 2026-11-25 — Reactive popup reconstraining lifecycle

- Added `PopupPlacementRegistry` to retain the latest client `PositionerState` for each live XDG popup. It is updated on `reposition` and cleaned up when popup resources are destroyed or no longer alive.
- Output selection now uses Smithay's `PositionerState::get_anchor_point()`, honoring the client's anchor edges instead of approximating the anchor rectangle center.
- Added `reconstrain_reactive_popups`. It recomputes only popups whose positioner requested `reactive`, updates pending geometry, and uses `send_pending_configure` so no configure is sent when state did not change and Smithay still enforces protocol-version/reactivity rules.
- Hooked reconstraining into surface commits, output topology/configuration reconciliation, and the compositor-level `AureaCompositor::set_toplevel_location` movement entry point. The movement API also refreshes surface/output membership and pointer focus.
- Non-reactive popups are deliberately left unchanged per XDG semantics. Nested popup descendants are recalculated again after their parent acknowledges and commits its new geometry.
- The move/drag gesture itself is not yet implemented; this change establishes the required compositor API for future move handlers and does not claim interactive window dragging is complete.
- No Cargo build, `cargo fmt`, check, tests or hardware validation were run. Keep verification grouped on Fedora.
 
### 2026-11-25 — Context-menu dismissal semantics

- Added `DismissContextMenu` to the compositor/Shell interaction contract and a shared atomic open-state flag owned by the interaction sender.
- Escape is intercepted only while the desktop context menu is requested; the matching release and repeat are consumed by the existing key filter. Otherwise Escape is forwarded to the focused Wayland client.
- A primary click on the desktop canvas, other non-secondary canvas button presses, or any click over an application surface dismisses the desktop menu. A secondary click on the desktop repositions it.
- Shell dismissal is scoped to `ContextMenu` and cannot close Overview or App Exposé.
- Added source tests for shared visibility state, panel isolation and runtime dismissal. Tests have not been run.
- The menu remains a semantic request only: native panel rendering and menu-item hit testing/actions are still missing. No Cargo build, fmt, check or tests were run.

### 2026-11-25 — Desktop context-menu layout contract

- Added `DesktopContextMenuLayout` and typed `DesktopContextMenuCommand` values to `aurea-desktop-api`.
- The initial desktop menu contract contains Create Folder, Use as Wallpaper, Display Settings and Personalization. Geometry is logical (248 × 160), with 36-unit rows and 8-unit vertical padding.
- Placement reuses output-constrained `DesktopPanelPlacement`; hit testing returns commands only for rows and distinguishes panel padding. Added focused source tests for command order, edge placement, padding and undersized outputs.
- This is not yet wired to the compositor's shared menu state, Vulkan panel renderer or command dispatcher. No build, rustfmt, check or tests were run.

### 2026-11-25 — Context-menu labels and command handoff

- Added canonical Italian labels to `DesktopContextMenuCommand`; the wallpaper action is explicitly `Usa come sfondo`.
- Added `ContextMenuCommandSelected { command }` to the compositor/Shell interaction contract. Sending a selected command clears the shared open-state flag.
- ShellRuntime closes the context-menu panel and retains the semantic command for the owning desktop service. Filesystem and settings operations are intentionally not executed inside the shell event loop.
- Added source coverage for command handoff and panel dismissal. No build, rustfmt, check or tests were run.


### 2026-11-25 — Context-menu pointer hit testing and Noto Sans artwork

- The compositor/Shell interaction sender now retains the context-menu anchor in shared logical coordinates and clears it on dismissal or command selection.
- Pointer presses resolve the output containing the anchor and use the same output-constrained DesktopContextMenuLayout as the menu model. A primary click on a row sends a typed command; padding dismisses without activating; an outside click dismisses and continues normal desktop/Wayland routing.
- The menu press and matching release are consumed so the click cannot leak through to a window underneath.
- Added light/dark SVG masters for the four menu labels and simple line icons, plus `tools/assets/render-context-menu.sh`. Noto Sans (Google Fonts, SIL OFL 1.1) is already retained in Human Interfaces at `fonts/source/google-fonts/noto-sans/NotoSans-Variable.ttf`; the script now points Fontconfig directly at that repository font through a temporary configuration. No host font package or runtime font dependency is required; only the host SVG rasterizer and Python 3 are needed.
- Generated RGBA8 assets still need to be produced and visually inspected on Fedora. RenderScene, texture upload and Vulkan overlay drawing are not connected yet, so the menu is not visible. No build, rustfmt, check or tests were run.

### 2026-11-25 — Context-menu render-scene handoff

- `RenderScene` now carries an optional immutable `DesktopContextMenuLayout` alongside desktop snapshots.
- `AureaCompositor::render_scene` reads the shared anchor, resolves it against the enabled output layout, and attaches the constrained geometry to the frame scene. Pointer hit testing and future visual placement now share one geometry contract.
- Noto Sans is already retained in Human Interfaces; the export script now uses that repository font through a temporary Fontconfig configuration. No separate host font package is required.
- This is geometry handoff only. RGBA asset loading, Vulkan texture binding and overlay drawing remain outstanding; the menu is not yet visible.
- No build, rustfmt, check or tests were run, preserving the agreed batched Fedora verification workflow.

### 2026-11-25 — Vulkan context-menu overlay

- Extended the immutable render scene with the active desktop theme, so Light/Dark artwork selection is frame-consistent.
- Added lazy, one-time RGBA8 loading through the existing explicit-root asset loader. The selected snapshot is retained by the per-device pipeline cache and uploaded through the bounded desktop-artwork texture cache; no filesystem access is performed per frame.
- Added the Vulkan overlay draw using the existing alpha-blended textured pipeline. The panel is composited after application surfaces and before cursor imagery, with output-local geometry derived from the logical output origin and effective scale, plus per-output scissor clipping.
- Marked client-provided cursor surface snapshots separately and draw them after the menu, preserving pointer stacking. Native cursor rendering remains last.
- Runtime asset root: `AUREA_HUMAN_INTERFACES_ROOT`; development points it at `aurea-human-interfaces`, installed systems default to `/usr/share/aureaos/human-interfaces`. Missing assets are reported once and do not abort compositor startup.
- Added source tests for 1x/2x geometry and unrelated monitor clipping. Generated RGBA assets still need export and visual inspection. No build, rustfmt, check or tests were run.
- Updated the shared PNG-to-RGBA converter to premultiply PNG's straight alpha, matching the Vulkan pipeline's `ONE / ONE_MINUS_SRC_ALPHA` blend factors and avoiding bright fringes around antialiased transparent artwork.


### 2026-11-25 — Native cursor first-frame guarantee

- Found a first-frame presentation gap in the asynchronous native-arrow upload: the startup-centering frame could finish without drawing the cursor, and no follow-up frame was guaranteed when no further input arrived.
- The 32×32 RGBA cursor texture is now uploaded and completed once before the first frame that needs it. The texture remains cached for the Vulkan device lifetime, so later frames do not repeat the upload.
- This prevents an idle desktop from remaining cursor-less after startup. It is a source-level fix, not a claim of physical movement validation.
- No build, rustfmt, check or tests were run, as agreed; grouped Fedora validation remains pending.


### 2026-11-26 — Defer pointer placement until client grabs end

- Fixed an output hot-unplug lifecycle gap: when a client owned a pointer grab, the compositor correctly avoided moving the pointer but could lose the requested recenter operation.
- Added `pointer_recenter_pending` to retain output-driven placement requests blocked by an active grab.
- Output reconciliation now detects when the current pointer position falls outside the enabled-output union during a grab and records a deferred placement.
- The DRM Calloop loop retries after each dispatch only while a placement is pending. Once Smithay releases the grab, the pointer is centered on an enabled output and output frames are requested.
- No build, rustfmt, check or tests were run. Physical hot-unplug and popup-grab validation remain pending for the grouped Fedora session.


### 2026-11-26 — Per-device pointer preference and shake isolation

- Relative motion now carries its physical `InputDeviceId` into the compositor handler. Acceleration already used the effective per-device profile; shake-to-locate now reads that same device-scoped preference instead of only the global default.
- Added `PointerShakeManager` with independent unaccelerated-motion histories per mouse/touchpad. Device disconnect and per-device feature disable clear only the matching history.
- Added source tests proving that interleaved motion from two devices cannot combine into a false shake and that cleanup is device-scoped. Tests were not executed.
- No build, rustfmt or check was run.


## 2026-11-26 — Live mouse preference channel

### Changes committed to `main`

- Added `PointerSettingsController`, `PointerSettingsCommand`, and a channel factory to the technology-independent desktop API.
- Added the shell runtime entry point for device-scoped mouse settings and connected the channel during application startup.
- Forwarded the receiver through the desktop factory, compositor factory, Wayland server and Calloop-owned state.
- Added compositor-side command draining with validation: unknown devices and non-mouse devices are ignored; accepted profiles update the compositor's per-device pointer settings.
- The existing libinput acceleration adapter samples the new profile on the next pointer-motion event. No mouse libinput handles are retained, avoiding stale handles after hot-unplug.
- Added a unit test for shell-to-compositor command delivery and updated compositor-factory test doubles.
- Updated the input event coverage audit and Italian engineering diary.

### Deferred by design

- No temporary pointer settings UI was created.
- Pointer speed/profile and double-click controls remain scheduled for the future System Settings input page alongside touchpad settings. The existing desktop double-click interval was not changed.

### Verification status

Source changes were committed directly to `main`. No Cargo build, `cargo check`, `cargo fmt`, tests, CI result review or physical-device validation was run. The added tests are therefore not yet confirmed. Run the grouped Rust formatting/build/test checks on Fedora when the user is back at the development machine.



### 2026-11-26 — Auxiliary mouse buttons, wheel and session restore

- Confirmed that `BTN_SIDE`, `BTN_EXTRA`, `BTN_FORWARD` and `BTN_BACK` already flow through libinput's common pointer-button path and are sent to Wayland unchanged. Restricted desktop context-menu capture to primary and secondary clicks so auxiliary buttons are not swallowed by desktop chrome; added unit coverage for that boundary.
- Confirmed that horizontal and vertical wheel events are sent as Wayland axis frames, preserving continuous values, high-resolution `v120`, source and relative direction. Existing numeric validation tests remain in place.
- Added a compositor-owned registry of live mouse libinput handles. It is populated on mouse connection and cleared on removal/reclassification. Pointer preference commands now apply immediately to the active handle.
- Reapply mouse acceleration profiles after libseat session activation. Touchpad acceleration profiles are restored through their separate live-handle registry. Existing release of held buttons on session pause and device removal is retained.
- Physical hotplug, auxiliary buttons, wheel and VT pause/resume still need validation on Fedora hardware. No build, rustfmt, cargo check or tests were run in this source-editing session.


### 2026-11-26 — Persist mouse and pointer profiles across reconnection

- Added a dedicated pointer-profile store keyed by the privacy-preserving physical device identity, not the transient libinput/session device ID.
- Mouse and touchpad pointer acceleration profile, speed and shake-to-locate preference are restored when the same physical device reconnects. Touchpad-specific settings remain in their separate store.
- Pointer preferences are persisted under the user's XDG configuration directory in `aureaos/input/pointers.toml`, with owner-only file permissions and atomic replacement.
- Malformed, invalid or newer-version files are opened read-only and are never overwritten. Devices without a reliable persistent identity continue to use session preferences/defaults rather than being guessed by name or vendor.
- Added store round-trip, device-isolation, malformed/future-format and invalid-speed source tests.
- No Cargo build, `cargo fmt`, `cargo check`, tests or physical-device validation were run. Keep verification grouped on Fedora as agreed.


### 2026-11-26 — Professional pointer precision and high-rate event path

- Removed periodic synchronous `eprintln!` diagnostics from the relative pointer-motion dispatch and compositor hot path. High-polling-rate mice must not block the input event loop for diagnostic output.
- Added finite-value validation for accelerated and unaccelerated relative motion samples. Invalid NaN/infinite samples are discarded; valid floating-point sub-pixel deltas are not rounded or quantized.
- Restricted pointer acceleration application in the motion dispatch path to registered mouse and touchpad classes. Tablet and touchscreen input remain on their dedicated absolute/pen paths.
- Audited the current scroll path: horizontal/vertical continuous values, v120 values, axis source and relative direction are passed through to the compositor's Wayland axis frame. Existing axis handling validates values and retains high-resolution v120 data.
- Audited the tablet path: Smithay tablet-tool updates already forward pressure, distance, tilt, rotation, slider and wheel changes when reported by libinput; tablet-tool events are routed separately from mouse acceleration.
- Added `docs/development/INPUT-PRECISION.md` with professional peripheral requirements, current implementation boundaries and a Fedora hardware-validation matrix.
- No Cargo build, `cargo fmt`, `cargo check`, unit tests, benchmarks or physical-device validation were run. Keep verification grouped on Fedora as agreed.


### 2026-11-26 — Durable pen-tablet calibration profiles

- Added `TabletProfileStore` in `input/pointing/tablet_store.rs`, keyed by the existing privacy-preserving physical tablet identity rather than session `InputDeviceId` or kernel event number.
- Persisted active area, rotation, aspect mode and device aspect ratio in `aureaos/input/tablets.toml` under the user's XDG configuration directory.
- Integrated restore into tablet discovery/reconnection and persistence into the existing set/reset calibration API.
- Added format-version validation, calibration revalidation on load, owner-only Unix permissions, atomic replacement, file/directory sync, and read-only preservation of malformed, invalid or future-version files.
- Failed writes roll back the store's in-memory mutation. Devices without reliable physical identity remain configurable for the current session without guessed persistence.
- Added store tests for profile round-trip, per-device isolation, malformed/future format preservation, invalid calibration rejection and reset persistence.
- **Monitor assignment is not yet persisted.** Runtime `OutputId` is explicitly not a persistent monitor identity, and the current logical output registry does not retain the validated DRM connector identity evidence. Persisting `OutputId` would risk silently assigning a tablet to the wrong screen after reboot or topology changes. See `docs/development/TABLET-PROFILES.md` for the safe follow-up architecture.
- No Cargo build, `cargo fmt`, `cargo check`, unit tests or hardware validation were run. Keep verification grouped on Fedora as agreed.


### 2026-11-26 — Continue and harden the pointer/input block

- Kept work within the existing pointer subsystem; no new desktop feature block was started.
- Removed synchronous `eprintln!` output from pointer-button and scroll-axis dispatch. These are frequent input events and must not block the libinput event loop. Discovery/removal diagnostics remain event-scoped.
- Hardened both mouse-pointer and touchpad persistent profile stores: failed writes now restore the previous in-memory value (or remove a newly inserted value), preventing memory/disk divergence.
- Extended failure-path tests to assert the rejected profile is not retained in memory when a malformed, future-version or otherwise unreadable settings file makes the store read-only.
- Updated `INPUT-PRECISION.md` to reflect the already-added persistent tablet calibration store and the current pointer hot-path guarantees.
- Existing implementation still provides per-device button aggregation and synthesized releases on disconnect/session pause, side-button capability detection, per-device relative acceleration, multi-axis/high-resolution scroll forwarding, and safe explicit absolute-device output assignment.
- No build, `cargo fmt`, `cargo check`, unit tests, benchmark or hardware validation was run. Keep the complete verification pass grouped on Fedora as requested.


### 2026-11-26 — Saturday visible-pointer readiness

- Confirmed the current native path end-to-end at source level: server initializes the Smithay seat pointer; the default named cursor is converted to the built-in 32×32 RGBA arrow; render scenes use the current seat pointer location; Vulkan uploads/caches the arrow texture and draws it after desktop surfaces; relative libinput motion updates the seat pointer and requests output frames; startup centers the pointer and requests an initial frame.
- Added render-scene tests for the default named cursor, absent pointer location and explicitly hidden cursor state.
- Added Vulkan cursor-geometry tests for the arrow intersecting the primary output and being clipped from another output.
- Hardened startup: when one or more active outputs exist, failure to center the pointer now aborts startup with a clear error rather than silently leaving it at a potentially off-screen default coordinate. Added one startup diagnostic line with active output count and placement result.
- Added `docs/development/SATURDAY-POINTER-BOOT-TEST.md` with the exact Fedora formatting/check/test sequence, native launcher, physical mouse acceptance criteria and diagnostic triage.
- **Not yet verified:** no build, rustfmt, cargo check, unit tests, native Fedora session, Vulkan presentation or physical mouse test has run in this pass. Saturday success is not claimed until the native test confirms a visible arrow and physical motion.


## 2026-11-26 — Live ISO Factory: systemd-boot and UKI path

- Confirmed the approved Roma 1.0 boot policy in `docs/architecture/SYSTEMD-BOOT-AND-UKI.md`: UEFI-only, systemd-boot primary, UKI boot artifacts, Secure Boot disabled, no GRUB in the installed/default product path. This work is limited to a development Live ISO; installer, disk partitioning, Btrfs installation and recovery are explicitly deferred.
- Added `tools/iso/os-release` for AureaOS identity metadata embedded in the UKI.
- Added `tools/iso/scripts/build-systemd-boot-live-iso.sh`. It takes the exact versioned kernel/initramfs pairs staged by Debian Live, builds UKIs with Trixie's `ukify` and systemd EFI stub, creates a FAT EFI boot image with systemd-boot at the UEFI removable-media fallback path, writes systemd-boot entries, and remasters the ISO with an El Torito EFI boot image and hybrid GPT data.
- The final image is UEFI-only. The intermediate GRUB payload created by live-build is removed before final ISO mastering; no GRUB configuration is intended to ship in the final image.
- Updated the Fedora/Podman wrapper to install Debian Trixie's `systemd-boot-efi`, `systemd-ukify`, `mtools` and `dosfstools` in the disposable builder container.
- Updated the ISO coordinator to call the systemd-boot/UKI assembly stage and the image validator before copying the candidate to the user's output directory.
- Replaced GRUB-specific image validation with checks for the EFI fallback loader, systemd-boot configuration, UKI contents and embedded `boot=live`, the Debian Live filesystem, and the ISO El Torito EFI catalog.
- Updated `ISO-FACTORY.md` to describe the Live-only scope and new boot path. The existing Aurea desktop binary and PAM/logind session service remain the runtime payload; the image builder does not substitute a demo desktop.
- Debian reference checks: Trixie's `live-boot` requires `boot=live` and uses initramfs-tools to locate the `/live` filesystem; Trixie packages systemd-boot EFI binaries and systemd-ukify. The UKI generator embeds kernel, initramfs, os-release and the controlled command line.
- **Not verified yet:** no ISO build, shellcheck/syntax run, ukify invocation, FAT-image inspection, El Torito inspection, Cargo checks, QEMU/OVMF boot or physical hardware boot was run. The previous `/scripts/live` kernel panic remains unresolved until the new UKI image is actually booted. The new preflight and final-image checks reduce the chance of repeating the missing-initramfs-script failure but do not prove boot success.
- Keep the established workflow: accumulate source changes, then run grouped Fedora `cargo fmt`, workspace check/tests, build the ISO, inspect logs and test via Ventoy on hardware. Do not generate an ISO during this source-editing pass.


### 2026-11-26 — ISO Live preflight staging correction

- A follow-up source review found that the chroot hook invoked `/usr/local/libexec/aureaos/validate-chroot-boot.sh`, but the coordinator created the include directory without copying the validator into it. A fresh build could therefore fail at the preflight invocation even when the script existed in the repository.
- Updated `build-live-iso.sh` to stage `tests/validate-chroot-boot.sh` into the chroot include tree with executable permissions before live-build runs.
- Strengthened final UKI validation to require the `.linux`, `.initrd`, `.osrel`, and `.cmdline` PE sections, the embedded `boot=live` argument, and the AureaOS `ID=aureaos` identity.
- Corrected ISO Factory EN/IT documentation: final Live media carries the squashfs payload plus the UKI, not separate kernel/initramfs files; only UEFI boot is in scope.
- No build, shell syntax check, ukify invocation, ISO generation, QEMU/OVMF or hardware test was run. This remains a source-level correction awaiting the grouped Fedora validation.

### 2026-11-26 — Live ISO candidate publication and provenance

- Hardened `tools/iso/build-live-iso.sh` so it no longer copies an unvalidated ISO over the stable `AureaOS.iso` name.
- The coordinator now validates the binary tree, copies the finished ISO to a hidden temporary candidate, validates that exact ISO, calculates its SHA-256 and byte size, then publishes a uniquely named historical candidate containing the UTC build timestamp and source commit.
- Added a JSON manifest beside each candidate with the source remote/branch/commit, working-tree cleanliness, Debian Trixie amd64 target, systemd-boot/UKI policy, kernel version, ISO size/hash and explicit automated/hardware validation states.
- The stable `AureaOS.iso` convenience alias and checksum are atomically replaced only after both automated image validations pass. Existing uniquely named candidates and manifests are never overwritten.
- The Podman Debian builder now explicitly installs Python 3 for JSON manifest generation.
- Installer, disk partitioning, Btrfs installation, Aurea Update and Aurea Recovery remain out of scope. This milestone is only the real repository desktop in Debian Live mode.
- **Not verified:** no shell syntax check, Python manifest execution, live-build, UKI generation, ISO validation, QEMU/OVMF or physical boot was run. The previous `/scripts/live` panic remains unresolved until the new image boots on real hardware.
- Follow-up static review found and fixed an ISO-validator path mismatch: the EFI image is now extracted to `boot/efi.img`, matching the validator's subsequent inspection path. Commit: `9de7855e`. This correction is not yet runtime-tested.
- Additional ISO Factory hardening: the Fedora Podman wrapper now chooses a default build workspace keyed by the source commit and refuses a dirty or untracked repository. This avoids resuming a completed chroot that contains an older desktop binary after the source commit changes. Resume is for the same clean commit only. Commits: `52d51009`, `3f04e842`. No build or runtime validation was run.


### 26 November 2026 — Debian Live factory hardening: isolated assembly and exact UKI initramfs gate

- The user's scope clarification: the factory must reliably deliver a booted Debian 13 Trixie Live base; after that handoff, compositor, Vulkan, desktop session and visible UI are AureaOS work. Do not repeatedly rework the known Debian boot chain without a failing observation.
- The UKI assembler previously removed kernel/initramfs and GRUB files directly from `BUILD_ROOT/binary`. If xorriso or validation failed, a later `RESUME=1` could no longer find the kernel/initramfs. It now creates a same-filesystem hard-link snapshot with `cp -al`, mutates only that snapshot, validates it, and masters the ISO from it. The resumable live-build tree remains intact on both success and failure.
- Fixed a temporary ISO naming regression during the staging refactor and removed a duplicate cleanup trap; the temporary output again uses the process ID. The helper's status output now names `boot/efi.img` inside the ISO rather than a temporary file that is deleted at exit.
- The final validator now inspects the actual SquashFS rootfs for Debian 13 Trixie identity, systemd PID 1 and installed `live-boot`, `live-boot-initramfs-tools`, `live-config`, `live-config-systemd` and `systemd-sysv` packages.
- More importantly, it extracts the `.initrd` section from every exact UKI with Debian Trixie's `ukify inspect --section=...:binary@...`, runs `lsinitramfs` on that extracted payload and requires `scripts/live`. This checks the bytes that firmware/systemd-boot will actually launch, not only an earlier initramfs in the chroot.
- The builder container now installs `initramfs-tools-core` for this final inspection. The chroot boot preflight runs before build-only dependency purge so a failed preflight can be retried without first losing the toolchain.
- The Podman host preflight now checks `sudo` explicitly.
- Static source changes only. No `bash -n`, ShellCheck, `ukify`, live-build, SquashFS validation, QEMU/OVMF or physical boot has been executed in this session. The Debian base gate is therefore implemented but not yet observed passing.

### 26 November 2026 — End-to-end Live image audit and corrections

- Audited the source chain from Debian Live packages and initramfs preflight through UKI assembly, systemd session registration, desktop launch, compositor startup and runtime artwork lookup.
- Found missing target build dependencies: the production desktop always builds `aurea-media`, whose C shim links TurboJPEG, and the ISO enables the `heif` feature. Added `libturbojpeg0` (runtime), `libturbojpeg0-dev` and `libheif-dev` (build-time) to the target package list. The development headers are removed after compilation; the runtime TurboJPEG library remains.
- Added a source-commit marker to each live-build workspace. The direct coordinator now refuses dirty source trees and refuses to resume a workspace whose recorded commit differs or whose marker is absent. This prevents stale chroot binaries from being published under a newer source manifest.
- The chroot hook now checks dynamic linkage with `ldd`, validates the installed systemd unit with `systemd-analyze verify`, and confirms that the desktop service is enabled.
- Added build-time export of the two context-menu RGBA8 assets from the canonical SVG masters in the disposable Debian builder. The coordinator stages only the raw artwork into the target rootfs; librsvg, Python, SVG/font sources never enter the installed image. Trash rasters remain deferred until the desktop-item draw pass consumes them.
- Corrected the Trash asset descriptor: it previously prefixed `aurea-human-interfaces/` even though the loader root already points at `/usr/share/aureaos/human-interfaces`, producing a duplicated path. The descriptor now resolves under `icons/generated/trash/`, with a regression assertion.
- Extended final ISO validation to inspect the actual SquashFS rootfs, verify the desktop binary, launcher, systemd unit, PAM session and enablement symlink, and check all 26 RGBA payloads and exact byte lengths. Added `squashfs-tools` to the Debian build container.
- Pinned `ukify` explicitly to the Debian UKI stub whose presence is checked by the assembly script.
- Debian Trixie package metadata showed that `live-config` also needs an init-system backend; because this image deliberately disables apt recommends, added `live-config-systemd` explicitly and extended the chroot preflight to require it.
- **Not verified in execution:** no shell syntax checks, Cargo build/tests, Debian live-build, UKI generation, SquashFS validation, QEMU/OVMF or physical boot have been run after these changes. The previous `/scripts/live` panic remains unresolved until the new ISO boots.
- **Desktop limitation:** repository documentation and source state still identify physical scanout/complete scene composition as unfinished. The Welcome Wayland client exists and renders a 720×440 preview surface after receiving XDG configure, but a past diagnostic log only reached “waiting for configure”; it is not evidence of a visible monitor window. Hardware proof must include receipt of configure, SHM frame submission, Vulkan/KMS presentation and visible output.

### 26 November 2026 — Cursor protocol-name integration groundwork

- Resumed from the cursor asset audit at commit `be813f20`. Confirmed that the repository contains 26 semantic SVG masters in both light and dark palettes, two manifests, the SVG/Xcursor build pipeline and Linux/toolkit aliases. The generated Xcursor theme files are build outputs; the Rust/Vulkan runtime still does not load them.
- Added `input/cursor_theme.rs`, a renderer-independent mapping from Smithay/W3C cursor icon names and the aliases emitted by the Xcursor builder to the canonical 26 AureaOS asset names. Unknown names explicitly fall back to `default`.
- Connected `RenderScene` to retain the selected canonical asset name for `CursorImageStatus::Named`. Added unit coverage for all 26 standard roles, Linux/toolkit aliases, unknown-name fallback and propagation of a named cursor into the render scene.
- Updated the cursor theme specification to distinguish the completed protocol-name mapping from the still-missing runtime image loading and Vulkan texture selection.
- Commits: `b106476f`, `c6d99c16`, `2fa29f93`, `3752f5e7`, `c1141660`, `28753031`.
- **Not verified in execution:** no Cargo build or tests were run, in accordance with the accumulated-work workflow. GitHub reported no check-runs for the latest commit at the time of inspection. The Vulkan renderer still uses the built-in arrow texture for all named cursor states; do not claim that the 26 cursor images are visible yet.
- **Next implementation block:** add a bounded Xcursor runtime loader that resolves the installed `AureaOS-Light` / `AureaOS-Dark` theme, selects the closest supported pixel size, validates image dimensions and hotspot, converts the chosen frame to the existing RGBA8 snapshot contract, and feeds the texture cache. Then add animated busy-frame scheduling and per-output fractional/high-DPI validation. Keep file I/O and Xcursor parsing out of the frame callback.

### 26 November 2026 — Xcursor binary decoder foundation

- Added a dedicated Vulkan-side Xcursor file decoder. It validates the file signature, header/version, table-of-contents bounds, image chunk type and size, dimensions (maximum 256×256), hotspot bounds, frame count and total file size (maximum 16 MiB) before reading image payloads.
- Converts Xcursor's little-endian ARGB CARDINAL pixels into the compositor's existing tightly packed RGBA8 snapshot representation. Preserves each frame's dimensions, hotspot and delay for subsequent texture-cache and animation work.
- Added synthetic fixture tests for successful ARGB conversion, hotspot/delay preservation, invalid magic, truncated pixel payloads, invalid hotspots and oversized images.
- Registered the decoder as a private Vulkan module. No third-party parser dependency or C library is introduced.
- **Not yet connected to the Vulkan cursor texture cache:** the renderer still uses the built-in arrow. The next change must add theme-path resolution, safe file loading, closest-size selection, cache ownership and actual draw geometry using the decoded hotspot. Animation frame scheduling remains a separate follow-up.
- No Cargo build or tests were run; this is source-only work accumulated for the planned verification pass.

### 26 November 2026 — Themed Xcursor assets connected to Vulkan rendering

- Replaced the built-in-arrow-only GPU resource with a per-device cache of named Xcursor assets, keyed by theme, cursor name and requested raster size.
- Added runtime theme lookup under `AUREA_CURSOR_THEME_ROOT` (default `/usr/share/icons`), bounded file-size checks, Xcursor decoding, nearest-size selection and upload of all frames for the selected size through the existing Vulkan SHM texture upload path.
- The render scene now preserves the global pointer anchor. Native cursor geometry uses the selected Xcursor frame dimensions and per-asset hotspot, while the desktop's existing Light/Dark theme selects the matching cursor palette.
- Missing/unavailable cursor packages fall back to the built-in high-contrast arrow. The fallback is resized to the requested physical raster size so it remains the intended logical size on HiDPI outputs.
- Updated Debian Live packaging to stage the canonical cursor sources and builder, install temporary `librsvg2-bin` and `xcursorgen` build dependencies, build both themes in the Debian chroot, install them under `/usr/share/icons`, verify all 26 canonical cursor files and remove the build-only dependencies.
- Updated cursor documentation with the runtime path, development override and remaining limitations.
- **Not verified in execution:** no `cargo fmt`, Cargo build/check/test, shell syntax check, Debian ISO build or visual hardware validation was run. This work was accumulated without per-change builds as requested.
- **Remaining:** the runtime uploads every frame for the selected size but currently draws frame zero only. Implement a frame scheduler for `busy` / `left_ptr_watch`, move file loading and decoding off the render callback, review cache memory across outputs, validate fractional/high-DPI geometry and run the full test suite on Fedora before claiming the integration is verified.

- Follow-up: cursor geometry and requested raster selection now use `RenderTarget::effective_scale()`, preserving the exact fractional output scale instead of approximating it from the legacy integer scale.
- Follow-up: the ISO's final SquashFS validator now extracts and checks both installed themes and all 52 canonical cursor files (26 per palette), in addition to the chroot hook's pre-publication checks.
- Latest source-only changes remain unbuilt and untested; no GitHub check-runs were reported for the documentation commit.

### 26 November 2026 — Asynchronous cursor decoding and animated Xcursor frames

- Added a dedicated bounded `aurea-xcursor-decode` worker. File metadata checks, file reads, Xcursor parsing and fallback-arrow raster generation now run outside the compositor render callback.
- The render thread polls completed decode results, creates RGBA8 snapshots, submits Vulkan uploads and polls their fences. It publishes the frame textures only after the upload is complete; no unconditional GPU wait remains in the cursor loading path.
- Each retained frame keeps its own dimensions, hotspot and Xcursor delay. The renderer selects the current frame using a monotonic timestamp and binds that frame's Vulkan texture.
- The event loop now keeps 16 ms output scheduling active for the animated `busy` cursor and during a three-second first-use warm-up window. This gives asynchronous decoding/upload time to finish even when no further pointer event arrives.
- **Not yet complete:** cache memory/LRU limits across cursor names and multiple output scales, reduced-motion behavior, possible `left_ptr_watch` animation, and full visual validation.
- **Not verified in execution:** no Cargo formatting, build, check, tests, shell validation or ISO build has been run. Continue accumulating source changes before the planned Fedora verification.

- Follow-up: added bounded native cursor texture residency (32 assets / 16 MiB) with least-recently-used eviction. A single animation is limited to 4 MiB; oversized animation data retains its first frame rather than exhausting the shared GPU texture budget.
- Added pure timing tests for Xcursor per-frame delays, exact frame boundaries, wraparound and zero-delay handling.
- Verified from the cursor manifest that `busy` defines 12 frames / 1100 ms, while `left_ptr_watch` is currently a static artwork asset; only `busy` is therefore scheduled continuously.

### 2026-11-26 — Startup pointer placement and shake-to-locate review

- Verified the DRM server builder initializes the Wayland pointer before output discovery, publishes the initial outputs, and then calls `center_pointer_on_primary_output()` before registering the Wayland socket. The default cursor image is `CursorImageStatus::default_named()`; the compositor's native cursor path has a first-frame guarantee, so the intended startup presentation is a visible AureaOS arrow at the center of the first enabled output.
- The current helper selects the first enabled output in registry order; it does not yet model a user-selected primary display. This is sufficient for a single display, but explicit primary-display identity should be added with display settings before relying on it for complex multi-monitor layouts.
- Added source tests for center-coordinate math, including negative monitor origins, odd logical dimensions and invalid geometry. These tests have not been executed.
- Verified that shake detection already exists and is enabled by default. It uses each physical device's unaccelerated relative motion, separate per-device histories, three direction reversals and a bounded time/distance window. The existing smooth scale animation peaks at 2.4× and returns to normal in about one second.
- Apple Support confirms that macOS supports the same locate gesture on both a mouse and a trackpad: quickly move the mouse or finger on the trackpad to temporarily enlarge the pointer. The user does not need to draw a circle; a quick back-and-forth shake is the documented interaction.
- **Not implemented yet:** a visual pointer trail/ghost effect. Apple's documented built-in feature is temporary enlargement; a trail would be an additional AureaOS visual treatment. The next implementation should keep a short, time-decayed history of pointer positions and render translucent cursor echoes behind the live pointer, with reduced-motion support and a per-device setting.
- Current detection is fed by the common libinput relative-motion path for registered mice and touchpads. Trackpad hardware validation remains pending; libinput gesture swipe/pinch streams are separate and must not trigger this effect.
- No Cargo build, rustfmt, check or tests were run.

## 2026-11-26 — Modular cursor locate trail foundation

Added `input/cursor_trail.rs` for a bounded three-position history with finite-coordinate validation and a 220 ms linear fade, plus `input/cursor_trail_animation.rs` to isolate trail frame-lifetime policy. Registered both modules in `input/mod.rs`. Unit tests cover sample bounds, newest-first ordering, fade/expiry, and invalid coordinates. The Vulkan drawing stage is intentionally not marked complete: alpha-correct cursor ghost rendering still needs a dedicated texture/shader path and scene integration. No build or tests were run, in accordance with the deferred verification workflow.

## 2026-11-26 — Vulkan cursor trail rendering path

Extended the locate-trail implementation from input history into render-scene snapshots and output-specific geometry. Pointer motion records global logical positions while locate animation is active; the event loop keeps requesting frames while recent samples remain visible. Added `surface_compositor/cursor_trail_geometry.rs`, reusing native cursor hotspot, output transform, fractional-scale and clipping calculations. Added a dedicated `cursor_trail.frag` shader and format-cached Vulkan pipeline; opacity is passed through fragment-stage push constants and impressions are composited before the live cursor. The regular surface pipeline and client cursor rendering remain unchanged. Still requires deferred Fedora verification: shader compilation, Rust formatting/checks/tests, Vulkan runtime blending validation, and multi-monitor testing. No builds/tests run yet.


### 2026-11-26 — First-frame native cursor warm-up correction

- Review found a startup timing gap: the three-second asynchronous Xcursor warm-up was initially started in AureaCompositor::new(), before DRM discovery, Vulkan initialization and output publication. On slower hardware those steps could consume the entire window; the first requested frame could then clear the desktop while the cursor texture was still decoding, with no further frame scheduled if the pointer remained idle.
- Added begin_cursor_asset_warmup() and call it only after active outputs are published and the pointer is successfully centered. It resets the three-second window and requests output frames immediately, so the decode worker and Vulkan upload can complete without requiring a mouse/trackpad event.
- This is a source-level startup fix. No build, rustfmt, Cargo check, tests or physical boot validation were run.


### 2026-11-26 — Immediate native pointer fallback

- The native cursor renderer previously withheld the compositor-owned cursor until the selected Xcursor theme file had been read, decoded and uploaded. A slow first load could therefore leave the first desktop frames without any visible pointer.
- Added a tiny built-in high-contrast arrow texture, uploaded once per requested raster size and retained in a bounded four-size fallback cache. It is drawn immediately while the themed cursor loads asynchronously; once the theme upload completes, the selected Xcursor asset takes precedence without changing pointer coordinates or hotspot handling.
- If the decode worker cannot be started or its queue is temporarily unavailable, the built-in arrow remains available instead of suppressing the pointer. The one-time fallback upload is synchronous and intentionally limited to a small cursor texture; themed file I/O and decoding remain on the worker.
- No build, formatting, tests or physical boot validation were run.

- Added GPU-independent unit tests for the built-in fallback raster: exact requested dimensions/hotspot/RGBA payload and lower/upper size clamping. These tests are committed but have not been executed.

### Native pointer-to-KMS path audit — 26 November 2026

A source-level audit followed the named-cursor path from startup through physical presentation:

- server/builder.rs publishes DRM-backed Wayland outputs, centers the seat pointer on the primary output, then starts the cursor warm-up. This ordering avoids consuming the warm-up during GPU/output discovery.
- compositor/rendering.rs snapshots the Smithay seat pointer location and named cursor role into RenderScene.
- render.rs distinguishes named compositor cursors from client-provided Wayland cursor surfaces. The former carry the global logical pointer anchor and canonical Xcursor asset name; the latter retain their own committed hotspot and SHM surface path.
- surface_compositor.rs calls ensure_native_cursor_texture before looking up the asset. A ready theme asset wins; otherwise the bounded built-in fallback is returned by the same native_cursor_asset lookup and is bound/drawn by the normal textured pass. Thus the fallback is not merely cached: it reaches the draw command.
- native_cursor_geometry converts the pointer anchor and asset hotspot into output-local geometry, then applies output origin, scale/fractional scale, transform and clipping. The geometry tests cover the default arrow, asset-specific hotspot and a pointer located on another monitor.
- Relative libinput motion is validated for finite values, constrained to the union of enabled logical output rectangles, routed through Smithay pointer.motion/pointer.frame, and followed by request_all_output_frames. Startup placement and absolute-device motion also request frames.
- event_loop.rs requests frames every 16 ms while the three-second first-use warm-up is active. server/rendering.rs consumes pending frames through the selected Vulkan direct-presentation or Vulkan-render/GBM-CPU-upload compatibility path. DRM page-flip/VBlank handlers retire the submitted scanout resources.

**Remaining limitation:** the three-second warm-up is a bounded polling window, not a worker-completion notification. If initial Xcursor file access, decode or upload takes longer than that window and the pointer is never moved, the built-in fallback remains visible until another frame is requested (for example by pointer motion). The pointer itself remains drawable and usable. A future improvement should wake/schedule the compositor from cursor-worker completion rather than extending the high-frequency warm-up arbitrarily.

**Verification boundary:** this was a repository source audit only. No Rust formatting, build, Cargo check/test, boot, physical mouse test or live KMS/Vulkan presentation test was run. The end-to-end path is structurally connected, but hardware operation is not yet proven.


### Follow-up — Render polling until native cursor upload completion

- Added a pending-work query to the native cursor texture owner. It remains active while an Xcursor decode request is queued or while one or more Vulkan upload fences are incomplete.
- Exposed that state through the surface pipeline cache and Vulkan backend.
- The DRM event loop now treats pending native cursor work as animation activity, requests output frames every 16 ms, and naturally returns to its 100 ms idle dispatch once decode and upload are complete.
- This polling is backend-independent: both direct Vulkan presentation and the CPU-upload compatibility route use the same per-GPU surface pipeline cache.
- The built-in fallback arrow remains available throughout. No build or runtime test was run in this source-only change; validation is deferred to the planned Fedora batch check.


## 2026-11-26 — Desktop artwork Vulkan preparation

Added `VulkanSurfacePipelineCache::prepare_desktop_item_texture`. It resolves the desktop item through the canonical asset descriptor (theme, Trash contents and effective output scale), loads the generated RGBA8 file beneath `AUREA_HUMAN_INTERFACES_ROOT` (default `/usr/share/aureaos/human-interfaces`), validates it through the existing loader, retains the CPU snapshot by texture key, and queues the texture in the existing Vulkan desktop-icon cache. Missing/unapproved item artwork returns `None` rather than substituting an unrelated icon. This is the preparation stage only: the per-output geometry and draw/binding path are not yet connected to `surface_compositor`; selection highlighting and visible desktop icon rendering remain pending. No build or tests were run.


## 2026-11-26 — Configurable desktop double-click interval

Replaced the fixed 400 ms comparison constant with per-compositor tracker state. The default remains 400 ms; the compositor now exposes setter/getter methods in milliseconds. The accepted range is clamped to 150–1000 ms, and changing the value cancels any pending first click to prevent mixing two timing policies. The 4-logical-pixel movement threshold is unchanged. Added source tests for interval updates and clamping. The Settings UI and persistence connection are still pending; no build or tests were run.

## 2026-11-26 — Global desktop pointer behavior channel and persistence

Added a dedicated global preference contract and channel for desktop pointer behavior, avoiding confusion with physical-device mouse/touchpad profiles. The channel is forwarded from `aurea-shell` through the desktop factory and compositor service into the Calloop event loop. The compositor now persists the double-click interval in versioned `desktop-pointer.toml` beneath the AureaOS input configuration directory, with atomic replacement and mode 0600. Malformed or newer-version files are opened read-only and are not overwritten. The stored interval is restored at startup; the default is 400 ms and accepted values are clamped to 150–1000 ms. The Settings UI and hardware validation are not yet implemented. No build or tests were run.

## 2026-11-26 — Mouse/touchpad block audit and Wacom deferral

- Re-audited the current main implementation and its wiring: mouse and touchpad preferences use distinct device-scoped channels from Shell to the compositor; live mouse changes update the retained libinput device, and touchpad changes apply through capability-aware libinput configuration.
- Confirmed physical-device persistence is separate from transient input event IDs. Mouse profiles are stored in pointers.toml, touchpad profiles in touchpads.toml, and global double-click behavior in desktop-pointer.toml; each store has version validation and safe atomic persistence.
- Confirmed touchpad profiles are restored before configuration on discovery and reapplied after libseat session activation. Pointer acceleration is reapplied for retained mouse and touchpad handles after session activation.
- Confirmed the System Settings UI is not part of this backend completion and remains a later task, as previously decided. No temporary settings UI was added.
- Release boundary: Wacom pen/tablet and Tablet Pad are deferred from AureaOS 1.0 acceptance. Tablet Pad controls remain unsupported through Smithay 0.7 and are not advertised. Existing touchscreen contact routing remains a separate input path.
- No cargo fmt, build, cargo check, tests or physical-device validation were run. The code-level audit does not replace grouped Fedora validation; do not claim runtime operation until that validation succeeds.

### 2026-11-26 — Cursor trail sampling regression coverage

Named the 24 ms cursor-trail sampling interval and added unit tests for high-frequency event throttling, sample ordering and full history/throttle reset after `clear()`. No build or tests were run; verification remains grouped for the planned Fedora session.

### 2026-11-26 — Cursor warm-up after hotplug and session resume

Centralized cursor warm-up in `center_pointer_on_primary_output()`. The initial asset-loading window now restarts after pointer recentering caused by output publication/hotplug or libseat session restoration, not only during the first server startup. This keeps frames flowing while a stationary pointer's texture is prepared for the newly available output. Removed the duplicate warm-up call from the DRM builder. Source-only change; not yet built or physically verified on Fedora.

### 2026-11-26 — Default cursor scene contract regression checks

Extended `RenderScene` tests: the default named cursor must preserve the global pointer anchor and canonical `default` asset name, in addition to its hotspot-adjusted origin. `CursorImageStatus::Hidden` must leave the native cursor location, anchor and asset absent. These checks protect the minimum visible-arrow path rather than adding features. Tests were added but not run; no build was started.

### 2026-11-26 — Native cursor readiness diagnostics

Added `tracing::info` diagnostics at the two readiness boundaries needed for first hardware validation: successful Vulkan upload of the built-in fallback arrow and publication of the themed Xcursor asset into the cache, including asset key, raster dimensions and frame count. Fedora logs can now distinguish asset-loading failures from geometry or KMS-presentation failures. No build or tests were run.
