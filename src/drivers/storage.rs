use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::sd::spi::SdSpiHostDriver;
use esp_idf_hal::sd::{SdCardConfiguration, SdCardDriver};
use esp_idf_hal::spi::{Dma, SpiDriver, SpiDriverConfig};
use esp_idf_svc::fs::fatfs::Fatfs;
use esp_idf_svc::io::vfs::MountedFatfs;

use crate::board::config::{self, BoardConfig};

pub fn init_sd_card(
    peripherals: &mut Peripherals,
    board_config: &'static BoardConfig, // Config has a 'static lifetime layout
) -> Result<
    MountedFatfs<Fatfs<SdCardDriver<SdSpiHostDriver<'static, SpiDriver<'static>>>>>, 
    Box<dyn std::error::Error>
> {
    log::info!(
        "[Board Profile: {}] Target Architecture: {}", 
        board_config.name, board_config.target
    );

    let sclk = board_config.get_any_pin(board_config.sd_sck_pin, peripherals)?;
    let mosi = board_config.get_any_pin(board_config.sd_mosi_pin, peripherals)?;
    let miso = board_config.get_any_pin(board_config.sd_miso_pin, peripherals)?;
    let cs = board_config.get_any_pin(board_config.sd_cs_pin, peripherals)?;

    let spi_config = SpiDriverConfig::new().dma(Dma::Auto(4096));

    // Safely extract the SPI2 registration token to avoid E0507 mutable move blocks
    let spi2_peripheral = unsafe { core::ptr::read(&peripherals.spi2) };

    let spi_driver = SpiDriver::new(
        spi2_peripheral, 
        sclk,
        mosi,
        Some(miso),
        &spi_config,
    )?;

    let sd_spi_host = SdSpiHostDriver::new(
        spi_driver,
        Some(cs),
        None::<AnyIOPin>,              
        None::<AnyIOPin>,              
        None::<AnyIOPin>,              
        Some(true),                        
    )?;

    let mut sd_config = SdCardConfiguration::new();
    sd_config.speed_khz = board_config.spi_clock_hz / 1000; 

    let sd_card_driver = SdCardDriver::new_spi(sd_spi_host, &sd_config)?;
    let fatfs = Fatfs::new_sdcard(0, sd_card_driver)?;
    
    let mounted = MountedFatfs::mount(fatfs, config::MOUNT_PATH, config::MAX_FILE_DESCRIPTORS)?;
    Ok(mounted)
}