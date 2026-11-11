<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<p align="center">
  <img src="assets/AureaOS-Social-Preview.jpg" alt="AureaOS — a golden A inspired by the Colosseum" width="800">
</p>

# AureaOS

**Take the simplicity people love in operating systems, and shape it to fit Linux.**

AureaOS is a Linux distribution project built on Debian GNU/Linux 13 (Trixie) for `amd64` (x86-64). Its first release is **AureaOS Roma 1.0**.

AureaOS aims to make Linux feel natural, coherent, and designed to respect users. Debian provides the lower-level system foundation, while AureaOS develops its own desktop experience, system integration, visual identity, and core applications.

## Project Status

AureaOS is in an early prototype stage. The repository includes a minimal Debian bootstrap, package manifests, design resources, and initial Rust components for the desktop session.

The project has progressed beyond its initial bootstrap stage. Its first system components are being developed as independent, modular Rust projects.

The Rust workspace is organized under `components/` and currently contains four crates:

- **`aurea-desktop`** — the desktop entry point and composition root. It wires the desktop components together; it is not yet a finished desktop session or graphical interface.
- **`aurea-shell`** — coordinates session lifecycle, events, compositor startup, and shutdown.
- **`aurea-desktop-api`** — provides shared contracts between desktop components.
- **`aurea-compositor`** — implements the early Smithay-based Wayland compositor and owns compositor, graphics-device, and output responsibilities.

The shell, compositor, API, and composition root are separate components with explicit responsibilities. Their presence in the workspace does not mean that Aurea Desktop is complete.

The compositor also includes foundational graphics-device and display-output management:

- DRM device discovery and device lifecycle management.
- GPU inventory and rendering-preference foundations.
- Udev event handling for device discovery and removal.
- DRM device event integration with the compositor event loop.
- Display-output registration and removal.
- Wayland output publication and lifecycle management.

These capabilities represent an important step in the graphics architecture, but they do not yet constitute a complete display system.

AureaOS is not yet a usable graphical desktop. Several essential capabilities remain under development:

- A complete Vulkan rendering backend and frame presentation to physical displays.
- Full DRM/KMS mode setting and display configuration.
- Complete keyboard, pointer, touch, and gesture input handling.
- Full window management and desktop interaction.
- A finished graphical shell and integrated system applications.
- A complete, bootable AureaOS installation image.

The Debian bootstrap profile also requires further validation and does not currently produce a complete bootable installation image.

Visual tokens, themes, icons, illustrations, and wallpapers are present as design assets. Their presence does not mean a working desktop or shared application library exists.

## Product Direction

AureaOS is planned as a coherent, GUI-first desktop with graphical workflows for everyday use.

The terminal remains available for advanced users, developers, and administrators. Ordinary desktop operations should not require terminal commands.

The desktop should provide a consistent environment in which system tools, applications, settings, notifications, and everyday workflows feel like parts of the same operating system.

AureaOS aims to reduce unnecessary configuration and provide sensible defaults without preventing users from making meaningful choices.

### Core AureaOS Experience

The first release, Roma 1.0, focuses on an AureaOS-designed and project-controlled desktop experience and a selected group of first-party applications:

- **Aurea Desktop:** the graphical desktop environment, shell, and window management.
- **AureaOS Human Interfaces:** shared design system and the planned native UI component library.
- **Aurea Wallet:** a first-party system application and an important pillar of the AureaOS experience.
- **Fides:** the graphical file manager.
- **Quaero:** global search for applications, files, folders, and settings.
- **Aurea Store:** the official graphical application store.
- **Aurea Update:** system updates for AureaOS components.
- **Aurea Time:** system backup and restore, from individual files through the full system.

The scope and implementation of these components remain under development. Their inclusion describes the current Roma 1.0 product direction, not completed functionality.

### Third-Party Applications

AureaOS will not attempt to develop every desktop application from scratch for its first release.

Common application categories will initially be covered by established third-party software, including:

- Web browsers.
- Email clients.
- Office and productivity suites.
- Other useful desktop applications.

Aurea Store will integrate with **Flathub** as an application source and **Flatpak** for application installation, execution, sandboxing, and updates.

