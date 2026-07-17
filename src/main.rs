use esp_idf_hal::peripherals::Peripherals;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use rus_toni_esp::board::config;
use rus_toni_esp::drivers::storage;
use rus_toni_esp::services::storage_service;
use rus_toni_esp::util;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // CRITICAL: Links internal lower level C hooks required for ESP32 runtimes
    esp_idf_svc::sys::link_patches();

    // Initialize global log macro router
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("=== SPI SD Card Debugging via ESP-IDF VFS std::fs ===");
    
    util::print_memory_summary("Baseline at Boot");

    let peripherals = Peripherals::take()?;

    // Initialize and Mount SD Card via infrastructure drivers
    let _mounted_fatfs = storage::init_sd_card(peripherals)?;

    let file_path = format!("{}/RUST_LOG.TXT", config::MOUNT_PATH);

    log::info!("👉 Writing a new file 'RUST_LOG.TXT' using std::fs::File...");
    match File::create(&file_path) {
        Ok(mut file) => {
            let data = b"Hello from Rust on Wokwi ESP32!\nWritten dynamically using the native std::fs library via ESP-IDF VFS.";
            match file.write_all(data) {
                Ok(_) => log::info!("✅ Successfully wrote to {}", file_path),
                Err(e) => log::error!("❌ Failed to write to file: {:?}", e),
            }
        }
        Err(e) => log::error!("❌ Failed to create file: {:?}", e),
    }

    log::info!("👉 Reading file contents using std::fs::read_to_string...");
    match fs::read_to_string(&file_path) {
        Ok(contents) => {
            log::info!("📖 File read successfully! Contents below:\n-----------------------------\n{}\n-----------------------------", contents);
        }
        Err(e) => log::error!("❌ Failed to read file: {:?}", e),
    }

    if let Err(e) = storage_service::generate_nested_test_files(config::MOUNT_PATH) {
        log::error!("❌ Error generating stress test files: {:?}", e);
    }

    util::print_memory_summary("Before Traversal");

    log::info!("\n📂 Listando contenido completo de la raíz del disco:");
    log::info!("------------------------------------------------------------");
    
    storage_service::list_dir(Path::new(config::MOUNT_PATH), 0);
    
    log::info!("------------------------------------------------------------");

    util::print_memory_summary("After Traversal (Peak Memory & Stack Usage Checked)");

    log::info!("🎉 SD Card self-check finished successfully! Exiting main program execution.");
    Ok(())
}