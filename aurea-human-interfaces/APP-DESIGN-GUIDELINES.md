<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Application Design Guidelines

**Status:** Design specification  
**Scope:** First-party AureaOS applications and third-party applications integrating with the AureaOS desktop  
**Canonical visual foundation:** `design-system/tokens/index.css`

## 1. Purpose

This document defines how applications should look and behave in AureaOS. Its purpose is to make the desktop feel like one carefully designed operating environment rather than a collection of unrelated applications.

The guidelines apply to:

- First-party applications developed for AureaOS.
- System utilities, settings panels, dialogs and other application surfaces.
- Third-party applications that choose to integrate with the AureaOS visual environment.

AureaOS should combine the strongest usability ideas found across modern desktop systems while maintaining its own identity. Familiarity is welcome; copying another operating system's appearance or branding is not.

Consistency does not mean that every application must have identical content or layout. It means that shared elements, terminology, visual rules and interaction patterns behave predictably everywhere.

## 2. Core principles

Every application should be:

- **Recognizably AureaOS:** use the shared visual language rather than imitating another desktop.
- **Consistent:** reuse common components, tokens, menu names and interaction states.
- **Readable:** prioritize text clarity, contrast and sensible information density.
- **Predictable:** controls with the same purpose should look and behave the same way.
- **Quiet and purposeful:** avoid decorative effects that compete with the content.
- **Accessible:** support keyboard navigation, visible focus, scalable text and reduced motion.
- **Efficient:** keep frequent actions easy to discover and reach.
- **Adaptable:** work across supported screen sizes, display scales and light/dark themes.

Application-specific identity belongs in the content, workflows and carefully controlled accent details—not in reinventing shared desktop conventions.

## 3. Shared application window

First-party applications must use the shared AureaOS window framework wherever available. The framework defines the window header, title placement, menu presentation, window controls, spacing, surfaces and interaction states.

### 3.1 Window header

The standard application window header must provide a consistent structure:

- Application identity, using its approved icon and/or name.
- A predictable title area.
- Standard minimize, maximize/restore and close controls.
- Consistent control order, dimensions, hit targets and hover/pressed states.
- A consistent relationship between the header and the application menu.

Window controls must remain in the same position and use the same behavior across first-party applications. Applications must not move, rename or visually reinvent these controls.

The header should remain visually calm. Avoid unnecessary gradients, thick borders, oversized controls or unrelated decorative elements.

### 3.2 Application menu

First-party desktop applications must use the same standard menu names and order:

1. **File**
2. **Edit**
3. **View**
4. **Navigate**
5. **Window**
6. **Help**

The labels are canonical identifiers; their visible text must be localized consistently. For example, the Italian interface uses **File**, **Modifica**, **Visualizza**, **Vai**, **Finestra** and **Aiuto**.

The menu structure must not change from one AureaOS application to another. Applications may provide different commands inside each menu according to their purpose, but must not invent alternative names for these shared categories.

Examples:

- Fides may place new folder, open, copy, move and properties commands in the appropriate standard menus.
- Quaero may expose search navigation and result-view commands.
- Aurea Wallet may expose wallet-specific actions under the relevant standard categories.

A command that is not relevant to an application must not be fabricated merely to fill a menu. Keep the shared menu structure understandable and avoid empty or misleading entries. Application-specific commands may be grouped inside the appropriate standard menu; genuinely contextual actions may also appear in contextual menus.

Menu items must support keyboard navigation, visible focus, clear disabled states, accelerators where appropriate and consistent dismissal behavior.

### 3.3 Context menus

Context menus must use the same typography, surfaces, corner treatment, spacing, icons, separators, focus and hover states as other AureaOS menus.

The available commands may depend on the selected object or context. Their wording must remain consistent with the corresponding commands in the main application menus.

Do not create a different visual menu language for each application.

## 4. Shared visual language

Applications must use the AureaOS design system as their source of truth for visual values.

### 4.1 Color and semantic tokens

Use semantic tokens for:

- Window and page surfaces.
- Elevated surfaces and panels.
- Primary, secondary and muted text.
- Borders and separators.
- Primary and secondary accents.
- Focus, selection, hover and pressed states.
- Success, warning, critical and informational states.
- Shadows and translucent surfaces.

