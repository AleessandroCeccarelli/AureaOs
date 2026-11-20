// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

use super::*;

#[test]
fn quarter_turn_output_transform_maps_logical_extent_to_native_scanout() {
    let geometry = SurfaceGeometry {
        x: 10,
        y: 20,
        width: 30,
        height: 40,
    };

    assert_eq!(
        transform_geometry_for_output(geometry, 1920, 1080, Transform::_90),
        Some(SurfaceGeometry {
            x: 1860,
            y: 10,
            width: 40,
            height: 30,
        })
    );
    assert_eq!(
        transform_geometry_for_output(geometry, 1920, 1080, Transform::_270),
        Some(SurfaceGeometry {
            x: 20,
            y: 1040,
            width: 40,
            height: 30,
        })
    );
}

#[test]
fn output_transform_preserves_native_extent_for_half_turns() {
    let geometry = SurfaceGeometry {
        x: 10,
        y: 20,
        width: 30,
        height: 40,
    };

    assert_eq!(
        transform_geometry_for_output(geometry, 1920, 1080, Transform::_180),
        Some(SurfaceGeometry {
            x: 1880,
            y: 1020,
            width: 30,
            height: 40,
        })
    );
}

#[test]
fn output_rotation_maps_rectangles_without_changing_scanout_extent() {
    let geometry = SurfaceGeometry {
        x: 10,
        y: 20,
        width: 30,
        height: 40,
    };

    assert_eq!(
        transform_surface_geometry(geometry, 200, 100, Transform::_90),
        Some(SurfaceGeometry {
            x: 40,
            y: 10,
            width: 40,
            height: 30,
        })
    );
    assert_eq!(
        transform_surface_geometry(geometry, 200, 100, Transform::_180),
        Some(SurfaceGeometry {
            x: 160,
            y: 40,
            width: 30,
            height: 40,
        })
    );
    assert_eq!(
        transform_surface_geometry(geometry, 200, 100, Transform::_270),
        Some(SurfaceGeometry {
            x: 20,
            y: 160,
            width: 40,
            height: 30,
        })
    );
}

#[test]
fn output_reflections_map_surface_rectangles_inside_target() {
    let geometry = SurfaceGeometry {
        x: 10,
        y: 20,
        width: 30,
        height: 40,
    };

    assert_eq!(
        transform_surface_geometry(geometry, 200, 100, Transform::Flipped),
        Some(SurfaceGeometry {
            x: 160,
            y: 20,
            width: 30,
            height: 40,
        })
    );
    assert_eq!(
        transform_surface_geometry(geometry, 200, 100, Transform::Flipped90),
        Some(SurfaceGeometry {
            x: 20,
            y: 10,
            width: 40,
            height: 30,
        })
    );
    assert_eq!(
        transform_surface_geometry(geometry, 200, 100, Transform::Flipped180),
        Some(SurfaceGeometry {
            x: 10,
            y: 40,
            width: 30,
            height: 40,
        })
    );
    assert_eq!(
        transform_surface_geometry(geometry, 200, 100, Transform::Flipped270),
        Some(SurfaceGeometry {
            x: 40,
            y: 160,
            width: 40,
            height: 30,
        })
    );
}

#[test]
fn composed_surface_transforms_match_sequential_uv_mapping() {
    let transforms = [
        Transform::Normal,
        Transform::_90,
        Transform::_180,
        Transform::_270,
        Transform::Flipped,
        Transform::Flipped90,
        Transform::Flipped180,
        Transform::Flipped270,
    ];
    let probes = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)];

    for buffer in transforms {
        for output in transforms {
            let composed = compose_surface_transforms(buffer, output).unwrap();
            for uv in probes {
                let sequential = buffer_uv_from_surface_uv(output, uv)
                    .and_then(|uv| buffer_uv_from_surface_uv(buffer, uv));
                assert_eq!(buffer_uv_from_surface_uv(composed, uv), sequential);
            }
        }
    }
}

#[test]
fn cursor_scale_keeps_global_hotspot_anchor_fixed() {
    let geometry = surface_geometry((90, 90), (20, 20), 1, Transform::Normal, (0, 0), 1).unwrap();
    let scaled = scale_surface_geometry(geometry, (100, 100), (0, 0), 1, 2.0).unwrap();

    assert_eq!((scaled.x(), scaled.y()), (80, 80));
    assert_eq!((scaled.width(), scaled.height()), (40, 40));
}

