AureaOS Roadmap

This roadmap records current work and intended next steps. It is a working plan, not a delivery schedule. AureaOS targets Debian GNU/Linux 13 (Trixie), amd64 (x86-64); the first release is AureaOS Roma 1.0.

Current status

Implemented in the repository

* Project vision, contribution guidance, technical direction, and initial architecture documentation.
* Debian package manifests and a bootstrap script for a minimal, terminal-only Debian root filesystem.
* AureaOS visual design resources, including tokens, themes, icons, illustrations, and wallpapers.
* Initial Rust aurea-shell and aurea-compositor components.
* Shell session lifecycle coordination and shutdown through an event bus.
* Initial Smithay-based Wayland server, XDG shell handling, and basic toplevel tracking.

The shell and compositor are early prototypes, not a usable graphical desktop. A complete rendering pipeline, physical display mode setting, input handling, and complete window management are not yet implemented.

The Debian bootstrap profile still needs package-resolution validation and does not produce a bootable installation image.

In development / not yet complete

* Validate the complete package profile and bootstrap process against Debian 13 Trixie repositories.
* Validate the shell/compositor workspace build and tests.
* Define system identity, service defaults, storage layout, boot configuration, and security policy.
* Produce and validate a bootable test image, then document hardware support and limitations.

Planned

* Complete the Wayland desktop foundation, including rendering, DRM/KMS output management, input, and window management.
* Develop Aurea Display Intelligence for display discovery, capability reporting, refresh-rate policies, VRR support, and safe mode changes.
* Integrate display policy with compositor presentation information and the DRM/KMS backend.
* Provide Automatic — AureaOS and manual display configuration through Settings → Display & Video.
* GUI-first graphical installation experience.
* Shared AureaOS Human Interfaces library and system integration.
* Core first-party applications and tools listed in the Application and Component Catalog.
* Flatpak and Flathub as the planned application distribution path.
* Separate update channels for Debian base packages and AureaOS system components.
* Complete and review the original AureaOS system sound theme.

Development priorities

1. Validate the shell/compositor workspace build and tests.
2. Continue the compositor foundation: rendering backend, DRM/KMS outputs, input, and surface lifecycle.
3. Establish reliable physical output discovery and mode-setting before implementing automatic display policies.
4. Develop Aurea Display Intelligence, including capability discovery, VRR and fixed-refresh strategies, and per-display policy.
5. Complete and validate the minimal Debian 13 amd64 foundation.
6. Define image, boot, installation, security, and update boundaries.
7. Build and test a bootable development image and record supported hardware.
8. Develop the shared GUI foundation and first-party applications.
9. Set and verify release criteria for AureaOS Roma 1.0.

Application and component status

The Application and Component Catalog is the authoritative inventory, including name variants, Roma 1.0 direction, candidates, and exploratory long-term concepts. The catalog records the desktop, shared design system, applications, system tools, and system subsystems without implying they are implemented.

Aurea Display Intelligence is a planned system subsystem. Its architecture and responsibilities are documented in Display Intelligence. Its implementation depends on reliable DRM/KMS output management and compositor integration.

Sonus, Visio, and Explora remain candidates; see Planned First-Party Applications for their concept notes. Wallet, system-level passkeys, and Aurea AI/ChatGPT integration remain exploratory long-term ideas. No milestones or delivery dates are set here.

Guiding priorities

1. Reliability before feature count.
2. Graphical workflows for everyday tasks.
3. A consistent and accessible experience.
4. Clear boundaries between Debian, AureaOS components, and third-party applications.
5. Hardware-aware behavior with safe fallbacks and respect for user preferences.
6. Honest reporting of implementation status, limitations, and licensing.
