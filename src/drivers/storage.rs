use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::sd::spi::SdSpiHostDriver;
use esp_idf_hal::sd::{SdCardConfiguration, SdCardDriver};
use esp_idf_hal::spi::SpiDriver;
use esp_idf_svc::fs::fatfs::Fatfs;
use esp_idf_svc::io::vfs::MountedFatfs;

use crate::board::config::{self, BoardConfig};

pub fn init_sd_card<'a>(
    sd_cs_pin: AnyIOPin<'static>, // Accept the pre-resolved pin directly
    board_config: &'static BoardConfig, 
    spi_driver: &'a SpiDriver<'static>, 
) -> Result<
    MountedFatfs<Fatfs<SdCardDriver<SdSpiHostDriver<'static, &'a SpiDriver<'static>>>>>, 
    Box<dyn std::error::Error>
> {
    log::info!(
        "[Board Profile: {}] Target Architecture: {}", 
        board_config.name, board_config.target
    );

    // Attach the SD Host Driver slot onto the shared master reference
    let sd_spi_host = SdSpiHostDriver::new(
        spi_driver,
        Some(sd_cs_pin),
        None::<AnyIOPin>,              
        None::<AnyIOPin>,              
        None::<AnyIOPin>,              
        Some(true),                        
    )?;

    let mut sd_config = SdCardConfiguration::new();
    sd_config.speed_khz = board_config.sd_spi_clock_hz / 1000; 

    let sd_card_driver = SdCardDriver::new_spi(sd_spi_host, &sd_config)?;
    let fatfs = Fatfs::new_sdcard(0, sd_card_driver)?;
    
    let mounted = MountedFatfs::mount(fatfs, config::MOUNT_PATH, config::MAX_FILE_DESCRIPTORS)?;
    Ok(mounted)
}