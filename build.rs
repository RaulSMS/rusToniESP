use std::env;
use std::fs;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize)]
struct BoardToml {
    board: BoardSection,
    sd: SdSection,
}

#[derive(Deserialize)]
struct BoardSection {
    name: String,
    target: String,
    mcuversion: String,
}

#[derive(Deserialize)]
struct SdSection {
    cs_pin: u32,
    sck_pin: u32,
    miso_pin: u32,
    mosi_pin: u32,
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

    // 4. Inject values to program text segment
    println!("cargo:rustc-env=BOARD_NAME={}", config.board.name);
    println!("cargo:rustc-env=BOARD_TARGET={}", config.board.target);
    println!("cargo:rustc-env=BOARD_MCU={}", config.board.mcuversion);
    println!("cargo:rustc-env=SD_CS_PIN={}", config.sd.cs_pin);
    println!("cargo:rustc-env=SD_SCK_PIN={}", config.sd.sck_pin);
    println!("cargo:rustc-env=SD_MISO_PIN={}", config.sd.miso_pin);
    println!("cargo:rustc-env=SD_MOSI_PIN={}", config.sd.mosi_pin);
    println!("cargo:rustc-env=SD_SPI_CLOCK_HZ={}", config.sd.spi_clock_hz);
}