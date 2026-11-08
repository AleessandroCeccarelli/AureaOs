// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Shutdown signal integration tests.

use std::thread;

use aurea_desktop_api::ShutdownSignal;

#[test]
fn shutdown_signal_starts_in_inactive_state() {
    let shutdown = ShutdownSignal::new();

    assert!(!shutdown.is_shutdown_requested());
}

#[test]
fn shutdown_request_is_visible_through_cloned_signal() {
    let shutdown = ShutdownSignal::new();
    let observer = shutdown.clone();

    shutdown.request_shutdown();

    assert!(observer.is_shutdown_requested());
}

#[test]
fn shutdown_request_is_visible_through_atomic_flag() {
    let shutdown = ShutdownSignal::new();
    let atomic_flag = shutdown.atomic_flag();

    shutdown.request_shutdown();

    assert!(atomic_flag.load(std::sync::atomic::Ordering::Acquire));
}

#[test]
fn shutdown_request_is_visible_across_threads() {
    let shutdown = ShutdownSignal::new();
    let worker_signal = shutdown.clone();

    let worker = thread::spawn(move || {
        while !worker_signal.is_shutdown_requested() {
            thread::yield_now();
        }

        worker_signal.is_shutdown_requested()
    });

    shutdown.request_shutdown();

    assert!(worker.join().expect("Worker thread panicked"));
}

#[test]
fn repeated_shutdown_requests_are_safe() {
    let shutdown = ShutdownSignal::new();

    shutdown.request_shutdown();
    shutdown.request_shutdown();

    assert!(shutdown.is_shutdown_requested());
}
