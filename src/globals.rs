use peak_alloc::PeakAlloc;

#[global_allocator]
pub static PEAK_ALLOC: PeakAlloc = PeakAlloc;

pub fn memory_check() -> (f32, f32) {
    let current_mem = PEAK_ALLOC.current_usage_as_mb();
    let peak_mem = PEAK_ALLOC.peak_usage_as_mb();
    (current_mem, peak_mem)
}
