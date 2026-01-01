# Building dealer3 for Windows

This document describes how to build the Windows 64-bit version of dealer3 from macOS or Linux.

## Prerequisites

### On macOS

1. Install Homebrew (if not already installed):
   ```bash
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```

2. Install mingw-w64 cross-compiler:
   ```bash
   brew install mingw-w64
   ```

3. Add Windows target to Rust:
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```

### On Linux (Ubuntu/Debian)

1. Install mingw-w64:
   ```bash
   sudo apt-get update
   sudo apt-get install mingw-w64
   ```

2. Add Windows target to Rust:
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```

## Building

### Using the Build Script (Recommended)

Simply run the provided build script:

```bash
./scripts/windows/build-windows.sh
```

This will:
- Check for required tools
- Build the Windows executable
- Create a distribution package with documentation
- Generate a zip file in `dist/dealer3-windows-x64.zip`

### Manual Build

1. Configure the cross-compiler (only needed once):

   Create `.cargo/config.toml`:
   ```toml
   [target.x86_64-pc-windows-gnu]
   linker = "x86_64-w64-mingw32-gcc"
   ar = "x86_64-w64-mingw32-ar"
   ```

2. Build for Windows:
   ```bash
   cargo build --release --target x86_64-pc-windows-gnu
   ```

3. The Windows executable will be at:
   ```
   target/x86_64-pc-windows-gnu/release/dealer.exe
   ```

## Testing the Windows Build

While you can't run the Windows executable directly on macOS/Linux, you can:

1. **Check the binary type**:
   ```bash
   file target/x86_64-pc-windows-gnu/release/dealer.exe
   ```
   Should output: `PE32+ executable (console) x86-64`

2. **Test on Windows**:
   - Copy `dealer.exe` to a Windows machine
   - Run in Command Prompt or PowerShell:
     ```cmd
     echo hcp(north) >= 15 | dealer.exe -p 5
     ```

3. **Test with Wine** (optional):
   ```bash
   brew install wine-stable
   wine target/x86_64-pc-windows-gnu/release/dealer.exe --version
   ```

## Distribution Package

The build script creates a distribution package that includes:

- `dealer.exe` - The Windows executable (5.5 MB)
- `README.txt` - Windows-specific usage instructions
- `LICENSE` - License file
- `CHANGELOG.md` - Version history

The complete package is zipped as `dist/dealer3-windows-x64.zip` (~1.8 MB compressed).

## Troubleshooting

### Error: "mingw-w64 not found"

Install mingw-w64 as described in Prerequisites.

### Error: "linker `x86_64-w64-mingw32-gcc` not found"

Ensure `.cargo/config.toml` exists and points to the correct linker path.

On macOS with Homebrew, verify the path:
```bash
which x86_64-w64-mingw32-gcc
```

Should be `/opt/homebrew/bin/x86_64-w64-mingw32-gcc` (Apple Silicon) or
`/usr/local/bin/x86_64-w64-mingw32-gcc` (Intel Mac).

### Large Binary Size

The Windows executable is larger than the macOS/Linux version due to static linking of dependencies. This is expected and ensures the executable runs on any Windows system without requiring additional DLLs.

To reduce size further, you can strip debug symbols (already done by default in release mode).

## CI/CD Integration

For automated builds, you can use GitHub Actions. Example workflow:

```yaml
name: Build Windows

on: [push]

jobs:
  build-windows:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: sudo apt-get update && sudo apt-get install -y mingw-w64
      - run: rustup target add x86_64-pc-windows-gnu
      - run: cargo build --release --target x86_64-pc-windows-gnu
      - uses: actions/upload-artifact@v4
        with:
          name: dealer-windows-x64
          path: target/x86_64-pc-windows-gnu/release/dealer.exe
```

## Cross-Platform Support

dealer3 supports building for:

- **Windows 64-bit** (x86_64-pc-windows-gnu) - This guide
- **macOS** (aarch64-apple-darwin, x86_64-apple-darwin)
- **Linux** (x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu)

See the main README for platform-specific build instructions.
