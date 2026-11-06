<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<p align="center">
  <img src="assets/AureaOS-Social-Preview.jpg" alt="AureaOS — a golden A inspired by the Colosseum" width="800">
</p>

# AureaOS

**Take the simplicity people love in operating systems, and shape it to fit Linux.**

AureaOS is a Linux distribution project built on Debian GNU/Linux 13 (Trixie) for `amd64` (x86-64). Its first release is **AureaOS Roma 1.0**.

The project aims to make Linux feel natural, coherent, and welcoming through a GUI-first, Wayland-first desktop experience.

## Project status

AureaOS is an early-stage prototype. The repository contains a minimal Debian bootstrap, package manifests, design resources, and initial Rust components for the desktop session.

The project is not yet a usable graphical operating system. Its current work focuses on establishing the foundations of the AureaOS desktop, particularly the shell, compositor, display infrastructure, and system architecture.

### Shell

`aurea-shell` coordinates the desktop session lifecycle and shutdown through an event-driven architecture. It is responsible for session coordination and starts the compositor as a local component.

### Compositor

`aurea-compositor` is an initial Smithay-based Wayland compositor.

The current implementation includes:

- Initial Wayland server and client handling.
- XDG shell integration and basic toplevel tracking.
- Internal output descriptors, registration, and layout management.
- Initial DRM connector and display mode discovery.
- Publication of discovered outputs as Wayland globals.
- Initial Vulkan command-buffer lifecycle and submission infrastructure behind an optional feature.

These components represent infrastructure groundwork, not a complete graphics pipeline.

In particular:

- DRM output discovery does not mean that physical DRM/KMS mode setting is implemented.
- Publishing a Wayland output does not mean that the compositor can render and display frames.
- The Vulkan infrastructure does not yet provide a complete rendering and presentation pipeline.
- A complete input and window-management implementation is still pending.

The compositor has not yet demonstrated a complete end-to-end graphical session.

### Debian bootstrap

The Debian bootstrap creates a minimal, terminal-only Debian root filesystem.

It is a development artifact, not an installable or bootable AureaOS image. Package resolution and the complete bootstrap profile still require validation.

### Design resources

Visual tokens, themes, icons, illustrations, and wallpapers are present in the repository.

Their presence does not mean that a complete desktop interface or shared native application library has been implemented.

## Product direction

AureaOS is planned as a coherent desktop with graphical workflows for everyday use.

The terminal will remain available for advanced users and system administration, but ordinary system configuration and everyday tasks should be accessible through graphical interfaces.

The system is intended to support local accounts without requiring a cloud account.

Planned components include:

- **Aurea Desktop:** desktop shell, window management, and everyday desktop interactions.
- **Quaero:** global search for applications, files, folders, and settings.
- **Fides:** graphical file manager.
- **Aurea Store:** graphical application discovery and management, planned around Flatpak and Flathub.
- **Aurea Update:** system updates for AureaOS components, separate from Debian base updates and application updates.
- **Aurea Time:** system backup and restore, from individual files through the full system, using compatible USB disks or network storage.

These names describe planned work, not available features. Scope and implementation choices may change as the project develops.

## Technical direction

- Debian GNU/Linux 13 (Trixie), `amd64`.
- Btrfs is the selected default installed filesystem.
- Backup and restore are planned through Aurea Time, not native Btrfs snapshots.
- GUI-first and Wayland-first desktop architecture.
- Rust is the primary programming language for first-party AureaOS development where appropriate.
- Smithay is used as the foundation for the Wayland compositor.
- DRM/KMS is the planned foundation for physical display management.
- Vulkan is an optional rendering technology under development, not yet a complete rendering solution.
- Separate update channels for the Debian base and AureaOS-specific system components.
- Flatpak and Flathub are the planned application distribution path.
- Open-source and proprietary third-party components may be considered, subject to technical, licensing, and distribution review.

## Display architecture

AureaOS is developing **Aurea Display Intelligence**, a planned subsystem responsible for display capability discovery and display policy.

Its intended responsibilities include:

- Discovering display identity, supported modes, and hardware capabilities.
- Managing variable refresh rate (VRR) when supported by the complete graphics pipeline.
- Selecting suitable fixed refresh rates when VRR is unavailable or unsuitable.
- Applying per-display policies based on hardware capabilities and user preferences.
- Providing automatic and manual configuration through Settings → Display & Video.

This architecture is documented separately and is not yet implemented.

See [Display Intelligence](docs/architecture/DISPLAY-INTELLIGENCE.md).

## Development priorities

The immediate development focus is the compositor and its graphics infrastructure.

The intended progression is:

1. Establish reproducible builds with the optional DRM and Vulkan features enabled.
2. Validate physical or virtual output initialization.
3. Implement and demonstrate actual frame rendering and presentation.
4. Display a real Wayland surface.
5. Integrate input and basic window interaction.
6. Expand automated testing and document verified hardware support and limitations.

The roadmap records the broader development direction and current priorities.

## Documentation

- [Vision](VISION.md) — goals and intended user experience.
- [Philosophy](PHILOSOPHY.md) — project principles.
- [Product experience](PRODUCT_EXPERIENCE.md) — planned workflows and components.
- [Technology principles](TECHNOLOGY.md) — technical direction and selection criteria.
- [Roadmap](ROADMAP.md) — development status and priorities.
- [Application and Component Catalog](docs/product/APPLICATION-CATALOG.md) — authoritative inventory of first-party applications, tools, desktop components, services, and shared design resources.
- [Initial architecture](docs/architecture/INITIAL-ARCHITECTURE.md) — system layers and open decisions.
- [Display Intelligence](docs/architecture/DISPLAY-INTELLIGENCE.md) — planned display discovery and refresh-rate architecture.
- [Debian base](docs/architecture/DEBIAN-BASE.md) — current bootstrap implementation and limitations.
- [Debian package manifests](debian/packages/README.md) — package groups and profile status.
- [Planned applications](docs/product/PLANNED-APPLICATIONS.md) — candidate first-party applications.
- [Planned system tools](docs/product/PLANNED-SYSTEM-TOOLS.md) — printer, scanner, and resource-monitor requirements.
- [AureaOS Human Interfaces](docs/human-interfaces/README.md) — design resources and planned shared UI library.
- [Contributing](CONTRIBUTING.md) — ways to contribute.
- [License](LICENSE) — license for original AureaOS work.

## Guiding idea

**“La tecnologia è bella quando c’è e non si vede.”**

*Technology is beautiful when it is there, but you barely notice it.*

The interface should help people do what they want without getting in their way.

## License

Original AureaOS source code is released under the **GNU General Public License v3.0 or later (GPL-3.0-or-later)**.

Third-party software, dependencies, fonts, icons, illustrations, wallpapers, and other external assets remain subject to their respective licenses and terms. Their licenses must be respected and documented independently.
