# chromaprint-sys-next

![Crates.io](https://img.shields.io/crates/v/chromaprint-sys-next)

Rust bindings for [Chromaprint](https://github.com/acoustid/chromaprint). Version tracks the Chromaprint version.

## Prerequisites

Install LLVM or Clang for `buildgen`.

On Windows, install manually: https://docs.rs/vcpkg/latest/vcpkg/index.html

### Linux

Install `pkg-config` and `chromaprint` through your package manager.

Debian:

```
sudo apt install pkg-config libchromaprint-dev
```

### macOS

Install `pkg-config` and `chromaprint`:

```
brew install pkg-config chromaprint
```

### Windows

1. Install `cargo-vcpkg`: `cargo install cargo-vcpkg`
2. Install `vcpkg` deps: `cargo vcpkg build`
3. Add `vcpkg` bin directory to path (for DLL lookup): `$VCPKG_ROOT\installed\x64-windows\bin`
4. Build and run: `cargo run`

**Note:** Static linking does not work on Windows due to issues with static linking `ffmpeg` using vcpkg. See: https://github.com/microsoft/vcpkg/issues/9571

### Building chromaprint from source

If the library is not found on the system, the script will try to build it from source. This requires `CMake` to be installed: https://cmake.org/download/

