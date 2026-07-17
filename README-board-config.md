# Board Configuration

The target board profile is determined **strictly by your active compilation target**. There is no need to pass manual environment variables during execution.

To select or switch the board layout, edit the master switchboard at the top of your `.cargo/config.toml` file:

* **ESP32-S3:** Set `target = "xtensa-esp32s3-espidf"`
* **ESP32 (WROOM/WROVER):** Set `target = "xtensa-esp32-espidf"`

### Project Build Command
Once the target is uncommented in `.cargo/config.toml`, simply run:

```bash
cargo build
```

---

## Technical Overview: Automated Target Switching

### What We Changed
Previously, board selection required manual matching logic or passing dynamic environment variables, which caused friction when moving between standard ESP32 layouts and the newer ESP32-S3 layouts.

To solve this, we moved the board-specific hardware definitions directly into the **build pipeline** via `build.rs` and automated profile resolution at compile time:
1. **Unified Pin Ergonomics:** Replaced explicit target-bound `match` variants with a unified `AnyIOPin::steal` architecture. This completely eliminates compiler blocks caused by the structural differences between MCU variants (such as input-only configurations on GPIO 34/35 for basic ESP32 chips).
2. **Compile-Time Env Injection:** The `build.rs` script now sniffs the active `CARGO_CFG_TARGET_TRIPLE` automatically, parses a corresponding local board configuration file, and injects the hardware values as environment variables directly into the compiler profile context.
3. **Static Lifecycles:** The return signature of `init_sd_card` demands structural stability for the underlying ESP-IDF VFS storage driver trees. `BoardConfig::load()` is now a zero-overhead `const fn` that exposes a `'static` reference layout to ensure effortless driver integration without borrow-checker or lifetime elision issues.

---

## How to Switch Targets
Switching the compilation target changes your entire pin matrix automatically.

1. Open `.cargo/config.toml`.
2. Locate the `[build]` section at the top.
3. Comment out the current target and uncomment your desired target:
   ```toml
   [build]
   # Target for ESP32-S3
   target = "xtensa-esp32s3-espidf"

   # Target for Standard ESP32
   # target = "xtensa-esp32-espidf"
   ```
4. Save the file and run `cargo build`. The build script will automatically swap profiles under the hood.

---

## How to Add a New Board Profile

To add support for a brand new hardware layout or custom board assembly, follow these steps to create and modify the necessary config structures:

### 1. Create the Board Specification File
In the root directory of the project, navigate to the config repository (or where your build configurations are housed) and create a configuration file named after your target triple, following the syntax `<target-triple>.toml` (e.g., `xtensa-esp32c3-espidf.toml` if adding an ESP32-C3 layout).

Populate it with the metadata and exact peripheral mapping definitions required for your layout:
```toml
BOARD_NAME = "My Custom ESP32 DevKit"
BOARD_TARGET = "xtensa-esp32c3-espidf"
BOARD_MCU = "esp32c3"

# SD Card Interface Hardware Pins
SD_CS_PIN = "7"
SD_SCK_PIN = "4"
SD_MOSI_PIN = "6"
SD_MISO_PIN = "5"
SD_SPI_CLOCK_HZ = "20000000" # 20 MHz
```

### 2. Update `build.rs`
Open the root `build.rs` script and ensure it is routing the detected target config file cleanly. Typically, you will update or verify that the target-matching logic correctly parses your new TOML definition into the build pipeline's literal injector loop:

```rust
// Inside build.rs
let target = std::env::var("CARGO_CFG_TARGET_TRIPLE").unwrap();
let config_path = format!("{}.toml", target);

// Parse and re-emit properties as compile-time env definitions
let config_content = std::fs::read_to_string(config_path)
    .expect("Failed to locate matching board configuration specification file.");
// Loop parses key-value pairs and issues: println!("cargo:rustc-env={}={}", key, value);
```

### 3. Verification
Once the metadata matching file is placed, add your new target string under the choice block in `.cargo/config.toml`, make sure the toolchain supports it, and trigger a clean build cycle:
```bash
cargo clean && cargo build
```
