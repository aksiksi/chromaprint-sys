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

1. Install `vcpkg`: https://vcpkg.io/en/getting-started.html
2. Set `VCPKG_ROOT` environment variable to installation directory
3. Install `chromaprint` (static):

```
vcpkg install chromaprint:x64-windows-static-md
```

### Building chromaprint from source

If the library is not found on the system, the script will try to build it from source. This requires `CMake` to be installed: https://cmake.org/download/
