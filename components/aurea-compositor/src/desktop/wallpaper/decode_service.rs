// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinates bounded background decoding for the selected desktop wallpaper.

use std::{
    error::Error,
    fmt,
    sync::mpsc::{Receiver, TryRecvError},
};

use aurea_media::{
    DecodeError, DecodeLimits, DecodeWorker, DecodeWorkerError, ImageFormat,
};

use super::{WallpaperAsset, WallpaperFormat, WallpaperPixels, WallpaperPixelsError};

/// Identifies one selected wallpaper decode generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallpaperDecodeKey {
    asset_id: &'static str,
    revision: u64,
}

impl WallpaperDecodeKey {
    pub const fn asset_id(self) -> &'static str {
        self.asset_id
    }

    pub const fn revision(self) -> u64 {
        self.revision
    }
}

/// A decoded wallpaper ready to be handed to the renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedWallpaper {
    key: WallpaperDecodeKey,
    pixels: WallpaperPixels,
}

impl DecodedWallpaper {
    pub const fn key(&self) -> WallpaperDecodeKey {
        self.key
    }

    pub const fn pixels(&self) -> &WallpaperPixels {
        &self.pixels
    }

    pub fn into_pixels(self) -> WallpaperPixels {
        self.pixels
    }
}

/// Errors raised while starting or submitting wallpaper decode work.
#[derive(Debug)]
pub enum WallpaperDecodeServiceError {
    WorkerStart(std::io::Error),
    Worker(DecodeWorkerError),
    Decode(DecodeError),
    InvalidPixels(WallpaperPixelsError),
    UnsupportedFormat,
    RevisionExhausted,
}

impl fmt::Display for WallpaperDecodeServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkerStart(error) => write!(formatter, "Wallpaper decode worker: {error}"),
            Self::Worker(error) => write!(formatter, "Wallpaper decode submission: {error}"),
            Self::Decode(error) => write!(formatter, "Wallpaper image decode: {error}"),
            Self::InvalidPixels(error) => write!(formatter, "Wallpaper pixel validation: {error}"),
            Self::UnsupportedFormat => formatter.write_str("Unsupported wallpaper image format"),
            Self::RevisionExhausted => formatter.write_str("Wallpaper revision counter exhausted"),
        }
    }
}

impl Error for WallpaperDecodeServiceError {}

type DecodeResponse = Result<aurea_media::DecodedImage, DecodeError>;

/// Owns the request lifecycle for one selected wallpaper.
///
/// The service never waits for decoding. A changed selection invalidates the
/// previous receiver; late results are discarded instead of being published.
pub struct WallpaperDecodeService {
    worker: DecodeWorker,
    requested_asset: Option<&'static str>,
    revision: u64,
    pending: Option<(WallpaperDecodeKey, Receiver<DecodeResponse>)>,
    ready: Option<DecodedWallpaper>,
}

impl WallpaperDecodeService {
    pub fn new() -> Result<Self, WallpaperDecodeServiceError> {
        let worker = DecodeWorker::new().map_err(WallpaperDecodeServiceError::WorkerStart)?;
        Ok(Self {
            worker,
            requested_asset: None,
            revision: 0,
            pending: None,
            ready: None,
        })
    }

    /// Synchronizes decode work with the currently selected wallpaper.
    ///
    /// Calling repeatedly with the same asset is idempotent. A changed asset
    /// increments the revision and invalidates both pending and ready results.
    pub fn select(
        &mut self,
        asset: Option<WallpaperAsset>,
    ) -> Result<Option<WallpaperDecodeKey>, WallpaperDecodeServiceError> {
        let asset_id = asset.map(WallpaperAsset::id);
        if asset_id == self.requested_asset {
            if let Some(ready) = self.ready.as_ref() {
                return Ok(Some(ready.key()));
            }
            if let Some((key, _)) = self.pending.as_ref() {
                return Ok(Some(*key));
            }
            if let Some(asset) = asset {
                return self.submit(asset).map(Some);
            }
            return Ok(None);
        }

        self.revision = self
            .revision
            .checked_add(1)
            .ok_or(WallpaperDecodeServiceError::RevisionExhausted)?;
        self.requested_asset = asset_id;
        self.pending = None;
        self.ready = None;

        let Some(asset) = asset else {
            return Ok(None);
        };

        self.submit(asset).map(Some)
    }

