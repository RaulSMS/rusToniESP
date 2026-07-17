use esp_idf_hal::gpio::{AnyIOPin, PinDriver};
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::io::EspIOError;
use std::sync::{Arc, Mutex};

/// Holds the HTTP server context and shared thread-safe access to the LED pin
pub struct WebServerContext {
    _server: EspHttpServer<'static>,
}

impl WebServerContext {
    /// Accepts a static hardware pin lifecycle context
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

            let html = format!(
                r#"<!DOCTYPE html>
                <html>
                <head>
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>ESP32 Controller</title>
                    <style>
                        body {{ font-family: sans-serif; text-align: center; margin-top: 50px; background: #1e293b; color: #f8fafc; }}
                        .btn {{ padding: 15px 35px; font-size: 1.2rem; color: white; background-color: {}; border: none; border-radius: 8px; cursor: pointer; transition: 0.2s; }}
                        .btn:hover {{ opacity: 0.9; }}
                        .status {{ font-size: 1.5rem; margin-bottom: 20px; font-weight: bold; }}
                    </style>
                </head>
                <body>
                    <h1>ESP32 Onboard Control</h1>
                    <div class="status">LED Status: {}</div>
                    <form action="/toggle" method="POST">
                        <button type="submit" class="btn">Toggle LED</button>
                    </form>
                </body>
                </html>"#,
                btn_color, status_text
            );

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

        log::info!("🚀 Web server successfully started on port 80");
        Ok(Self { _server: server })
    }
}