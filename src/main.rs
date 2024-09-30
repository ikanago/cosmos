#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::arch::asm;

use cosmos::{hlt, memset, println, trap::{handle_trap, kernel_entry}};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("panic: {}", info);
    hlt();
}
extern "C" {
    static __bss_start: u8;
    static __bss_end: u8;
    static __stack_top: u8;
}

#[link_section = ".text.boot"]
#[no_mangle]
pub unsafe extern "C" fn boot() -> ! {
    asm!(
        "mv sp, {stack_top}",
        "j {kernel_main}",
        stack_top = in(reg) &__stack_top,
        kernel_main = sym kernel_main,
    );
    loop {}
}

fn kernel_main() -> ! {
    let bss_size = unsafe { &__bss_end as *const _ as usize - &__bss_start as *const _ as usize };
    memset(unsafe { &__bss_start as *const _ as *mut _ }, 0, bss_size);
    unsafe {
        asm!("csrw stvec, {}", in(reg) kernel_entry);
    }
    println!("Hello, Cosmos!");

    unsafe {
        asm!("unimp");
    }
    hlt();
}