#[test]
fn scaling_rejects_invalid_factors() {
    let geometry = surface_geometry((0, 0), (10, 10), 1, Transform::Normal, (0, 0), 1).unwrap();

    assert!(scale_surface_geometry(geometry, (0, 0), (0, 0), 1, f64::NAN).is_none());
    assert!(scale_surface_geometry(geometry, (0, 0), (0, 0), 1, 3.5).is_none());
}

#[test]
fn translates_global_position_and_applies_both_scales() {
    let geometry =
        surface_geometry((110, 70), (80, 40), 2, Transform::Normal, (100, 50), 2).unwrap();

    assert_eq!(
        geometry,
        SurfaceGeometry {
            x: 20,
            y: 40,
            width: 80,
            height: 40,
        }
    );
}

#[test]
fn quarter_turn_transform_swaps_destination_dimensions() {
    let geometry = surface_geometry((0, 0), (80, 40), 1, Transform::_90, (0, 0), 1).unwrap();

    assert_eq!((geometry.width(), geometry.height()), (40, 80));
}

#[test]
fn rounds_scaled_buffer_dimensions_to_nearest_physical_pixel() {
    let geometry = surface_geometry((0, 0), (5, 3), 2, Transform::Normal, (0, 0), 1).unwrap();

    assert_eq!((geometry.width(), geometry.height()), (3, 2));
}

#[test]
fn fractional_cursor_scale_keeps_hotspot_fixed_on_output() {
    let output_scale = ScaleFactor::from_percent(125).unwrap();
    let geometry = surface_geometry_with_fractional_scale(
        (80, 80),
        (20, 20),
        1,
        Transform::Normal,
        (0, 0),
        output_scale,
    )
    .unwrap();
    let scaled = scale_surface_geometry_with_fractional_scale(
        geometry,
        (100, 100),
        (0, 0),
        output_scale,
        2.0,
    )
    .unwrap();

    assert_eq!((scaled.x(), scaled.y()), (75, 75));
    assert_eq!((scaled.width(), scaled.height()), (50, 50));
}

#[test]
fn fractional_output_scale_controls_surface_origin_and_extent() {
    let scale = ScaleFactor::from_percent(125).unwrap();
    let geometry = surface_geometry_with_fractional_scale(
        (10, -10),
        (800, 600),
        1,
        Transform::Normal,
        (0, 0),
        scale,
    )
    .unwrap();

    assert_eq!(
        (
            geometry.x(),
            geometry.y(),
            geometry.width(),
            geometry.height(),
        ),
        (13, -13, 1000, 750)
    );
}

#[test]
fn fractional_scale_keeps_multi_output_origin_local() {
    let scale = ScaleFactor::from_percent(150).unwrap();
    let geometry = surface_geometry_with_fractional_scale(
        (1700, 100),
        (800, 600),
        1,
        Transform::Normal,
        (1920, 0),
        scale,
    )
    .unwrap();

    assert_eq!((geometry.x(), geometry.y()), (-330, 150));
    assert_eq!((geometry.width(), geometry.height()), (1200, 900));
    assert_eq!(
        geometry.scissor(1920, 1080),
        Some(SurfaceScissor {
            x: 0,
            y: 150,
            width: 870,
            height: 900,
        })
    );
}

#[test]
fn clips_negative_surface_origin_without_changing_destination() {
    let geometry = surface_geometry((-10, 5), (20, 10), 1, Transform::Normal, (0, 0), 1).unwrap();

    assert_eq!(
        geometry.scissor(100, 100),
        Some(SurfaceScissor {
            x: 0,
            y: 5,
            width: 10,
            height: 10,
        })
    );
    assert_eq!(geometry.x(), -10);
}

