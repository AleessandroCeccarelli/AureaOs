# AureaOS Vulkan renderer foundation

Status: the Vulkan device, resource, rendering-pass, offscreen-readback and DRM/KMS lifecycle foundations are present on `main`. A compositor-owned diagnostic scanout path is now connected to the Wayland server startup: it clears a DRM-compatible Vulkan image to a blue diagnostic color, exports it through DMA-BUF/GBM, registers a KMS framebuffer and submits it to the output's primary plane. This code path has not yet been validated on a physical display. GitHub Actions validates compilation and offscreen rendering only; it does not validate physical DRM/KMS scanout.

## Ownership and device baseline

AureaOS owns physical-device selection, logical-device creation, queue and command-pool lifetime, device-extension negotiation, image allocation, external-memory interoperability, GPU submission scheduling, rendering resources and synchronization. Smithay remains an integration layer for Wayland and DRM plumbing.

The renderer selects only GPUs exposing Vulkan 1.3, Dynamic Rendering and Synchronization 2. These features are queried on each candidate physical device and explicitly enabled during logical-device creation. A Vulkan 1.3 instance alone is not treated as proof that a physical GPU supports 1.3.

## Image and attachment resources

The native allocator creates and binds VkImage/VkDeviceMemory using the AureaOS logical device. Image views are separately owned and borrow their source image, preventing image destruction while a view is alive.

The DRM path negotiates an exact FourCC/modifier/usage combination, queries DMA-BUF export support, creates a modifier-tiled image, allocates dedicated exportable memory and exports an owned FD with plane layouts queried from Vulkan. DMA-BUF import and KMS framebuffer registration are implemented as separate lifecycle stages; their presence does not by itself mean that the interactive renderer presents frames.

## Dynamic rendering and synchronization

AureaOS uses Vulkan 1.3 Dynamic Rendering instead of a fixed VkRenderPass/VkFramebuffer object graph for the initial compositor path. The rendering scope supports a color attachment and balances vkCmdBeginRendering/vkCmdEndRendering while borrowing the attachment view.

Image layout transitions are explicit Synchronization 2 barriers. The helper covers color-attachment, sampled-read, transfer and general-use states. It intentionally does not maintain global image state: the frame graph/resource owner must serialize conflicting uses and provide the correct previous layout. Dynamic rendering itself is not a synchronization primitive.

## Shaders and graphics pipeline

SPIR-V modules are validated for basic container invariants before Vulkan module creation and are owned through RAII. The graphics-pipeline builder creates a dynamic-rendering-compatible triangle-list pipeline with empty vertex input, dynamic viewport/scissor and no descriptor sets or push constants yet. Shader modules must belong to the same Vulkan logical device as the pipeline.

The compositor includes GLSL 450 vertex and fragment shader sources under `components/aurea-compositor/shaders/`. The vertex shader generates a triangle from `gl_VertexIndex` and interpolates RGB colors; no vertex buffer or descriptor set is required. The fragment shader writes the interpolated color. The reproducible Fedora compilation helper is `tools/shaders/compile-vulkan-shaders.sh`; generated SPIR-V files are kept outside source control.

This is still not a complete scene renderer: texture descriptors, sampled Wayland surface buffers, color management, damage tracking and full scene composition remain to be integrated.

## Frames in flight and display ownership

The Vulkan frame scheduler uses a bounded ring of independent frame slots. Each slot owns a transient command pool and completion fence. The default is two frames in flight, configurable from one to three. Acquiring a slot waits only for that slot's previous GPU submission. Submitting queues work with its fence without waiting for GPU completion.

GPU completion and display release are separate lifetime events. The DRM frame coordinator and presentation backend track framebuffer ownership and KMS completion per output, including the synchronization requirements for handing Vulkan images to KMS. The initial diagnostic path now drives these stages through the active output renderer and the Wayland event loop. Framebuffers are retained until KMS reports the page flip and Vulkan reacquires foreign ownership before retirement.

## Current implementation and validation

The repository currently contains:

- Vulkan physical-device selection and logical-device creation.
- Device-extension and rendering-feature negotiation.
- Native image allocation, image views and explicit image transitions.
- Dynamic-rendering clear and draw passes.
- A bounded frames-in-flight scheduler.
- Offscreen rendering and host-visible pixel readback.
- GLSL sources and an opt-in GPU smoke test for the offscreen triangle.
- DRM DMA-BUF layout handling, framebuffer registration and per-output KMS presentation lifecycle components.
- A connected first-frame Vulkan-to-DRM scanout path that clears the output to a diagnostic blue color.
- Unit tests for the Vulkan resource, scheduler and DRM presentation lifecycle.

The latest checked GitHub Actions run for `main` passed all four build/test commands:

- `cargo check --workspace --locked`
- `cargo check -p aurea-compositor --features drm --locked`
- `cargo test --workspace --locked`
- `cargo test -p aurea-compositor --features drm --locked`

The formatting check now passes. The workflow also compiles both GLSL shaders and runs the previously opt-in triangle smoke test on Mesa lavapipe. The test completed successfully: it created a Vulkan 1.3 device, rendered the triangle, read back the RGBA8 pixels and verified both the clear corner and triangle interior. The CI runner is a hosted Ubuntu VM; this validates offscreen GPU rendering through software Vulkan, not physical DRM/KMS scanout on the target laptop.

## Remaining work

1. Run the diagnostic blue frame on the target Fedora laptop and validate physical DRM/KMS scanout.
2. Verify GPU-to-KMS synchronization, page-flip completion, framebuffer retirement and session pause/resume on physical hardware.
3. Import Wayland client surface buffers into Vulkan sampled-image resources and descriptors.
4. Build scene composition, damage tracking and the full interactive frame scheduler.
5. Implement the actual visual shell as a Wayland client; the current `aurea-shell` crate coordinates application lifecycle and events but does not yet draw the desktop interface.

## Offscreen execution and readback

`VulkanOffscreenRenderer` provides a synchronous diagnostic execution path for a headless Vulkan device. It allocates an RGBA8 image with color-attachment and transfer-source usage, creates a host-visible staging buffer, records rendering and image-to-buffer copy commands, inserts a transfer-write-to-host-read buffer barrier, submits the command buffer and waits for its fence, then maps and returns the pixel bytes. Non-coherent host memory is invalidated before CPU access. This path is intended for GPU validation (including Mesa lavapipe); it is not the interactive DRM/KMS presentation path.

The opt-in GPU smoke test lives in `components/aurea-compositor/tests/unit/vulkan_offscreen_triangle.rs`. After compiling the shaders on Fedora:

```sh
sudo dnf install glslang
bash tools/shaders/compile-vulkan-shaders.sh
cargo test -p aurea-compositor --features vulkan --test unit -- --ignored
```

The test checks the output byte count, an opaque-black corner outside the triangle and a non-black opaque pixel at the triangle center. It remains ignored in ordinary test runs so hosts without Vulkan are not required to provide a GPU. GitHub Actions explicitly enables it with Mesa lavapipe after discovering the installed ICD.
