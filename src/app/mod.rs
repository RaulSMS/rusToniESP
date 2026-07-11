// Application startup and wiring logic
use crate::drivers::storage::{SpiSdCardDriver, StorageDriver};

pub fn init() {
    log::info!("Initializing application...");

    let storage_driver = SpiSdCardDriver::new("/sdcard");
    match storage_driver.mount() {
        Ok(()) => {
            log::info!("SD card mounted at {}", storage_driver.mount_point());

            let mount_point = storage_driver.mount_point();
            if storage_driver.is_mounted() {
                log::info!("SD card mount point is ready: {}", mount_point);
                log::info!("SD card smoke test completed successfully");
            } else {
                log::error!("SD card mount point was not marked as mounted");
            }
        }
        Err(err) => log::error!("SD card mount failed: {}", err),
    }
}
