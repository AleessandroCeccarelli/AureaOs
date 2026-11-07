
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Vision

## Guiding idea

**“La tecnologia è bella quando c’è e non si vede.”**

*Technology is beautiful when it is there, but you barely notice it.*

AureaOS aims to make Linux easier to use without hiding its flexibility. An operating system should help people do what they want without making them manage unnecessary complexity.

## What we believe

- Technology should adapt to people.
- Simplicity and capability can coexist.
- The desktop, applications, settings, and system tools should feel consistent.
- Users should not need a cloud account to use the system.
- Visual design and motion should support clarity and usability.
- Installation, updates, backup, and recovery should be understandable and dependable.
- A modular architecture should make the system easier to maintain, extend, and improve.

## Our approach

AureaOS is a GUI-first, Wayland-first Linux distribution based on Debian GNU/Linux 13 (Trixie), targeting `amd64` (x86-64). The first release is **AureaOS Roma 1.0**.

Debian is the technical foundation. AureaOS aims to build its own desktop experience, system integration, and applications while reusing established Linux infrastructure where it makes sense.

The repository has progressed beyond the initial Debian bootstrap. It now includes early Rust implementations of `aurea-shell` and `aurea-compositor`, together with initial Wayland, DRM device, and display output management foundations.

These components are an early technical foundation, not a complete graphical desktop. The desktop UI, complete graphics pipeline, installer, bootable system image, and first-party end-user applications remain under development or planned.

See the [roadmap](ROADMAP.md) for current implementation status and next steps.

## A desktop that feels natural

The intended desktop should be modern, accessible, and coherent.

Warm accents, considered typography, translucent surfaces, and restrained motion are design directions; they must serve readability and usability.

Everyday tasks should have graphical workflows. The terminal remains available for advanced users, developers, and administrators, but ordinary desktop operations should not require terminal commands.

The desktop should feel like one integrated environment rather than a collection of unrelated applications and system tools.

## First release

**AureaOS Roma 1.0** is the project's first release target.

The initial platform is Debian 13 Trixie on `amd64`.

Development should proceed in stages, establishing a reliable system foundation and desktop architecture before expanding the user experience and application ecosystem.

AureaOS should prioritize stability, consistency, accessibility, and maintainability over feature count.

## Motto

**Take the simplicity people love in operating systems, and shape it to fit Linux.**

AureaOS aims to make Linux capabilities easier to reach while respecting the openness and flexibility of the platform.
