use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("macos") => build_macos_helper(),
        Ok("windows") => embed_windows_icon(),
        _ => {}
    }
}

/// Universal build: the helper runs inside /usr/bin/perl, whose architecture may differ from ours.
fn build_macos_helper() {
    println!("cargo:rerun-if-changed=macos/helper.m");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("helper.dylib");
    let status = Command::new("xcrun")
        .args([
            "clang",
            "-fobjc-arc",
            "-Wall",
            "-Werror",
            "-O2",
            "-dynamiclib",
        ])
        .args([
            "-arch",
            "arm64",
            "-arch",
            "x86_64",
            "-mmacosx-version-min=12.0",
        ])
        .args(["-framework", "Foundation", "macos/helper.m", "-o"])
        .arg(&out)
        .status()
        .expect("run xcrun clang");
    assert!(status.success(), "failed to build macos/helper.m");
}

fn embed_windows_icon() {
    println!("cargo:rerun-if-changed=assets/AppIcon.ico");
    // Only on a Windows host: cross-checks from macOS have no resource compiler.
    if cfg!(windows) {
        winresource::WindowsResource::new()
            .set_icon("assets/AppIcon.ico")
            .compile()
            .expect("embed Windows icon");
    }
}
