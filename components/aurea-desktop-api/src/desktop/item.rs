// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! User-owned entries displayed on the desktop.

/// Stable identifier for one desktop entry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DesktopItemId(String);

impl DesktopItemId {
    /// Creates an identifier when its value is non-empty and contains no whitespace.
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.is_empty() || value.trim() != value || value.chars().any(char::is_whitespace) {
            return None;
        }
        Some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Logical position on the desktop canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DesktopPosition {
    pub x: i32,
    pub y: i32,
}

impl DesktopPosition {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// The kind of resource represented by a desktop entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopItemKind {
    File { uri: String },
    Folder { uri: String },
    Application { desktop_id: String },
    Trash,
}

/// One user-visible desktop entry and its saved canvas position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopItem {
    id: DesktopItemId,
    name: String,
    kind: DesktopItemKind,
    position: DesktopPosition,
}

impl DesktopItem {
    pub fn new(
        id: DesktopItemId,
        name: impl Into<String>,
        kind: DesktopItemKind,
        position: DesktopPosition,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            position,
        }
    }

    pub fn id(&self) -> &DesktopItemId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn kind(&self) -> &DesktopItemKind {
        &self.kind
    }
    pub const fn position(&self) -> DesktopPosition {
        self.position
    }

    pub fn set_position(&mut self, position: DesktopPosition) {
        self.position = position;
    }

    /// Returns the file extension without the leading dot, if present.
    pub fn file_extension(&self) -> Option<String> {
        let DesktopItemKind::File { uri } = &self.kind else {
            return None;
        };
        let path = uri.split(['?', '#']).next()?;
        let file_name = path.rsplit('/').next()?;
        let (_, extension) = file_name.rsplit_once('.')?;
        if extension.is_empty() {
            return None;
        }
        Some(extension.to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopItem, DesktopItemId, DesktopItemKind, DesktopPosition};

    #[test]
    fn rejects_empty_or_whitespace_identifiers() {
        assert!(DesktopItemId::new("").is_none());
        assert!(DesktopItemId::new(" desktop:item").is_none());
        assert!(DesktopItemId::new("desktop item").is_none());
    }

    #[test]
    fn extracts_case_insensitive_extension_from_file_uri() {
        let item = DesktopItem::new(
            DesktopItemId::new("file:one").unwrap(),
            "Photo",
            DesktopItemKind::File {
                uri: "file:///home/user/Picture.JPEG?download=1".into(),
            },
            DesktopPosition::default(),
        );
        assert_eq!(item.file_extension().as_deref(), Some("jpeg"));
    }

    #[test]
    fn folders_have_no_file_extension() {
        let item = DesktopItem::new(
            DesktopItemId::new("folder:one").unwrap(),
            "Pictures",
            DesktopItemKind::Folder {
                uri: "file:///home/user/Pictures".into(),
            },
            DesktopPosition::default(),
        );
        assert_eq!(item.file_extension(), None);
    }
}
