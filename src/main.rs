use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, Configuration, EspWifi, Protocol};
use std::fs::File; // Required for SD card test
use std::io::Write; // Required for SD card test
use std::path::Path; // Required for SD card test

use enumset::enum_set;
use heapless::String;

use rus_toni_esp::board::config::{self, BoardConfig};
use rus_toni_esp::drivers::storage;
use rus_toni_esp::services::storage_service; // Required for SD card check
use rus_toni_esp::services::web_service::WebServerContext;
use rus_toni_esp::util; // Required for memory summary

static BOARD: BoardConfig = BoardConfig::load();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    util::print_memory_summary("Baseline at Boot");

    let mut peripherals = Peripherals::take()?;

    // 1. Initialize and Mount SD Card (Kept for verification)
    let _mounted_fatfs = storage::init_sd_card(&mut peripherals, &BOARD)?;

    let file_path = format!("{}/RUST_LOG.TXT", config::MOUNT_PATH);
    if let Ok(mut file) = File::create(&file_path) {
        let _ = file.write_all(b"SD Card Verify: OK\n");
    }

    if let Err(e) = storage_service::generate_nested_test_files(config::MOUNT_PATH) {
        log::error!("❌ SD Card test files failed: {:?}", e);
    }
    storage_service::list_dir(Path::new(config::MOUNT_PATH), 0);

    // 2. Fetch LED pin
    let led_pin = BOARD.get_any_pin(2, &mut peripherals)?;

    // 3. Network Stack
    let nvs = EspDefaultNvsPartition::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let mut wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;

    let ssid = String::<32>::try_from("ESP32-Toni-Net")?;
    let password = String::<64>::try_from("")?;

    let ap_config = AccessPointConfiguration {
        ssid,
        ssid_hidden: false,
        channel: 6,
        secondary_channel: None,
        protocols: enum_set!(Protocol::P802D11BGN),
        auth_method: AuthMethod::None,
        password,
        max_connections: 4,
    };

    wifi.set_configuration(&Configuration::AccessPoint(ap_config))?;
    wifi.start()?;

    // 4. Web Server
    let _web_server = WebServerContext::init(led_pin)?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}