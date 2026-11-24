<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Aurea Link and Aurea Share — Cross-device sharing architecture

- Status: Product direction and architecture proposal; no implementation or dependency is approved here.
- Target: AureaOS Roma 1.0 foundations and subsequent companion-app releases.
- Compatibility authority: Debian GNU/Linux 13 (Trixie), amd64.
- Product rule: simple by default, explicit consent, local-first, no mandatory cloud account.

## 1. Product goal

AureaOS should exchange files, media and links naturally with Android phones, iPhone/iPad, macOS and other Linux/Windows devices. Users can select a photo, audio file, video, document, link or text and send it to a nearby paired device, or receive it into a clear destination.

Separate two workflows:
- **Device handoff:** transfer directly between the user's devices, preferably over the local network, without a cloud account.
- **App sharing:** hand selected content to an installed application such as Telegram or WhatsApp. AureaOS opens the registered app/target; the user reviews and confirms recipient and message there. AureaOS does not automate private messaging APIs.

## 2. Product components

### Aurea Link — device connectivity and transfer

Owns user-approved pairing and peer identities, local discovery, transfer sessions, consent, progress, cancellation, integrity verification, safe receive destinations, privacy controls and protocol adapters behind a stable Aurea API.

Initial scope: files, multiple files, directories where safe, links and plain text. Preserve original file bytes; never silently transcode media. Previews are optional and resource-bounded.

### Aurea Share — system-wide share experience

Owns the user-facing Share action and destination registry, not the transport protocol. It can send selected content to Aurea Link devices, open compatible installed apps such as Telegram/WhatsApp, expose a consistent action in Fides and first-party apps, and receive content from registered share targets/companion apps.

Do not assume every Linux desktop app supports a universal share-sheet API. Start with Aurea-owned apps and explicit app adapters/desktop entries. Keep XDG Desktop Portal for file access and sandbox permission boundaries; FileChooser is not a share portal.

## 3. Interoperability strategy

### First implementation target: LocalSend Protocol v2.x

Evaluate and implement interoperability with the published LocalSend REST protocol instead of inventing a proprietary wire protocol for the first release. It is designed for local-network transfers without external servers and supports discovery plus HTTP(S) transfer flows.

- Review current v2.2 specification and upstream implementation before coding.
- Prefer HTTPS app-to-app transfer with authenticated certificate identity/pinning and explicit receiver approval.
- Verify checksums when supplied; stream with strict resource limits instead of buffering large media in memory.
- Treat discovery announcements as untrusted hints, never as proof of identity or authorization.
- Support multicast where available and a visible manual IP/QR pairing fallback when discovery is blocked by Wi-Fi isolation, VPNs or network policy.
- Version protocol support and compatibility tests explicitly.
- Check current upstream license, security posture, protocol changes and Debian runtime requirements before code reuse. Protocol interoperability does not mean copying or vendoring LocalSend application code.

LocalSend is a cross-platform compatibility baseline for Linux, macOS, Windows, Android and iOS, not the AureaOS UI or a mandatory bundled application.

### Secondary interoperability

- Evaluate KDE Connect as an optional peer ecosystem, especially for iOS/Android share-sheet workflows, clipboard and media-control use cases. Do not make KDE/Qt desktop integration an AureaOS dependency.
- Bluetooth OBEX or USB import may be fallback workflows, not the primary large-media transport.
- Apple AirDrop/AWDL remains research-only. OWL and reverse-engineered implementations have hardware/driver limitations and experimental status; do not promise AirDrop compatibility or make it a release dependency.
- A future Aurea Link mobile companion should integrate Android Sharesheet (ACTION_SEND / ACTION_SEND_MULTIPLE) and iOS Share Extensions so users can choose Aurea Link from native share menus in Photos, Files, Telegram or WhatsApp, subject to platform extension/background limits.
- A macOS companion may be evaluated later. Initially interoperate with existing LocalSend clients on macOS rather than requiring a proprietary Mac app.

## 4. Security and privacy

