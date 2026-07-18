use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, Configuration, EspWifi, Protocol};
use std::path::Path;
use std::sync::Arc;

use enumset::enum_set;
use heapless::String;

// esp-idf-hal imports for lower-level shared SPI client creation
use esp_idf_svc::hal::gpio::AnyOutputPin;
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver, config::DriverConfig, Dma};
use esp_idf_svc::hal::spi::config::Config as SpiConfig;

use rus_toni_esp::board::config::{self, BoardConfig};
use rus_toni_esp::drivers::storage;
use rus_toni_esp::services::storage_service;
use rus_toni_esp::services::web_service::WebServerContext;
use rus_toni_esp::services::rfid_service; 
use rus_toni_esp::util;

static BOARD: BoardConfig = BoardConfig::load();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    util::print_memory_summary("Baseline at Boot");

    let mut peripherals = Peripherals::take()?;

    log::info!("🎛️ Initializing Master Shared SPI Bus Controller...");
    
    // Resolve bus I/O configurations directly from the board TOML profile
    let sclk = BOARD.get_any_pin(BOARD.spi_sck_pin, &mut peripherals)?;
    let mosi = BOARD.get_any_pin(BOARD.spi_mosi_pin, &mut peripherals)?;
    let miso = BOARD.get_any_pin(BOARD.spi_miso_pin, &mut peripherals)?;
    
    let sd_cs_pin = BOARD.get_any_pin(BOARD.sd_cs_pin, &mut peripherals)?;
    
    let rfid_cs_pin = BOARD.get_any_pin(BOARD.rfid_cs_pin, &mut peripherals)?; 
    let rfid_rst_pin = BOARD.get_any_pin(BOARD.rfid_rst_pin, &mut peripherals)?; 
    
    let led_pin = BOARD.get_any_pin(2, &mut peripherals)?;

    // Configure DriverConfig to use DMA, enabling large block transfers required for SD/FATFS
    let mut driver_config = DriverConfig::new();
    driver_config.dma = Dma::Channel2(4096);
    
    // Allocate the underlying hardware bus context inside an Arc container
    let spi_driver = Arc::new(SpiDriver::new(
        peripherals.spi2,
        sclk,
        mosi,
        Some(miso),
        &driver_config,
    )?);

    // 2. Initialize and Mount SD Card Driver Stack using pre-resolved pin
    let _mounted_fatfs = storage::init_sd_card(sd_cs_pin, &BOARD, &spi_driver)?;

    // 3. Execute the isolated read/write/delete testing workflow
    if let Err(e) = util::run_sd_card_init_test(config::MOUNT_PATH) {
        log::error!("❌ Transient SD Card initialization verification failed: {:?}", e);
    }

    // List the remaining directory state
    storage_service::list_dir(Path::new(config::MOUNT_PATH), 0);

    // -------------------------------------------------------------------------
    // 💡 RFID Subsystem Integration (Shared SPI Client Slot)
    // -------------------------------------------------------------------------
    log::info!("🛰️ Attaching RFID Device Client onto Shared SPI Bus Layer...");

    // Device Config for MFRC522 RFID Interface pulled directly from unified TOML configurations
    let mut rfid_spi_config = SpiConfig::new();
    rfid_spi_config.baudrate = BOARD.rfid_spi_clock_hz.into();

    // Register a secondary client device slot on the shared master reference
    let rfid_spi_device = SpiDeviceDriver::new(
        spi_driver.clone(),
        Some(rfid_cs_pin),
        &rfid_spi_config,
    )?;

    // Fire up the background polling thread with type-erased AnyOutputPin
    let type_erased_rst: AnyOutputPin = rfid_rst_pin.into();
    rfid_service::start_rfid_polling_service(rfid_spi_device, type_erased_rst);
    // -------------------------------------------------------------------------

    // 4. Network Stack Setup (moves peripherals.modem safely now)
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

    // Loop executing tracking statistics every 5 seconds
    let mut loop_counter = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        loop_counter += 1;
        
        if loop_counter >= 5 {
            util::print_memory_summary("Runtime Maintenance Check");
            loop_counter = 0;
        }
    }
}