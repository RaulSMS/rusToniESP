# rusToniESP 🦀📻

> **⚠️ Learning Project & Proof of Concept — Not for Production Use**
>
> `rusToniESP` is a **personal learning project** created to explore embedded Rust on ESP32
> hardware. It is a **Rust port / reimplementation** of the fantastic
> **[ESPuino](https://github.com/biologist79/ESPuino)** project by
> [@biologist79](https://github.com/biologist79), which is a mature, battle-tested, feature-rich
> RFID audio player built on the Arduino/ESP-IDF ecosystem.
>
> **If you are looking for a reliable, production-ready device for your children, please use the
> original [ESPuino](https://github.com/biologist79/ESPuino).** This project is a PoC / beta and
> makes no guarantees of stability, feature completeness, or hardware compatibility.

---

## 🎓 Why This Exists

This project exists to:
- Learn **idiomatic embedded Rust** with `esp-idf-hal` / `esp-idf-svc` on real hardware.
- Explore how a well-architected C++ embedded project can be translated into Rust's ownership and type-safety model.
- Experiment with Rust concurrency primitives (`mpsc`, `Arc<Mutex<T>>`, `std::thread`) in a constrained embedded environment.

All credit for the original concept, hardware design, and feature set belongs to the
**[ESPuino project](https://github.com/biologist79/ESPuino)** and its community.

---

A modern, type-safe DIY physical audio player for children, inspired by **TonUINO** and **ESPuino**. Built entirely in **Rust** using the ESP-IDF standard library ecosystem (`esp-idf-hal` / `esp-idf-svc`) targeting Espressif microcontrollers.

`rusToniESP` pairs the physical interaction of RFID tags/figures with a reliable, memory-safe, and asynchronous embedded Rust core.


## 🎯 Features
- **High-Quality Audio:** Supports reading audio files (MP3/WAV) from local Micro-SD and future audio streaming over Wi-Fi.
- **Physical RFID Trigger:** Detects placed figures or cards to play specific music/audiobooks.
- **Robust Asymmetric Core:** Multi-threaded or cooperative asynchronous loops to decouple audio rendering from hardware button/RFID polling.
- **Memory Safety:** Rust guarantees compile-time memory safety, eliminating memory leaks or random crashes common in C++ DIY players.

---

## 🛠️ Hardware Stack
- **MCU:** ESP32-S3 (or ESP32-WROOM/WROVER)
- **RFID Reader:** RC522 or PN532 (SPI/I2C)
- **Audio DAC/Amp:** MAX98357A (I2S)
- **Storage:** Micro-SD Card Breakout (SPI/SDMMC)

---

## 🚀 Getting Started

### 1. Prerequisites
Ensure you have the Rust compiler and Espressif Xtensa toolchain installed.

```bash
# Sourcing the Espressif toolchain (runs in current session)
. $HOME/export-esp.sh
```

### 2. Configure Your MCU Target
Before building or running, verify your MCU target in [.cargo/config.toml](file:///.cargo/config.toml):

* **For ESP32-S3 (Recommended / Simulator default):**
  ```toml
  target = "xtensa-esp32s3-espidf"
  MCU = "esp32s3"
  ```
* **For standard ESP32 (WROOM/WROVER):**
  ```toml
  target = "xtensa-esp32-espidf"
  MCU = "esp32"
  ```

---

## 🖥️ Simulation & Testing with Wokwi

`rusToniESP` supports hardware-accurate software simulation using the **Wokwi** simulator. This allows you to test code changes directly from VS Code without needing physical hardware connected.

### Wokwi Configuration Files
- [diagram.json](file:///diagram.json): Defines the virtual board layout (currently configured with a virtual ESP32-S3).
- [wokwi.toml](file:///wokwi.toml): Specifies the ELF file and firmware paths to use in the simulation.

### Running the Simulator
1. Install the **Wokwi** extension in VS Code.
2. Authenticate the extension with your free Wokwi account when prompted.
3. Build the project to generate the binary debug artifact:
   ```bash
   cargo build
   ```
4. Press `Cmd + Shift + P` (macOS) or `Ctrl + Shift + P` (Windows/Linux) in VS Code.
5. Select **Wokwi: Start Simulator** to boot up the virtual board.
6. The simulator terminal will display the serial log output: `Hello, world!`

---

## 📦 Flashing to Physical Hardware

Connect your microcontroller via USB and run:

```bash
# Builds and flashes directly to the target port, then opens serial monitor
espflash flash --monitor --port /dev/cu.usbserial-0001
```

> Note: Ensure your serial port is correct (e.g. `/dev/cu.usbserial-0001` or `/dev/cu.usbmodem*`).
