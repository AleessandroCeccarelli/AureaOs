
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Initial Architecture

## 1. Purpose

This document describes the initial architecture of AureaOS Roma 1.0.

It establishes clear boundaries between the Debian foundation, AureaOS system integration, desktop infrastructure, and the future graphical user experience.

It also records the current implementation status. Architectural goals, prototypes, and completed capabilities must be clearly distinguished.

## 2. Project Goals

AureaOS aims to deliver a coherent, reliable, and approachable Linux operating system built on Debian.

The initial release targets `amd64` (x86-64) hardware and prioritizes:

- A reproducible and testable system foundation.
- A modern Wayland-based desktop architecture.
- Sensible defaults and consistent system behavior.
- Local user accounts without a mandatory cloud account.
- Straightforward installation, maintenance, updates, backup, and recovery.
- Clear documentation of supported hardware, limitations, dependencies, and licenses.
- Modular components with well-defined responsibilities.

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

## 4. System Architecture

The system is divided into distinct layers.

### 4.1 Debian Base

The target base is Debian GNU/Linux 13 (Trixie), `amd64` (x86-64), with Btrfs selected as the default installed filesystem.

The repository contains an early `debootstrap` script and package manifests for a minimal, terminal-only root filesystem.

This bootstrap is not a complete or bootable AureaOS image and does not yet configure the final installed filesystem.

Aurea Time is planned to own backup and restore. AureaOS will not use Btrfs native snapshots as its backup and restore workflow.

See [Debian Base](DEBIAN-BASE.md) for implementation status and limitations.

### 4.2 AureaOS System Configuration

This layer contains AureaOS-specific configuration and integration built on top of Debian.

Its responsibilities include:

- Base package selection.
- System defaults and configuration.
- Service configuration.
- System identity and release metadata.
- Integration of selected third-party components.
- System-level update and maintenance policies.

These responsibilities must remain separate from the desktop user interface.

Only requirements that have been evaluated and documented should be implemented.

### 4.3 Desktop Session: Aurea Shell

`aurea-shell` is the component responsible for coordinating the desktop session lifecycle.

Its current prototype provides:

- Session lifecycle coordination.
- An event bus for communication between session components.
- Lifecycle transition handling.
- Compositor startup and coordination.
- Shutdown handling, including interruption through Ctrl-C.

The shell and compositor are separate components. The shell owns session coordination; it does not own the compositor's rendering or display-device responsibilities.

The current implementation is an early technical foundation. It is not yet a complete graphical desktop shell and does not provide the final user interface.

### 4.4 Wayland Compositor: Aurea Compositor

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

- A concrete graphics rendering backend.
- Frame rendering and presentation to physical displays.
- Complete DRM/KMS mode setting.
- Full input handling.
- Complete window management and desktop interactions.
- Cross-GPU rendering and presentation.

GPU discovery and rendering preference selection do not establish working multi-GPU rendering.

Likewise, the presence of Vulkan-related feature declarations does not mean Vulkan rendering or physical display presentation is implemented.

### 4.5 Graphics and Display Architecture

The graphics architecture is built around the Linux graphics stack.

The compositor currently owns the initial DRM device and output lifecycle integration.

The responsibilities are separated as follows:

- **DRM device management:** owns the lifecycle of opened DRM devices.
- **DRM device registry:** maintains discovered device metadata.
- **GPU management:** maintains GPU inventory and rendering preferences.
- **Output management:** owns registered outputs, their configuration, and logical layout.
- **Wayland integration:** publishes and removes the corresponding Wayland output globals.
- **Display Intelligence:** planned system-level capability and display-policy subsystem.

The current implementation supports initial device discovery, output lifecycle management, and Wayland output publication.

It does not yet provide complete physical display mode setting, frame presentation, VRR control, or adaptive refresh policy.

The planned display architecture is documented in [Display Intelligence](DISPLAY-INTELLIGENCE.md).

### 4.6 Build and Validation Tools

Build tools provide the documented process for preparing and validating AureaOS system artifacts.

The current bootstrap implementation uses `debootstrap`.

The package profile, complete image-generation process, and reproducible release pipeline still require validation and further development.

### 4.7 Applications and User Experience

The desktop experience is planned as a separate layer integrated with the underlying system.

The [Application and Component Catalog](../product/APPLICATION-CATALOG.md) is the authoritative inventory of first-party applications, system tools, desktop components, and shared design resources.

The graphical desktop UI and first-party end-user applications are not yet implemented.

Flatpak and Flathub are the planned application distribution path.

## 5. Component Boundaries

The architecture must preserve clear ownership between components.

