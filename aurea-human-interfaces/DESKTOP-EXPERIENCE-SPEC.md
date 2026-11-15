<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Desktop Experience Specification

**Status:** Design specification  
**Scope:** Desktop shell composition, launcher, dock, workspaces, window organization, notifications and quick settings  
**Visual source of truth:** `design-system/tokens/index.css`  
**Related application rules:** `APP-DESIGN-GUIDELINES.md`

## 1. Purpose and design identity

The AureaOS desktop is the primary environment through which users access applications, organize work and control the system.

Its design combines useful ideas from modern desktop environments without reproducing another system's interface. The intended result is recognizably AureaOS: calm blue-gray surfaces, a luminous blue visual identity, restrained glass materials, clear typography, predictable controls and purposeful motion.

The desktop must feel like one coherent product. Dock, launcher, windows, notifications, quick settings and applications must share the same design tokens and interaction rules.

This document defines the intended experience. It does not claim that every described behavior is already implemented.

## 2. Desktop composition

The default desktop is an uncluttered workspace with:

- A wallpaper from the approved AureaOS visual collection.
- A full-width intelligent dock along the bottom.
- Four virtual workspaces.
- Floating system surfaces for launcher, notifications and quick settings.
- Application windows managed consistently by the desktop shell and compositor.

The default desktop should not be crowded with decorative widgets or unnecessary permanent panels. Information should appear when useful and remain easy to reach.

## 3. Dock

### 3.1 Structure

The dock spans the full usable width of the screen and is divided into three functional zones:

**Left — system entry and workspaces**
- Aurea **A** mark, which opens the launcher.
- Workspace navigation and current-workspace indication.

**Center — applications**
- Pinned applications.
- Running applications.
- Clear active and attention-needed indicators.
- Window previews when supported.

**Right — system status**
- Clock and date.
- Network status.
- Audio status.
- Battery and power status when available.
- Notification entry point.
- Quick Settings entry point.

The order and grouping of these zones must remain stable. Items may adapt to available space, but must not become unpredictably rearranged.

### 3.2 Appearance

The dock uses the shared AureaOS glass surface tokens, restrained blur, a subtle border and controlled elevation. It must remain legible over both light and dark wallpapers.

Icons use a consistent visual size and alignment. Running, pinned and attention states must be distinguishable without relying only on color.

### 3.3 Adaptive behavior

The dock has an intelligent presentation:

- When there is sufficient desktop space, it floats slightly above the bottom edge.
- When a maximized or tiled window needs the available space, the dock can attach to the bottom edge and reduce its visual footprint.
- Users may choose an always-visible or auto-hide behavior where supported.
- In auto-hide mode, the dock reappears through pointer-edge or keyboard navigation.
- It must not obscure important window controls or content.
- Its geometry and animation must remain stable and predictable during window changes.

Exact dimensions, thresholds and animation timing must be validated in the interactive prototype and then in native implementation.

## 4. Application launcher

### 4.1 Entry and dismissal

The launcher opens when the user:

- Clicks the Aurea **A** mark on the dock.
- Presses the **Super / Windows** key.

The opening animation must be short, subtle and consistent with the system motion language. It should communicate where the launcher appears without delaying interaction.

Behavior:
- Pressing Super while the launcher is closed opens it.
- Pressing Super again toggles it closed.
- Escape closes it.
- Selecting an application launches it and closes the launcher unless a future interaction explicitly requires it to remain open.
- Clicking outside dismisses it where this does not interfere with another active operation.
- The launcher receives keyboard focus and makes search immediately available.

The launcher respects the system reduced-motion preference.

### 4.2 Information architecture

The launcher should contain:

1. Search field at the top.
2. Pinned or favorite applications.
3. Recently used applications or relevant recent items, where enabled.
4. Application catalog organized by categories.
5. A clearly accessible route to system settings and power/session actions.

The launcher must not become a crowded dashboard. Search and pinned applications are the primary access paths; secondary content should remain visually subordinate.

### 4.3 Search

The launcher integrates with Quaero rather than implementing a separate competing search engine.

The search experience is intended to cover:
- Installed applications.
- Files and folders.
- System settings.
- Supported system actions.
- Other indexed sources only when explicitly supported and permission-appropriate.

Results must be grouped or labeled clearly, ordered by relevance and navigable from the keyboard. The interface must communicate when results are loading, empty, unavailable or restricted.

Search must respect file permissions and service boundaries. The launcher must not expose content that the current user is not authorized to access.

## 5. Window organization

The desktop must make window organization discoverable without overwhelming the user.

### 5.1 Window controls and title area

All first-party windows use the shared window framework and application chrome defined in `APP-DESIGN-GUIDELINES.md`.

Minimize, maximize/restore and close controls must have consistent placement and behavior. Window titles and application identity must remain clear.

### 5.2 Snap and tiling

The intended desktop experience includes pointer-accessible snap choices from the maximize control and predictable keyboard operations.

The snap system should support common arrangements such as:
- Left and right halves.
- Four screen quadrants.
- One large area with a smaller adjacent area.
- Other layouts only when they remain understandable and useful.

Snap previews must clearly show the destination before the window is placed. Keyboard and pointer workflows should offer equivalent access.

The exact layout catalogue, keyboard bindings and compositor implementation must be specified and tested before being treated as implemented.

### 5.3 Window overview

