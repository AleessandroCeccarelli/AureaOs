<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Screen Capture and Recording

**Status:** Architecture contract — planned, not yet implemented  
**Scope:** AureaOS Roma 1.0 desktop, shell, compositor, capture service and portal backend

## 1. Purpose

AureaOS must provide system-integrated screenshots and screen recording without compromising the Wayland security model, compositor responsiveness or the project's Vulkan-only rendering architecture.

Screenshot and recording share source discovery, authorization, geometry and session lifecycle. They must not become one monolithic subsystem: image export, video encoding and audio capture have separate pipelines and failure modes.

## 2. Component ownership

| Component | Responsibility | Must not own |
|---|---|---|
| Aurea Shell | Entry points, source-selection UI, global shortcut dispatch, recording indicator, pause/stop controls and user feedback | GPU readback, encoding, muxing or audio device ownership |
| Aurea Compositor | Authoritative Wayland surface/output state, source geometry, capture eligibility and authorized final-frame delivery | File dialogs, media encoding, audio mixing or persistent user policy |
| Aurea Capture Service | Session orchestration, authorization checks, source lifetime, cancellation, pipeline coordination and output handoff | Window management or a second graphics renderer |
| Video pipeline | Frame pacing, timestamps, bounded buffering, encoding and container muxing | Wayland permission decisions |
| Audio pipeline | PipeWire source selection, stream lifecycle, timestamps and optional track mixing | Screen-source authorization |
| XDG Portal backend | Sandboxed request mediation, user consent, source selection and narrowly scoped results | Unrestricted host capture access |

The service may initially be a modular part of the desktop workspace. A separate crate or daemon should be introduced only when its process boundary, privilege model and IPC contract are justified. Do not create speculative crates merely to make the architecture diagram look complete.

## 3. Capture sources and user experience

### Screenshots

Planned source types:
- Full display, with an explicit output choice on multi-display systems.
- Active window.
- User-selected window.
- User-selected rectangular area.

PNG is the initial target format. Additional formats require a concrete user need and validated export support.

### Screen recording

Planned source types:
- One selected display or a supported set of displays.
- Active or user-selected window.
- User-selected area, provided geometry can be tracked correctly for the entire session.

Audio choices are independent:
- No audio.
- System/output audio.
- Microphone.
- System audio plus microphone.

System audio and microphone must be independently authorized and represented as separate sources. When both are captured, preserve synchronization and support separate tracks when the selected container allows it. Mixing may be an explicit option, not an irreversible default.

The user controls start, pause/resume where implemented, and stop. The shell displays a persistent recording indicator and a direct stop action in Quick Settings. A recording must never run invisibly. Do not advertise controls or formats before their runtime path exists.

## 4. Frame acquisition and Vulkan boundary

The compositor is the authority for which outputs and surfaces exist and for their committed geometry. Capture must use the final AureaOS-composed frame for display capture; window and area capture additionally require reliable source geometry and clipping.

Native capture must:
- Reuse the existing Vulkan renderer and composition results.
- Avoid a second renderer and avoid introducing OpenGL.
- Use asynchronous GPU-to-CPU transfer where CPU encoding requires readback.
- Use a bounded staging-buffer pool/ring and explicit synchronization rather than blocking every present.
- Apply backpressure or controlled frame dropping when encoding cannot keep up; memory use must remain bounded.
- Timestamp frames from a monotonic clock and preserve frame order.
- Define cursor inclusion as an explicit capture mode.
- Handle output scale, transform, hotplug, window movement, resizing, destruction and subsurface changes.

A future zero-copy or DMA-BUF path may be evaluated, but must be justified by actual renderer, encoder and driver compatibility. It is not a prerequisite for the first safe implementation.

The Wayland `ext-image-copy-capture-v1` protocol may be evaluated as it matures and as Smithay support is verified. It is not a required dependency while staging. Do not build on deprecated `wlr-screencopy`.

## 5. Audio and media pipeline

