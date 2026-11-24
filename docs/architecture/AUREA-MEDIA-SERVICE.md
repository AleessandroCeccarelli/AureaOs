# Aurea Media Service — initial architecture decision

- SPDX-License-Identifier: GPL-3.0-or-later
- Status: Proposed architecture; the legacy JPEG API passed the Debian 13 and Fedora container ABI/decode smoke tests. Local Fedora host verification and the remaining security/integration reviews are still pending.
- Scope: the first production consumer is the desktop wallpaper pipeline.

## Decision

AureaOS owns the media service, its public API, validation policy, resource limits, scheduling, error model, and integration with the desktop. It does not initially reimplement the JPEG entropy decoder, inverse DCT, chroma upsampling, and color conversion algorithms.

Use a small dedicated Rust workspace crate named `aurea-media` as an in-process service/library in the first phase. Do not create a standalone daemon or IPC protocol yet. A daemon would add process lifecycle, protocol versioning, transport, permissions, and failure recovery before there are multiple independent clients that need it.

The first JPEG backend is the legacy TurboJPEG C API from system libjpeg-turbo, called through a very small Aurea-owned C shim and private Rust FFI module. The compatibility probe has now compiled, linked and decoded the committed wallpaper in GitHub Actions containers for Debian 13 Trixie and Fedora. The tested packages were Debian `libturbojpeg0` / `libturbojpeg0-dev` `1:2.1.5-4` and Fedora `libjpeg-turbo` / `turbojpeg` `3.1.4.1-2.fc45`. Both environments resolved `libturbojpeg.so.0` at runtime and produced identical 3840 × 2160 RGBA8 output. This proves the tested legacy API surface and runtime linkage in those container images; the user's Fedora workstation still needs its own local verification before the adapter is approved.

## Responsibilities

### `aurea-media` crate

Owns:
- a stable, backend-independent Rust API;
- input and decoded-image limits;
- format identification and dispatch;
- decode scheduling contract and cancellation/error propagation;
- conversion to an owned, tightly packed RGBA8 result;
- backend selection behind private modules;
- tests for malformed input, dimension overflow, limits, output length and format policy.

Does not own:
- Vulkan objects, GPU upload, texture lifetime or render scheduling;
- wallpaper fit/crop/layout policy;
- desktop scene composition;
- image file selection or user preferences.

### Proposed module boundaries

Keep each file narrowly scoped; split further if a file grows beyond the project's normal size budget.

- `lib.rs`: public exports only.
- `image.rs`: backend-independent decoded image type and pixel layout.
- `limits.rs`: input, dimension, pixel-count and output-memory limits.
- `error.rs`: stable service errors; do not leak C-library error strings as the public API.
- `decoder.rs`: decoder trait and dispatch contract.
- `jpeg/mod.rs`: private JPEG backend module boundary.
- `jpeg/turbojpeg.rs`: all TurboJPEG FFI and unsafe calls, no Vulkan or compositor imports.
- `service.rs`: request orchestration and worker-facing API, if needed; do not add a thread pool until an actual caller requires one.

The first implementation now exists as the `components/aurea-media` workspace crate. Its public API exposes `decode`, `ImageFormat`, `DecodeLimits`, `DecodedImage`, `PixelFormat`, and stable `DecodeError` values. The only implemented format is JPEG. The native boundary is a narrow C shim compiled against the system `turbojpeg.h`; Rust FFI calls only Aurea-owned shim functions. TurboJPEG handles, constants and codec ABI details stay in C. The compositor remains a consumer and must convert the returned owned RGBA8 buffer into its local `WallpaperPixels` contract. `aurea-media` has no dependency on the compositor, Smithay or Vulkan.

## Decode contract — first JPEG milestone

Input:
- borrowed compressed bytes;
- explicit source format or a validated JPEG signature;
- caller-selected limits, never implicit unbounded allocation.

Output:
- width and height;
- tightly packed RGBA8;
- exact byte length = width × height × 4, using checked arithmetic;
- no GPU handles, no borrowed C buffers, no C allocator ownership crossing the FFI boundary.

Initial JPEG policy:
- accept baseline and progressive JPEG;
- accept RGB, YCbCr and grayscale inputs;
- reject CMYK/YCCK until a deliberate color-management path exists;
- reject malformed, truncated, unsupported and over-limit images;
- enable the backend's progressive scan limit and fail on decoder warnings;
- never decode on the Vulkan/render thread.

