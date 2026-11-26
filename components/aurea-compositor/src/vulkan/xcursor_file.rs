// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounds-checked reader for the standard little-endian Xcursor file format.

use std::{error::Error, fmt};

const FILE_MAGIC: &[u8; 4] = b"Xcur";
const FILE_HEADER_BYTES: usize = 16;
const TOC_BYTES: usize = 12;
const IMAGE_CHUNK_TYPE: u32 = 0xfffd_0002;
const IMAGE_HEADER_BYTES: usize = 36;
const MAX_CURSOR_FILE_BYTES: usize = 16 * 1024 * 1024;
const MAX_CURSOR_DIMENSION: u32 = 256;
const MAX_CURSOR_FRAMES: usize = 128;

/// A decoded Xcursor image frame, normalized to tightly packed RGBA8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct XcursorFrame {
    pub width: u32,
    pub height: u32,
    pub hotspot_x: u32,
    pub hotspot_y: u32,
    pub delay_ms: u32,
    pub rgba8: Vec<u8>,
}

/// Errors returned for malformed, unsupported or oversized Xcursor files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum XcursorFileError {
    TooLarge,
    Truncated,
    InvalidMagic,
    InvalidHeader,
    UnsupportedVersion,
    InvalidTableOfContents,
    InvalidImageChunk,
    InvalidDimensions,
    InvalidHotspot,
    InvalidPixelData,
    NoImages,
}

impl fmt::Display for XcursorFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Xcursor file error: {self:?}")
    }
}

impl Error for XcursorFileError {}

