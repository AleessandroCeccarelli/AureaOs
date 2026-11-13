<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Security and Lifecycle Audit — 2026-11-13

**Baseline revision:** `0ee00fbfa778cc75b0bd780dc2c847faa454c68a`  
**Latest source/test remediation revisions:** audit `01aa9a8c2646402cefa1aef44e30238fb0d9e6b0`; fixture tests `a49df445e8b18a75613870c0013ee4e28e004a3a`  
**Scope:** current Rust workspace, compositor DRM/KMS and Vulkan lifecycle, Debian bootstrap, rootfs permission audit.  
**Status:** partial source audit with initial remediation commits; not a security certification and not a declaration that AureaOS is production-hardened.

## Executive summary

The current baseline contains several good defensive measures: Rust ownership for device handles, libseat-mediated DRM opening, Wayland socket-name validation, live-client and bounded Wayland-object limits, explicit KMS surface cleanup, a restrictive bootstrap umask, package-name validation, rejection of pre-existing and repository-local rootfs targets, and package configuration inside systemd-nspawn.

The repository is still an early development system. The remaining gaps below must be addressed before describing the bootstrap or physical compositor path as hardened.

## Findings

### AUR-SEC-004 — Wayland surface resource exhaustion

**Severity:** Medium (local denial of service)

**Original exposure:** the server limited simultaneous Wayland clients to 64, but did not bound the number of `wl_surface` resources created by one client or by the compositor. A connected local client could create a large number of surfaces and consume compositor memory.

**Source remediation (2026-11-13):** added `server/resource_limits.rs`, which enumerates all Wayland objects for every connected client after each display dispatch. A client is disconnected above 2048 objects; a global budget of 16384 objects is enforced by disconnecting the largest object owners until the total is back within budget. This counts uncommitted surfaces and other protocol objects, not only managed windows. Boundary tests cover per-client and global thresholds.

**Residual scope:** the check runs after a dispatch batch, so a client can temporarily allocate objects during one batch before the next quota scan. The thresholds must be validated against normal AureaOS desktop workloads on Fedora; add a hostile-client flood test and consider request-batch limits if transient peaks prove excessive.



### AUR-SEC-001 — Rootfs permission audit is incomplete

**Severity:** High (hardening gap; no direct exploit demonstrated)

`tools/security/audit-rootfs-permissions.sh` checks selected ownership/modes for critical paths, scans world-writable filesystem objects across the rootfs (on the rootfs filesystem, excluding symlink metadata), checks SUID/SGID filesystem objects against an explicit allowlist and exact modes, and scans file capabilities with `getcap`. It does not yet enforce a comprehensive ownership/mode policy for all of `/var`, `/home`, `/run`, service state, or every installed path, and its symlink policy is incomplete.

