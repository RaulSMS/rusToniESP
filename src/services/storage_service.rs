// High-level filesystem and storage services
use std::fs;
use std::path::Path;

pub fn list_dir(dir_path: &Path, start_depth: usize) {
    let mut stack = vec![(dir_path.to_path_buf(), start_depth)];

    while let Some((current_path, depth)) = stack.pop() {
        let indent = "  ".repeat(depth);

        if let Ok(entries) = fs::read_dir(&current_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let file_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();

                    if path.is_dir() {
                        log::info!("{}{}/", indent, file_name);
                        stack.push((path, depth + 1));
                    } else {
                        let file_size = entry
                            .metadata()
                            .map(|m| m.len())
                            .unwrap_or(0);
                        
                        log::info!("{}{:<25} SIZE: {} bytes", indent, file_name, file_size);
                    }
                }
            }
        } else {
            log::error!("{}❌ Failed to open directory: {:?}", indent, current_path);
        }
    }
}