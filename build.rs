use std::env;
use std::fs;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize)]
struct BoardToml {
    board: BoardSection,
    spi: SpiSection,
    sd: SdSection,
    rfid: RfidSection,
}

#[derive(Deserialize)]
struct BoardSection {
    name: String,
    target: String,
    mcuversion: String,
}

#[derive(Deserialize)]
struct SpiSection {
    sck_pin: u32,
    miso_pin: u32,
    mosi_pin: u32,
}

#[derive(Deserialize)]
struct SdSection {
    cs_pin: u32,
    spi_clock_hz: u32,
}

#[derive(Deserialize)]
struct RfidSection {
    cs_pin: u32,
    rst_pin: u32,
    spi_clock_hz: u32,
}

fn main() {
    embuild::espidf::sysenv::output();

    // 1. Detect target architecture straight from the active compiler toolchain
    let target = env::var("TARGET").unwrap_or_default();
    
    let board_name = if target.contains("esp32s3") {
        "esp32s3"
    } else if target.contains("esp32") && !target.contains("esp32s2") && !target.contains("esp32s3") {
        "esp32"
    } else {
        panic!("❌ Error: Unsupported compiler target: '{}'. Check your .cargo/config.toml", target);
    };

    // 2. Map the toolchain architecture directly to your configuration layout profile
    let toml_path = format!("config/boards/{}.toml", board_name);
    println!("cargo:rerun-if-changed={}", toml_path);

    if !Path::new(&toml_path).exists() {
        panic!("❌ Error: Configuration board profile layout not found at: {}", toml_path);
    }

    // 3. Process the file contents
    let toml_content = fs::read_to_string(&toml_path)
        .unwrap_or_else(|_| panic!("Failed to read target profile configuration file: {}", toml_path));
    
    let config: BoardToml = toml::from_str(&toml_content)
        .unwrap_or_else(|_| panic!("Failed to parse profile properties inside: {}", toml_path));

    // 4. Inject values to program text segment matching config.rs expectations
    println!("cargo:rustc-env=BOARD_NAME={}", config.board.name);
    println!("cargo:rustc-env=BOARD_TARGET={}", config.board.target);
    println!("cargo:rustc-env=BOARD_MCU={}", config.board.mcuversion);
    
    // Shared Bus Pins
    println!("cargo:rustc-env=SPI_SCK_PIN={}", config.spi.sck_pin);
    println!("cargo:rustc-env=SPI_MISO_PIN={}", config.spi.miso_pin);
    println!("cargo:rustc-env=SPI_MOSI_PIN={}", config.spi.mosi_pin);
    
    // SD Card Parameters
    println!("cargo:rustc-env=SD_CS_PIN={}", config.sd.cs_pin);
    println!("cargo:rustc-env=SD_SPI_CLOCK_HZ={}", config.sd.spi_clock_hz);
    
    // RFID Parameters
    println!("cargo:rustc-env=RFID_CS_PIN={}", config.rfid.cs_pin);
    println!("cargo:rustc-env=RFID_RST_PIN={}", config.rfid.rst_pin);
    println!("cargo:rustc-env=RFID_SPI_CLOCK_HZ={}", config.rfid.spi_clock_hz);
}