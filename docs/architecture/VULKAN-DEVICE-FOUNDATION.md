# AureaOS Vulkan device foundation

Status: initial implementation in progress; hardware validation pending.

## Ownership

AureaOS owns physical-device selection policy, logical-device creation,
queue and command-pool lifetime, device-extension negotiation, image
allocation, external-memory interoperability, synchronization and KMS
presentation. Smithay remains an integration layer for Wayland and DRM
plumbing and may provide adapters where its abstractions are useful.

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

## Important limitations

Extension presence alone does not prove that a specific format, modifier,
image usage, memory allocation, DMA-BUF import/export, or KMS presentation
will succeed. Each operation needs a separate capability query and a
runtime error path.

The native image foundation creates and binds VkImage/VkDeviceMemory using
AureaOS's logical device and owns both resources through RAII. The DRM path
queries support for the exact format, modifier, usage and DMA-BUF export
handle; creates a modifier-tiled image with dedicated exportable memory;
exports an owned DMA-BUF FD; and queries each memory-plane offset and row pitch
from Vulkan. This path still requires Fedora compilation and hardware
validation. DMA-BUF import, explicit synchronization and KMS presentation are
not yet implemented.

## Next modules

1. Validate modifier-image allocation and DMA-BUF export on Fedora hardware.
2. DMA-BUF import with per-memory-plane binding and FD ownership handling.
3. Explicit synchronization and buffer retirement across GPU and KMS.
4. KMS framebuffer creation, atomic test-only commit and page-flip lifecycle.
