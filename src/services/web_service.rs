use esp_idf_hal::gpio::{AnyIOPin, PinDriver};
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::io::EspIOError;
use std::sync::{Arc, Mutex};
use std::fs::{self, File};
use std::io::Write;

use crate::board::config::MOUNT_PATH;

// Static compile-time inclusion of asset templates
static INDEX_TEMPLATE: &str = include_str!("web_assets/index.html");
static FILES_TEMPLATE: &str = include_str!("web_assets/files.html");

pub struct WebServerContext {
    _server: EspHttpServer<'static>,
}

impl WebServerContext {
    pub fn init(led_pin: AnyIOPin<'static>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut led = PinDriver::output(led_pin)?;
        led.set_low()?;

        let shared_led = Arc::new(Mutex::new(led));
        let config = Configuration::default();
        let mut server = EspHttpServer::new(&config)?;

        // 1. Root UI Endpoint handler
        let led_root = shared_led.clone();
        server.fn_handler("/", esp_idf_svc::http::Method::Get, move |request| -> Result<(), EspIOError> {
            let is_high = {
                let led_lock = led_root.lock().unwrap();
                led_lock.is_set_high()
            };

            let status_text = if is_high { "ON" } else { "OFF" };
            let btn_color = if is_high { "#ef4444" } else { "#22c55e" };

            // Inject template arguments using runtime replacement string operations
            let html = INDEX_TEMPLATE
                .replace("{0}", btn_color)
                .replace("{1}", status_text);

            let mut response = request.into_ok_response()?;
            response.write(html.as_bytes())?;
            Ok(())
        })?;

        // 2. POST Toggle Endpoint handler
        let led_toggle = shared_led.clone();
        server.fn_handler("/toggle", esp_idf_svc::http::Method::Post, move |request| -> Result<(), EspIOError> {
            {
                let mut led_lock = led_toggle.lock().unwrap();
                if led_lock.is_set_high() {
                    led_lock.set_low().unwrap();
                } else {
                    led_lock.set_high().unwrap();
                }
            }

            let mut response = request.into_response(303, Some("See Other"), &[("Location", "/")])?;
            response.write(&[])?;
            Ok(())
        })?;

        // 3. GET Files (ls) + Upload UI Endpoint handler
        server.fn_handler("/files", esp_idf_svc::http::Method::Get, move |request| -> Result<(), EspIOError> {
            let mut file_rows = String::new();

            match fs::read_dir(MOUNT_PATH) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let file_name = entry.file_name().to_string_lossy().into_owned();
                        let metadata = entry.metadata();
                        
                        let (is_dir, file_size) = match metadata {
                            Ok(meta) => (meta.is_dir(), meta.len()),
                            Err(_) => (false, 0),
                        };

                        let icon = if is_dir { "📁" } else { "📄" };
                        let details = if is_dir { 
                            "&lt;DIR&gt;".to_string() 
                        } else { 
                            format!("{} bytes", file_size) 
                        };

                        file_rows.push_str(&format!(
                            "<tr><td>{} {}</td><td>{}</td></tr>",
                            icon, file_name, details
                        ));
                    }
                }
                Err(e) => {
                    file_rows.push_str(&format!(
                        "<tr><td colspan='2' style='color:#ef4444;'>Error reading directory: {:?}</td></tr>",
                        e
                    ));
                }
            }

            // Inject template arguments using runtime replacement string operations
            let html = FILES_TEMPLATE
                .replace("{0}", MOUNT_PATH)
                .replace("{1}", &file_rows);

            let mut response = request.into_ok_response()?;
            response.write(html.as_bytes())?;
            Ok(())
        })?;

        // 4. Raw Binary POST Upload Endpoint handler
        server.fn_handler("/upload", esp_idf_svc::http::Method::Post, move |mut request| -> Result<(), EspIOError> {
            let file_name = request
                .header("X-File-Name")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "uploaded_file.bin".to_string());

            let full_path = format!("{}/{}", MOUNT_PATH, file_name);
            log::info!("💾 Streaming incoming upload directly to: {}", full_path);

            match File::create(&full_path) {
                Ok(mut file) => {
                    let mut buffer = [0u8; 512];
                    let mut total_bytes = 0;

                    loop {
                        let bytes_read = request.read(&mut buffer)?;
                        if bytes_read == 0 {
                            break; 
                        }
                        if let Err(e) = file.write_all(&buffer[..bytes_read]) {
                            log::error!("❌ Failed writing chunk to SD card: {:?}", e);
                            let mut response = request.into_status_response(500)?;
                            response.write(b"Disk write error")?;
                            return Ok(());
                        }
                        total_bytes += bytes_read;
                    }

                    log::info!("✅ File write complete! Saved {} bytes.", total_bytes);
                    let mut response = request.into_ok_response()?;
                    response.write(b"Upload completed successfully")?;
                }
                Err(e) => {
                    log::error!("❌ Failed to create file template: {:?}", e);
                    let mut response = request.into_status_response(500)?;
                    response.write(b"Failed to create file target")?;
                }
            }

            Ok(())
        })?;

        log::info!("🚀 Web server successfully started on port 80");
        Ok(Self { _server: server })
    }
}