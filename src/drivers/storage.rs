use std::fmt;
use std::sync::Mutex;

use embedded_hal::spi::MODE_0;
use embedded_sdmmc::{Mode, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use embedded_io::Write;
use esp_idf_hal::units::FromValueType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    MountFailed,
    UnmountFailed,
    NotMounted,
    AlreadyMounted,
    IoError,
    FilesystemNotAvailable,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::MountFailed => write!(f, "Failed to mount storage"),
            StorageError::UnmountFailed => write!(f, "Failed to unmount storage"),
            StorageError::NotMounted => write!(f, "Storage is not mounted"),
            StorageError::AlreadyMounted => write!(f, "Storage is already mounted"),
            StorageError::IoError => write!(f, "I/O error occurred on storage"),
            StorageError::FilesystemNotAvailable => write!(f, "SD filesystem is not available"),
        }
    }
}

impl std::error::Error for StorageError {}

pub trait StorageDriver {
    /// Mounts the SD Card filesystem.
    fn mount(&self) -> Result<(), StorageError>;

    /// Unmounts the PDF/SD Card filesystem.
    fn unmount(&self) -> Result<(), StorageError>;

    /// Returns the mount point path (e.g. "/sdcard").
    fn mount_point(&self) -> &str;

    /// Checks if the storage media is mounted.
    fn is_mounted(&self) -> bool;
}

pub struct MockStorageDriver {
    mounted: Mutex<bool>,
    mount_point: String,
}

impl MockStorageDriver {
    pub fn new(mount_point: impl Into<String>) -> Self {
        Self {
            mounted: Mutex::new(false),
            mount_point: mount_point.into(),
        }
    }
}

pub struct SpiSdCardDriver {
    mounted: Mutex<bool>,
    mount_point: String,
}

impl SpiSdCardDriver {
    pub fn new(mount_point: impl Into<String>) -> Self {
        Self {
            mounted: Mutex::new(false),
            mount_point: mount_point.into(),
        }
    }
}

struct SdCardClock;

impl TimeSource for SdCardClock {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp::from_calendar(2024, 1, 1, 0, 0, 0).unwrap_or(Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        })
    }
}

impl StorageDriver for MockStorageDriver {
    fn mount(&self) -> Result<(), StorageError> {
        let mut mounted = self.mounted.lock().unwrap();
        if *mounted {
            return Err(StorageError::AlreadyMounted);
        }

        std::fs::create_dir_all(&self.mount_point).map_err(|_| StorageError::MountFailed)?;
        *mounted = true;
        Ok(())
    }

    fn unmount(&self) -> Result<(), StorageError> {
        let mut mounted = self.mounted.lock().unwrap();
        if !*mounted {
            return Err(StorageError::NotMounted);
        }
        *mounted = false;
        Ok(())
    }

    fn mount_point(&self) -> &str {
        &self.mount_point
    }

    fn is_mounted(&self) -> bool {
        *self.mounted.lock().unwrap()
    }
}

impl StorageDriver for SpiSdCardDriver {
    fn mount(&self) -> Result<(), StorageError> {
        let mut mounted = self.mounted.lock().unwrap();
        if *mounted {
            return Err(StorageError::AlreadyMounted);
        }

        #[cfg(target_os = "espidf")]
        {
            let board_config = crate::board::config::BoardConfig::load();
            let peripherals = esp_idf_hal::peripherals::Peripherals::take()
                .map_err(|_| StorageError::MountFailed)?;

            let spi_config = esp_idf_hal::spi::config::Config::new()
                .baudrate(400.kHz().into())
                .data_mode(MODE_0);

            let spi = peripherals.spi2;

            #[cfg(feature = "esp32s3")]
            let (sclk, sdo, sdi, cs) = (
                peripherals.pins.gpio8,
                peripherals.pins.gpio11,
                peripherals.pins.gpio18,
                peripherals.pins.gpio10,
            );

            #[cfg(not(feature = "esp32s3"))]
            let (sclk, sdo, sdi, cs) = (
                peripherals.pins.gpio18,
                peripherals.pins.gpio23,
                peripherals.pins.gpio19,
                peripherals.pins.gpio5,
            );

            log::info!(
                "Initializing SD card on SPI2 using CS{} SCLK{} MOSI{} MISO{}",
                board_config.sd_cs_pin,
                board_config.sd_sck_pin,
                board_config.sd_mosi_pin,
                board_config.sd_miso_pin
            );

            let spi_device = esp_idf_hal::spi::SpiDeviceDriver::new_single(
                spi,
                sclk,
                sdo,
                Some(sdi),
                Some(cs),
                &esp_idf_hal::spi::SpiDriverConfig::new(),
                &spi_config,
            )
            .map_err(|_| StorageError::MountFailed)?;

            let sd_card = embedded_sdmmc::SdCard::new(spi_device, esp_idf_hal::delay::Ets);
            let volume_mgr = VolumeManager::new(sd_card, SdCardClock);
            let volume = volume_mgr
                .open_volume(VolumeIdx(0))
                .map_err(|_| StorageError::MountFailed)?;
            let root_dir = volume
                .open_root_dir()
                .map_err(|_| StorageError::MountFailed)?;
            let mut file = root_dir
                .open_file_in_dir("RUSTONI.TXT", Mode::ReadWriteCreateOrAppend)
                .map_err(|_| StorageError::MountFailed)?;
            file.write_all(b"hello from rusToniESP\n")
                .map_err(|_| StorageError::IoError)?;
            file.flush().map_err(|_| StorageError::IoError)?;
        }

        #[cfg(not(target_os = "espidf"))]
        {
            std::fs::create_dir_all(&self.mount_point).map_err(|_| StorageError::MountFailed)?;
        }

        *mounted = true;
        Ok(())
    }

    fn unmount(&self) -> Result<(), StorageError> {
        let mut mounted = self.mounted.lock().unwrap();
        if !*mounted {
            return Err(StorageError::NotMounted);
        }
        *mounted = false;
        Ok(())
    }

    fn mount_point(&self) -> &str {
        &self.mount_point
    }

    fn is_mounted(&self) -> bool {
        *self.mounted.lock().unwrap()
    }
}

