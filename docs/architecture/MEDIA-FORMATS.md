<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Official Media Format Registry

- Status: **Official product policy**
- Version: 1.0
- Date: 2026-11-24
- Primary language: English
- Italian companion: [FORMATS-MULTIMEDIALI.md](../it/FORMATS-MULTIMEDIALI.md)
- Scope: formats accepted by AureaOS applications and system services. This registry is authoritative; implementation status is tracked separately.

## 1. Policy

AureaOS supports a deliberately selected set of media formats. A format is not supported merely because a third-party library can decode it. New formats require an explicit product decision, a Debian-compatible implementation, licensing and dependency review, resource limits, tests, and an update to both language versions of this registry.

The wallpaper system accepts **still raster images only**. Animated images and video files are not desktop wallpaper formats.

File extensions are hints for file pickers, not a security boundary. Importers must inspect file signatures/container metadata, validate decoded content, enforce dimension and memory limits, and reject malformed or unsupported input.

## 2. Official desktop wallpaper image formats

The following formats are approved for the AureaOS wallpaper format policy. “Approved” describes the product decision; it does not claim that every decoder has already been integrated or verified on Fedora and Debian.

| Format | Accepted extensions | Intended use | Implementation status |
|---|---|---|---|
| JPEG | `.jpg`, `.jpeg`, `.jpe` | Photographs and general-purpose images | Existing TurboJPEG path; local validation pending |
| PNG | `.png` | Lossless images, transparency and digital artwork | Decoder source present; build/test verification pending |
| BMP | `.bmp` | Bitmap compatibility and legacy images | Decoder source present; build/test verification pending |
| TIFF | `.tif`, `.tiff` | Professional photography, scans, publishing and archival images | Decoder source present; build/test verification pending |
| WebP (static) | `.webp` | Efficient web and general-purpose still images | Decoder source present; build/test verification pending |
| HEIF / HEIC (still image) | `.heif`, `.heic` | Modern camera and smartphone photographs | Approved; libheif still-image adapter pending |
| AVIF (still image) | `.avif` | Efficient modern still images | Approved; libheif/AV1 still-image adapter pending |

### Image handling rules

- JPEG, PNG, BMP, TIFF, WebP, HEIF/HEIC and AVIF are the complete approved wallpaper image-format list for this policy version.
- GIF is **not accepted**, including as a first-frame still image.
- APNG and animated WebP are rejected. The importer must not silently turn an animated asset into a wallpaper by selecting a frame.
- TIFF is a still-image format. If a TIFF container has multiple pages/images, the wallpaper importer uses the first page only. It does not play or cycle through pages.
- HEIF/HEIC and AVIF support means still-image decoding only. It does not authorize HEVC or AV1 video playback.
- Decoded wallpaper pixels are normalized to the compositor's validated RGBA8 input contract. High-bit-depth or HDR source preservation and color-managed HDR output are not implied by this initial wallpaper policy.
- Extensions alone never establish support. Content signatures and decoder capability must agree.

## 3. Explicitly not supported for desktop wallpapers

The following formats are outside the approved wallpaper list for version 1.0:

| Format / type | Decision |
|---|---|
| GIF | Rejected |
| APNG | Rejected |
| Animated WebP | Rejected |
| Video containers and codecs (including MP4, MOV, MKV, H.264, HEVC and AV1 video) | Rejected |
| SVG | Not accepted as a wallpaper input; vector rendering is a separate product decision |
| PDF | Not accepted as a wallpaper input; document rasterization is a separate product decision |
| Camera RAW formats | Not accepted in the initial wallpaper scope |
| JPEG 2000 | Not approved |
| OpenEXR | Not approved in the initial wallpaper scope |

This list is intentionally closed. Do not add fallback decoders or silently accept additional formats without a registry decision.

## 4. Audio and music formats

**No audio or music codec/container has been approved by this registry yet.**

This section is reserved for future product decisions covering, at minimum:
- music playback formats and containers;
- lossless and lossy audio codecs;
- metadata and cover-art handling;
- recording, streaming and system-sound requirements, if approved separately.

Do not infer audio support from the presence of a library, Linux system package, or another application's capability. Audio formats must be added here only after an explicit decision and compatibility/licensing review for Debian.

## 5. Other media categories

Additional categories (for example video playback, capture, streaming, subtitles, and professional media interchange) are **not approved by this document**. They require separate scope and architecture decisions before any codec or container is added.

## 6. Dependency and implementation requirements

- Debian 13 is the compatibility baseline. Fedora is the development and validation environment, not the product dependency authority.
- Keep codec-specific implementations inside `aurea-media` or another explicitly designated media component. The compositor consumes validated pixel buffers and must not own image codec dependencies.
- Prefer narrowly enabled decoder features. Do not install broad codec/plugin bundles or encoders when only decoding is required.
- Review upstream and transitive licenses, Debian package availability, runtime plugin discovery, security history, maintenance, footprint and ABI compatibility.
- Decode on bounded workers, never in a compositor frame/render callback.
- Apply input-size, dimension, pixel-count, allocation and output-byte limits. Reject malformed files and unsupported features deterministically.
- Record the exact decoder/library and package decisions in the relevant architecture document and development log.
- Update this English registry and its Italian companion in the same change whenever policy changes.

## 7. Change control

A new format may be added only when all of the following are recorded:
1. Product use case and exact extensions/container scope.
2. Static versus animated behavior, where applicable.
3. Debian-compatible decoder/library and required runtime packages.
4. License and transitive dependency review.
5. Resource limits and malformed-input behavior.
6. Test coverage and verification status on Debian and Fedora.
7. Matching updates to both registry language versions.

A codec being technically available is not, by itself, approval to ship it.
