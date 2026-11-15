<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Component Reference Examples

The canonical component specifications live in `../design-system/components/`.

This directory contains small, focused examples showing how specified components are composed. It is not a second specification tree and must not define competing tokens.

## Structure
- `examples/` — concise examples and reference compositions.

Each example identifies the component specification it demonstrates, uses canonical design tokens and shows relevant states. Native Rust implementation belongs in the appropriate AureaOS component crate, not here.