Third-party applications will run in their respective sandbox environments. They remain separate from AureaOS first-party applications and system components.

Future releases may introduce first-party alternatives for selected application categories when development resources and technical priorities make this practical.

## Technical Direction

AureaOS follows a modular architecture and aims to reuse established Linux infrastructure wherever it makes technical sense.

Its current technical direction includes:

- Debian GNU/Linux 13 (Trixie), `amd64`.
- Btrfs as the selected default installed filesystem.
- GUI-first and Wayland-first desktop architecture.
- Rust as the primary programming language for first-party AureaOS development where appropriate.
- Smithay as the current Wayland compositor framework.
- Wayland protocols for desktop communication.
- Calloop for event-loop integration.
- libseat for session and device access management.
- DRM and Udev for graphics-device discovery and output lifecycle foundations.
- Vulkan as the selected and exclusive graphics rendering API.
- PipeWire for multimedia infrastructure.
- NetworkManager for network management.
- systemd for system and service management.
- GRUB2 as the planned bootloader.
- Separate update domains for Debian base packages, AureaOS system components, and Flatpak applications.
- Flatpak and Flathub for third-party application distribution.

The selected technologies do not imply that their complete integration is already implemented.

In particular, Vulkan rendering, physical frame presentation, complete display configuration, and the final bootable system image remain under development.

Aurea Time will own system backup and restore. Btrfs native snapshots are not the planned backup and restore workflow.

Open-source and proprietary third-party components may be considered, subject to technical, licensing, security, and distribution review.

## Documentation

The repository includes documentation covering the project's vision, philosophy, architecture, technical direction, product experience, development roadmap, and design resources.

The main documentation files are:

- [Vision](VISION.md) — goals and intended user experience.
- [Philosophy](PHILOSOPHY.md) — project principles.
- [Product experience](PRODUCT_EXPERIENCE.md) — planned workflows and components.
- [Technology principles](TECHNOLOGY.md) — technical direction and selection criteria.
- [Roadmap](ROADMAP.md) — development status and priorities.
- [Application and Component Catalog](docs/product/APPLICATION-CATALOG.md) — authoritative inventory of first-party apps, tools, desktop components, services, and shared design resources.
- [Initial architecture](docs/architecture/INITIAL-ARCHITECTURE.md) — system layers and open decisions.
- [Debian base](docs/architecture/DEBIAN-BASE.md) — current bootstrap implementation and limitations.
- [Debian package manifests](debian/packages/README.md) — package groups and profile status.
- [Planned applications](docs/product/PLANNED-APPLICATIONS.md) — candidate first-party applications.
- [Planned system tools](docs/product/PLANNED-SYSTEM-TOOLS.md) — printer, scanner, and resource-monitor requirements.
- [AureaOS Human Interfaces](docs/human-interfaces/README.md) — design resources and planned shared UI library.
- [Contributing](CONTRIBUTING.md) — ways to contribute.
- [License](LICENSE) — license for original AureaOS work.

## Guiding Idea

**“La tecnologia è bella quando c’è e non si vede.”**

*Technology is beautiful when it is there, but you barely notice it.*

The interface should help people do what they want without getting in their way.

Technology should support the experience rather than become an obstacle. The system should feel coherent, predictable, and approachable, allowing people to concentrate on what they want to accomplish.

## License

Original AureaOS work is released under the GNU General Public License version 3 or later (GPL-3.0-or-later), unless stated otherwise. Third-party software and assets remain subject to their own licenses and terms.


## Licensing

Original AureaOS project work is currently distributed under **GPL-3.0-or-later**, including the desktop, system components, first-party applications, documentation, and design-system source authored for the project, unless a specific file clearly identifies a different license for third-party material.

AureaOS product ownership and visual identity are separate from software licensing. Describing the desktop, Store, or applications as AureaOS-owned or first-party does not mean that their source code is closed or exempt from the current project license. Third-party components retain their own licenses and notices.

The project is not adopting a proprietary or dual-licensing model at this stage. Any future change to the licensing or distribution model is an open strategic decision and must be reviewed separately, with attention to copyright ownership, contributor permissions, and applicable license obligations.
