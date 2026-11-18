<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Technical Reference Boundaries

This document records external projects and specifications used to review AureaOS engineering decisions. A reference is not a dependency, a source-code origin, or a commitment to reproduce another project's architecture.

## Reference hierarchy

| Reference | Role in AureaOS | Boundary |
|---|---|---|
| [Wayland protocol](https://gitlab.freedesktop.org/wayland/wayland) | Normative protocol definitions and interoperability requirements | Protocol behavior is authoritative; implementation details are not copied from unrelated compositors |
| [Smithay](https://github.com/Smithay/smithay) | Rust infrastructure used by Aurea Compositor | Use its public APIs and documented extension points; AureaOS owns its compositor policy and implementation |
| [Vulkan](https://github.com/KhronosGroup/Vulkan-Docs) | Graphics API specification and validation reference | Vulkan is the exclusive AureaOS rendering API |
| [COSMIC compositor](https://github.com/pop-os/cosmic-comp) | Comparative implementation and engineering reference | Read-only study; not a runtime dependency, code source, or architectural template |

## COSMIC comparison policy

COSMIC is included as a permanent comparative reference because it is a substantial Rust/Smithay compositor. Its implementation may be reviewed when evaluating bounded topics such as:

- Cursor image lifecycle, hotspot handling, and cursor-surface rendering.
- Pointer focus, input routing, and interaction state.
- Wayland surface-tree traversal and stacking.
- Output lifecycle, DRM/KMS integration, and frame scheduling.
- Rendering resource ownership, caching, and compositing.

Every adopted idea must be independently evaluated against AureaOS requirements, implemented in AureaOS-owned code, and covered by project-specific tests. No COSMIC source code is copied into AureaOS, and COSMIC is not added as a Cargo dependency.

AureaOS explicitly excludes incompatible rendering and platform paths from its target architecture, including OpenGL/GLES rendering, Pixman rendering, X11, and XWayland. The presence of such paths in a reference project does not change AureaOS's Wayland-only and Vulkan-only requirements.

## Decision record

Before adding an external dependency or adapting a design, the change must identify:

1. The concrete AureaOS problem being solved.
2. The relevant protocol or API contract.
3. Why the selected approach fits the Wayland-only, Vulkan-only architecture.
4. Its ownership, lifecycle, security, and resource limits.
5. The tests and hardware validation required.

References inform review; they do not replace protocol specifications, API documentation, threat analysis, or AureaOS design decisions.
