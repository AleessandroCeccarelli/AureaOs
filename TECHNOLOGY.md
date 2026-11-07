

<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Technology Principles

## Platform

AureaOS Roma 1.0 targets Debian GNU/Linux 13 (Trixie) on `amd64` (x86-64).

The project is GUI-first and Wayland-first. Its intended desktop experience is designed around Wayland rather than a traditional X11 desktop.

The repository has progressed beyond the initial Debian bootstrap. It includes early Rust-based system components, including `aurea-shell` and `aurea-compositor`, as well as foundational DRM device discovery and display-output management.

These components are under active development. AureaOS does not yet provide a complete graphical desktop, a finished system image, or a usable daily operating system.

See [Debian Base](docs/architecture/DEBIAN-BASE.md) for implementation details and limitations.

## Technology selection

Choose components for the needs of AureaOS and its users. Open-source and proprietary software or firmware may be considered when technically justified and legally distributable. Evaluate each candidate for:

- Compatibility and reliability on the target platform.
- Security, maintenance, and upstream support.
- Performance and resource use.
- Usability, accessibility, and integration.
- License, attribution, redistribution, patent, and other applicable terms.
- Long-term sustainability and available alternatives.

Third-party components retain their own licenses. GPL-3.0-or-later applies to original AureaOS work as stated in the repository license; it does not automatically apply to third-party components.

## Build on established infrastructure

AureaOS should reuse mature Linux infrastructure for capabilities already served well by the ecosystem. Project development should focus on the integrated desktop experience and the AureaOS-specific parts that are missing.

**Rust is the primary programming language for AureaOS development**, including first-party system applications and components where appropriate. A different language may be used when a component has a clear technical reason; document the exception and its maintenance implications. This policy does not require third-party software to use Rust.

### Current compositor foundation

The current `aurea-compositor` implementation uses:

- **Smithay** as its Wayland compositor framework.
- **Wayland** for the display protocol.
- **Calloop** for event-loop integration.
- **libseat** for session and device access management.
- **DRM and Udev** for graphics-device discovery and output-device lifecycle foundations.

These technologies provide the foundation for the compositor. Their presence does not mean that all graphics, display, input, or session-management capabilities are complete.

In particular, full DRM/KMS mode setting, frame rendering and presentation to physical displays, complete input handling, and a finished window-management experience remain unfinished.

Cross-GPU rendering and presentation are not considered implemented or validated.

### User interface technologies

No native UI toolkit has been selected.

Toolkit and web technologies may be evaluated for individual components according to integration, performance, accessibility, maintenance, and security needs.

CSS is used for the existing web theme tokens; it does not imply that AureaOS applications or a shared UI toolkit have been implemented.

Vulkan remains a candidate rendering technology, not a confirmed choice.

See the [Application and Component Catalog](docs/product/APPLICATION-CATALOG.md) for per-component language and technology status.

## One coherent experience

Components from different projects should work together through consistent interaction patterns, accessible interfaces, clear system behavior, and maintainable integration.

Visual consistency must not depend on fragile changes to third-party applications.

First-party components should follow the AureaOS design principles and shared design system wherever appropriate.

## Updates and distribution

The Debian base and AureaOS-specific system components are intended to use separate update channels.

Their update mechanisms, trust model, compatibility rules, and recovery behavior still need to be designed and validated.

Flatpak and Flathub are planned for distributing applications through Aurea Store.

Application provenance, permissions, maintenance, and trust remain part of the design; a graphical store alone does not guarantee that an application is safe.

## Compatibility

The desktop direction is Wayland-first.

Compatibility decisions should consider user needs, security, reliability, and maintenance cost. Any exceptions to the intended desktop direction must be evaluated and documented rather than assumed.

## Guiding principle

Choose suitable, maintained technologies. Reuse proven foundations, build what is missing, and describe decisions and implementation status accurately.

