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

- A GUI-first, Wayland-first Aurea Desktop and graphical installation experience.
- Shared AureaOS Human Interfaces library and system integration.
- First-party components including Quaero, Fides, Aurea Store, Aurea Update, Aurea Backup, and Aurea Recovery.
- Flatpak and Flathub as the planned application distribution path.
- Separate update channels for Debian base packages and AureaOS-specific system components. Application updates through Flatpak are a separate domain.

The desktop UI, AureaOS applications, installer, and graphical system tools have not yet been implemented.

## Development priorities

1. Complete and validate the minimal Debian 13 `amd64` foundation.
2. Define the system image, boot, installation, security, and update boundaries.
3. Build and test a bootable development image and record its supported hardware.
4. Develop the shared GUI foundation and Wayland desktop.
5. Add first-party applications and system tools in coherent, testable increments.
6. Set and verify release criteria for AureaOS Roma 1.0.

## Planned components

All items below are planned unless explicitly listed in the implemented section above.

| Component | Intended role |
|---|---|
| Aurea Desktop | Desktop shell and window management |
| AureaOS Human Interfaces | Shared design system and UI components |
| Quaero | Search for applications, files, folders, and settings |
| Fides | Graphical file manager |
| Aurea Store | Application discovery and management using Flatpak and Flathub |
| Aurea Update | Updates for AureaOS-specific system components, separate from Debian base updates |
| Aurea Backup | User data backup |
| Aurea Recovery | System recovery |
| Printer and scanner tools | Graphical device setup and management using suitable Linux infrastructure |
| Resource Monitor | Graphical view of system activity |

The candidate applications Sonus, Visio, and Explora are described in [Planned First-Party Applications](docs/product/PLANNED-APPLICATIONS.md). Additional ideas in product documents remain proposals until explicitly selected and implemented.

## Guiding priorities

1. Reliability before feature count.
2. Graphical workflows for everyday tasks.
3. A consistent and accessible experience.
4. Clear boundaries between Debian, AureaOS components, and third-party applications.
5. Honest reporting of implementation status, limitations, and licensing.