PipeWire is the audio transport. WirePlumber manages device policy and routing. System audio is captured from an explicitly authorized monitor/loopback source; microphone capture uses a separately selected input source.

Do not assume the XDG ScreenCast portal video stream automatically includes system audio. Audio authorization and source acquisition are separate.

GStreamer is the planned encoding and muxing framework because it provides a reusable media pipeline, but the implementation must validate:
- Installed encoder and muxer capabilities.
- Hardware encoder availability and stability.
- Software fallback on supported hardware.
- Codec and container licensing and redistribution obligations.
- Audio/video timestamp synchronization and drift.
- Device removal, stream errors and graceful finalization.

Do not hard-code a codec as universally available. Select from tested capabilities. MP4 and Matroska are candidate containers; codec defaults remain a release decision after packaging and licensing review.

For simultaneous system and microphone capture, prefer separate audio tracks where supported. If a mixed track is offered, expose it as a deliberate user choice and define gain/clipping behavior.

## 6. Security and privacy requirements

- Every capture session requires an explicit user-mediated grant.
- Screen, microphone and system-audio permissions are distinct.
- Sandboxed applications receive only the selected stream or destination grant, never unrestricted access to compositor memory or host paths.
- The shell maintains an unmistakable recording indicator and a direct stop action for the whole active session.
- Stop and revocation must terminate source access and close the associated streams.
- Lock-screen and secure authentication surfaces must not be captured. Capture must stop or enter a defined safe suspended state before sensitive session UI is exposed.
- Treat client metadata, window titles, source requests and destination names as untrusted input.
- Use private, permission-restricted temporary storage; clean incomplete temporary output on cancellation and recover safely after process failure.
- Do not persist capture grants across sessions unless a future specification and explicit user-facing policy justify it.
- Avoid recording the capture-selection overlay before the user confirms the source. The recording indicator itself must remain visible on the desktop while recording.

## 7. XDG portal interoperability

The portal backend is the supported integration boundary for Flatpak and other sandboxed clients.

Implement XDG Screenshot and ScreenCast interfaces according to the versions actually supported by AureaOS. Use system-controlled source selection and return only contract-authorized results or PipeWire streams. Report unsupported source types, cursor modes, persistence and options accurately.

Use the document portal or an equivalent user-mediated destination flow for sandboxed output. Cancellation, denial, revocation, client exit and compositor/output loss are first-class outcomes.

Portal compatibility must be verified against real Flatpak clients; implementing an internal native capture action does not by itself establish portal compatibility.

## 8. Output, resource and failure policy

- Screenshot output: PNG initially.
- Recording output: candidate MP4 or Matroska, with codecs selected by validated runtime capability.
- Hardware encoding is an optimization, never a requirement for correctness.
- CPU, GPU and memory use must be bounded, with configurable quality/frame-rate limits where appropriate.
- Disk-full, permission-denied, source-disconnected, encoder-failed and application-exited cases must produce understandable errors.
- Finalization must flush and close the container when possible; incomplete output must not be mistaken for a valid completed recording.

## 9. Modular implementation direction

Keep each responsibility in a small module, following the AureaOS rule “one file, one responsibility”. Candidate internal boundaries (names are illustrative, not a mandate to create all files immediately):

- capture session lifecycle
- authorization and grants
- source discovery and geometry
- compositor frame adapter
- asynchronous Vulkan readback
- screenshot PNG export
- video frame scheduling and timestamps
- GStreamer encoder/muxer adapter
- PipeWire audio source adapter
- audio/video synchronization
- portal request handling
- output destination and temporary-file lifecycle

Do not place all capture logic in `aurea-compositor`, `aurea-shell` or `aurea-desktop-api`. The shared API should contain only contracts genuinely shared across components.

## 10. Verification requirements

Before declaring the feature ready, test at minimum:

