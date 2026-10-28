
# AureaOS Human Interfaces — Shared Components

## 1. Purpose

This document defines the expected appearance, behavior, and accessibility of shared AureaOS interface components.

The objective is to provide a coherent native desktop experience across AureaOS applications, system settings, and desktop services.

Common components should be implemented in the shared AureaOS Human Interfaces library wherever practical. Applications should not independently recreate standard interface behavior without a justified reason.

## 2. Shared Component Contract

Every shared component should define:

- Its purpose and expected behavior.
- Its visual states and layout rules.
- Mouse, keyboard, trackpad, and touch interactions where applicable.
- Focus and accessibility behavior.
- Light and dark theme appearance.
- Animation and reduced-motion behavior.
- Error, disabled, and unavailable states where relevant.

Applications may provide context-specific content and actions while retaining the shared component's appearance and interaction rules.

## 3. Context Menus

Context menus are a core part of the AureaOS interaction model.

All native AureaOS applications must use the shared context-menu component unless a documented technical or usability requirement justifies an exception.

### 3.1 Visual Consistency

Context menus must share:

- Typography and text hierarchy.
- Padding, spacing, and row heights.
- Corner radii, borders, shadows, and surface treatment.
- Light and dark theme behavior.
- Hover, focus, selected, disabled, and pressed states.
- Animation timing and opening and closing behavior.
- Alignment of labels, icons, shortcuts, and submenu indicators.

Menus may contain different commands, but their visual structure must remain recognizable across applications.

### 3.2 Content and Organization

Context menus should:

- Show actions relevant to the selected object or current context.
- Group related commands logically.
- Use separators only when they improve understanding.
- Display keyboard shortcuts consistently.
- Clearly distinguish unavailable actions.
- Avoid excessive menu depth and unnecessary commands.

Destructive actions should be clearly identifiable and use confirmation when the consequences justify it.

### 3.3 Interaction

Context menus should support:

- Secondary mouse click.
- The keyboard context-menu key where available.
- Appropriate keyboard shortcuts and navigation.
- Touch alternatives where relevant.
- Predictable submenu navigation.
- Dismissal through Escape or an appropriate outside interaction.

Keyboard focus must remain visible. Menus must not depend exclusively on pointer hover.

## 4. Windows, Dialogs, and Controls

### 4.1 Windows

Native application windows should follow common rules for:

- Title bars and window controls.
- Activation and focus.
- Moving, resizing, minimizing, maximizing, restoring, and closing.
- Window borders and resize hit areas.
- Placement and behavior across virtual desktops and displays.

Applications may use specialized layouts, but basic window behavior must remain consistent.

### 4.2 Dialogs

Dialogs should use shared layouts and interaction patterns.

They must clearly identify their purpose, present the relevant information, and distinguish primary from secondary actions.

Keyboard navigation, Escape behavior, and confirmation requirements must be predictable.

### 4.3 Buttons and Input Fields

Buttons and fields should share dimensions, typography, spacing, borders, and interaction states.

The design system must define hover, focus, pressed, disabled, invalid, and loading states where applicable.

Essential actions must remain discoverable and accessible without relying on color alone.

## 5. Input Devices

AureaOS must support a coherent interaction model across different input devices.

### 5.1 Mouse

The desktop should provide precise pointer interaction, secondary-click menus, clear hover feedback, and predictable drag-and-drop behavior where supported.

Hover effects must not be the only way to discover or activate essential functionality.

### 5.2 Keyboard

Core desktop and application operations should be accessible through the keyboard.

The system should provide consistent focus navigation, documented shortcuts, window switching, menu navigation, and visible focus indicators.

Conflicting shortcuts should be avoided or handled through a clear priority model.

### 5.3 Laptop Trackpads

Trackpads should support natural scrolling, configurable pointer speed and sensitivity, and secondary-click behavior.

Common gestures may include switching virtual desktops and showing open windows, subject to hardware capabilities and user preferences.

Gestures must be discoverable, consistent, and configurable where practical. They must not be the only way to access essential functions.

### 5.4 Touch and Pen

Where supported, touch interaction should provide appropriately sized targets and clear feedback.

Compatible graphics tablets and pens should expose available capabilities such as pressure, tilt, and programmable buttons through the relevant Linux input stack and drivers.

Capabilities must not be advertised when the hardware or driver does not provide them.

## 6. Window Management and Multiple Displays

AureaOS should support multiple displays with different resolutions, scaling factors, and refresh rates where supported by the hardware and graphics stack.

Users should be able to select a primary display and configure display arrangement and scaling through graphical settings.

Windows should be placed and restored predictably when displays are connected, disconnected, or rearranged.

The system must avoid leaving windows inaccessible on disconnected displays.

The floating desktop bar, launcher, notifications, and quick settings must appear on appropriate displays according to documented placement rules.

Virtual desktops should provide simple, predictable transitions. Rotating cubes, burning windows, and other decorative desktop effects are outside the intended design direction.