/// Decodes all image chunks from an Xcursor file, preserving table order.
pub(crate) fn decode_xcursor(bytes: &[u8]) -> Result<Vec<XcursorFrame>, XcursorFileError> {
    if bytes.len() > MAX_CURSOR_FILE_BYTES {
        return Err(XcursorFileError::TooLarge);
    }
    if bytes.len() < FILE_HEADER_BYTES {
        return Err(XcursorFileError::Truncated);
    }
    if bytes.get(0..4) != Some(FILE_MAGIC.as_slice()) {
        return Err(XcursorFileError::InvalidMagic);
    }

    let header_bytes = read_u32(bytes, 4)? as usize;
    let version = read_u32(bytes, 8)?;
    let toc_count = read_u32(bytes, 12)? as usize;
    if header_bytes < FILE_HEADER_BYTES || header_bytes > bytes.len() {
        return Err(XcursorFileError::InvalidHeader);
    }
    if version >> 16 != 1 {
        return Err(XcursorFileError::UnsupportedVersion);
    }
    if toc_count > MAX_CURSOR_FRAMES {
        return Err(XcursorFileError::InvalidTableOfContents);
    }
    let toc_end = header_bytes
        .checked_add(toc_count.checked_mul(TOC_BYTES).ok_or(XcursorFileError::InvalidTableOfContents)?)
        .ok_or(XcursorFileError::InvalidTableOfContents)?;
    if toc_end > bytes.len() {
        return Err(XcursorFileError::InvalidTableOfContents);
    }

    let mut frames = Vec::new();
    for index in 0..toc_count {
        let offset = header_bytes + index * TOC_BYTES;
        let chunk_type = read_u32(bytes, offset)?;
        let _subtype = read_u32(bytes, offset + 4)?;
        let position = read_u32(bytes, offset + 8)? as usize;
        if chunk_type != IMAGE_CHUNK_TYPE {
            continue;
        }

        let chunk_header = read_u32(bytes, position)? as usize;
        let chunk_type = read_u32(bytes, position + 4)?;
        let chunk_version = read_u32(bytes, position + 12)?;
        if chunk_header < IMAGE_HEADER_BYTES
            || chunk_type != IMAGE_CHUNK_TYPE
            || chunk_version == 0
        {
            return Err(XcursorFileError::InvalidImageChunk);
        }
        let width = read_u32(bytes, position + 16)?;
        let height = read_u32(bytes, position + 20)?;
        let hotspot_x = read_u32(bytes, position + 24)?;
        let hotspot_y = read_u32(bytes, position + 28)?;
        let delay_ms = read_u32(bytes, position + 32)?;
        if width == 0 || height == 0 || width > MAX_CURSOR_DIMENSION || height > MAX_CURSOR_DIMENSION {
            return Err(XcursorFileError::InvalidDimensions);
        }
        if hotspot_x >= width || hotspot_y >= height {
            return Err(XcursorFileError::InvalidHotspot);
        }

        let pixel_count = usize::try_from(width)
            .ok()
            .and_then(|w| usize::try_from(height).ok().and_then(|h| w.checked_mul(h)))
            .ok_or(XcursorFileError::InvalidPixelData)?;
        let pixel_bytes = pixel_count.checked_mul(4).ok_or(XcursorFileError::InvalidPixelData)?;
        let pixels_start = position.checked_add(chunk_header).ok_or(XcursorFileError::InvalidPixelData)?;
        let pixels_end = pixels_start.checked_add(pixel_bytes).ok_or(XcursorFileError::InvalidPixelData)?;
        if pixels_end > bytes.len() {
            return Err(XcursorFileError::Truncated);
        }

        let mut rgba8 = Vec::with_capacity(pixel_bytes);
        for pixel in bytes[pixels_start..pixels_end].chunks_exact(4) {
            // Xcursor stores each pixel as a CARDINAL ARGB value in file byte order.
            let argb = u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);
            rgba8.extend_from_slice(&[
                ((argb >> 16) & 0xff) as u8,
                ((argb >> 8) & 0xff) as u8,
                (argb & 0xff) as u8,
                ((argb >> 24) & 0xff) as u8,
            ]);
        }
        frames.push(XcursorFrame { width, height, hotspot_x, hotspot_y, delay_ms, rgba8 });
    }

    if frames.is_empty() {
        return Err(XcursorFileError::NoImages);
    }
    Ok(frames)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, XcursorFileError> {
    let end = offset.checked_add(4).ok_or(XcursorFileError::Truncated)?;
    let raw: [u8; 4] = bytes
        .get(offset..end)
        .ok_or(XcursorFileError::Truncated)?
        .try_into()
        .map_err(|_| XcursorFileError::Truncated)?;
    Ok(u32::from_le_bytes(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(width: u32, height: u32, hotspot: (u32, u32), argb: &[u32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(FILE_MAGIC);
        bytes.extend_from_slice(&(FILE_HEADER_BYTES as u32).to_le_bytes());
        bytes.extend_from_slice(&(0x0001_0000_u32).to_le_bytes());
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&IMAGE_CHUNK_TYPE.to_le_bytes());
        bytes.extend_from_slice(&32_u32.to_le_bytes());
        bytes.extend_from_slice(&(28_u32).to_le_bytes());
        bytes.extend_from_slice(&(IMAGE_HEADER_BYTES as u32).to_le_bytes());
        bytes.extend_from_slice(&IMAGE_CHUNK_TYPE.to_le_bytes());
        bytes.extend_from_slice(&1_u32.to_le_bytes()); // subtype
        bytes.extend_from_slice(&1_u32.to_le_bytes()); // image version
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
        bytes.extend_from_slice(&hotspot.0.to_le_bytes());
        bytes.extend_from_slice(&hotspot.1.to_le_bytes());
        bytes.extend_from_slice(&125_u32.to_le_bytes());
        for pixel in argb {
            bytes.extend_from_slice(&pixel.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn decodes_argb_pixels_and_preserves_hotspot_and_delay() {
        let bytes = fixture(1, 1, (0, 0), &[0x7f12_34a5]);
        let frames = decode_xcursor(&bytes).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!((frames[0].width, frames[0].height), (1, 1));
        assert_eq!((frames[0].hotspot_x, frames[0].hotspot_y), (0, 0));
        assert_eq!(frames[0].delay_ms, 125);
        assert_eq!(frames[0].rgba8, [0x12, 0x34, 0xa5, 0x7f]);
    }

    #[test]
    fn rejects_invalid_magic_and_truncated_payloads() {
        assert_eq!(decode_xcursor(b"nope________________").unwrap_err(), XcursorFileError::InvalidMagic);
        let mut bytes = fixture(1, 1, (0, 0), &[0xff00_0000]);
        bytes.pop();
        assert_eq!(decode_xcursor(&bytes).unwrap_err(), XcursorFileError::Truncated);
    }

    #[test]
    fn rejects_hotspots_outside_the_image() {
        let bytes = fixture(1, 1, (1, 0), &[0xff00_0000]);
        assert_eq!(decode_xcursor(&bytes).unwrap_err(), XcursorFileError::InvalidHotspot);
    }

    #[test]
    fn rejects_images_larger_than_the_cursor_limit() {
        let bytes = fixture(257, 1, (0, 0), &[]);
        assert_eq!(decode_xcursor(&bytes).unwrap_err(), XcursorFileError::InvalidDimensions);
    }
}
