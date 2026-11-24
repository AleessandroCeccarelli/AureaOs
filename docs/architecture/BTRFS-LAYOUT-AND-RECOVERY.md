<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Btrfs Selection and System Disk Layout

## Status

Approved filesystem and partitioning direction. This document does not specify installer implementation or unapproved Btrfs feature tuning.

## Why Btrfs

Btrfs is selected as AureaOS's modern Linux filesystem. It is not selected to introduce a snapshot-based backup, update or rollback architecture.

## Installed disk layout

The intended installed layout is:
- One EFI System Partition (ESP), formatted for UEFI firmware.
- One Btrfs partition containing the complete installed operating system.
- No separate partitions for `/home`, `/var`, cache, snapshots or backup.
- No XBOOTLDR partition in the current systemd-boot design.

The exact ESP size, filesystem mount options and installer behavior require validation. The Btrfs partition is the sole system filesystem partition; external backup storage is not part of the system disk layout.

## Backup and restore ownership

Aurea Time is the only planned AureaOS backup and restore system. It is intended to support versioned, time-based backups to external USB storage or a NAS and recovery from individual files and folders through settings and the complete operating system.

An independent Aurea Recovery environment must be able to start without relying on the installed OS and restore it using an authorized Aurea Time backup. The detailed storage format, consistency model, retention, encryption policy and recovery procedure remain open design work.

## Explicit exclusions

- Native Btrfs snapshots are not the planned backup or restore mechanism.
- No snapshot-based update transaction or rollback workflow is planned.
- No dedicated snapshot or backup partition is planned.
- No separate `/home`, `/var` or cache partition is planned.

Btrfs capabilities such as subvolumes, compression, scrub, quotas, RAID, encryption and swap/hibernation are not selected by this document. They require separate evaluation and explicit approval.

## Related architecture

- [systemd-boot and UKI](SYSTEMD-BOOT-AND-UKI.md)
- Aurea Time owns backup history and restore behavior.
- Aurea Recovery provides the independent environment for full-system recovery.

This document records architecture decisions; it does not claim that installation, backup or recovery is implemented.
