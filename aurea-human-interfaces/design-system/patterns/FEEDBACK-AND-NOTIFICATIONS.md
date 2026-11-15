<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Feedback and Notifications Pattern

**Status:** Design specification

## Purpose
Feedback confirms an action. Notifications communicate events that may matter beyond the current focused task.

## Choose the right surface
- **Inline feedback:** validation or status directly related to current content.
- **Toast:** brief, non-blocking confirmation of a routine action.
- **Notification:** an event that may need attention later or originates outside the current task.
- **Dialog:** only for a decision that must be resolved before continuing.

## Behavior
Messages state what happened and, when useful, what the user can do next. Notifications identify their source and expose only supported actions. Dismissal and grouping follow system policy.

## Accessibility
Important status changes are announced accessibly. Severity is not communicated by color alone. Avoid repeatedly announcing transient updates.
