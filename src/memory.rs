// src/memory.rs

use linked_list_allocator::LockedHeap;

use limine::{memmap::{Entry, MEMMAP_USABLE}, request::{HhdmRequest, MemmapRequest}};

// Request passed to limine to get hardware memory map at boot
#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new();

// Request passed to limine to get HHDM offset at boot
#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() {
    let hhdm_offset = HHDM_REQUEST
        .response()
        .expect("Failed to get HHDM offset")
        .offset;
    
    let memory_map_response = MEMORY_MAP_REQUEST
        .response()
        .expect("Failed to get memory map")
        .entries();
    
    let (mut largest_entry_index, mut largest_entry_length): (usize, u64) = (0, 0);
    let mut found_entry = false;
    
    for (index, entry) in memory_map_response.iter().enumerate() {
        if entry.type_ == MEMMAP_USABLE && entry.length > largest_entry_length {
            largest_entry_index = index;
            largest_entry_length = entry.length;
            found_entry = true;
        }
    }

    assert!(found_entry, "Usable memory entry wasn't found");
    
    let heap_start = memory_map_response[largest_entry_index].base + hhdm_offset;
    let heap_end = heap_start + largest_entry_length;
    let heap_size = (heap_end - heap_start) as usize;
    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
}