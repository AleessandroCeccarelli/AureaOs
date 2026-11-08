// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Shell
// Copyright (C) 2026 Alessandro Ceccarelli
//
// Application lifecycle and compositor integration tests.

use std::io;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle, ThreadId};

use aurea_desktop_api::{CompositorFactory, CompositorService, ShutdownSignal};

use super::{join_compositor_thread, request_shutdown};
use crate::config::ShellConfig;
use crate::error::ShellError;
use crate::event::ShellEvent;
use crate::event_bus::EventBus;

/// Factory that records its initialization thread and returns an error.
struct InitializationFailureFactory {
    observed_thread: Arc<Mutex<Option<ThreadId>>>,
}

impl CompositorFactory for InitializationFailureFactory {
    fn create(&self) -> io::Result<Box<dyn CompositorService>> {
        *self
            .observed_thread
            .lock()
            .expect("Thread ID mutex was poisoned") = Some(thread::current().id());

        Err(io::Error::other("Compositor initialization test failure"))
    }
}

/// Factory that panics during initialization.
struct InitializationPanicFactory;

impl CompositorFactory for InitializationPanicFactory {
    fn create(&self) -> io::Result<Box<dyn CompositorService>> {
        panic!("Compositor initialization test panic");
    }
}

/// Service that returns a controlled runtime error.
struct RuntimeFailureService {
    observed_shutdown: Arc<Mutex<Option<ShutdownSignal>>>,
}

impl CompositorService for RuntimeFailureService {
    fn run(&mut self, shutdown: &ShutdownSignal) -> io::Result<()> {
        *self
            .observed_shutdown
            .lock()
            .expect("Shutdown signal mutex was poisoned") = Some(shutdown.clone());

        Err(io::Error::other("Compositor runtime test failure"))
    }
}

/// Factory for the runtime failure service.
struct RuntimeFailureFactory {
    observed_shutdown: Arc<Mutex<Option<ShutdownSignal>>>,
}

impl CompositorFactory for RuntimeFailureFactory {
    fn create(&self) -> io::Result<Box<dyn CompositorService>> {
        Ok(Box::new(RuntimeFailureService {
            observed_shutdown: Arc::clone(&self.observed_shutdown),
        }))
    }
}

/// Service that panics during runtime execution.
struct RuntimePanicService;

impl CompositorService for RuntimePanicService {
    fn run(&mut self, _shutdown: &ShutdownSignal) -> io::Result<()> {
        panic!("Compositor runtime test panic");
    }
}

/// Factory for the runtime panic service.
struct RuntimePanicFactory;

impl CompositorFactory for RuntimePanicFactory {
    fn create(&self) -> io::Result<Box<dyn CompositorService>> {
        Ok(Box::new(RuntimePanicService))
    }
}

/// Service that exits successfully and records its execution.
struct SuccessfulService {
    started: Arc<AtomicBool>,
}

impl CompositorService for SuccessfulService {
    fn run(&mut self, _shutdown: &ShutdownSignal) -> io::Result<()> {
        self.started.store(true, Ordering::Release);
        Ok(())
    }
}

/// Factory for the successful service.
struct SuccessfulFactory {
    started: Arc<AtomicBool>,
}

impl CompositorFactory for SuccessfulFactory {
    fn create(&self) -> io::Result<Box<dyn CompositorService>> {
        Ok(Box::new(SuccessfulService {
            started: Arc::clone(&self.started),
        }))
    }
}

/// Service that waits until the shared shutdown signal is requested.
struct ShutdownAwareService {
    started: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
}

impl CompositorService for ShutdownAwareService {
    fn run(&mut self, shutdown: &ShutdownSignal) -> io::Result<()> {
        self.started.store(true, Ordering::Release);

        while !shutdown.is_shutdown_requested() {
            thread::yield_now();
        }

        self.stopped.store(true, Ordering::Release);

        Ok(())
    }
}

/// Factory for the shutdown-aware service.
struct ShutdownAwareFactory {
    started: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
}

impl CompositorFactory for ShutdownAwareFactory {
    fn create(&self) -> io::Result<Box<dyn CompositorService>> {
        Ok(Box::new(ShutdownAwareService {
            started: Arc::clone(&self.started),
            stopped: Arc::clone(&self.stopped),
        }))
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

    let result = super::run_with_shutdown_handler(ShellConfig::default(), factory, |_| true);

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
        "Compositor initialization errors must be propagated"
    );
}

