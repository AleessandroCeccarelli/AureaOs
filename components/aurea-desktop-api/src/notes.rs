// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Application-facing notes contracts.
//!
//! Note content is private user data. Access must be mediated by the system
//! service and an explicit application permission or user-mediated selection.

/// Opaque identifier assigned by the notes provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoteId(String);

impl NoteId {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return None;
        }
        Some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Metadata returned without exposing note content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteSummary {
    id: NoteId,
    title: String,
    modified_unix_seconds: i64,
}

impl NoteSummary {
    pub fn new(id: NoteId, title: impl Into<String>, modified_unix_seconds: i64) -> Self {
        Self {
            id,
            title: title.into(),
            modified_unix_seconds,
        }
    }

    pub fn id(&self) -> &NoteId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub const fn modified_unix_seconds(&self) -> i64 {
        self.modified_unix_seconds
    }
}

/// Explicit access level requested by an application.
///
/// The system permission broker decides whether to grant it. In particular,
/// access to one user-selected note does not imply access to the whole library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotesAccess {
    ReadSelectedNote,
    CreateNote,
    ReadAllNotes,
    ModifyAllNotes,
    DeleteNotes,
}

/// Notes provider operations. The IPC/permission layer must authorize every
/// operation and bind selected-note grants to the selected NoteId.
pub trait NotesProvider: Send + Sync {
    fn list_notes(&self) -> Result<Vec<NoteSummary>, NotesError>;
    fn read_note(&self, id: &NoteId) -> Result<String, NotesError>;
    fn create_note(&self, title: &str, content: &str) -> Result<NoteId, NotesError>;
    fn update_note(&self, id: &NoteId, title: &str, content: &str) -> Result<(), NotesError>;
    fn delete_note(&self, id: &NoteId) -> Result<(), NotesError>;
}

/// Notes provider failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotesError {
    Unavailable,
    NotFound,
    AccessDenied,
    InvalidInput,
}
