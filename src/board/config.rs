use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::peripherals::Peripherals;

// Constants unchanged across targets
pub const MOUNT_PATH: &str = "/sdcard";
pub const MAX_FILE_DESCRIPTORS: usize = 5;

/// High-level board profile generated entirely at compile-time 
/// from the targeted board TOML specifications.
pub struct BoardConfig {
    pub name: &'static str,
    pub target: &'static str,
    pub mcu: &'static str,
    pub sd_cs_pin: i32,
    pub sd_sck_pin: i32,
    pub sd_mosi_pin: i32,
    pub sd_miso_pin: i32,
    pub spi_clock_hz: u32,
}

impl BoardConfig {
    /// Zero-overhead load using compile-time environment literals injection
    pub const fn load() -> Self {
        Self {
            name: env!("BOARD_NAME"),
            target: env!("BOARD_TARGET"),
            mcu: env!("BOARD_MCU"),
            sd_cs_pin: const_str_to_i32(env!("SD_CS_PIN")),
            sd_sck_pin: const_str_to_i32(env!("SD_SCK_PIN")),
            sd_mosi_pin: const_str_to_i32(env!("SD_MOSI_PIN")),
            sd_miso_pin: const_str_to_i32(env!("SD_MISO_PIN")),
            spi_clock_hz: const_str_to_u32(env!("SD_SPI_CLOCK_HZ")),
        }
    }

    /// Safely creates an AnyIOPin directly from its raw ID type wrapper
    pub fn get_any_pin(&self, pin_num: i32, _peripherals: &mut Peripherals) -> Result<AnyIOPin<'_>, Box<dyn std::error::Error>> {
        if pin_num < 0 || pin_num > 48 {
            return Err(format!("GPIO pin {} is out of range for this target MCU.", pin_num).into());
        }
        
        // Cast i32 to u8 as expected by the underlying hardware register mapping
        let pin = unsafe { AnyIOPin::steal(pin_num as u8) };
        Ok(pin)
    }
}

// Compile-time helpers to parse string literals into integers within constants
const fn const_str_to_i32(s: &str) -> i32 {
    let bytes = s.as_bytes();
    let mut val = 0;
    let mut i = 0;
    while i < bytes.len() {
        val = val * 10 + (bytes[i] - b'0') as i32;
        i += 1;
    }
    val
}

const fn const_str_to_u32(s: &str) -> u32 {
    let bytes = s.as_bytes();
    let mut val = 0;
    let mut i = 0;
    while i < bytes.len() {
        val = val * 10 + (bytes[i] - b'0') as u32;
        i += 1;
    }
    val
}