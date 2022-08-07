use core::arch::asm;

#[inline]
pub fn hlt() {
    unsafe { asm!("hlt") }
}

// Write data to 16 bit address `port`.
#[inline]
pub fn out32(port: u16, data: u32) {
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") data,
            options(nomem, nostack)
        )
    }
}

// Read 32 bit data from 16 bit address `port`.
#[inline]
pub fn in32(port: u16) -> u32 {
    let eax: u32;
    unsafe {
        asm!(
            "in eax, dx",
            out("eax") eax,
            in("dx") port,
            options(nomem, nostack)
        )
    }
    eax
}
