<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS — Universal Desktop and Technology Strategy

- Status: Product and architecture direction; individual integrations remain subject to implementation and verification.
- Platform baseline: Debian GNU/Linux 13 (Trixie), amd64.
- Product: AureaOS Roma 1.0 and the longer-term Aurea Desktop ecosystem.

## 1. Product conclusion

AureaOS is a desktop for people, not a desktop for a technical demographic. It must be approachable to children, older adults, first-time computer users, students, creative professionals, office workers, administrators and software developers.

These are not separate desktop editions or rigid user profiles. They are different needs that one coherent desktop should serve through progressive disclosure, sensible defaults, accessible interaction, optional advanced tools and user-controlled preferences.

The product promise is:

**Simple by default, capable when needed, accessible from the beginning.**

A user should not need to understand Linux, a terminal, a package manager, a codec, a display server or a permission model to perform ordinary tasks. Advanced users must still be able to inspect, automate, configure and develop without artificial restrictions.

Multimedia and productivity are core desktop capabilities, not optional extras or future decorations.

## 2. Universal experience requirements

### 2.1 Accessibility is foundational

Accessibility is an architectural requirement for Aurea Desktop, Human Interfaces and first-party applications. It must not be postponed to a final polish phase.

The product must progressively support:
- scalable text and UI, high contrast, clear focus indication and reduced motion;
- keyboard-only navigation and discoverable shortcuts;
- mouse, touchpad, touchscreen, stylus and switch-friendly interaction where hardware and protocol support permit;
- screen-reader semantics, accessible names, roles, states, actions and change notifications;
- screen magnification, text-to-speech and speech input integrations;
- captions/subtitles, audio-device clarity and controls that do not rely on color alone;
- simple language, predictable confirmations, undo/recovery and safe defaults.

Children and older adults benefit from the same fundamentals: clear language, large targets, consistent behavior, understandable feedback and protection from destructive mistakes. Do not stereotype users or hide capabilities based on age.

Use the Linux accessibility ecosystem rather than inventing a private incompatible interface. Evaluate AT-SPI2 over D-Bus as the interoperability layer for assistive technologies and AccessKit as a possible Rust-facing abstraction for AureaOS-native controls. The selected native UI architecture must expose meaningful accessibility trees and be tested with real assistive technology, including Orca. These are integration targets, not claims of current support.

### 2.2 Progressive disclosure, not separate editions

- First-run experience should offer a small set of clear, reversible choices and explain them in ordinary language.
- Default surfaces should emphasize common actions; advanced controls remain discoverable rather than removed.
- Settings should use consistent terminology, previews and safe reset paths.
- Destructive actions require clear consequences and recovery options where possible.
- Touch targets, keyboard focus, scaling and contrast must be considered in the shared design system.
- Provide optional simplified layouts or guided experiences only when they solve a real task; do not create a second, divergent desktop shell.

### 2.3 Productivity and creative work

Aurea Desktop should make it easy to find, open, organize, create, share and recover work. The product direction includes:
- Fides for files, archives, external storage, previews and common file operations;
- Quaero for applications, files, settings and permitted system actions;
- notifications, clipboard, workspaces, multi-window and multi-monitor workflows;
- document, office, mail, browser and collaboration applications through a curated third-party ecosystem initially;
- screenshots, screen recording, camera, microphone, audio routing and media playback;
- local-first search and essential workflows that do not require a cloud account;
- backup and restore through Aurea Time.

Do not attempt to write an office suite, browser, mail client, video editor or every codec for Roma 1.0. Integrate mature applications through documented system interfaces and Flatpak portals, then consider first-party applications only where they create a clear product advantage.

### 2.4 Developer and administrator workflows

Advanced tools are a first-class use case but must not dominate the default experience:
- a reliable terminal and shell;
- developer tools, compilers, debuggers, Git and language toolchains;
- isolated development environments and containers, with clear resource and permission controls;
- logs, diagnostics, hardware information and system status presented both graphically and through standard CLI tools;
- automation and documented APIs for supported desktop services.

Do not make developer tools a mandatory part of the base image if they can be installed cleanly through Aurea Store or documented system packages.

## 3. Technology selection model

AureaOS uses three adoption lanes. A new technology is welcome, but novelty alone is not a reason to make it a system dependency.

### Lane A — Foundation

Use for core boot, session, input, display, security and everyday multimedia:
- mature upstream project, clear maintenance and security process;
- available in Debian stable or with a documented, supportable backport;
- stable interfaces and known upgrade path;
- observable failure modes, tests and practical recovery;
- no dependence on another desktop environment.

### Lane B — Product integration

