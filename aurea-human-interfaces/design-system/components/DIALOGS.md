<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Dialog Specification

**Status:** Design specification

## Purpose
Dialogs interrupt the current surface only when a focused decision or task requires it. Prefer inline feedback for routine reversible actions.

## Structure
Use a clear title, concise explanation, task content and explicit action row. Titles describe the task, not a generic phrase such as “Attention”.

## Behavior
Move focus into a modal dialog, keep focus within it while active, restore focus on close and prevent background interaction. Escape closes dismissible dialogs where safe. Do not discard entered data without warning. Name destructive actions explicitly.

## Layout
Use consistent width constraints, padding, typography, action alignment and spacing. Adapt to smaller screens without clipping essential content.

## Themes and accessibility
Use semantic surface, text, border, overlay and focus tokens. Provide accessible role, title and description. Verify both themes and text scaling.

## Avoid
Nested modal dialogs, vague action labels, excessive confirmation dialogs and using a dialog for non-blocking information.
