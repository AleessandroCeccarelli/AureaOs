 
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS — User Interface Specification

## 1. Purpose

This document describes the intended desktop experience and user interface direction for AureaOS.

The goal is to create a coherent, modern and approachable desktop environment that feels familiar to people who use Windows or macOS, while maintaining a distinct AureaOS identity.

This specification defines the intended user experience. It does not imply that the described interface or features are already implemented.

The visual system must remain consistent across the desktop shell, system settings, applications and other system surfaces.

## 2. Main Design Direction

AureaOS should be:

- Familiar in workflow and behavior.
- Clean, modern and readable.
- Calm and visually balanced.
- Efficient for everyday work.
- Consistent across system components.
- Accessible to users with different needs.
- Modular and adaptable as the system evolves.

The interface should avoid unnecessary complexity and decoration. Every visual element should have a clear purpose.

Familiar interaction patterns may be used where they improve usability, but AureaOS must not become a visual copy of another operating system.

## 3. Visual Language

AureaOS uses a blue-gray visual foundation with a clear primary blue accent.

The design system is the source of truth for colors, semantic roles and theme values.

Canonical token location:

`aurea-human-interfaces/design-system/tokens/index.css`

The light and dark themes must share the same semantic token structure. Components should use semantic design tokens rather than introducing independent color values.

### 3.1 Light Theme

The light theme should feel bright, calm and comfortable.

Its visual direction includes:

- Light blue-gray backgrounds.
- White or near-white surfaces.
- Clear separation between windows, panels and the desktop.
- Dark, readable text.
- Blue primary actions and interactive states.
- Restrained secondary accents.

The interface should maintain sufficient contrast without relying on excessive borders or shadows.

### 3.2 Dark Theme

The dark theme should remain comfortable during extended use.

Its visual direction includes:

- Deep blue-gray backgrounds.
- Dark slate surfaces.
- Soft, readable foreground text.
- Consistent blue primary accents.
- Carefully balanced contrast.
- Limited use of bright surfaces.

The dark theme should not rely on pure black as its dominant background color.

### 3.3 Accent Colors

Ivory, graphite, gold and other warm tones may be used as secondary accents where appropriate.

They are not the primary palette of AureaOS.

Accent colors must not replace semantic colors or reduce readability.

### 3.4 Transparency and Effects

Transparency, blur and subtle shadows may be used to establish visual hierarchy and depth.

These effects must remain restrained and must never compromise:

- Text readability.
- Interface contrast.
- Performance.
- Accessibility.
- Consistency between themes.

The desktop must remain usable when transparency or visual effects are reduced or disabled.

## 4. Windows and Surfaces

Windows should be clear, consistent and predictable.

The intended visual characteristics include:

- Well-defined surfaces.
- Consistent corner radii.
- Subtle borders.
- Restrained shadows.
- Clear spacing and alignment.
- Readable typography.
- Consistent controls and interaction states.

Window behavior and appearance should be governed by shared Human Interfaces components wherever possible.

Applications should not introduce unrelated window styles without a clear functional reason.

## 5. Desktop Shell and Dock

The intended default desktop layout uses a bottom dock as its main shell element.

The dock should provide access to applications and essential system functions without occupying unnecessary screen space.

### 5.1 Dock Behavior

The planned dock should:

- Be positioned at the bottom of the screen.
- Use a compact and visually balanced layout.
- Support running and pinned applications.
- Provide clear indicators for active applications.
- Remain accessible through pointer and keyboard interaction.
- Support appropriate visibility behavior.

The dock may use a floating or translucent appearance, provided readability and usability remain priorities.

Its final dimensions, positioning and behavior must be validated through implementation and usability testing.

### 5.2 Launcher Entry Point

The AureaOS logo should provide access to the application launcher.

The launcher is intended to provide a central entry point for:

- Installed applications.
- Quaero search.
- System settings.
- Files and folders.
- Recent items.
- System tools.
- Relevant system actions.

### 5.3 System Status Area

The dock or an associated system area should provide access to essential status information, including:

- Time and date.
- Network connectivity.
- Audio status.
- Battery status on supported devices.
- Relevant system notifications.
- Quick system controls.

The final placement and organization of these elements should prioritize clarity and consistency.

## 6. Quick Settings

Quick Settings should provide convenient access to frequently used system controls.

The planned controls include:

- Volume and audio output.
- Display brightness.
- Wi-Fi and network connectivity.
- Bluetooth.
- Light and dark appearance.
- Accessibility options.
- Display configuration.
- Power-related actions.

Controls must reflect the capabilities actually available on the device.

For example, brightness controls should only be exposed when the system can manage the relevant display or backlight.

Quick Settings should use shared interface components and remain consistent with both system themes.

