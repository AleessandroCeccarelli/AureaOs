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
- `welcome/dimensions.rs`: validates and bounds compositor-provided dimensions.
- `welcome/frame.rs`: owns the buffer, pool, and mapped pixels for each submitted frame.
- `welcome/buffer.rs`: allocates SHM frames, attaches them, and retires resources
  after `wl_buffer.release`.
- `welcome/scaling.rs`: scales the fixed artwork to the configured buffer size.
- `welcome/painter.rs`: paints the reference Welcome artwork.
- `welcome/typography.rs`: owns the small bitmap typeface used by the painter.

The client waits for the initial XDG configure before acknowledging it and
committing its first buffer. Later XDG toplevel configure events update the
requested dimensions. Dimensions are bounded to 560×360 through 1920×1200;
zero-sized configure hints retain the 720×440 reference size. The artwork is
scaled into a newly allocated SHM frame when the resolved size changes. Each
submitted frame retains its pool and mapping until its matching
`wl_buffer.release` event arrives.

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

- The Welcome artwork is authored at 720×440 and nearest-neighbour scaled to
  the configured buffer dimensions; responsive layout and vector rendering are
  future work.
- Frame resources are retained until the compositor sends the corresponding
  `wl_buffer.release`. This is a single-frame-at-a-time redraw strategy, not a
  reusable multi-buffer animation pipeline.
- Popup lifecycle, interactive move/resize, and full window decorations are
  not part of this first visible-surface milestone.

These limitations must be addressed before treating Welcome as a production
shell surface or adding clients that continuously redraw. The current
resizing path handles XDG size changes but does not yet implement interactive
window move/resize requests or a full damage-tracked rendering loop.

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
