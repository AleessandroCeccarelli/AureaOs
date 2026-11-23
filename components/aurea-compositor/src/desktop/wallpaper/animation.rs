// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Backend-independent contracts and playback policy for animated wallpapers.
//!
//! Media decoders must run on a worker thread. The compositor render thread
//! consumes completed frames and must never call a decoder or wait for media I/O.

use std::time::Duration;

use super::WallpaperPixels;

/// Upper bound for wallpaper animation frame rate.
pub const MAX_WALLPAPER_FRAME_RATE: u8 = 60;
/// Lower bound avoids zero-rate pacing and division by zero.
pub const MIN_WALLPAPER_FRAME_RATE: u8 = 1;

/// Policy shared by image-sequence and video wallpaper providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallpaperPlaybackPolicy {
    max_frame_rate: u8,
    pause_when_obscured: bool,
    pause_when_output_disabled: bool,
    pause_when_session_locked: bool,
    pause_when_system_suspended: bool,
    respect_power_saving: bool,
    reduced_motion: bool,
    muted: bool,
}

impl WallpaperPlaybackPolicy {
    pub const fn max_frame_rate(self) -> u8 { self.max_frame_rate }
    pub const fn pause_when_obscured(self) -> bool { self.pause_when_obscured }
    pub const fn pause_when_output_disabled(self) -> bool { self.pause_when_output_disabled }
    pub const fn pause_when_session_locked(self) -> bool { self.pause_when_session_locked }
    pub const fn pause_when_system_suspended(self) -> bool { self.pause_when_system_suspended }
    pub const fn respect_power_saving(self) -> bool { self.respect_power_saving }
    pub const fn reduced_motion(self) -> bool { self.reduced_motion }
    pub const fn muted(self) -> bool { self.muted }

    /// Sets a bounded frame-rate ceiling.
    pub fn with_max_frame_rate(mut self, frames_per_second: u8) -> Option<Self> {
        if !(MIN_WALLPAPER_FRAME_RATE..=MAX_WALLPAPER_FRAME_RATE)
            .contains(&frames_per_second)
        {
            return None;
        }
        self.max_frame_rate = frames_per_second;
        Some(self)
    }

    pub const fn with_reduced_motion(mut self, enabled: bool) -> Self {
        self.reduced_motion = enabled;
        self
    }

    pub const fn frame_interval(self) -> Duration {
        Duration::from_nanos(1_000_000_000 / self.max_frame_rate as u64)
    }

    /// Evaluates whether animation should advance in the current desktop state.
    pub fn decision(self, environment: WallpaperPlaybackEnvironment) -> WallpaperPlaybackDecision {
        if self.reduced_motion {
            return WallpaperPlaybackDecision::Paused(WallpaperPauseReason::ReducedMotion);
        }
        if self.pause_when_system_suspended && environment.system_suspended {
            return WallpaperPlaybackDecision::Paused(WallpaperPauseReason::SystemSuspended);
        }
        if self.pause_when_session_locked && environment.session_locked {
            return WallpaperPlaybackDecision::Paused(WallpaperPauseReason::SessionLocked);
        }
        if self.pause_when_output_disabled && !environment.output_enabled {
            return WallpaperPlaybackDecision::Paused(WallpaperPauseReason::OutputDisabled);
        }
        if self.pause_when_obscured && !environment.visible {
            return WallpaperPlaybackDecision::Paused(WallpaperPauseReason::Obscured);
        }
        if self.respect_power_saving && environment.power_saving {
            return WallpaperPlaybackDecision::Paused(WallpaperPauseReason::PowerSaving);
        }
        WallpaperPlaybackDecision::Playing
    }
}

impl Default for WallpaperPlaybackPolicy {
    fn default() -> Self {
        Self {
            max_frame_rate: 30,
            pause_when_obscured: true,
            pause_when_output_disabled: true,
            pause_when_session_locked: true,
            pause_when_system_suspended: true,
            respect_power_saving: true,
            reduced_motion: false,
            muted: true,
        }
    }
}

/// Desktop conditions used to make a deterministic playback decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WallpaperPlaybackEnvironment {
    pub visible: bool,
    pub output_enabled: bool,
    pub session_locked: bool,
    pub system_suspended: bool,
    pub power_saving: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperPauseReason {
    ReducedMotion,
    SystemSuspended,
    SessionLocked,
    OutputDisabled,
    Obscured,
    PowerSaving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperPlaybackDecision {
    Playing,
    Paused(WallpaperPauseReason),
}

/// One decoded frame ready to be queued for GPU upload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallpaperFrame {
    index: u64,
    presentation_time: Duration,
    pixels: WallpaperPixels,
}

