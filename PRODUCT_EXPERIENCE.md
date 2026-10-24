# AureaOS — Product Vision and User Experience

### 1. What AureaOS Wants to Be

AureaOS wants to bring the user experience of Windows and macOS to Linux, without forgetting its roots.

It is not intended to be another generic Linux distribution, nor a Linux system disguised with a Windows-like theme. AureaOS aims to become a complete, coherent and carefully designed operating system: one that combines the freedom, openness and technical foundations of Linux with the simplicity, accessibility, hardware integration and attention to detail that users expect from a modern desktop operating system.

The goal is to make Linux approachable without stripping away its identity.

AureaOS should offer an experience in which the desktop, system settings, applications, hardware management, security, updates and everyday workflows feel like parts of the same product. Users should not have to assemble their own operating system from unrelated components before they can start using their computer.

AureaOS should feel like a complete operating system, not a collection of packages that the user must finish configuring.

The guiding principle is simple:

The technology should be there when people need it, and invisible when they don’t.

### 2. Windows and macOS as Inspiration, Linux as Our Foundation

Windows and macOS have established important expectations for personal computing: a recognizable desktop, intuitive settings, straightforward application installation, consistent interactions, automatic hardware configuration and integrated system services.

AureaOS wants to deliver that level of usability on Linux.

We do not intend to copy the interface of Windows or macOS, reproduce their branding, or turn Linux into an imitation of another operating system. Instead, we want to learn from the experiences they provide and build our own interpretation, with a distinct visual identity, original applications and a coherent design philosophy.

Linux remains the foundation. Its flexibility, transparency, openness and technical capabilities are strengths to build upon, not obstacles to hide.

AureaOS will make deliberate choices about its default experience. The system should be ready to use after installation, with sensible defaults and essential functionality already integrated. Customization will remain possible, but users should not need to spend hours configuring the desktop to obtain a polished and productive environment.

We want the approachability of Windows and macOS, the foundations of Linux, and the identity of AureaOS.

### 3. Debian as the Foundation, AureaOS as the Product

AureaOS will use Debian Stable as its initial technical foundation, retaining the Linux kernel and proven system infrastructure wherever they serve the project’s goals.

There is no reason to reinvent mature technologies simply to claim that everything is new. The objective is to build a better integrated experience on top of reliable foundations.

Debian provides the foundation; AureaOS defines the experience.

The project will develop its own desktop environment, system tools, first-party applications and integration layer. The result should have a clear identity and a consistent behavior instead of feeling like a standard Debian installation with a different wallpaper and theme.

AureaOS is intended to be an independent product with its own design decisions, workflows and long-term technical direction.

The initial target is x86-64 hardware. Hardware compatibility, installation, usability, security and system maintenance are core parts of the product, not secondary concerns.

### 4. Hardware Recognition and Vendor Drivers

Hardware should work without forcing users to become Linux administrators.

AureaOS aims to detect and configure supported hardware automatically, minimizing manual intervention and making the computer usable as soon as possible after installation.

This includes graphics, audio, Wi-Fi, Bluetooth, touchpads, keyboards, printers, cameras, power management, suspend and resume, and other essential hardware features.

When a suitable driver or firmware supplied by the hardware manufacturer is available, AureaOS will prioritize evaluating and using the manufacturer’s solution.

Vendor-provided drivers can offer access to hardware-specific functionality, performance optimizations, power-management features and compatibility that may not be available to the same extent in alternative implementations.

AureaOS will not reject proprietary drivers simply because they are proprietary. The objective is to provide the best practical experience on supported hardware, rather than impose ideological restrictions on which technologies can be used.

This does not mean assuming that every proprietary driver is always superior. Results depend on the manufacturer, device, driver quality, maintenance, security, compatibility and update support. Where an open driver provides a better or more sustainable result, it remains a valid choice.

Driver selection must also respect licensing, redistribution conditions, security requirements and long-term maintainability.

The principle is straightforward: hardware support should be guided by compatibility, reliability and user experience, with vendor drivers preferred when they are the most suitable available solution.

Users should not have to search forums, execute obscure terminal commands or manually hunt for drivers to make essential hardware function.

### 5. Aurea Desktop — A Desktop Environment Built for AureaOS

AureaOS will develop its own desktop environment rather than making an existing desktop the identity of the product.

Aurea Desktop will be designed as an integrated environment, bringing together the graphical shell, window management, panels, dock, workspaces, notifications, system tray, application launcher and system settings.

