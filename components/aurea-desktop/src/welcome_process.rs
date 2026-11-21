// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Process ownership and teardown for the first desktop client.

use std::{
    io,
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

const WELCOME_CLIENT_ARGUMENT: &str = "--welcome-client";
const GRACEFUL_STOP_TIMEOUT: Duration = Duration::from_secs(2);
const STOP_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Owns the Welcome client and guarantees that it is reaped on teardown.
pub(super) struct WelcomeClientProcess {
    child: Option<Child>,
}

impl WelcomeClientProcess {
    /// Starts the current desktop executable in its Welcome client mode.
    pub(super) fn spawn(display_name: &str) -> io::Result<Self> {
        let executable = std::env::current_exe()?;
        let child = Command::new(executable)
            .arg(WELCOME_CLIENT_ARGUMENT)
            // Force the client to use AureaOS's socket, not an inherited fd.
            .env_remove("WAYLAND_SOCKET")
            .env("WAYLAND_DISPLAY", display_name)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("cannot start the Aurea Welcome client: {error}"),
                )
            })?;

        Ok(Self { child: Some(child) })
    }

    /// Stops a running client and returns its final process status.
    pub(super) fn stop(&mut self) -> io::Result<Option<ExitStatus>> {
        let Some(mut child) = self.child.take() else {
            return Ok(None);
        };

        match child.try_wait() {
            Ok(Some(status)) => Ok(Some(status)),
            Ok(None) => {
                // Give the client a chance to handle SIGTERM and release its
                // Wayland resources before using an unconditional SIGKILL.
                let signal_result = unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) };
                let signal_error = (signal_result != 0).then(io::Error::last_os_error);
                let deadline = Instant::now() + GRACEFUL_STOP_TIMEOUT;

                while Instant::now() < deadline {
                    match child.try_wait() {
                        Ok(Some(status)) => return Ok(Some(status)),
                        Ok(None) => thread::sleep(STOP_POLL_INTERVAL),
                        Err(error) => {
                            let _ = child.kill();
                            let _ = child.wait();
                            return Err(error);
                        }
                    }
                }

                let kill_result = child.kill();
                match child.wait() {
                    Ok(status) => Ok(Some(status)),
                    Err(wait_error) => {
                        let detail = signal_error
                            .map(|error| format!("SIGTERM failed: {error}; "))
                            .unwrap_or_default();
                        let kill_detail = kill_result
                            .err()
                            .map(|error| format!("; SIGKILL failed: {error}"))
                            .unwrap_or_default();
                        Err(io::Error::new(
                            wait_error.kind(),
                            format!(
                                "{detail}cannot reap Welcome client ({wait_error}){kill_detail}"
                            ),
                        ))
                    }
                }
            }
            Err(error) => {
                // Still attempt to terminate and reap the child before
                // returning the original inspection error.
                let _ = child.kill();
                let _ = child.wait();
                Err(error)
            }
        }
    }
}

impl Drop for WelcomeClientProcess {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            eprintln!("Aurea Desktop: cannot cleanly reap Welcome client: {error}");
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::WelcomeClientProcess;
    use std::{process::Command, thread, time::Duration};

    #[test]
    fn stop_preserves_the_exit_status_of_an_exited_client() {
        let child = Command::new("sh")
            .args(["-c", "exit 7"])
            .spawn()
            .expect("test client should start");
        let mut client = WelcomeClientProcess { child: Some(child) };

        loop {
            if client
                .child
                .as_mut()
                .expect("child is present")
                .try_wait()
                .expect("test client status should be readable")
                .is_some()
            {
                break;
            }

            thread::sleep(Duration::from_millis(1));
        }

        let status = client.stop().expect("client should be reaped");

        assert_eq!(status.and_then(|status| status.code()), Some(7));
    }

    #[test]
    fn stop_terminates_and_reaps_a_running_client() {
        let child = Command::new("sh")
            .args(["-c", "exec sleep 30"])
            .spawn()
            .expect("test client should start");
        let mut client = WelcomeClientProcess { child: Some(child) };

        let status = client
            .stop()
            .expect("running client should be terminated and reaped");

        assert!(status.is_some_and(|status| !status.success()));
        assert!(client.stop().expect("stop is idempotent").is_none());
    }
}