    fn submit(
        &mut self,
        asset: WallpaperAsset,
    ) -> Result<WallpaperDecodeKey, WallpaperDecodeServiceError> {
        let format = match asset.format() {
            WallpaperFormat::Jpeg => ImageFormat::Jpeg,
        };
        let key = WallpaperDecodeKey {
            asset_id: asset.id(),
            revision: self.revision,
        };
        let response = self
            .worker
            .try_submit(format, asset.bytes().to_vec(), DecodeLimits::default())
            .map_err(WallpaperDecodeServiceError::Worker)?;
        self.pending = Some((key, response));
        Ok(key)
    }

    /// Polls once without blocking and publishes only the current generation.
    pub fn poll(&mut self) -> Result<Option<&DecodedWallpaper>, WallpaperDecodeServiceError> {
        let Some((key, receiver)) = self.pending.as_ref() else {
            return Ok(self.ready.as_ref());
        };

        match receiver.try_recv() {
            Ok(result) => {
                let key = *key;
                self.pending = None;
                let decoded = result.map_err(WallpaperDecodeServiceError::Decode)?;
                let pixels = WallpaperPixels::new(
                    decoded.width(),
                    decoded.height(),
                    decoded.into_pixels(),
                )
                .map_err(WallpaperDecodeServiceError::InvalidPixels)?;
                if self.requested_asset == Some(key.asset_id()) && self.revision == key.revision()
                {
                    self.ready = Some(DecodedWallpaper { key, pixels });
                }
                Ok(self.ready.as_ref())
            }
            Err(TryRecvError::Empty) => Ok(self.ready.as_ref()),
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                Err(WallpaperDecodeServiceError::Worker(
                    DecodeWorkerError::WorkerStopped,
                ))
            }
        }
    }

    pub fn ready(&self) -> Option<&DecodedWallpaper> {
        self.ready.as_ref()
    }

    pub const fn pending_key(&self) -> Option<WallpaperDecodeKey> {
        match &self.pending {
            Some((key, _)) => Some(*key),
            None => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::WallpaperDecodeService;
    use crate::desktop::wallpaper::WallpaperAsset;

    #[test]
    fn selected_wallpaper_decodes_off_the_calling_thread() {
        let asset = WallpaperAsset::default_wallpaper().unwrap();
        let mut service = WallpaperDecodeService::new().unwrap();
        let key = service.select(Some(asset)).unwrap().unwrap();

        let mut completed = false;
        for _ in 0..10_000 {
            if let Some(decoded) = service.poll().unwrap() {
                assert_eq!(decoded.key(), key);
                assert_eq!(decoded.pixels().width(), u32::from(asset.dimensions().width()));
                assert_eq!(decoded.pixels().height(), u32::from(asset.dimensions().height()));
                completed = true;
                break;
            }
            std::thread::yield_now();
        }

        assert!(completed, "wallpaper decode should complete");
    }

    #[test]
    fn clearing_selection_invalidates_an_in_flight_decode() {
        let asset = WallpaperAsset::default_wallpaper().unwrap();
        let mut service = WallpaperDecodeService::new().unwrap();
        service.select(Some(asset)).unwrap();
        service.select(None).unwrap();

        for _ in 0..10_000 {
            assert!(service.poll().unwrap().is_none());
            if service.pending_key().is_none() {
                break;
            }
            std::thread::yield_now();
        }
        assert!(service.ready().is_none());
    }

    #[test]
    fn selecting_the_same_asset_does_not_create_a_new_revision() {
        let asset = WallpaperAsset::default_wallpaper().unwrap();
        let mut service = WallpaperDecodeService::new().unwrap();
        let first = service.select(Some(asset)).unwrap().unwrap();
        let second = service.select(Some(asset)).unwrap().unwrap();
        assert_eq!(first, second);
    }
}
