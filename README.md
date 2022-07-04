# chromaprint-sys-next

![Crates.io](https://img.shields.io/crates/v/chromaprint-sys-next)

Rust bindings for [Chromaprint](https://github.com/acoustid/chromaprint). Version tracks the Chromaprint version.

## Prerequisites

### General

* LLVM or Clang for `buildgen`.

### Linux (Debian/Ubuntu)

Static linking (**preferred**):

```
sudo apt install pkg-config cmake libfftw3-dev
```

Dynamic linking:

```
sudo apt install pkg-config libchromaprint-dev
```

### macOS

```
brew install cmake
```

### Windows

1. Install `cargo-vcpkg`: `cargo install cargo-vcpkg`
2. Install `vcpkg` deps: `cargo vcpkg build`
3. Build and run: `cargo run`

### Building chromaprint from source

If the library is not found on the system, the script will try to build it from source. This requires:

1. `CMake`: https://cmake.org/download/
2. FFTW3 (optional, but _highly_ recommended): https://www.fftw.org/

