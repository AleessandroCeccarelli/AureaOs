<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Aurea File Intelligence and Metadata Architecture

## Status

Architecture decision record — proposed foundation, not an implementation claim.

This document defines how AureaOS should discover, describe, index and expose user files to Fides, Quaero and the Desktop. It separates Linux interoperability from AureaOS-owned product behavior.

## Product principles

1. AureaOS must feel familiar without reproducing another desktop's visual language, internal architecture or proprietary behavior.
2. File information has one authoritative owner. UI components consume shared contracts and do not maintain competing indexes.
3. Do not write hidden sidecar files into user directories merely to remember presentation state.
4. Prefer existing Linux/FreeDesktop facilities where they solve the problem; add Aurea-owned behavior only where the platform leaves a real gap.
5. Indexing is incremental, resource-bounded, permission-aware and recoverable.
6. The index is derived data. User-authored metadata and user preferences are durable data and must not be lost when a derived index is rebuilt.
7. Every expensive extractor runs outside the compositor/render loop and outside latency-sensitive UI paths.
8. Network, removable, virtual and unsupported filesystems must degrade gracefully.

## Responsibility model

### Aurea File Intelligence

Aurea File Intelligence is the product-level capability responsible for maintaining a local catalog of files and exposing structured queries. It is not a file manager, desktop renderer or search UI.

Its responsibilities:

- Discover configured roots and exclusions.
- Perform bounded initial scans and incremental reconciliation.
- Identify file types using the system MIME database rather than filename extensions alone.
- Collect inexpensive filesystem facts.
- Schedule optional metadata extraction and content indexing by type and policy.
- Maintain a local query index and report freshness/coverage.
- Expose stable, permission-aware query and metadata contracts to desktop components.
- Handle removals, moves, inaccessible paths, mount lifecycle and index rebuilds.

Initial deployment should be an Aurea-owned library/service hosted by the desktop session composition root. Do not create a permanent daemon or D-Bus protocol before lifecycle, multi-process clients, isolation or measured workload justify it. Keep the API boundary transport-neutral so a dedicated user-session service can be introduced later without redesigning clients.

### Fides (Aurea Files)

Owns file operations and navigation: browse, copy, move, rename, delete, restore, open-with and user-facing property editing. It requests catalog data from File Intelligence but does not own a second index.

### Quaero

Owns search experience, ranking policy and result presentation. It queries File Intelligence and must show whether results are complete, still indexing or unavailable because of permissions/mount state. Search must not bypass filesystem access controls.

### Desktop

Owns desktop-specific item placement, selection, arrangement and presentation preferences. It does not catalog the whole filesystem and does not store folder view state in per-directory hidden files. Desktop positions are user/session configuration scoped to a stable item identity and output/layout context.

### Aurea Media

Owns bounded media probing, thumbnail generation and decoding adapters. It runs work on workers, enforces file-size/time/memory limits, and returns results to the catalog/cache. It does not decide which directories to index or how Quaero ranks results.

### Compositor

Owns Wayland surfaces, outputs, pointer routing and Vulkan composition. It must never crawl directories, parse documents, extract metadata or access the catalog from the render loop.

## Metadata taxonomy

Keep these classes distinct in both storage and APIs:

1. **Filesystem facts** — path/URI, file identity hints, size, timestamps, permissions, ownership and mount context. Query from the filesystem when freshness matters; cache only with explicit freshness semantics.
2. **Format-native metadata** — EXIF, XMP, ID3, container metadata and document properties. Read without modifying source files unless the user explicitly edits the file's native metadata.
3. **Extracted metadata** — MIME type, dimensions, duration, codec, page count, text content and generated preview references. Derived and rebuildable.
4. **User-authored metadata** — tags, ratings, descriptions, labels and favorites. Durable, backed up and never silently discarded during re-indexing.
5. **Presentation state** — per-folder view mode/sort preferences and desktop icon positions. Owned by the relevant UI component, not the global content index.
6. **Thumbnail/cache artifacts** — disposable, versioned, bounded cache entries. Never authoritative.

Do not assume inode numbers are globally stable. A file identity should combine mount/device context and filesystem identity where available, while retaining a URI/path fallback and handling replacement, rename and cross-volume moves. Content hashing is optional and on-demand; never hash every file during a normal scan.

## Storage direction

### Durable user data

Store Aurea-authored metadata and preferences under the user's XDG data/config locations, with clear backup and migration semantics. Never place Aurea-private metadata files inside arbitrary user folders by default.

### Derived catalog

Use a local embedded database as the leading candidate. SQLite is the preferred first candidate because it is mature, inspectable, transactional, portable and requires no database server. This is a candidate decision, not yet a committed dependency.

Before implementation, verify on Debian 13 and Fedora:
- SQLite library and development package availability.
- FTS5 availability in the selected system build.
- Rust binding maintenance, licensing, MSRV and static/dynamic linking behavior.
- WAL behavior and safe recovery on power loss.
- Migration and schema-version strategy.
- Database size and write-amplification under realistic workloads.

Keep the schema normalized for canonical metadata and use a dedicated full-text index only for explicitly indexed textual content. Do not put large binary previews or media payloads in database rows; keep them in a separate bounded cache.

### No per-directory metadata sidecars

Do not create a hidden metadata file in every folder. Folder presentation state should live in the user's Aurea configuration/data store, keyed by a normalized folder URI and validated on access. If interoperability later requires reading/writing another desktop's folder metadata, implement it as an explicit compatibility layer, never as the canonical Aurea format.

