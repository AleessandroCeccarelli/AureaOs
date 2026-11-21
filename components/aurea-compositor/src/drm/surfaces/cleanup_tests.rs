// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Hardware-independent tests for DRM surface cleanup ownership.

use std::{cell::RefCell, collections::HashMap, fmt};

use super::clear_and_retain;

fn clear_each<T, E>(
    items: impl IntoIterator<Item = T>,
    mut clear: impl FnMut(T) -> Result<(), E>,
) -> Vec<String>
where
    E: fmt::Display,
{
    items
        .into_iter()
        .filter_map(|item| clear(item).err().map(|error| error.to_string()))
        .collect()
}

#[test]
fn clear_attempts_every_surface_in_iteration_order() {
    let calls = RefCell::new(Vec::new());
    let errors = clear_each([1, 2, 3], |item| {
        calls.borrow_mut().push(item);
        Ok::<(), &'static str>(())
    });

    assert!(errors.is_empty());
    assert_eq!(*calls.borrow(), vec![1, 2, 3]);
}

#[test]
fn clear_continues_after_failures_and_collects_them() {
    let calls = RefCell::new(Vec::new());
    let errors = clear_each([1, 2, 3], |item| {
        calls.borrow_mut().push(item);
        if item != 2 {
            Err(format!("surface {item} failed"))
        } else {
            Ok(())
        }
    });

    assert_eq!(*calls.borrow(), vec![1, 2, 3]);
    assert_eq!(errors, vec!["surface 1 failed", "surface 3 failed"]);
}

#[test]
fn failed_cleanup_entry_is_retained_and_can_be_retried() {
    let mut entries = HashMap::from([(1_u8, false), (2_u8, true)]);
    let errors = clear_and_retain(&mut entries, |should_fail| {
        if *should_fail {
            Err("KMS disable failed")
        } else {
            Ok(())
        }
    });

    assert_eq!(entries.len(), 1);
    assert_eq!(entries.get(&2), Some(&true));
    assert_eq!(errors, vec!["KMS disable failed"]);

    let retry_errors = clear_and_retain(&mut entries, |_| Ok::<(), &'static str>(())); 
    assert!(retry_errors.is_empty());
    assert!(entries.is_empty());
}

#[test]
fn clearing_an_empty_collection_is_successful() {
    let errors = clear_each(Vec::<u8>::new(), |_| Ok::<(), &'static str>(())); 
    assert!(errors.is_empty());
}