Every part should follow a shared design language and behave consistently.

The desktop must be responsive, accessible, visually coherent and pleasant to use. Animations, transparency, typography and visual effects should support the experience rather than compromise performance or distract from the user’s work.

The goal is not to recreate the Windows desktop or the macOS interface. Aurea Desktop should establish its own identity while offering the familiarity and predictability people expect from a modern operating system.

The intended architecture is native Wayland, with a compositor and shell designed around AureaOS’s needs. Rust is a preferred implementation language where appropriate, and Vulkan is a rendering direction to evaluate where the architecture and hardware make it suitable.

These are architectural goals, not claims that the components are already implemented.

### 6. First-Party Applications and System Integration

AureaOS will develop its own applications and system tools for the functions that define its experience.

These applications should not merely share a logo. They should share design principles, interaction patterns, accessibility, security expectations and integration with the operating system.

The planned core components include:

* Aurea Desktop — the native desktop environment and graphical shell.
* Quaero — global search for applications, files, folders, settings and other relevant system content.
* Fides — the AureaOS file manager, designed for everyday file management and integrated with the desktop and system services.
* Aurea Store — a graphical application store centered on Flatpak and Flathub.
* Aurea Update — a clear and understandable system and application update experience.
* Aurea Backup — tools to help users protect their personal data.
* Aurea Recovery — recovery facilities for common system problems.
* Aurea Wallet — a planned system component for securely managing supported credentials and related security features.
* System Passkeys — operating-system-level support for passkeys, designed to make modern authentication accessible and consistent across supported applications and services.
* ChatGPT System Integration — a planned integration that makes ChatGPT available as part of the operating-system experience rather than merely as a website the user must open manually.

These components are part of the intended product direction. Their implementation, interfaces, security models and technical scope will be defined as development progresses.

### 7. Fides — A File Manager That Belongs to AureaOS

File management is one of the most frequent tasks performed on a computer. It should be intuitive, efficient and consistent with the rest of the operating system.

Fides will be AureaOS’s own file manager, designed around everyday workflows such as browsing folders, searching for files, copying and moving content, managing external drives, handling archives and accessing network locations where supported.

It should integrate naturally with Quaero, the desktop, application dialogs and other system services.

The objective is to make file management accessible to new users without unnecessarily limiting experienced users. Common actions should be easy to discover, while advanced functionality should remain available when needed.

Fides should feel like a native part of AureaOS, not an unrelated application that happens to run on it.

### 8. Quaero — Global Search Across the System

Quaero will provide a unified way to find and access content throughout AureaOS.

Instead of forcing users to navigate menus and folders to locate every item, global search should offer a direct route to applications, files, folders, settings and other supported system functions.

The long-term direction is a search experience that becomes a central entry point for working with the computer: open an application, locate a document, find a setting or reach a supported action from one consistent interface.

Search should be fast, predictable and relevant. Its results must respect user permissions and privacy, and users should have control over which content sources are indexed.

Quaero should reduce the distance between what a user wants to do and the action that accomplishes it.

### 9. Aurea Wallet and System-Level Passkeys

Security should be integrated into the operating system instead of being left entirely to individual applications.

AureaOS plans to explore Aurea Wallet as a dedicated system component for supported credentials and related security functionality. Its precise scope will be established through explicit security and architecture decisions.

A central objective is to support passkeys at the operating-system level, allowing compatible applications and services to use modern passwordless authentication through a consistent system experience.

Where supported, users should be able to create, manage and use passkeys without having to understand the underlying authentication protocols or manage a different process for every application.

This requires careful integration with established authentication standards, secure credential storage, access controls, application permissions and appropriate recovery mechanisms.

Passkeys are not simply another type of password, and a system Wallet must not be treated as an ordinary file manager or an unprotected database of secrets. Credential security, user consent and protection against unauthorized access are fundamental requirements.

AureaOS should make strong security easier to use, without pretending that security can be achieved through interface design alone.

### 10. ChatGPT as Part of the Operating-System Experience

AureaOS aims to explore a deeper integration of ChatGPT into the desktop and system experience.

The intention is to make AI assistance available where it can genuinely help users, rather than limiting access to a separate browser tab.

Depending on technical feasibility, available APIs, privacy requirements and service terms, the planned integration could provide an accessible system entry point for asking questions, understanding information, getting help with everyday tasks and interacting with supported workflows.

The integration should feel native to AureaOS, with clear boundaries between the operating system, the AI service and the applications involved.

