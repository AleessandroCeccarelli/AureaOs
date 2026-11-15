<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Desktop Experience Specification

**Status:** Design specification  
**Scope:** Desktop shell, launcher, dock, workspaces, window organization, notifications and quick settings  
**Visual source of truth:** `design-system/tokens/index.css`  
**Interaction conventions:** `design-system/KEYBOARD-AND-COMMAND-CONVENTIONS.md`  
**Application rules:** `APP-DESIGN-GUIDELINES.md`

## 1. Product direction

AureaOS must feel immediately familiar to people who use a modern Windows desktop. Familiarity is a usability requirement, not a lack of originality.

The desktop adopts:
- **Windows as the primary reference:** taskbar/launcher expectations, file-manager mental model, standard menus, window controls, keyboard shortcuts, context menus and everyday workflows.
- **KDE Plasma as a secondary reference:** a floating bottom dock, adaptable panel behavior and practical workspace organization.
- **macOS as a refinement reference:** consistency between applications, careful visual details, coherent surfaces, restrained motion and polished notification behavior.

AureaOS must not reproduce another product's branding, artwork or exact interface. It must preserve established user expectations while integrating them into one coherent visual system.

**First-login test:** a Windows user should be able to open the application launcher, find files, use the right-click menu, minimize/maximize/close a window, switch applications and use common shortcuts without having to learn a new desktop model.

This specification describes the intended experience. It does not claim every behavior is implemented.

## 2. Desktop composition

The default desktop contains:
- An approved AureaOS wallpaper.
- A floating bottom dock with familiar taskbar behavior.
- A launcher opened from the Aurea A mark or Super key.
- Four virtual workspaces, presented without obscuring ordinary task switching.
- Application windows with predictable title bars and standard controls.
- System notifications and quick settings.
- Familiar access to files, applications, settings and power/session actions.

The desktop must remain uncluttered. Do not introduce widgets or permanent panels merely for decoration.

## 3. Dock and taskbar behavior

The dock is AureaOS's refined interpretation of a familiar desktop taskbar.

### 3.1 Structure

The dock spans the usable screen width and has three stable zones:

**Left — system entry**
- Aurea A launcher button.
- Workspace entry or indicator, without displacing the launcher.

**Center — applications**
- Pinned applications.
- Running applications.
- Clear active-window indicators.
- Window previews where supported.
- Predictable pin/unpin and application grouping behavior.

**Right — system status**
- Clock and date.
- Network and audio.
- Battery and power status when available.
- Notification entry point.
- Quick Settings entry point.

Users must understand the difference between pinned applications, running applications and minimized windows at a glance.

### 3.2 Appearance and placement

The dock floats slightly above the bottom edge by default, with a restrained translucent surface, subtle border and soft shadow. It must remain legible over varied wallpapers in both themes.

Its visual behavior may take inspiration from KDE Plasma, but its taskbar role and application interactions remain familiar to Windows users.

### 3.3 Visibility

Provide predictable modes:
- Intelligent: floating while useful, adapting when windows occupy the available space.
- Always visible.
- Auto-hide.

Do not change dock position, grouping or interaction unexpectedly. Dock animations must be brief and must not delay access.

## 4. Application launcher

### 4.1 Entry and dismissal

- Super / Windows key opens the launcher.
- Selecting the Aurea A mark opens it.
- Pressing Super again or Escape closes it.
- Clicking outside dismisses it when safe.
- Search receives focus immediately.
- Selecting an application launches it and closes the launcher.

The launcher should be recognizable as an application menu, not an unfamiliar full-screen control center.

### 4.2 Information architecture

Use a familiar hierarchy:
1. Search at the top.
2. Pinned/favorite applications.
3. Recommended or recent items, if enabled.
4. Full application list, with clear categories or alphabetical browsing.
5. User/session and power actions in a predictable, clearly separated area.

Search and pinned applications are primary. Recommendations must not dominate or obscure the installed application list. Users should be able to find an application without knowing its category.

### 4.3 Search

The launcher integrates with Quaero; it must not create a competing search engine.

Search may include installed applications, files and folders, settings and supported system actions. Results must be labeled, keyboard navigable, permission-aware and clear about loading, empty, unavailable or restricted states.

## 5. Window behavior

### 5.1 Standard controls

Every first-party window provides recognizable minimize, maximize/restore and close controls in a consistent location and order. Their symbols, hit targets, hover states and behavior must remain consistent.

- Minimize sends the window to the dock/taskbar's running-app area.
- Maximize fills the available work area without covering reserved system surfaces.
- Restore returns to the previous window geometry.
- Close closes the window and respects unsaved work.

Do not replace standard controls with unfamiliar icons or hide them behind a custom interaction.

### 5.2 Title bar and menus

