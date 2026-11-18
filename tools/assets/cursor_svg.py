# SPDX-License-Identifier: GPL-3.0-or-later
"""Validate the security and geometry constraints of cursor SVG masters."""

from __future__ import annotations

import re
import xml.etree.ElementTree as ET
from pathlib import Path

SVG_NAMESPACE = "{http://www.w3.org/2000/svg}"
DIMENSION_PATTERN = re.compile(r"^([0-9]+)(px)?$")


def _dimension(value: str | None) -> int | None:
    if value is None:
        return None
    match = DIMENSION_PATTERN.fullmatch(value)
    return int(match.group(1)) if match else None


def validate_svg(path: Path, width: int, height: int, allow_animation: bool) -> list[str]:
    errors: list[str] = []
    try:
        source = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        return [f"cannot read SVG: {error}"]

    lowered = source.lower()
    if "<!doctype" in lowered or "<!entity" in lowered:
        return ["DOCTYPE and entity declarations are forbidden"]

    try:
        root = ET.fromstring(source)
    except ET.ParseError as error:
        return [f"invalid XML: {error}"]

    if root.tag != f"{SVG_NAMESPACE}svg":
        errors.append("root element must be an SVG element")

    if _dimension(root.get("width")) != width:
        errors.append(f"SVG width must be {width}px")
    if _dimension(root.get("height")) != height:
        errors.append(f"SVG height must be {height}px")
    if root.get("viewBox") != f"0 0 {width} {height}":
        errors.append(f"viewBox must be '0 0 {width} {height}'")

    animated_elements = 0
    for element in root.iter():
        local_name = element.tag.rsplit("}", 1)[-1].lower()
        if local_name == "script":
            errors.append("script elements are forbidden")
        if local_name.startswith("animate"):
            animated_elements += 1
        for attribute, value in element.attrib.items():
            attribute_name = attribute.rsplit("}", 1)[-1].lower()
            if attribute_name.startswith("on"):
                errors.append(f"event handler attribute is forbidden: {attribute_name}")
            if attribute_name in {"href", "src"} and (
                not value.startswith("#") or value.startswith("//")
            ):
                errors.append("external SVG resource references are forbidden")
            if "url(" in value.lower() and "url(#" not in value.lower():
                errors.append("external CSS resource references are forbidden")

    if animated_elements and not allow_animation:
        errors.append("animation is only permitted in the busy cursor source")
    if allow_animation and animated_elements == 0:
        errors.append("busy cursor must contain an SVG animation element")

    return errors
