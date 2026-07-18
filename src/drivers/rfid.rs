use esp_idf_svc::hal::gpio::{Output, PinDriver, AnyOutputPin};
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_svc::sys::EspError;
use mfrc522::{Mfrc522, comm::blocking::spi::SpiInterface};

// Define a type alias for the exact closure type expected by the mfrc522 crate's interface
type RfidDelayClosure = fn();

pub struct RfidDriver<'a> {
    // The delay generic parameter here is the function pointer type matching our FreeRTOS delay wrapper
    mfrc522: Mfrc522<SpiInterface<SpiDeviceDriver<'a, std::sync::Arc<SpiDriver<'a>>>, RfidDelayClosure>, mfrc522::Initialized>,
    _rst: PinDriver<'a, Output>,
}

impl<'a> RfidDriver<'a> {
    pub fn new(
        spi_device: SpiDeviceDriver<'a, std::sync::Arc<SpiDriver<'a>>>,
        rst_pin: AnyOutputPin<'a>,
    ) -> Result<Self, EspError> {
        let mut rst = PinDriver::output(rst_pin)?;
        
        // Hard-reset the MFRC522 chip at boot
        rst.set_high()?;
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(50);
        rst.set_low()?;
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(50);
        rst.set_high()?;
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(50);

        // 1. Initialize the baseline SPI interface with default dummy metrics
        let spi_interface_base = SpiInterface::new(spi_device);
        
        // 2. Attach the required FnMut() signature delay block using microsecond timing.
        let delay_closure: RfidDelayClosure = || {
            esp_idf_svc::hal::delay::Ets::delay_us(10);
        };
        let spi_interface = spi_interface_base.with_delay(delay_closure);

        // 3. Initialize driver state machines
        let mut mfrc522 = Mfrc522::new(spi_interface)
            .init()
            .map_err(|_| EspError::from_infallible::<-1>())?;

        match mfrc522.version() {
            Ok(ver) => log::info!("📡 MFRC522 Hardware Version Reg: 0x{:02X}", ver),
            Err(e) => log::error!("❌ Failed to read MFRC522 version: {:?}", e),
        }

        Ok(Self {
            mfrc522,
            _rst: rst,
        })
    }

    /// Polls the reader for a card presence. Returns the UID string if successful.
    pub fn read_card_uid(&mut self) -> Option<String> {
        match self.mfrc522.reqa() {
            Ok(atqa) => {
                log::info!("✅ reqa() succeeded");
                match self.mfrc522.select(&atqa) {
                    Ok(uid) => {
                        let uid_bytes = uid.as_bytes();
                        let hex_string: String = uid_bytes
                            .iter()
                            .map(|b| format!("{:02X}", b))
                            .collect();
                        let _ = self.mfrc522.hlta();
                        return Some(hex_string);
                    }
                    Err(e) => {
                        log::error!("❌ select() failed: {:?}", e);
                    }
                }
            }
            Err(e) => {
                if !matches!(e, mfrc522::Error::Timeout) {
                    log::error!("❌ reqa() error: {:?}", e);
                }
            }
        }
        None
    }
}