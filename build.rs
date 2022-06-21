extern crate bindgen;
extern crate cc;
extern crate cmake;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn output_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn chromaprint_version() -> String {
    // TODO(aksiksi): Make this configurable
    "v1.5.1".to_string()
}

fn is_static() -> bool {
    env::var("CARGO_FEATURE_STATIC").is_ok()
}

fn main() {
    // 1. Clone the Chromaprint repo
    let version = chromaprint_version();
    let clone_dest_dir = format!("chromaprint-{}", version);
    // Remove directory if it exists.
    std::fs::remove_dir_all(output_dir().join(&clone_dest_dir))
        .or(std::io::Result::Ok(()))
        .unwrap();
    let status = Command::new("git")
        .current_dir(output_dir())
        .arg("clone")
        .arg("--depth=1")
        .arg("-b")
        .arg(version)
        .arg("https://github.com/acoustid/chromaprint")
        .arg(&clone_dest_dir)
        .status()
        .expect("failed to run git clone");
    if !status.success() {
        panic!("git clone failed")
    }

    // TODO(aksiksi): This is trying to fix static linking on macOS. WIP.
    if is_static() && cfg!(target_os = "macos") {
        // Taken from here: https://github.com/zmwangx/rust-ffmpeg-sys/blob/master/build.rs
        let frameworks = vec![
            "AppKit",
            "AudioToolbox",
            "AVFoundation",
            "CoreFoundation",
            "CoreGraphics",
            "CoreMedia",
            "CoreServices",
            "CoreVideo",
            "Foundation",
            "OpenCL",
            "OpenGL",
            "QTKit",
            "QuartzCore",
            "Security",
            "VideoDecodeAcceleration",
            "VideoToolbox",
        ];
        for f in frameworks {
            println!("cargo:rustc-link-lib=framework={}", f);
        }
    }

    // 2. Build Chromaprint using CMake
    let mut cmake_config = cmake::Config::new(output_dir().join(&clone_dest_dir));
    if is_static() {
        cmake_config.cflag("-static");
        if cfg!(not(target_os = "macos")) {
            cmake_config
                .cflag("-static-libgcc")
                .cflag("-static-libstdc++");
        }
    }
    let chromaprint_dst = cmake_config.build();
    // For some reason, the cmake builder returns the path to the output directory
    // instead of the lib directory.
    println!(
        "cargo:rustc-link-search=native={}",
        chromaprint_dst.join("lib").display()
    );
    if is_static() {
        println!("cargo:rustc-link-lib=static=chromaprint");
    } else {
        println!("cargo:rustc-link-lib=chromaprint");
    }

    // 3. Generate the bindings
    let header_path = output_dir()
        .join(&clone_dest_dir)
        .join("src")
        .join("chromaprint.h")
        .display()
        .to_string();
    let bindings = bindgen::Builder::default()
        .header(header_path)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(output_dir().join("bindings.rs"))
        .expect("Couldn't write bindings");
}
