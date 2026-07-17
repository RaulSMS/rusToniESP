use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use esp_idf_hal::sys::EspError;
use esp_idf_svc::http::server::EspHttpConnection;
use crate::board::config::MOUNT_PATH;
use super::utils::{extract_query_param, sanitize_fat_filename};

static FILES_TEMPLATE: &str = include_str!("../web_assets/files.html");

/// Helper to safely convert an optional OS error code into a concrete EspError
fn map_io_err(e: std::io::Error) -> EspError {
    let code = e.raw_os_error().unwrap_or(-1);
    // Ensure code is non-zero so NonZeroI32::new doesn't return None
    let non_zero_code = core::num::NonZeroI32::new(if code == 0 { -1 } else { code }).unwrap();
    EspError::from_non_zero(non_zero_code)
}

pub fn handle_get_files(connection: &mut EspHttpConnection) -> Result<(), EspError> {
    let uri = connection.uri();
    let target_path = extract_query_param(&uri, "path").unwrap_or_else(|| MOUNT_PATH.to_string());

    let mut file_rows = String::new();

    let nav_html = if target_path != MOUNT_PATH {
        let path_obj = Path::new(&target_path);
        let parent_str = path_obj.parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| MOUNT_PATH.to_string());
        format!("<a href=\"/files?path={}\">&larr; Up to Parent Directory</a>", parent_str)
    } else {
        "<span>Main Storage Root</span>".to_string()
    };

    match fs::read_dir(&target_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().into_owned();
                let full_item_path = if target_path.ends_with('/') {
                    format!("{}{}", target_path, file_name)
                } else {
                    format!("{}/{}", target_path, file_name)
                };

                let metadata = entry.metadata();
                let (is_dir, file_size) = match metadata {
                    Ok(meta) => (meta.is_dir(), meta.len()),
                    Err(_) => (false, 0),
                };

                if is_dir {
                    file_rows.push_str(&format!(
                        "<tr>
                            <td>📁 <a href=\"/files?path={}\">{}</a></td>
                            <td>&lt;DIR&gt;</td>
                            <td><button class=\"btn btn-del\" onclick=\"deleteItem('{}')\">Delete</button></td>
                        </tr>",
                        full_item_path, file_name, full_item_path
                    ));
                } else {
                    file_rows.push_str(&format!(
                        "<tr>
                            <td>📄 {}</td>
                            <td>{} bytes</td>
                            <td>
                                <a href=\"/download?path={}\" class=\"btn-down\">Download</a>
                                <button class=\"btn btn-del\" onclick=\"deleteItem('{}')\">Delete</button>
                            </td>
                        </tr>",
                        file_name, file_size, full_item_path, full_item_path
                    ));
                }
            }
        }
        Err(e) => {
            file_rows.push_str(&format!(
                "<tr><td colspan='3' style='color:#ef4444;'>Error opening directory: {:?}</td></tr>", e
            ));
        }
    }

    let html = FILES_TEMPLATE
        .replace("{0}", &target_path)
        .replace("{1}", &file_rows)
        .replace("{2}", &nav_html);

    connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/html")])?;
    connection.write(html.as_bytes())?;
    Ok(())
}

pub fn handle_download(connection: &mut EspHttpConnection) -> Result<(), EspError> {
    let uri = connection.uri();
    let target_file_path = match extract_query_param(&uri, "path") {
        Some(p) => p,
        None => {
            connection.initiate_response(400, Some("Bad Request"), &[])?;
            connection.write(b"Missing file path parameter")?;
            return Ok(());
        }
    };

    let path_obj = Path::new(&target_file_path);
    let filename = path_obj.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file.bin".to_string());

    match File::open(&target_file_path) {
        Ok(mut file) => {
            let disposition_header = format!("attachment; filename=\"{}\"", filename);
            connection.initiate_response(
                200,
                Some("OK"),
                &[
                    ("Content-Type", "application/octet-stream"),
                    ("Content-Disposition", &disposition_header),
                ],
            )?;

            let mut chunk_buffer = [0u8; 512];
            loop {
                let read_bytes = file.read(&mut chunk_buffer).map_err(map_io_err)?;
                if read_bytes == 0 { break; }
                connection.write(&chunk_buffer[..read_bytes])?;
            }
        }
        Err(e) => {
            log::error!("❌ File issue: {:?}", e);
            connection.initiate_response(404, Some("Not Found"), &[])?;
            connection.write(b"File not found on storage stack")?;
        }
    }
    Ok(())
}

