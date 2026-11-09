<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Initial Architecture

## 1. Purpose

This document describes the architecture of AureaOS Roma 1.0.

It establishes clear boundaries between the Debian foundation, AureaOS system integration, desktop infrastructure, graphics architecture, first-party applications, and third-party software.

It also records the current implementation status. Architectural goals, prototypes, and completed capabilities must be clearly distinguished.

## 2. Project Goals

AureaOS aims to deliver a coherent, reliable, and approachable Linux operating system built on Debian GNU/Linux 13 (Trixie).

The initial release targets `amd64` (x86-64) hardware and prioritizes:

- A reproducible and testable system foundation.
- A modern Wayland-based desktop architecture.
- A proprietary AureaOS desktop experience.
- First-party system applications with clear responsibilities.
- Sensible defaults and consistent system behavior.
- Local user accounts without a mandatory cloud account.
- Straightforward installation, maintenance, updates, backup, and recovery.
- Clear documentation of supported hardware, limitations, dependencies, and licenses.
- Modular components with well-defined responsibilities.

AureaOS uses Debian as its lower-level system foundation. The desktop environment, system integration, visual identity, and first-party applications belong to the AureaOS product layer.

## 3. Architectural Principles

The architecture follows these principles:

1. **Debian as the foundation:** reuse proven Linux infrastructure instead of unnecessarily reimplementing existing system capabilities.
2. **Separation of responsibilities:** distinguish the base operating system, AureaOS system integration, desktop components, applications, build tools, and tests.
3. **Reproducibility:** system artifacts must be produced through documented and repeatable procedures.
4. **Reliability first:** changes must be testable, documented, and introduced incrementally.
5. **Pragmatic technology selection:** open-source and proprietary components may both be selected when justified by compatibility, reliability, security, performance, usability, maintenance, licensing, and redistribution terms.
6. **Wayland-first desktop:** the intended desktop architecture for Roma 1.0 is designed for Wayland.
7. **Coherent user experience:** system components must support a consistent and accessible operating environment.
8. **No unnecessary reinvention:** AureaOS should focus on integration and features that distinguish the project.
9. **Rust as the primary language:** first-party AureaOS applications and components should use Rust where appropriate. A different language requires a documented technical reason.
10. **Modularity:** components should have explicit ownership, limited responsibilities, and well-defined communication boundaries.
11. **Clear product boundaries:** Debian packages, AureaOS system components, and third-party applications have separate ownership and update responsibilities.

## 4. System Architecture

The system is divided into distinct layers.

### 4.1 Debian Base

The target base is Debian GNU/Linux 13 (Trixie), `amd64` (x86-64), with Btrfs selected as the default installed filesystem.

The repository contains an early `debootstrap` script and package manifests for a minimal, terminal-only root filesystem.

This bootstrap is not a complete or bootable AureaOS image and does not yet configure the final installed filesystem.

Aurea Time is planned to own backup and restore. AureaOS will not use Btrfs native snapshots as its backup and restore workflow.

Debian remains responsible for its own base packages and security updates.

See [Debian Base](DEBIAN-BASE.md) for implementation status and limitations.

### 4.2 AureaOS System Configuration

This layer contains AureaOS-specific configuration and integration built on top of Debian.

Its responsibilities include:

- Base package selection.
- System defaults and configuration.
- Service configuration.
- System identity and release metadata.
- Integration of selected third-party system components.
- System-level update and maintenance policies.
- Configuration required by the AureaOS desktop and first-party applications.

These responsibilities must remain separate from desktop presentation and application-specific user interfaces.

Only requirements that have been evaluated and documented should be implemented.

### 4.3 Desktop Composition: Aurea Desktop

`aurea-desktop` is the desktop composition root.

It is responsible for bringing together the desktop session, shell, compositor, and the interfaces required by the graphical environment.

The composition root must coordinate the components without taking ownership of responsibilities that belong to them.

The intended component relationship is:

- `aurea-desktop` composes and starts the desktop environment.
- `aurea-shell` coordinates session lifecycle and events.
- `aurea-desktop-api` provides technology-neutral contracts shared between desktop components.
- `aurea-compositor` implements the Wayland compositor and owns graphics and display responsibilities.

