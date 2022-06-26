# chromaprint-sys

Rust bindings for Chromaprint.

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
2. Install `vcpkg` dependencies: `cargo vcpkg build`
3. Build: `cargo build`

### Building chromaprint from source

If the library is not found on the system, the script will try to build it from source. This requires `CMake` to be installed: https://cmake.org/download/
