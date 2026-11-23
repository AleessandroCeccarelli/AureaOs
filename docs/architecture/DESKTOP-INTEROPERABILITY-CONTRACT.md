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
| Sandboxed applications | XDG Desktop Portal interfaces | Permission-mediated access to files, screenshots, screen-cast streams, notifications and other supported capabilities |
| Application metadata | AppStream metadata | Populate the store with accurate names, icons, summaries, categories, licenses and release information |
| Autostart | XDG Autostart specification | Apply explicit user and system startup policy |
| Icons | Freedesktop icon naming/theme conventions where applicable | Provide AureaOS artwork while retaining predictable icon lookup and fallbacks |

AureaOS must consult the current specification and the actual toolkit/runtime behavior before implementing each integration. Freedesktop specifications are interoperability specifications, not a guarantee that every desktop implements every interface identically.


## AureaOS system API boundary

AureaOS-specific capabilities must be exposed through documented contracts rather than private compositor or shell internals. The API has distinct layers:

- **Desktop component contracts:** `components/aurea-desktop-api` contains technology-independent Rust contracts shared by AureaOS components. Its `input` module owns keyboard layouts, pointer preferences, device capabilities and optional peripheral-extension metadata. Its `appearance` module exposes the system theme, resolved color scheme, accessibility appearance preferences and semantic color-token identifiers.
- **Human Interfaces:** `aurea-human-interfaces/design-system/tokens/index.css` remains the canonical source of visual values. The future native UI library consumes those tokens and provides reusable controls, typography, spacing, icons and interaction patterns. Applications must not copy palette values or invent a parallel AureaOS theme.
- **Native AureaOS applications:** first-party applications should use the shared Rust contracts and Human Interfaces library rather than depending on compositor implementation types.
- **Sandboxed and third-party applications:** use standard Wayland protocols and XDG portals/settings interfaces for the capabilities they support. AureaOS-specific cross-process APIs require a documented, versioned and permission-controlled service boundary; the transport and complete application SDK remain design work.

Input privacy is part of this boundary. Applications receive normal keyboard and pointer events for their own focused Wayland surfaces. They do not receive unrestricted global keystrokes, typed text or raw pointer streams. Global shortcuts, device inventory, layout changes and privileged hardware configuration belong to trusted system services and require the appropriate authorization.

A manufacturer extension is optional and additive: standard keyboards and mice must remain fully useful without it. Extensions may expose only capabilities verified for the connected device, and they must run outside the compositor with explicit permissions. A vendor or product identifier is a matching hint, not proof that a proprietary feature is supported.

The current Rust contracts are foundations, not a complete application-facing IPC implementation. A capability is not supported merely because its type exists; its provider, authorization, persistence where required, failure behavior and integration tests must also exist.

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

### Screenshot and screen-cast portal requirements

The portal backend is the supported integration boundary for sandboxed applications requesting screenshots or screen recording.

- Implement the XDG Screenshot and ScreenCast interfaces according to their published versions and request/response contracts.
- Present a system-controlled source picker; applications must not silently choose arbitrary screens or windows.
- Scope each grant to the requested operation, source and session. Treat screenshot, screen video, microphone and system-audio permissions as distinct capabilities.
- Return only the streams, file handles or results authorized by the portal contract. Do not expose unrestricted compositor buffers or host filesystem paths.
- Support cancellation, denial, session closure, application disconnection and permission revocation.
- Report unsupported source types or options honestly. Do not advertise area selection, cursor modes, persistence or audio capture unless each is implemented and tested.
- Use the document portal or an equivalent user-mediated destination flow for sandboxed output files.

The portal's screen-cast video stream does not by itself grant system-audio or microphone access. Audio capture must follow a separate PipeWire permission and routing path.

### Familiarity across operating systems

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

1. Complete technology-independent desktop API contracts and their focused tests.
2. Implement trusted system providers for input-device inventory, keyboard layouts, pointer settings and appearance preferences.
3. Define the versioned application-facing service and permission boundary.
4. Application discovery and Desktop Entry validation.
5. MIME database integration and default application selection.
6. XDG user directories and file-manager integration.
7. Notifications and action handling.
8. Flatpak integration and the portal interfaces required by the first-party shell.
9. AppStream metadata ingestion for the software store.
10. Autostart policy and user-visible controls.
11. Accessibility and keyboard workflow audits across the shell.

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
