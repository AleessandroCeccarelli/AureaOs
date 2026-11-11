# AureaOS Vulkan and DMA-BUF boundary

Status: accepted architecture direction; implementation and hardware validation are in progress.

## Scope

AureaOS owns its rendering architecture. Smithay is an integration library, not the authority that dictates the lifetime, scheduling, or internal representation of AureaOS renderer resources.

The compositor keeps four responsibilities separate:

- DRM/KMS owns physical display discovery, connectors, CRTCs, planes, framebuffer import, atomic state and page-flip lifecycle.
- Vulkan device management owns physical-device selection, logical-device creation, queues, command pools, synchronization and renderer-owned images.
- Buffer interoperability owns DMA-BUF metadata validation, import/export and explicit synchronization contracts.
- Smithay adapters translate between Smithay's public types and AureaOS-owned interfaces. They must not leak Smithay implementation details through the renderer API.

## Smithay Vulkan allocator assessment

Smithay 0.7's `VulkanAllocator` is useful and should not be discarded merely because it owns an internal Vulkan logical device. It can allocate images with DRM modifiers, validate image usage for requested dimensions, and export `VulkanImage` as DMA-BUF. The exported DMA-BUF is the interoperability boundary.

The allocator does not expose its internal `ash::Device` or raw `VkImage` as a renderer-owned resource. AureaOS must therefore not record commands against an image it does not own.

For initial integration, AureaOS may use Smithay's allocator as a DMA-BUF producer and import the exported DMA-BUF into the AureaOS-owned Vulkan logical device. This must be treated as a real external-memory import, not as shared raw Vulkan handles.

A later AureaOS-native allocator may replace the Smithay allocator if it provides a concrete benefit. It must implement the same internal buffer-allocation contract and pass the same interoperability tests.

## Device identity

The Vulkan physical device used for capability discovery and allocation must correspond to the DRM device used for scan-out. Query both primary and render node identities through `VK_EXT_physical_device_drm`. A missing extension or unavailable identity is a reason to reject the direct path, not to guess.

When selecting a device for a known DRM node, match either its primary node or render node. The exact node used for KMS operations remains the primary DRM node owned by the DRM device manager.

## Format and layout contract

A DMA-BUF is not just a file descriptor. The importer must preserve and validate:

- FourCC format and DRM modifier;
- width and height;
- number of memory planes;
- one FD handle per plane as represented by the DMA-BUF object;
- each plane's offset and stride;
- Y-inversion and other relevant buffer flags.

Never infer the Vulkan DRM modifier layout from FourCC alone. Never silently replace an explicit modifier with linear or implicit layout. Reject unsupported layouts explicitly.

## Synchronization contract

The full producer-to-consumer chain must use a consistent synchronization model. AureaOS must choose and document either:

1. explicit synchronization, using supported Vulkan semaphore-FD / sync-file and DRM synchronization interfaces; or
2. a fully implicit synchronization path where every participating API and driver supports the required implicit fences.

Do not mix explicit and implicit synchronization assumptions. Buffer reuse and destruction must wait for all relevant GPU and KMS consumers to finish.

## Presentation sequence

The planned path is:

1. Discover DRM device and output capabilities.
2. Select the Vulkan physical device associated with that DRM node.
3. Intersect exact FourCC/modifier pairs and verify image usage for the intended dimensions.
4. Allocate a render target and export/import its DMA-BUF into the AureaOS renderer device.
5. Render using AureaOS-owned Vulkan resources and synchronization.
6. Import the DMA-BUF into DRM/KMS, create a framebuffer, and issue an atomic test-only commit.
7. Only after a successful test-only commit, schedule a real atomic commit/page flip.
8. Retire and recycle buffers only after GPU and KMS completion.

## Validation policy

Static capability intersection is not proof of successful allocation, import, framebuffer creation, atomic commit or presentation. Each stage needs its own error path and test.

Unit tests cover deterministic metadata and policy validation. Hardware tests cover Vulkan allocation, DMA-BUF export/import, synchronization, KMS framebuffer creation, atomic test-only commit, page flips, hotplug and device loss.

No renderer API may claim presentation support until the hardware path has been exercised on supported devices.
