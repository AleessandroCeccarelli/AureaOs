# AureaOS Technology Principles

## Looking Forward, Building on Proven Foundations

AureaOS is built for the future. Its technology choices reflect a clear principle: adopt modern, mature, well-maintained, and genuinely open technologies, then integrate and shape them into one coherent operating system experience.

We do not reinvent infrastructure simply for the sake of doing so. We build on the strengths of the Linux ecosystem and focus our own development on the parts that make AureaOS distinctive.

## Modern by Design

AureaOS Roma 1.0 is designed around a native Wayland desktop. X11 and XWayland are not part of the intended desktop architecture. Applications selected for the default experience must be compatible with this direction; any exceptions must be deliberate, documented, and justified rather than included merely for legacy compatibility.

Our goal is not to preserve every historical workflow indefinitely. We want a desktop designed around modern display, input, security, and application technologies.

## Open Technologies, Carefully Chosen

AureaOS prioritizes technologies that are:

- **Open source**, with licenses compatible with their intended use and distribution.
- **Mature and proven**, with real-world use, active maintenance, and appropriate testing.
- **Modern and forward-looking**, using current interfaces and avoiding unnecessary dependence on obsolete technologies.
- **Interoperable**, so components work together instead of creating a fragmented experience.
- **Maintainable**, with documentation, a sustainable development path, and clear upgrade strategies.
- **Adaptable**, allowing AureaOS to integrate, configure, theme, and extend them where their licenses and architecture permit.

Open source alone is not enough: each dependency must be evaluated for its license, security, maintenance, compatibility, resource usage, and long-term suitability.

## Build What Makes AureaOS Different

AureaOS will reuse reliable Linux infrastructure for essential system capabilities instead of duplicating work that the ecosystem has already solved well.

Our own development will focus on the integrated desktop experience and the components that define AureaOS: its interface, window management, system settings, search, file management, application store, update experience, backup, and recovery tools.

We will select technologies according to the needs of each component. Native applications are appropriate where close system integration, efficiency, or performance matters. Web technologies, including Electron where suitable, can help us build consistent applications and web-based tools. No single implementation technology is mandatory for every application. Every choice must serve the overall experience.

## One Coherent System

AureaOS should not feel like a collection of unrelated projects assembled together. Even when its underlying components come from different open-source communities, they should work together as a unified product.

That means consistent interaction patterns, visual design, accessibility, system integration, and clear user-facing behavior. Complexity should remain behind the scenes whenever possible.

## Compatibility Is a Deliberate Choice

AureaOS will not pursue compatibility with every legacy technology at any cost. For Roma 1.0, the project prioritizes a modern Wayland environment and applications that support it.

Compatibility decisions will be made consciously, based on security, reliability, user needs, maintenance costs, and the project’s long-term direction. Requirements and known limitations must be documented honestly.

## Our Guiding Principle

Use proven open technologies. Move forward deliberately. Build what is missing. Integrate everything into one coherent experience.

AureaOS looks to the future without confusing novelty with reliability. We choose technologies because they are suitable, well-maintained, and aligned with our vision—not simply because they are new, nor because they have always been there.

---

*Technology is beautiful when it is there, but you barely notice it.*