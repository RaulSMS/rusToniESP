use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::peripherals::Peripherals;

pub const MOUNT_PATH: &str = "/sdcard";
pub const MAX_FILE_DESCRIPTORS: usize = 5;

pub struct BoardConfig {
    pub name: &'static str,
    pub target: &'static str,
    pub mcu: &'static str,
    
    // Unified SPI Bus Pins
    pub spi_sck_pin: i32,
    pub spi_mosi_pin: i32,
    pub spi_miso_pin: i32,
    
    // SD Card Configuration
    pub sd_cs_pin: i32,
    pub sd_spi_clock_hz: u32,
    
    // RFID Configuration
    pub rfid_cs_pin: i32,
    pub rfid_rst_pin: i32,
    pub rfid_spi_clock_hz: u32,
}

impl BoardConfig {
    pub const fn load() -> Self {
        Self {
            name: env!("BOARD_NAME"),
            target: env!("BOARD_TARGET"),
            mcu: env!("BOARD_MCU"),
            
            // Shared Bus
            spi_sck_pin: const_str_to_i32(env!("SPI_SCK_PIN")),
            spi_mosi_pin: const_str_to_i32(env!("SPI_MOSI_PIN")),
            spi_miso_pin: const_str_to_i32(env!("SPI_MISO_PIN")),
            
            // SD Client
            sd_cs_pin: const_str_to_i32(env!("SD_CS_PIN")),
            sd_spi_clock_hz: const_str_to_u32(env!("SD_SPI_CLOCK_HZ")),
            
            // RFID Client
            rfid_cs_pin: const_str_to_i32(env!("RFID_CS_PIN")),
            rfid_rst_pin: const_str_to_i32(env!("RFID_RST_PIN")),
            rfid_spi_clock_hz: const_str_to_u32(env!("RFID_SPI_CLOCK_HZ")),
        }
    }

    pub fn get_any_pin(&self, pin_num: i32, _peripherals: &mut Peripherals) -> Result<AnyIOPin<'_>, Box<dyn std::error::Error>> {
        if pin_num < 0 || pin_num > 48 {
            return Err(format!("GPIO pin {} is out of range for this target MCU.", pin_num).into());
        }
        let pin = unsafe { AnyIOPin::steal(pin_num as u8) };
        Ok(pin)
    }
}

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