Do not independently hard-code a new palette inside each first-party application. Do not use the Aurea warm/gold accent as a replacement for the primary blue or for semantic status colors.

The canonical token source is:

`design-system/tokens/index.css`

The color-system documentation is:

`../docs/human-interfaces/COLOR-SYSTEM.md`

If a required semantic token does not exist, propose it for the shared design system instead of silently introducing a local substitute.

### 4.2 Light, dark and automatic appearance

All first-party applications must support the system appearance setting:

- **Light**
- **Dark**
- **Automatic / Follow system**

The system appearance service is authoritative. When the user changes the system theme, open first-party applications must update their surfaces, text, borders, icons, controls, menus, dialogs and notifications coherently.

Applications must not maintain independent theme choices that silently contradict the system setting. An application-specific override may only be offered if the platform later defines a consistent, explicit override policy; the default must always follow the system.

Light and dark themes must use the same semantic token roles. Do not simply invert colors. Each theme must be designed and checked for its own contrast, hierarchy and readability.

### 4.3 Surfaces, transparency and blur

Use clean, well-defined surfaces with restrained borders and shadows. Translucency and blur are appropriate for floating system surfaces, menus, panels and selected overlays when they improve hierarchy.

Application content areas should prioritize legibility and stable contrast. Do not apply glass effects to every surface. Text and controls must remain readable over any background.

Visual effects must degrade gracefully when transparency or blur is reduced or unavailable.

### 4.4 Shape, spacing and density

Use shared design tokens for corner radii, spacing, control sizes and elevation. Related controls should align consistently and have predictable hit targets.

Avoid mixing unrelated corner styles, arbitrary spacing scales or inconsistent control heights within one application.

Applications may use denser layouts when their purpose requires it, but must preserve the shared spacing logic, typography hierarchy and interaction states.

### 4.5 Typography

Use the shared AureaOS typography roles and approved font configuration when available. Do not bundle or impose an unrelated interface font in first-party applications.

Typography must establish a clear hierarchy between window titles, section headings, body text, labels, metadata and monospaced content. Text must remain legible at supported display scales.

The final system font choice remains subject to the separate font approval process; applications must not treat an unapproved candidate as the definitive AureaOS font.

### 4.6 Icons and symbols

Use the AureaOS icon and symbol libraries for shared actions and system concepts. Icons representing the same action must have the same meaning and visual treatment across applications.

Application icons may have their own recognizable identity, but must follow the AureaOS icon and brand guidance. Do not copy Windows, ChromeOS, macOS or third-party application icon artwork.

Icons must not be the only way to communicate an important action: use labels, accessible names or tooltips where needed.

## 5. Interaction and motion

Shared interactions must behave consistently across first-party applications.

### 5.1 Control states

Buttons, fields, tabs, lists, menus and other controls must provide consistent states:

- Default.
- Hover.
- Focus.
- Pressed.
- Selected or active.
- Disabled.
- Loading, where applicable.
- Error or validation state, where applicable.

Do not rely on color alone to communicate a state.

### 5.2 Animation

Animations should be subtle, brief and purposeful. Use motion to explain a change of state, establish spatial relationships or help users understand where a panel or window came from.

Avoid exaggerated bounce, unnecessary overshoot, long transitions and motion that delays the user's task.

All first-party applications must respect the system reduced-motion preference. When reduced motion is enabled, replace nonessential movement with immediate state changes or minimal fades.

### 5.3 Keyboard and pointer

All essential actions must be available through keyboard navigation where practical. Focus order must be logical and visible. Pointer interactions must have equivalent accessible routes wherever possible.

Use established AureaOS keyboard conventions and the shared shortcut policy. Applications must not intercept global shortcuts that belong to the desktop shell.

## 6. Launcher and desktop integration

The application launcher is a desktop-level component, not an application-specific imitation.

The launcher opens when the user:

- Selects the Aurea **A** mark on the dock.
- Presses the **Super / Windows** key.

Opening uses a subtle, short animation. Pressing Super again toggles the launcher closed; Escape dismisses it. The launcher must be keyboard-ready, with search immediately available, and must respect reduced-motion settings.

Applications should integrate with the launcher through standard application metadata, including a clear display name, approved icon, categories and launch entry. They must not create their own competing global launcher.

Applications should integrate with dock behavior through standard desktop application identity and window association. Running and pinned states must be represented consistently by the shell.