pub fn handle_upload(connection: &mut EspHttpConnection) -> Result<(), EspError> {
    let uri = connection.uri();
    let active_dir = extract_query_param(&uri, "path").unwrap_or_else(|| MOUNT_PATH.to_string());

    let mut raw_file_name = "up_file.bin".to_string();
    if let Some(header_val) = connection.header("X-File-Name") {
        raw_file_name = header_val.to_string();
    }

    let safe_name = sanitize_fat_filename(&raw_file_name);
    let full_path = if active_dir.ends_with('/') {
        format!("{}{}", active_dir, safe_name)
    } else {
        format!("{}/{}", active_dir, safe_name)
    };

    log::info!("💾 Streaming incoming upload directly to: {}", full_path);

    match File::create(&full_path) {
        Ok(mut file) => {
            let mut buffer = [0u8; 512];
            let mut total_bytes = 0;
            loop {
                let bytes_read = connection.read(&mut buffer)?;
                if bytes_read == 0 { break; }
                
                file.write_all(&buffer[..bytes_read]).map_err(map_io_err)?;
                total_bytes += bytes_read;
            }
            log::info!("✅ File write complete! Saved {} bytes.", total_bytes);
            connection.initiate_response(200, Some("OK"), &[])?;
            connection.write(b"Upload completed successfully")?;
        }
        Err(e) => {
            log::error!("❌ Failed to create file: {:?}", e);
            connection.initiate_response(500, Some("Internal Error"), &[])?;
            connection.write(b"Failed to create file target")?;
        }
    }
    Ok(())
}

pub fn handle_delete(connection: &mut EspHttpConnection) -> Result<(), EspError> {
    let uri = connection.uri();
    let target_to_delete = match extract_query_param(&uri, "path") {
        Some(p) => p,
        None => {
            connection.initiate_response(400, Some("Bad Request"), &[])?;
            connection.write(b"Missing targeted deletion path")?;
            return Ok(());
        }
    };

    let path_obj = Path::new(&target_to_delete);
    if !path_obj.exists() {
        connection.initiate_response(404, Some("Not Found"), &[])?;
        connection.write(b"Target asset does not exist")?;
        return Ok(());
    }

    let result = if path_obj.is_dir() {
        log::info!("🗑️ Emptying and removing directory: {}", target_to_delete);
        
        // 1. Manually clear files inside the directory first to ensure it's empty
        // (ESP-IDF's FAT VFS requires a directory to be completely empty before removal)
        if let Ok(entries) = fs::read_dir(path_obj) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let _ = fs::remove_file(p);
                }
            }
        }
        
        // 2. Use standard remove_dir instead of remove_dir_all
        fs::remove_dir(path_obj)
    } else {
        log::info!("🗑️ Removing file: {}", target_to_delete);
        fs::remove_file(path_obj)
    };

    match result {
        Ok(_) => {
            log::info!("✅ Successfully deleted asset node.");
            connection.initiate_response(200, Some("OK"), &[])?;
            connection.write(b"Deleted")?;
        }
        Err(e) => {
            log::error!("❌ Failed deleting asset: {:?}", e);
            
            // Helpful warning for macOS meta-directories
            if target_to_delete.contains("SPOTLI") || target_to_delete.contains("TRASHE") {
                connection.initiate_response(403, Some("Forbidden"), &[])?;
                connection.write(b"Cannot delete system-protected index directories")?;
            } else {
                connection.initiate_response(500, Some("Internal Server Error"), &[])?;
                connection.write(format!("Deletion failure: {:?}", e).as_bytes())?;
            }
        }
    }
    Ok(())
}