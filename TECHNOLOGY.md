# AureaOS Technology Principles

## Platform

AureaOS Roma 1.0 targets Debian GNU/Linux 13 (Trixie) on `amd64` (x86-64). The project is GUI-first and Wayland-first: the intended desktop is designed for Wayland. The desktop and its applications remain planned and are not yet implemented.

The repository currently contains a bootstrap script and package manifests for a minimal, terminal-only Debian root filesystem. This is an early development artifact, not a complete AureaOS system image. See [Debian Base](docs/architecture/DEBIAN-BASE.md) for implementation details and limitations.

## Technology selection

Choose components for the needs of AureaOS and its users. Open-source and proprietary software or firmware may be considered when technically justified and legally distributable. Evaluate each candidate for:

- Compatibility and reliability on the target platform.
- Security, maintenance, and upstream support.
- Performance and resource use.
- Usability, accessibility, and integration.
- License, attribution, redistribution, patent, and other applicable terms.
- Long-term sustainability and available alternatives.

Third-party components retain their own licenses. GPL-3.0 applies to original AureaOS work as stated in the repository license; it does not automatically apply to third-party components.

## Build on established infrastructure

AureaOS should reuse mature Linux infrastructure for capabilities already served well by the ecosystem. Project development should focus on the integrated desktop experience and the AureaOS-specific parts that are missing.

No single implementation language, toolkit, or application technology is selected for every component. Native and web technologies may be evaluated where they fit the component's integration, performance, accessibility, maintenance, and security needs.

## One coherent experience

Components from different projects should work together through consistent interaction patterns, accessible interfaces, clear system behavior, and maintainable integration. Visual consistency must not depend on fragile changes to third-party applications.

## Updates and distribution

The Debian base and AureaOS-specific system components are intended to use separate update channels. Their update mechanisms, trust model, compatibility rules, and recovery behavior still need to be designed and validated.

Flatpak and Flathub are planned for distributing applications through Aurea Store. Application provenance, permissions, maintenance, and trust remain part of the design; a graphical store alone does not guarantee that an application is safe.

## Compatibility

The desktop direction is Wayland-first. Compatibility decisions should consider user needs, security, reliability, and maintenance cost. Any exceptions to the intended desktop direction must be evaluated and documented rather than assumed.

## Guiding principle

Choose suitable, maintained technologies. Reuse proven foundations, build what is missing, and describe decisions and implementation status accurately.