## 7. Notifications

Notifications should communicate relevant information without unnecessarily interrupting the user.

The intended experience includes:

- Clear notification titles and descriptions.
- Readable surfaces and text.
- Consistent application identity.
- Appropriate grouping.
- Clear notification actions where supported.
- Predictable dismissal behavior.
- Accessible keyboard interaction.

Visual effects such as transparency and shadows may be used, but readability must always take priority.

Notifications should integrate with the system notification infrastructure rather than relying on unrelated application-specific implementations.

## 8. Launcher and Quaero Search

The application launcher should provide a straightforward way to access applications and system functions.

Quaero is the planned system search service and should provide a unified search experience.

The intended search areas include:

- Installed applications.
- Files and folders.
- System settings.
- Relevant system actions.
- Other indexed content where supported.

Search results should be organized by relevance and category.

The launcher and Quaero should remain modular components. Their integration must not require applications or other system services to duplicate search functionality.

## 9. Keyboard Workflow

AureaOS should support an efficient keyboard-first workflow without making the desktop intimidating to less experienced users.

The following shortcuts are proposed defaults:

| Shortcut | Intended action |
|---|---|
| Super | Open launcher or desktop overview |
| Alt + Tab | Switch between applications |
| Alt + F4 | Close the active window |
| Ctrl + Alt + T | Open terminal |
| Super + E | Open Fides file manager |
| Super + L | Lock the session |
| Super + Arrow keys | Arrange or snap windows |

These shortcuts are proposed behavior and may be adjusted during implementation.

The system should provide a discoverable way to view and customize keyboard shortcuts.

Keyboard navigation must remain available throughout the desktop experience.

## 10. Accessibility

Accessibility must be considered from the beginning of the design and implementation process.

The interface should support:

- Sufficient text and control contrast.
- Readable typography and scalable text.
- Visible keyboard focus indicators.
- Keyboard navigation.
- Screen-reader compatibility through suitable accessibility interfaces.
- Reduced-motion preferences.
- Alternatives to color-only communication.
- Clear and understandable error messages.

Visual effects and animations should never be required to understand the interface or complete an operation.

Accessibility behavior should be consistent across system components and applications.

## 11. Typography

Typography should prioritize readability, consistency and availability across supported environments.

The final AureaOS font family has not yet been selected.

Aurea Sans is a candidate name only and must not be treated as an approved or implemented font.

The typography system should define:

- Interface font roles.
- Heading and body text hierarchy.
- Control and label sizes.
- Monospaced text for terminal and code-related interfaces.
- Appropriate fallback fonts.

Font selection must consider licensing, language coverage, rendering quality and long-term maintenance.

All applications should use shared typography tokens where possible instead of defining unrelated font systems.

## 12. Motion and Interaction

Animations should communicate state changes and help users understand the interface.

Motion should be:

- Subtle.
- Consistent.
- Responsive.
- Short enough not to interrupt workflows.
- Compatible with reduced-motion preferences.

Transitions should not introduce unnecessary delays or make the desktop feel slower.

Interactive elements must provide clear visual feedback for relevant states, including hover, focus, pressed, selected, disabled and error states.

## 13. Design System Integration

The AureaOS Human Interfaces project is responsible for shared design foundations and reusable interface components.

The design system should provide:

- Semantic color tokens.
- Light and dark theme definitions.
- Typography roles.
- Spacing and sizing conventions.
- Shared component behavior.
- Consistent interaction states.
- Accessibility guidelines.

Canonical design tokens are maintained in:

`aurea-human-interfaces/design-system/tokens/`

The system should remain modular. Applications and shell components should reuse shared foundations rather than creating independent visual systems.

Changes to the design system should be reflected consistently across dependent components.

## 14. Implementation Principles

The user interface should be implemented incrementally.

Development priorities should include:

1. Establish the shared design tokens and theme foundations.
2. Define reusable Human Interfaces components.
3. Implement the basic desktop shell structure.
4. Establish window and application interaction patterns.
5. Integrate launcher and Quaero search.
6. Implement system status and Quick Settings.
7. Integrate notifications and accessibility services.
8. Validate behavior across different displays and hardware configurations.

The implementation should preserve separation between the shell, compositor, system services and applications.

The graphical experience must not depend on undocumented assumptions about hardware or system capabilities.

## 15. Final Design Principle

AureaOS should feel:

- Familiar without being derivative.
- Modern without unnecessary decoration.
- Calm without sacrificing clarity.
- Productive without overwhelming the user.
- Consistent without restricting application identity.
- Accessible without treating accessibility as an optional feature.

The final experience should express a recognizable AureaOS identity through coherent design decisions, reliable behavior and careful attention to everyday usability.
