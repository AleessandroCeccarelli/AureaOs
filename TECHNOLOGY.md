# AureaOS Technology Principles

## Looking Forward, Building on Proven Foundations

AureaOS is built for the future. Its technology choices reflect a clear principle: select modern, mature, well-maintained components that deliver the best overall result, then integrate and shape them into one coherent operating system experience. Open-source and proprietary components are both eligible when their technical merits and distribution terms justify their use.

We do not reinvent infrastructure simply for the sake of doing so. We build on the strengths of the Linux ecosystem and focus our own development on the parts that make AureaOS distinctive.

## Modern by Design

AureaOS Roma 1.0 is designed around a native Wayland desktop. X11 and XWayland are not part of the intended desktop architecture. Applications selected for the default experience must be compatible with this direction; any exceptions must be deliberate, documented, and justified rather than included merely for legacy compatibility.

Our goal is not to preserve every historical workflow indefinitely. We want a desktop designed around modern display, input, security, and application technologies.

## Technology Selection, Without Dogma

AureaOS evaluates components according to the needs of the system and the people using it. Open-source status is not an absolute requirement, and proprietary software or firmware may be included when it materially improves compatibility, reliability, performance, usability, or hardware support and can legally be distributed.

Each candidate should be assessed for:

- **Compatibility and reliability**, including behavior on supported hardware.
- **Security and maintenance**, including update support and response to vulnerabilities.
- **Performance and resource use**, especially on everyday and lower-powered computers.
- **Usability and integration**, including whether the component supports a coherent graphical experience.
- **Licensing and distribution terms**, including attribution, redistribution, patent, trademark, and other obligations where applicable.
- **Long-term sustainability**, including documentation, upstream health, and the availability of alternatives.

Open-source components are valuable because they can offer transparency and flexibility. Proprietary components can also be the practical choice in some cases. Neither label replaces a technical and legal evaluation. Third-party components retain their own licenses; the GPL-3.0 license for original AureaOS code does not automatically apply to them.

## Build What Makes AureaOS Different

AureaOS will reuse reliable Linux infrastructure for essential system capabilities instead of duplicating work that the ecosystem has already solved well.

Our own development will focus on the integrated desktop experience and the components that define AureaOS: its interface, window management, system settings, search, file management, application store, update experience, backup, and recovery tools.

We will select technologies according to the needs of each component. Native applications are appropriate where close system integration, efficiency, or performance matters. Web technologies, including Electron where suitable, can help us build consistent applications and web-based tools. No single implementation technology is mandatory for every application. Every choice must serve the overall experience.

## One Coherent System

AureaOS should not feel like a collection of unrelated projects assembled together. Even when its underlying components come from different projects, vendors, and licensing models, they should work together as a unified product.

That means consistent interaction patterns, visual design, accessibility, system integration, and clear user-facing behavior. Complexity should remain behind the scenes whenever possible.

## Compatibility Is a Deliberate Choice

AureaOS will not pursue compatibility with every legacy technology at any cost. For Roma 1.0, the project prioritizes a modern Wayland environment and applications that support it.

Compatibility decisions will be made consciously, based on security, reliability, user needs, maintenance costs, and the project’s long-term direction. Requirements and known limitations must be documented honestly.

## Our Guiding Principle

Choose the best-suited technologies. Move forward deliberately. Build what is missing. Integrate everything into one coherent experience.

AureaOS looks to the future without confusing novelty with reliability. We choose technologies because they are suitable, well-maintained, and aligned with our vision—not simply because they are new, nor because they have always been there.

---

*Technology is beautiful when it is there, but you barely notice it.*