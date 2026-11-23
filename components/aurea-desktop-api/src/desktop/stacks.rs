// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Derived visual stacks. Building stacks never changes or moves source files.

use std::collections::BTreeMap;
use super::{DesktopItem, DesktopItemId, DesktopItemKind, StackGrouping};

/// Stable, deterministic identifier for a derived stack.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DesktopStackId(String);

impl DesktopStackId {
    pub fn as_str(&self) -> &str { &self.0 }
}

/// Broad file family used by category grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DesktopStackCategory {
    Images,
    Documents,
    Video,
    Audio,
    Archives,
    Other,
}

/// A stack is a visual grouping of existing desktop item identifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopStack {
    id: DesktopStackId,
    label: String,
    items: Vec<DesktopItemId>,
}

impl DesktopStack {
    pub const fn id(&self) -> &DesktopStackId { &self.id }
    pub fn label(&self) -> &str { &self.label }
    pub fn items(&self) -> &[DesktopItemId] { &self.items }
}

/// Derives stacks from eligible files. Groups with only one item stay as icons.
pub(super) fn build_stacks(
    items: &[DesktopItem],
    grouping: StackGrouping,
    is_excluded: impl Fn(&DesktopItemId) -> bool,
) -> Vec<DesktopStack> {
    let mut groups: BTreeMap<String, (String, Vec<DesktopItemId>)> = BTreeMap::new();

    for item in items {
        if is_excluded(item.id()) || !matches!(item.kind(), DesktopItemKind::File { .. }) {
            continue;
        }
        let Some(extension) = item.file_extension() else { continue };
        let (key, label) = match grouping {
            StackGrouping::ByExtension => (format!("extension:{extension}"), format!(".{extension}")),
            StackGrouping::ByCategory => {
                let category = category_for_extension(&extension);
                (format!("category:{category:?}"), category_label(category).to_owned())
            }
        };
        groups.entry(key).or_insert_with(|| (label, Vec::new())).1.push(item.id().clone());
    }

    groups.into_iter().filter_map(|(key, (label, items))| {
        (items.len() > 1).then_some(DesktopStack {
            id: DesktopStackId(key),
            label,
            items,
        })
    }).collect()
}

fn category_for_extension(extension: &str) -> DesktopStackCategory {
    match extension {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "heic" | "avif" => DesktopStackCategory::Images,
        "pdf" | "doc" | "docx" | "odt" | "rtf" | "txt" | "md" | "xls" | "xlsx" | "ppt" | "pptx" | "ods" | "odp" => DesktopStackCategory::Documents,
        "mp4" | "mkv" | "mov" | "avi" | "webm" | "mpeg" | "mpg" => DesktopStackCategory::Video,
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus" => DesktopStackCategory::Audio,
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst" => DesktopStackCategory::Archives,
        _ => DesktopStackCategory::Other,
    }
}

fn category_label(category: DesktopStackCategory) -> &'static str {
    match category {
        DesktopStackCategory::Images => "Immagini",
        DesktopStackCategory::Documents => "Documenti",
        DesktopStackCategory::Video => "Video",
        DesktopStackCategory::Audio => "Audio",
        DesktopStackCategory::Archives => "Archivi",
        DesktopStackCategory::Other => "Altri file",
    }
}

#[cfg(test)]
mod tests {
    use super::super::{DesktopItem, DesktopItemId, DesktopItemKind, DesktopPosition, StackGrouping};
    use super::build_stacks;

    fn file(id: &str, uri: &str) -> DesktopItem {
        DesktopItem::new(
            DesktopItemId::new(id).unwrap(),
            id,
            DesktopItemKind::File { uri: uri.into() },
            DesktopPosition::default(),
        )
    }

    #[test]
    fn groups_multiple_files_by_extension_without_moving_them() {
        let items = vec![file("one", "file:///home/user/a.pdf"), file("two", "file:///home/user/b.PDF")];
        let stacks = build_stacks(&items, StackGrouping::ByExtension, |_| false);
        assert_eq!(stacks.len(), 1);
        assert_eq!(stacks[0].label(), ".pdf");
        assert_eq!(stacks[0].items().len(), 2);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn category_grouping_combines_different_image_extensions() {
        let items = vec![file("one", "file:///home/user/a.png"), file("two", "file:///home/user/b.jpeg")];
        let stacks = build_stacks(&items, StackGrouping::ByCategory, |_| false);
        assert_eq!(stacks.len(), 1);
        assert_eq!(stacks[0].label(), "Immagini");
    }

    #[test]
    fn singletons_and_excluded_files_remain_individual_entries() {
        let items = vec![file("one", "file:///home/user/a.png"), file("two", "file:///home/user/b.png"), file("three", "file:///home/user/c.png")];
        let excluded = DesktopItemId::new("three").unwrap();
        let stacks = build_stacks(&items, StackGrouping::ByExtension, |id| id == &excluded);
        assert_eq!(stacks.len(), 1);
        assert_eq!(stacks[0].items().len(), 2);
    }
}
