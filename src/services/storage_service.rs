// High-level filesystem and storage services
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub fn generate_nested_test_files(base_path: &str) -> std::io::Result<()> {
    log::info!("🛠️ Generating 20 nested files using strict 8.3 limits...");

    let dirs = [
        format!("{}/DIR_A", base_path),
        format!("{}/DIR_A/SUB_A", base_path),
        format!("{}/DIR_B", base_path),
        format!("{}/DIR_B/SUB_B", base_path),
    ];

    for dir in &dirs {
        let path = Path::new(dir);
        match fs::create_dir(path) {
            Ok(_) => log::info!("[VFS] Created directory: {}", dir),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    log::info!("[VFS] Directory already exists: {}", dir);
                } else {
                    return Err(e);
                }
            }
        }
    }

    for i in 1..=4 {
        let file_path = format!("{}/FILE{}.TXT", base_path, i);
        let mut file = File::create(&file_path)?;
        writeln!(file, "Root file index {}.", i)?;
    }
    log::info!("[VFS] Wrote 4 files to Root");

    for i in 1..=4 {
        let file_path = format!("{}/AFILE{}.TXT", dirs[0], i);
        let mut file = File::create(&file_path)?;
        writeln!(file, "DIR_A file index {}.", i)?;
    }
    log::info!("[VFS] Wrote 4 files to DIR_A");

    for i in 1..=4 {
        let file_path = format!("{}/SAFILE{}.TXT", dirs[1], i);
        let mut file = File::create(&file_path)?;
        writeln!(file, "SUB_A deep file index {}.", i)?;
    }
    log::info!("[VFS] Wrote 4 files to DIR_A/SUB_A");

    for i in 1..=4 {
        let file_path = format!("{}/BFILE{}.TXT", dirs[2], i);
        let mut file = File::create(&file_path)?;
        writeln!(file, "DIR_B file index {}.", i)?;
    }
    log::info!("[VFS] Wrote 4 files to DIR_B");

    for i in 1..=4 {
        let file_path = format!("{}/SBFILE{}.TXT", dirs[3], i);
        let mut file = File::create(&file_path)?;
        writeln!(file, "SUB_B deep file index {}.", i)?;
    }
    log::info!("[VFS] Wrote 4 files to DIR_B/SUB_B");

    log::info!("✅ Hardened 8.3 Stress-Test files ready!");
    Ok(())
}

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