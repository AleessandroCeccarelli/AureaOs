# AureaOS Debian Base

## Base Distribution

- Distribution: Debian GNU/Linux
- Release: Debian 13 (Trixie)
- Release channel: Stable
- Target architecture: amd64 (x86-64)

## Initial System Profile

The initial AureaOS base is a minimal, terminal-only system.

It must not include a graphical desktop environment or desktop applications.

Only the packages and services required for a functional, maintainable base system should be included.

## Build Objectives

- Define the base system configuration in the AureaOS repository.
- Use official Debian repositories and packages where appropriate.
- Keep package selection and system configuration version-controlled.
- Establish a reproducible Linux build environment targeting amd64.
- Validate the resulting filesystem and its boot behavior.
- Document dependencies, licenses, and known limitations.

## Scope

This document defines the initial Debian base.

The graphical desktop, AureaOS applications, installer, and final release image are outside the scope of the initial base implementation.

## Implementation Status

The Debian base is specified but has not yet been built or validated.