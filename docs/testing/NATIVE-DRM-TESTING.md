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

## Interactive native test launch

From the repository root, run:

```bash
bash tools/run-aurea-native-test.sh
```

The launcher is interactive by default: it does not impose a 45-second deadline.
Aurea Welcome remains alive for the lifetime of the compositor session. Press
Ctrl+C in the launching TTY to request graceful shutdown. The launcher first
attempts to recover unfinished diagnostic sessions from the previous boot, then
builds the desktop binaries with the committed lockfile and starts the native
DRM/Vulkan desktop. Each session gets a private report directory under
`diagnostics/runs/`; the launcher captures stdout/stderr and a hardware,
driver, and Vulkan summary snapshot.
The desktop composition root starts and owns its Welcome Wayland client itself;
the launcher deliberately does not start a second client. The log records
Welcome connection, XDG configure, and first SHM frame submission, plus libinput
device add/remove metadata and rate-limited pointer motion/position samples.
The hardware snapshot includes the kernel input-device inventory, libinput device
inventory when available, seat status, input device nodes, DRM/GPU drivers, and
Vulkan summary. The client connects
to the compositor's `wayland-0` socket. The launcher enables Rust backtraces,
stores output under `$XDG_STATE_HOME/aureaos/logs` (or
`~/.local/state/aureaos/logs`), and waits for an interactive shutdown by default. With AUREA_TEST_TIMEOUT set,
it sends SIGINT at the deadline and allows 30 seconds for graceful shutdown before
sending SIGKILL. On normal exit it
collects only the current boot's kernel journal and Aurea Desktop process
journal entries, the persistent Aurea Desktop log entries for that session,
and Aurea Desktop coredump metadata when available. It does not export the
machine-wide system journal or copy core dump contents.

An optional bounded run can be requested without editing the script:

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
- After reboot, return to the repository and start the same launcher again.
  It automatically finds unfinished report folders and attempts to collect
  the previous boot's kernel and Aurea Desktop journal entries plus the
  matching persistent application log before starting another test. You can also invoke
  `bash tools/collect-previous-boot.sh RUN_ID` manually.

## Limits of the safety measures

The timeout bounds the compositor userspace process only while the kernel continues to
schedule it. The desktop owns and stops the Welcome client when the compositor
exits. It cannot prevent or recover from a kernel panic, GPU driver deadlock,
firmware hang, or complete hardware lockup. The persistent desktop log records
startup, normal termination, errors, and Rust panic details; the launcher
additionally captures compositor stderr/stdout, a hardware/driver snapshot,
time-bounded kernel and Aurea Desktop journal entries, and Aurea Desktop
coredump metadata when available. A hard power loss can still
truncate the final log entries.

A successful process start is not a successful display test. Record separately
whether libseat acquired the session, DRM/KMS initialized, a physical output
was registered, Vulkan presented a frame, and the Wayland Welcome client
connected and became visible.

## Pointer and touchpad acceptance test

Run this checklist during the first successful native DRM/Vulkan session. These
are physical acceptance checks; Cargo tests and a successful compositor start
do not replace them.

Record each item as **PASS**, **FAIL**, or **NOT AVAILABLE** in the session
report. Do not mark the pointer block complete while any required item is
unverified.

### Mouse

1. Confirm that the AureaOS arrow is visible on the active monitor immediately
   after startup, without moving the mouse first.
2. Move slowly and quickly across the full display. Confirm that the arrow
   follows continuously, does not disappear, and remains inside the active
   display.
3. On a multi-monitor setup, cross each monitor boundary in both directions.
   Include mixed-DPI displays if available.
4. Test left click, right click, middle click, and any side/back/forward buttons
   present on the device. Confirm that auxiliary buttons are not converted to
   primary or secondary clicks.
5. Perform single and double clicks on a known interactive target. Confirm
   that one physical click produces one activation and that double-click
   timing is consistent.
6. Test vertical and horizontal wheel scrolling, including a slow wheel turn
   and a rapid sequence.

### Touchpad

1. Move the pointer with one finger across the full usable touchpad area.
2. Test physical click, tap-to-click when supported, and two-finger scrolling.
3. Test each advertised gesture capability on that device. Record unsupported
   gestures as **NOT AVAILABLE**, not as failures.
4. Confirm that typing does not unexpectedly move or click the pointer when
   disable-while-typing is supported and enabled.
5. Disconnect and reconnect an external mouse while the touchpad remains
   active. Confirm that both devices continue to work and that the pointer
   does not become stuck.

### Reconnection and session recovery

1. Unplug and reconnect a USB mouse. Confirm that it is rediscovered and moves
   the same seat pointer.
2. Disconnect a device while a button is held. Confirm that no button remains
   logically stuck.
3. Switch to another TTY and return to AureaOS. Confirm that the pointer,
   buttons and touchpad settings recover after the libseat session resumes.

### Evidence to retain

Keep the session report directory and note the tested hardware, connection
type, monitor layout and DPI scale. Correlate the visual result with the
captured libinput device-added/device-removed messages and pointer motion
samples. A log entry showing input events is not, by itself, proof that the
cursor was visibly rendered or that the click reached the intended target.
