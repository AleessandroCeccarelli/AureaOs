# AureaOS Initial Architecture

## 1. Purpose

This document defines the initial technical architecture of AureaOS Roma 1.0.

Its purpose is to establish clear boundaries between the Debian foundation, AureaOS-specific system integration, and the future desktop experience.

This is an initial architecture document. Decisions that have not yet been evaluated or implemented must not be treated as completed work.

## 2. Project Goals

AureaOS aims to deliver a coherent, reliable, and approachable Linux operating system built on Debian.

The initial release targets x86-64 hardware and prioritizes:

- A reproducible and testable system foundation.
- A modern Wayland-based desktop architecture.
- Sensible defaults and consistent system behavior.
- Local user accounts without a mandatory cloud account.
- Straightforward installation, maintenance, updates, backup, and recovery.
- Clear documentation of supported hardware, limitations, dependencies, and licenses.

## 3. Architectural Principles

The architecture follows these principles:

1. **Debian as the foundation:** reuse proven Linux infrastructure instead of unnecessarily reimplementing existing system capabilities.
2. **Separation of responsibilities:** distinguish the base operating system, AureaOS-specific configuration, build tools, tests, and desktop applications.
3. **Reproducibility:** system artifacts must be produced through documented and repeatable procedures.
4. **Reliability first:** changes must be testable, documented, and introduced incrementally.
5. **Open technologies:** dependencies must be evaluated for licensing, security, maintenance, compatibility, and long-term suitability.
6. **Wayland-first desktop:** the intended desktop architecture for Roma 1.0 excludes X11 and XWayland. Exceptions, if any, require explicit evaluation and documentation.
7. **Coherent user experience:** system components must support a consistent and accessible operating environment.
8. **No unnecessary reinvention:** AureaOS development should focus on integration and features that distinguish the project.

## 4. Initial System Layers

The system is conceptually divided into the following layers.

### 4.1 Debian Base

The Debian base provides the underlying Linux operating system and established system infrastructure.

Its exact release, package selection, initialization system, service configuration, and security policies must be documented before implementation.

### 4.2 AureaOS System Configuration

This layer contains AureaOS-specific configuration and integration built on top of Debian.

Its responsibilities may include:

- Base package selection.
- System defaults and configuration.
- Service configuration.
- System identity and release metadata.
- Integration of selected third-party components.

Only requirements that have been evaluated and documented should be implemented.

### 4.3 Build and Validation Tools

Build tools will provide a documented process for preparing and validating AureaOS system artifacts.

The chosen build technology must support the project's target architecture, reproducibility goals, maintainability, and licensing requirements.

The build technology and its implementation remain to be selected and verified.

### 4.4 AureaOS Desktop and Applications

The desktop experience will be developed as a separate layer that integrates with the underlying system.

Planned components include:

- Aurea Desktop.
- Quaero.
- Fides.
- Aurea Store.
- Aurea Update.
- Aurea Backup.
- Aurea Recovery.

These components are planned features and are not considered implemented by this document.

## 5. Repository Organization

The repository will organize project documentation, system configuration, build tooling, and tests into clearly defined areas.

The proposed initial structure is:

```text
AureaOs/
├── assets/
├── docs/
│   └── architecture/
│       └── INITIAL-ARCHITECTURE.md
├── debian/
│   ├── packages/
│   ├── config/
│   └── services/
├── tools/
│   └── build/
├── tests/
├── README.md
├── ROADMAP.md
├── TECHNOLOGY.md
├── VISION.md
├── PHILOSOPHY.md
├── PRODUCT_EXPERIENCE.md
├── CONTRIBUTING.md
└── LICENSE
```

This structure is a proposal, not a claim that all directories or implementation files already exist.

Directories will be created when their responsibilities and contents are sufficiently defined. Their organization may be refined as the build process and system architecture become clearer.

## 6. Dependencies and Licensing

Third-party components remain subject to their respective licenses.

Before integrating a dependency, AureaOS should record, as applicable:

- Component name and version.
- Source and upstream project.
- License.
- Intended purpose.
- Maintenance and security considerations.
- Compatibility with the target architecture and desktop direction.

A dedicated dependency inventory and a process for reviewing licenses must be established during the technical foundation phase.

## 7. Security and System Maintenance

Security and maintainability are architectural requirements, not optional additions.

The initial design must account for:

- Supported security updates.
- Appropriate system permissions.
- Documented service responsibilities.
- Safe system configuration and upgrade procedures.
- Recovery and troubleshooting.
- Clear separation between system-managed files and user data.

Detailed security policies, update mechanisms, and recovery procedures remain to be designed and validated.

## 8. Testing and Validation

The technical foundation must be validated incrementally.

Initial validation goals include:

- Verifying repository structure and build-tool prerequisites.
- Checking the consistency of package and configuration definitions.
- Establishing repeatable build procedures.
- Producing a bootable x86-64 test image once the build foundation is ready.
- Testing boot, installation, and basic hardware compatibility.
- Documenting failures, limitations, and recovery procedures.

A successful build alone does not establish that the resulting operating system is reliable. Testing must verify the resulting artifacts and their behavior.

## 9. Decisions Still to Be Defined

The following decisions require evaluation and documentation before the corresponding implementation begins:

- The Debian release and repository policy.
- The system build technology and build environment.
- The initial package selection.
- The initialization system and service-management approach.
- The Wayland compositor and associated desktop infrastructure.
- The graphics, networking, audio, and hardware integration strategy.
- The security and system-update policies.
- The dependency inventory and license-review process.
- The initial automated testing strategy.

These decisions must be made according to the project's technology principles and documented before being treated as established architecture.

## 10. Initial Implementation Priorities

Development will proceed in the following order:

1. Review and document the initial architecture.
2. Establish the development environment.
3. Select and document the Debian build approach.
4. Create the initial system configuration and package definitions.
5. Implement a repeatable build process.
6. Validate the resulting system artifacts.
7. Prepare a bootable x86-64 test image.
8. Test boot, installation, and basic hardware compatibility.
9. Document known limitations and recovery procedures.

Desktop application development will expand as the technical foundation becomes sufficiently reliable.

## 11. Current Status

AureaOS is in the planning and early setup stage.

This document establishes the initial architectural direction. It does not indicate that the system configuration, build tools, desktop components, or test image have already been implemented.

Architecture decisions and implementation status must be updated as the project progresses.

---

**AureaOS Roma 1.0 — Reliability first. A coherent experience. Built on proven foundations.**