<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Roma 1.0 — Hardware Requirements and Graphics Support Policy

This document defines the supported hardware floor for the first AureaOS desktop release. It is a capability policy, not a vendor/model whitelist. Display support and performance certification are defined in tiers: 1280×720 at 60 Hz is the minimum display-compatibility floor; 1366×768 at 60 Hz is the low-end laptop reference profile; and 1920×1080 or 1920×1200 at 60 Hz is the standard desktop/laptop profile. Higher resolutions, refresh rates, multiple displays, and heavier effects require correspondingly stronger hardware.

## Minimum installation and runtime floor

- **Architecture / CPU:** AMD64 / x86-64 with the complete x86-64-v3 feature level.
- **CPU family target:** Intel Core 8th generation (Coffee Lake) or newer, or AMD Ryzen 2000-series (Zen+) or newer, with at least 4 cores / 8 threads as the practical minimum target. Equivalent Intel/AMD CPUs are accepted when they meet x86-64-v3 and comparable sustained performance. Older x86-64-v3 CPUs may be technically capable of booting, but are not the Roma 1.0 performance baseline.
- **Memory:** at least 8 GiB RAM. 16 GiB is the recommended configuration for everyday browser, video, and multitasking use.
- **Graphics:** at least one hardware integrated or discrete GPU with Vulkan 1.3 and the required renderer features dynamicRendering and synchronization2.
- **Display:** a working Linux DRM/KMS driver and at least one connected, usable output.
- **Modesetting:** atomic KMS on every output enabled by AureaOS.
- **Buffer allocation:** GBM on the display's DRM device, with at least one primary-plane format allocatable for CPU writing and scanout.
- **Session:** working libseat-compatible access to the DRM device.

The x86-64-v3 CPU check is vendor-neutral and tests active kernel-exposed CPU feature flags, not marketing names. It is the architectural floor, not a promise that every processor at that floor will deliver the same browser or multitasking performance. The installer preflight is tools/installer/check-hardware-requirements.sh. It is deliberately separate from tools/build/bootstrap-debian.sh: creating a Debian root filesystem on a build host must not be blocked by the build host's hardware.

## Supported GPU families — practical baseline

The following families define the intended Roma 1.0 support starting points. A model is eligible only when its actual Linux driver reports the required Vulkan API/features and the DRM/KMS path works. A generation label or Vulkan loader presence is never sufficient proof.

| Vendor / class | Intended minimum family | Driver path | Notes |
|---|---|---|---|
| Intel integrated graphics | Gen9.5 / Kaby Lake–Coffee Lake class (e.g. UHD 620/630, Iris Plus) and newer | Intel i915 + Mesa ANV | Gen8/older Intel graphics are not the Roma 1.0 baseline. Newer Iris Xe and Arc are preferred. |
| AMD integrated graphics | Vega-class APUs and newer | amdgpu + Mesa RADV | Actual Vulkan features and KMS presentation must pass runtime checks. |
| AMD discrete graphics | Polaris / Radeon RX 400–500 family and newer | amdgpu + Mesa RADV | Vega and RDNA generations are preferred. |
| NVIDIA discrete graphics | Pascal / GeForce GTX 10 family and newer | NVIDIA proprietary Vulkan driver as the initial supported path; NVK may be enabled after validation | Maxwell and older are outside the intended Roma 1.0 performance baseline even where a driver can expose Vulkan 1.3. Turing/RTX 20 and newer are preferred. |

These family boundaries are product-support targets, not hard-coded PCI-ID allowlists. If a newer or otherwise unlisted GPU meets the complete runtime contract and passes the relevant certification tests, it can be supported without changing a model whitelist. Conversely, a model from a listed family is not certified merely by its family name.

The Intel Vulkan 1.3 conformant-product records include UHD 620/630-class and Iris Xe products; AMD RADV documents Vulkan 1.3 for GFX6–7 and Vulkan 1.4 for GFX8 and newer; NVIDIA's published Vulkan driver support includes Pascal and newer families. These upstream API statements do not replace AureaOS's own feature and presentation tests.

## Visual quality and performance tiers

Resolution is a display mode, not a proxy for overall hardware capability. Certification must account for GPU rendering throughput, memory bandwidth, CPU scheduling, video decode support, output refresh rate, and compositor frame times. The same visual-correctness and accessibility requirements apply at every tier.

- **Minimum display compatibility:** 1280×720 at 60 Hz, one display. The system must detect and use a valid mode, render the desktop correctly, and keep the interface legible. This is a compatibility floor, not the primary performance-certification target.
- **Low-end laptop reference:** 1366×768 at 60 Hz, one display. This is the primary low-end laptop certification profile for wallpaper, pointer, desktop composition, ordinary transitions, browser UI, and supported video playback.
- **Standard desktop/laptop profile:** 1920×1080 or 1920×1200 at 60 Hz, one display. Both 16:9 and 16:10 modes must be tested; this tier is not a minimum requirement for every laptop.
- **Advanced profile:** 2560×1440, 2560×1600, 4K, refresh rates above 60 Hz, and/or multiple displays. Each combination requires separate performance and presentation validation.
- **Recommended everyday configuration:** 16 GiB RAM, 4–8 modern CPU cores, and a GPU with at least 4 GiB dedicated VRAM where applicable. Integrated GPUs use shared system memory and must be evaluated by measured capability.

