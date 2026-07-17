use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, Configuration, EspWifi, Protocol};
use std::path::Path;

use enumset::enum_set;
use heapless::String;

use rus_toni_esp::board::config::{self, BoardConfig};
use rus_toni_esp::drivers::storage;
use rus_toni_esp::services::storage_service;
use rus_toni_esp::services::web_service::WebServerContext;
use rus_toni_esp::util;

static BOARD: BoardConfig = BoardConfig::load();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    util::print_memory_summary("Baseline at Boot");

    let mut peripherals = Peripherals::take()?;

    // 1. Initialize and Mount SD Card Driver Stack
    let _mounted_fatfs = storage::init_sd_card(&mut peripherals, &BOARD)?;

    // 2. Execute the isolated read/write/delete testing workflow
    if let Err(e) = util::run_sd_card_init_test(config::MOUNT_PATH) {
        log::error!("❌ Transient SD Card initialization verification failed: {:?}", e);
    }

    // List the remaining directory state (Should show only your persistent files)
    storage_service::list_dir(Path::new(config::MOUNT_PATH), 0);

    // 3. Fetch LED pin
    let led_pin = BOARD.get_any_pin(2, &mut peripherals)?;

    // 4. Network Stack Setup
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

    // 5. Run Web Server Services
    let _web_server = WebServerContext::init(led_pin)?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}