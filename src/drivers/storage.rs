use esp_idf_hal::gpio::AnyInputPin;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::sd::spi::SdSpiHostDriver;
use esp_idf_hal::sd::{SdCardConfiguration, SdCardDriver};
use esp_idf_hal::spi::{Dma, SpiDriver, SpiDriverConfig};
use esp_idf_svc::fs::fatfs::Fatfs;
use esp_idf_svc::io::vfs::MountedFatfs;
use crate::board::config;

pub fn init_sd_card(
    peripherals: Peripherals,
) -> Result<
    MountedFatfs<Fatfs<SdCardDriver<SdSpiHostDriver<'static, SpiDriver<'static>>>>>, 
    Box<dyn std::error::Error>
> {
    log::info!("[Debug] Initializing SPI2 host (SCLK: 18, MOSI: 23, MISO: 19)...");
    
    let spi_config = SpiDriverConfig::new().dma(Dma::Auto(4096));

    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio23,
        Some(peripherals.pins.gpio19),
        &spi_config,
    )?;

    log::info!("[Debug] Configuring SPI SD Card Driver with CS on GPIO 5...");
    
    let sd_spi_host = SdSpiHostDriver::new(
        spi_driver,
        Some(peripherals.pins.gpio5),
        None::<AnyInputPin>,              
        None::<AnyInputPin>,              
        None::<AnyInputPin>,              
        Some(true),                        
    )?;

    let mut sd_config = SdCardConfiguration::new();
    sd_config.speed_khz = config::SPI_SPEED_KHZ; 

    let sd_card_driver = SdCardDriver::new_spi(sd_spi_host, &sd_config)?;

    log::info!("👉 Attempting to mount SD Card Volume to VFS structure at '{}'...", config::MOUNT_PATH);
    
    let fatfs = Fatfs::new_sdcard(0, sd_card_driver)?;
    let mounted = MountedFatfs::mount(fatfs, config::MOUNT_PATH, config::MAX_FILE_DESCRIPTORS)?;
    
    log::info!("✅ SD Card mounted successfully at '{}'!", config::MOUNT_PATH);
    Ok(mounted)
}