## 7. Dialogs, notifications and system panels

First-party applications should use shared AureaOS components for dialogs, confirmation prompts, file selection, notifications and other common surfaces whenever available.

- Dialogs must have clear titles, concise explanations and explicit actions.
- Destructive actions must be distinguishable and should not be styled as ordinary primary actions.
- Default and cancel actions must be predictable.
- Notifications must respect system notification policy and must not imitate system alerts with unrelated custom designs.
- Applications must not create fake system panels, fake security prompts or misleading operating-system notifications.

System-owned panels and controls remain under the control of AureaOS shell and services.

## 8. Third-party application integration

AureaOS must remain compatible with third-party applications, including applications distributed through Flatpak. Visual integration must not break application functionality, sandbox boundaries or upstream compatibility.

Third-party applications should be encouraged to adopt AureaOS styling through documented toolkit integration, shared theme support, icon conventions, portals and platform APIs where available.

### 8.1 Integration levels

Use the following practical levels:

**Level 1 — Native AureaOS integration**

The application uses the shared AureaOS design system and window components. Its menus, controls, typography, themes and interaction patterns follow this document.

**Level 2 — Theme-aware integration**

The application uses its existing supported toolkit but follows the system light/dark preference, shared color scheme where supported, icon conventions, accessibility settings and desktop integration.

**Level 3 — Compatible third-party application**

The application retains its upstream interface because it cannot reasonably adopt the AureaOS components. AureaOS should preserve reliable window behavior, launching, keyboard operation, scaling and accessibility integration wherever technically possible.

A third-party application must never be modified in a way that breaks its functionality or security merely to make it look native.

### 8.2 What AureaOS can and cannot require

AureaOS controls the desktop shell, system panels, supported native APIs and its own first-party applications. It cannot guarantee that every independently developed third-party application will use identical menus, title bars, widgets or visual effects.

Where an application does not support the AureaOS design system, the desktop should provide the best compatible integration available without pretending that the application is fully native.

Flatpak sandboxing and application permissions must remain intact. Visual integration must not bypass portals or grant additional privileges.

## 9. Accessibility and internationalization

First-party applications must support:

- Keyboard navigation and visible focus.
- Accessible names and roles for controls.
- Scalable text and supported display scaling.
- Sufficient contrast in both themes.
- Reduced motion and reduced transparency where supported.
- Clear error messages and validation feedback.
- Localization of all user-visible strings.
- Correct text direction and language-sensitive formatting where applicable.

Shared menu identifiers and command meanings must remain stable across translations. Localization changes visible labels, not the underlying menu structure or meaning.

## 10. Conformance checklist

A first-party application is not ready for design-system conformance review until the following can be verified:

- [ ] Uses the shared AureaOS window structure and standard window controls.
- [ ] Uses the standard menu categories and their canonical order.
- [ ] Uses shared semantic color, spacing, typography, shape and elevation tokens.
- [ ] Follows system Light, Dark and Automatic appearance.
- [ ] Updates its appearance when the system theme changes.
- [ ] Uses consistent menus, dialogs, controls and interaction states.
- [ ] Respects reduced-motion and accessibility settings.
- [ ] Uses approved AureaOS icons and symbols for shared actions.
- [ ] Does not duplicate desktop-level launcher, dock or system-panel functions.
- [ ] Preserves keyboard conventions and does not steal global shortcuts.
- [ ] Remains readable at supported display scales.
- [ ] Does not introduce unrelated visual styles without an approved functional reason.

Third-party applications should be assessed against the integration levels above rather than being rejected solely because they cannot implement native AureaOS components.

## 11. Governance and evolution

This document defines shared design expectations; it does not claim that every component or service described here is already implemented.

Changes to shared visual rules should be made in the design system and documented centrally. First-party applications should consume those shared definitions rather than creating local forks.

When a new shared control, menu category, token or interaction pattern is needed:

1. Identify the user need.
2. Check whether an existing AureaOS pattern already solves it.
3. Define the behavior and visual treatment in the shared design system.
4. Review accessibility and both themes.
5. Update affected applications consistently.

The goal is not to prevent applications from having personality. The goal is to make every application feel as though it belongs to the same coherent operating system.

---

**AureaOS design principle:** one system, one visual language, one predictable experience—while preserving the freedom and compatibility of third-party software.
