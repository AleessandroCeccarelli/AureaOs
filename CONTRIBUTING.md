

<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Contributing to AureaOS

First of all, thank you for your interest in AureaOS.

AureaOS is an open-source project built around a simple idea:

**Take the simplicity people love in operating systems, and shape it to fit Linux.**

Whether you are a developer, designer, tester, technical writer, or simply someone with ideas to share, your contribution can help shape the project.

## Rome Wasn't Built in a Day

AureaOS is in its early stages. There is still a lot to define, design, build, and test.

The project is developed incrementally. Existing components, architectural decisions, and documented limitations should be considered before proposing substantial changes.

We value thoughtful contributions, constructive feedback, and realistic proposals. You do not need to contribute code to help the project.

## How You Can Help

There are many ways to contribute:

- **Ideas and discussion:** suggest improvements and discuss the project's direction.
- **Design:** help shape a consistent, accessible, and elegant user experience.
- **Development:** contribute code to existing components or help develop new ones.
- **Testing:** help identify usability issues, bugs, regressions, and compatibility problems.
- **Documentation:** improve explanations, guides, architecture descriptions, and technical documentation.
- **Feedback:** share your experience and help us understand what users need.

Before working on a substantial change, open a discussion or issue so we can agree on the goal and approach.

## Architecture and Modularity

AureaOS is designed as a modular system.

Components should have clear responsibilities and well-defined boundaries. Changes should avoid unnecessary dependencies, duplicated functionality, and tight coupling between unrelated parts of the system.

When contributing code:

- Respect the existing component structure.
- Keep responsibilities within the appropriate component.
- Prefer clear interfaces over direct dependencies on internal implementation details.
- Reuse established project infrastructure when it is suitable.
- Avoid introducing new dependencies without a clear technical reason.
- Document important architectural decisions and trade-offs.

Changes to shared interfaces or architectural foundations should be discussed before implementation.

## Code Quality and Testing

Contributions should prioritize readability, maintainability, reliability, and security.

Where applicable:

- Follow the conventions and formatting rules of the language used by the component.
- Keep functions and modules focused on clear responsibilities.
- Handle errors explicitly and avoid silently ignoring failures.
- Add or update tests when changing behavior.
- Avoid claiming that a feature is complete when important limitations remain.
- Update documentation when implementation details or project status change.

Tests should verify actual behavior rather than simply confirm that code compiles.

## Technology and Design Consistency

Contributions should respect the project's documented technical direction.

Rust is the primary programming language for first-party AureaOS development where appropriate. Other technologies may be considered when there is a clear technical justification.

The desktop direction is GUI-first and Wayland-first.

Design contributions should follow the shared design foundations and approved visual tokens. The canonical token entry point is:

`aurea-human-interfaces/design-system/tokens/index.css`

Do not introduce conflicting palettes, typography decisions, or unrelated visual styles without discussing them first.

## Documentation

Documentation is part of the project, not an afterthought.

Contributions that change architecture, implementation status, dependencies, supported behavior, or design decisions should include the corresponding documentation updates.

Please distinguish clearly between:

- Implemented functionality.
- Partially implemented functionality.
- Planned features.
- Open architectural decisions.

This helps keep the repository accurate and prevents future contributors from relying on assumptions that are not supported by the code.

## A Project Built in the Open

AureaOS aims to be developed transparently and collaboratively.

Please keep contributions respectful, explain the reasoning behind proposed changes, and focus discussions on making the project better.

Before submitting a substantial contribution, make sure that its scope and intended behavior have been discussed.

Contributions to the project's original work are expected to follow the applicable project license, GPL-3.0-or-later, unless stated otherwise.

Third-party components, libraries, fonts, icons, illustrations, and other assets remain subject to their own licenses and terms. Contributors should identify relevant license and attribution requirements when introducing external material.

More detailed contribution procedures may be added as the development workflow becomes established.

## Thank You

Every useful suggestion, test, improvement, and contribution matters.

**Rome wasn't built in a day, and neither will AureaOS be. Let's build it together, one step at a time.**

