use std::thread;
use esp_idf_svc::hal::gpio::AnyOutputPin;
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};
use crate::drivers::rfid::RfidDriver;

pub fn start_rfid_polling_service(
    spi_device: SpiDeviceDriver<'static, std::sync::Arc<SpiDriver<'static>>>,
    rst_pin: AnyOutputPin<'static>,
) {
    thread::spawn(move || {
        log::info!("⚡ Initializing RFID Background Service Subsystem...");
        
        let mut rfid = match RfidDriver::new(spi_device, rst_pin) {
            Ok(driver) => {
                log::info!("✅ MFRC522 Hardware Interface attached successfully.");
                driver
            }
            Err(e) => {
                log::error!("❌ Failed to initialize MFRC522 Driver: {:?}", e);
                return;
            }
        };

        log::info!("📡 RFID Reader active. Awaiting card swipe target... ");

        loop {
            // Check for card presence
            if let Some(uid) = rfid.read_card_uid() {
                log::info!("");
                log::info!("=================================================");
                log::info!("💳 [RFID TAG DETECTED]");
                log::info!("   • Serial UID Hex : {}", uid);
                log::info!("=================================================");
                log::info!("");
                
                // Extra breathing room after a successful read burst so we don't spam the console
                esp_idf_svc::hal::delay::FreeRtos::delay_ms(500);
            }
            
            // 👇 CRITICAL FIX: Explicitly yield the CPU core back to the FreeRTOS scheduler 
            // between every single polling attempt. This forces IDLE1 to satisfy the Watchdog.
            esp_idf_svc::hal::delay::FreeRtos::delay_ms(50);
        }
    });
}