It must not be represented as though ChatGPT were an offline system component or as though it automatically had unrestricted access to files, applications or personal data.

Privacy, transparency and user control are essential. Users should understand when information is sent to an external service, what information is involved and which permissions or integrations are active. Sensitive actions should require appropriate confirmation, and access to local content must be governed by explicit permissions and secure implementation.

The goal is to make AI a useful part of the computing experience without sacrificing user control or turning the operating system into an opaque service.

### 11. Aurea Store — Applications Without Unnecessary Complexity

AureaOS plans to provide its own graphical application store, centered on Flatpak and Flathub where appropriate.

Users should be able to discover, install, launch and update applications through a clear interface without needing to understand package managers or execute terminal commands for ordinary software installation.

The store should present useful application information and explain relevant permissions in an understandable way.

Flatpak and Flathub will be important parts of the application-distribution strategy, but the project must still consider application provenance, permissions, maintenance, security and trust. A graphical store does not automatically make every application safe.

Operating-system updates and desktop application updates should be integrated into the overall maintenance experience while remaining technically distinguishable where necessary.

The aim is to make software installation feel like a natural part of AureaOS rather than a separate technical task.

### 12. Updates, Recovery and Reliability

A system that is easy to use must also be easy to maintain.

AureaOS should provide clear update workflows, meaningful progress information and understandable explanations when something goes wrong. Users should not have to guess whether an update has completed or whether the system needs attention.

Aurea Backup and Aurea Recovery are intended to help users protect their data and recover from supported problems.

Reliability must be considered throughout the design: safe defaults, clear error handling, predictable behavior, maintainable components and careful testing are all part of the user experience.

AureaOS should favor dependable functionality over unnecessary complexity and avoid introducing features that cannot be maintained responsibly.

### 13. Privacy, Freedom and Linux Roots

Bringing a more approachable experience to Linux must not mean abandoning the principles that make Linux valuable.

AureaOS aims to preserve user control, support open technologies and remain transparent about its architecture and dependencies. The system should not require a cloud account simply to use the desktop or perform ordinary local tasks.

At the same time, the project recognizes that a practical operating system may need proprietary drivers, firmware, third-party applications or online services to provide the best possible experience on particular hardware or for particular tasks.

Those choices should be made deliberately, documented honestly and evaluated according to compatibility, security, licensing, privacy and maintainability.

Openness is a technical and philosophical strength. It should enable informed choices, not become a reason to ignore the needs of ordinary users.

AureaOS should respect Linux’s roots while making the benefits of Linux accessible to more people.

### 14. Technical Direction

The intended technical direction includes:

* Debian Stable as the initial system foundation.
* Linux kernel and established system infrastructure.
* Rust for AureaOS components where appropriate.
* Wayland as the native desktop display architecture.
* Vulkan as a rendering direction where technically suitable.
* PipeWire and WirePlumber for audio.
* NetworkManager for network connectivity.
* systemd for system startup and service management.
* Flatpak and Flathub for the planned application-store experience.
* Vendor drivers and firmware when they are the most suitable supported solution and their use is permitted.
* Native AureaOS applications and system services to provide the integrated product experience.

These choices describe the intended direction. Specific dependencies, technical requirements, licensing conditions and implementation details must be validated during development.

The project will not rewrite mature infrastructure merely for the sake of originality. It will focus its development effort on the components that establish AureaOS’s identity and improve the experience as a whole.

### 15. What Success Looks Like

AureaOS will be moving toward its goal when a user can install it on supported hardware and immediately access a coherent desktop, working connectivity, functional audio and graphics, useful system settings, global search, an integrated file manager and straightforward application installation.

The user should be able to manage updates, protect personal data and use supported authentication features without being forced to understand the underlying system.

The operating system should also provide a clear and responsible path toward system-level passkeys, Aurea Wallet and ChatGPT integration, with each feature implemented according to its real technical and security requirements.

Experienced users should retain access to Linux tools and the underlying system. Simplicity must not mean removing capability, hiding important choices or taking control away from the user.

AureaOS is not about hiding Linux, imitating Windows or copying macOS. It is about building a distinct operating system that brings the familiarity and ease of use people expect from mainstream desktop platforms to the flexibility and foundations of Linux.

AureaOS: the experience people expect, built on the foundations we believe in.

⸻

### This document describes the product vision, architectural direction and planned components of AureaOS. It does not imply that every feature described is already implemented. Implementation status, technical decisions and development priorities must be tracked separately in the project documentation.