The desktop composition architecture is an early implementation foundation. It does not yet provide a complete graphical desktop or finished user experience.

### 4.4 Desktop Session: Aurea Shell

`aurea-shell` is the component responsible for coordinating the desktop session lifecycle.

Its responsibilities include:

- Session lifecycle coordination.
- Event handling and communication between session components.
- Lifecycle transition handling.
- Compositor startup and coordination.
- Shutdown handling.

The shell and compositor are separate components. The shell owns session coordination; it does not own the compositor's rendering or display-device responsibilities.

The current implementation is an early technical foundation. It is not yet a complete graphical desktop shell and does not provide the final user interface.

### 4.5 Desktop API: Aurea Desktop API

`aurea-desktop-api` provides shared contracts for communication between desktop components.

Its purpose is to keep component boundaries explicit and reduce unnecessary coupling between implementations.

The API layer should remain independent of implementation details belonging to the shell or compositor.

Its contracts must describe responsibilities and communication interfaces rather than duplicate the internal behavior of individual components.

### 4.6 Wayland Compositor: Aurea Compositor

`aurea-compositor` is the Wayland compositor component.

Its current implementation uses:

- Rust as its programming language.
- Smithay 0.7 for Wayland compositor infrastructure.
- Calloop for event-loop integration.
- libseat for session and device access integration.
- Udev for hardware discovery and events.
- Linux DRM interfaces for graphics-device and output discovery.

The current prototype includes:

- A Wayland server.
- Initial XDG shell integration.
- Client and toplevel surface tracking.
- An initial window-management model.
- An initial four-workspace model: mapped toplevels belong to one workspace, and render scenes include only windows on the active workspace.
- An initial Wayland seat and XKB keyboard, with libinput event routing when the DRM feature is enabled.
- Super + Ctrl + Left / Right bindings for workspace navigation.
- Conversion of mapped XDG surface trees into Smithay render elements, including logical window positions and output scaling.
- DRM device discovery and management.
- DRM device event integration with the compositor event loop.
- Session pause and activation handling.
- GPU inventory and rendering preference models.
- DRM connector and display-mode discovery.
- Stable output identity generation.
- Output registration, removal, and configuration.
- Logical output layout and output reconciliation.
- Publication of registered outputs as Wayland output globals.

The compositor is not yet a complete graphical desktop.

The following capabilities remain incomplete:

- A complete Vulkan rendering and presentation pipeline.
- Frame rendering and presentation to physical displays.
- Complete DRM/KMS mode setting.
- Full pointer, touch, and gesture handling.
- Complete window management and desktop interactions.
- Cross-GPU rendering and presentation.

GPU discovery and rendering preference selection do not establish working multi-GPU rendering.

The current Vulkan backend can report `PresentationUnavailable` because the swapchain and physical presentation path are not yet implemented.

### 4.7 Graphics and Display Architecture

Vulkan is the selected and exclusive graphics rendering API for AureaOS.

The graphics architecture is built around the Linux graphics stack, with the compositor owning the initial DRM device and output lifecycle integration.

The responsibilities are separated as follows:

- **DRM device management:** owns the lifecycle of opened DRM devices.
- **DRM device registry:** maintains discovered device metadata.
- **GPU management:** maintains GPU inventory and rendering preferences.
- **Output management:** owns registered outputs, their configuration, and logical layout.
- **Wayland integration:** publishes and removes the corresponding Wayland output globals.
- **Vulkan rendering:** provides the selected graphics rendering API and the planned rendering and presentation pipeline.
- **Display Intelligence:** planned system-level capability and display-policy subsystem.

The current implementation supports initial device discovery, output lifecycle management, and Wayland output publication.

Vulkan is selected, but its complete rendering and physical presentation pipeline is not yet implemented.

The architecture does not yet provide complete physical display mode setting, frame presentation, VRR control, or adaptive refresh policy.

The planned display architecture is documented in [Display Intelligence](DISPLAY-INTELLIGENCE.md).

