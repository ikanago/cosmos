#![cfg_attr(not(test), no_std)]

use core::arch::asm;

pub mod print;

pub fn hlt() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

struct SbiRet {
    error: usize,
    value: usize,
}

fn sbi_call(
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    fid: usize,
    eid: usize,
) -> SbiRet {
    let error: usize;
    let value: usize;
    unsafe {
        asm!(
            "ecall",
            inout("a0") a0 => error,
            inout("a1") a1 => value,
            in("a2") a2,
            in("a3") a3,
            in("a4") a4,
            in("a5") a5,
            in("a6") fid,
            in("a7") eid,
        );
    }
    SbiRet { error, value }
}

pub fn memset(start: *mut u8, char: u8, len: usize) {
    let mut i = 0;
    while i < len {
        unsafe {
            *start.offset(i as isize) = char;
        }
        i += 1;
    }
}
