// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Compiles compositor shaders for the native Vulkan DRM renderer.

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let shader_dir = manifest_dir.join("shaders");

    for shader in [
        "surface_texture.vert",
        "surface_texture.frag",
        "cursor_trail.frag",
        "wallpaper_texture.vert",
        "wallpaper_texture.frag",
        "display_color.vert",
        "display_color.frag",
    ] {
        println!(
            "cargo:rerun-if-changed={}",
            shader_dir.join(shader).display()
        );
    }
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_DRM");

    if env::var_os("CARGO_FEATURE_DRM").is_none() {
        return;
    }

    let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    compile_shader(
        &shader_dir.join("surface_texture.vert"),
        &output_dir.join("surface_texture.vert.spv"),
        "vert",
    );
    compile_compatibility_shader(
        &shader_dir.join("surface_texture.vert"),
        &output_dir.join("surface_texture.vert.compat.spv"),
        "vert",
    );
    println!(
        "cargo:rustc-env=AUREA_SURFACE_TEXTURE_VERT_COMPAT_SPV={}",
        output_dir.join("surface_texture.vert.compat.spv").display()
    );
    compile_shader(
        &shader_dir.join("surface_texture.frag"),
        &output_dir.join("surface_texture.frag.spv"),
        "frag",
    );
    compile_compatibility_shader(
        &shader_dir.join("surface_texture.frag"),
        &output_dir.join("surface_texture.frag.compat.spv"),
        "frag",
    );
    println!(
        "cargo:rustc-env=AUREA_SURFACE_TEXTURE_FRAG_COMPAT_SPV={}",
        output_dir.join("surface_texture.frag.compat.spv").display()
    );

    println!(
        "cargo:rustc-env=AUREA_SURFACE_TEXTURE_VERT_SPV={}",
        output_dir.join("surface_texture.vert.spv").display()
    );
    println!(
        "cargo:rustc-env=AUREA_SURFACE_TEXTURE_FRAG_SPV={}",
        output_dir.join("surface_texture.frag.spv").display()
    );

    compile_shader(
        &shader_dir.join("cursor_trail.frag"),
        &output_dir.join("cursor_trail.frag.spv"),
        "frag",
    );
    compile_compatibility_shader(
        &shader_dir.join("cursor_trail.frag"),
        &output_dir.join("cursor_trail.frag.compat.spv"),
        "frag",
    );
    println!(
        "cargo:rustc-env=AUREA_CURSOR_TRAIL_FRAG_COMPAT_SPV={}",
        output_dir.join("cursor_trail.frag.compat.spv").display()
    );
    println!(
        "cargo:rustc-env=AUREA_CURSOR_TRAIL_FRAG_SPV={}",
        output_dir.join("cursor_trail.frag.spv").display()
    );

    compile_shader(
        &shader_dir.join("wallpaper_texture.vert"),
        &output_dir.join("wallpaper_texture.vert.spv"),
        "vert",
    );
    compile_compatibility_shader(
        &shader_dir.join("wallpaper_texture.vert"),
        &output_dir.join("wallpaper_texture.vert.compat.spv"),
        "vert",
    );
    println!(
        "cargo:rustc-env=AUREA_WALLPAPER_TEXTURE_VERT_COMPAT_SPV={}",
        output_dir.join("wallpaper_texture.vert.compat.spv").display()
    );
    compile_shader(
        &shader_dir.join("wallpaper_texture.frag"),
        &output_dir.join("wallpaper_texture.frag.spv"),
        "frag",
    );
    compile_compatibility_shader(
        &shader_dir.join("wallpaper_texture.frag"),
        &output_dir.join("wallpaper_texture.frag.compat.spv"),
        "frag",
    );
    println!(
        "cargo:rustc-env=AUREA_WALLPAPER_TEXTURE_FRAG_COMPAT_SPV={}",
        output_dir.join("wallpaper_texture.frag.compat.spv").display()
    );

    println!(
        "cargo:rustc-env=AUREA_WALLPAPER_TEXTURE_VERT_SPV={}",
        output_dir.join("wallpaper_texture.vert.spv").display()
    );
    println!(
        "cargo:rustc-env=AUREA_WALLPAPER_TEXTURE_FRAG_SPV={}",
        output_dir.join("wallpaper_texture.frag.spv").display()
    );

    compile_shader(
        &shader_dir.join("display_color.vert"),
        &output_dir.join("display_color.vert.spv"),
        "vert",
    );
    compile_compatibility_shader(
        &shader_dir.join("display_color.vert"),
        &output_dir.join("display_color.vert.compat.spv"),
        "vert",
    );
    println!(
        "cargo:rustc-env=AUREA_DISPLAY_COLOR_VERT_COMPAT_SPV={}",
        output_dir.join("display_color.vert.compat.spv").display()
    );
    compile_shader(
        &shader_dir.join("display_color.frag"),
        &output_dir.join("display_color.frag.spv"),
        "frag",
    );
    compile_compatibility_shader(
        &shader_dir.join("display_color.frag"),
        &output_dir.join("display_color.frag.compat.spv"),
        "frag",
    );
    println!(
        "cargo:rustc-env=AUREA_DISPLAY_COLOR_FRAG_COMPAT_SPV={}",
        output_dir.join("display_color.frag.compat.spv").display()
    );

    println!(
        "cargo:rustc-env=AUREA_DISPLAY_COLOR_VERT_SPV={}",
        output_dir.join("display_color.vert.spv").display()
    );
    println!(
        "cargo:rustc-env=AUREA_DISPLAY_COLOR_FRAG_SPV={}",
        output_dir.join("display_color.frag.spv").display()
    );
}

fn compile_shader(source: &Path, destination: &Path, stage: &str) {
    let result = Command::new("glslangValidator")
        .args(["-V", "--target-env", "vulkan1.3", "-S", stage])
        .arg(source)
        .arg("-o")
        .arg(destination)
        .output()
        .unwrap_or_else(|error| {
            panic!(
                "Unable to execute glslangValidator ({error}). Install the shader compiler (Debian: glslang-tools; Fedora: glslang)"
            )
        });

    if !result.status.success() {
        panic!(
            "Vulkan shader compilation failed for {}:\n{}",
            source.display(),
            format!(
                "{}{}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            )
        );
    }
}


fn compile_compatibility_shader(source: &Path, destination: &Path, stage: &str) {
    let result = Command::new("glslangValidator")
        .args(["-V", "--target-env", "vulkan1.0", "-S", stage])
        .arg(source)
        .arg("-o")
        .arg(destination)
        .output()
        .unwrap_or_else(|error| {
            panic!(
                "Unable to execute glslangValidator ({error}). Install the shader compiler (Debian: glslang-tools; Fedora: glslang)"
            )
        });

    if !result.status.success() {
        panic!(
            "Vulkan 1.0 compatibility shader compilation failed for {}:\n{}",
            source.display(),
            format!(
                "{}{}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            )
        );
    }
}
