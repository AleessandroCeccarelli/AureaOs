<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Security and Lifecycle Audit — 2026-11-13

**Baseline revision:** `0ee00fbfa778cc75b0bd780dc2c847faa454c68a`  
**Latest source remediation revision:** `cf90d2c99f5d9edbd0b8eabcc1e3734374141754`  
**Scope:** current Rust workspace, compositor DRM/KMS and Vulkan lifecycle, Debian bootstrap, rootfs permission audit.  
**Status:** partial source audit with initial remediation commits; not a security certification and not a declaration that AureaOS is production-hardened.

## Executive summary

The current baseline contains several good defensive measures: Rust ownership for device handles, libseat-mediated DRM opening, Wayland socket-name validation, a live-client limit, explicit KMS surface cleanup, a restrictive bootstrap umask, package-name validation, rejection of pre-existing and repository-local rootfs targets, and package configuration inside systemd-nspawn.

The repository is still an early development system. The remaining gaps below must be addressed before describing the bootstrap or physical compositor path as hardened.

## Findings

### AUR-SEC-001 — Rootfs permission audit is incomplete

**Severity:** High (hardening gap; no direct exploit demonstrated)

`tools/security/audit-rootfs-permissions.sh` checks selected ownership/modes for critical paths, scans world-writable regular files and directories across the rootfs (on the rootfs filesystem), checks SUID/SGID entries against an explicit allowlist and exact modes, and scans file capabilities with `getcap`. It does not yet enforce a comprehensive ownership/mode policy for all of `/var`, `/home`, `/run`, service state, or every installed path, and its symlink policy is incomplete.

**Remediation in progress (2026-11-13):** the audit scans world-writable regular files/directories across the rootfs, permits only sticky `1777` shared temporary directories (`/tmp`, `/var/tmp`, `/dev/shm`), rejects SUID/SGID paths outside an explicit initial Debian utility allowlist, enforces the expected `4755` mode for each allowed entry, and fails closed if `getcap` is unavailable, reports capabilities, or exits unsuccessfully during the scan. The bootstrap checks for `getcap` before starting. The allowlist and exact modes have not yet been validated against a completed rootfs generated from the current package profile. **Additional test coverage added:** `tools/security/tests/test-audit-rootfs-permissions.sh` builds a disposable minimal rootfs fixture and checks valid permissions, accepted and rejected SUID modes, world-writable rejection, and fail-closed behavior when `getcap` exits unsuccessfully. **Still required:** execute this fixture suite on a root-capable host/CI runner, validate the allowlist against the final installed package profile, and add broader service-state ownership/mode and symlink checks.

### AUR-SEC-002 — systemd-nspawn capability policy is not a demonstrated minimum

**Severity:** High (hardening gap; no container escape demonstrated)

`tools/build/bootstrap-debian.sh` drops selected capabilities (`CAP_SYS_ADMIN`, `CAP_SYS_MODULE`, `CAP_SYS_RAWIO`, `CAP_SYS_BOOT`, `CAP_SYS_PTRACE`) but does not establish or test a minimal explicit capability allowlist, nor does the current script prove a complete namespace and writable-path policy.

**Required work:** derive the smallest capability set that still permits the required Debian maintainer scripts; explicitly configure and test namespace, device, mount, and network exposure; keep the host filesystem and user data inaccessible; add negative tests and a disposable Debian Trixie build test before changing the invocation.

### AUR-SEC-003 — Bootstrap target path has a privileged-operation race window

**Severity:** Medium (local race condition to investigate; exploitability depends on parent-directory control and timing)

The script checks that the destination does not exist and later performs privileged operations beneath that path. The destination's parent is normally user-writable. The source does not currently demonstrate an atomic reservation of the destination or protection against replacement of the target path between validation and subsequent privileged operations.

**Status:** not remediated. A reliable fix requires changing how privileged tools address the target (descriptor-relative/no-follow operations or a root-owned private staging parent); adding more `realpath` or string checks would not close the race. Keep this finding open until an implementation and adversarial replacement tests are available.

### AUR-DRM-001 — KMS clear failures are followed by dependent-resource release

**Severity:** High (display-resource lifecycle risk; hardware failure scenario not reproduced)

The original baseline cleared its surface and CRTC ownership maps even if one or more KMS clear operations failed, and then released GBM/Vulkan resources during session pause. That behavior could discard userspace ownership while the kernel might still be scanning out a framebuffer.