Initial conservative limits:
- compressed input: 64 MiB;
- each image dimension: 16384 pixels;
- decoded RGBA8: 256 MiB;
- also enforce a pixel-count limit and checked stride/length arithmetic before allocation.
These are initial policy values, not universal constants; the API should make limits explicit and testable.

## Current implementation boundary

- The API is synchronous by design. Callers must invoke it from a bounded worker; no thread pool or cancellation framework is introduced until a real caller defines those requirements.
- Default limits are 64 MiB compressed input, 16,384 pixels per axis, 64 million pixels, and 256 MiB decoded RGBA8 output. Callers can provide stricter limits.
- Header inspection occurs before output allocation. Rust uses checked arithmetic and `try_reserve_exact`; the C shim rechecks dimensions and exact output length before writing.
- The C shim enables `TJFLAG_STOPONWARNING | TJFLAG_LIMITSCANS`, accepts RGB, YCbCr and grayscale, and rejects CMYK/YCCK.
- The header is inspected twice (pre-allocation and immediately before decode). This intentionally avoids retaining a native decoder handle across the allocation boundary.

## Security and process isolation

The C codec is a parser for untrusted binary data. Unsafe FFI must be contained in one private module and every pointer, length, return code, and handle lifetime must be checked.

For the built-in wallpaper milestone, an in-process library called from a bounded background worker is acceptable. Before exposing decoding to arbitrary files, downloads, previews, thumbnails, or third-party applications, reassess process isolation. If threat modelling requires it, move the decoder backend behind a small restricted helper process with a versioned local IPC protocol. Do not pay the daemon/IPC complexity cost before that need exists.

## Debian/Fedora dependency policy

- Fedora is the development environment, not the dependency authority.
- Debian 13 Trixie is the target baseline.
- Fedora development headers are build-time only.
- The Debian runtime package must be included in the eventual system package/image dependency graph.
- Do not vendor a second codec copy or silently bundle a static copy.
- Verify actual headers, exported legacy symbols, link name and runtime SONAME on both distributions.
- Review package security updates and license notices at release time; do not pin an old codec version indefinitely.

## Implementation gates

1. **Passed in CI containers:** compile and run `tools/dependencies/jpeg/turbojpeg-abi-probe.c` against the committed wallpaper on Debian 13 Trixie and Fedora. Both reported `TurboJPEG legacy API: OK`, 3840 × 2160, 33,177,600 RGBA8 bytes and opaque alpha.
2. **Passed in CI containers:** link and load `libturbojpeg.so.0` in both environments. Compilation and execution exercise the required legacy declarations and symbols, including the progressive scan-limit flag.
3. **Pending:** repeat the probe on the actual Fedora development workstation and record its installed package build; inspect and record the target runtime package inclusion in the Debian image.
4. Review the probe and error paths before translating the minimal calls into Rust FFI.
5. Add the `aurea-media` crate and isolated backend only after gates 1–4 pass.
6. Unit-test the backend contract and run the workspace checks locally on Fedora, then verify the Debian runtime package in the target environment.
7. Only then connect the returned pixels to the Vulkan wallpaper texture path.

## Explicit non-goals for this milestone

- writing a JPEG codec from scratch;
- building a general image editor or transcoder;
- introducing PNG, WebP, AVIF or SVG decoders before a real requirement;
- creating a permanent media daemon or IPC protocol now;
- adding GPU code to the media crate;
- modifying the existing wallpaper renderer before decoded pixels are proven correct.


## Bounded decode worker

The crate now exposes an optional `DecodeWorker` for callers that need to keep synchronous decoding away from latency-sensitive threads.

- One worker thread processes requests sequentially.
- The waiting queue has capacity one; one request may be in flight and one may wait.
- Submission uses non-blocking `try_send`; a full queue is reported as back-pressure.
- Compressed input is checked against the caller's byte limit before enqueueing.
- Each accepted request returns a one-shot receiver for its decoded result.
- Dropping all worker handles closes the request channel; the worker exits after queued work is drained.
- The worker is backend-neutral and has no compositor, Smithay or Vulkan dependency.

The worker is infrastructure only at this stage. The compositor does not yet own or call it, and the decoded pixels are not yet connected to the Vulkan wallpaper upload cache. The next integration must define ownership and lifecycle outside the render callback, poll completed results without blocking, and discard stale results when the wallpaper selection/revision changes.


## Wallpaper decode service integration boundary

The compositor now depends on `aurea-media` and contains a wallpaper-specific `WallpaperDecodeService` in `desktop/wallpaper/decode_service.rs`. The service owns the bounded media worker, submits the selected asset's compressed bytes, polls results without blocking, validates the returned RGBA8 buffer through `WallpaperPixels`, and tags successful results with an asset ID and monotonically increasing revision. Selection changes invalidate pending and ready results; a queue-full submission can be retried by selecting the same asset again.

