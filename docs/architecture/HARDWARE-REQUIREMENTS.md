<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Roma 1.0 — Minimum Hardware Requirements

This is the supported hardware floor for the first AureaOS desktop release. It applies to every supported computer; no GPU vendor, model, or developer machine is the reference platform.

## Installation floor

- **CPU:** AMD64 / x86-64 with the x86-64-v3 feature level.
- **Memory:** at least 8 GiB RAM.
- **Graphics:** at least one hardware Vulkan 1.3-capable integrated or discrete GPU.
- **Vulkan features:** the selected rendering device must support dynamicRendering and synchronization2.
- **Display:** a working Linux DRM/KMS driver and at least one connected, usable output.
- **Modesetting:** atomic KMS on every output enabled by AureaOS.
- **Buffer allocation:** GBM on the display's DRM device, with at least one primary-plane format allocatable for CPU writing and scanout.
- **Session:** working libseat-compatible access to the DRM device.

CPU vendor is not a policy input: Intel and AMD processors are accepted when the running Linux kernel exposes the complete x86-64-v3 feature set. The installer checks active CPU feature flags, not a marketing name or CPU generation.

The preflight script is tools/installer/check-hardware-requirements.sh. It is deliberately separate from tools/build/bootstrap-debian.sh: creating a Debian root filesystem on a build host must not be blocked by the build host's hardware.

## GPU and driver policy

AureaOS does not whitelist GPU vendors or model names. AMD, Intel, NVIDIA, and other GPUs are eligible only when their installed kernel and userspace drivers satisfy the platform contract. GPU vendor, PCI generation, integrated/discrete classification, or the presence of a Vulkan loader is not proof of compatibility.

The current preflight checks for at least one non-CPU Vulkan 1.3 device. This is an installation screening check only. Runtime device selection additionally verifies the Vulkan 1.3 feature plan, including dynamicRendering and synchronization2. A Vulkan API version alone is insufficient.

The baseline rendering path is Vulkan scene composition followed by host readback and GBM/KMS presentation. DMA-BUF export, DRM format modifiers, explicit sync-file export, foreign queue-family ownership, and direct Vulkan scanout are optional acceleration capabilities, not minimum requirements. Their absence must select a compatible fallback rather than disqualify an otherwise usable rendering GPU.

Atomic KMS and a CPU-writable GBM scanout buffer are currently mandatory for physical output. Outputs that do not meet those requirements are outside the Roma 1.0 support floor; they must be reported explicitly rather than treated as generic GPU failures.

## Multi-GPU and multiple displays

Rendering and presentation devices are identified independently. A system with multiple GPUs is supported only when AureaOS can establish a verified rendering-to-output path for each enabled display. Device ordering and vendor preference must never substitute for verified compatibility.

Each output is evaluated independently. One unsupported connector, plane, format, or modifier must not be treated as support for every other output. An incompatible output must produce an output-specific diagnostic without silently invalidating other usable outputs.

Cross-GPU rendering is not guaranteed merely because two GPUs are present. It requires a verified transfer/import and synchronization path. Until that path is implemented and validated, AureaOS must use a verified same-device path or report that the output is unsupported.

## Explicit exclusions from the initial support floor

- CPUs that do not expose x86-64-v3.
- Systems with less than 8 GiB RAM.
- Systems without a hardware Vulkan 1.3 GPU exposing dynamicRendering and synchronization2.
- Systems without a usable DRM/KMS output or atomic KMS on the selected output.
- Display devices for which GBM cannot allocate a CPU-writable primary-plane scanout buffer.
- Systems without a usable libseat-managed DRM session.

These are deliberate minimum requirements for Roma 1.0, not claims that every Linux graphics device is supported. Older hardware may become eligible in a later release if a separately maintained backend is implemented and validated.

## Validation status

- [x] Vendor-neutral x86-64-v3 CPU flag check.
- [x] Minimum 8 GiB RAM check.
- [x] Hardware Vulkan 1.3 presence check (software-only Vulkan is rejected).
- [x] Runtime Vulkan 1.3 dynamicRendering and synchronization2 feature validation.
- [ ] Installer preflight check for the required Vulkan 1.3 features.
- [ ] Per-output atomic KMS and GBM CPU-scanout capability validation in installer preflight.
- [ ] Cross-GPU DMA-BUF import and synchronization.
- [ ] Intel, AMD, and NVIDIA physical scanout matrix.
- [ ] Hybrid-laptop display/render routing matrix.
- [ ] Physical atomic KMS scanout tests.

The installer preflight is not full hardware certification. Actual support claims require validation on representative AMD, Intel, and NVIDIA hardware and on both integrated and discrete GPU configurations.
