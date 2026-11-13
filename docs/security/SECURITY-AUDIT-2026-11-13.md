<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Security and Lifecycle Audit — 2026-11-13

**Audited revision:** `0ee00fbfa778cc75b0bd780dc2c847faa454c68a`  
**Scope:** current Rust workspace, compositor DRM/KMS and Vulkan lifecycle, Debian bootstrap, rootfs permission audit.  
**Status:** partial source audit; not a security certification and not a declaration that AureaOS is production-hardened.

## Executive summary

The current baseline contains several good defensive measures: Rust ownership for device handles, libseat-mediated DRM opening, Wayland socket-name validation, a live-client limit, explicit KMS surface cleanup, a restrictive bootstrap umask, package-name validation, rejection of pre-existing and repository-local rootfs targets, and package configuration inside systemd-nspawn.

The repository is still an early development system. The remaining gaps below must be addressed before describing the bootstrap or physical compositor path as hardened.

## Findings

### AUR-SEC-001 — Rootfs permission audit is incomplete

**Severity:** High (hardening gap; no direct exploit demonstrated)

`tools/security/audit-rootfs-permissions.sh` checks selected ownership/modes, checks world-writable files and directories only under `/etc` and `/usr`, and prints an SUID/SGID inventory. It does not fail on an unapproved SUID/SGID entry, does not inspect file capabilities, and does not enforce the documented policy for all of `/var`, `/home`, `/run`, or other installed paths.

**Required work:** define a reviewed SUID/SGID allowlist; audit file capabilities; check world-writable paths across the complete rootfs with documented exceptions; validate symlink targets and ownership/modes for service state; make unexpected privileged metadata a hard failure.

### AUR-SEC-002 — systemd-nspawn capability policy is not a demonstrated minimum

**Severity:** High (hardening gap; no container escape demonstrated)

`tools/build/bootstrap-debian.sh` drops selected capabilities (`CAP_SYS_ADMIN`, `CAP_SYS_MODULE`, `CAP_SYS_RAWIO`, `CAP_SYS_BOOT`, `CAP_SYS_PTRACE`) but does not establish or test a minimal explicit capability allowlist, nor does the current script prove a complete namespace and writable-path policy.

**Required work:** derive the smallest capability set that still permits the required Debian maintainer scripts; explicitly configure and test namespace, device, mount, and network exposure; keep the host filesystem and user data inaccessible; add negative tests and a disposable Debian Trixie build test before changing the invocation.

### AUR-SEC-003 — Bootstrap target path has a privileged-operation race window

**Severity:** Medium (local race condition to investigate; exploitability depends on parent-directory control and timing)

The script checks that the destination does not exist and later performs privileged operations beneath that path. The destination's parent is normally user-writable. The source does not currently demonstrate an atomic reservation of the destination or protection against replacement of the target path between validation and subsequent privileged operations.

**Required work:** redesign target creation and cleanup around a safely reserved directory and descriptor-relative/no-follow operations where practical; verify the target identity before every privileged operation; add adversarial symlink/replacement tests. Do not attempt a superficial string/path check as a substitute for race resistance.

### AUR-DRM-001 — KMS clear failures are followed by dependent-resource release

**Severity:** High (display-resource lifecycle risk; hardware failure scenario not reproduced)

`DrmSurfaceManager::clear` attempts to clear all surfaces, then clears its surface and CRTC ownership maps even if one or more KMS clear operations failed. The session-pause handler then releases GBM and Vulkan resources regardless of the returned cleanup error, before reporting the error to the event loop.

This preserves best-effort teardown and prevents one failed output from blocking cleanup of the others. However, a failed KMS disable means the code has not proved that the kernel stopped scanning out every framebuffer before userspace releases its related resources.

**Required work:** define a per-output failure state and an explicit recovery policy. Preserve or quarantine resources whose scanout-disable operation failed until a successful disable, device removal, or another documented kernel-guaranteed release point. Add fault-injection tests for one and multiple failed outputs. Do not treat a successful Rust build as proof of safe physical scanout teardown.

### AUR-DRM-002 — Graphics backend initialization is best-effort and can leave an output without a render path

**Severity:** Medium (availability/consistency issue; not a privilege escalation)

GBM initialization failures are logged and do not stop later Vulkan and KMS surface initialization. The render dispatcher subsequently cancels pending output frames when the GBM device is unavailable. This can leave a discovered/published output without a functioning scanout path.

**Required work:** make readiness explicit per DRM node. Only advertise a usable rendering output when the required DRM, GBM, Vulkan, and KMS resources are ready, or expose a clearly represented degraded state with a retry path. Keep device-local failures isolated from unrelated GPUs.

### AUR-VK-001 — Offscreen Vulkan validation is not physical KMS validation

**Severity:** Release gate (not a code vulnerability)

The offscreen triangle test validates Vulkan command execution and readback. It does not validate DRM atomic modesetting, DMA-BUF import/export interoperability on every target GPU, page-flip completion, session pause/resume, hot-unplug, or safe teardown under KMS errors.

The source explicitly implements a clear-only scanout path; scene composition is not yet connected.

**Required work:** add hardware-independent fault-injection tests for state transitions and resource ownership, then perform controlled physical-device tests on a recoverable test setup. Do not use the current developer laptop as a blind KMS test target.

## Verified defensive measures

- DRM device paths must be absolute and devices are opened through the active libseat session.
- The opened DRM device identity is checked against the Udev identity before integration.
- Wayland socket names are restricted to a basename with a bounded ASCII character set.
- The Wayland server limits simultaneous live clients.
- Vulkan device extension selection is based on advertised device support, and DRM-associated selection requires the DMA-BUF interop extension plan.
- KMS surfaces have explicit clear logic and a best-effort Drop fallback.
- Bootstrap uses `set -Eeuo pipefail`, `umask 077`, validates package manifest tokens, rejects an existing target, and rejects targets inside the repository.
- Package configuration is run through systemd-nspawn rather than directly on the host.
- Temporary `policy-rc.d` cleanup is registered on script exit.

These are source-level observations, not proof against all runtime or kernel-level failures.

## Verification record

- Fedora workspace formatting, workspace checks, DRM-feature checks, and workspace tests were reported passing by the developer before this audit.
- Vulkan shader compilation and `spirv-val --target-env vulkan1.3` were reported passing.
- The ignored Vulkan offscreen triangle test was explicitly run and passed.
- No physical KMS/display test was performed during this audit.
- The GitHub commit inspected is the repository baseline identified above.

## Release gates

1. Complete the rootfs metadata audit and capability allowlist.
2. Add bootstrap negative tests for existing targets, repository-local targets, symlink targets, malformed manifests, and target-path replacement attempts.
3. Test the nspawn sandbox with an explicit, measured capability and namespace policy.
4. Add DRM/KMS fault-injection tests for pause, activation, unplug, failed clear, and shutdown.
5. Make render readiness and degraded-output behavior explicit.
6. Run physical KMS validation only on a recoverable test setup, with a known-good VT/session recovery procedure.
7. Re-run formatting, locked workspace checks, and all safe tests after each source change.

**Conclusion:** AureaOS has a promising defensive foundation, but the bootstrap and physical graphics lifecycle still have material hardening work outstanding. No claim of complete security should be made at this stage.
