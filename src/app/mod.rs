// Application startup and wiring logic
use crate::board::config::BoardConfig;
use crate::drivers::storage::{SpiSdCardDriver, StorageDriver};

pub fn init() {
    log::info!("Initializing application...");

    let board_config = BoardConfig::load();
    log::info!("Using board config: {}", board_config.name);
    log::info!("SD CS pin: {}", board_config.sd_cs_pin);

    let storage_driver = SpiSdCardDriver::new("/sdcard");
    match storage_driver.mount() {
        Ok(()) => {
            log::info!("SD card mounted at {}", storage_driver.mount_point());
            log::info!("SD card smoke test completed successfully");
        }
        Err(err) => log::error!("SD card mount failed: {}", err),
    }
}
