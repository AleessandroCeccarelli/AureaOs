// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application lifecycle and compositor integration tests.

use std::io;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle, ThreadId};

use aurea_desktop_api::{CompositorFactory, CompositorService, ShutdownSignal};

use super::{join_compositor_thread, request_shutdown};
use crate::config::ShellConfig;
use crate::error::ShellError;
use crate::event::ShellEvent;
use crate::event_bus::EventBus;

/// Factory that records the thread used for compositor initialization
/// and then returns a controlled initialization error.
struct InitializationFailureFactory {
    observed_thread: Arc<Mutex<Option<ThreadId>>>,
}

impl CompositorFactory for InitializationFailureFactory {
    fn create(&self) -> io::Result<Box<dyn CompositorService>> {
        let current_thread = thread::current().id();

        *self
            .observed_thread
            .lock()
            .expect("Thread ID mutex was poisoned") = Some(current_thread);

        Err(io::Error::other("Compositor initialization test failure"))
    }
}

/// Compositor service that returns a controlled runtime error.
struct RuntimeFailureService;

impl CompositorService for RuntimeFailureService {
    fn run(&mut self, _shutdown: &ShutdownSignal) -> io::Result<()> {
        Err(io::Error::other("Compositor runtime test failure"))
    }
}

/// Factory that creates the controlled runtime failure service.
struct RuntimeFailureFactory;

impl CompositorFactory for RuntimeFailureFactory {
    fn create(&self) -> io::Result<Box<dyn CompositorService>> {
        Ok(Box::new(RuntimeFailureService))
    }
}

#[test]
fn shutdown_request_publishes_shutdown_event() {
    let (event_bus, receiver) = EventBus::new();

    request_shutdown(&event_bus);

    assert_eq!(receiver.recv().unwrap(), ShellEvent::ShutdownRequested);
}

#[test]
fn compositor_factory_is_initialized_on_worker_thread() {
    let caller_thread = thread::current().id();
    let observed_thread = Arc::new(Mutex::new(None));

    let factory = InitializationFailureFactory {
        observed_thread: Arc::clone(&observed_thread),
    };

    let result = super::run(ShellConfig::default(), factory);

    let compositor_thread = observed_thread
        .lock()
        .expect("Thread ID mutex was poisoned")
        .expect("Compositor factory was not called");

    assert_ne!(
        caller_thread, compositor_thread,
        "Compositor must be initialized on its dedicated worker thread"
    );

    assert!(
        matches!(result, Err(ShellError::Compositor(_))),
        "Compositor initialization errors must be propagated to the shell"
    );
}

#[test]
fn compositor_runtime_failure_is_propagated_to_shell() {
    let result = super::run(ShellConfig::default(), RuntimeFailureFactory);

    assert!(
        matches!(result, Err(ShellError::Compositor(_))),
        "Compositor runtime errors must be propagated to the shell"
    );
}

#[test]
fn compositor_thread_join_returns_success() {
    let compositor_thread: JoinHandle<io::Result<()>> = thread::spawn(|| Ok(()));

    let result = join_compositor_thread(compositor_thread);

    assert!(result.is_ok());
}

#[test]
fn compositor_thread_join_propagates_io_error() {
    let compositor_thread: JoinHandle<io::Result<()>> =
        thread::spawn(|| Err(io::Error::other("Compositor runtime test failure")));

    let result = join_compositor_thread(compositor_thread);

    assert!(matches!(result, Err(ShellError::Compositor(_))));
}

#[test]
fn compositor_thread_join_detects_panic() {
    let compositor_thread: JoinHandle<io::Result<()>> = thread::spawn(|| {
        panic!("Compositor worker thread test panic");
    });

    let result = join_compositor_thread(compositor_thread);

    assert!(matches!(result, Err(ShellError::CompositorThreadPanicked)));
}
