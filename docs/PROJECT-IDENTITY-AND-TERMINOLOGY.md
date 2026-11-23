<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS — Project Identity and Terminology

This document defines how AureaOS describes its product identity, source licensing, technical choices, and implementation status in public documentation.

## First-party, AureaOS-developed, and open source

AureaOS is developing its own desktop environment, shell, compositor integration, system APIs, services, and selected applications. These are **first-party** or **AureaOS-developed** components because their product direction, design, implementation, and maintenance belong to this project.

First-party does **not** mean closed-source. Original AureaOS work is licensed under GPL-3.0-or-later unless a file-level notice states otherwise. Third-party code, libraries, firmware, fonts, icons, and other assets retain their own licenses and attribution requirements.

For public communication, prefer:
- **AureaOS-developed desktop environment**
- **first-party AureaOS applications**
- **AureaOS-native system services and APIs**
- **independent desktop architecture**

Avoid calling the software *proprietary* when the intended meaning is that AureaOS owns and maintains its own implementation. In common software terminology, *proprietary software* usually implies restrictions associated with closed-source licensing and therefore creates an avoidable ambiguity.

The project may own its product names, visual identity, design decisions, and roadmap while keeping the source code open.

## Product philosophy

AureaOS is not intended to teach people how to use a computer again. It aims to preserve familiar, predictable workflows for users arriving from Windows, macOS, or another Linux desktop while providing its own coherent identity.

The priority is continuity, familiarity, integration, and dependable defaults—not novelty for its own sake. The desktop should work locally without requiring a cloud account. Online services and future account-based synchronization are optional extensions, not prerequisites for ordinary desktop use.

## Native desktop services and APIs

The long-term architecture is to expose desktop capabilities to applications through documented, versioned, permission-controlled interfaces. Applications should request narrowly scoped capabilities rather than receive unrestricted access to desktop internals, user data, or privileged services.

The existing `aurea-desktop-api` crate contains internal contracts between desktop workspace components. It must not currently be described as a stable public application SDK, a complete system-services framework, or a finished permission broker.

Local-first service concepts include shared file access, notifications, search integration, text services, and accessibility interfaces. A system-wide dictionary, spellchecking, autocorrection, personal dictionaries, and text substitutions are product directions, not implemented services. Optional future synchronization of contacts, calendars, mail, notes, and preferences requires a later account design and must not become a prerequisite for local use.

ChatGPT/Aurea AI integration is exploratory. It is not a currently implemented desktop service. Any future integration must clearly distinguish local processing from external API calls, require user consent, and disclose what information leaves the device.

## Technical commitments and trade-offs

The current target architecture is:
- Debian GNU/Linux 13 (Trixie), initially targeting `amd64`.
- Rust as the primary language for first-party system components and applications where technically appropriate.
- Wayland-only desktop session; no X11 or XWayland compatibility layer is planned.
- Smithay as the current Wayland compositor framework.
- Vulkan as the exclusive AureaOS graphics rendering API; no OpenGL rendering backend is planned.
- Flatpak and Flathub as the planned distribution route for selected third-party applications.

These choices do not guarantee compatibility with every Linux application. X11-only applications, applications requiring XWayland, and applications with OpenGL-only rendering paths require individual evaluation. No global Flathub compatibility percentage has been measured or should be claimed.

Smithay is a set of building blocks for Wayland compositors, not a complete desktop environment or a finished window manager. AureaOS remains responsible for implementing its own shell, window-management behavior, input integration, rendering, settings, accessibility, and end-user experience. See the [Smithay project](https://github.com/Smithay/smithay) and its [documentation](https://docs.rs/smithay/latest/smithay/).

## Honest implementation status

The repository is an early prototype, not a usable daily desktop or a bootable AureaOS release. The Rust workspace contains the desktop composition root, shell, shared internal API, and Smithay-based compositor foundations. DRM device/output lifecycle and an initial Vulkan SHM composition/scanout path exist in source.

At the latest recorded CI checkpoint (GitHub Actions, Ubuntu 24.04, 17 November 2026):
- `cargo fmt --all -- --check` passed.
- `cargo check --workspace --locked` and `cargo check -p aurea-compositor --features drm --locked` passed.
- Workspace tests, compositor DRM/Vulkan tests, shader compilation, and the Mesa lavapipe offscreen triangle smoke test passed; one test remained intentionally ignored.
- `cargo audit` completed with two allowed warnings for transitive `cgmath 0.18.0`; this is not a clean dependency audit.
- Physical DRM/KMS display validation and a bootable desktop image remain pending.

The seven compiler errors identified on 15 November were fixed in commits `ae6605e` and `b749427`. See [the successful GitHub Actions run](https://github.com/AleessandroCeccarelli/AureaOs/actions/runs/44056628316).

The presence of code or design documentation is not evidence that a feature is complete. Each component must be labelled as implemented, prototype, foundation, planned, candidate, or undecided according to the evidence in the repository.

## Public communication rule

Describe the vision confidently, but separate it from the implementation. State what exists in source, what has compiled and been tested, what has been validated on hardware, and what remains a design goal. Do not imply that Fides, Quaero, Aurea Wallet, system passkeys, a system-wide dictionary, a public application SDK, or ChatGPT integration are already available unless the corresponding implementation and verification exist.
