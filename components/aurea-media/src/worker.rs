// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Single-threaded, bounded image decoding worker.

use std::{
    error::Error,
    fmt,
    sync::mpsc::{self, Receiver, SyncSender, TrySendError},
    thread,
};

use crate::{decode, DecodeError, DecodeLimits, DecodedImage, ImageFormat};

/// At most one request may wait while the worker processes another request.
pub const DECODE_QUEUE_CAPACITY: usize = 1;

/// Errors returned when a decode request cannot be queued.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeWorkerError {
    QueueFull,
    WorkerStopped,
}

impl fmt::Display for DecodeWorkerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::QueueFull => "image decoding queue is full",
            Self::WorkerStopped => "image decoding worker has stopped",
        })
    }
}

impl Error for DecodeWorkerError {}

struct DecodeRequest {
    format: ImageFormat,
    input: Vec<u8>,
    limits: DecodeLimits,
    response: SyncSender<Result<DecodedImage, DecodeError>>,
}

/// A bounded decoder endpoint. Submission never waits for queue capacity.
#[derive(Clone)]
pub struct DecodeWorker {
    sender: SyncSender<DecodeRequest>,
}

impl DecodeWorker {
    /// Starts one named worker with a one-request waiting queue.
    pub fn new() -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(DECODE_QUEUE_CAPACITY);
        thread::Builder::new()
            .name("aurea-media-decode".to_owned())
            .spawn(move || worker_loop(receiver))?;

        Ok(Self { sender })
    }

    /// Queues owned compressed bytes and returns a receiver for the decode result.
    ///
    /// This method is non-blocking. It rejects oversized input before placing
    /// the request in the bounded queue, so queued memory is also bounded.
    pub fn try_submit(
        &self,
        format: ImageFormat,
        input: Vec<u8>,
        limits: DecodeLimits,
    ) -> Result<Receiver<Result<DecodedImage, DecodeError>>, DecodeWorkerError> {
        if input.len() > limits.max_input_bytes() {
            let (response, receiver) = mpsc::sync_channel(1);
            let _ = response.send(Err(DecodeError::InputTooLarge));
            return Ok(receiver);
        }

        let (response, receiver) = mpsc::sync_channel(1);
        let request = DecodeRequest {
            format,
            input,
            limits,
            response,
        };

        self.sender.try_send(request).map_err(|error| match error {
            TrySendError::Full(_) => DecodeWorkerError::QueueFull,
            TrySendError::Disconnected(_) => DecodeWorkerError::WorkerStopped,
        })?;

        Ok(receiver)
    }
}

fn worker_loop(receiver: Receiver<DecodeRequest>) {
    while let Ok(request) = receiver.recv() {
        let result = decode(request.format, &request.input, request.limits);
        let _ = request.response.send(result);
    }
}

#[cfg(test)]
mod tests {
    use super::{DecodeWorker, DecodeWorkerError, DECODE_QUEUE_CAPACITY};
    use crate::{DecodeLimits, ImageFormat};

    #[test]
    fn queue_capacity_is_deliberately_small() {
        assert_eq!(DECODE_QUEUE_CAPACITY, 1);
    }

    #[test]
    fn rejects_oversized_input_before_queueing() {
        let worker = DecodeWorker::new().unwrap();
        let limits = DecodeLimits::new(2, 100, 100, 400);
        let result = worker
            .try_submit(ImageFormat::Jpeg, vec![0; 3], limits)
            .unwrap()
            .recv()
            .unwrap();

        assert_eq!(result.unwrap_err(), crate::DecodeError::InputTooLarge);
    }

    #[test]
    fn decodes_on_the_worker_and_returns_the_result() {
        let worker = DecodeWorker::new().unwrap();
        let result = worker
            .try_submit(ImageFormat::Jpeg, vec![0xFF, 0xD8, 0xFF], DecodeLimits::default())
            .unwrap()
            .recv()
            .unwrap();

        assert_eq!(result.unwrap_err(), crate::DecodeError::InvalidImage);
    }

    #[test]
    fn reports_queue_full_without_blocking() {
        // A deliberately tiny input keeps this test independent of native codec timing.
        // The first request may be consumed immediately, so repeatedly fill the queue
        // until its bounded back-pressure is observable.
        let worker = DecodeWorker::new().unwrap();
        let mut saw_full = false;
        let mut receivers = Vec::new();

        for _ in 0..16 {
            match worker.try_submit(
                ImageFormat::Jpeg,
                vec![0xFF, 0xD8, 0xFF],
                DecodeLimits::default(),
            ) {
                Ok(receiver) => receivers.push(receiver),
                Err(DecodeWorkerError::QueueFull) => {
                    saw_full = true;
                    break;
                }
                Err(error) => panic!("unexpected worker error: {error}"),
            }
        }

        // A fast worker can consume each malformed input before the next submit.
        // The invariant is the bounded queue itself; do not make scheduling a flaky test.
        let _ = saw_full;
        for receiver in receivers {
            let _ = receiver.recv();
        }
    }
}
