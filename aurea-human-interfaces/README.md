# Aurea Human Interfaces

This directory is the visual foundation for AureaOS.

AureaOS should feel familiar to users coming from Windows or macOS, but it must keep its own identity. The goal is not to copy another desktop, but to create a coherent environment that feels natural to people already used to modern operating systems: clear navigation, predictable controls, keyboard shortcuts, quick access, and strong discoverability.

This layer is responsible for the desktop language of the system: visual identity, theme tokens, reusable components, shared assets, and interaction patterns.

## Design goals

- Familiar without being derivative
- Calm, readable, and efficient
- Strong hierarchy without visual noise
- Consistent across all first-party applications and system surfaces
- Clear and discoverable for users coming from desktop ecosystems they already know
- Distinct enough to feel like AureaOS, not a Windows or macOS clone

## Separation of concerns

This area is intentionally separate from the runtime and system foundations:

- `design-system/` defines the visual language and reusable patterns
- `assets/` contains wallpaper, illustrations, textures, and derived visual resources
- `brand/` contains product identity and presentation material
- `components/` contains reusable interface primitives
- `fonts/` contains type and source assets
- `icons/` and `symbols/` contain graphical symbols used across the OS
- `prototypes/` contains experiments and concept validation
- `previews/` contains visual review samples and screenshots

## Interaction intent

AureaOS should provide:

- predictable app launch paths
- strong desktop and task management
- familiar keyboard shortcuts for fast power users
- no confusing hidden logic or overloaded UI states
- a clear visual sign that the system is native to AureaOS

This is not a generic Linux theme. It is a designed desktop experience with a clear user model.

## Future direction

This directory should evolve into a proper design system package for the entire OS, with reusable patterns for:

- desktop shell
- settings and system panels
- file manager
- installers and recovery flows
- application windows and controls
- notifications and dialogs