Use established infrastructure behind AureaOS-owned APIs and user experience. Examples include PipeWire, WirePlumber, GStreamer, Flatpak, XDG Desktop Portal, AT-SPI2, libinput and NetworkManager.

AureaOS owns policy, UX, integration and product-specific contracts, not a duplicate implementation of mature infrastructure.

### Lane C — Research and incubation

New, immature or rapidly evolving projects may be evaluated in isolated prototypes:
- separate crate, process or feature flag where appropriate;
- no mandatory boot/session dependency;
- no unreviewed transitive dependency added to the stable base;
- explicit version, license, maintenance, security and compatibility review;
- comparison against a stable baseline;
- promotion only after measurable benefit and a migration/rollback plan.

Experimental work can inspire AureaOS APIs or eventually replace a component, but must not silently become a release requirement.

## 4. Recommended technology map

This is a direction and evaluation map, not a claim that the components are integrated.

| Domain | Recommended foundation | AureaOS responsibility | Maturity / decision |
|---|---|---|---|
| Base OS | Debian 13 stable, kernel and packaged drivers | Product integration, package policy and supported hardware matrix | Foundation; keep stable updates current |
| Session and services | systemd, D-Bus, logind/session mechanisms already selected | Aurea session lifecycle, service policy and user-facing controls | Foundation |
| Wayland compositor | Smithay, Wayland protocols, Calloop, libseat, DRM/KMS, Udev | Aurea compositor, shell, output and window policies | Foundation; existing project direction |
| Rendering | Vulkan only for AureaOS rendering | Aurea render graph, resource lifecycle and presentation | Foundation; physical hardware validation still required |
| Input | libinput, xkbcommon, kernel input interfaces | Unified settings, interaction policy, gestures and accessible behavior | Foundation |
| Audio/video routing | PipeWire + WirePlumber | Aurea controls, device UX, privacy indicators and routing UI | Foundation/product integration |
| Media pipelines | GStreamer 1.x | Aurea Media API, supported-format policy, capture/playback workflows | Product integration; Debian Trixie packages GStreamer 1.26.x |
| Still-image decoding | System libjpeg-turbo/TurboJPEG initially; evaluate other maintained decoders by format | aurea-media API, validation, limits, errors, scheduling and backend isolation | Conditional; ABI probe on Fedora and Debian required |
| Sandboxed apps | Flatpak + Flathub | Aurea Store, curated catalogue, permissions and app lifecycle UX | Product integration |
| App/system interoperability | XDG Desktop Portal + AureaOS portal backend | Native file chooser, screenshot, screencast, settings, open URI and permission UX | Product integration; implement only supported interfaces |
| Accessibility | AT-SPI2, Orca ecosystem; evaluate AccessKit for Rust-native UI | Accessible widget contracts, semantics, testing and settings | Required product capability; toolkit integration to prove |
| Text and typography | Approved AureaOS fonts; evaluate Parley or cosmic-text for Rust text layout | Shared text API, spelling, substitutions and typography | Candidate libraries; prototype before commitment |
| UI toolkit | No final selection yet | AureaOS Human Interfaces and native widget system | Open decision; benchmark with Vulkan/Wayland/accessibility |
| Networking | NetworkManager | Aurea network UI and connectivity workflows | Foundation |
| Documents and office | Mature third-party applications, Flatpak where suitable | Integration, file associations, portals and consistent UX | Product integration; no office suite rewrite for Roma 1.0 |
| Development | Debian toolchains and curated developer packages; containers where justified | Friendly developer setup, diagnostics and optional tool bundles | Product integration |
| Backup/recovery | Btrfs foundation as selected; Aurea Time owns user workflow | Backup, restore, recovery validation and clear UX | First-party product responsibility |

Version examples above describe the Debian 13 package baseline reviewed for this document, not a permanent pin. The dependency register must record exact package versions, architectures, source repositories, runtime/build roles and verification evidence.

## 5. Multimedia strategy

Multimedia must be a system capability with clear boundaries.

- PipeWire transports and routes audio/video streams; WirePlumber manages session policy and device behavior.
- GStreamer provides modular playback, recording, encoding, muxing, streaming and media pipeline building blocks. Select plugins and codecs deliberately; plugin availability, patent/licensing and redistribution rules vary.
- aurea-media provides a stable AureaOS-owned API for still-image decoding and future image operations. It must not own Vulkan textures or compositor scene composition.
- aurea-capture coordinates capture sessions and permissions; PipeWire/GStreamer adapters perform stream transport and encoding. The compositor supplies only the authorized final frames.
- Hardware acceleration is an optimization, never the only path. Keep a tested software fallback where feasible.
- Decode and encode work must not block the compositor render/presentation thread.
- Untrusted media parsing requires bounded memory/time, strict validation, prompt security updates and a process-isolation review before broad exposure to arbitrary content.