### 4.8 Build and Validation Tools

Build tools provide the documented process for preparing and validating AureaOS system artifacts.

The current bootstrap implementation uses `debootstrap`.

The package profile, complete image-generation process, and reproducible release pipeline still require validation and further development.

The final build process must distinguish Debian base preparation from AureaOS-specific system integration and desktop components.

### 4.9 Applications and User Experience

The application and user-experience layer contains AureaOS first-party applications and the integration of third-party desktop software.

The [Application and Component Catalog](../product/APPLICATION-CATALOG.md) is the authoritative inventory of first-party applications, system tools, desktop components, and shared design resources.

The first-party Roma 1.0 product direction includes:

- **Aurea Desktop:** the proprietary graphical desktop environment and window-management experience.
- **AureaOS Human Interfaces:** the shared design system and planned native UI library.
- **Aurea Wallet:** a first-party system application and an important pillar of Roma 1.0.
- **Fides:** the AureaOS file manager.
- **Quaero:** global search across applications, files, folders, and settings.
- **Aurea Store:** the official application store.
- **Aurea Update:** updates for AureaOS system components.
- **Aurea Time:** system backup and restore.

These applications and components define the intended first-party product experience. Their inclusion in the architecture does not mean they are already implemented or complete.

For Roma 1.0, established third-party applications will cover common needs such as web browsing, email, office and productivity work, and other everyday desktop tasks.

Aurea Store will integrate with Flathub as an application source and Flatpak for installation, execution, sandboxing, and updates.

Third-party applications remain distinct from AureaOS first-party products.

AureaOS will maintain separate update responsibilities for:

1. Debian base packages.
2. AureaOS system components and first-party system software.
3. Flatpak applications.

The graphical desktop UI and first-party end-user applications remain under development or planned.

## 5. Component Boundaries

The architecture must preserve clear ownership between components.

| Component | Primary responsibility |
|---|---|
| Debian base | Kernel, core system packages, and Linux infrastructure |
| AureaOS system configuration | Distribution defaults, services, and system integration |
| `aurea-desktop` | Desktop composition and component startup |
| `aurea-shell` | Desktop session lifecycle and coordination |
| `aurea-desktop-api` | Technology-neutral desktop contracts |
| `aurea-compositor` | Wayland server, surfaces, outputs, and compositor responsibilities |
| DRM device management | DRM device discovery and lifecycle |
| GPU management | GPU inventory and rendering preferences |
| Output management | Output registration, configuration, and logical layout |
| Vulkan rendering | Selected graphics rendering and presentation architecture |
| Display Intelligence | Planned display capability analysis and display policy |
| AureaOS first-party applications | Product-specific user-facing workflows |
| Aurea Store | Application discovery and installation integration |
| Flatpak ecosystem | Third-party application execution, sandboxing, and updates |

The shell and compositor must remain separate components.

The desktop composition root coordinates the desktop components but must not absorb their internal responsibilities.

The desktop API must remain focused on shared contracts rather than implementation-specific behavior.

Display policy must not become an uncontrolled responsibility of the compositor. The compositor provides output lifecycle and presentation information, while the planned Display Intelligence subsystem coordinates higher-level display decisions.

No two components should independently own the same device lifecycle or compete to apply the same display configuration.

## 6. Repository Organization

The repository contains documentation, Debian bootstrap resources, design resources, and Rust desktop components.

The principal project areas include:

```text
AureaOs/
├── assets/
├── components/
│   ├── aurea-desktop-api/
│   ├── aurea-shell/
│   ├── aurea-compositor/
│   └── aurea-desktop/
├── debian/
│   └── packages/
├── docs/
│   ├── architecture/
│   ├── human-interfaces/
│   └── product/
├── aurea-human-interfaces/
├── tools/
│   └── build/
├── README.md
├── ROADMAP.md
├── TECHNOLOGY.md
├── VISION.md
├── PHILOSOPHY.md
├── PRODUCT_EXPERIENCE.md
├── CONTRIBUTING.md
└── LICENSE
