<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS UI Design Specification

This document describes the intended desktop experience for AureaOS.

The core idea is simple: AureaOS should feel familiar and comfortable to people who already use Windows or macOS, but it must still feel like AureaOS and not like a visual copy of another operating system.

## 1. Main design direction

AureaOS should be:

- familiar in workflow and behavior
- clean, modern, and readable
- calm and premium in appearance
- quick and productive in daily use
- visually consistent across shell, apps, and system surfaces

## 2. Visual language

The interface should use a soft, modern Windows-like light theme and a refined dark theme with strong readability. The shell should feel clear and stable rather than overloaded.

### 2.1 Light theme

The light theme should feel bright and calm.

Suggested palette:

- desktop background: soft pale blue-gray
- window surface: white or near-white
- panel surfaces: translucent white-blue glass
- text: near-black or dark gray
- buttons and controls: dark gray or almost black
- primary accent: soft blue
- warm accent: subtle gold / amber

This should feel close to Windows 11 in clarity, but cleaner and more premium.

### 2.2 Dark theme

The dark theme should stay modern and comfortable, especially for long work sessions.

Suggested palette:

- desktop background: deep blue-gray slate
- window surface: dark navy slate
- panel surfaces: translucent dark glass
- text: soft white and gray-blue
- buttons and controls: light gray-blue
- primary accent: bright blue
- warm accent: soft gold

The dark theme should not be black-heavy or harsh. It should stay gentle and readable.

## 3. Window style

Windows should be clean and precise.

- white or dark-slate surfaces
- soft rounded corners
- light border lines
- small, readable shadows
- crisp controls and readable text
- consistent spacing and strong hierarchy

The window style should feel modern, clear, and reliable.

## 4. Dock and shell layout

AureaOS should use a bottom dock as the main shell element.

### 4.1 Dock behavior

- floating dock at the bottom of the screen
- semi-transparent glass panel
- compact but easy to use
- hidden or minimized when not needed
- revealed near the bottom edge when the pointer approaches
- no large permanent top bar in the default layout

This creates a calmer desktop and keeps the shell cleaner than a traditional full-height panel layout.

### 4.2 Left side of dock

The left side of the dock should include the AureaOS logo.

This should open the launcher and serve as the system entry point for:

- apps
- search
- settings
- folders
- recent items
- system tools

### 4.3 Right side of dock

The right side should include:

- time and date
- network status
- mounted disks
- audio status
- quick system access

When clicked, these items should open a compact quick settings panel.

## 5. Quick settings panel

The quick settings panel should be compact, modern, and easy to use.

It should include:

- volume and sound controls
- brightness control
- Wi-Fi and network status
- Bluetooth
- themes
- accessibility options
- power and display settings

Like the dock, it should be glass-like and soft, but still readable and precise.

## 6. Notifications

Notifications should be floating, glass-like, and modern.

They should:

- feel like soft floating cards
- be readable even on busy backgrounds
- avoid being too heavy or too opaque
- have gentle shadow and clear contrast
- support a premium feeling without losing usability

The notification system should feel close to modern liquid-glass design, but with stronger readability and less risk of visual clutter.

## 7. Launcher and app search

The launcher should be fast and simple.

It should support:

- app search
- file search
- settings search
- recent files and apps
- folder access
- quick actions for system functions

The launcher should be the default entry point for productivity and system access.

## 8. Keyboard workflow

AureaOS should support a familiar keyboard-first workflow without intimidating casual users.

Recommended shortcuts:

- Super / Windows key = launcher or overview
- Alt + Tab = app switching
- Alt + F4 = close window
- Ctrl + Alt + T = terminal
- Super + E = file manager
- Super + L = lock screen
- Super + Arrow keys = window snapping

## 9. Accessibility

Accessibility should be built into the system from the beginning.

This includes:

- strong text contrast
- readable controls
- visible focus styles
- keyboard-friendly navigation
- subtle and comfortable motion

## 10. Fonts

AureaOS should use a modern sans-serif system stack with crisp readability and a clean desktop feel.

Recommended font stack:

- Segoe UI Variable
- Segoe UI
- Inter
- SF Pro Display
- Helvetica Neue
- Arial
- sans-serif

A monospaced font should also be available for terminal and code use, such as:

- Cascadia Code
- JetBrains Mono
- SFMono-Regular
- Consolas

The typography should be elegant but practical, and it should support both UI work and developer use.

## 11. Final visual rule

AureaOS should feel like:

- familiar to Windows and macOS users
- modern and premium in appearance
- calm and productive in daily use
- clearly distinct from generic Linux themes

The final system should not feel like a copy of another OS. It should feel like a new operating system that respects the habits of modern users while expressing its own identity.
