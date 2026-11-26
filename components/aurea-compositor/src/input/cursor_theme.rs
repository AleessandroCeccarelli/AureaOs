// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Maps Smithay/W3C pointer names to AureaOS cursor-theme assets.
//!
//! This module deliberately contains no filesystem or renderer code. It keeps
//! protocol-name compatibility separate from theme loading and GPU resources.

/// Returns the canonical AureaOS asset name for a Smithay cursor icon name.
///
/// Unknown names use the default arrow. Multiple protocol and Xcursor aliases
/// intentionally resolve to the same artwork where their meaning is equivalent.
pub fn asset_name_for_icon(icon_name: &str) -> &'static str {
    match icon_name {
        "default" | "arrow" | "left_ptr" => "default",
        "pointer" | "hand2" => "pointer",
        "text" | "ibeam" | "xterm" => "text",
        "vertical-text" | "vertical_text" => "vertical-text",
        "crosshair" => "crosshair",
        "help" | "question_arrow" => "help",
        "wait" | "watch" | "busy" => "busy",
        "progress" | "left_ptr_watch" => "left_ptr_watch",
        "not-allowed" | "forbidden" | "no-drop" => "not-allowed",
        "copy" => "copy",
        "alias" => "alias",
        "grab" | "openhand" => "grab",
        "grabbing" | "closedhand" => "grabbing",
        "move" | "fleur" => "move",
        "all-scroll" | "all_scroll" => "all-scroll",
        "ew-resize" | "col-resize" | "size_hor" => "ew-resize",
        "ns-resize" | "row-resize" | "size_ver" => "ns-resize",
        "nesw-resize" | "resize-nesw" | "size_bdiag" => "resize-nesw",
        "nwse-resize" | "resize-nwse" | "size_fdiag" => "resize-nwse",
        "pencil" | "pen" => "pencil",
        "up-arrow" | "alternate" => "up-arrow",
        "pin" => "pin",
        "person" => "person",
        "zoom-in" | "zoom_in" => "zoom-in",
        "zoom-out" | "zoom_out" => "zoom-out",
        "context-menu" | "context_menu" => "context-menu",
        _ => "default",
    }
}

#[cfg(test)]
mod tests {
    use super::asset_name_for_icon;

    #[test]
    fn maps_standard_pointer_roles_to_canonical_assets() {
        let cases = [
            ("default", "default"),
            ("pointer", "pointer"),
            ("text", "text"),
            ("vertical-text", "vertical-text"),
            ("crosshair", "crosshair"),
            ("help", "help"),
            ("wait", "busy"),
            ("progress", "left_ptr_watch"),
            ("not-allowed", "not-allowed"),
            ("copy", "copy"),
            ("alias", "alias"),
            ("grab", "grab"),
            ("grabbing", "grabbing"),
            ("move", "move"),
            ("all-scroll", "all-scroll"),
            ("ew-resize", "ew-resize"),
            ("ns-resize", "ns-resize"),
            ("nesw-resize", "resize-nesw"),
            ("nwse-resize", "resize-nwse"),
            ("pencil", "pencil"),
            ("up-arrow", "up-arrow"),
            ("pin", "pin"),
            ("person", "person"),
            ("zoom-in", "zoom-in"),
            ("zoom-out", "zoom-out"),
            ("context-menu", "context-menu"),
        ];

        for (icon, expected) in cases {
            assert_eq!(asset_name_for_icon(icon), expected, "{icon}");
        }
    }

    #[test]
    fn maps_linux_and_toolkit_aliases_without_duplicate_artwork() {
        assert_eq!(asset_name_for_icon("left_ptr"), "default");
        assert_eq!(asset_name_for_icon("hand2"), "pointer");
        assert_eq!(asset_name_for_icon("ibeam"), "text");
        assert_eq!(asset_name_for_icon("watch"), "busy");
        assert_eq!(asset_name_for_icon("no-drop"), "not-allowed");
        assert_eq!(asset_name_for_icon("openhand"), "grab");
        assert_eq!(asset_name_for_icon("closedhand"), "grabbing");
        assert_eq!(asset_name_for_icon("col-resize"), "ew-resize");
        assert_eq!(asset_name_for_icon("row-resize"), "ns-resize");
        assert_eq!(asset_name_for_icon("size_bdiag"), "resize-nesw");
        assert_eq!(asset_name_for_icon("size_fdiag"), "resize-nwse");
    }

    #[test]
    fn unknown_cursor_names_fall_back_to_the_default_arrow() {
        assert_eq!(asset_name_for_icon("future-cursor"), "default");
        assert_eq!(asset_name_for_icon(""), "default");
    }
}
