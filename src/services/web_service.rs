use esp_idf_hal::gpio::{AnyIOPin, PinDriver};
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::io::EspIOError;
use std::sync::{Arc, Mutex};
use std::fs;

use crate::board::config::MOUNT_PATH;

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

            let html = format!(
                r#"<!DOCTYPE html>
                <html>
                <head>
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>ESP32 Controller</title>
                    <style>
                        body {{ font-family: sans-serif; text-align: center; margin-top: 50px; background: #1e293b; color: #f8fafc; }}
                        .btn {{ padding: 15px 35px; font-size: 1.2rem; color: white; background-color: {}; border: none; border-radius: 8px; cursor: pointer; transition: 0.2s; margin-bottom: 20px; }}
                        .btn:hover {{ opacity: 0.9; }}
                        .status {{ font-size: 1.5rem; margin-bottom: 20px; font-weight: bold; }}
                        a {{ color: #38bdf8; text-decoration: none; font-size: 1.1rem; }}
                        a:hover {{ text-decoration: underline; }}
                    </style>
                </head>
                <body>
                    <h1>ESP32 Onboard Control</h1>
                    <div class="status">LED Status: {}</div>
                    <form action="/toggle" method="POST">
                        <button type="submit" class="btn">Toggle LED</button>
                    </form>
                    <br>
                    <a href="/files">📁 View SD Card Storage (ls)</a>
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

        // 3. New GET Files (ls) Endpoint handler
        server.fn_handler("/files", esp_idf_svc::http::Method::Get, move |request| -> Result<(), EspIOError> {
            let mut file_rows = String::new();

            // Iterate over the target VFS mount point
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

            let html = format!(
                r#"<!DOCTYPE html>
                <html>
                <head>
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>SD Card Directory Index</title>
                    <style>
                        body {{ font-family: sans-serif; margin: 30px; background: #1e293b; color: #f8fafc; }}
                        table {{ width: 100%; max-width: 600px; margin: 20px auto; border-collapse: collapse; background: #0f172a; border-radius: 8px; overflow: hidden; }}
                        th, td {{ padding: 12px 15px; text-align: left; border-bottom: 1px solid #334155; }}
                        th {{ background-color: #334155; color: #38bdf8; }}
                        tr:hover {{ background-color: #1e293b; }}
                        .back-link {{ display: block; text-align: center; margin-top: 20px; color: #94a3b8; text-decoration: none; }}
                        .back-link:hover {{ color: #f8fafc; }}
                        h2 {{ text-align: center; color: #38bdf8; }}
                    </style>
                </head>
                <body>
                    <h2>Index of {}</h2>
                    <table>
                        <thead>
                            <tr>
                                <th>Name</th>
                                <th>Size / Type</th>
                            </tr>
                        </thead>
                        <tbody>
                            {}
                        </tbody>
                    </table>
                    <a href="/" class="back-link">&larr; Back to Dashboard</a>
                </body>
                </html>"#,
                MOUNT_PATH, file_rows
            );

            let mut response = request.into_ok_response()?;
            response.write(html.as_bytes())?;
            Ok(())
        })?;

        log::info!("🚀 Web server successfully started on port 80");
        Ok(Self { _server: server })
    }
}