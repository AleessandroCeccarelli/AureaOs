<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Btrfs Layout and Recovery Design

## Status
Proposed architecture for review. This is not an installer specification or an implemented recovery system.

## Goals
- Roll back operating-system state deliberately.
- Preserve user documents and personal data during system rollback.
- Keep package-manager state consistent with installed system files.
- Preserve diagnostic logs across failed updates.
- Make snapshot coverage explicit.
- Avoid nested subvolumes initially.
- Treat snapshots as local recovery points, never backups.

## Proposed initial layout

Use flat sibling subvolumes below the Btrfs top-level tree (subvolume ID 5). Mount the selected root at / and other subvolumes at explicit paths. Do not nest subvolumes.

| Subvolume | Mount point | Policy | Reason |
|---|---|---|---|
| @root | / | Snapshot and rollback | OS files, /usr, /etc, /opt, /var/lib, package database and system Flatpak state remain one consistent system state |
| @home | /home | Preserve by default | User documents and personal settings must not be silently reverted |
| @snapshots | /.snapshots (candidate) | Preserve | Snapshot catalog lives outside the root snapshot |
| @var-log | /var/log | Preserve | Failed-update diagnostics survive rollback |
| @var-cache | /var/cache | Regenerable; policy to define | Cache is generally rebuildable and need not be restored with the OS |
| EFI System Partition | /boot/efi (if selected) | Outside Btrfs snapshots | Firmware boot files reside on a separate FAT partition |

Keep /var/lib inside @root. It contains package-manager and service state that must match the system snapshot. In particular, Debian's dpkg database must roll back with the installed system files. System-wide Flatpak installation state follows the same consistency rule; per-user Flatpak data remains under home.

The location of /boot and bootloader integration are open. If /boot is inside @root, kernel and initramfs files are included in root snapshots. The EFI System Partition remains outside Btrfs snapshots and needs its own recovery strategy.

The proposed /.snapshots mount is not a dependency on a particular snapshot manager. Snapshot storage must not be nested under @root.

## Why a flat layout
Btrfs snapshots are not recursive. A nested subvolume forms a snapshot boundary; its contents do not appear in the parent snapshot. A flat layout makes coverage explicit and avoids accidental omissions. The system manager must coordinate any additional subvolumes intentionally included in an update transaction.

## Update transaction lifecycle
An update must be a transaction with explicit states, not a shell script that happens to create a snapshot.

1. **Preflight:** verify filesystem identity, expected subvolume layout, free-space headroom, snapshot destination, boot mode and package-manager state.
2. **Quiesce:** prevent overlapping update transactions; coordinate services holding databases or mutable system state where needed.
3. **Snapshot:** create a read-only pre-update snapshot of @root. Record subvolume ID/UUID, timestamp, package transaction identity and boot configuration context.
4. **Apply:** run the approved Debian package transaction against the active root.
5. **Validate:** verify package-manager completion, required boot artifacts, initramfs generation and critical configuration.
6. **Commit:** mark the new generation current only after validation; retain the pre-update snapshot according to policy.
7. **Failure:** leave the pre-update snapshot untouched, record diagnostics and offer controlled recovery.

Package-manager integration, transaction locking, validation criteria and atomic boot selection still need detailed design.

## Rollback model
Rollback must run from a recovery environment or equivalent context where the target root is not active.

It must:
- identify snapshots through stable metadata, not display names alone;
- verify filesystem and subvolume identity before mutation;
- preserve the failed current root as a diagnostic point when space permits;
- create a new writable root from the selected read-only snapshot instead of mutating the original snapshot;
- update boot selection through tested bootloader integration;
- keep boot/EFI artifacts consistent with the restored root;
- record the operation and result;
- provide a fallback boot/recovery entry.

Never silently roll back /home. A future user-data recovery feature needs preview and explicit confirmation.

## Retention and storage pressure
Retention is undecided. It must consider at least one known-good recovery point, pre-update snapshots with package transaction identity, optional manual snapshots, count/age limits, free space, metadata pressure and safe cleanup. Never delete the active root or the only recovery point.

If preflight cannot guarantee enough space, stop before package mutation. Snapshot creation failure must never be ignored.

## Backups, encryption and integrity
- Snapshots are not backups: filesystem or device damage can affect both live data and snapshots.
- A separate backup/export strategy is required. Btrfs send/receive may be evaluated, but is not a complete backup policy.
- Do not assume Btrfs provides transparent native data-at-rest encryption. Evaluate LUKS2/dm-crypt as a separate installation and recovery layer. TPM-assisted unlock requires an explicit threat model and recovery-key design.
- Define scrub scheduling, device health reporting and degraded-filesystem behavior separately.
- Do not promise self-healing or redundancy without a supported multi-device profile and tested policy.

## Mount and performance policy
Validate exact mount options on target hardware and workloads. Evaluate compress=zstd, noatime/relatime, discard policy and SSD behavior. Do not set nodatacow globally: it changes checksumming and copy-on-write behavior.

Swapfile and hibernation need separate design and compatibility tests. Do not create a Btrfs swapfile by default until constraints, resume configuration and recovery are validated.

## Installer and recovery requirements
The installer must identify the target disk and destructive actions, create the top-level filesystem and named subvolumes deterministically, install documented mount and bootloader configuration, record a machine-readable layout/version manifest, verify mounts and subvolume identities before first boot, and provide recovery if installation is interrupted.

Recovery tooling must inspect filesystem UUID, subvolume UUID/ID and mount topology. A subvolume name alone does not prove identity.

## Open decisions
- GRUB versus systemd-boot and snapshot boot-entry integration.
- Whether /boot remains inside @root and exact ESP layout.
- @var-cache policy and whether further /var separation is justified.
- LUKS2 defaults and recovery-key UX.
- Snapshot retention limits and free-space thresholds.
- Swapfile and hibernation support.
- Supported single-disk and multi-device profiles.
- Dedicated update transaction service and package-manager API ownership.

## Implementation sequence
1. Validate subvolume graph and mount table on Debian 13.
2. Specify bootloader and recovery-environment behavior before rollback implementation.
3. Define update transaction states, locks, failure handling and validation.
4. Define snapshot metadata and retention.
5. Specify encryption, backup, scrub and storage-health policies.
6. Implement installer/recovery support only after contract review.
7. Test interrupted updates, failed boot, low disk space, rollback, EFI mismatch and user-data preservation on disposable systems.

## References
- Btrfs Subvolumes: https://btrfs.readthedocs.io/en/latest/Subvolumes.html
- btrfs-subvolume(8): https://btrfs.readthedocs.io/en/latest/btrfs-subvolume.html

No installer, snapshot manager, update transaction or rollback implementation is claimed here.
