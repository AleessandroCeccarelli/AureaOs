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
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let shader_dir = manifest_dir.join("shaders");

    for shader in ["surface_texture.vert", "surface_texture.frag"] {
        println!("cargo:rerun-if-changed={}", shader_dir.join(shader).display());
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
    compile_shader(
        &shader_dir.join("surface_texture.frag"),
        &output_dir.join("surface_texture.frag.spv"),
        "frag",
    );

    println!(
        "cargo:rustc-env=AUREA_SURFACE_TEXTURE_VERT_SPV={}",
        output_dir.join("surface_texture.vert.spv").display()
    );
    println!(
        "cargo:rustc-env=AUREA_SURFACE_TEXTURE_FRAG_SPV={}",
        output_dir.join("surface_texture.frag.spv").display()
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
                "Unable to execute glslangValidator ({error}). Install it with: sudo dnf install glslang"
            )
        });

    if !result.status.success() {
        panic!(
            "Vulkan shader compilation failed for {}:\n{}",
            source.display(),
            String::from_utf8_lossy(&result.stderr)
        );
    }
}
