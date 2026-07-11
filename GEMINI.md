
# Custom Agent Instructions: rusToniESP Development Context

This file serves as the strict operational persona, technical boundaries, and coding standards for all AI agents assisting with the development of the **rusToniESP** project. 

---

## 🎯 Project Core Profile
- **Project Name:** `rusToniESP` (crate name: `rus-toni-esp`)
- **Target Hardware:** ESP32-S3 / ESP32 (WROOM/WROVER) Microcontrollers
- **Primary Language:** 100% Rust
- **Ecosystem Focus:** Embedded Rust (`esp-idf-hal` & `esp-idf-svc` for `std` features, built on ESP-IDF).
- **Core Intent:** A highly reliable, memory-safe, tangible physical audio player for children, triggered via RFID tags.

---

## 🛠️ Mandatory Technical Constraints

### 1. Absolute Language Enforcement
- **NEVER** write, suggest, or translate snippets into C or C++. 
- **NEVER** utilize standard Arduino `.ino` sketches or framework wrappers.
- All code must be idiomatic, modern Rust.

### 2. Embedded Safety Standards
- Prioritize compile-time safety using Rust's ownership model.
- Leverage type-safe peripheral handling using `esp-idf-hal` abstractions.
- Avoid raw pointers or unchecked `unsafe` code unless absolutely necessary for low-level hardware abstraction layer bindings.
- Use structured logging (`log` crate with `EspLogger`) instead of generic print macros.

### 3. Audio & Peripherals Strategy
- Target standard library (`std`) features where helpful, using ESP-IDF system services (`esp-idf-svc` for Wi-Fi, NVS, system time, and event loops).
- Decouple hardware access by using traits (from standard `embedded-hal` or custom interfaces) to keep code flexible.
- Implement multi-threading/concurrency using standard Rust threads (`std::thread`), mutexes (`std::sync::Mutex` or `esp_idf_hal::mutex::Mutex`), and channels (`std::sync::mpsc` or `embassy-sync`) for smooth background audio rendering and responsive button/RFID interfaces.

---

## 🗺️ Architecture Reference
- The planned module structure, thread layout, event bus design, HAL traits, and Cargo feature flags are documented in **[`architecture.md`](architecture.md)**.
- **The architecture is open and evolving.** As the project develops, module boundaries, trait definitions, and feature flags may be refactored. Always consult the current `architecture.md` before scaffolding new modules or adding dependencies — and update it when significant structural decisions are made.
- When generating new modules or wiring up subsystems, follow the layering rules in `architecture.md` (e.g. modules communicate via the `SystemEvent` / `PlayerCommand` channels, not by calling each other directly).

---

## 🧪 Testing & CI Policy

Every feature or module added to this project **must include tests**. Tests are the primary verification mechanism and are automatically run in CI via **[`.github/workflows/rust_ci.yml`](.github/workflows/rust_ci.yml)**.

### Test tiers

| Tier | What to write | Where it runs |
|------|---------------|---------------|
| **Unit tests** | Pure-logic functions in `core/`, `audio/playlist`, `rfid/config_mapping`, etc. Use `#[test]` inside `#[cfg(test)]` blocks. Mock hardware traits with simple `struct MockXxx`. | `cargo test --lib` on host (CI `unit-tests` job) |
| **Integration / sim tests** | End-to-end flows using Wokwi — assert expected serial output with `--expect-text` / `--fail-text`. Define scenarios in YAML via `--scenario`. | `wokwi-cli` (CI `wokwi-sim` job — enable when first scenario is ready) |

### Rules for agents

1. **New module = new `#[cfg(test)]` block.** Every `mod.rs` or implementation file must contain at least one unit test before it is considered complete.
2. **Hardware traits must be mockable.** When writing a new `trait`, ensure its surface area allows a `MockXxx` struct that implements it without any `esp-idf-hal` dependency. Use `cfg(test)` to swap in mocks.
3. **CI file is a living document.** When adding a new Cargo feature or subsystem, update `rust_ci.yml` accordingly — e.g. add the feature to the `clippy` args, or add a new simulation scenario to the `wokwi-sim` job.
4. **Do not disable failing tests.** Fix the root cause. If a test is temporarily skipped, it must be tracked with a `TODO` comment and a GitHub issue reference.

---

## 🤖 Interaction Directive
When answering questions or generating code updates for this project:
- Keep snippets clean, self-contained, and accompanied by correct `Cargo.toml` dependency requirements.
- Target the Standard library (`std`) environment via ESP-IDF.
- Strictly adhere to standard linting and formatting policies (Ruff-style precision for tools, and idiomatic `rustfmt` conventions).
- Proactively flag potential hardware bottlenecks, pin assignment conflicts, or unsafe peripheral access patterns.
- Note target differences (e.g. `xtensa-esp32s3-espidf` vs `xtensa-esp32-espidf` depending on physical board choice).
- **Simulation / Testing:** Use the `wokwi-cli` terminal tool to run and verify logic in the Wokwi simulator. The MCP server integration (`wokwi-cli mcp`) is **not available** — use only the CLI directly.
  - Simulator config files: `diagram.json` and `wokwi.toml`. Target is `esp32s3` → build for `xtensa-esp32s3-espidf`.
  - **Key `wokwi-cli` options to use:**
    - `--elf <path>` — point to the compiled ELF binary (e.g. `target/xtensa-esp32s3-espidf/debug/<crate>.elf`)
    - `--diagram-file <path>` — override the default `diagram.json` if needed
    - `--timeout <ms>` — set simulation timeout (default 30 000 ms)
    - `--expect-text <string>` — assert expected serial output (use for pass/fail CI checks)
    - `--fail-text <string>` — fail the run if a specific string appears in serial output
    - `--serial-log-file <path>` — capture all serial output to a file for later inspection
    - `--interactive` — redirect stdin to the simulated serial port for manual testing
    - `--scenario <path>` — drive the simulation via a YAML scenario file
    - `--screenshot-part <id> --screenshot-time <ms> --screenshot-file <path>` — capture a screenshot of a specific part at a given simulation time
    - `--vcd-file <path>` — dump a logic-analyzer VCD file for signal tracing
    - `-q / --quiet` — suppress status messages (useful in scripts)
  - **Typical run command:** `wokwi-cli --elf target/xtensa-esp32s3-espidf/debug/rus-toni-esp.elf --timeout 60000 --serial-log-file sim.log .`
  - Ensure hardware features are compatible or modularized to work gracefully during simulation runs.