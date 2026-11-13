<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS hardware requirements

## Installation floor

The initial AureaOS installation profile requires:

- AMD64 / x86-64 with the x86-64-v3 feature level.
- At least 8 GiB of system RAM.
- At least one hardware Vulkan 1.3-capable integrated or discrete GPU.
- A working Linux DRM device for display output.

CPU vendor is not a policy input: Intel and AMD processors are accepted when the running Linux kernel exposes the complete x86-64-v3 feature set. The installer must check the active CPU feature flags, not infer support from a marketing name or CPU generation.

The preflight script is `tools/installer/check-hardware-requirements.sh`. It is deliberately separate from `tools/build/bootstrap-debian.sh`: creating a Debian root filesystem on a build host must not be blocked by the build host's hardware.

## GPU policy

AureaOS does not whitelist GPU vendors or model names. Intel, AMD and NVIDIA GPUs are evaluated by their actual Vulkan and DRM capabilities. On hybrid laptops, every DRM device is discovered independently and a Vulkan physical device is paired with the corresponding DRM node when the driver exposes a trustworthy device identity. The compositor must never guess a GPU-to-display association.

The current preflight checks that at least one non-CPU Vulkan physical device reports Vulkan 1.3. This is only an installation screening check, not a certification of the full graphics path.

Full runtime eligibility additionally requires the selected display/render device to pass the AureaOS Vulkan feature plan, DMA-BUF import/export checks, compatible DRM FourCC/modifier negotiation, allocation/import, synchronization and atomic KMS presentation. A Vulkan 1.3 API version alone is not sufficient.

A hybrid laptop may expose a display-connected integrated GPU and a discrete render GPU. Cross-GPU rendering is not assumed merely because both GPUs are present. Until explicit cross-device DMA-BUF import, synchronization and copy paths are implemented and tested, AureaOS uses only a verified same-device path for direct scanout. Unsupported paths must fail clearly rather than silently selecting an unrelated GPU.

## Debian boundary

Debian remains the amd64 distribution base and keeps its own package ABI baseline. The x86-64-v3 requirement applies to the AureaOS installation target, not to rebuilding Debian packages or compiling the entire Debian base with `-march=x86-64-v3`.

The current repository does not yet contain a graphical installer. The preflight script is a standalone, testable gate intended for integration into the future installer before disk changes begin.

## Validation status

- [x] Vendor-neutral x86-64-v3 CPU flag check.
- [x] Minimum 8 GiB RAM check.
- [x] Hardware Vulkan 1.3 presence check (software-only Vulkan is rejected).
- [x] Unit-style shell tests for accepted and rejected hosts.
- [ ] Per-device Vulkan extension and image-usage validation in the installer.
- [ ] Cross-GPU DMA-BUF import and synchronization.
- [ ] Intel, AMD and NVIDIA physical scanout matrix.
- [ ] Hybrid-laptop display/render routing matrix.
- [ ] Physical atomic KMS scanout tests.

The preflight must not be described as full hardware certification until the remaining items are implemented and tested on physical machines.
