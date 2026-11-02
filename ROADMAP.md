# AureaOS Roadmap

This roadmap records current work and intended next steps. It is a working plan, not a delivery schedule. AureaOS targets Debian GNU/Linux 13 (Trixie), `amd64` (x86-64); the first release is AureaOS Roma 1.0.

## Current status

### Implemented in the repository

- Project vision, contribution guidance, technical direction, and initial architecture documentation.
- Debian package manifest files and `tools/build/bootstrap-debian.sh` for a minimal, terminal-only Debian root filesystem.
- AureaOS visual design resources, including CSS color tokens, themes, icons, illustrations, and wallpapers.

These are repository artifacts. The bootstrap profile has not completed full package-resolution validation against Debian repositories, and it does not produce a bootable installation image. Design assets do not constitute a functioning desktop or application library.

### In development / not yet complete

- Validate the complete package profile and bootstrap process against Debian 13 Trixie repositories.
- Define the system identity, service defaults, storage layout, boot configuration, and security policy.
- Produce and validate a bootable test image, then document hardware support and limitations.

### Planned

- GUI-first, Wayland-first Aurea Desktop and graphical installation experience.
- Shared AureaOS Human Interfaces library and system integration. Design documentation and tokens exist; the reusable UI library is not implemented.
- Core first-party system applications and tools listed as Roma 1.0 direction in the [Application and Component Catalog](docs/product/APPLICATION-CATALOG.md).
- Flatpak and Flathub as the planned application distribution path.
- Complete the AureaOS system sound theme: generate original synthesized `.oga` event sounds with a local Python script on macOS or Linux, review them, and add the approved assets under `aurea-human-interfaces/resources/sounds/Aurea/stereo/`. The sound event specification and theme metadata are in place; audio generation remains pending.
- Separate update channels for Debian base packages and AureaOS-specific system components. Flatpak application updates are a separate domain.

The desktop UI, AureaOS applications, installer, and graphical system tools have not yet been implemented.

## Development priorities

1. Complete and validate the minimal Debian 13 `amd64` foundation.
2. Define the system image, boot, installation, security, and update boundaries.
3. Build and test a bootable development image and record its supported hardware.
4. Develop the shared GUI foundation and Wayland desktop.
5. Add first-party applications and system tools in coherent, testable increments.
6. Set and verify release criteria for AureaOS Roma 1.0.

## Application and component status

The [Application and Component Catalog](docs/product/APPLICATION-CATALOG.md) is the authoritative inventory, including name variants, Roma 1.0 direction, candidates, and exploratory long-term concepts. The catalog records the desktop, shared design system, applications, printer/scanner management, and Resource Monitor without implying they are implemented.

Sonus, Visio, and Explora remain candidates; see [Planned First-Party Applications](docs/product/PLANNED-APPLICATIONS.md) for their concept notes. Wallet, system-level passkeys, and Aurea AI/ChatGPT integration remain exploratory long-term ideas. No milestones or delivery dates are set here.

## Guiding priorities

1. Reliability before feature count.
2. Graphical workflows for everyday tasks.
3. A consistent and accessible experience.
4. Clear boundaries between Debian, AureaOS components, and third-party applications.
5. Honest reporting of implementation status, limitations, and licensing.
