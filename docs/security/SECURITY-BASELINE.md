<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Security Baseline

**Status:** architecture contract for the current bootstrap and compositor foundation. This document is not a certification or a claim that the current root filesystem is hardened.

## 1. Security objective

AureaOS follows least privilege and defense in depth. Filesystem ownership and mode bits are mandatory controls, not substitutes for process isolation, authorization policy, or application-level validation.

No component receives root privileges merely because it is part of the desktop. Privileged operations must have a narrowly defined owner, interface, authorization rule, and audit trail.

## 2. Trust boundaries

| Boundary | Required rule |
| --- | --- |
| Host build machine | Bootstrap may create a target root filesystem only at a new, explicit path outside the Git worktree. It must not install packages or change configuration on the host. |
| Bootstrap process | Privileged host operations are limited to creating and configuring the target root filesystem. Temporary files and cleanup must be deterministic and fail closed. |
| Target root filesystem | Package maintainer scripts execute inside systemd-nspawn, never directly on the host. Container capabilities and writable paths must be reduced to the minimum demonstrated by tests. |
| System services | Each daemon runs as a dedicated system account, not as root, unless a documented kernel-facing operation requires a narrowly scoped privilege. |
| User session | The compositor and shell run as the logged-in user. Device access is brokered through libseat/logind session ownership; users are not permanently added to broad device groups merely to make graphics or input work. |
| Applications | Applications receive user-level permissions only. Future sandboxed applications must not inherit compositor or system-service privileges. |

## 3. Filesystem hierarchy and ownership contract

The following is the intended installed-system policy. It must be implemented by package installation rules and verified against a built image; it must not be applied blindly to a development checkout.

| Path | Owner and mode | Purpose |
| --- | --- | --- |
| `/`, `/usr`, `/usr/bin`, `/usr/lib`, `/usr/share` | `root:root`; directories generally `0755`, regular vendor files `0644`, executables `0755` | System-owned runtime and immutable vendor payload. Users must not be able to replace executables or libraries. |
| `/etc` and system configuration | `root:root`; directories generally `0755`, non-secret files `0644` | Administrator-controlled configuration. Secrets must use tighter service-specific ownership and modes. |
| `/var/lib/<service>` | Dedicated service account; normally `0750` or stricter | Persistent private service state. Never shared writable state between unrelated daemons. |
| `/var/cache/<service>` | Dedicated service account; normally `0750` | Re-creatable service cache. |
| `/var/log/<service>` | Dedicated service account or root-managed logging; normally `0750` | Service logs, without world-readable sensitive data. |
| `/run/<service>` | Dedicated service account; normally `0750` | Volatile runtime state and sockets. Sockets belong under `/run`, not arbitrary writable directories. |
| `/home/<user>` | User-owned; default target `0700` unless a documented sharing feature requires otherwise | Private user data. |
| `/root` | `root:root`, `0700` | Administrator home. |
| `/tmp`, `/var/tmp` | `root:root`, sticky bit (`1777`) | Shared temporary storage. Mount options must be selected and tested for compatibility. |

These are policy targets, not instructions to recursively chmod a live filesystem. Package-specific exceptions must be explicit, narrow, documented, and tested. Set-user-ID/set-group-ID binaries and file capabilities require an inventory and justification.

## 4. Service isolation baseline

Every future systemd service must be reviewed individually. The default starting point is:

- dedicated `User=` and `Group=` or `DynamicUser=yes` where persistent identity is unnecessary;
- `NoNewPrivileges=yes`;
- `ProtectSystem=strict`, with only required state directories writable;
- `ProtectHome=yes` unless access to user data is a documented requirement;
- `PrivateTmp=yes`;
- `PrivateDevices=yes` unless specific device access is required;
- `ProtectKernelTunables=yes`, `ProtectKernelModules=yes`, `ProtectControlGroups=yes`, `ProtectKernelLogs=yes`;
- `RestrictSUIDSGID=yes`, `LockPersonality=yes`, `RestrictRealtime=yes`;
- a minimal `CapabilityBoundingSet=` and `AmbientCapabilities=`;
- restricted `RestrictAddressFamilies=` and `SystemCallFilter=` where compatible.

These settings are not to be copied indiscriminately into the compositor. A compositor needs session, DRM, input, graphics and IPC access. Its device descriptors must come from the active libseat session; a blanket `PrivateDevices=yes` or `DevicePolicy=closed` can break that design. Hardening is per unit and must be tested on real hardware and in CI.

Use `systemd-analyze security <unit>` as a review aid, not as proof of security. A score never replaces functional tests or threat analysis.

## 5. Authorization and privilege policy

- No desktop process may invoke unrestricted `sudo` for routine operation.
- Do not grant users permanent membership in `sudo`, `disk`, `input`, `video`, or similar groups solely to bypass a missing authorization design.
- Use libseat/logind for session-scoped device access.
- Use narrowly scoped polkit actions for user-approved administrative operations. Rules must identify the exact action, subject, and authorization requirement; never use broad allow-all rules.
- Do not introduce custom setuid-root executables.
- Linux capabilities are granted to a specific service or executable only when the required operation and threat boundary are documented.

## 6. Bootstrap requirements

The Debian bootstrap must:

1. Reject an existing target, a target inside the repository, invalid paths, and missing or malformed package manifests before privileged changes.
2. Keep the host unchanged apart from the explicitly selected target path.
3. Keep package installation and maintainer scripts inside systemd-nspawn.
4. Use a restrictive umask for bootstrap-created temporary data and explicit modes for installed configuration.
5. Avoid recursive host bind mounts and never expose host secrets, home directories, or writable system paths to the target container.
6. Minimize container capabilities and writable paths; any exception must be justified by a failing reproducible test.
7. Remove temporary policy files on success and failure without deleting pre-existing user data.
8. Verify the resulting root filesystem's ownership, modes, package state, and security-sensitive defaults before reporting success.

The current script already rejects an existing target and a target inside the repository, uses debootstrap's foreign stage, runs target package configuration in systemd-nspawn, and avoids recursive host `/dev` and `/sys` binds. This is useful isolation, but it is not yet a complete capability, namespace, mount, or permission audit.

## 7. Current gaps — must be closed before calling the base hardened

- No dedicated AureaOS system-service accounts or service units exist yet.
- The security package manifest installs nftables and polkit, but no AureaOS firewall policy or polkit action/rule set is implemented.
- The bootstrap does not yet verify the final rootfs ownership/mode policy or audit SUID/SGID files and file capabilities.
- The nspawn invocation drops selected capabilities, but does not yet demonstrate a minimal allowlist or a complete user/mount/network namespace policy.
- The target has no defined regular-user provisioning policy, home-directory mode policy, or sudo authorization policy.
- Secure Boot, kernel command-line policy, initramfs integrity, update trust, rollback protection, and recovery are not implemented by the current terminal-only bootstrap.

## 8. Required verification gates

A security change is accepted only when:

1. Static checks and shell tests pass.
2. Bootstrap negative tests prove that invalid, existing, and repository-contained targets are rejected without modification.
3. A fresh disposable Debian Trixie rootfs can be built successfully.
4. A permission audit checks ownership and modes, world-writable paths, SUID/SGID files, and file capabilities against an explicit allowlist.
5. Service hardening is tested per unit with both `systemd-analyze security` and functional tests.
6. A VM boot test confirms that security settings do not break startup, networking, graphics, input, or recovery.

A passing CI build is necessary but does not by itself prove the installed operating system is secure.