This service is not yet wired into `AureaCompositor::render_output` or `RenderScene`, and its decoded pixels are not yet submitted to `VulkanWallpaperTextureCache`. This separation is intentional: next we must add a narrow handoff from compositor-owned decode state to the frame scene, then let the Vulkan backend consume only completed, current-generation pixels. No decode, wait, or allocation should be added to the render callback.


## Vulkan wallpaper decode handoff

The Vulkan surface pipeline cache now owns a lazily created `WallpaperDecodeService`. During frame preparation, the selected `WallpaperAsset` is synchronized with that service, the worker is polled once without blocking, and a completed `DecodedWallpaper` transfers its `WallpaperPixels` buffer into `VulkanWallpaperTextureCache` without cloning the pixel allocation. The cache uploads the pixels asynchronously and exposes the texture only after its fence completes and only while the matching asset/revision remains selected. Queue saturation is treated as transient and retried on a later frame. Clearing the selection invalidates decode state and clears the selected texture.

The decode worker performs JPEG work off-thread. Vulkan image allocation and upload submission remain on the backend's resource-preparation path; GPU completion is polled rather than waited on by the CPU frame path.
## Wallpaper decoder integration — 2026-11-24

The compositor now owns one WallpaperDecodeService alongside its persistent Vulkan surface pipeline cache. The bounded media worker is created during cache initialization, not lazily from the frame-composition callback. Frame composition only submits/retries a bounded request, polls its one-shot receiver without waiting, and hands completed RGBA8 pixels to the bounded Vulkan upload cache.

The decode service records the current asset/revision as failed when decoding, pixel validation, or worker delivery fails. Re-selecting the same asset does not submit the same failing request on every frame; selecting a different asset clears the failure state and starts a new revision. Queue saturation remains transient and is retried on a later frame.

The renderer still performs GPU upload asynchronously and only exposes the texture after its fence completes and its asset/revision matches the active selection. Local Fedora visual verification remains pending; CI compilation/tests must be run after the current grouped implementation work is complete.

## Wallpaper upload ownership and failure handling — 2026-11-24

The decode service retains its completed `DecodedWallpaper` while Vulkan prepares the texture upload. `VulkanWallpaperTexture::submit_pixels` now borrows the validated pixel buffer and copies it into the staging allocation; it does not consume the CPU buffer before command setup and queue submission have succeeded. The pipeline calls `take_ready` only after the texture cache confirms that the upload was accepted.

If texture allocation, staging setup, command setup, or queue submission fails, the decoded pixels remain owned by the decode service. The texture cache records the failed asset/revision and does not retry the same failed upload on every frame. A different wallpaper revision clears that per-selection failure latch. Clearing the wallpaper also clears the latch. This is a bounded failure policy, not an automatic recovery mechanism: retrying the same selection after a transient resource failure still needs an explicit retry policy.

The retained CPU buffer remains subject to the existing 256 MiB decoded-image limit. Staging still performs the expected transfer copy; no second full RGBA clone is introduced.


## Wallpaper selection invalidation — 2026-11-24

The Vulkan wallpaper cache now invalidates the previously visible texture as soon as a different asset/revision key is selected, before any fallible image allocation or queue submission. This prevents an older ready texture from remaining eligible when preparation of the replacement fails. Already submitted uploads are not destroyed or cancelled: they remain fence-backed and owned by the cache until completion can be established, then are discarded if their key is no longer selected. Clearing the wallpaper similarly removes visibility immediately while preserving pending-upload lifetime safety.


## Fatal Vulkan device loss handling — 2026-11-24

Wallpaper fence polling now preserves `ERROR_DEVICE_LOST` as a typed surface-composition error instead of flattening it into a generic texture string. The DRM scanout error carries an explicit device-loss flag to the server rendering dispatcher.

When the direct Vulkan render path reports device loss, the dispatcher stops the output/session rendering path instead of attempting the CPU compatibility fallback. That fallback shares the same logical Vulkan device and cannot recover a lost device. The scanout renderer also skips framebuffer cancellation, reacquisition, and retirement when device completion is uncertain; those operations could issue further Vulkan work or release resources still referenced by the GPU.

This is a fail-stop policy, not device recreation. Recreating the Vulkan backend requires a higher-level lifecycle that rebuilds the device, dependent caches, frame pools, and output bindings. That lifecycle remains future work. No automatic retry is performed after `ERROR_DEVICE_LOST`.