## 7. Motion and Animation

AureaOS uses restrained, consistent animation to communicate changes in state and spatial relationships.

### 7.1 Approved Patterns

Appropriate patterns include:

- Short fades for notifications and transient panels.
- Smooth opening and closing transitions for menus and dialogs.
- Subtle movement when minimizing and restoring windows.
- Brief transitions for the launcher and quick settings.
- Discreet hover and focus feedback.
- Controlled transitions between light and dark themes.

Window minimization may use a subtle deformation toward the relevant bar icon, inspired by the sense of continuity in macOS Genie. The implementation must remain recognizably AureaOS and avoid reproducing another system's exact visual identity.

### 7.2 Timing and Performance

Simple interactions should generally use short transitions, often around 120–200 milliseconds as an initial design target.

More complex transitions may take longer when needed for clarity.

These values are starting guidelines, not rigid requirements. Perceived responsiveness and usability take priority over matching a duration exactly.

Animations must not delay essential actions, block interaction unnecessarily, or require continuous decorative rendering.

### 7.3 Accessibility

The system should respect reduced-motion preferences and provide simpler transitions when appropriate.

Essential state changes must remain understandable even when animations are reduced or disabled.

## 8. Floating Desktop Bar

The desktop bar should float above the bottom edge with visible spacing around it.

It should use rounded corners, restrained translucency, and subtle background blur.

The AureaOS A mark should be larger than ordinary application shortcuts and visually distinct from rounded-square application icons. A circular highlight may be used for interaction feedback without permanently enclosing the mark.

Browser, email, Fides, and Aurea Store shortcuts should remain compact.

Running applications should have a subtle running indicator, with a distinct state for the active application.

### 8.1 Window Previews

Hovering over a running application should reveal a small preview of its open window or windows.

Previews should show a recognizable thumbnail and application name. Selecting a preview should activate the corresponding window.

A short hover delay should prevent previews from appearing continuously during rapid pointer movement.

The preview should remain available while the pointer moves from the icon to the preview itself.

Equivalent access should be provided through appropriate keyboard navigation and touch interactions where supported.

## 9. Launcher and Quick Settings

The launcher should provide a prominent search field, favorite applications, and a clear application grid.

Its layout should remain spacious and readable in both themes.

Quick settings should provide graphical access to common controls such as network connectivity, Bluetooth, brightness, volume, and Do Not Disturb where applicable.

Both components must use shared spacing, typography, icons, surfaces, focus states, and animation rules.

## 10. Notifications

Notifications should use softly rounded, translucent panels with controlled background blur.

The notification source, main message, supporting information, and available actions must have a clear hierarchy.

Contrast must remain sufficient over different wallpapers and background content.

Notifications should not obscure essential interaction or become unnecessarily intrusive.

## 11. Printers and Scanners

Printer and scanner management should be integrated into the graphical system settings.

AureaOS should reuse mature Linux printing and scanning infrastructure, including CUPS and SANE where appropriate.

The shared interface should provide consistent device lists, status indicators, configuration forms, error messages, and progress feedback.

Device-specific settings must be presented only when supported by the installed driver or device.

Detailed product requirements are documented in `docs/product/PLANNED-SYSTEM-TOOLS.md`.

## 12. System Buttons and Status Indicators

System controls should share consistent visual states, spacing, accessibility behavior, and feedback.

Network, audio, battery, clock, notification, and related indicators should be presented in a predictable location.

Unavailable capabilities should be hidden or clearly marked as unavailable rather than shown as functioning controls.

## 13. Accessibility and Feedback

Shared components should support:

- Visible keyboard focus.
- Clear labels and accessible names.
- Adequate contrast.
- Scalable text and interface elements.
- Screen-reader semantics.
- Predictable keyboard navigation.
- Appropriate feedback for success, failure, and ongoing operations.
- Reduced-motion preferences.

Color, animation, and pointer hover must not be the only means of communicating essential information.

## 14. Third-Party Applications

AureaOS should provide a consistent experience across its own native applications and integrate third-party applications as effectively as practical.

Third-party applications may use different toolkits, design systems, or platform conventions. Complete visual uniformity cannot be guaranteed without control over their implementation.

Where practical and technically appropriate, AureaOS should use supported integration mechanisms, shared system services, and compatible theming rather than fragile modifications.

Security, application isolation, accessibility, and upstream maintainability must take priority over forcing visual uniformity.

## 15. Implementation Status

This document describes intended component behavior and design requirements.

The shared component library and the described integrations are planned work unless explicitly implemented and verified.

Components should be introduced incrementally, with tests for visual states, keyboard behavior, accessibility, and interaction consistency.

## 16. Guiding Principle

**One coherent interaction language, adapted to different applications and devices.**

AureaOS should feel familiar without copying another operating system, remain accessible without requiring a specific input device, and use effects only when they improve clarity or feedback.