Window title bars clearly identify the application and current document or view where relevant. The application menu uses the shared menu categories and ordering defined in `APP-DESIGN-GUIDELINES.md`.

Context menus use familiar right-click behavior, standard terminology and consistent placement near the pointer, while remaining inside screen bounds.

### 5.3 Snap and tiling

Provide discoverable snap layouts from the maximize control and predictable keyboard operations. Initial layouts should cover left/right halves and four quadrants. Additional arrangements are allowed only when they remain easy to understand.

Show a clear preview before placement. Pointer and keyboard workflows must offer equivalent access. Do not claim a snap feature is implemented until compositor support exists.

### 5.4 Task switching and overview

Alt+Tab must provide familiar switching between open windows. The visual switcher should clearly identify applications and windows, preserve predictable ordering and support keyboard navigation.

A separate overview may expose workspaces and all windows, but must not replace or break the familiar Alt+Tab workflow.

## 6. File access and Fides integration

The desktop must provide familiar entry points to Desktop, Documents, Downloads, Music, Pictures and Videos. Fides is the default file manager and must preserve recognizable file-manager concepts:

- Navigation sidebar and common locations.
- Breadcrumb/location path.
- Search within the current location.
- List and grid views.
- Sorting, selection, copy, cut, paste, rename, delete and properties.
- Familiar right-click context menus.
- Clear access to mounted drives and removable media.

AureaOS may add its own folder artwork and visual finish, but must not rename common locations or hide expected file operations.

## 7. Workspaces

Four workspaces are available by default. They complement, rather than replace, ordinary window switching.

The desktop must show the active workspace clearly, provide direct selection and predictable previous/next navigation, and keep window membership understandable. Gestures are optional enhancements; every essential action must have a discoverable pointer or keyboard route.

## 8. Quick Settings

Quick Settings is a system-owned panel opened from the dock status area. It groups real controls for connectivity, audio, display, appearance, accessibility and power/session actions where supported.

Controls must reflect actual service state. Unavailable hardware capabilities must be hidden or explicitly marked unavailable. Never simulate a successful action.

The panel should be compact, recognizable and consistent with the desktop's established visual hierarchy.

## 9. Notifications

Notifications are system-managed and visually consistent across the desktop. They should be unobtrusive, readable, actionable where appropriate and grouped according to system policy.

Use macOS as a reference for refinement and consistency, not as a reason to relocate or reinvent familiar desktop workflows. Notifications must not steal focus unexpectedly, cover essential controls or imitate security prompts.

Users need predictable controls for dismissing notifications and accessing notification history where implemented.

## 10. Visual system

Dock, launcher, windows, menus, context menus, quick settings and notifications use the same semantic tokens.

The visual target combines:
- Familiar window and menu proportions.
- Soft, realistic shadows that communicate elevation.
- Restrained transparency and blur for floating surfaces.
- Clear borders and strong text contrast.
- Consistent rounded corners and icon alignment.
- Light and dark themes with equal hierarchy and legibility.
- Short, purposeful transitions.

Transparency is not mandatory when it harms readability. Reduced transparency and reduced motion settings must be respected. Do not apply glass effects indiscriminately.

## 11. Accessibility and input

All primary desktop tasks must be available by keyboard and pointer. Provide visible focus, logical focus restoration, accessible names, scalable text and status communication that does not rely only on color.

Use the shared keyboard convention document as the source of truth. Global shortcuts belong to the shell; applications must not intercept them without an explicit, documented exception.

## 12. Implementation boundaries

- **Compositor:** windows, placement, focus, input routing, workspace mechanics and presentation.
- **Shell:** dock, launcher, overview, panels and notification presentation.
- **System services:** network, audio, power, display, appearance and notification state.
- **Fides / Quaero:** file-management and search workflows.
- **Human Interfaces:** tokens, assets, component specifications, interaction rules and prototypes.

A visual control must never claim a capability that the corresponding runtime service does not provide.

## 13. First-login validation

- [ ] A Windows user recognizes the launcher and can find installed applications.
- [ ] Super opens and closes the launcher.
- [ ] Common shortcuts match the shared shortcut policy.
- [ ] Alt+Tab switches windows predictably.
- [ ] Minimize, maximize/restore and close controls are recognizable.
- [ ] Right-click opens a familiar context menu.
- [ ] Fides exposes common folders and file operations.
- [ ] The dock distinguishes pinned, running and minimized applications.
- [ ] Quick Settings shows real system state.
- [ ] Notifications are consistent and non-disruptive.
- [ ] Light and dark themes preserve the same information hierarchy.
- [ ] Reduced motion and reduced transparency remain usable.

## 14. Final principle

AureaOS should feel familiar before users notice its originality. Preserve the mental model they already know; improve the experience through integration, consistency, polish and reliability.