**Remediation in progress (2026-11-13):** the audit scans world-writable non-symlink filesystem objects across the rootfs, including FIFOs and other special files, permits only sticky `1777` shared temporary directories (`/tmp`, `/var/tmp`, `/dev/shm`), rejects SUID/SGID paths outside an explicit initial Debian utility allowlist (including unexpected setgid directories), enforces exact numeric owner, group, and mode for each allowlisted privileged executable (including Debian's `sudo`, D-Bus launcher helper, and shadow-group helpers), and fails closed if either filesystem traversal (`find`) is incomplete, if `getcap` is unavailable, if capabilities are reported, or if `getcap` exits unsuccessfully. The bootstrap checks for `getcap` before starting. The expanded allowlist now includes the expected Debian Trixie setuid/setgid helpers from the current package manifest (`sudo`, D-Bus, passwd/login and PAM dependencies); exact installed ownership and modes remain unverified until a completed rootfs is audited on Fedora. **Test coverage:** `tools/security/tests/test-audit-rootfs-permissions.sh` builds a disposable minimal rootfs fixture and checks valid permissions (including sticky `/tmp`, `/var/tmp`, and `/dev/shm`), accepted and rejected SUID modes and owners, SGID directory rejection, world-writable regular and special-file rejection, reported file-capability rejection, fail-closed behavior when `getcap` exits unsuccessfully, and fail-closed behavior when rootfs traversal fails. The fixture is intended for execution as root on the project's Fedora build machine; GitHub Actions workflows are configured as manual-only and are not part of this validation cycle. **Still required:** validate the allowlist against the final installed package profile and add broader service-state ownership/mode and symlink checks.

### AUR-SEC-002 — systemd-nspawn capability policy is not a demonstrated minimum

**Severity:** High (hardening gap; no container escape demonstrated)

`tools/build/bootstrap-debian.sh` drops selected capabilities (`CAP_SYS_ADMIN`, `CAP_SYS_MODULE`, `CAP_SYS_RAWIO`, `CAP_SYS_BOOT`, `CAP_SYS_PTRACE`) but does not establish or test a minimal explicit capability allowlist. The direct `systemd-nspawn` invocation previously had user namespacing disabled by default, which is unsafe for untrusted maintainer scripts. **Source change (2026-11-13):** the command now explicitly requests `--private-users=pick --private-users-ownership=map`, selecting a private UID/GID range while using an id-mapped mount instead of recursively changing on-disk ownership. This is intended to preserve rootfs numeric ownership and fail closed if id-mapped mounts are unavailable. This behavior has not yet been exercised on the Fedora host and is not considered verified.

The command also does not request `--private-network` or a veth; the Trixie manual states that without private networking the container has full access to the host network. This is relevant because APT requires outbound access, but it means network access remains shared with the host. The script does avoid recursive host `/dev` and `/sys` binds, which is a useful control but does not compensate for the remaining shared-network exposure.

**Still unverified:** confirm support on the actual Fedora kernel/systemd/filesystem, verify the user-namespace mapping and rootfs ownership before/after nspawn, and run a disposable Debian Trixie bootstrap. The capability set is still a selective drop list, not a demonstrated minimum allowlist. Network access remains shared with the host because APT currently needs outbound repository access; do not treat the container as network-isolated. A later network design should permit only required authenticated Debian mirrors without breaking package installation.

### AUR-SEC-003 — Bootstrap target path has a privileged-operation race window

**Severity:** Medium (local race condition to investigate; exploitability depends on parent-directory control and timing)

The script checks that the destination does not exist and later performs privileged operations beneath that path. The destination's parent is normally user-writable. The source does not currently demonstrate an atomic reservation of the destination or protection against replacement of the target path between validation and subsequent privileged operations.

**Remediation in progress (2026-11-13):** the bootstrap no longer runs privileged debootstrap, APT, maintainer-script, audit, or nspawn operations against the user-selected final pathname. It creates a root-owned staging directory with `mktemp` under `/var/tmp`, requires `/var/tmp` to be a real root-owned `1777` directory, and refuses to proceed unless staging and the destination parent are on the same filesystem. The script opens the canonical destination parent as its working directory, verifies that its physical path still matches the previously canonicalized path, and pins that directory inode. Publication is delegated to `tools/build/publish_rootfs.py`: it opens `/var/tmp` and the destination parent as directory file descriptors with `O_NOFOLLOW`, verifies root ownership and exact modes on `/var/tmp` and staging, checks same-filesystem and inode identity, then calls `renameat2(RENAME_NOREPLACE)` through libc. It has no check-then-rename fallback; if the kernel/filesystem cannot provide atomic no-replace semantics, publication fails and staging is preserved. The shell fixture checks collision preservation, symlink preservation, rejection of a substituted parent symlink, and that the destination parent remains pinned if its pathname is renamed and replaced. The root-only Python fixture `tools/build/tests/test_publish_rootfs.py` covers inode-preserving publication, existing directory/symlink collisions, and staging-parent permissions. These fixtures are retained for manual local execution; GitHub Actions workflows are manual-only and are not run as part of this development cycle.

**Remaining limitations:** the fixtures exercise the publication helper and shell primitives, not a full concurrent end-to-end bootstrap under adversarial scheduling. The final directory can be renamed by its owner after the post-publication identity check, and a completed Debian rootfs build has not yet been run with the new staging flow. Keep this finding open until a disposable end-to-end bootstrap and targeted race testing confirm the behavior on the supported Fedora environment.

### AUR-DRM-001 — KMS clear failures are followed by dependent-resource release

**Severity:** High (display-resource lifecycle risk; hardware failure scenario not reproduced)

The original baseline cleared its surface and CRTC ownership maps even if one or more KMS clear operations failed, and then released GBM/Vulkan resources during session pause. That behavior could discard userspace ownership while the kernel might still be scanning out a framebuffer.

**Initial remediation (2026-11-13):** `DrmSurfaceManager::clear` now removes only surfaces whose KMS clear succeeded and retains failed surfaces plus their CRTC ownership for retry. Session-pause and orderly shutdown paths now retain GBM/Vulkan dependencies whenever any surface clear fails. A hardware-independent unit test covers retention and successful retry. The Udev hotplug integration rollback path now also attempts an explicit KMS clear before removing the surface manager; if that clear fails, it returns an error while retaining the surface manager, notifier, DRM device, GBM allocator, and Vulkan backend rather than dropping dependent resources. **Additional remediation (2026-11-13):** the confirmed Udev device-removal path now uses `DrmSurfaceManager::release_after_device_removal()` instead of issuing KMS disable ioctls against an unplugged device. This releases userspace scanout references only after the kernel has reported device removal, then removes the notifier, GBM allocator, Vulkan backend, and DRM device in dependency order. This avoids retaining a stale device identity indefinitely and permits a re-added GPU to be discovered again. Generic KMS clear failures during pause/reconfiguration still retain the manager; they have no dedicated retry/quarantine owner yet, and process termination still invokes best-effort `Drop`. **Still required:** define an explicit retry/quarantine owner for generic clear failures and terminal destruction, add integration fault-injection for multiple physical outputs, and prove the kernel release conditions.

### AUR-DRM-002 — Graphics backend initialization is best-effort and can leave an output without a render path

**Severity:** Medium (availability/consistency issue; not a privilege escalation)

**Initial remediation (2026-11-13):** initial server construction now creates KMS surfaces only for devices with both GBM and Vulkan ready, and registers logical outputs only after KMS surface creation succeeds. If logical output registration itself fails, the created surface manager remains owned by server state for orderly cleanup rather than being dropped immediately. Udev device integration uses the same dependency gate; connector refresh initializes KMS surfaces before publishing the reconciled output set. If KMS recovery fails after pause or connector change, stale Wayland output globals are withdrawn. Existing surfaces are not replaced unless their KMS clear succeeds.

The baseline initialized GBM and Vulkan independently and could register logical outputs before confirming that the complete render path existed. Pending frames were later cancelled when a dependency was missing.

**Additional source correction (2026-11-13):** review of the deferred connector-change path found that it refreshed/published logical outputs before recreating KMS surfaces after session activation. The order is now reversed: native KMS surfaces are restored first, logical outputs are reconciled second, and stale globals are withdrawn if either step fails. This correction is awaiting Fedora runtime validation.

**Still required:** add integration tests for multi-GPU hotplug and recovery, verify that failed output reconciliation cannot leave stale globals, and ensure retries republish outputs only after all prerequisites are restored.

### AUR-VK-001 — Offscreen Vulkan validation is not physical KMS validation

**Severity:** Release gate (not a code vulnerability)

The offscreen triangle test validates Vulkan command execution and readback. It does not validate DRM atomic modesetting, DMA-BUF import/export interoperability on every target GPU, page-flip completion, session pause/resume, hot-unplug, or safe teardown under KMS errors.

The source explicitly implements a clear-only scanout path; scene composition is not yet connected.

**Required work:** add hardware-independent fault-injection tests for state transitions and resource ownership, then perform controlled physical-device tests on a recoverable test setup. Do not use the current developer laptop as a blind KMS test target.

## Verified defensive measures

- Udev DRM device removal now attempts explicit KMS surface clearing before releasing the surface manager or dependent GBM/Vulkan/DRM resources; on clear failure the dependency chain remains retained in `WaylandState`. This is not yet a managed quarantine/retry solution.

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
- Rootfs capability scanning now treats `getcap` execution errors as audit failures. Both world-writable and SUID/SGID filesystem traversals now capture `find` output and fail closed if traversal exits unsuccessfully, instead of relying on process substitution whose producer status is otherwise lost.
- Rootfs world-writable checks, SUID/SGID allowlist enforcement, and capability scanning are present in source. A disposable positive/negative fixture test script passed in the historical GitHub Actions run `43485619945`; no new GitHub-hosted test runs are being used, and no completed-rootfs validation has been run.
- No post-remediation Fedora build/test run has been performed yet. The rootfs publication fixture and rootfs permission fixture are retained for local execution on the Fedora machine. Both GitHub Actions workflows are manual-only and are not being run. The nspawn private-user/idmapped-mount option and the corrected DRM session-activation ordering are also awaiting Fedora validation.
- No physical KMS/display test was performed during this audit.
- Source review includes the baseline and the remediation commits recorded in this report; the latest DRM source change is `58e3e78da7e01ffc2749f49b93d4c8b2f4be2340`; the latest rootfs audit change is `01aa9a8c2646402cefa1aef44e30238fb0d9e6b0`; the latest rootfs fixture-test change is `a49df445e8b18a75613870c0013ee4e28e004a3a`.

## Release gates

1. Execute the new rootfs fixture test suite, then validate the allowlist against a disposable completed Debian rootfs on Fedora.
2. Validate the new protected-staging and no-clobber publication flow with a disposable end-to-end Debian bootstrap on Fedora, including id-mapped private-user behavior, then add concurrent race testing.
3. Test the nspawn sandbox with an explicit, measured capability and namespace policy.
4. Complete KMS quarantine semantics for terminal destruction/device removal and add multi-output fault injection; hotplug rollback retention is only an intermediate mitigation.
5. Make render readiness and degraded-output behavior explicit across startup and Udev hotplug.
6. Run physical KMS validation only on a recoverable test setup, with a known-good VT/session recovery procedure.
7. On the Fedora machine, re-run formatting, locked workspace checks, and all safe tests after the source changes; do not run the test workflows on GitHub.

**Conclusion:** AureaOS has a promising defensive foundation, but the bootstrap and physical graphics lifecycle still have material hardening work outstanding. No claim of complete security should be made at this stage.