A window overview should provide a clear view of open windows and workspaces. It must help users find and switch tasks without introducing duplicate or confusing window representations.

The overview must be keyboard accessible and should support direct selection of a window or workspace.

## 6. Virtual workspaces

AureaOS provides four virtual workspaces by default.

The desktop should:
- Show the active workspace clearly.
- Allow direct workspace selection.
- Provide predictable previous/next navigation.
- Keep window placement and workspace membership understandable.
- Animate workspace transitions subtly and respect reduced motion.
- Support keyboard navigation and touchpad gestures where hardware and compositor support are available.

The visual design must not imply that a gesture or shortcut works until the relevant input and compositor behavior is implemented.

## 7. Quick Settings

Quick Settings is a system-owned panel opened from the dock's status area.

It should group controls by purpose rather than present an unstructured collection of toggles.

Planned groups include:
- Connectivity: Wi-Fi, wired network, Bluetooth and airplane mode, where supported.
- Audio: volume, mute and available output/input selection.
- Display: brightness and display configuration when supported by hardware and services.
- Appearance: light, dark and automatic theme selection.
- Accessibility: direct access to relevant system accessibility settings.
- Capture: screen recording controls only when the recording service is available.
- Power and session: clearly separated system actions.

Controls must display actual system state. An unavailable capability must be hidden or clearly marked unavailable; the interface must never simulate a successful hardware action.

Destructive or disruptive actions must be separated from routine toggles and require appropriate confirmation.

## 8. Notifications

Notifications are system-managed surfaces and must follow the shared notification policy.

The intended desktop presentation is a restrained stack along the left side of the screen.

Each notification should:
- Identify its originating application.
- Show a concise title and message.
- Expose only relevant and supported actions.
- Provide predictable dismissal behavior.
- Remain readable in both themes.
- Support keyboard access and accessible announcements.

Notifications must not cover essential application controls or steal focus unexpectedly. Grouping, priority and interruption behavior should be governed by system policy rather than each application inventing its own rules.

## 9. Theme and materials

The desktop follows the global AureaOS appearance setting:
- Light.
- Dark.
- Automatic / follow system.

Dock, launcher, workspace overview, quick settings, notifications and all first-party windows must update together when the system appearance changes.

Use semantic tokens from `design-system/tokens/index.css`. Do not introduce separate palettes for individual desktop panels.

Transparency and blur are reserved for surfaces where they improve hierarchy. Text contrast and legibility take priority over glass effects. The desktop must remain usable when transparency, blur or animation is reduced.

## 10. Motion and feedback

Desktop motion must be purposeful and consistent.

Use brief transitions for:
- Opening and closing the launcher.
- Showing and dismissing panels.
- Switching workspaces.
- Displaying snap previews.
- Dock visibility changes.
- Notification appearance and dismissal.

Avoid long delays, excessive bounce, decorative movement and animations that obscure the current state.

All motion must respect the system reduced-motion preference. State changes must remain understandable without animation.

## 11. Accessibility and input

The desktop must support:
- Keyboard access to all primary shell functions.
- Visible focus indicators.
- Logical focus order and restoration.
- Accessible names and roles for controls.
- Pointer alternatives for keyboard operations and vice versa where practical.
- Scalable text and display scaling.
- Reduced motion and reduced transparency.
- Status communication that does not rely only on color.

Touchpad gestures are enhancements, not the only way to perform an operation.

## 12. Implementation boundaries

The desktop experience is shared across distinct technical responsibilities:

- **Compositor:** window surfaces, placement, focus, input routing, workspace mechanics and presentation.
- **Shell:** dock, launcher, overview, panels, notifications presentation and user-facing desktop state.
- **System services:** network, audio, power, display, appearance, notifications and search capabilities.
- **Human Interfaces:** design tokens, assets, component specifications, interaction patterns and prototypes.

A visual control must not claim a capability that the relevant service or compositor interface does not provide. UI state must reflect real service state and report errors or unavailable capabilities clearly.

The Human Interfaces directory defines the intended visual and interaction contract; it does not replace runtime APIs or implementation documentation.

## 13. Prototype validation checklist

Before promoting a desktop behavior to a production requirement, validate it in the interactive prototype:

- [ ] Dock zones and item order are understandable.
- [ ] Dock remains usable in floating, attached and auto-hide states.
- [ ] Launcher opens from the A mark and Super key.
- [ ] Launcher animation is subtle and does not delay search.
- [ ] Search results have clear categories and keyboard navigation.
- [ ] Quick Settings controls are grouped and state-oriented.
- [ ] Notifications remain readable and do not obscure essential work.
- [ ] Workspace switching is discoverable.
- [ ] Window snap previews are understandable.
- [ ] Light and dark themes cover every desktop surface.
- [ ] Reduced-motion behavior is defined.
- [ ] Layout remains usable at supported display sizes and scales.

A prototype can validate appearance and interaction flow; it cannot prove compositor, hardware or system-service functionality.

## 14. Final principle

The AureaOS desktop should provide one coherent place to launch, organize and control work.

The dock is the persistent navigation and status anchor. The launcher is the fast entry point. Windows and workspaces organize tasks. Quick Settings exposes real system controls. Notifications communicate without interrupting unnecessarily.

Every part must share one visual language, one predictable interaction model and a distinct AureaOS identity.
