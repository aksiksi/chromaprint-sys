extern crate bindgen;
extern crate cfg_if;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn output_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

// The major and minor versions of this crate track the version of Chromaprint.
fn chromaprint_version() -> String {
    format!("{}.{}.0", env!("CARGO_PKG_VERSION_MAJOR"), env!("CARGO_PKG_VERSION_MINOR"))
}

fn is_dynamic() -> bool {
    env::var("CHROMAPRINT_SYS_DYNAMIC").is_ok() || cfg!(feature = "dynamic")
}

fn is_static() -> bool {
    !is_dynamic()
        && (env::var("CHROMAPRINT_SYS_STATIC").is_ok()
            || cfg!(feature = "static"))
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
                compile_error!("vDSP can only be used on macOS or iOS");
            }

            fftlib = Some("vdsp");
        }
    }

    if let Some(fftlib) = fftlib {
        cmake_config.define("FFTLIB", fftlib);
    }
}

// TODO(aksiksi): Avoid downloading code. Instead, add chromaprint as a Git submodule.
// Only question is how to "pin" the submodule to a specific version.
fn clone_repo_and_build() -> PathBuf {
    let version = format!("v{}", chromaprint_version());
    let dest_dir = format!("chromaprint-{}", &version);

    // Remove the destination directory if it already exists.
    if std::fs::read_dir(output_dir().join(&dest_dir)).is_ok() {
        std::fs::remove_dir_all(output_dir().join(&dest_dir)).unwrap();
    }

    // Clone the repo.
    let status = Command::new("git")
        .current_dir(output_dir())
        .arg("clone")
        .arg("--depth=1")
        .arg("-b")
        .arg(version)
        .arg("https://github.com/acoustid/chromaprint")
        .arg(&dest_dir)
        .status()
        .expect("failed to run git clone");
    if !status.success() {
        panic!("git clone failed")
    }

    // Setup CMake based on provided feature flags.
    let mut cmake_config = cmake::Config::new(output_dir().join(&dest_dir));
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
    output_dir().join(&dest_dir).join("src")
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut include_path = None;

    // Check if the library is already available on the system.
    #[cfg(not(windows))]
    {
        // Use pkg-config on Linux and macOS.
        let library = pkg_config::Config::new()
            .atleast_version(&chromaprint_version())
            .statik(is_static())
            .probe("libchromaprint");
        if let Ok(library) = library {
            if library.include_paths.len() == 0 {
                println!("cargo:warning=No include paths found!");
                return;
            }
            include_path = Some(library.include_paths[0].clone());
        }
    }
    #[cfg(windows)]
    {
        // Use vcpkg on Windows.
        // NOTE: There is apparently no way to pin to specific version...
        if !is_static() {
            env::set_var("VCPKGRS_DYNAMIC", "1");
            env::set_var("VCPKGRS_TRIPLET", "x64-windows");
        }
        let library = vcpkg::find_package("chromaprint");
        if let Ok(library) = library {
            if library.include_paths.len() == 0 {
                println!("cargo:warning=No include paths found!");
                return;
            }
            include_path = Some(library.include_paths[0].clone());
        }
    }

    // If the library is not available, clone it from Github and build it.
    if include_path.is_none() {
        include_path = Some(clone_repo_and_build());
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
