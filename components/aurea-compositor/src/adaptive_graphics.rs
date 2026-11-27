// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Capability-based graphics-path policy for AureaOS.
//!
//! This module deliberately does not probe hardware or create Vulkan devices.
//! Linux/Smithay provide device discovery; the Vulkan layer supplies verified
//! capabilities here. Keeping policy separate prevents API-version checks from
//! being mistaken for proof that a renderer can run.

/// Rendering path selected from verified device capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsPath {
    /// Native renderer using Vulkan 1.3 core dynamic rendering and synchronization2.
    ModernVulkan,
    /// Hardware-accelerated renderer using the legacy render-pass/synchronization path.
    CompatibilityVulkan,
    /// CPU Vulkan device (for example, Lavapipe), using the modern renderer.
    SoftwareVulkan,
}

/// Vulkan and renderer capabilities verified for one physical device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphicsCapabilities {
    pub api_major: u32,
    pub api_minor: u32,
    pub dynamic_rendering: bool,
    pub synchronization2: bool,
    /// True only after a legacy render-pass backend is implemented and validated.
    pub compatibility_renderer_available: bool,
    /// True when Vulkan identifies this device as a CPU implementation.
    pub software_device: bool,
    pub has_graphics_queue: bool,
}

impl GraphicsCapabilities {
    fn supports_vulkan_13(self) -> bool {
        (self.api_major, self.api_minor) >= (1, 3)
            && self.dynamic_rendering
            && self.synchronization2
            && self.has_graphics_queue
    }
}

/// A discovered GPU together with capabilities queried from its Vulkan device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphicsCandidate {
    /// Stable index into the VulkanContext physical-device inventory.
    pub device_index: usize,
    pub name: String,
    pub capabilities: GraphicsCapabilities,
}

/// Result of the automatic graphics-path decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveGraphicsPlan {
    pub device_index: usize,
    pub device_name: String,
    pub path: GraphicsPath,
}

/// Aurea Adaptive Graphics policy.
///
/// Candidates must contain queried capabilities, not assumptions based on
/// vendor names or marketing model numbers. Hardware acceleration is preferred;
/// a CPU Vulkan device is considered only after hardware candidates.
#[derive(Debug, Default, Clone, Copy)]
pub struct AureaAdaptiveGraphics;

impl AureaAdaptiveGraphics {
    /// Selects the strongest implemented path from the supplied device inventory.
    ///
    /// The current compositor implements only the modern Vulkan path. The
    /// compatibility branch remains dormant until its legacy renderer exists;
    /// it must never be selected merely because a GPU exposes Vulkan 1.0–1.2.
    pub fn select(candidates: &[GraphicsCandidate]) -> Option<AdaptiveGraphicsPlan> {
        candidates
            .iter()
            .filter(|candidate| !candidate.capabilities.software_device)
            .find(|candidate| candidate.capabilities.supports_vulkan_13())
            .map(|candidate| AdaptiveGraphicsPlan {
                device_index: candidate.device_index,
                device_name: candidate.name.clone(),
                path: GraphicsPath::ModernVulkan,
            })
            .or_else(|| {
                candidates
                    .iter()
                    .filter(|candidate| !candidate.capabilities.software_device)
                    .find(|candidate| {
                        candidate.capabilities.has_graphics_queue
                            && candidate.capabilities.compatibility_renderer_available
                    })
                    .map(|candidate| AdaptiveGraphicsPlan {
                        device_index: candidate.device_index,
                        device_name: candidate.name.clone(),
                        path: GraphicsPath::CompatibilityVulkan,
                    })
            })
            .or_else(|| {
                candidates
                    .iter()
                    .filter(|candidate| candidate.capabilities.software_device)
                    .find(|candidate| candidate.capabilities.supports_vulkan_13())
                    .map(|candidate| AdaptiveGraphicsPlan {
                        device_index: candidate.device_index,
                        device_name: candidate.name.clone(),
                        path: GraphicsPath::SoftwareVulkan,
                    })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(
        device_index: usize,
        name: &str,
        software: bool,
        modern: bool,
        compatibility: bool,
    ) -> GraphicsCandidate {
        GraphicsCandidate {
            device_index,
            name: name.to_owned(),
            capabilities: GraphicsCapabilities {
                api_major: 1,
                api_minor: if modern { 3 } else { 2 },
                dynamic_rendering: modern,
                synchronization2: modern,
                compatibility_renderer_available: compatibility,
                software_device: software,
                has_graphics_queue: true,
            },
        }
    }

    #[test]
    fn prefers_modern_hardware_over_software() {
        let candidates = [
            candidate(1, "Lavapipe", true, true, false),
            candidate(0, "Integrated GPU", false, true, false),
        ];
        let plan = AureaAdaptiveGraphics::select(&candidates).unwrap();
        assert_eq!(plan.device_name, "Integrated GPU");
        assert_eq!(plan.device_index, 0);
        assert_eq!(plan.path, GraphicsPath::ModernVulkan);
    }

    #[test]
    fn rejects_vulkan_13_devices_missing_required_core_features() {
        let mut candidate = candidate(5, "Incomplete Vulkan 1.3 GPU", false, true, false);
        candidate.capabilities.synchronization2 = false;
        assert!(AureaAdaptiveGraphics::select(&[candidate]).is_none());

        let mut candidate = candidate(6, "GPU without dynamic rendering", false, true, false);
        candidate.capabilities.dynamic_rendering = false;
        assert!(AureaAdaptiveGraphics::select(&[candidate]).is_none());
    }

    #[test]
    fn selects_compatibility_hardware_only_when_legacy_backend_is_available() {
        let candidates = [candidate(2, "Legacy GPU", false, false, true)];
        let plan = AureaAdaptiveGraphics::select(&candidates).unwrap();
        assert_eq!(plan.device_index, 2);
        assert_eq!(plan.path, GraphicsPath::CompatibilityVulkan);
    }

    #[test]
    fn does_not_select_legacy_hardware_without_legacy_backend() {
        let candidates = [candidate(3, "Older GPU", false, false, false)];
        assert!(AureaAdaptiveGraphics::select(&candidates).is_none());
    }

    #[test]
    fn does_not_claim_legacy_support_without_legacy_renderer() {
        let candidates = [candidate(3, "Older GPU", false, false, false)];
        assert!(AureaAdaptiveGraphics::select(&candidates).is_none());
    }

    #[test]
    fn software_is_a_fallback_not_a_hardware_replacement() {
        let candidates = [candidate(4, "Lavapipe", true, true, false)];
        let plan = AureaAdaptiveGraphics::select(&candidates).unwrap();
        assert_eq!(plan.path, GraphicsPath::SoftwareVulkan);
        assert_eq!(plan.device_index, 4);
    }
}
