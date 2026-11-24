// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

fn main() {
    println!("cargo:rerun-if-changed=src/jpeg/turbojpeg_shim.c");
    println!("cargo:rustc-link-lib=turbojpeg");

    cc::Build::new()
        .file("src/jpeg/turbojpeg_shim.c")
        .warnings(true)
        .extra_warnings(true)
        .warnings_into_errors(true)
        .compile("aurea_turbojpeg_shim");
}
