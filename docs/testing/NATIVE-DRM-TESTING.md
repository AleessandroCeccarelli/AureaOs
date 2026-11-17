# Native DRM/Vulkan test procedure

This procedure is for controlled physical-display tests of AureaOS on a dedicated
test machine. It does not guarantee recovery from a kernel, GPU, firmware, or
hardware lockup.

## Before the first native test

1. Update the working tree and build from the repository root.
2. Keep a second TTY available (for example, Ctrl+Alt+F3) and confirm that you
   can log in there before starting the compositor.
3. Save any work and disconnect displays that are not needed for the test.
4. Do not run the desktop with `sudo`; the compositor must acquire the seat
   through the configured libseat/logind session.
5. Confirm that the test account can access the required seat and DRM device.

## Bounded test launch

From the repository root, run:

```bash
bash tools/run-aurea-native-test.sh
```

The default test window is 45 seconds. The launcher builds with the committed
lockfile, enables Rust backtraces, stores output under
`$XDG_STATE_HOME/aureaos/logs` (or `~/.local/state/aureaos/logs`), and sends
SIGINT at the deadline. It allows 10 seconds for graceful shutdown before
sending SIGKILL.

A different bounded duration can be selected without editing the script:

```bash
AUREA_TEST_TIMEOUT=90 bash tools/run-aurea-native-test.sh
```

The permitted range is 5–600 seconds. A timeout exit status of 124 means the
test reached its deadline; 137 means forced termination was necessary. Neither
status proves that the GPU or kernel is healthy.

## Recovery

- If the desktop process is responsive, press Ctrl+C and allow it to shut down.
- If the display is still responsive but the desktop is not, switch to the
  previously verified TTY with Ctrl+Alt+F3, log in, and inspect the newest log.
- If the machine stops responding to all TTY switching and input, userspace
  timeout handling cannot recover it. Use the machine's documented hardware
  recovery procedure as a last resort. Do not assume SysRq is enabled.
- After reboot, inspect the newest file in
  `~/.local/state/aureaos/logs/` (or the configured XDG state directory) and
  the kernel journal before repeating the test.

## Limits of the safety measures

The timeout bounds a userspace process only while the kernel continues to
schedule it. It cannot prevent or recover from a kernel panic, GPU driver
deadlock, firmware hang, or complete hardware lockup. The persistent desktop
log records startup, normal termination, errors, and Rust panic details; the
launcher additionally captures compositor stderr/stdout. A hard power loss
can still truncate the final log entries.

A successful process start is not a successful display test. Record separately
whether libseat acquired the session, DRM/KMS initialized, a physical output
was registered, Vulkan presented a frame, and the Wayland Welcome client
connected and became visible.
