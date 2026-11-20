# AureaOS diagnostics

This directory stores opt-in diagnostic reports from native hardware tests.

## Native DRM/Vulkan test

From the repository root, in a normal login session on a local TTY (not as root):

```bash
bash tools/run-aurea-native-test.sh
```

The launcher writes a new directory under `diagnostics/runs/<YYYYMMDD-HHMMSS>/`.
It records the build and application output in `native-test.txt`, plus a small
`run-info.txt` containing non-secret environment indicators, the tested Git
revision, and the test start time. The test remains time-bounded by
`AUREA_TEST_TIMEOUT` (default: 45 s). If a run directory already exists for
that second, the launcher stops rather than overwriting or mixing reports.

## After a freeze or forced reboot

Boot Fedora normally, return to the same checkout, and run the collector using
the run id printed by the launcher:

```bash
bash bash tools/collect-previous-boot.sh 20261120-083000
```

The collector attempts to save the previous boot's kernel journal and system
journal into that run directory. It first tries the current user's journal
permissions, then asks for sudo access if needed. Only `journalctl` runs with
elevated privileges; report files remain owned by the logged-in user.

It also extracts entries from Aurea Desktop's persistent application log
(`$XDG_STATE_HOME/aureaos/logs/aurea-desktop.log`, or
`~/.local/state/aureaos/logs/aurea-desktop.log`) whose timestamps are at or
after the test start time. These entries are saved as
`aurea-desktop-app.txt`.

Persistent previous-boot journal data is available only if Fedora's systemd
journal retained it. This collector does not enable persistent journaling or
change system configuration.

## Before committing or pushing

Inspect the report locally. Logs can disclose usernames, hostnames, device
identifiers, filesystem paths, hardware details, and other machine-specific
information. Remove anything private before sharing. Never commit passwords,
tokens, private keys, or unrelated personal data.

Reports use text files rather than `.log` extensions so they are not silently
excluded by the repository's existing `.gitignore` rules. Review Git status
before committing; do not commit build artifacts or unrelated files.

## Limitations

A hard lock can prevent the kernel from flushing its final messages. No software
logger can guarantee capture of events that were never written to persistent
storage. The application log captures entries flushed by Aurea Desktop; the
previous-boot journal adds kernel and system context when available.
