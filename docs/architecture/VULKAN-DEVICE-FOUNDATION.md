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


## First color-pass recording

The renderer includes a minimal `VulkanClearPass`. It records a Synchronization 2 transition into `COLOR_ATTACHMENT_OPTIMAL`, begins Vulkan 1.3 dynamic rendering with a clear load operation, ends the rendering scope, and records the requested transition for the next consumer. The pass intentionally does not submit work: `VulkanFrameScheduler` remains responsible for frame-slot acquisition and submission. The clear pass validates finite color components and rejects an undefined final state before recording commands. This is a real color-attachment operation, but it is not yet a complete visible desktop frame: presentation, readback validation, and KMS scanout remain separate stages.


## Offscreen execution and readback

`VulkanOffscreenRenderer` provides a synchronous diagnostic execution path for a headless Vulkan device. It allocates an RGBA8 image with color-attachment and transfer-source usage, creates a host-visible staging buffer, records the clear pass and image-to-buffer copy, inserts a transfer-write-to-host-read buffer barrier, submits the command buffer and waits for its fence, then maps and returns the pixel bytes. Non-coherent host memory is invalidated before CPU access. This path is intended for real GPU validation (including Mesa lavapipe); it is not the interactive frames-in-flight or DRM/KMS presentation path.


## First graphics draw pass

`VulkanDrawPass` records a non-indexed draw into a single dynamic-rendering
color attachment. It validates non-zero vertex/instance counts, rejects an
undefined final image state, checks attachment usage, and verifies that the
pipeline belongs to the same logical device and targets the attachment format.
The pass records the transition into color-attachment layout, begins a
load-discard rendering scope, binds the pipeline, sets viewport/scissor, issues
the draw, ends rendering and records the requested final transition.

The pass does not own shader assets, submit commands or wait for completion.
A caller must provide compatible SPIR-V modules when creating the pipeline and
must keep all resources alive until GPU completion. The next step is to provide
a reproducible known-good shader pair and connect this pass to the synchronous
offscreen readback path for pixel-level GPU validation.


## First shader-driven offscreen triangle

The compositor now includes small GLSL 450 vertex and fragment shaders under
`components/aurea-compositor/shaders/`. The vertex shader generates three
positions and interpolated RGB colors from `gl_VertexIndex`, so no vertex
buffer or descriptor set is required. The fragment shader writes the
interpolated color.

Compile the sources reproducibly on Fedora with:

```sh
sudo dnf install glslang
bash tools/shaders/compile-vulkan-shaders.sh
```

The script targets Vulkan 1.3 and writes the two SPIR-V modules to
`target/aurea-shaders/` by default. The output directory can be overridden
with its first argument. Generated binaries stay out of source control; GLSL
is the maintained source of truth.

`VulkanOffscreenRenderer::render_triangle` accepts the compiled vertex and
fragment SPIR-V bytes, creates the shader modules and compatible graphics
pipeline, clears an RGBA8 target to opaque black, records a three-vertex draw,
copies the result to host-visible memory, waits for completion and returns
RGBA8 pixels. The background clear is intentional: pixels outside the
triangle are deterministic rather than undefined. This is a synchronous
diagnostic route; the interactive frame scheduler remains a separate path.

A GPU-backed smoke test should load the two generated files, create a Vulkan
1.3 logical device, call `render_triangle(128, 128, ...)`, and verify both a
black corner pixel and a non-black pixel in the triangle interior. Run it
first with Mesa lavapipe, then on the target GPU. No hardware execution is
claimed by this change.