#[test]
fn clipping_each_output_edge_preserves_full_surface_geometry() {
    let cases = [
        (
            SurfaceGeometry {
                x: -5,
                y: 10,
                width: 20,
                height: 30,
            },
            (-5, 10, 20, 30),
            SurfaceScissor {
                x: 0,
                y: 10,
                width: 15,
                height: 30,
            },
        ),
        (
            SurfaceGeometry {
                x: 90,
                y: 10,
                width: 20,
                height: 30,
            },
            (90, 10, 20, 30),
            SurfaceScissor {
                x: 90,
                y: 10,
                width: 10,
                height: 30,
            },
        ),
        (
            SurfaceGeometry {
                x: 10,
                y: -5,
                width: 20,
                height: 30,
            },
            (10, -5, 20, 30),
            SurfaceScissor {
                x: 10,
                y: 0,
                width: 20,
                height: 25,
            },
        ),
        (
            SurfaceGeometry {
                x: 10,
                y: 90,
                width: 20,
                height: 30,
            },
            (10, 90, 20, 30),
            SurfaceScissor {
                x: 10,
                y: 90,
                width: 20,
                height: 10,
            },
        ),
    ];

    for (geometry, expected_geometry, expected_scissor) in cases {
        assert_eq!(
            (
                geometry.x(),
                geometry.y(),
                geometry.width(),
                geometry.height(),
            ),
            expected_geometry
        );
        assert_eq!(geometry.scissor(100, 100), Some(expected_scissor));
    }
}

#[test]
fn one_surface_is_clipped_independently_for_outputs_with_different_scales() {
    let left_output =
        surface_geometry((1700, 100), (800, 600), 1, Transform::Normal, (0, 0), 1).unwrap();
    let right_output =
        surface_geometry((1700, 100), (800, 600), 1, Transform::Normal, (1920, 0), 2).unwrap();

    assert_eq!((left_output.x(), left_output.width()), (1700, 800));
    assert_eq!(
        left_output.scissor(1920, 1080),
        Some(SurfaceScissor {
            x: 1700,
            y: 100,
            width: 220,
            height: 600,
        })
    );

    assert_eq!((right_output.x(), right_output.width()), (-440, 1600));
    assert_eq!(
        right_output.scissor(2560, 2160),
        Some(SurfaceScissor {
            x: 0,
            y: 200,
            width: 1160,
            height: 1200,
        })
    );
}

#[test]
fn skips_surfaces_outside_output_bounds() {
    let geometry = surface_geometry((120, 10), (20, 20), 1, Transform::Normal, (0, 0), 1).unwrap();

    assert_eq!(geometry.scissor(100, 100), None);
}

#[test]
fn rejects_invalid_scale_and_zero_sized_buffers() {
    assert!(surface_geometry((0, 0), (10, 10), 0, Transform::Normal, (0, 0), 1).is_none());
    assert!(surface_geometry((0, 0), (10, 10), 1, Transform::Normal, (0, 0), 0).is_none());
    assert!(surface_geometry((0, 0), (0, 10), 1, Transform::Normal, (0, 0), 1).is_none());
}
#[test]
fn buffer_uv_mapping_applies_inverse_wayland_transform_for_all_eight_cases() {
    let surface_uv = (0.2, 0.3);
    let cases = [
        (Transform::Normal, (0.2, 0.3)),
        (Transform::_90, (0.3, 0.8)),
        (Transform::_180, (0.8, 0.7)),
        (Transform::_270, (0.7, 0.2)),
        (Transform::Flipped, (0.8, 0.3)),
        (Transform::Flipped90, (0.3, 0.2)),
        (Transform::Flipped180, (0.2, 0.7)),
        (Transform::Flipped270, (0.7, 0.8)),
    ];

    for (transform, expected) in cases {
        let actual = buffer_uv_from_surface_uv(transform, surface_uv).unwrap();
        assert!(
            (actual.0 - expected.0).abs() < f32::EPSILON
                && (actual.1 - expected.1).abs() < f32::EPSILON,
            "unexpected UV mapping for {transform:?}: {actual:?}"
        );
    }
}

#[test]
fn buffer_uv_mapping_rejects_non_finite_coordinates() {
    assert!(buffer_uv_from_surface_uv(Transform::Normal, (f32::NAN, 0.5)).is_none());
    assert!(buffer_uv_from_surface_uv(Transform::Normal, (0.5, f32::INFINITY)).is_none());
}
