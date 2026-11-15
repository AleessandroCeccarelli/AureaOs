<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Dialog Specification

**Status:** Design specification

## Purpose

Dialogs interrupt the current surface only when a focused decision or task requires it. Prefer inline feedback for routine reversible actions.

## Structure

Use a clear title, concise explanation, task content and explicit action row. Titles describe the task, not a generic phrase such as “Attention”.

## Surface and appearance

A dialog is a stable, readable application surface, not a notification card. Use the canonical window/elevated surface tokens, subtle border, shared radius and soft shadow. A restrained backdrop dim may separate a modal dialog from its owner. Do not use strong glass blur behind dense dialog content; preserve crisp text and controls.

## Behavior

Move focus into a modal dialog, keep focus within it while active, restore focus on close and prevent background interaction. Escape closes dismissible dialogs where safe. Do not discard entered data without warning. Name destructive actions explicitly.

## Layout

Use consistent width constraints, padding, typography, action alignment and spacing. Adapt to smaller screens without clipping essential content.

## Themes and accessibility

Use semantic surface, text, border, overlay and focus tokens. Provide accessible role, title and description. Verify both themes, text scaling, reduced motion and reduced transparency.

## Avoid

Nested modal dialogs, vague action labels, excessive confirmation dialogs, using a dialog for non-blocking information or applying notification-style glass to every application window.
