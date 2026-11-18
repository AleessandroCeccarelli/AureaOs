<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Development Log

This is the chronological engineering diary for the AureaOS repository. It complements, rather than replaces, the architecture documents, security audits, roadmap, and Git history.

## 2026-11-18 — Repository baseline and display-scale foundation

### Repository state

- The active development branch is `main`.
- The repository currently has no other branches and no open pull requests.
- The Rust workspace contains four crates: `aurea-desktop`, `aurea-shell`, `aurea-desktop-api`, and `aurea-compositor`.
- The repository already has dedicated architecture, product, security, testing, and human-interface documentation. This entry adds a chronological record instead of duplicating those documents.
- No source files or branches were removed as part of this review: the inspection did not establish that any tracked item was obsolete or safe to delete.

### Engineering work recorded

Recent commits establish the initial display-coordinate and fractional-scale foundation:

- Exact rational scale representation and checked coordinate conversion.
- Separation of global logical coordinates from output-local physical pixels.
- Logical output placement based on scale-aware widths.
- Carrying the rational scale into render targets and pointer geometry.
- Registration of the Wayland fractional-scale manager global.
- Conservative initial preferred scale of 1.0 for newly created fractional-scale objects.

Relevant commits include:

- `9796c023` — document display coordinate foundation and migration status.
- `26f47d60` — define the Linux desktop interoperability contract.
- `6bebd8f1` and `d77db70d` — logical output placement and scale-aware placement tests.
- `8a0df018`, `9cce2778`, `2ae83868`, `d9ff4e36`, and `91f2b214` — fractional-scale protocol and manager-state work.
- `12cddaa8` — fix compositor fractional-scale state access and formatting.

### Verification

GitHub Actions run [44290834407](https://github.com/AleessandroCeccarelli/AureaOs/actions/runs/44290834407), for commit `12cddaa8fe35dff6d8b5e1eece5c5f70ce267424`, completed successfully on 2026-11-18.

All three jobs passed:

- Rust formatting.
- Workspace build and tests.
- Rust dependency security audit.

This is CI evidence for the checks configured in that workflow. It is not evidence of a successful boot on physical hardware, complete DRM/KMS scanout, a finished graphical desktop, or a bootable installation ISO.

### Known limitations and open work

- Fractional scaling is not end-to-end. Output-aware preferred-scale updates, surface buffer/viewport handling, cursor hotspots, clipping, hit-testing, and full renderer integration remain to be connected and validated.
- Physical DRM/KMS output and scanout still require testing on supported hardware.
- The Debian bootstrap and rootfs publication flow still require a complete end-to-end validation on the supported Fedora build host.
- The security audit dated 2026-11-13 records open hardening work. Its findings must not be treated as closed merely because the Rust dependency-audit CI job passes.
- The RustSec advisory affecting transitive `cgmath 0.18.0` remains documented in the security audit and needs an upstream-aware resolution; a successful audit job is not the same as a warning-free dependency report.

### Next engineering actions

1. Continue the fractional-scale integration by inspecting surface/output association and the relevant Smithay 0.7 APIs before changing behavior.
2. Define and test a consistent output-enter/leave and preferred-scale policy for toplevels, popups, and child surfaces.
3. Complete the buffer, viewport, cursor, clipping, and hit-testing integration in small, independently testable modules.
4. Preserve the explicit distinction between CI validation and physical-hardware validation in project documentation.
5. Revisit the open security-audit findings and schedule the Fedora rootfs/bootstrap validation when the build machine is available.

---

## Entry template

Use this structure for future entries:

- **Date and scope**
- **Repository/commit references**
- **Changes actually committed**
- **Verification performed and exact result**
- **Known limitations or regressions**
- **Next actions**

Do not record intentions as completed work.
