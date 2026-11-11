# AureaOS Vulkan device foundation

Status: native device, image, and dynamic-rendering foundations implemented in draft changes; Fedora compilation and hardware validation pending.

## Ownership

AureaOS owns physical-device selection policy, logical-device creation,
queue and command-pool lifetime, device-extension negotiation, image
allocation, external-memory interoperability, GPU submission scheduling,
synchronization and KMS presentation. Smithay remains an integration layer
for Wayland and DRM plumbing and may provide adapters where its abstractions
are useful.

## Device and rendering-feature policy

The logical-device path verifies that the selected physical GPU supports
Vulkan 1.3, Dynamic Rendering and Synchronization 2. Both Vulkan 1.3 features
are explicitly enabled during logical-device creation. AureaOS does not infer
feature availability from the instance API version alone.

The compositor creates an immutable extension plan for the selected physical
device. The plan records which required DMA-BUF extensions are available and
which are missing. Only supported names are passed to logical-device creation.

The DRM-bound Vulkan backend fails closed if the selected physical device
does not expose the complete DMA-BUF interoperability extension set. Generic
offscreen Vulkan initialization may still proceed with the supported subset,
but still requires the renderer's Vulkan 1.3 feature baseline.

## Native image and DMA-BUF path

The native image foundation creates and binds VkImage/VkDeviceMemory using
AureaOS's logical device and owns both resources through RAII. The DRM path
queries support for the exact format, modifier, usage and DMA-BUF export
handle; creates a modifier-tiled image with dedicated exportable memory;
exports an owned DMA-BUF FD; and queries each memory-plane offset and row pitch
from Vulkan.

## Dynamic rendering

AureaOS uses Vulkan 1.3 Dynamic Rendering instead of introducing a fixed
VkRenderPass/VkFramebuffer object graph for the initial compositor path.
Color attachment views are owned separately from their underlying images.
A rendering scope records vkCmdBeginRendering/vkCmdEndRendering and keeps the
attachment view borrowed for the lifetime of the scope.

The scope does not perform image-layout transitions. Callers must record
explicit Synchronization 2 barriers before and after attachment use. The
rendering scope is not itself a synchronization primitive.

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
will succeed. Each operation needs a separate capability query and runtime
error path.

The current dynamic-rendering scope supports one color attachment. It does
not yet create graphics pipelines, load SPIR-V shaders, bind descriptor sets,
record draw calls, or composite Wayland surfaces. DMA-BUF import,
cross-API explicit synchronization, KMS framebuffer creation, atomic TEST_ONLY
commit and page-flip retirement are not yet implemented.

## Next modules

1. Add Synchronization 2 image-layout transition helpers.
2. Add shader-module loading and pipeline-layout ownership.
3. Add a graphics pipeline compatible with the dynamic-rendering attachment format.
4. Record a first clear-and-draw command sequence and validate it on Fedora.
5. Connect scene elements to textures and compositing.
6. Integrate KMS presentation and buffer retirement.
