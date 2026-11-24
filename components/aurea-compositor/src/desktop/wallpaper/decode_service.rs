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

use aurea_media::{DecodeError, DecodeLimits, DecodeWorker, DecodeWorkerError, ImageFormat};

use super::{WallpaperAsset, WallpaperFormat, WallpaperPixels, WallpaperPixelsError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallpaperDecodeKey {
    asset_id: String,
    revision: u64,
}

impl WallpaperDecodeKey {
    pub fn asset_id(&self) -> &str { &self.asset_id }
    pub const fn revision(&self) -> u64 { self.revision }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedWallpaper {
    key: WallpaperDecodeKey,
    pixels: WallpaperPixels,
}

impl DecodedWallpaper {
    pub fn key(&self) -> &WallpaperDecodeKey { &self.key }
    pub const fn pixels(&self) -> &WallpaperPixels { &self.pixels }
    pub fn into_pixels(self) -> WallpaperPixels { self.pixels }
}

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

pub struct WallpaperDecodeService {
    worker: DecodeWorker,
    requested_asset: Option<String>,
    revision: u64,
    pending: Option<(WallpaperDecodeKey, Receiver<DecodeResponse>)>,
    ready: Option<DecodedWallpaper>,
    completed: Option<WallpaperDecodeKey>,
    failed: Option<WallpaperDecodeKey>,
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
            completed: None,
            failed: None,
        })
    }

    pub fn select(
        &mut self,
        asset: Option<WallpaperAsset>,
    ) -> Result<Option<WallpaperDecodeKey>, WallpaperDecodeServiceError> {
        let asset_id = asset.as_ref().map(|asset| asset.id().to_owned());
        if asset_id == self.requested_asset {
            if let Some(completed) = self.completed.as_ref() { return Ok(Some(completed.clone())); }
            if let Some(failed) = self.failed.as_ref() { return Ok(Some(failed.clone())); }
            if let Some(ready) = self.ready.as_ref() { return Ok(Some(ready.key().clone())); }
            if let Some((key, _)) = self.pending.as_ref() { return Ok(Some(key.clone())); }
            if let Some(asset) = asset { return self.submit(asset).map(Some); }
            return Ok(None);
        }

        self.revision = self.revision.checked_add(1)
            .ok_or(WallpaperDecodeServiceError::RevisionExhausted)?;
        self.requested_asset = asset_id;
        self.pending = None;
        self.ready = None;
        self.completed = None;
        self.failed = None;

        match asset {
            Some(asset) => self.submit(asset).map(Some),
            None => Ok(None),
        }
    }

    fn submit(
        &mut self,
        asset: WallpaperAsset,
    ) -> Result<WallpaperDecodeKey, WallpaperDecodeServiceError> {
        let format = match asset.format() {
            WallpaperFormat::Jpeg => ImageFormat::Jpeg,
            WallpaperFormat::Png => ImageFormat::Png,
            WallpaperFormat::Bmp => ImageFormat::Bmp,
            WallpaperFormat::Gif => ImageFormat::Gif,
            WallpaperFormat::Tiff => ImageFormat::Tiff,
            WallpaperFormat::WebP => ImageFormat::WebP,
        };
        let key = WallpaperDecodeKey {
            asset_id: asset.id().to_owned(),
            revision: self.revision,
        };
        let response = self.worker
            .try_submit(format, asset.bytes().to_vec(), DecodeLimits::default())
            .map_err(WallpaperDecodeServiceError::Worker)?;
        self.pending = Some((key.clone(), response));
        Ok(key)
    }

    pub fn poll(&mut self) -> Result<Option<&DecodedWallpaper>, WallpaperDecodeServiceError> {
        let Some((key, receiver)) = self.pending.as_ref() else {
            return Ok(self.ready.as_ref());
        };

        match receiver.try_recv() {
            Ok(result) => {
                let key = key.clone();
                self.pending = None;
                let decoded = match result {
                    Ok(decoded) => decoded,
                    Err(error) => {
                        self.failed = Some(key);
                        return Err(WallpaperDecodeServiceError::Decode(error));
                    }
                };
                let pixels = match WallpaperPixels::new(
                    decoded.width(),
                    decoded.height(),
                    decoded.into_pixels(),
                ) {
                    Ok(pixels) => pixels,
                    Err(error) => {
                        self.failed = Some(key);
                        return Err(WallpaperDecodeServiceError::InvalidPixels(error));
                    }
                };
                if self.requested_asset.as_deref() == Some(key.asset_id())
                    && self.revision == key.revision()
                {
                    self.ready = Some(DecodedWallpaper { key, pixels });
                }
                Ok(self.ready.as_ref())
            }
            Err(TryRecvError::Empty) => Ok(self.ready.as_ref()),
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                self.failed = self.requested_asset.as_ref().map(|asset_id| WallpaperDecodeKey {
                    asset_id: asset_id.clone(),
                    revision: self.revision,
                });
                Err(WallpaperDecodeServiceError::Worker(DecodeWorkerError::WorkerStopped))
            }
        }
    }

    pub fn take_ready(&mut self) -> Option<DecodedWallpaper> {
        let ready = self.ready.take()?;
        self.completed = Some(ready.key().clone());
        Some(ready)
    }

    pub fn ready(&self) -> Option<&DecodedWallpaper> { self.ready.as_ref() }

    pub fn pending_key(&self) -> Option<&WallpaperDecodeKey> {
        self.pending.as_ref().map(|(key, _)| key)
    }
}

#[cfg(test)]
mod tests {
    use super::WallpaperDecodeService;
    use crate::desktop::wallpaper::WallpaperAsset;

    #[test]
    fn selected_wallpaper_decodes_off_the_calling_thread() {
        let asset = WallpaperAsset::default_wallpaper().unwrap();
        let expected_dimensions = asset.dimensions().unwrap();
        let mut service = WallpaperDecodeService::new().unwrap();
        let key = service.select(Some(asset)).unwrap().unwrap();

        let mut completed = false;
        for _ in 0..10_000 {
            if let Some(decoded) = service.poll().unwrap() {
                assert_eq!(decoded.key(), &key);
                assert_eq!(decoded.pixels().width(), expected_dimensions.width());
                assert_eq!(decoded.pixels().height(), expected_dimensions.height());
                completed = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
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
            if service.pending_key().is_none() { break; }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(service.ready().is_none());
    }

    #[test]
    fn selecting_the_same_asset_does_not_create_a_new_revision() {
        let asset = WallpaperAsset::default_wallpaper().unwrap();
        let mut service = WallpaperDecodeService::new().unwrap();
        let first = service.select(Some(asset.clone())).unwrap().unwrap();
        let second = service.select(Some(asset)).unwrap().unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn user_selected_asset_is_submitted_to_the_same_worker() {
        let asset = WallpaperAsset::from_bytes("user-wallpaper", b"BM".to_vec()).unwrap();
        let mut service = WallpaperDecodeService::new().unwrap();
        let key = service.select(Some(asset)).unwrap().unwrap();
        assert_eq!(key.asset_id(), "user-wallpaper");
    }
}
