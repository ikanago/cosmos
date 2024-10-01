use core::{alloc::GlobalAlloc, cell::UnsafeCell};

use crate::memset;

extern "C" {
    static __free_ram_start: u8;
    static __free_ram_end: u8;
}

struct BumpAllocator {
    head: UnsafeCell<*const u8>,
    end: *const u8,
}

#[global_allocator]
static HEAP: BumpAllocator = BumpAllocator {
    head: UnsafeCell::new(unsafe { &__free_ram_start } as *const _),
    end: unsafe { &__free_ram_end } as *const _,
};

unsafe impl Sync for BumpAllocator {}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let head = self.head.get();
        let res = *head as usize % align;
        let start = if res == 0 {
            *head
        } else {
            *head.add(align - res)
        };
        let next_head = start.add(size);

        if next_head > self.end {
            core::ptr::null_mut()
        } else {
            *head = next_head as *const u8;
            memset(*head as *mut u8, 0, size);
            start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        // Do nothing
    }
}
