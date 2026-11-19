<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Design System

The canonical specification for AureaOS visual tokens, shared component behavior, recurring interface patterns and keyboard conventions. The named visual foundations are **Aurea Palette** (semantic color system) and **Aurea Form** (geometry and surface language), under AureaOS Human Interfaces. It turns the product direction into consistent rules for the desktop shell and first-party applications.

## Authority and boundaries

- `tokens/index.css` is the sole source of CSS design values, including light and dark themes.
- `../themes/` contains compatibility wrappers only; it must not define a competing palette.
- [Desktop Experience](../DESKTOP-EXPERIENCE-SPEC.md) owns the overall desktop and shell experience.
- [Application Design Guidelines](../APP-DESIGN-GUIDELINES.md) owns shared application conventions and menu structure.
- Component specifications define reusable control behavior and states.
- Pattern specifications define how controls and surfaces work together.
- [Keyboard and Command Conventions](KEYBOARD-AND-COMMAND-CONVENTIONS.md) owns shared shortcuts and command consistency.

Collection-specific READMEs own asset provenance, format, licensing and packaging details. Do not duplicate those inventories here.

## Structure

- `tokens/index.css` — sole canonical CSS token source.
- `components/` — component anatomy, behavior, states, accessibility and theme contracts.
- `patterns/` — recurring compositions and workflows.
- `KEYBOARD-AND-COMMAND-CONVENTIONS.md` — shortcut defaults, command placement and ownership.

## Product principles

1. Familiar desktop behavior takes priority over novelty.
2. Windows is the primary usability reference; KDE Plasma and macOS inform selected desktop and refinement decisions.
3. AureaOS owns its visual identity and implementation; references are not templates to copy.
4. Use semantic tokens, shared components and stable terminology.
5. Keep the same command in the same menu and preserve familiar shortcut behavior.
6. Readability and accessibility take priority over transparency, blur and decoration.
7. Light and dark themes must preserve the same hierarchy and interaction model.
8. Motion, transparency and scaling preferences must be respected.

## Current specification coverage

### Tokens

The canonical token set covers desktop/window surfaces, panels, text, borders, accent and status colors, focus, dock, notifications, glass materials, shadows, radii and spacing. Propose missing shared values in `tokens/index.css` before introducing component-local substitutes.

### Components

The current component contracts cover:
- Buttons.
- Text fields.
- Menus.
- Dialogs.
- Toggles and selection controls.
- Window chrome.

See [Component Specifications](components/README.md).

### Patterns

The current pattern set covers:
- Desktop shell.
- Application navigation.
- Feedback and notifications.
- Forms and settings.

See [Patterns](patterns/README.md).

### Desktop and applications

The desktop specification defines the launcher, floating dock, workspaces, window behavior, overview, Quick Settings, notifications and Fides integration. Application guidelines define standard menus, context menus, dialogs, shared visual behavior and third-party compatibility.

## Implementation status

These documents define design contracts. They do not prove that native Rust components, a theme service, an asset loader, compositor behavior or application integration are implemented.

Native implementation belongs in the appropriate AureaOS component. Shared behavior should be implemented once and consumed across applications rather than copied into each app. Keep modules focused and avoid introducing a new toolkit or dependency without a demonstrated requirement.

## Change process

1. Identify a real shell or application need.
2. Check the existing token, component and pattern contracts.
3. Update the canonical specification at the correct level.
4. Update examples and previews only when they help review the change.
5. Record implementation status separately from design approval.
6. Check light/dark behavior, keyboard access, focus, reduced motion, reduced transparency and scalable text.

Do not add speculative components, duplicate tokens or decorative effects without a clear user benefit.
