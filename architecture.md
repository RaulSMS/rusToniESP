# rusToniESP – Rust Architecture

> **Reference:** ESPuino feature set — [github.com/biologist79/ESPuino](https://github.com/biologist79/ESPuino)
> **Target MCU:** ESP32-S3 (`xtensa-esp32s3-espidf`) · **Runtime:** `std` via ESP-IDF

---

## Design Principles

| Principle | How it is achieved |
|-----------|-------------------|
| **Modularity** | Each subsystem lives in its own module directory (`src/<subsystem>/mod.rs`). Cross-cutting concerns are kept in `src/core/`. |
| **Reusability** | Every hardware-touching module is hidden behind a trait (e.g. `RfidReader`, `AudioBackend`, `LedDriver`). New hardware variants add a new `impl`, not new call sites. |
| **Expandability** | Optional features are gated with Cargo `features`. Enabling a feature pulls in its module; disabling it compiles it away entirely (zero-cost). |
| **Concurrency** | Threads communicate exclusively through typed `std::sync::mpsc` channels and `Arc<Mutex<T>>` shared state. No global mutable state, no `unsafe` statics. |
| **Safety** | No raw pointers outside ESP-IDF FFI boundary. All peripherals accessed through `esp-idf-hal` typed wrappers. |

---

## High-Level Layer Diagram

```
+-------------------------------------------------------------+
|                    main.rs  (boot & wiring)                 |
+----------------+--------------------------------------------+
                 |             |
+----------------v---+  +------v------------------------------------+
|  core/             |  |  connectivity/                           |
|  +- config         |  |  +- wifi                                 |
|  +- events         |  |  +- mqtt        (feature-gated)          |
|  +- commands       |  |  +- ftp         (feature-gated)          |
|  +- error          |  |  +- web_server                           |
+--------------------+  |  +- rest_api                             |
                        +-------------------------------------------+
+-------------------------------------------------------------+
|  audio/                                                     |
|  +- player     (playback state machine)                     |
|  +- playlist   (track ordering, shuffle, repeat)            |
|  +- i2s_out    (I2S DAC driver — MAX98357A / PT2811)        |
|  +- web_stream (HTTP audio streaming)                       |
|  +- bluetooth  (A2DP sink + source, feature-gated)         |
+-------------------------------------------------------------+
+--------------------+  +--------------------------------------+
|  rfid/             |  |  storage/                            |
|  +- reader (trait) |  |  +- sdcard  (SDMMC / SPI)           |
|  +- mfrc522_spi    |  |  +- nvs     (ESP-IDF NVS)           |
|  +- mfrc522_i2c    |  |  +- playlist_db                     |
|  +- pn5180         |  +--------------------------------------+
|  +- runtime_detect |  +--------------------------------------+
|  +- config_mapping |  |  ui/                                 |
+--------------------+  |  +- buttons   (up to 5 + debounce)  |
                        |  +- rotary_encoder                   |
                        |  +- led       (NeoPixel WS2812B)     |
                        |  +- ir_receiver (feature-gated)      |
                        +--------------------------------------+
+-------------------------------------------------------------+
|  power/                                                     |
|  +- battery    (ADC measurement + LUT)                      |
|  +- sleep      (light/deep sleep, LPCD wake)                |
|  +- shutdown   (graceful halt + deepsleep fallback)         |
+-------------------------------------------------------------+
+-------------------------------------------------------------+
|  hal/  (Board-specific pin assignments & bus configs)       |
|  +- board_esp32s3_devkit.rs                                 |
|  +- board_custom.rs                                         |
+-------------------------------------------------------------+
```

---

## Directory Layout

```
src/
+-- main.rs                   # Boot: init HAL, spawn threads, wire channels
|
+-- core/
|   +-- mod.rs
|   +-- config.rs             # NVS-backed runtime config (volume, hostname, ...)
|   +-- events.rs             # SystemEvent enum - the bus all modules publish to
|   +-- commands.rs           # PlayerCommand enum consumed by audio::player
|   +-- error.rs              # AppError, thiserror wrappers
|
+-- hal/
|   +-- mod.rs                # re-exports the active board config
|   +-- board_esp32s3.rs      # Wokwi / devkit pin map
|   +-- board_custom.rs       # Template for custom PCBs
|
+-- audio/
|   +-- mod.rs
|   +-- player.rs             # Playback FSM (Playing/Paused/Stopped/...)
|   +-- playlist.rs           # Playlist, TrackMode (single/folder/random/...)
|   +-- i2s_out.rs            # esp-idf-hal I2S driver, volume curve LUT
|   +-- web_stream.rs         # HTTP chunked audio streaming (feature = "webstream")
|   +-- bluetooth.rs          # A2DP sink/source (feature = "bluetooth")
|
+-- rfid/
|   +-- mod.rs
|   +-- reader.rs             # trait RfidReader { fn poll(&mut self) -> Option<TagId>; }
|   +-- mfrc522_spi.rs        # MFRC522 via SPI
|   +-- mfrc522_i2c.rs        # MFRC522 via I2C (feature = "rfid_i2c")
|   +-- pn5180.rs             # PN5180 (ISO-15693 / Tonies) (feature = "pn5180")
|   +-- runtime_detect.rs     # Auto-detect reader type at boot
|   +-- config_mapping.rs     # TagId -> Action DB (NVS + SD)
|
+-- storage/
|   +-- mod.rs
|   +-- sdcard.rs             # SD-MMC (1-bit) + SPI fallback, trait FileSystem
|   +-- nvs.rs                # Typed NVS wrapper (settings, last-played, ...)
|   +-- playlist_db.rs        # Scans SD for audio files, builds playlists
|
+-- ui/
|   +-- mod.rs
|   +-- buttons.rs            # Up to 5 GPIO buttons, debounce, short/long press
|   +-- rotary_encoder.rs     # Quadrature encoder + push (feature = "rotary")
|   +-- led.rs                # NeoPixel ring (RMT-based WS2812B)
|   +-- ir_receiver.rs        # IR remote control (feature = "ir_remote")
|
+-- connectivity/
|   +-- mod.rs
|   +-- wifi.rs               # STA + AP mode, captive portal for first-run
|   +-- web_server.rs         # Embedded HTTP server (esp-idf-svc::http)
|   +-- rest_api.rs           # REST endpoints (file browse, RFID assign, settings)
|   +-- mqtt.rs               # MQTT pub/sub (feature = "mqtt")
|   +-- ftp.rs                # FTP server for SD uploads (feature = "ftp")
|
+-- power/
    +-- mod.rs
    +-- battery.rs            # ADC voltage divider, % estimation, LUT
    +-- sleep.rs              # Light sleep / deep sleep entry, LPCD wake (PN5180)
    +-- shutdown.rs           # Graceful shutdown: flush NVS, stop threads, sleep
```

---

## Central Event Bus

All modules communicate through a single `SystemEvent` enum passed via `std::sync::mpsc` channels.
No module calls another module's functions directly — they emit events or commands.

```rust
// src/core/events.rs

pub enum SystemEvent {
    // RFID
    TagDetected(TagId),
    TagRemoved,

    // Audio
    PlaybackStarted { track: TrackInfo },
    PlaybackStopped,
    PlaybackProgress { position_ms: u32, duration_ms: u32 },
    VolumeChanged(u8),

    // UI
    ButtonPressed { id: ButtonId, kind: PressKind },
    RotaryTurned(i8),  // +1 / -1

    // Power
    BatteryLevel(u8),   // 0-100 %
    ShutdownRequested,

    // Connectivity
    WifiConnected,
    WifiDisconnected,
    MqttMessage { topic: String, payload: String },
}
```

```rust
// src/core/commands.rs  (sent TO the audio player thread)

pub enum PlayerCommand {
    Play(PlaySource),
    Pause,
    Resume,
    Stop,
    Next,
    Previous,
    SetVolume(u8),
    Seek(u32),
    SetMode(TrackMode),
    EnableSleep(bool),
}

pub enum PlaySource {
    SdFile(String),       // absolute path on SD
    SdFolder(String),     // folder - playlist built at runtime
    WebStream(String),    // URL
    Bluetooth,
}
```

---

## Thread Layout

| Thread | Owns | Communicates via |
|--------|------|-----------------|
| **main** | boot, all spawning | — |
| **rfid_thread** | RFID reader peripheral | `event_tx: Sender<SystemEvent>` |
| **audio_thread** | I2S peripheral, playback FSM | `cmd_rx: Receiver<PlayerCommand>`, `event_tx` |
| **ui_thread** | buttons, rotary encoder | `event_tx` |
| **led_thread** | NeoPixel ring | `event_rx: Receiver<SystemEvent>` (subscriber clone) |
| **connectivity_thread** | WiFi, HTTP server, MQTT, FTP | `event_tx`, `cmd_tx: Sender<PlayerCommand>` |
| **power_thread** | ADC battery, sleep logic | `event_tx`, `event_rx` |

---

## Cargo Feature Flags

```toml
[features]
default = ["neopixel", "rotary", "sdmmc", "rfid_auto"]

# RFID readers
rfid_auto   = []          # runtime auto-detect (default)
rfid_spi    = []          # force MFRC522 SPI
rfid_i2c    = []          # force MFRC522 I2C
pn5180      = []          # enable PN5180 (Tonies, ISO-15693)

# SD interface
sdmmc       = []          # SD-MMC 1-bit (faster, default)
sd_spi      = []          # SPI SD (fallback)

# UI
neopixel    = []          # WS2812B LED ring
rotary      = []          # rotary encoder
ir_remote   = []          # IR receiver

# Connectivity
mqtt        = []
ftp         = []
webstream   = []          # HTTP audio streaming

# Audio output
bluetooth   = []          # A2DP sink + source
headphone   = []          # headphone jack volume/mono adjust
play_mono   = []          # sum stereo -> mono for single speaker

# Power
battery_adc = []          # battery voltage measurement via ADC
lpcd_wake   = ["pn5180"] # low-power card detection wake-up
```

---

## Key Traits (HAL Abstraction Layer)

```rust
// Swap hardware without changing call sites

pub trait RfidReader: Send {
    fn poll(&mut self) -> Option<TagId>;
    fn reader_type(&self) -> RfidReaderType;
}

pub trait AudioBackend: Send {
    fn play(&mut self, source: &PlaySource) -> Result<(), AppError>;
    fn pause(&mut self);
    fn resume(&mut self);
    fn set_volume(&mut self, level: u8);
    fn seek(&mut self, ms: u32);
    fn position_ms(&self) -> u32;
}

pub trait LedDriver: Send {
    fn show_idle(&mut self);
    fn show_progress(&mut self, fraction: f32);
    fn show_volume(&mut self, level: u8);
    fn show_battery(&mut self, percent: u8);
    fn show_error(&mut self);
    fn off(&mut self);
}

pub trait FileSystem: Send + Sync {
    fn open_file(&self, path: &str) -> Result<Box<dyn Read + Send>, AppError>;
    fn list_dir(&self, path: &str) -> Result<Vec<DirEntry>, AppError>;
    fn write_file(&self, path: &str, data: &[u8]) -> Result<(), AppError>;
}
```

---

## Configuration & Persistence

| What | Where | Notes |
|------|-------|-------|
| Runtime settings (hostname, volume, sleep timer) | NVS namespace `settings` | Typed wrappers in `storage::nvs` |
| RFID → Action mappings | NVS namespace `rfid` | Key = tag ID (hex), value = JSON |
| Last played track/position | NVS namespace `lastplay` | Written on pause/stop, restored on boot |
| Audio files | SD card | Discovered at boot, lazily indexed |
| Web interface assets | Embedded in firmware via `include_bytes!` | Served from `connectivity::web_server` |

---

## ESPuino Feature → Rust Module Mapping

| ESPuino module / feature | rusToniESP module | Cargo feature |
|--------------------------|-------------------|---------------|
| `AudioPlayer.cpp` | `audio::player` + `audio::i2s_out` | always on |
| `Playlist.h` | `audio::playlist` | always on |
| `SdCard.cpp` | `storage::sdcard` | `sdmmc` / `sd_spi` |
| `Rfid*.cpp` | `rfid::*` | `rfid_auto` / `pn5180` |
| `RfidConfig.cpp` | `rfid::config_mapping` | always on |
| `Button.cpp` | `ui::buttons` | always on |
| `RotaryEncoder.cpp` | `ui::rotary_encoder` | `rotary` |
| `Led.cpp` | `ui::led` | `neopixel` |
| `IrReceiver.cpp` | `ui::ir_receiver` | `ir_remote` |
| `Wlan.cpp` | `connectivity::wifi` | always on |
| `Web.cpp` | `connectivity::web_server` + `rest_api` | always on |
| `Mqtt.cpp` | `connectivity::mqtt` | `mqtt` |
| `Ftp.cpp` | `connectivity::ftp` | `ftp` |
| `Battery.cpp` + `BatteryMeasureVoltage.cpp` | `power::battery` | `battery_adc` |
| `Power.cpp` | `power::sleep` + `power::shutdown` | always on |
| `Bluetooth.cpp` | `audio::bluetooth` | `bluetooth` |
| `System.cpp` | `core::config` + `core::events` | always on |
| `Cmd.cpp` / `Queues.cpp` | `core::commands` (typed channels) | always on |
| `Port.cpp` (PCA9555) | `hal::port_expander` | (HAL-level, not a feature flag) |
| `HallEffectSensor.cpp` | `ui::hall_sensor` *(future)* | `hall_sensor` |
| Settings HAL files | `hal::board_*.rs` | selected at compile time |

---

## Implementation Phases

### Phase 1 — Foundation (current state)
- [x] Cargo workspace, `esp-idf-svc` scaffold, EspLogger, Wokwi sim

### Phase 2 — Core I/O
- [ ] `hal::board_esp32s3` pin map
- [ ] `storage::sdcard` (SDMMC 1-bit)
- [ ] `ui::buttons` with debounce + short/long press detection
- [ ] `ui::led` (RMT-based WS2812B)

### Phase 3 — RFID & Playback
- [ ] `rfid::reader` trait + `rfid::mfrc522_spi` implementation
- [ ] `rfid::config_mapping` (NVS tag -> action lookup)
- [ ] `audio::i2s_out` (MAX98357A via I2S)
- [ ] `audio::player` FSM + `audio::playlist`

### Phase 4 — Connectivity
- [ ] `connectivity::wifi` (STA + AP + captive portal)
- [ ] `connectivity::web_server` + `connectivity::rest_api`
- [ ] `storage::nvs` typed wrappers

### Phase 5 — Extended Features
- [ ] `connectivity::mqtt`
- [ ] `connectivity::ftp`
- [ ] `audio::web_stream`
- [ ] `power::battery` + `power::sleep`
- [ ] `ui::rotary_encoder`

### Phase 6 — Advanced
- [ ] `rfid::pn5180` (ISO-15693 / Tonies, LPCD wake)
- [ ] `audio::bluetooth` (A2DP)
- [ ] `ui::ir_receiver`

---

> **Tip:** Every new hardware variant (RFID reader, DAC chip, LED controller) should be added as a
> new `impl` of the relevant trait — never by adding `if cfg!(feature = "x")` branches inside
> existing logic.
>
> **Note:** Use the `/goal` slash command to kick off a long-running implementation session
> targeting a specific phase.
