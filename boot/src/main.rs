#![no_main]
#![no_std]
#![feature(abi_efiapi)]

use core::fmt::Write;
use uefi::prelude::entry;
use uefi::table::{Boot, SystemTable};
use uefi::{Handle, Status};

#[entry]
fn efi_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("Failed to initialize utilities");
    system_table.stdout().reset(false).unwrap();
    writeln!(system_table.stdout(), "Hello, UEFI!").unwrap();

    loop {}
}
