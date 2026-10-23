AureaOS Roadmap

This roadmap describes the direction of the project. It is a plan, not a promise that every feature is already implemented. Priorities may change as we learn and test.

Phase 0 — Project foundations

* [x]	Create the public GitHub repository.
* [x]	Add the project README.
* [x]	Choose GNU GPL v3.0 for original code, while respecting the separate licenses of included components.
* [ ]	Document the initial architecture and project conventions.
* [ ]	Define how third-party licenses and notices will be tracked.

Phase 1 — Build a reliable base

* [ ]	Set up the development environment.
* [ ]	Create a repeatable build process based on Debian tools.
* [ ]	Produce a first bootable x86-64 test image.
* [ ]	Test booting and basic hardware support on real computers.
* [ ]	Document known limitations and recovery steps.

Phase 2 — Establish the system experience

* [ ]	Define the Aurea Design System: typography, spacing, colors, icons, controls, and motion.
* [ ]	Prototype the desktop shell and core window-management behavior.
* [ ]	Establish a consistent settings and permissions model.
* [ ]	Define how system updates and recovery will work.

Phase 3 — Develop original components

Planned components include:

* Aurea Desktop — desktop shell and window experience.
* Quaero — global search for applications, files, folders, and settings.
* Fides — file manager.
* Aurea Store — application discovery and management.
* Aurea Update — operating-system updates.
* Aurea Backup — backup tools.
* Aurea Recovery — recovery and rollback tools.

Each component will be planned, implemented, and tested in small steps. These names describe intended projects, not completed features.

Phase 4 — AureaOS Roma 1.0

* [ ]	Deliver a bootable and installable x86-64 system.
* [ ]	Provide a clear installation experience.
* [ ]	Make core hardware and networking work reliably on supported devices.
* [ ]	Establish a dependable update and recovery path.
* [ ]	Publish installation instructions, known issues, and release notes.

Guiding priorities

1. Reliability before adding more features.
2. A usable system before visual polish alone.
3. Small, testable changes instead of large, fragile rewrites.
4. Reuse mature Linux technologies where appropriate.
5. Keep the project open, documented, and respectful of upstream licenses.

Current status: planning and early project setup. No roadmap item should be considered complete unless the repository contains the implementation and its results have been tested.