| Component | Primary responsibility |
|---|---|
| Debian base | Kernel, core system packages, and Linux infrastructure |
| AureaOS system configuration | Distribution defaults, services, and system integration |
| `aurea-shell` | Desktop session lifecycle and coordination |
| `aurea-compositor` | Wayland server, surfaces, outputs, and compositor responsibilities |
| DRM device management | DRM device discovery and lifecycle |
| GPU management | GPU inventory and rendering preferences |
| Output management | Output registration, configuration, and logical layout |
| Display Intelligence | Planned display capability analysis and display policy |
| AureaOS applications | User-facing graphical workflows |

The shell and compositor must remain separate components.

Display policy must not become an uncontrolled responsibility of the compositor. The compositor provides output lifecycle and presentation information, while the planned Display Intelligence subsystem coordinates higher-level display decisions.

No two components should independently own the same device lifecycle or compete to apply the same display configuration.

## 6. Repository Organization

The repository contains documentation, Debian bootstrap resources, design resources, and Rust desktop components.

The current structure includes the following principal areas:

```text
AureaOs/
├── assets/
├── components/
│   ├── aurea-shell/
│   └── aurea-compositor/
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
```

This overview describes the principal project areas, not every file or nested directory.

Each component should keep its implementation, tests, and documentation organized around its own responsibilities.

Shared functionality should be introduced only when there is a clear need and a well-defined interface.

## 7. Dependencies and Licensing

Third-party components remain subject to their respective licenses.

AureaOS is not restricted to open-source-only components. Proprietary software and firmware may be considered when they materially improve compatibility or the user experience and their distribution terms permit the intended use.

The project license applies to original AureaOS work and does not automatically relicense third-party components.

Before integrating a dependency, AureaOS should record, as applicable:

- Component name and version.
- Source and upstream project.
- License.
- Intended purpose.
- Maintenance and security considerations.
- Compatibility with the target architecture and desktop direction.

Dependency and license-review procedures must be maintained as the project grows.

## 8. Security and System Maintenance

Security and maintainability are architectural requirements, not optional additions.

The system design must account for:

- Supported security updates.
- Appropriate system permissions.
- Documented service responsibilities.
- Safe system configuration and upgrade procedures.
- Recovery and troubleshooting.
- Clear separation between system-managed files and user data.
- Controlled access to hardware devices and privileged operations.

Detailed security policies, update mechanisms, and recovery procedures remain to be designed and validated.

## 9. Testing and Validation

The technical foundation must be validated incrementally.

Current Rust components include automated tests for selected shell, compositor, output-management, DRM lifecycle, and session-handling behavior.

Further validation must include:

- Workspace formatting and compilation checks.
- Automated tests for component behavior and integration boundaries.
- Hardware-independent testing wherever possible.
- Physical DRM device and display testing.
- Hotplug and session lifecycle validation.
- Boot and installation testing once a bootable image exists.
- Hardware compatibility testing across representative systems.

A successful build alone does not establish that the resulting operating system is reliable.

Testing must verify the resulting artifacts and their behavior.

## 10. Decisions Still to Be Defined

The following decisions require evaluation and documentation before the corresponding implementation is considered complete:

- Debian repository policy beyond the current Trixie bootstrap sources.
- The complete system build technology and build environment.
- The final initial package selection.
- Service configuration and security policy.
- The concrete graphics rendering backend.
- Physical DRM/KMS mode-setting and presentation strategy.
- Complete input handling and window-management behavior.
- Display Intelligence interfaces and policy ownership.
- Graphics, networking, audio, and hardware integration strategy.
- Separate update mechanisms for Debian base packages and AureaOS system components.
- Dependency inventory and license-review process.
- Bootable image generation and installation strategy.

Existing implementation choices should be documented accurately and should not be presented as unresolved decisions.

## 11. Implementation Priorities

Development will proceed incrementally:

1. Continue validating the shell and compositor components.
2. Complete the concrete graphics rendering and physical presentation foundation.
3. Complete physical DRM/KMS mode setting and display configuration.
4. Implement input handling and advance window and surface lifecycle management.
5. Validate output discovery, hotplug, and session handling on physical hardware.
6. Develop Display Intelligence on top of a reliable display pipeline.
7. Complete and validate the Debian 13 system foundation.
8. Define image generation, boot, installation, security, and update boundaries.
9. Produce and test a bootable development image.
10. Develop the shared graphical foundation and first-party applications.

Desktop application development will expand as the technical foundation becomes sufficiently reliable.

## 12. Current Status

AureaOS has progressed beyond the initial Debian bootstrap and documentation stage.

The repository now includes early Rust implementations of the desktop session coordinator and Wayland compositor, together with initial DRM device discovery, output lifecycle management, and Wayland output publication.

These are meaningful technical foundations, but they do not yet constitute a usable graphical desktop.

The Debian package profile, bootable image, installer, complete rendering pipeline, physical display presentation, full input handling, desktop UI, and first-party end-user applications remain incomplete.

Architecture decisions and implementation status must be updated as the project progresses.

---

**AureaOS Roma 1.0 — Reliability first. A coherent experience. Built on proven foundations.**
