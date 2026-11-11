<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Technology Principles

## 1. Platform and Product Architecture

AureaOS Roma 1.0 targets Debian GNU/Linux 13 (Trixie) on amd64 (x86-64).

AureaOS is a GUI-first, Wayland-first Linux distribution.

Its architecture is divided into two principal layers:

* Debian foundation: provides the Linux kernel, drivers, firmware, core system packages, and established Linux infrastructure.
* AureaOS product layer: provides the desktop environment, system integration, user experience, and first-party applications developed specifically for AureaOS.

Debian is the technical foundation, not the identity of the desktop experience.

AureaOS owns the design and development of its upper system layer. This does not mean that every application available to users must be developed by the project.

For the first release, AureaOS will combine its own integrated desktop and essential system applications with selected third-party applications distributed through Aurea Store.

See Debian Base for the bootstrap implementation and its current limitations.

## 2. Technology Selection Principles

Technologies must be selected according to the needs of AureaOS and its users.

Open-source and proprietary software or firmware may be considered when technically justified and legally distributable.

Evaluate each component for:

* Compatibility and reliability on the target platform.
* Security, maintenance, and upstream support.
* Performance and resource use.
* Usability, accessibility, and system integration.
* License, attribution, redistribution, patent, and other applicable terms.
* Long-term sustainability and available alternatives.

AureaOS should reuse mature Linux infrastructure rather than rewrite established technologies without a clear reason.

Development effort should focus on the components that define the AureaOS experience and the integration that makes the system coherent.

Third-party components retain their own licenses. GPL-3.0-or-later applies to original AureaOS work as stated in the repository license; it does not automatically apply to third-party components.

## 3. Programming Language

Rust is the primary programming language for AureaOS-developed system components and first-party applications.

This includes the desktop environment, system services, and graphical applications wherever technically appropriate.

A different language may be used when a component has a clear technical reason. The exception and its maintenance implications should be documented.

This policy does not require third-party applications, drivers, firmware, libraries, or infrastructure to use Rust.

## 4. Desktop and Graphics Architecture

AureaOS follows a Wayland-first desktop architecture.

The desktop is developed as an AureaOS-owned environment, with separate responsibilities for the shell, compositor, system services, and user-facing applications.

### Current compositor foundation

The current aurea-compositor implementation uses:

* Smithay as its Wayland compositor framework.
* Wayland for desktop display communication.
* Calloop for event-loop integration.
* libseat for session and device access management.
* DRM and Udev for graphics-device discovery and output-device lifecycle foundations.

The current Rust workspace is located under `components/` and contains four crates with distinct responsibilities:

* `aurea-desktop`: desktop entry point and composition root.
* `aurea-shell`: session lifecycle and event coordination, including compositor startup and shutdown.
* `aurea-desktop-api`: shared contracts between desktop components.
* `aurea-compositor`: Wayland compositor, graphics-device, and output responsibilities.

The composition root wires these components together; it must not absorb their internal responsibilities. The shell coordinates the session and does not own compositor rendering or display-device management.

These components establish the initial technical foundation. They do not yet constitute a complete graphical desktop.

Full rendering, physical frame presentation, complete DRM/KMS mode setting, full input handling, and finished window management remain under development.

### Vulkan

Vulkan is the selected and exclusive graphics rendering API for AureaOS.

The project will not introduce alternative rendering APIs as parallel graphics backends.

This is an architectural decision, not a claim that the rendering pipeline is already complete.

The concrete Vulkan backend, frame rendering, swapchain and presentation integration, and physical display output still require implementation and validation.

### User interface technologies

No native UI toolkit has been selected.

The project will evaluate suitable technologies for individual first-party components according to:

* Integration with AureaOS.
* Performance and resource use.
* Accessibility and input support.
* Consistency with AureaOS Human Interfaces.
* Maintainability and long-term sustainability.

CSS is currently used for design tokens and web-oriented theme resources. Its presence does not imply that a native UI toolkit or shared native component library has been implemented.

See the Application and Component Catalog for per-component language and technology status.

## 5. AureaOS-Owned System Layer

The upper product layer is developed and maintained by AureaOS.

The first-release direction includes:

* Aurea Desktop: the native desktop environment, graphical shell, and window-management experience.
* Aurea Wallet: an important first-party system application and core part of the Roma 1.0 product.
* Fides: the AureaOS file manager.
* Quaero: global search across applications, files, folders, and settings.
* Aurea Store: the first-party graphical application store.
* Aurea Update: the update experience for AureaOS-specific system components.

These components must share a consistent design language, accessible interactions, clear system boundaries, and appropriate integration with system services.

They are not intended to be replaced by unrelated desktop environments or generic applications.

Their implementation status and detailed scope are documented in the Application and Component Catalog and Roadmap.

## 6. Third-Party Applications and Aurea Store

AureaOS will not attempt to develop every common desktop application from scratch for Roma 1.0.

Applications such as web browsers, email clients, and office suites will initially be provided by established third-party projects.

This allows the project to concentrate on the desktop, system integration, Wallet, file management, global search, and application distribution experience.

Aurea Store is an AureaOS-developed application that integrates with Flathub and Flatpak.

Its intended responsibilities include:

* Discovering applications available through the configured Flathub catalog.
* Presenting application descriptions and relevant information.
* Installing and removing Flatpak applications.
* Launching installed applications.
* Managing application updates.
* Explaining application permissions and sandbox access in understandable language.

Third-party Flatpak applications run within their respective sandbox environments.

The Store must make application origin, permissions, and relevant security information understandable to users.

Flatpak is the distribution and execution mechanism for third-party applications. It is not the foundation of Aurea Desktop or a replacement for AureaOS first-party system components.

AureaOS must evaluate application provenance, maintenance, permissions, security, and licensing. Availability through a graphical store does not automatically guarantee that an application is safe.

## 7. Updates and Distribution Boundaries

AureaOS separates software maintenance into three distinct domains.

### Debian base updates

Debian packages, the Linux kernel, security updates, drivers, and underlying system infrastructure are maintained through the configured Debian repositories.

### AureaOS system updates

AureaOS-specific components are maintained through a separate project-managed update mechanism.

This includes the desktop environment, system services, and first-party applications distributed as part of the AureaOS product.

The update mechanism, signing, compatibility rules, and recovery behavior still need to be designed and validated.

### Third-party application updates

Flatpak applications are managed through Aurea Store and their configured Flatpak remotes, including Flathub.

Their installation and update lifecycle remains distinct from Debian base packages and AureaOS system updates.

The three domains must have clear ownership and must not silently interfere with one another.

## 8. Other System Infrastructure

AureaOS intends to reuse established Linux infrastructure for essential system capabilities.

The current or intended technical direction includes:

* systemd for system startup and service management.
* PipeWire and WirePlumber for audio infrastructure.
* NetworkManager for network connectivity.
* DRM/KMS for graphics-device and display management.
* GRUB 2 for the planned bootloader configuration.
* Btrfs as the selected default installed filesystem.
* Udev for hardware discovery and device events.

The exact package selection, service configuration, integration, and release behavior must be validated during development.

Btrfs is the selected default filesystem, but AureaOS does not intend to use native Btrfs snapshots as its backup and restore workflow. Aurea Time is responsible for the planned backup and recovery experience.

## 9. Compatibility, Drivers, and Firmware

AureaOS aims to provide reliable hardware support on its target platform.

Open-source and proprietary drivers or firmware may be considered according to compatibility, performance, security, maintenance, licensing, and redistribution conditions.

The project should prefer the solution that provides the most reliable and sustainable result for supported hardware.

The use of proprietary components must be documented and must respect their applicable distribution terms.

## 10. Modularity and Maintainability

AureaOS components must have clearly defined responsibilities and interfaces.

The project follows a modular architecture:

* Components own their specific responsibilities.
* Shared functionality must have a clear purpose and interface.
* Technology-specific implementation details should remain within the components that own them.
* System-wide policy must not be duplicated across unrelated components.
* Files and modules should be organized around coherent responsibilities.

The guiding development principle is:

One file, one responsibility.

This does not mean creating unnecessary files or fragmenting a component into tiny modules. The objective is a structure that remains understandable, testable, and maintainable as the system grows.

## 11. Guiding Principle

AureaOS should build what defines its identity and reuse what the Linux ecosystem already does well.

Debian provides the foundation. AureaOS builds the desktop and integrated system experience. Established third-party applications extend the product where developing replacements would not be a responsible first-release priority.

The goal is not to reinvent everything.

The goal is to create a coherent operating system whose parts work together naturally.