impl WallpaperFrame {
    pub fn new(index: u64, presentation_time: Duration, pixels: WallpaperPixels) -> Self {
        Self { index, presentation_time, pixels }
    }

    pub const fn index(&self) -> u64 { self.index }
    pub const fn presentation_time(&self) -> Duration { self.presentation_time }
    pub const fn pixels(&self) -> &WallpaperPixels { &self.pixels }
    pub fn into_pixels(self) -> WallpaperPixels { self.pixels }
}

/// Errors reported by a decoder worker, never by the Vulkan render pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperProviderError {
    DecodeFailed,
    InvalidFrame,
    ResourceLimitExceeded,
}

/// Backend-neutral source of timestamped frames.
///
/// Implementations may wrap GIF/APNG/WebP sequence decoders or video decoders.
/// Call this trait only from a dedicated worker; never from the compositor's
/// render thread. Returning None means the source reached its end.
pub trait WallpaperFrameProvider: Send {
    fn next_frame(&mut self) -> Result<Option<WallpaperFrame>, WallpaperProviderError>;
    fn restart(&mut self) -> Result<(), WallpaperProviderError>;
}

#[cfg(test)]
mod tests {
    use super::{
        WallpaperPauseReason, WallpaperPlaybackDecision, WallpaperPlaybackEnvironment,
        WallpaperPlaybackPolicy, MAX_WALLPAPER_FRAME_RATE,
    };

    fn visible_environment() -> WallpaperPlaybackEnvironment {
        WallpaperPlaybackEnvironment {
            visible: true,
            output_enabled: true,
            ..WallpaperPlaybackEnvironment::default()
        }
    }

    #[test]
    fn defaults_are_silent_and_pause_when_not_needed() {
        let policy = WallpaperPlaybackPolicy::default();
        assert!(policy.muted());
        assert!(policy.pause_when_obscured());
        assert_eq!(policy.max_frame_rate(), 30);
        assert_eq!(policy.decision(visible_environment()), WallpaperPlaybackDecision::Playing);
    }

    #[test]
    fn pauses_for_each_power_and_visibility_condition() {
        let policy = WallpaperPlaybackPolicy::default();
        let mut environment = visible_environment();

        environment.visible = false;
        assert_eq!(policy.decision(environment), WallpaperPlaybackDecision::Paused(WallpaperPauseReason::Obscured));

        environment = visible_environment();
        environment.output_enabled = false;
        assert_eq!(policy.decision(environment), WallpaperPlaybackDecision::Paused(WallpaperPauseReason::OutputDisabled));

        environment = visible_environment();
        environment.session_locked = true;
        assert_eq!(policy.decision(environment), WallpaperPlaybackDecision::Paused(WallpaperPauseReason::SessionLocked));

        environment = visible_environment();
        environment.system_suspended = true;
        assert_eq!(policy.decision(environment), WallpaperPlaybackDecision::Paused(WallpaperPauseReason::SystemSuspended));

        environment = visible_environment();
        environment.power_saving = true;
        assert_eq!(policy.decision(environment), WallpaperPlaybackDecision::Paused(WallpaperPauseReason::PowerSaving));
    }

    #[test]
    fn reduced_motion_always_pauses_animation() {
        let policy = WallpaperPlaybackPolicy::default().with_reduced_motion(true);
        assert_eq!(
            policy.decision(visible_environment()),
            WallpaperPlaybackDecision::Paused(WallpaperPauseReason::ReducedMotion)
        );
    }

    #[test]
    fn frame_rate_is_bounded_and_produces_a_stable_interval() {
        let policy = WallpaperPlaybackPolicy::default().with_max_frame_rate(60).unwrap();
        assert_eq!(policy.max_frame_rate(), MAX_WALLPAPER_FRAME_RATE);
        assert_eq!(policy.frame_interval().as_nanos(), 16_666_666);
        assert!(WallpaperPlaybackPolicy::default().with_max_frame_rate(0).is_none());
        assert!(WallpaperPlaybackPolicy::default().with_max_frame_rate(61).is_none());
    }
}
