use core::arch::asm;

#[inline]
pub fn hlt() {
    unsafe { asm!("hlt") }
}

#[inline]
pub fn out32(port: u16, data: u32) {
    unsafe {
        asm!(
            "out dx, ax",
            in("dx") port,
            in("eax") data
        )
    }
}

#[inline]
pub fn in32(port: u16) -> u32 {
    let eax: u32;
    unsafe {
        asm!(
            "in eax, dx",
            out("eax") eax,
            in("dx") port
        )
    }
    eax
}