The compositor must measure output mode, render cost, and presentation timing rather than promise high-resolution or high-refresh performance on the minimum tier. Lower output resolutions reduce pixel workload, but do not compensate for missing Vulkan features, inadequate drivers, or an unsupported DRM/KMS presentation path.

AureaOS must preserve visual correctness across tiers. Adaptive policy may reduce expensive optional workload (for example, blur radius, animation complexity, or frame rate under sustained pressure) to maintain responsiveness, but must not silently substitute a broken or visibly degraded rendering path. The user-facing effects policy and its thresholds must be explicitly designed and tested.

## Vulkan and renderer policy

The native renderer currently requires Vulkan 1.3 core Dynamic Rendering and Synchronization 2. Both features are queried on each physical device and explicitly enabled during logical-device creation. A Vulkan API version alone is insufficient.

The compatibility-renderer enum/path is reserved for a separately implemented and validated legacy backend. It must remain unavailable until the complete renderer path—resource creation, render passes, pipeline compatibility, synchronization, wallpaper/surface/cursor composition, and presentation—has been implemented and tested. A render-pass helper alone does not qualify a GPU for compatibility mode.

The baseline rendering path is Vulkan scene composition followed by host readback and GBM/KMS presentation. DMA-BUF export, DRM format modifiers, explicit sync-file export, foreign queue-family ownership, and direct Vulkan scanout are optional acceleration capabilities, not minimum requirements. Their absence must select a compatible, validated fallback rather than disqualify an otherwise usable rendering GPU.

Atomic KMS and a CPU-writable GBM scanout buffer are currently mandatory for physical output. Outputs that do not meet those requirements are outside the Roma 1.0 support floor; they must be reported explicitly rather than treated as generic GPU failures.

## Multi-GPU and multiple displays

Rendering and presentation devices are identified independently. A system with multiple GPUs is supported only when AureaOS can establish a verified rendering-to-output path for each enabled display. Device ordering and vendor preference must never substitute for verified compatibility.

Each output is evaluated independently. One unsupported connector, plane, format, or modifier must not be treated as support for every other output. An incompatible output must produce an output-specific diagnostic without silently invalidating other usable outputs.

Cross-GPU rendering is not guaranteed merely because two GPUs are present. It requires a verified transfer/import and synchronization path. Until that path is implemented and validated, AureaOS must use a verified same-device path or report that the output is unsupported.

## Explicit exclusions from the initial support floor

- CPUs that do not expose x86-64-v3.
- Systems with less than 8 GiB RAM.
- Systems without a hardware Vulkan 1.3 GPU exposing dynamicRendering and synchronization2.
- Intel integrated graphics older than the intended Gen9.5 baseline.
- AMD graphics older than Vega (integrated) / Polaris (discrete) for the Roma 1.0 performance baseline.
- NVIDIA graphics older than Pascal for the Roma 1.0 performance baseline.
- Systems without a usable DRM/KMS output or atomic KMS on the selected output.
- Display devices for which GBM cannot allocate a CPU-writable primary-plane scanout buffer.
- Systems without a usable libseat-managed DRM session.

These are deliberate product-support boundaries for Roma 1.0. They are not claims that older GPUs cannot run Linux or that no Vulkan driver exists for them.

## Validation status

- [x] Vendor-neutral x86-64-v3 CPU flag check.
- [x] Minimum 8 GiB RAM check.
- [x] Hardware Vulkan 1.3 presence check (software-only Vulkan is rejected).
- [x] Runtime Vulkan 1.3 dynamicRendering and synchronization2 feature validation.
- [ ] Installer preflight check for the required Vulkan 1.3 features.
- [ ] Per-output atomic KMS and GBM CPU-scanout capability validation in installer preflight.
- [ ] Intel Gen9.5+, AMD Vega/Polaris+, and NVIDIA Pascal+ physical scanout matrix.
- [ ] Hybrid-laptop display/render routing matrix.
- [ ] Cross-GPU DMA-BUF import and synchronization.
- [ ] Physical atomic KMS scanout tests.
- [ ] Frame-time and visual-quality certification at 1366×768/60 Hz on low-end laptop reference hardware.
- [ ] Standard-profile certification at 1920×1080 and 1920×1200/60 Hz.
- [ ] Minimum display-compatibility test at 1280×720/60 Hz.

The installer preflight is not full hardware certification. Actual support claims require representative physical systems from each GPU family, current supported Linux drivers, and both integrated and discrete/hybrid configurations.
