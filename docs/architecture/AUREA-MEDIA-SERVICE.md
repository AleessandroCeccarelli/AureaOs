# Aurea Media Service — initial architecture decision

- SPDX-License-Identifier: GPL-3.0-or-later
- Status: Proposed architecture; JPEG backend remains pending local ABI verification.
- Scope: the first production consumer is the desktop wallpaper pipeline.

## Decision

AureaOS owns the media service, its public API, validation policy, resource limits, scheduling, error model, and integration with the desktop. It does not initially reimplement the JPEG entropy decoder, inverse DCT, chroma upsampling, and color conversion algorithms.

Use a small dedicated Rust workspace crate named `aurea-media` as an in-process service/library in the first phase. Do not create a standalone daemon or IPC protocol yet. A daemon would add process lifecycle, protocol versioning, transport, permissions, and failure recovery before there are multiple independent clients that need it.

The first JPEG backend candidate is the legacy TurboJPEG C API from system libjpeg-turbo, called through a very small Aurea-owned Rust FFI module. This remains conditional on compiling and running the compatibility probe against the actual Fedora development host and Debian 13 Trixie runtime/development packages. Debian Trixie currently packages libturbojpeg 2.1.5; Fedora package streams reviewed are on 3.1.x. The API overlap must be proven, not assumed.

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

The compositor remains a consumer: it passes the selected wallpaper bytes to the media API and receives validated RGBA8 pixels. The existing wallpaper asset metadata and `WallpaperPixels` contract should be reused or moved only after a focused dependency-direction review. Do not duplicate the pixel buffer type.

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

1. Compile and run `tools/dependencies/jpeg/turbojpeg-abi-probe.c` on Fedora against the committed wallpaper.
2. Repeat against Debian 13 Trixie. The same source must compile and run on both.
3. Inspect the actual `turbojpeg.h` declarations and exported symbols; confirm the legacy API and scan-limit flag exist on both.
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