- Full-display, active-window, selected-window and area geometry.
- Multi-monitor scale, transform, hotplug and removal.
- Window movement, resize, unmap and destruction during capture.
- Subsurface changes and cursor inclusion modes.
- Vulkan asynchronous readback, synchronization, backpressure and bounded memory.
- Frame timestamp ordering and audio/video drift.
- Microphone and system-audio permissions independently and together.
- Audio device removal and PipeWire restart.
- Cancellation, denial, revocation, client exit and service failure.
- Disk-full and encoder/muxer failure, including safe output finalization.
- Lock-screen and secure-surface exclusion.
- Portal behavior with real sandboxed clients.
- Keyboard-only operation and persistent stop access.

Compilation alone is not acceptance. Unit tests, portal integration tests and hardware validation are separate evidence.

## 11. Implementation status

The technology-independent `aurea-capture` crate now contains an initial implementation foundation:

- Capture request and target types for screenshot and recording operations.
- A single-session coordinator with explicit authorization, lifecycle transitions and separate screen/system-audio/microphone grants.
- Validated owned RGBA8/BGRA8 frame data with dimensions, stride and monotonic timestamp metadata.
- A non-blocking frame queue with both a frame-count limit and a strict queued-byte limit. New frames are rejected and counted when either limit is reached.
- Unit tests covering request/session behavior, frame validation, queue ordering, drop accounting and memory limits.

These primitives do not yet acquire screen pixels or audio. They do not implement a compositor adapter, PNG output, video encoding, PipeWire capture, portal backend or user interface.

The compositor now also exposes a generic asynchronous Vulkan readback submission primitive. It records an image-to-host-visible-buffer copy, submits it with a fence, supports non-blocking completion polling, and retains the source image view until the submission is retired. It deliberately does not perform image-layout transitions: the caller must provide an RGBA8 image in TRANSFER_SRC_OPTIMAL with TRANSFER_SRC usage. If queue acceptance or completion is uncertain, resources are conservatively retained rather than freed while the GPU may still reference them.

The original VulkanReadbackBuffer and offscreen renderer remain synchronous diagnostic helpers. The new asynchronous submission primitive is not yet wired into the compositor's final-scene path. The existing DRM scanout images are allocated for color-attachment use and are released to KMS through a synchronous composition lifecycle; capture integration must first negotiate a safe transfer-source strategy and coordinate GPU completion with KMS ownership. Do not simply add a readback copy to the current render path or block the presentation loop.

The compositor now also has a dedicated `VulkanCaptureTarget` allocator. It creates an optimal-tiled RGBA8 image with `COLOR_ATTACHMENT | TRANSFER_SRC` usage only after checking the physical device's optimal-tiling format features. This target is independent of DRM/KMS scanout storage and can create an owned image view for the asynchronous readback API. It is currently a low-level allocation primitive: no bounded capture-target pool, final-scene rendering integration, worker handoff or `aurea-capture` connection exists yet.

The Vulkan readback layer now includes a bounded staging-buffer pool with explicit worker-side preallocation, non-blocking checkout of matching idle buffers, RAII return, and limits based on the driver's reported Vulkan memory requirements. Pool saturation and lock contention are reported so a capture worker can skip a frame instead of waiting for capacity. The asynchronous submission retains its source image view and staging lease through fence completion. Shared logical-device command-buffer allocation and release are externally synchronized.

A separate readback coordinator now caps the number of in-flight submissions, polls fences without waiting, retires completed submissions and returns their RGBA8 frames in submission order. It stops at the first incomplete or failed submission so later frames cannot overtake earlier ones. When its limit is reached, it rejects a new frame immediately. Completed frames and per-submission errors are returned together; failed submissions remain owned by the coordinator for retry or safe shutdown.

These are low-level primitives only. The pool and submission are not yet connected to the technology-independent capture queue, the compositor's final-scene path, DRM/KMS ownership transitions, PNG export, video encoding, PipeWire or portals. The current pool tests validate dimension and byte-size arithmetic; Vulkan allocation, synchronization and hardware behavior still require build, unit/integration and device validation. Each subsequent layer must be delivered and verified independently before the UI advertises it as available.
