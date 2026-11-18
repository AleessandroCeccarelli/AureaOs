<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Capture–Compositor Adapter

**Status:** Design contract — implementation pending  
**Scope:** Native display capture between the Vulkan compositor and `aurea-capture`

## 1. Goal

Deliver the final composed output to the capture pipeline without borrowing, delaying or changing the lifetime of a DRM/KMS scanout image.

The adapter is an optional consumer of completed compositor frames. It must not become a second renderer, own Wayland policy, or make presentation wait for the encoder.

## 2. Verified current renderer constraints

The current implementation establishes the following facts:

- `VulkanOutputScanoutBackend` allocates scanout images with `COLOR_ATTACHMENT` usage. The selected DRM FourCC/modifier pair is negotiated against the primary KMS plane.
- `VulkanDrmScanoutFrame` owns both the exported Vulkan image and its KMS framebuffer wrapper. The frame pool retains the pair until the DRM lifecycle coordinator confirms that KMS and Vulkan ownership have been released.
- `surface_compositor::render_and_release` records the final SHM surface composition, optional cursor and optional display-color pass into the scanout target, records release to the foreign queue family, then calls `submit_and_wait_with`.
- The scanout image is therefore released to foreign/KMS ownership before the presentation commit. It must not be sampled or read back by a later capture worker.
- The optional display-color intermediate is allocated only when the warmth effect is active. It is a reusable composition resource, not a capture-owned frame, and it currently lacks `TRANSFER_SRC` usage.
- `VulkanReadbackSubmission` requires an owned shared-image view, an image created with `TRANSFER_SRC` usage, and the image already in `TRANSFER_SRC_OPTIMAL` with the caller having synchronized preceding writes.
- `aurea-capture` currently accepts owned RGBA8/BGRA8 frames and provides a bounded, non-blocking frame queue. It has no dependency on Vulkan or the compositor.
- The compositor crate does not currently depend on `aurea-capture`.

These constraints rule out treating the KMS framebuffer or the optional color intermediate as a generally safe asynchronous capture source.

## 3. Required ownership model

A capture frame must have a resource lifetime independent of scanout:

1. The compositor owns a bounded per-output pool of dedicated capture targets.
2. A target is checked out only when a capture session is active and a slot is available. A busy pool causes a skipped capture frame, never a wait in the presentation path.
3. The target is a device-owned optimal-tiled image with a capture-compatible pixel format and explicit `COLOR_ATTACHMENT | TRANSFER_SRC` usage, after querying format support.
4. The final composed scene is written to the scanout target and the dedicated capture target within the same accepted GPU submission. The capture target is not exported to KMS.
5. The capture target remains leased until the asynchronous readback fence has completed and its pixel bytes have been copied into owned CPU memory.
6. Only then may the target return to the pool. The readback submission retains the image view and device until completion.
7. The capture worker converts the completed pixels to an owned `CaptureFrame` and attempts a non-blocking enqueue. Queue saturation drops the capture frame without affecting KMS presentation.

The pool, target lease and adapter must have separate responsibilities and bounded resource limits. No capture resource may be reclaimed while a GPU submission can still reference it.

## 4. Frame recording and synchronization

- Capture is opt-in per authorized session. No target allocation or additional render work is required while capture is inactive.
- The capture target's previous image state is tracked independently from the scanout image state.
- The command buffer must transition the capture target from its tracked state to color-attachment write, render the final scene, then transition it to transfer-source read.
- The transfer readback submission must be ordered after the composition submission on a synchronization path explicitly supported by the logical device. If separate submissions or queues are used, an explicit semaphore/fence dependency is required; CPU timing is not a substitute.
- The capture target must not be reused until the readback fence is complete and the CPU copy has succeeded.
- Submission failure with uncertain queue acceptance must retain or quarantine all referenced resources. It must not return a possibly in-flight target to the pool.
- Capture polling and CPU mapping happen on a capture worker. The compositor must never poll, map or wait for readback completion.
- Frame timestamps use a monotonic clock and are attached at a clearly defined capture point. Completed frames are retired in submission order.

The current composition path waits synchronously for its scanout submission. This design does not claim that presentation is already asynchronous; it isolates capture readback so it can be polled independently without introducing a second wait in the KMS path.

## 5. Pixel format and conversion

The current readback buffer helper supports only `R8G8B8A8_UNORM`. The capture target must therefore use that format only when the physical device reports the required optimal-tiling color-attachment and transfer-source features.

If that format is unavailable, capture must report an unsupported capability or use a separately implemented and tested conversion path. It must not reinterpret BGRA/XRGB scanout bytes as RGBA.

The adapter must preserve width, height, stride, pixel format and monotonic timestamp when constructing `CaptureFrame`. Any row packing or format conversion must be explicit and tested.

## 6. Dependency and security boundary

- `aurea-capture` remains technology-independent: it must not depend on Smithay, Vulkan, DRM or KMS.
- The compositor-facing adapter may depend on the capture core only after the workspace dependency direction and process boundary are deliberately selected. Do not introduce a reverse dependency from capture core to compositor.
- The compositor exposes frames only to an internal sink created for an authorized capture session. Source selection, user consent, grants, revocation and session policy remain outside the Vulkan resource pool.
- Stopping or revoking capture prevents new target checkouts and drains or safely cancels in-flight work without releasing GPU-owned resources early.

## 7. Backpressure and resource bounds

The implementation must independently bound:

- capture target count and total image allocation bytes per output;
- readback staging buffer count and allocated bytes;
- asynchronous submissions in flight;
- completed CPU frames waiting for the encoder.

A saturated stage skips or drops a capture frame and records the reason. It must not block rendering, grow memory without limit, or silently reorder timestamps.

Output resize, scale change, hotplug, device loss and session stop must retire targets only after all GPU/readback leases are resolved. Old-size targets may remain temporarily retained while in flight, but must not be checked out for a new geometry.

## 8. Implementation sequence

1. Add a small capture-target allocator and bounded target pool, independent of the DRM framebuffer pool.
2. Add an explicit capture-target lease/state contract, including safe handling of in-flight readback ownership.
3. Extend the final-scene recording path to optionally render into a checked-out capture target in the same GPU submission, without changing the scanout image's KMS lifecycle.
4. Connect the capture target lease to `VulkanReadbackSubmission` and `VulkanReadbackCoordinator` on a worker.
5. Add a narrow adapter that creates `CaptureFrame` values and performs non-blocking handoff to the capture queue.
6. Add unit tests for pool bounds, lease retirement, format capability rejection, frame ordering and backpressure; then validate Vulkan synchronization and resource lifetime on real hardware.

Each step is independently reviewable. Do not wire the shell, portal, PNG writer or video encoder until this boundary is validated.
