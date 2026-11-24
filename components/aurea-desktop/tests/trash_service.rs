// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Integration tests for the public Trash service contract.

use aurea_desktop::trash::{TrashError, TrashService, TrashVisualState};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct TestHome(PathBuf);

impl TestHome {
    fn new() -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aurea-trash-test-{id}"));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn maps_service_visual_state_to_the_shared_desktop_contract() {
    use aurea_desktop_api::desktop::DesktopTrashState;

    assert_eq!(DesktopTrashState::from(TrashVisualState::Empty), DesktopTrashState::Empty);
    assert_eq!(
        DesktopTrashState::from(TrashVisualState::PartiallyFull),
        DesktopTrashState::PartiallyFull,
    );
    assert_eq!(DesktopTrashState::from(TrashVisualState::Full), DesktopTrashState::Full);
}

#[test]
fn moves_lists_restores_and_updates_visual_state() {
    let home = TestHome::new();
    let source = home.0.join("report.txt");
    fs::write(&source, "hello").unwrap();
    let trash = TrashService::new(&home.0).unwrap();

    assert_eq!(trash.snapshot().unwrap().visual_state(), TrashVisualState::Empty);
    trash.move_to_trash(&source).unwrap();
    assert!(!source.exists());

    let entries = trash.entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        trash.snapshot().unwrap().visual_state(),
        TrashVisualState::PartiallyFull,
    );

    let restored = trash.restore(entries[0].name()).unwrap();
    assert_eq!(restored, source);
    assert_eq!(fs::read_to_string(restored).unwrap(), "hello");
    assert_eq!(trash.snapshot().unwrap().visual_state(), TrashVisualState::Empty);
}

#[test]
fn refuses_to_overwrite_restore_destination() {
    let home = TestHome::new();
    let source = home.0.join("report.txt");
    fs::write(&source, "old").unwrap();
    let trash = TrashService::new(&home.0).unwrap();
    trash.move_to_trash(&source).unwrap();
    fs::write(&source, "new").unwrap();

    let entry = trash.entries().unwrap().remove(0);
    assert!(matches!(
        trash.restore(entry.name()),
        Err(TrashError::RestoreDestinationExists(_))
    ));
    assert_eq!(fs::read_to_string(source).unwrap(), "new");
}

#[test]
fn full_state_uses_configured_threshold() {
    let home = TestHome::new();
    let trash = TrashService::with_full_threshold(&home.0, 1).unwrap();
    let source = home.0.join("item");
    fs::write(&source, "x").unwrap();

    trash.move_to_trash(&source).unwrap();

    assert_eq!(trash.snapshot().unwrap().visual_state(), TrashVisualState::Full);
}

#[test]
fn permanently_deletes_only_the_selected_entry() {
    let home = TestHome::new();
    let source = home.0.join("report.txt");
    fs::write(&source, "hello").unwrap();
    let trash = TrashService::new(&home.0).unwrap();
    trash.move_to_trash(&source).unwrap();

    let entry = trash.entries().unwrap().remove(0);
    trash.delete_permanently(entry.name()).unwrap();

    assert!(trash.entries().unwrap().is_empty());
}
