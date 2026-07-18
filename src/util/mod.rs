use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

/// Internal low-footprint payload containing clean raw metrics fields
struct RawHardwareMetrics {
    free_heap: u32,
    lowest_heap: u32,
    internal_free: u32,
    internal_total: u32, // Added to compute percentage
    #[allow(dead_code)]
    psram_free: u32, // It is uesd only in the advanced.html so rust complains 
    task_unused_bytes: u32,
    cpu_cores: u32,
    task_count: u32,     // Added FreeRTOS total active tasks
    temperature: f32,
}

impl RawHardwareMetrics {
    fn capture() -> Self {
        unsafe {
            use esp_idf_svc::sys as esp_idf_sys;

            let free_heap = esp_idf_sys::esp_get_free_heap_size();
            let lowest_heap = esp_idf_sys::esp_get_minimum_free_heap_size();
            
            let internal_free = esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_INTERNAL) as u32;
            let internal_total = esp_idf_sys::heap_caps_get_total_size(esp_idf_sys::MALLOC_CAP_INTERNAL) as u32;
            let psram_free = esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_SPIRAM) as u32;
            
            let task_unused_bytes = esp_idf_sys::uxTaskGetStackHighWaterMark(core::ptr::null_mut()) * 4;
            let task_count = esp_idf_sys::uxTaskGetNumberOfTasks() as u32;

            let cpu_cores = if esp_idf_sys::portNUM_PROCESSORS > 1 { 2 } else { 1 };
            let temperature: f32 = 0.0;

            Self {
                free_heap,
                lowest_heap,
                internal_free,
                internal_total,
                psram_free,
                task_unused_bytes,
                cpu_cores,
                task_count,
                temperature,
            }
        }
    }
}

/// Recursively deletes a directory tree from the bottom up.
pub fn native_recursive_delete<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let child_path = entry.path();
            if child_path.is_dir() {
                native_recursive_delete(&child_path)?;
            } else {
                fs::remove_file(&child_path)?;
            }
        }
        fs::remove_dir(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Runs a transient nested read/write/delete verification on the SD Card
pub fn run_sd_card_init_test(base_path: &str) -> std::io::Result<()> {
    log::info!("🧪 [SD Init Test] Starting transient filesystem verification...");

    let test_root = format!("{}/INIT_TST", base_path);
    let nested_dir = format!("{}/NEST_DIR", test_root);
    let file_path = format!("{}/TEST_TXT.TXT", nested_dir);

    let test_root_path = Path::new(&test_root);
    let nested_dir_path = Path::new(&nested_dir);

    if test_root_path.exists() {
        let _ = native_recursive_delete(test_root_path);
    }

    fs::create_dir(test_root_path)?;
    fs::create_dir(nested_dir_path)?;
    log::info!("   └─ Created nested tree structure successfully.");

    let test_payload = b"ESP32-Rust-VFS-Verification-String";
    {
        let mut file = File::create(&file_path)?;
        file.write_all(test_payload)?;
        file.flush()?;
    }
    log::info!("   └─ Wrote verification data file.");

    let metadata = fs::metadata(&file_path)?;
    log::info!("   └─ Verified file size allocation: {} bytes.", metadata.len());
    
    if metadata.len() != test_payload.len() as u64 {
        log::error!("❌ [SD Init Test] Critical validation error: File size mismatch!");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "VFS payload length corrupted"));
    }

    log::info!("   └─ Erasing test artifacts dynamically...");
    native_recursive_delete(test_root_path)?;

    if test_root_path.exists() {
        log::error!("❌ [SD Init Test] Validation failure: Test folder structure still exists on disk!");
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Failed to delete transient test directory completely",
        ));
    }
    log::info!("   └─ Confirmed: Test folder structure completely removed.");
    Ok(())
}

/// Prints a comprehensive summary of memory allocation and stack safety
pub fn print_memory_summary(context_label: &str) {
    static BASELINE_STACK: AtomicU32 = AtomicU32::new(0);

    let metrics = RawHardwareMetrics::capture();

    let mut total_stack = BASELINE_STACK.load(Ordering::Relaxed);
    if total_stack == 0 {
        total_stack = metrics.task_unused_bytes;
        BASELINE_STACK.store(total_stack, Ordering::Relaxed);
    }

    let main_task_used_bytes = total_stack.saturating_sub(metrics.task_unused_bytes);
    let main_task_used_pct = if total_stack > 0 {
        (main_task_used_bytes as f32 / total_stack as f32) * 100.0
    } else {
        0.0
    };

    log::info!("");
    log::info!("================== MEMORY STATUS: {} ==================", context_label);
    log::info!("  [SYSTEM HEAP]");
    log::info!("    • Current Free Heap : {:<8} bytes ({:.2} KB)", metrics.free_heap, metrics.free_heap as f32 / 1024.0);
    log::info!("    • Lowest Heap Ever  : {:<8} bytes ({:.2} KB)", metrics.lowest_heap, metrics.lowest_heap as f32 / 1024.0);
    log::info!("  [TASK STACKS]");
    log::info!("    • Task Name         : main");
    log::info!("    • Estimated Stack   : {:<8} bytes (~{:.1} KB)", total_stack, total_stack as f32 / 1024.0);
    log::info!("    • Unused Stack Room : {:<8} bytes", metrics.task_unused_bytes);
    log::info!("    • Used Stack (Peak) : {:<8} bytes ({:.1}% utilized)", main_task_used_bytes, main_task_used_pct);
    
    if metrics.task_unused_bytes < 2048 {
        log::warn!("  ⚠️ WARNING: 'main' task is dangerously close to a stack overflow! (< 2KB left)");
    } else {
        log::info!("  ✅ 'main' task stack is healthy ({} bytes safety margin).", metrics.task_unused_bytes);
    }
    log::info!("=======================================================================");
    log::info!("");
}

/// Compiles advanced hardware telemetry statistics directly into a flat JSON format
pub fn get_hardware_metrics_json() -> String {
    let metrics = RawHardwareMetrics::capture();

    format!(
        "{{\
            \"free_heap\":{},\
            \"lowest_heap\":{},\
            \"internal_free\":{},\
            \"internal_total\":{},\
            \"http_stack_margin\":{},\
            \"cores\":{},\
            \"task_count\":{},\
            \"temperature\":{:.1}\
        }}",
        metrics.free_heap, metrics.lowest_heap, metrics.internal_free, 
        metrics.internal_total, metrics.task_unused_bytes, metrics.cpu_cores, 
        metrics.task_count, metrics.temperature
    )
}