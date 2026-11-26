// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Encoding and parsing of freedesktop Trash .trashinfo metadata.

use std::path::{Path, PathBuf};

pub(super) fn encode_path(path: &Path) -> String {
    let mut encoded = String::new();
    for byte in path.to_string_lossy().bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

pub(super) fn decode_path(value: &str) -> Option<PathBuf> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = *bytes.get(index + 1)?;
            let low = *bytes.get(index + 2)?;
            let hex = |byte: u8| -> Option<u8> {
                match byte {
                    b'0'..=b'9' => Some(byte - b'0'),
                    b'A'..=b'F' => Some(byte - b'A' + 10),
                    b'a'..=b'f' => Some(byte - b'a' + 10),
                    _ => None,
                }
            };
            decoded.push((hex(high)? << 4) | hex(low)?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok().map(PathBuf::from)
}

pub(super) fn write_info(original: &Path, deletion_date: &str) -> String {
    format!(
        "[Trash Info]\nPath={}\nDeletionDate={}\n",
        encode_path(original),
        deletion_date
    )
}

pub(super) fn read_original_path(info: &str) -> Option<PathBuf> {
    let value = info.lines().find_map(|line| line.strip_prefix("Path="))?;
    decode_path(value)
}

#[cfg(test)]
mod tests {
    use super::{decode_path, encode_path, read_original_path, write_info};
    use std::path::Path;

    #[test]
    fn path_encoding_round_trips_spaces_and_unicode() {
        let original = Path::new("/home/test/Documenti/idee è nuove.txt");
        assert_eq!(
            decode_path(&encode_path(original)).as_deref(),
            Some(original)
        );
    }

    #[test]
    fn trash_info_round_trips_original_path() {
        let original = Path::new("/home/test/file name.txt");
        let info = write_info(original, "2026-11-23T21:00:00");
        assert_eq!(read_original_path(&info).as_deref(), Some(original));
    }

    #[test]
    fn malformed_percent_encoding_is_rejected() {
        assert!(decode_path("/tmp/%Q0").is_none());
        assert!(decode_path("/tmp/%2").is_none());
    }
}
