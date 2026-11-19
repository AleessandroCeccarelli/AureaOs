# Native DRM/Vulkan test procedure

This procedure is for controlled physical-display tests of AureaOS on a dedicated
test machine. It does not guarantee recovery from a kernel, GPU, firmware, or
hardware lockup.

## Automated checks before native testing

Before testing on a physical display, run the same checks used by CI:

```bash
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --locked
cargo check -p aurea-compositor --features drm --locked
cargo test -p aurea-compositor --features drm --locked
shellcheck tools/run-aurea-native-test.sh
```

The SHM validation and surface lifecycle are described in
[`docs/architecture/WAYLAND-SURFACE-LIFECYCLE.md`](../architecture/WAYLAND-SURFACE-LIFECYCLE.md).
The automated suite validates software behaviour; it cannot confirm that a
physical DRM connector is active or that a frame is visible on the target
monitor.

## Before the first native test

1. Update the working tree and build from the repository root.
2. Keep a second TTY available (for example, Ctrl+Alt+F3) and confirm that you
   can log in there before starting the compositor.
3. Save any work and disconnect displays that are not needed for the test.
4. Do not run the desktop with `sudo`; the compositor must acquire the seat
   through the configured libseat/logind session.
5. Confirm that the test account can access the required seat and DRM device.
6. Start from a text TTY after ending the XFCE graphical session. Do not launch
   this native DRM test from inside the active graphical desktop.

## Bounded test launch

From the repository root, run:

```bash
bash tools/run-aurea-native-test.sh
```

The default test window is 45 seconds. The launcher builds the desktop
binaries with the committed lockfile and starts the native DRM/Vulkan desktop.
The desktop composition root starts and owns its Welcome Wayland client itself;
the launcher deliberately does not start a second client. The client connects
to the compositor's `wayland-0` socket. The launcher enables Rust backtraces,
stores output under `$XDG_STATE_HOME/aureaos/logs` (or
`~/.local/state/aureaos/logs`), and sends SIGINT at the deadline. It allows
10 seconds for graceful shutdown before sending SIGKILL.

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

The timeout bounds the compositor userspace process only while the kernel continues to
schedule it. The desktop owns and stops the Welcome client when the compositor
exits. It cannot prevent or recover from a kernel panic, GPU driver deadlock,
firmware hang, or complete hardware lockup. The persistent desktop log records
startup, normal termination, errors, and Rust panic details; the launcher
additionally captures compositor stderr/stdout. A hard power loss can still
truncate the final log entries.

A successful process start is not a successful display test. Record separately
whether libseat acquired the session, DRM/KMS initialized, a physical output
was registered, Vulkan presented a frame, and the Wayland Welcome client
connected and became visible.
