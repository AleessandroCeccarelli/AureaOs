
# AureaOS Roadmap

This roadmap describes the intended direction of AureaOS. It is a working plan, not a promise that every feature will be delivered exactly as written.

The project will progress in stages, prioritizing a reliable foundation before expanding the desktop experience.

## Phase 0 — Project Setup

- [x] Create the public GitHub repository.
- [x] Add the initial README.
- [x] Choose GPL-3.0 for the project's original work.
- [x] Document the project's vision and guiding principles.
- [x] Add contribution guidelines.
- [ ] Document the initial technical architecture.
- [ ] Define how third-party components and their licenses will be tracked.

## Phase 1 — Technical Foundation

The first priority is to establish a reproducible, testable foundation.

- [ ] Set up the development environment.
- [ ] Define the initial system architecture.
- [ ] Create a repeatable Debian-based build process.
- [ ] Produce a bootable x86-64 test image.
- [ ] Test boot, installation, and basic hardware compatibility.
- [ ] Document supported hardware and known limitations.
- [ ] Define initial system recovery and troubleshooting procedures.

## Phase 2 — Desktop Experience

Once the technical foundation is reliable, work can begin on the desktop experience.

- [ ] Define the AureaOS design system.
- [ ] Prototype the Aurea Desktop shell.
- [ ] Establish window management and core desktop interactions.
- [ ] Define consistent interfaces for settings and system tools.
- [ ] Design system notifications and application integration.
- [ ] Plan user permissions and essential security workflows.
- [ ] Design the update and recovery experience.

## Phase 3 — Core Components

### Planned Components

The following components are planned. They are not yet implemented.

* [ ]	Aurea Desktop: native desktop shell, window management, floating dock, application previews, virtual desktops, and desktop interactions.
* [ ]	Aurea Greeter: graphical login screen and user session entry.
* [ ]	Aurea Human Interfaces: shared UI component library and design system for first-party applications.
* [ ]	Aurea System API: shared interfaces for system services and native applications.
* [ ]	Quaero: global search for applications, files, folders, and settings.
* [ ]	Fides: graphical file manager, archive handling, and file previews.
* [ ]	Aurea Store: graphical application discovery, installation, and management using Flatpak and Flathub.
* [ ]	Aurea Update: graphical system update manager, separate from application updates.
* [ ]	Aurea Time: backup application with incremental backups, version history, encryption, and restoration.
* [ ]	Aurea Recovery: graphical system recovery and reinstallation tools.
* [ ]	Aurea AI: planned AI integration for the AureaOS environment.
* [ ]	Aurea Wallet: planned secure management of passwords and passkeys.
* [ ]	Printing and Scanning: graphical printer and scanner configuration, printing queues, and document scanning using Linux infrastructure such as CUPS and SANE.

### Printing and Scanning

* [ ]	Integrate CUPS for printing and SANE for scanner support.
* [ ]	Support compatible USB and Wi-Fi network printers and scanners.
* [ ]	Enable automatic network device discovery using supported standard protocols.
* [ ]	Provide a native graphical interface for printer and scanner setup.
* [ ]	Manage print queues, job status, and errors.
* [ ]	Support scanning to PDF and image formats, including multipage scanning when available.
* [ ]	Test compatibility with multifunction devices from different manufacturers.

### Shared Design and Integration

- [ ] Define shared design conventions and reusable interface components.
- [ ] Establish consistent notification and badge behavior.
- [ ] Define integration points for application windows and desktop features.
- [ ] Establish clear permission and confirmation patterns.
- [ ] Document the interfaces needed for future system components.

## Phase 4 — AureaOS Roma 1.0

The first planned release will focus on a usable, installable, and coherent system rather than an excessive number of features.

- [ ] Produce an installable x86-64 system image.
- [ ] Provide a straightforward graphical installation experience.
- [ ] Establish a defined set of supported hardware.
- [ ] Deliver the initial desktop experience and essential system tools.
- [ ] Provide a reliable update workflow.
- [ ] Provide documented backup and recovery procedures.
* [ ]	Validate printing and scanning with supported USB and Wi-Fi multifunction devices.
- [ ] Publish installation, usage, and troubleshooting documentation.
- [ ] Document known limitations and release criteria.

## Guiding Priorities

Development decisions should follow these priorities:

1. **Reliability first:** build a stable foundation before adding complexity.
2. **A coherent experience:** make the system feel like one product.
3. **Usability:** prioritize graphical workflows for everyday tasks.
4. **Freedom and transparency:** respect open-source principles and third-party licenses.
5. **Incremental progress:** prefer working, testable milestones over promises.

## Current Status

AureaOS is in the planning and early setup stage. The repository establishes the project's direction and provides a place to organize future development.

Implementation status will be updated as work progresses.

**Rome wasn't built in a day, and neither will AureaOS be.**
