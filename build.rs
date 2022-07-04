extern crate bindgen;
extern crate cfg_if;

use std::env;
use std::path::PathBuf;
use std::process::Command;

const CHROMAPRINT_SRC_DIR: &str = "src/chromaprint";

fn output_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

// The major and minor versions of this crate track the version of Chromaprint.
fn chromaprint_version() -> String {
    format!(
        "{}.{}.0",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR")
    )
}

fn is_static() -> bool {
    // Allow overriding with _DYNAMIC env variable.
    !env::var("CHROMAPRINT_SYS_DYNAMIC").is_ok()
        && (env::var("CHROMAPRINT_SYS_STATIC").is_ok() || cfg!(feature = "static"))
}

fn set_fft_library(cmake_config: &mut cmake::Config) {
    let fftlib: Option<&str> = None;

    cfg_if::cfg_if! {
        if #[cfg(feature = "avfft")] {
            fftlib = Some("avfft");
        } else if #[cfg(feature = "fftw3")] {
            fftlib = Some("fftw3");
        } else if #[cfg(feature = "fftw3f")] {
            fftlib = Some("fftw3f");
        } else if #[cfg(feature = "kissfft")] {
            fftlib = Some("kissfft");
        } else if #[cfg(feature = "vdsp")] {
            if #[cfg(not(any(target_os = "macos", target_os = "ios")))] {
                panic!("vDSP can only be used on macOS or iOS");
            }

            fftlib = Some("vdsp");
        }
    }

    if let Some(fftlib) = fftlib {
        cmake_config.define("FFTLIB", fftlib);
    }
}

fn build_chromaprint() -> Option<PathBuf> {
    let version = format!("v{}", chromaprint_version());

    // Checkout the required version in the submodule.
    let status = Command::new("git")
        .current_dir(CHROMAPRINT_SRC_DIR)
        .arg("checkout")
        .arg(format!("tags/{}", version))
        .status()
        .expect("failed to run command");
    if !status.success() {
        println!(
            "cargo:warning=unable to checkout version {}: {}",
            version,
            status.to_string()
        );
        return None;
    }

    // Setup CMake based on provided feature flags.
    let mut cmake_config = cmake::Config::new(CHROMAPRINT_SRC_DIR);
    if is_static() {
        cmake_config.cflag("-static");
        if cfg!(not(target_os = "macos")) {
            cmake_config
                .cflag("-static-libgcc")
                .cflag("-static-libstdc++");
        }
    }

    // Set the selected FFT library, if any. By default, we defer selection to Chromaprint.
    set_fft_library(&mut cmake_config);

    // Build the chromaprint library using CMake.
    let chromaprint_dst = cmake_config.build();

    // For some reason, the "cmake" builder returns the path to the output directory
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

    // Headers are located in the "src" directory of the chromaprint repo.
    Some(chromaprint_dst.join("include"))
}

#[cfg(not(target_env = "msvc"))]
fn try_vcpkg() -> Option<PathBuf> {
    None
}

#[cfg(target_env = "msvc")]
fn try_vcpkg() -> Option<PathBuf> {
    println!("cargo:rerun-if-env-changed=VCPKGRS_DYNAMIC");
    println!("cargo:rerun-if-env-changed=VCPKGRS_TRIPLET");

    if !is_static() {
        env::set_var("VCPKGRS_DYNAMIC", "1");
        env::set_var("VCPKGRS_TRIPLET", "x64-windows");
    }

    match vcpkg::find_package("chromaprint") {
        Ok(library) => {
            if library.include_paths.len() == 0 {
                println!("cargo:warning=no include paths found");
                return None;
            }
            Some(library.include_paths[0].clone())
        }
        Err(e) => {
            println!("cargo:warning=vcpkg did not find chromaprint: {}", e);
            None
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CHROMAPRINT_SYS_DYNAMIC");
    println!("cargo:rerun-if-env-changed=CHROMAPRINT_SYS_STATIC");

    let mut include_path = None;

    if cfg!(linux) && !is_static() {
        // Use pkg-config on Linux if linking dynamically.
        let library = pkg_config::Config::new()
            .atleast_version(&chromaprint_version())
            .probe("libchromaprint");
        match library {
            Ok(library) => {
                if library.include_paths.len() == 0 {
                    println!("cargo:warning=no include paths found");
                } else {
                    include_path = Some(library.include_paths[0].clone());
                }
            }
            Err(e) => {
                println!(
                    "cargo:warning=pkg-config did not find libchromaprint: {}",
                    e
                );
            }
        }
    } else if cfg!(target_env = "msvc") {
        // Try using vcpkg on Windows.
        include_path = try_vcpkg();
    }

    // Build from source in all other cases.
    if include_path.is_none() {
        include_path = build_chromaprint();
    }

    let include_path = include_path.unwrap();

    // Generate the bindings.
    let header_path = output_dir()
        .join(&include_path)
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
