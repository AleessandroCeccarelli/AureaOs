// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Defensive ISO-BMFF file-type inspection for still HEIF and AVIF images.

use crate::ImageFormat;

const FTYP: &[u8; 4] = b"ftyp";
const AVIF: &[u8; 4] = b"avif";
const HEIC: &[u8; 4] = b"heic";
const HEIX: &[u8; 4] = b"heix";
const AVIS: &[u8; 4] = b"avis";
const MSF1: &[u8; 4] = b"msf1";

/// Identifies supported still-image brands without invoking a codec.
///
/// Animated/sequence brands are rejected even when they appear only in the
/// compatible-brand list. Generic `mif1` files are deliberately not guessed.
pub(super) fn detect_container_format(bytes: &[u8]) -> Option<ImageFormat> {
    let brands = parse_ftyp_brands(bytes)?;
    if brands.iter().any(|brand| *brand == AVIS || *brand == MSF1) {
        return None;
    }
    if brands.iter().any(|brand| *brand == AVIF) {
        return Some(ImageFormat::Avif);
    }
    if brands.iter().any(|brand| *brand == HEIC || *brand == HEIX) {
        return Some(ImageFormat::Heic);
    }
    None
}

fn parse_ftyp_brands(bytes: &[u8]) -> Option<Vec<&[u8; 4]>> {
    if bytes.get(4..8)? != FTYP {
        return None;
    }
    let short_size = u32::from_be_bytes(bytes.get(0..4)?.try_into().ok()?);
    let (box_size, header_size) = match short_size {
        0 => (u64::try_from(bytes.len()).ok()?, 8usize),
        1 => (u64::from_be_bytes(bytes.get(8..16)?.try_into().ok()?), 16usize),
        size => (u64::from(size), 8usize),
    };
    let available = u64::try_from(bytes.len()).ok()?;
    if box_size > available || box_size < u64::try_from(header_size + 8).ok()? {
        return None;
    }
    let box_end = usize::try_from(box_size).ok()?;
    let payload_len = box_end.checked_sub(header_size)?;
    if payload_len < 8 || (payload_len - 8) % 4 != 0 {
        return None;
    }
    let major: &[u8; 4] = bytes.get(header_size..header_size + 4)?.try_into().ok()?;
    let mut brands = vec![major];
    let mut offset = header_size.checked_add(8)?;
    while offset < box_end {
        let end = offset.checked_add(4)?;
        brands.push(bytes.get(offset..end)?.try_into().ok()?);
        offset = end;
    }
    Some(brands)
}

#[cfg(test)]
mod tests {
    use super::detect_container_format;
    use crate::ImageFormat;

    fn ftyp(major: &[u8; 4], compatible: &[&[u8; 4]]) -> Vec<u8> {
        let size = 16 + compatible.len() * 4;
        let mut bytes = Vec::with_capacity(size);
        bytes.extend_from_slice(&(size as u32).to_be_bytes());
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(major);
        bytes.extend_from_slice(&0u32.to_be_bytes());
        for brand in compatible {
            bytes.extend_from_slice(*brand);
        }
        bytes
    }

    #[test]
    fn recognizes_avif_and_heic_still_brands() {
        assert_eq!(detect_container_format(&ftyp(b"avif", &[])), Some(ImageFormat::Avif));
        assert_eq!(detect_container_format(&ftyp(b"heic", &[])), Some(ImageFormat::Heic));
        assert_eq!(detect_container_format(&ftyp(b"mif1", &[b"heix"])), Some(ImageFormat::Heic));
    }

    #[test]
    fn rejects_sequence_brands_even_when_compatible() {
        assert_eq!(detect_container_format(&ftyp(b"avif", &[b"avis"])), None);
        assert_eq!(detect_container_format(&ftyp(b"heic", &[b"msf1"])), None);
        assert_eq!(detect_container_format(&ftyp(b"avis", &[])), None);
    }

    #[test]
    fn rejects_generic_mif1_and_malformed_boxes() {
        assert_eq!(detect_container_format(&ftyp(b"mif1", &[])), None);
        assert_eq!(detect_container_format(b"not an image"), None);
        assert_eq!(detect_container_format(&[0, 0, 0, 15, b'f', b't', b'y', b'p']), None);
    }

    #[test]
    fn supports_extended_size_and_rejects_truncation() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(&24u64.to_be_bytes());
        bytes.extend_from_slice(b"avif");
        bytes.extend_from_slice(&0u32.to_be_bytes());
        assert_eq!(detect_container_format(&bytes), Some(ImageFormat::Avif));
        bytes.pop();
        assert_eq!(detect_container_format(&bytes), None);
    }
}
