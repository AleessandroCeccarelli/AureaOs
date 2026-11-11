# AureaOS Vulkan renderer foundation

Status: initial native rendering modules implemented in draft changes; Fedora compilation and hardware validation pending.

## Ownership and device baseline

AureaOS owns physical-device selection, logical-device creation, queue and
command-pool lifetime, device-extension negotiation, image allocation,
external-memory interoperability, GPU submission scheduling, rendering
resources, synchronization and KMS presentation. Smithay remains an
integration layer for Wayland and DRM plumbing.

The renderer selects only GPUs exposing Vulkan 1.3, Dynamic Rendering and
Synchronization 2. These features are queried on each candidate physical
device and explicitly enabled during logical-device creation. A Vulkan 1.3
instance alone is not treated as proof that a physical GPU supports 1.3.

## Image and attachment resources

The native allocator creates and binds VkImage/VkDeviceMemory using the
AureaOS logical device. Image views are separately owned and borrow their
source image, preventing image destruction while a view is alive.

The DRM path negotiates an exact FourCC/modifier/usage combination, queries
DMA-BUF export support, creates a modifier-tiled image, allocates dedicated
exportable memory and exports an owned FD with plane layouts queried from
Vulkan. Fedora compilation and hardware validation are still pending.

## Dynamic rendering and synchronization

AureaOS uses Vulkan 1.3 Dynamic Rendering instead of a fixed
VkRenderPass/VkFramebuffer object graph for the initial compositor path.
The first scope supports one color attachment and balances
vkCmdBeginRendering/vkCmdEndRendering while borrowing the attachment view.

Image layout transitions are explicit Synchronization 2 barriers. The helper
covers the initial color-attachment, sampled-read, transfer and general-use
states. It intentionally does not maintain global image state: the frame
graph/resource owner must serialize conflicting uses and provide the correct
previous layout. Dynamic rendering itself is not a synchronization primitive.

## Shaders and graphics pipeline

SPIR-V modules are validated for basic container invariants before Vulkan
module creation and are owned through RAII. The initial graphics-pipeline
builder creates a dynamic-rendering-compatible triangle-list pipeline with
empty vertex input, dynamic viewport/scissor, no blending and no descriptor
sets or push constants yet. Shader modules must belong to the same Vulkan
logical device as the pipeline.

This is pipeline infrastructure, not yet a complete scene renderer. There
are no bundled compositor shaders, texture descriptors, vertex data, draw
submission policy, color management or Wayland surface compositing yet.

## Frames in flight

The Vulkan frame scheduler uses a bounded ring of independent frame slots.
Each slot owns a transient command pool and completion fence. The default is
two frames in flight, configurable from one to three. Acquiring a slot waits
only for that slot's previous GPU submission. Submitting queues work with its
fence without waiting for GPU completion.

The scheduler does not track KMS page-flip completion. GPU completion and
display release are separate lifetime events; future buffer retirement must
wait for both before recycling a buffer.

## Remaining work

1. Validate all new modules with cargo fmt/check/test on Fedora.
2. Add a small known-good SPIR-V vertex/fragment shader pair and a clear/draw test.
3. Add descriptor-set and sampled-texture support for Wayland surface buffers.
4. Build the scene compositor and damage-aware rendering path.
5. Integrate DMA-BUF import, explicit GPU/KMS synchronization and buffer retirement.
6. Add KMS framebuffer creation, atomic TEST_ONLY commit and page-flip presentation.
