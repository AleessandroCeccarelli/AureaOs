<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Vision

## Guiding Idea

**“La tecnologia è bella quando c’è e non si vede.”**

*Technology is beautiful when it is there, but you barely notice it.*

AureaOS aims to make Linux easier to use without hiding its flexibility. An operating system should help people do what they want without making them manage unnecessary complexity.

Technology should support the experience rather than become an obstacle.

## What We Believe

- Technology should adapt to people.
- Simplicity and capability can coexist.
- The desktop, applications, settings, and system tools should feel consistent.
- Users should not need a cloud account to use the system.
- Visual design and motion should support clarity and usability.
- Installation, updates, backup, and recovery should be understandable and dependable.
- A modular architecture should make the system easier to maintain, extend, and improve.
- Established software should be reused when it makes more sense than reinventing it.
- The operating system should have its own identity, without forcing every application to be developed by the project.

## Our Approach

AureaOS is a GUI-first, Wayland-first Linux distribution based on Debian GNU/Linux 13 (Trixie), targeting `amd64` (x86-64). The first release is **AureaOS Roma 1.0**.

Debian provides the lower-level system foundation. AureaOS develops its own desktop environment, system integration, visual identity, and core applications.

The goal is to build a coherent operating system experience rather than simply repackage Debian with a different appearance.

AureaOS will reuse mature Linux infrastructure and selected third-party software wherever this is technically and practically appropriate.

The repository has progressed beyond the initial Debian bootstrap. It includes early Rust implementations of `aurea-shell` and `aurea-compositor`, together with initial Wayland, DRM device, and display output management foundations.

These components are an early technical foundation, not a complete graphical desktop. The desktop UI, complete Vulkan rendering and presentation pipeline, installer, bootable system image, and first-party end-user applications remain under development or planned.

See the [roadmap](ROADMAP.md) for current implementation status and next steps.

## A Desktop That Feels Natural

The intended desktop should be modern, accessible, and coherent.

Warm accents, considered typography, translucent surfaces, and restrained motion are design directions; they must serve readability and usability.

Everyday tasks should have graphical workflows. The terminal remains available for advanced users, developers, and administrators, but ordinary desktop operations should not require terminal commands.

The desktop should feel like one integrated environment rather than a collection of unrelated applications and system tools.

AureaOS should provide sensible defaults while allowing users to retain control over their system.

## Our Own Product Identity

AureaOS is responsible for its own desktop experience and the system applications that define its identity.

The first release is centered on:

- **Aurea Desktop:** the AureaOS-designed graphical desktop environment and window-management experience.
- **AureaOS Human Interfaces:** the shared design system and planned native UI library.
- **Aurea Wallet:** a first-party system application and an important pillar of the product.
- **Fides:** the AureaOS file manager.
- **Quaero:** global search across applications, files, folders, and settings.
- **Aurea Store:** the official application store.
- **Aurea Update:** updates for AureaOS system components.
- **Aurea Time:** system backup and restore.

These components represent the intended Roma 1.0 product direction. Their presence in the vision does not mean they are already implemented or complete.

## A Pragmatic Application Ecosystem

AureaOS does not need to reinvent every desktop application to provide a complete operating system.

For Roma 1.0, established third-party applications will cover common needs such as:

- Web browsing.
- Email.
- Office and productivity work.
- Other everyday desktop tasks.

Aurea Store will integrate with Flathub as an application source and Flatpak for installation, execution, sandboxing, and updates.

This approach allows AureaOS to concentrate its resources on its own desktop environment, Wallet, file manager, global search, store, and system integration.

Third-party applications remain distinct from AureaOS first-party products.

Future releases may introduce first-party alternatives for selected application categories when development resources and technical priorities make that practical.

## Technology and Architecture

AureaOS follows a modular architecture and uses Rust as its primary programming language for first-party development where appropriate.

Its technical direction includes:

- Debian GNU/Linux 13 (Trixie).
- Wayland and Smithay for the desktop graphics architecture.
- Vulkan as the selected and exclusive graphics rendering API.
- PipeWire for multimedia infrastructure.
- NetworkManager for network management.
- systemd for system and service management.
- GRUB2 as the planned bootloader.
- Btrfs as the selected default installed filesystem.

The selection of these technologies does not mean their complete integration is already implemented.

AureaOS aims to keep responsibilities clearly separated between the Debian foundation, AureaOS system components, desktop infrastructure, first-party applications, and third-party software.

The architecture should remain modular, testable, and maintainable as the project grows.

## A Dependable System

A good desktop experience depends on more than its appearance.

AureaOS should provide:

- A clear and reliable installation process.
- Predictable system updates.
- Separate update responsibilities for Debian, AureaOS, and Flatpak applications.
- Understandable backup and recovery workflows through Aurea Time.
- Appropriate security boundaries and transparent application permissions.
- Accessible interfaces and consistent behavior.
- Clear documentation of supported hardware and limitations.

Reliability, maintainability, and user trust must take priority over unnecessary feature count.

## First Release

**AureaOS Roma 1.0** is the project's first release target.

The initial platform is Debian 13 Trixie on `amd64`.

The first release should establish a reliable Debian foundation and deliver a coherent AureaOS desktop with its essential first-party applications and system integration.

It should also provide access to established third-party applications through Aurea Store.

The objective is not to develop every possible application. It is to deliver a usable operating system with a recognizable identity, a consistent experience, and a solid technical foundation.

## Motto

**Take the simplicity people love in operating systems, and shape it to fit Linux.**

AureaOS aims to make Linux capabilities easier to reach while respecting the openness and flexibility of the platform.


## Project Licensing

Original AureaOS work is currently licensed under **GPL-3.0-or-later**. The project's ownership of its product identity, design direction, desktop experience, and first-party applications does not imply closed-source software. Third-party components remain under their respective licenses. Any future change to this policy is undecided and will be evaluated separately.