## Linux interoperability

Use and respect existing Linux conventions:

- XDG Base Directory for config, data, cache and state placement.
- Shared MIME-info for MIME identification and descriptions.
- Desktop Entry and Icon Theme conventions for application discovery and icons.
- FreeDesktop Trash specification for trash storage and restore interoperability.
- Thumbnail Managing Standard and compatible thumbnailer conventions where applicable.
- File URIs and existing Wayland data-transfer protocols for file references and drag-and-drop.
- Filesystem permissions and mount semantics as the security boundary.

Do not depend on GNOME Tracker, KDE Baloo, GTK, Qt or another desktop environment's private database/service. They may be useful research references, not runtime dependencies.

The FreeDesktop shared-file-metadata specification is not treated as a universally deployed stable contract. Revisit its status before promising interoperability.

## Discovery and change tracking

Use a two-part model:

1. **Reconciliation scanner** — authoritative discovery of configured roots, with bounded batches, cancellation, mount awareness and persisted progress.
2. **Change event adapter** — Linux-native notifications for low-latency updates.

A Rust watcher crate such as notify may be evaluated, but do not adopt its newest major version automatically. Pin a stable release compatible with the workspace MSRV and inspect its Linux backend, licensing and known limitations. A narrow native inotify adapter is also an option if it provides a materially better lifecycle/resource model.

Notifications are hints, not a source of truth. Coalesce bursts, handle watch invalidation/overflow, re-register after directory replacement, and schedule reconciliation after gaps, resume, mount changes or detected inconsistency. Network filesystems may require explicit polling fallback. Avoid recursively watching every file if directory-level watches and bounded reconciliation are sufficient.

Do not start with fanotify permission interception or privileged kernel monitoring. Those mechanisms increase privilege and failure complexity and are not needed for a user-owned desktop catalog.

## Indexing policy and resource budget

- Default roots should be explicit user locations, not the entire filesystem.
- Exclude system trees, cache directories, hidden implementation directories and development/build trees by default; allow user overrides.
- Do not traverse symlinks outside configured roots unless the user explicitly enables it.
- Removable and network volumes are opt-in or policy-controlled; report their indexed/online state.
- Apply per-file size, time, memory and recursion limits to extractors.
- Prioritize metadata needed for visible Fides views and recent Quaero queries; defer expensive full-text extraction.
- Pause or reduce work on battery, high system load, low free space or explicit user request.
- Provide controls to inspect indexed locations, exclusions, status, pause/resume and rebuild.
- Respect permissions at discovery, extraction and query time. An index hit must never grant access to the underlying file.
- Treat malformed files and hostile media/document inputs as untrusted. Extractors must be isolated from the compositor and have strict resource limits.

## Query and freshness contract

Queries should return structured records with:
- Stable Aurea item identifier.
- URI/path and best available filesystem identity.
- MIME type and typed metadata.
- Match/ranking explanation fields where appropriate.
- Last observed/verified timestamp and freshness state.
- Availability state (online, offline, inaccessible, stale, pending extraction).
- Thumbnail reference only when a valid cached artifact exists.

A missing index result must not be presented as proof that a file does not exist. Clients should be able to request a direct filesystem fallback for the current folder, while clearly distinguishing live results from catalog results.

## Future-facing capabilities

Keep extension points for:
- Semantic and natural-language search over locally indexed content, opt-in and privacy-preserving.
- Content embeddings or vector retrieval as a separate optional index, never a prerequisite for ordinary filename search.
- Per-user encrypted/private indexes where threat model and key lifecycle are defined.
- Content providers beyond local files (document portals, cloud providers, removable media) through explicit provider interfaces.
- Metadata provenance and confidence, so extracted facts are distinguishable from user edits and format-native values.

Do not introduce a vector database, machine-learning runtime, cloud index or permanent AI process in the initial desktop. Establish the typed metadata and provider boundaries first; add advanced retrieval only when there is a measurable user benefit and hardware budget.

## Implementation phases

1. Finalize the public item/metadata/query contracts and stable identity rules.
2. Audit existing Aurea file, desktop, search and media modules to avoid duplicate ownership.
3. Prototype database and FTS5 availability on Debian 13 and Fedora without adding a runtime dependency yet.
4. Implement a scanner and reconciliation state machine with synthetic filesystem tests.
5. Add change-event integration with overflow/recovery behavior.
6. Add low-cost metadata and MIME extraction.
7. Add bounded thumbnail/media extractors through Aurea Media.
8. Connect Fides and Quaero; add Desktop-specific presentation state separately.
9. Add optional content indexing and future semantic capabilities only after profiling.

## Explicit non-goals

- No clone of macOS per-directory metadata files.
- No dependence on GNOME Tracker, KDE Baloo or their private APIs.
- No full-disk indexing by default.
- No privileged fanotify-based surveillance.
- No mandatory cloud service, vector database or AI model.
- No metadata extraction in the compositor or Vulkan render loop.
- No implementation/build/test in this architecture decision record.

## References

- FreeDesktop specifications: https://www.freedesktop.org/specifications/
- Shared file metadata specification: https://www.freedesktop.org/specifications/shared-filemetadata-spec/
- Rust notify documentation: https://docs.rs/notify/latest/notify/
- Linux inotify documentation: https://man7.org/linux/man-pages/man7/inotify.7.html
