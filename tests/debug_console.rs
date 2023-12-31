#![feature(naked_functions, fn_align)]
#![no_std]
#![no_main]

use core::ptr::NonNull;

use sbi::PhysicalAddress;

mod common;

static WRITE_OK: &str = "🆗 Successfully wrote to console with write_ptr";
static READ_MSG: &str = "\r\nReading test text from QEMU input file\r\n";

extern "C" fn main(_hart_id: usize, _fdt: usize) -> ! {
    let mut buf = &mut [0u8; 256];

    let read =
        unsafe { sbi::debug_console::read_ptr(PhysicalAddress::from_ptr(buf)).expect("read ok") };

    assert_eq!(
        core::str::from_utf8(&mut buf[..read]).unwrap(),
        "This is some test UART input",
        "❌ didn't read any input"
    );

    println!("🆗 Successfully read test input");

    unsafe {
        sbi::debug_console::write_ptr(PhysicalAddress::from_ptr(
            WRITE_OK.as_bytes() as *const [u8] as *mut [u8],
        ))
        .expect("write ok");
    }

    sbi::debug_console::write_byte(b'A').expect("write ok");

    common::exit(0);
}
