# Wayland surface lifecycle

## Scope

This document records the current lifecycle of the first native Wayland
application and the compositor's top-level surface path. It describes the
implementation as it exists today; it is not a claim that all desktop window
behaviour is complete.

## Welcome client

The standalone `aurea-welcome` binary is a regular Wayland client. Its
responsibilities are separated by file:

- `welcome/mod.rs`: connects to the Wayland display, binds required globals,
  creates the XDG toplevel, and owns the client state.
- `welcome/protocol.rs`: handles XDG ping/configure/close events and protocol
  dispatch.
- `welcome/buffer.rs`: creates the private SHM backing file, maps it, creates
  the `wl_shm_pool` and `wl_buffer`, and attaches the first frame.
- `welcome/painter.rs`: paints the welcome surface into the supplied pixel
  buffer.
- `welcome/typography.rs`: owns the small bitmap typeface used by the painter.

The client waits for the initial XDG configure before acknowledging it and
committing its first buffer. It keeps the pool, buffer, and mapping alive for
the client lifetime.

## Compositor surface path

1. The XDG shell handler registers a new toplevel with the window manager.
2. The window manager sends the initial configure and tracks the surface as
   unmapped until it receives a buffer.
3. On a surface commit, the compositor detects buffer assignment/removal.
4. Supported SHM buffers are validated and copied into compositor-owned memory
   while Smithay holds the SHM access callback. No borrowed client-memory
   slice escapes that callback.
5. A mapping transition updates window state and focus; ordinary content
   replacement does not repeatedly trigger a focus transition.
6. The render scene includes live mapped toplevels on the active workspace.
   The DRM/Vulkan output path composes and presents the scene.

## SHM validation guarantees

The snapshot path currently checks:

- supported ARGB8888 and XRGB8888 formats;
- positive dimensions and non-negative offsets;
- stride large enough for one complete pixel row;
- checked arithmetic for row sizes and pool offsets;
- final row bounds against the SHM pool length;
- a maximum packed snapshot size of 256 MiB;
- non-null pool memory before copying.

The snapshot is tightly packed and owned by the compositor. ARGB/XRGB native
little-endian byte order is converted to RGBA in place when required.

## Current limitations

- The Welcome client currently paints a fixed 720x440 buffer.
- A subsequent XDG configure does not yet allocate and attach a replacement
  buffer; the client currently keeps its first buffer until it exits.
- The Welcome client's `wl_buffer.release` handler is presently a no-op. This
  is safe for the current single-buffer, immutable-frame demonstration because
  the buffer is not rewritten or destroyed while the surface is using it.
  It is not a complete multi-buffer or animated-client strategy.
- Popup lifecycle, interactive move/resize, and full window decorations are
  not part of this first visible-surface milestone.

These limitations must be addressed before treating Welcome as a production
shell surface or adding clients that continuously redraw.

## Verification

Run the focused compositor tests and the full workspace suite:

```bash
cargo test -p aurea-compositor --locked
cargo test --workspace --locked
cargo fmt --all -- --check
```

The SHM layout tests cover tight and padded rows, invalid dimensions, negative
offsets, undersized stride, pool-boundary violations, the configured memory
limit, and ARGB/XRGB conversion. Native DRM/Vulkan presentation still requires
the separate physical-display procedure in
[`docs/testing/NATIVE-DRM-TESTING.md`](../testing/NATIVE-DRM-TESTING.md).
