use std::sync::atomic::{AtomicU32, Ordering};

/// Prints a comprehensive summary of memory allocation and stack safety
pub fn print_memory_summary(context_label: &str) {
    static BASELINE_STACK: AtomicU32 = AtomicU32::new(0);

    unsafe {
        use esp_idf_svc::sys as esp_idf_sys;

        let free_heap = esp_idf_sys::esp_get_free_heap_size();
        let min_free_heap = esp_idf_sys::esp_get_minimum_free_heap_size();

        let unused_stack_words = esp_idf_sys::uxTaskGetStackHighWaterMark(core::ptr::null_mut());
        let unused_stack_bytes = unused_stack_words * 4;

        let mut total_stack = BASELINE_STACK.load(Ordering::Relaxed);
        if total_stack == 0 {
            total_stack = unused_stack_bytes;
            BASELINE_STACK.store(total_stack, Ordering::Relaxed);
        }

        let main_task_used_bytes = total_stack.saturating_sub(unused_stack_bytes);
        let main_task_used_pct = if total_stack > 0 {
            (main_task_used_bytes as f32 / total_stack as f32) * 100.0
        } else {
            0.0
        };

        log::info!("");
        log::info!("================== MEMORY STATUS: {} ==================", context_label);
        log::info!("  [SYSTEM HEAP]");
        log::info!("    • Current Free Heap : {:<8} bytes ({:.2} KB)", free_heap, free_heap as f32 / 1024.0);
        log::info!("    • Lowest Heap Ever  : {:<8} bytes ({:.2} KB)", min_free_heap, min_free_heap as f32 / 1024.0);
        log::info!("  [TASK STACKS]");
        log::info!("    • Task Name         : main");
        log::info!("    • Estimated Stack   : {:<8} bytes (~{:.1} KB)", total_stack, total_stack as f32 / 1024.0);
        log::info!("    • Unused Stack Room : {:<8} bytes", unused_stack_bytes);
        log::info!("    • Used Stack (Peak) : {:<8} bytes ({:.1}% utilized)", main_task_used_bytes, main_task_used_pct);
        
        if unused_stack_bytes < 2048 {
            log::warn!("  ⚠️ WARNING: 'main' task is dangerously close to a stack overflow! (< 2KB left)");
        } else {
            log::info!("  ✅ 'main' task stack is healthy ({} bytes safety margin).", unused_stack_bytes);
        }
        log::info!("=======================================================================");
        log::info!("");
    }
}