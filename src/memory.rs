// src/memory.rs

use talc::{*, source::Claim};

#[global_allocator]
static TALC: TalcLock<spinning_top::RawSpinlock, Claim> = TalcLock::new(unsafe {
    static mut INITIAL_HEAP: [u8; min_first_heap_size::<DefaultBinning>() + 100000] =
        [0; min_first_heap_size::<DefaultBinning>() + 100000];

    Claim::array(&raw mut INITIAL_HEAP)
});

const LOW_MEM: u64 = 0x10_0000; // leave the first MiB alone
const MIN_REGION: u64 = 64 * 1024; // not worth claiming anything smaller

/// Claims all usable RAM for the heap and returns the number of bytes claimed.
///
/// `usable` yields `(physical_base, length)` for each USABLE memory map entry.
///
/// # Safety
/// Call once, early, after the HHDM is available. Every region passed in must be
/// genuinely free memory that nothing else (frame allocator, DMA, etc.) will use.
pub unsafe fn init_heap<I>(usable: I, hhdm_offset: u64) -> usize
where
    I: Iterator<Item = (u64, u64)> + Clone,
{
    let clamp = |(base, len): (u64, u64)| -> Option<(u64, u64)> {
        let end = base.checked_add(len)?;
        let start = base.max(LOW_MEM);
        (end > start && end - start >= MIN_REGION).then_some((start, end - start))
    };

    // The first claim has to fit talc's metadata, so start with the biggest region.
    let Some(largest) = usable.clone().filter_map(clamp).max_by_key(|&(_, len)| len) else {
        return 0;
    };

    let mut talc = TALC.lock();
    let mut total = 0usize;

    let mut claim = |(base, len): (u64, u64)| {
        let ptr = (base + hhdm_offset) as *mut u8;
        if unsafe { talc.claim(ptr, len as usize) }.is_some() {
            total += len as usize;
        }
    };

    claim(largest);
    for region in usable.filter_map(clamp) {
        if region != largest {
            claim(region);
        }
    }

    total
}