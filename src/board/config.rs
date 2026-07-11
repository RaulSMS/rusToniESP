#[derive(Debug, Clone)]
pub struct BoardConfig {
    pub name: &'static str,
    pub target: &'static str,
    pub mcu: &'static str,
    pub sd_cs_pin: u32,
    pub sd_sck_pin: u32,
    pub sd_miso_pin: u32,
    pub sd_mosi_pin: u32,
    pub sd_spi_clock_hz: u32,
}

impl BoardConfig {
    pub fn load() -> Self {
        #[cfg(feature = "esp32s3")]
        {
            return Self {
                name: "esp32s3",
                target: "xtensa-esp32s3-espidf",
                mcu: "esp32s3",
                sd_cs_pin: 10,
                sd_sck_pin: 8,
                sd_miso_pin: 18,
                sd_mosi_pin: 11,
                sd_spi_clock_hz: 400_000,
            };
        }

        Self {
            name: "esp32",
            target: "xtensa-esp32-espidf",
            mcu: "esp32",
            sd_cs_pin: 5,
            sd_sck_pin: 18,
            sd_miso_pin: 19,
            sd_mosi_pin: 23,
            sd_spi_clock_hz: 400_000,
        }
    }
}
