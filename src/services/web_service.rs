pub mod utils;
pub mod handlers;

use esp_idf_hal::gpio::{AnyIOPin, PinDriver};
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::io::EspIOError;
use std::sync::{Arc, Mutex};

static INDEX_TEMPLATE: &str = include_str!("web_assets/index.html");
static ADVANCED_TEMPLATE: &str = include_str!("web_assets/advanced.html");

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

        // 1. GET Root Dashboard UI Layout
        server.fn_handler("/", esp_idf_svc::http::Method::Get, move |request| -> Result<(), EspIOError> {
            let mut response = request.into_ok_response()?;
            response.write(INDEX_TEMPLATE.as_bytes())?;
            Ok(())
        })?;

        // 2. GET Advanced Functional Diagnostic Tools Panel
        let led_advanced = shared_led.clone();
        server.fn_handler("/advanced", esp_idf_svc::http::Method::Get, move |request| -> Result<(), EspIOError> {
            let is_high = {
                let led_lock = led_advanced.lock().unwrap();
                led_lock.is_set_high()
            };

            let status_text = if is_high { "ON" } else { "OFF" };
            let btn_color = if is_high { "#ef4444" } else { "#22c55e" };

            let html = ADVANCED_TEMPLATE
                .replace("{0}", btn_color)
                .replace("{1}", status_text);

            let mut response = request.into_ok_response()?;
            response.write(html.as_bytes())?;
            Ok(())
        })?;

        // 3. GET Live Memory Status JSON Endpoint
        server.fn_handler("/api/memory", esp_idf_svc::http::Method::Get, move |request| -> Result<(), EspIOError> {
            // Calls the unified engine; updates automatically without code drift!
            let json_payload = crate::util::get_hardware_metrics_json();

            let mut response = request.into_response(
                200, 
                Some("OK"), 
                &[("Content-Type", "application/json"), ("Cache-Control", "no-cache")]
            )?;
            response.write(json_payload.as_bytes())?;
            Ok(())
        })?;

        // 4. POST Toggle Onboard LED (Redirects back to Advanced Tools view)
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
            let mut response = request.into_response(303, Some("See Other"), &[("Location", "/advanced")])?;
            response.write(&[])?;
            Ok(())
        })?;

        // 5. GET Interactive File Explorer (Streaming execution layout)
        server.fn_handler("/files", esp_idf_svc::http::Method::Get, move |mut request| -> Result<(), EspIOError> {
            handlers::handle_get_files(request.connection()).map_err(|e| EspIOError::from(e))?;
            Ok(())
        })?;

        // 6. GET Download Endpoint
        server.fn_handler("/download", esp_idf_svc::http::Method::Get, move |mut request| -> Result<(), EspIOError> {
            handlers::handle_download(request.connection()).map_err(|e| EspIOError::from(e))?;
            Ok(())
        })?;

        // 7. POST Multi-chunk File Upload
        server.fn_handler("/upload", esp_idf_svc::http::Method::Post, move |mut request| -> Result<(), EspIOError> {
            handlers::handle_upload(request.connection()).map_err(|e| EspIOError::from(e))?;
            Ok(())
        })?;

        // 8. DELETE Storage Asset Endpoint
        server.fn_handler("/delete", esp_idf_svc::http::Method::Delete, move |mut request| -> Result<(), EspIOError> {
            handlers::handle_delete(request.connection()).map_err(|e| EspIOError::from(e))?;
            Ok(())
        })?;

        log::info!("🚀 Modular Web server successfully started on port 80");
        Ok(Self { _server: server })
    }
}