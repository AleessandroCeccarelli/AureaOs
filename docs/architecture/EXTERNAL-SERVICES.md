<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS — External Services Register

This file records external platforms and services that AureaOS uses or plans to integrate. It is an evolving inventory: an entry marked **planned** is not a claim that its integration is complete.

## Source hosting and version control

| Service / project | Role in AureaOS | Integration | Status |
|---|---|---|---|
| [Git](https://git-scm.com/) | Local version control and transport to remote repositories | Debian package `git` | Included in bootstrap manifest |
| [GitHub](https://github.com/) | Current source-code hosting and project collaboration | Git over HTTPS/SSH; browser-based project workflows | Current project host |
| [GitLab](https://gitlab.com/) | Alternative source hosting, issues, merge requests and CI/CD | Debian package `glab` provides the GitLab CLI; Git handles repository operations | CLI included in bootstrap; account and remote are optional |
| [Codeberg](https://codeberg.org/) | Potential alternative Git hosting | Git over HTTPS/SSH; no dedicated client required | Available as a compatible service; not specifically integrated |

Git is the underlying version-control client. `glab` complements Git with GitLab-specific operations; it is not a Git server and does not require a GitLab account to be installed.

## Operating-system and application sources

| Service / project | Role in AureaOS | Integration | Status |
|---|---|---|---|
| [Debian](https://www.debian.org/) | Base operating system packages and security updates | APT repositories for Debian 13 Trixie, updates and security | Used by bootstrap |
| [Flathub](https://flathub.org/) | Third-party desktop application source | Aurea Store will discover applications through Flathub | Planned |
| [Flatpak](https://flatpak.org/) | Application installation, runtime isolation, sandboxing and updates | Flatpak tooling and runtime integration | Planned; not yet in the current bootstrap profile |
| [crates.io](https://crates.io/) | Rust package registry used by Cargo for Rust dependencies | Cargo registry configuration and dependency resolution | Used by Rust development; target-system role depends on build packaging |

## Integration rules

- Prefer packages from the configured official Debian repositories when they meet the requirement.
- Do not add third-party APT repositories or install remote bootstrap scripts without a separate security and maintenance review.
- Keep optional hosting-service clients separate from the core Git client when their functionality is service-specific.
- Installing a client does not create an account, configure credentials, add a remote, or enable a hosted service.
- Update this register when a service is adopted, removed, replaced, or moves from planned to implemented.

## Current bootstrap scope

The `version-control.list` package manifest installs `git` and `glab` in the Debian target root filesystem through the bootstrap's shared APT download and offline installation flow. It does not install any Git hosting server.
