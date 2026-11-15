<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Application Design Guidelines

**Status:** Design specification  
**Scope:** First-party AureaOS applications and compatible third-party applications  
**Canonical visual foundation:** `design-system/tokens/index.css`  
**Keyboard conventions:** `design-system/KEYBOARD-AND-COMMAND-CONVENTIONS.md`

## 1. Purpose

AureaOS applications must feel like parts of one operating system and remain immediately understandable to people familiar with Windows desktop software.

Windows is the primary usability reference for ordinary desktop conventions: menu organization, file operations, context menus, window controls, keyboard shortcuts and common terminology. KDE Plasma informs the floating dock and desktop flexibility. macOS informs visual consistency, careful finishing and notification quality.

These references guide usability, not copying. AureaOS owns its artwork, branding, colors and implementation. It must not move familiar controls or rename standard commands merely to appear different.

## 2. Core rules

- Preserve established desktop mental models wherever practical.
- Keep the same command in the same menu and use the same wording across applications.
- Use familiar keyboard shortcuts by default.
- Use shared components and semantic tokens.
- Keep window controls and title-bar behavior consistent.
- Prioritize readability over transparency, blur or decoration.
- Support keyboard, pointer, accessibility and both system themes.
- Do not imitate system-level UI inside ordinary applications.

## 3. Window structure

First-party windows use a shared structure with application identity, predictable title area, standard minimize/maximize-restore/close controls and consistent hit targets.

Window controls must not be independently relocated, renamed or redesigned by each application. Maximize/restore uses one predictable control and state. Closing must respect unsaved work.

The native window framework and compositor own window behavior; application content must not imitate system decorations.

## 4. Standard application menus

First-party desktop applications use these shared menu categories in this order:

1. **File**
2. **Edit**
3. **View**
4. **Navigate**
5. **Window**
6. **Help**

Italian labels are **File**, **Modifica**, **Visualizza**, **Vai**, **Finestra** and **Aiuto**.

Applications may omit a category only when it has no meaningful commands. They must not rename categories, reorder them or invent synonyms. Do not add empty menus just to fill the row.

Commands belong in the same category across applications whenever their meaning is shared. For example, copy, cut, paste, select all and undo remain under Edit. View contains layout, display and visibility choices. Window contains window-level actions. Help contains documentation, support and version information.

Fides may add file-specific commands under File; Quaero may add search-specific commands under Navigate or View; Aurea Wallet may add wallet-specific commands under the relevant standard category. Contextual commands may also appear in right-click menus.

## 5. Context menus

Right-click opens a context menu for the selected object or current surface. Menus appear near the pointer when possible, stay within screen bounds and use the same typography, icons, separators, shadows and interaction states throughout AureaOS.

Use familiar command wording. Do not create application-specific visual menu languages. Destructive actions must be clearly distinguished and must not be placed beside routine actions without separation.

## 6. Shared visual language

Use semantic tokens from `design-system/tokens/index.css` for surfaces, text, borders, accent, focus, status, shadows, radii and spacing.

Light, Dark and Automatic appearance follow the system setting. First-party applications update together when the system appearance changes. Both themes must preserve the same hierarchy and interaction model.

Use transparency and blur selectively for floating surfaces, menus and panels. Application content areas prioritize stable contrast. Effects must degrade gracefully when reduced or unavailable.

Typography and icons use approved AureaOS resources. Application icons may have distinct identities but must follow the shared icon guidance. Do not reuse another operating system's icon artwork.

## 7. Interaction and motion

Buttons, fields, lists, menus and tabs must define default, hover, focus, pressed, selected, disabled and applicable loading/error states. Do not rely on color alone.

Animations are brief and purposeful. All first-party applications respect reduced motion. Essential operations remain available by keyboard and pointer, with visible focus and logical focus order.

Use the shared shortcut policy. Applications must not intercept global shell shortcuts.

## 8. Launcher and desktop integration

The launcher is owned by the desktop shell. It opens from the Aurea A dock mark or Super key and provides immediate search, pinned applications and a discoverable full application list.

Applications integrate through standard desktop metadata, approved icons and stable application identity. They must not create competing global launchers or docks.

## 9. Dialogs and notifications

Use shared dialog and notification components where available. Dialogs have clear titles, concise explanations and explicit actions. Default and cancel actions must be predictable. Destructive actions are clearly distinguished.

Notifications follow system policy and are presented by the shell. Applications must not create fake system alerts, security prompts or system panels.

## 10. Third-party compatibility

AureaOS must preserve compatibility with Flatpak and other supported third-party applications. Integration levels are:

- **Native:** application uses AureaOS components and APIs.
- **Theme-aware:** application retains its toolkit while following supported system theme, icons and accessibility settings.
- **Compatible:** upstream appearance remains, while window behavior, launching, keyboard use and scaling work correctly.

Never break application functionality, sandboxing or permissions to force visual uniformity.

## 11. Accessibility and conformance

First-party applications must provide keyboard navigation, visible focus, accessible names and roles, scalable text, adequate contrast, reduced-motion support, localized strings and predictable command meanings.

Before approval, verify:
- [ ] Standard menu names and ordering.
- [ ] Familiar command placement and wording.
- [ ] Standard window controls.
- [ ] Shared tokens and both themes.
- [ ] Keyboard shortcuts and no global shortcut conflicts.
- [ ] Context menus and dialogs follow shared behavior.
- [ ] Reduced motion and accessibility behavior.
- [ ] No duplicate launcher, dock or fake system surfaces.

**Principle:** applications may express their purpose through content and workflow, but shared desktop conventions remain stable across AureaOS.