**Initial remediation (2026-11-13):** `DrmSurfaceManager::clear` now removes only surfaces whose KMS clear succeeded and retains failed surfaces plus their CRTC ownership for retry. Session-pause and orderly shutdown paths now retain GBM/Vulkan dependencies whenever any surface clear fails. A hardware-independent unit test covers retention and successful retry. The Udev hotplug integration rollback path now also attempts an explicit KMS clear before removing the surface manager; if that clear fails, it returns an error while retaining the surface manager, notifier, DRM device, GBM allocator, and Vulkan backend rather than dropping dependent resources. **Still required:** define behavior for terminal object destruction and confirmed device removal, add integration fault-injection for multiple physical outputs, and prove the kernel release conditions. This is a meaningful mitigation, not a complete quarantine guarantee.

### AUR-DRM-002 — Graphics backend initialization is best-effort and can leave an output without a render path

**Severity:** Medium (availability/consistency issue; not a privilege escalation)

**Initial remediation (2026-11-13):** initial server construction now creates KMS surfaces only for devices with both GBM and Vulkan ready, and registers logical outputs only after KMS surface creation succeeds. If logical output registration itself fails, the created surface manager remains owned by server state for orderly cleanup rather than being dropped immediately. Udev device integration uses the same dependency gate; connector refresh initializes KMS surfaces before publishing the reconciled output set. If KMS recovery fails after pause or connector change, stale Wayland output globals are withdrawn. Existing surfaces are not replaced unless their KMS clear succeeds.

The baseline initialized GBM and Vulkan independently and could register logical outputs before confirming that the complete render path existed. Pending frames were later cancelled when a dependency was missing.

**Still required:** add integration tests for multi-GPU hotplug and recovery, verify that failed output reconciliation cannot leave stale globals, and ensure retries republish outputs only after all prerequisites are restored.

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
- Bootstrap uses `set -Eeuo pipefail`, `umask 077`, validates package manifest tokens, rejects an existing target, and rejects targets inside the repository. These path checks do not prevent a concurrent replacement of a user-writable target parent.
- Package configuration is run through systemd-nspawn rather than directly on the host.
- Temporary `policy-rc.d` cleanup is registered on script exit.

These are source-level observations, not proof against all runtime or kernel-level failures.

## Verification record

- Fedora workspace formatting, workspace checks, DRM-feature checks, and workspace tests were reported passing by the developer before this audit.
- Vulkan shader compilation and `spirv-val --target-env vulkan1.3` were reported passing.
- The ignored Vulkan offscreen triangle test was explicitly run and passed.
- Initial KMS cleanup-retention and retry unit coverage was added in the remediation commits; it is hardware-independent and does not exercise the Smithay/kernel KMS operation.
- Startup and Udev hotplug paths were changed to gate output registration/publication on GBM, Vulkan, and KMS surface readiness; a pure readiness-gate unit test covers all four dependency combinations.
- Startup retains ownership of successfully created KMS surfaces even if logical output registration fails, so they remain available to explicit shutdown cleanup.
- Udev integration rollback now retains the complete graphics dependency chain if a KMS clear fails, rather than unconditionally removing the surface manager and its dependencies.
- Rootfs capability scanning now treats `getcap` execution errors as audit failures.
- Rootfs world-writable checks, SUID/SGID allowlist enforcement, and capability scanning are present in source. A disposable positive/negative fixture test script has been added but has not yet been executed; no completed-rootfs validation has been run.
- No post-remediation Fedora build/test run has been performed yet.
- No physical KMS/display test was performed during this audit.
- The GitHub commit inspected is the repository baseline identified above.

## Release gates

1. Execute the new rootfs fixture test suite, then validate the allowlist against a disposable completed Debian rootfs.
2. Close the bootstrap target-path race with root-owned staging and no-follow/descriptor-relative privileged operations; add replacement tests.
3. Test the nspawn sandbox with an explicit, measured capability and namespace policy.
4. Complete KMS quarantine semantics for terminal destruction/device removal and add multi-output fault injection; hotplug rollback retention is only an intermediate mitigation.
5. Make render readiness and degraded-output behavior explicit across startup and Udev hotplug.
6. Run physical KMS validation only on a recoverable test setup, with a known-good VT/session recovery procedure.
7. Re-run formatting, locked workspace checks, and all safe tests after each source change.

**Conclusion:** AureaOS has a promising defensive foundation, but the bootstrap and physical graphics lifecycle still have material hardening work outstanding. No claim of complete security should be made at this stage.
