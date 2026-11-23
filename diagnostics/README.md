# AureaOS diagnostics

This directory stores opt-in diagnostic reports from native hardware tests.

## Native DRM/Vulkan test

From the repository root, in a normal login session on a local TTY (not as root):

```bash
bash tools/run-aurea-native-test.sh
```

The launcher first looks for unfinished sessions and automatically attempts
to recover their previous-boot journals and application logs. It then creates
a new directory under `diagnostics/runs/<YYYYMMDD-HHMMSS>/`. It records
application output in `native-test.txt`, a hardware/driver snapshot in
`system-snapshot.txt`, and technical session metadata in `run-info.txt`.
After a normal exit or timeout it automatically collects kernel diagnostics,
Aurea Desktop process messages, the persistent Aurea Desktop application log,
and an index of matching systemd coredumps when available. It does not export
the machine-wide system journal or copy core dump contents. The test remains time-bounded by `AUREA_TEST_TIMEOUT` (default:
45 s). If a run directory already exists for that second, the launcher stops
rather than overwriting or mixing reports.

## After a freeze or forced reboot

Boot Fedora normally and start the same launcher again. It automatically
detects run folders without a `finished_at` or `recovered_at` marker and
attempts previous-boot recovery before starting the next session. Manual
recovery remains available using the run id printed by the launcher:

```bash
bash tools/collect-previous-boot.sh 20261120-083000
```

The collector saves only the previous boot's kernel journal and messages
attributed to the Aurea Desktop process. It does not export the full system
journal. It first tries the current user's journal permissions, then asks for
sudo access if needed. Only `journalctl` runs with elevated privileges;
report files remain owned by the logged-in user.

It also extracts entries from Aurea Desktop's persistent application log
(`$XDG_STATE_HOME/aureaos/logs/aurea-desktop.log`, or
`~/.local/state/aureaos/logs/aurea-desktop.log`) bounded to the session
start and finish/recovery timestamps. These entries are saved as
`aurea-desktop-app.txt`. If `coredumpctl` is available, a time-bounded
`coredumps.txt` index is collected; core dump contents are not copied.

Persistent previous-boot journal data is available only if Fedora's systemd
journal retained it. This collector does not enable persistent journaling or
change system configuration.

## Diagnostic data scope

Reports are intended to contain technical diagnostics only: application
stdout/stderr, Aurea Desktop logs, kernel/DRM/GPU messages, PCI device and
driver identifiers, Vulkan summary, session type, kernel version, test
timestamps, exit status, and a coredump index. The launcher does not collect
documents, browser data, user files, passwords, tokens, private keys, or the
machine-wide system journal. Core dump contents are never copied.

Reports use text files rather than `.log` extensions so they are not silently
excluded by the repository's existing `.gitignore` rules. Review Git status
before committing; do not commit build artifacts or unrelated files.

## Limitations

A hard lock can prevent the kernel from flushing its final messages. No software
logger can guarantee capture of events that were never written to persistent
storage. The application log captures entries flushed by Aurea Desktop; the
previous-boot journal adds kernel and system context when available.
