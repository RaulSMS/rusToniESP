pub mod utils;
pub mod handlers;

use esp_idf_hal::gpio::{AnyIOPin, PinDriver};
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::io::EspIOError;
use std::sync::{Arc, Mutex};

static INDEX_TEMPLATE: &str = include_str!("web_assets/index.html");

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

        // 1. GET Root Dashboard UI
        let led_root = shared_led.clone();
        server.fn_handler("/", esp_idf_svc::http::Method::Get, move |request| -> Result<(), EspIOError> {
            let is_high = {
                let led_lock = led_root.lock().unwrap();
                led_lock.is_set_high()
            };

            let status_text = if is_high { "ON" } else { "OFF" };
            let btn_color = if is_high { "#ef4444" } else { "#22c55e" };

            let html = INDEX_TEMPLATE
                .replace("{0}", btn_color)
                .replace("{1}", status_text);

            let mut response = request.into_ok_response()?;
            response.write(html.as_bytes())?;
            Ok(())
        })?;

        // 2. POST Toggle Onboard LED
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

        // 3. GET Interactive File Explorer
        server.fn_handler("/files", esp_idf_svc::http::Method::Get, move |mut request| -> Result<(), EspIOError> {
            handlers::handle_get_files(request.connection()).map_err(|e| EspIOError::from(e))?;
            Ok(())
        })?;

        // 4. GET Download Endpoint
        server.fn_handler("/download", esp_idf_svc::http::Method::Get, move |mut request| -> Result<(), EspIOError> {
            handlers::handle_download(request.connection()).map_err(|e| EspIOError::from(e))?;
            Ok(())
        })?;

        // 5. POST Multi-chunk File Upload
        server.fn_handler("/upload", esp_idf_svc::http::Method::Post, move |mut request| -> Result<(), EspIOError> {
            handlers::handle_upload(request.connection()).map_err(|e| EspIOError::from(e))?;
            Ok(())
        })?;

        // 6. DELETE Storage Asset Endpoint
        server.fn_handler("/delete", esp_idf_svc::http::Method::Delete, move |mut request| -> Result<(), EspIOError> {
            handlers::handle_delete(request.connection()).map_err(|e| EspIOError::from(e))?;
            Ok(())
        })?;

        log::info!("🚀 Modular Web server successfully started on port 80");
        Ok(Self { _server: server })
    }
}