# AureaOS Roadmap

This roadmap records current work and intended next steps. It is a working plan, not a delivery schedule. AureaOS targets Debian GNU/Linux 13 (Trixie), `amd64` (x86-64); the first release is AureaOS Roma 1.0.

# Current status

### Implemented in the repository

* Project vision, contribution guidance, technical direction, and initial architecture documentation.
* Debian package manifests and a bootstrap script for a minimal, terminal-only Debian root filesystem.
* AureaOS visual design resources, including tokens, themes, icons, illustrations, and wallpapers.
* Initial Rust aurea-shell and aurea-compositor components.
* Shell session lifecycle coordination and shutdown through an event bus.
* Initial Smithay-based Wayland server, XDG shell handling, and basic toplevel tracking.

The shell and compositor are early prototypes, not a usable graphical desktop. Rendering, output management, input handling, and complete window management are not yet implemented.

The Debian bootstrap profile still needs package-resolution validation and does not produce a bootable installation image.

## In development / not yet complete

* Validate the complete package profile and bootstrap process against Debian 13 Trixie repositories.
* Validate the shell/compositor workspace build and tests.
* Define system identity, service defaults, storage layout, boot configuration, and security policy.
* Produce and validate a bootable test image, then document hardware support and limitations.

## Planned

* Complete the Wayland desktop foundation, including rendering, outputs, input, and window management.
* GUI-first graphical installation experience.
* Shared AureaOS Human Interfaces library and system integration.
* Core first-party applications and tools listed in the Application and Component Catalog.
* Flatpak and Flathub as the planned application distribution path.
* Separate update channels for Debian base packages and AureaOS system components.
* Complete and review the original AureaOS system sound theme.

## Development priorities

1. Validate the shell/compositor workspace build and tests.
2. Continue the compositor foundation: rendering backend, outputs, input, and surface lifecycle.
3. Complete and validate the minimal Debian 13 amd64 foundation.
4. Define image, boot, installation, security, and update boundaries.
5. Build and test a bootable development image and record supported hardware.
6. Develop the shared GUI foundation and first-party applications.
7. Set and verify release criteria for AureaOS Roma 1.0.

## Application and component status

The [Application and Component Catalog](docs/product/APPLICATION-CATALOG.md) is the authoritative inventory, including name variants, Roma 1.0 direction, candidates, and exploratory long-term concepts. The catalog records the desktop, shared design system, applications, printer/scanner management, and Resource Monitor without implying they are implemented.

Sonus, Visio, and Explora remain candidates; see [Planned First-Party Applications](docs/product/PLANNED-APPLICATIONS.md) for their concept notes. Wallet, system-level passkeys, and Aurea AI/ChatGPT integration remain exploratory long-term ideas. No milestones or delivery dates are set here.

## Guiding priorities

1. Reliability before feature count.
2. Graphical workflows for everyday tasks.
3. A consistent and accessible experience.
4. Clear boundaries between Debian, AureaOS components, and third-party applications.
5. Honest reporting of implementation status, limitations, and licensing.
