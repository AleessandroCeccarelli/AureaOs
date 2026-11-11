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

Smithay 0.7's Vulkan allocator is a useful reference and a possible transitional adapter, but it is not the target owner of AureaOS render targets. It creates and owns its own Vulkan logical device and does not expose its internal raw image as an AureaOS renderer resource.

The earlier adapter around Smithay's allocator may remain temporarily for compatibility experiments and comparison tests. It must not be treated as the production allocator for the AureaOS renderer.

AureaOS will implement its own allocator using the logical Vulkan device owned by the compositor. The native allocator will create, bind, track and destroy its own VkImage and VkDeviceMemory objects, and export DMA-BUF handles for KMS interoperability. It will expose an AureaOS-owned resource contract, not Smithay allocator types.

Smithay remains useful for DRM, Wayland and public DMA-BUF integration types. If a Smithay abstraction prevents AureaOS from meeting a documented requirement, the compositor may use a narrower adapter or a native implementation behind the same internal interface.

## Device identity

The Vulkan physical device used for capability discovery and allocation must correspond to the DRM device used for scan-out. Query both primary and render node identities through VK_EXT_physical_device_drm. A missing extension or unavailable identity is a reason to reject the direct path, not to guess.

When selecting a device for a known DRM node, match either its primary node or render node. The exact node used for KMS operations remains the primary DRM node owned by the DRM device manager.

The extension plan is specific to one physical device and is passed into logical-device creation. A DRM-bound backend must fail closed when required DMA-BUF interoperability extensions are unavailable.

## Format and layout contract

A DMA-BUF is not just a file descriptor. The importer must preserve and validate:

- FourCC format and DRM modifier;
- width and height;
- number of memory planes;
- each plane's FD handle, offset and stride;
- Y-inversion and other relevant buffer flags.

Never infer the Vulkan DRM modifier layout from FourCC alone. Never silently replace an explicit modifier with linear or implicit layout. Reject unsupported layouts explicitly. Multi-plane formats must be handled according to their Vulkan DRM modifier memory-plane binding rules, not by assuming one FD equals one Vulkan image plane.

## Synchronization contract

The full producer-to-consumer chain must use a consistent synchronization model. AureaOS will implement explicit synchronization where supported, with a documented and capability-checked fallback only when the complete producer/consumer path supports implicit synchronization.

Do not mix explicit and implicit synchronization assumptions. Buffer reuse and destruction must wait for all relevant GPU and KMS consumers to finish.

## Presentation sequence

The planned path is:

1. Discover DRM device and output capabilities.
2. Select the Vulkan physical device associated with that DRM node.
3. Negotiate required device extensions and intersect exact FourCC/modifier pairs.
4. Verify image-format properties and the requested usage for the intended dimensions.
5. Allocate a render target with the AureaOS-owned allocator and export its DMA-BUF.
6. Render using AureaOS-owned Vulkan resources and synchronization.
7. Import the DMA-BUF into DRM/KMS, create a framebuffer, and issue an atomic test-only commit.
8. Only after a successful test-only commit, schedule a real atomic commit/page flip.
9. Retire and recycle buffers only after GPU and KMS completion.

## Validation policy

Static capability intersection is not proof of successful allocation, import, framebuffer creation, atomic commit or presentation. Each stage needs its own error path and test.

Unit tests cover deterministic metadata and policy validation. Hardware tests cover Vulkan allocation, DMA-BUF export/import, synchronization, KMS framebuffer creation, atomic test-only commit, page flips, hotplug and device loss.

No renderer API may claim presentation support until the hardware path has been exercised on supported devices.