Do not create a monolithic Aurea Media daemon now. Start with small libraries and adapters. Introduce a dedicated restricted helper process only when the threat model, multiple clients or fault containment justify it.

## 6. Interoperability and sandboxing

A Wayland-only and Vulkan-only product has real compatibility costs. AureaOS must not imply that every Flatpak application will work.

- Prioritize native Wayland applications and test real Flatpak packages on Aurea Desktop.
- Use XDG Desktop Portal as the supported permission-mediated bridge for file access, opening URIs, screenshots, screencasts, settings and other supported functions.
- Develop an AureaOS portal backend for the interfaces the compositor and shell can genuinely provide. Do not depend on GNOME/KDE desktop backends or import their desktop session.
- Keep the portal frontend's security model: validate requests, enforce least privilege, require consent for sensitive resources and isolate applications.
- Publish a compatibility matrix with tested application, architecture, runtime, Wayland protocol and graphics requirements.
- If an application requires X11/XWayland or OpenGL, mark it incompatible or unsupported unless a separately approved solution exists. Do not quietly violate the product's Wayland/Vulkan decisions.

## 7. Native UI toolkit decision

Do not select a toolkit because it is popular or because it is written in Rust. A native AureaOS toolkit must be evaluated against:
- Wayland client and compositor integration without X11/XWayland;
- Vulkan rendering compatibility without silently introducing OpenGL;
- text shaping, bidi, complex scripts, font fallback, IME and clipboard;
- AT-SPI2/AccessKit semantics and screen-reader behavior;
- keyboard, pointer, touch, stylus, high-DPI and fractional scaling;
- performance, memory, startup, theming and animation;
- stable APIs, upstream health, license and Debian packaging;
- testability and maintainability for a small team.

Prototype at least one representative settings window, a text-heavy form, a file picker and an accessible control before selecting. Evaluate Parley/cosmic-text as text-layout building blocks, not as complete UI toolkits. AccessKit is an accessibility building block, not a substitute for a complete toolkit or a complete AT-SPI2 integration.

AureaOS Human Interfaces should define design tokens, interaction contracts and accessibility semantics independently of whichever rendering/widget implementation is selected.

## 8. Technology lifecycle and dependency governance

Every dependency must have an owner and an entry in the English and Italian dependency registers.

Record:
- exact package/crate and version, upstream and Debian package name;
- runtime, build-time, test-only or optional classification;
- license and redistribution conditions;
- stable API/ABI expectations and supported version range;
- security policy, update cadence and known advisories;
- required features/plugins/codecs and their optionality;
- Fedora development verification and Debian target verification;
- fallback, replacement and removal plan.

Use Debian stable as the compatibility authority. Fedora is the fast development workstation and early-warning environment. A Fedora-only success does not establish Debian compatibility.

Prefer Debian-packaged shared libraries for system integration. Avoid vendoring or static bundling without a documented reason. Do not pin to an old vulnerable version merely to preserve a probe result; track supported ABI ranges and security updates.

## 9. Quality gates for user-facing capabilities

A feature is not ready merely because it compiles. Before release, assess:
- accessibility with keyboard-only operation and assistive technology;
- usability with first-time users and people with different needs;
- touchpad, touchscreen, pointer and multi-monitor behavior where relevant;
- performance on modest supported hardware and resource limits;
- offline/local-first behavior and privacy;
- permission denial, cancellation, failure and recovery;
- Debian package/runtime compatibility;
- security updates, logging and diagnostics;
- documentation and support boundaries.

The quality bar applies to the default experience, not only to an optional accessibility or advanced mode.

## 10. Immediate priorities

1. Continue the current desktop foundations and complete real hardware validation of the Vulkan/display/input path.
2. Keep the JPEG task narrowly scoped: validate the existing TurboJPEG probe on Fedora and Debian before implementing aurea-media.
3. Preserve and advance aurea-capture through explicit PipeWire/GStreamer adapters and portal contracts, without moving encoding into the compositor.
4. Design the AureaOS portal backend requirements before relying on Flatpak applications for core workflows.
5. Establish accessibility contracts in Human Interfaces and evaluate AT-SPI2/AccessKit before choosing the native widget toolkit.
6. Prototype candidate text/UI stacks with the actual Wayland/Vulkan constraints before making a toolkit decision.
7. Build the application compatibility matrix and curate the Roma 1.0 productivity/media app set.

No new crate, system package, daemon or UI toolkit is approved solely by this strategy document. Each implementation still requires a focused design, dependency-register entry, tests and Fedora/Debian verification.
