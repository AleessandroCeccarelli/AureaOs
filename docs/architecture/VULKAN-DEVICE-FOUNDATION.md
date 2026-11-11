# AureaOS Vulkan device foundation

Status: native device and image foundations implemented in draft changes; Fedora compilation and hardware validation pending.

## Ownership

AureaOS owns physical-device selection policy, logical-device creation,
queue and command-pool lifetime, device-extension negotiation, image
allocation, external-memory interoperability, GPU submission scheduling,
synchronization and KMS presentation. Smithay remains an integration layer
for Wayland and DRM plumbing and may provide adapters where its abstractions
are useful.

## Device-extension policy

The compositor creates an immutable extension plan for the selected physical
device. The plan records which required DMA-BUF extensions are available and
which are missing. Only supported names are passed to logical-device creation.

The DRM-bound Vulkan backend fails closed if the selected physical device
does not expose the complete DMA-BUF interoperability extension set. Generic
offscreen Vulkan initialization may still proceed with the subset available.

The required Linux interop extensions are:

- VK_KHR_external_memory_fd
- VK_EXT_external_memory_dma_buf
- VK_EXT_image_drm_format_modifier

The extension plan is device-specific. It must never be reused to create a
logical device from another physical device.

## Native image and DMA-BUF path

The native image foundation creates and binds VkImage/VkDeviceMemory using
AureaOS's logical device and owns both resources through RAII. The DRM path
queries support for the exact format, modifier, usage and DMA-BUF export
handle; creates a modifier-tiled image with dedicated exportable memory;
exports an owned DMA-BUF FD; and queries each memory-plane offset and row pitch
from Vulkan.

This path still requires Fedora compilation and hardware validation. The
exported FD and plane metadata are not proof that KMS can create a framebuffer
or scan it out successfully.

## GPU frame scheduling

The Vulkan frame scheduler uses a bounded ring of independent frame slots.
Each slot owns its command pool and completion fence. The default is two
frames in flight, with a configurable range of one to three.

Acquiring a slot waits only for that slot's previous GPU submission and resets
its command pool. Submitting ends the command buffer and queues it with that
slot's fence without waiting for GPU completion. The CPU can therefore prepare
the next frame while the GPU processes the previous one.

The scheduler waits for the device only during teardown. It does not yet track
KMS page-flip completion and must not be treated as a complete buffer-retirement
solution. GPU completion and display release are separate lifetime events;
the future presentation backend must wait for both before recycling a buffer.

## Important limitations

Extension presence alone does not prove that a specific format, modifier,
image usage, memory allocation, DMA-BUF import/export, or KMS presentation
will succeed. Each operation needs a separate capability query and a
runtime error path.

DMA-BUF import, cross-API explicit synchronization, KMS framebuffer creation,
atomic TEST_ONLY commit and page-flip retirement are not yet implemented.
The renderer still does not draw and present a complete AureaOS scene.

## Next modules

1. Validate modifier-image allocation and DMA-BUF export on Fedora hardware.
2. Add a Vulkan render-pass/pipeline foundation and command recording.
3. Import external buffers with per-memory-plane binding and FD ownership.
4. Implement explicit GPU/KMS synchronization and buffer retirement.
5. Add KMS framebuffer creation, atomic test-only commit and page-flip lifecycle.
