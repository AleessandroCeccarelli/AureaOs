// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Logical desktop geometry and deterministic item hit-testing.

use super::{DesktopItem, DesktopItemId, DesktopPosition};

/// A point expressed in global desktop logical coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopPoint {
    pub x: i32,
    pub y: i32,
}

impl DesktopPoint {
    pub const fn new(x: i32, y: i32) -> Self { Self { x, y } }
}

/// A logical rectangle with a top-left origin and positive dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopRect {
    origin: DesktopPoint,
    width: u32,
    height: u32,
}

impl DesktopRect {
    pub const fn new(origin: DesktopPoint, width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 { return None; }
        Some(Self { origin, width, height })
    }

    pub const fn from_position(position: DesktopPosition, width: u32, height: u32) -> Option<Self> {
        Self::new(DesktopPoint::new(position.x, position.y), width, height)
    }

    /// Uses a left/top-inclusive and right/bottom-exclusive hit region.
    pub fn contains(self, point: DesktopPoint) -> bool {
        let x = i64::from(point.x);
        let y = i64::from(point.y);
        let left = i64::from(self.origin.x);
        let top = i64::from(self.origin.y);
        x >= left && y >= top
            && x < left + i64::from(self.width)
            && y < top + i64::from(self.height)
    }
}

/// Hit-testing policy for desktop entries.
///
/// Later entries are treated as visually above earlier entries when hit regions
/// overlap. Dimensions are supplied by the presentation layer in logical units.
pub struct DesktopHitTest;

impl DesktopHitTest {
    pub fn item_at<'a>(
        items: &'a [DesktopItem],
        point: DesktopPoint,
        hit_width: u32,
        hit_height: u32,
    ) -> Option<&'a DesktopItem> {
        if hit_width == 0 || hit_height == 0 { return None; }
        items.iter().rev().find(|item| {
            DesktopRect::from_position(item.position(), hit_width, hit_height)
                .is_some_and(|bounds| bounds.contains(point))
        })
    }

    pub fn item_id_at(
        items: &[DesktopItem],
        point: DesktopPoint,
        hit_width: u32,
        hit_height: u32,
    ) -> Option<DesktopItemId> {
        Self::item_at(items, point, hit_width, hit_height).map(|item| item.id().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopHitTest, DesktopPoint, DesktopRect};
    use crate::desktop::{DesktopItem, DesktopItemId, DesktopItemKind, DesktopPosition};

    fn item(id: &str, x: i32, y: i32) -> DesktopItem {
        DesktopItem::new(
            DesktopItemId::new(id).unwrap(),
            id,
            DesktopItemKind::File { uri: format!("file:///tmp/{id}") },
            DesktopPosition::new(x, y),
        )
    }

    #[test]
    fn rectangle_uses_inclusive_origin_and_exclusive_far_edges() {
        let bounds = DesktopRect::new(DesktopPoint::new(10, 20), 32, 48).unwrap();
        assert!(bounds.contains(DesktopPoint::new(10, 20)));
        assert!(bounds.contains(DesktopPoint::new(41, 67)));
        assert!(!bounds.contains(DesktopPoint::new(42, 67)));
        assert!(!bounds.contains(DesktopPoint::new(41, 68)));
    }

    #[test]
    fn rejects_zero_sized_hit_regions() {
        assert!(DesktopRect::new(DesktopPoint::new(0, 0), 0, 10).is_none());
        assert!(DesktopRect::new(DesktopPoint::new(0, 0), 10, 0).is_none());
        assert!(DesktopHitTest::item_at(&[item("one", 0, 0)], DesktopPoint::new(1, 1), 0, 10).is_none());
    }

    #[test]
    fn overlapping_items_resolve_to_the_last_item() {
        let items = [item("back", 0, 0), item("front", 8, 8)];
        let hit = DesktopHitTest::item_at(&items, DesktopPoint::new(10, 10), 32, 32).unwrap();
        assert_eq!(hit.id().as_str(), "front");
    }

    #[test]
    fn hit_testing_uses_logical_item_positions() {
        let items = [item("one", 100, 200)];
        let hit = DesktopHitTest::item_id_at(&items, DesktopPoint::new(110, 210), 48, 48).unwrap();
        assert_eq!(hit.as_str(), "one");
        assert!(DesktopHitTest::item_at(&items, DesktopPoint::new(50, 50), 48, 48).is_none());
    }
}
