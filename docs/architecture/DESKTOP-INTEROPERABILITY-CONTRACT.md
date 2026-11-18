<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Desktop Interoperability Contract

## Purpose

AureaOS owns its desktop experience and implementation, but it must not invent a private replacement for every desktop convention. Applications should behave predictably when installed, launched, opened from files, granted permissions, or asked to display notifications.

This document defines the interoperability direction for AureaOS. It is an architecture contract and roadmap, not a claim that every item is already implemented.

## Compatibility principle

- Keep AureaOS visual identity, shell, settings, and first-party applications independently designed.
- Prefer established Linux desktop specifications for application metadata, MIME handling, notifications, sandbox integration, and user directories.
- Use documented, versioned APIs for AureaOS-specific capabilities.
- Do not require applications to depend on private shell internals.
- Do not claim compatibility with X11 or XWayland: AureaOS targets a native Wayland session.
- Evaluate application compatibility by actual runtime and graphics requirements, not by the presence of an application in Flathub alone.

## Linux desktop interoperability

The project should implement and test relevant freedesktop.org/XDG specifications where they fit the product:

| Area | Interoperability target | AureaOS responsibility |
|---|---|---|
| Application launch | Desktop Entry specification | Discover, validate, launch and manage installed applications |
| File associations | Shared MIME-info database and desktop entry MIME declarations | Predictable open-with selection and default application management |
| User folders | XDG Base Directory and user-dirs conventions | Respect user data/config/cache locations and localized standard folders |
| Notifications | Desktop Notifications specification | Present consistent notifications with actions and replacement semantics |
| Sandboxed applications | XDG Desktop Portal interfaces | Permission-mediated access to files, screenshots, notifications and other supported capabilities |
| Application metadata | AppStream metadata | Populate the store with accurate names, icons, summaries, categories, licenses and release information |
| Autostart | XDG Autostart specification | Apply explicit user and system startup policy |
| Icons | Freedesktop icon naming/theme conventions where applicable | Provide AureaOS artwork while retaining predictable icon lookup and fallbacks |

AureaOS must consult the current specification and the actual toolkit/runtime behavior before implementing each integration. Freedesktop specifications are interoperability specifications, not a guarantee that every desktop implements every interface identically.

## Flatpak and portals

Flatpak applications are expected to use the portal model for supported desktop integration. AureaOS must provide a portal backend appropriate to its shell and compositor, or clearly report unsupported portal interfaces.

The portal boundary is a security boundary:

- Grant the narrowest capability needed for the requested operation.
- Make user consent explicit for sensitive access.
- Do not silently turn portal requests into unrestricted host filesystem access.
- Preserve document-portal file descriptors and grants according to the portal contract.
- Treat application-provided metadata and request parameters as untrusted input.
- Test denial, cancellation, revocation, and application exit paths, not only successful access.

AureaOS must not advertise a portal interface as supported until its behavior and security properties are implemented and tested.

## Familiarity across operating systems

Continuity with Windows and macOS is a product-level behavior goal, not a promise to reproduce their private APIs or internal design.

AureaOS should provide familiar workflows for:

- Keyboard navigation, focus, shortcuts, copy/paste, selection and drag-and-drop.
- Window move, resize, minimize, maximize, close, switching and workspace navigation.
- File opening, saving, renaming, moving, trashing and choosing default applications.
- Device connection, display arrangement, audio selection and removable media.
- Application installation, updates, permissions and uninstall.
- Accessibility, text scaling, keyboard alternatives and clear system feedback.

Where Linux already has an established convention, AureaOS should prefer that convention over a proprietary behavior that makes third-party applications harder to integrate.

## Wayland and graphics boundary

Native Wayland is the supported desktop protocol. Applications that require X11/XWayland are outside the initial compatibility target and must not be represented as supported.

Vulkan remains the exclusive AureaOS rendering API for first-party compositor rendering. This does not mean every Linux application available through Flatpak uses Vulkan. Third-party applications may have their own graphics requirements; compatibility must be assessed and documented per application/runtime. AureaOS must not silently add an OpenGL compositor backend to make a marketing compatibility claim.

## Implementation order

1. Application discovery and Desktop Entry validation.
2. MIME database integration and default application selection.
3. XDG user directories and file-manager integration.
4. Notifications and action handling.
5. Flatpak integration and the portal interfaces required by the first-party shell.
6. AppStream metadata ingestion for the software store.
7. Autostart policy and user-visible controls.
8. Accessibility and keyboard workflow audits across the shell.

Each integration must have a small owner module, a documented contract, focused tests, and an explicit implementation-status label. Avoid one large "desktop integration" module that owns unrelated specifications.

## Acceptance criteria

An integration is not complete merely because it compiles. It must demonstrate:

- Valid and invalid input handling.
- Permission denial and failure behavior.
- Predictable interaction with third-party clients.
- Correct lifecycle handling, including disconnect, restart and removal.
- Tests for its public contract.
- Clear user-facing behavior and documentation.
- No hidden dependency on X11, XWayland, OpenGL, or another desktop environment.

## References

- [freedesktop.org specification repository](https://specifications.freedesktop.org/)
- [freedesktop.org interoperability specifications overview](https://wiki.freedesktop.org/www/Specifications/)
- [Flatpak documentation](https://docs.flatpak.org/)
- [XDG Desktop Portal documentation](https://flatpak.github.io/xdg-desktop-portal/)
