# Contributing to rusToniESP 🦀📻

Thank you for taking the time to contribute to **rusToniESP**! This document provides step-by-step instructions to set up your local development environment for the **ESP32-S3** architecture using standard Rust (`std`), as well as configuring the **Wokwi** simulator for virtual testing.

---

## 🛠️ Global Prerequisites & System Tools

Because the ESP32-S3 utilizes an Xtensa processor core, your host machine requires specific compilation tools to cross-compile and link Rust binaries successfully.

### 1. Install Rust
Install the standard Rust toolchain manager via `rustup`:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
*Choose option `1` (default installation) and restart your terminal or run `source "$HOME/.cargo/env"`.*

### 2. Install Host Compilation Utilities (macOS Example)
Your host machine needs CMake and Ninja to build the underlying Espressif C-based ecosystem libraries:
```bash
brew install cmake ninja dfu-util ccache
```

### 3. Install the ESP32 Rust Toolchain Suite
Install the global Espressif management tools and flashing utilities via Cargo:
```bash
cargo install espup cargo-generate espflash
```

### 4. Install and Active the Xtensa Compiler Path
Run `espup` to fetch the custom LLVM/Xtensa patches needed for the S3 chip:
```bash
espup install
```
Whenever you open a new terminal tab to work on this repository, you must source the environment variables to activate the Xtensa toolchain:
```bash
source "$HOME/export-esp.sh"
```

---

## 📂 Project Initialization & Compilation

1. **Verify your environment:** Ensure the Xtensa target is active.
   ```bash
   rustc --print target-spec-json --target xtensa-esp32s3-espidf > /dev/null
   ```
2. **Build the project:** Run a standard cargo compilation cycle. The initial build will compile the underlying ESP-IDF frameworks (this may take a few minutes).
   ```bash
   cargo build --release
   ```

---

## 💻 Simulating with Wokwi (Optional)

If you do not have the physical hardware yet, you can run an instruction-accurate simulation of the ESP32-S3 right inside **VS Code**.

### 1. VS Code Extension Setup
1. Search for and install the **Wokwi** extension in VS Code.
2. Press `Cmd + Shift + P` (or `Ctrl + Shift + P`) and select **Wokwi: Start Simulator**.
3. Follow the prompt to authenticate with your free Wokwi account.

### 2. Required Configuration Files
The repository includes the necessary configuration blueprints in the root directory:

*   **`wokwi.toml`**: Instructs the extension where to find the compiled project binary matching the Xtensa architecture.
*   **`diagram.json`**: Renders the visual representation of the ESP32-S3-DevKitC-1 development board.

### 3. Launching the Simulator
Once your local compilation succeeds (`cargo build --release`), open the VS Code command palette, run **Wokwi: Start Simulator**, and the interactive virtual chip window will boot immediately.

---

## 🚀 Working with Physical Hardware

When your **ESP32-S3 (N16R8 / N8R8 with PSRAM)** arrives, connect it to your computer via the native USB port and use `espflash` to flash and monitor execution simultaneously:

```bash
cargo espflash flash --release --monitor
```

---

## ✍️ Coding Guidelines & Code Generation

When implementing new features or resolving issues:
1. **Follow the Architecture:** Read [architecture.md](architecture.md). Place logic in its matching module (e.g., `src/audio/`, `src/rfid/`).
2. **Decompose via Traits:** Hardware interfaces must be hidden behind traits (e.g., `RfidReader`, `LedDriver`). Always provide a mock version (`MockXxx`) under `#[cfg(test)]` so the logic can be tested on the host.
3. **Event-Driven Communication:** Do not call methods on other subsystem structs directly. Emit a `SystemEvent` or send a `PlayerCommand` via standard Rust channels (`std::sync::mpsc`).
4. **Use Structured Logging:** Use the `log` crate macros (like `log::info!`, `log::error!`) with the `EspLogger`. Do not use raw `println!`.

---

## 🧪 How to Test Your Changes

Every new feature or bug fix must be covered by tests before submitting a Pull Request.

### 1. Host-Side Unit Tests
Unit tests run locally on your host machine (no ESP-IDF installation is compiled, which makes them extremely fast).
- Write tests inside a `#[cfg(test)]` module in the same file as your logic.
- Run the tests using cargo:
  ```bash
  # CARGO_CFG_UNIX=1 stops esp-idf-sys from building native ESP-IDF binaries on your host
  CARGO_CFG_UNIX=1 cargo test --lib --workspace
  ```

### 2. Simulator Integration Tests
To test full integration flows (e.g. serial log verification) using the Wokwi simulator:
- Use the `wokwi-cli` terminal tool directly. (The Wokwi MCP server integration is not supported).
- Compile the binary first:
  ```bash
  cargo build
  ```
- Run the simulation with serial verification:
  ```bash
  wokwi-cli --elf target/xtensa-esp32s3-espidf/debug/rus-toni-esp.elf --timeout 60000 --serial-log-file sim.log .
  ```
- Use `--expect-text "<string>"` or `--fail-text "<string>"` to automatically assert expected output in serial logs.

### 3. Continuous Integration
All builds, formatting, clippy checks, and unit tests are automatically verified on every push and pull request via the GitHub Actions workflow in [`.github/workflows/rust_ci.yml`](.github/workflows/rust_ci.yml). Ensure your changes pass `cargo fmt --all -- --check` and `cargo clippy` locally before committing.