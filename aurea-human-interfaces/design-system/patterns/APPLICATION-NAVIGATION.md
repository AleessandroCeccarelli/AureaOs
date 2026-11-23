<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Application Navigation Pattern

**Status:** Design specification

## Purpose
Application navigation helps users understand where they are, what content is available and how to move between related views.

## Principles
- Keep primary navigation visible and stable.
- Use a consistent location for navigation within the same application family.
- Make the current location or selected section explicit.
- Use breadcrumbs only when they clarify hierarchy and provide useful navigation.
- Keep global commands in standard menu categories.
- Do not duplicate system-level navigation inside every application.

## Keyboard and accessibility
Navigation items must be keyboard reachable, expose selected state and have meaningful accessible names. Focus order follows the visual and task hierarchy.

## Responsive behavior
When space is constrained, navigation may collapse into a clearly labeled compact control. Do not hide the only route to a primary task.