#[test]
fn compositor_initialization_panic_is_detected() {
    let result = super::run_with_shutdown_handler(
        ShellConfig::default(),
        InitializationPanicFactory,
        |_| true,
    );

    assert!(
        matches!(result, Err(ShellError::CompositorThreadPanicked)),
        "Initialization panics must be detected"
    );
}

#[test]
fn compositor_runtime_failure_is_propagated_and_shutdown_is_requested() {
    let observed_shutdown = Arc::new(Mutex::new(None));

    let factory = RuntimeFailureFactory {
        observed_shutdown: Arc::clone(&observed_shutdown),
    };

    let result = super::run_with_shutdown_handler(ShellConfig::default(), factory, |_| true);

    assert!(
        matches!(result, Err(ShellError::Compositor(_))),
        "Compositor runtime errors must be propagated"
    );

    let shutdown = observed_shutdown
        .lock()
        .expect("Shutdown signal mutex was poisoned")
        .clone()
        .expect("Compositor service did not receive the shutdown signal");

    assert!(
        shutdown.is_shutdown_requested(),
        "Shutdown must be requested after compositor failure"
    );
}

#[test]
fn compositor_runtime_panic_is_detected_without_blocking_shell() {
    let result =
        super::run_with_shutdown_handler(ShellConfig::default(), RuntimePanicFactory, |_| true);

    assert!(
        matches!(result, Err(ShellError::CompositorThreadPanicked)),
        "Runtime panics must be propagated after shell shutdown"
    );
}

#[test]
fn successful_compositor_exit_stops_shell_runtime() {
    let started = Arc::new(AtomicBool::new(false));

    let factory = SuccessfulFactory {
        started: Arc::clone(&started),
    };

    let result = super::run_with_shutdown_handler(ShellConfig::default(), factory, |_| true);

    assert!(result.is_ok());

    assert!(
        started.load(Ordering::Acquire),
        "Compositor service must have been started"
    );
}

#[test]
fn shutdown_handler_requests_application_shutdown() {
    let started = Arc::new(AtomicBool::new(false));
    let stopped = Arc::new(AtomicBool::new(false));

    let factory = ShutdownAwareFactory {
        started: Arc::clone(&started),
        stopped: Arc::clone(&stopped),
    };

    let result =
        super::run_with_shutdown_handler(ShellConfig::default(), factory, |mut handler| {
            handler();
            true
        });

    assert!(result.is_ok());

    assert!(
        started.load(Ordering::Acquire),
        "Compositor service must have started"
    );

    assert!(
        stopped.load(Ordering::Acquire),
        "Compositor service must stop after shutdown is requested"
    );
}

#[test]
fn shutdown_handler_registration_failure_cleans_up_runtime() {
    let started = Arc::new(AtomicBool::new(false));

    let factory = SuccessfulFactory {
        started: Arc::clone(&started),
    };

    let result = super::run_with_shutdown_handler(ShellConfig::default(), factory, |_| false);

    assert!(
        matches!(
            result,
            Err(ShellError::InitializationFailed(
                "Failed to register shutdown handler"
            ))
        ),
        "Handler registration failure must be reported"
    );

    assert!(
        !started.load(Ordering::Acquire),
        "Compositor must not start when handler registration fails"
    );
}

#[test]
fn compositor_thread_join_returns_success() {
    let compositor_thread: JoinHandle<io::Result<()>> = thread::spawn(|| Ok(()));

    assert!(join_compositor_thread(compositor_thread).is_ok());
}

#[test]
fn compositor_thread_join_propagates_io_error() {
    let compositor_thread: JoinHandle<io::Result<()>> =
        thread::spawn(|| Err(io::Error::other("Compositor runtime test failure")));

    assert!(matches!(
        join_compositor_thread(compositor_thread),
        Err(ShellError::Compositor(_))
    ));
}

#[test]
fn compositor_thread_join_detects_panic() {
    let compositor_thread: JoinHandle<io::Result<()>> = thread::spawn(|| {
        panic!("Compositor worker thread test panic");
    });

    assert!(matches!(
        join_compositor_thread(compositor_thread),
        Err(ShellError::CompositorThreadPanicked)
    ));
}
