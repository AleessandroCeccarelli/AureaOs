// SPDX-License-Identifier: GPL-3.0-or-later
// AureaOS — Color Management

use crate::MAX_PROFILE_SIZE;

/// Validated ICC tag table entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IccTag {
    pub signature: [u8; 4],
    pub offset: usize,
    pub size: usize,
}

/// Parsed fields needed to classify and safely inspect an ICC profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IccHeader {
    declared_size: usize,
    pub version: [u8; 4],
    pub device_class: [u8; 4],
    pub color_space: [u8; 4],
    pub connection_space: [u8; 4],
    pub manufacturer: [u8; 4],
    pub model: [u8; 4],
    tags: Vec<IccTag>,
}

/// Safe summary that does not expose borrowed or unchecked file offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IccProfileSummary {
    pub declared_size: usize,
    pub version: [u8; 4],
    pub device_class: [u8; 4],
    pub color_space: [u8; 4],
    pub connection_space: [u8; 4],
    pub manufacturer: [u8; 4],
    pub model: [u8; 4],
    pub tag_count: usize,
}

/// ICC structural validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IccHeaderError {
    TooSmall,
    TooLarge { maximum_bytes: usize },
    DeclaredSizeMismatch,
    InvalidSignature,
    InvalidTagTable,
    InvalidTagRange,
    NonZeroReservedBytes,
}

impl IccHeader {
    /// Parses the 128-byte ICC header and validates every tag-table range.
    pub fn parse(bytes: &[u8]) -> Result<Self, IccHeaderError> {
        if bytes.len() < 132 {
            return Err(IccHeaderError::TooSmall);
        }
        if bytes.len() > MAX_PROFILE_SIZE {
            return Err(IccHeaderError::TooLarge { maximum_bytes: MAX_PROFILE_SIZE });
        }
        let declared_size = read_u32(bytes, 0)? as usize;
        if declared_size != bytes.len() || declared_size < 132 {
            return Err(IccHeaderError::DeclaredSizeMismatch);
        }
        if &bytes[36..40] != b"acsp" {
            return Err(IccHeaderError::InvalidSignature);
        }
        if bytes[100..128].iter().any(|byte| *byte != 0) {
            return Err(IccHeaderError::NonZeroReservedBytes);
        }
        let count = read_u32(bytes, 128)? as usize;
        let table_size = count.checked_mul(12).ok_or(IccHeaderError::InvalidTagTable)?;
        let table_end = 132_usize.checked_add(table_size).ok_or(IccHeaderError::InvalidTagTable)?;
        if table_end > declared_size {
            return Err(IccHeaderError::InvalidTagTable);
        }
        let mut tags = Vec::with_capacity(count);
        for index in 0..count {
            let start = 132 + index * 12;
            let mut signature = [0_u8; 4];
            signature.copy_from_slice(&bytes[start..start + 4]);
            let offset = read_u32(bytes, start + 4)? as usize;
            let size = read_u32(bytes, start + 8)? as usize;
            let end = offset.checked_add(size).ok_or(IccHeaderError::InvalidTagRange)?;
            if size == 0 || offset < table_end || end > declared_size {
                return Err(IccHeaderError::InvalidTagRange);
            }
            tags.push(IccTag { signature, offset, size });
        }
        Ok(Self {
            declared_size,
            version: copy_signature(bytes, 8),
            device_class: copy_signature(bytes, 12),
            color_space: copy_signature(bytes, 16),
            connection_space: copy_signature(bytes, 20),
            manufacturer: copy_signature(bytes, 48),
            model: copy_signature(bytes, 52),
            tags,
        })
    }

    /// Returns a metadata-only snapshot.
    pub fn summary(&self) -> IccProfileSummary {
        IccProfileSummary {
            declared_size: self.declared_size,
            version: self.version,
            device_class: self.device_class,
            color_space: self.color_space,
            connection_space: self.connection_space,
            manufacturer: self.manufacturer,
            model: self.model,
            tag_count: self.tags.len(),
        }
    }

    /// Returns validated tag entries.
    pub fn tags(&self) -> &[IccTag] {
        &self.tags
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, IccHeaderError> {
    let end = offset.checked_add(4).ok_or(IccHeaderError::TooSmall)?;
    let slice = bytes.get(offset..end).ok_or(IccHeaderError::TooSmall)?;
    Ok(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn copy_signature(bytes: &[u8], offset: usize) -> [u8; 4] {
    [bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]
}