- Pairing is explicit. A discovered device is not automatically trusted.
- Before acceptance, show sender identity, file names, count, total size and destination.
- Prompt for unknown devices by default. Trusted-device policy must be configurable and revocable.
- Never silently overwrite files. Use collision-safe names or ask.
- Validate filenames, paths, MIME metadata and sizes; prevent traversal, symlink escape and special-file creation.
- Stream into a controlled temporary file, verify length/hash, then atomically publish the completed file.
- Enforce per-file, per-batch, concurrent-session, disk-space and rate limits. Support cancellation and cleanup after interruption.
- Use TLS with authenticated peer identity for app-to-app transfers. Never silently downgrade to unauthenticated HTTP.
- Do not expose an unrestricted LAN file server. Bind only required interfaces, use short-lived session tokens and close listeners when idle.
- No message, notification, clipboard or contact access without separate explicit permission.
- Never auto-open received content. The user decides whether to preview or open it.

## 5. Architecture boundaries

Proposed logical modules; these do not imply separate daemons:
- aurea-link-core: peer model, pairing, transfer state machine, validation and stable API.
- aurea-link-localsend: LocalSend protocol adapter and discovery/transfer implementation.
- aurea-link-storage: safe temporary receive, atomic publication and destination policy.
- aurea-share: content descriptors/intents and registered destination resolution.
- aurea-share-ui: Human Interfaces share sheet and incoming-transfer UI.
- Optional mobile companion: thin platform integration around the same protocol and explicit share extension/intent handling.

Start with small Rust libraries and a narrowly scoped session component only if lifecycle/security requires it. Do not add a monolithic always-on daemon before there is a real need. Network listeners activate only while receiving is enabled or a transfer is active.

Aurea Link must not become part of the compositor, render thread, Aurea Media decoder or GStreamer pipeline. Media bytes are transferred as files; playback and decoding remain separate responsibilities.

## 6. User experience

- One obvious Share action in Fides and first-party media/document apps.
- Nearby devices appear with clear names and device types; users can rename or forget them.
- Send: choose destination → inspect selection → send → progress → completion.
- Receive: visible request → inspect sender/content → accept/reject → progress → safe destination.
- Offer QR/manual pairing when multicast discovery is unavailable.
- Use plain language, large targets, keyboard navigation, screen-reader labels, clear progress and recoverable errors.
- Users should not need to understand IP addresses, ports, TLS certificates or protocol versions.
- Local transfer works offline and does not use cloud services by default.

## 7. Phased delivery

1. Protocol/threat-model review: confirm v2.x behavior, license, security posture, discovery constraints and Debian packages. No dependency yet.
2. Headless interoperability prototype: send/receive small files and large videos against official LocalSend clients on Fedora and Debian Trixie; test rejection, cancellation, hash mismatch, malformed metadata, network changes and restart.
3. Aurea Link core: pairing, identity, transfer state machine, bounded streaming, safe storage and diagnostics without content collection.
4. Aurea Share desktop integration: Fides and first-party app action; Telegram/WhatsApp destinations only when installed and supported.
5. Mobile companion: Android Sharesheet and iOS Share Extension; validate background restrictions and app-store requirements.
6. Advanced continuity: clipboard sync, notification relay, remote controls, media handoff or AirDrop interoperability only as separate opt-in designs with independent security review.

## 8. Acceptance criteria

- Works between AureaOS and at least one maintained LocalSend client on Android, iOS and macOS; record exact tested versions.
- Handles single/multiple files, large video, interrupted transfer, duplicate names, low disk space, rejection and device disappearance.
- Verifies integrity and never exposes partial files as complete.
- No transfer without user consent; discovery alone never establishes trust.
- Works on Debian 13 without GNOME/KDE session dependencies, X11/XWayland or OpenGL.
- No media transfer blocks compositor or rendering.
- Accessible by keyboard and screen reader, with readable progress and recoverable errors.
- No cloud account or Internet connection required for local-network transfer.

## 9. Explicit non-goals for the first milestone

- Reimplement Telegram, WhatsApp, AirDrop or the entire KDE Connect feature set.
- Read/sync private messages, contacts, notifications or clipboard by default.
- Background synchronization of entire photo libraries.
- Cloud relay, account system or remote Internet transfer.
- Live audio/video streaming between devices; first scope is file transfer and link/text handoff.
- Bundle third-party application code without separate license and maintenance review.

The first milestone is deliberately narrow: reliable, secure, cross-platform local file handoff with a coherent AureaOS share experience.
