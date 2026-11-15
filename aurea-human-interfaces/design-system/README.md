<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Design System

Canonical source for shared visual tokens, component contracts, recurring interaction patterns and keyboard conventions.

## Structure

- `tokens/index.css` — sole canonical CSS token source, including light/dark values.
- `components/` — component anatomy, behavior, states and accessibility contracts.
- `patterns/` — recurring compositions and workflows.
- `KEYBOARD-AND-COMMAND-CONVENTIONS.md` — shortcut defaults, command placement and ownership.

Applications and prototypes consume canonical tokens. Legacy `../themes/` files are compatibility wrappers only and must not contain an independent palette.

## Rules

Familiar desktop behavior takes priority over novelty. Keep shared commands, menu order and window controls consistent. Review both themes together. Define keyboard, pointer, focus, disabled, error and loading behavior. Respect reduced motion, reduced transparency, scalable text and accessibility. Keep specifications focused. Specifications describe intended contracts; native Rust implementation and service integration are tracked separately.
