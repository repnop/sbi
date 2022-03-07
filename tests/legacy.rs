#![feature(naked_functions, fn_align)]
#![no_std]
#![no_main]

mod common;

static mut HART_ID: usize = 0;

#[no_mangle]
extern "C" fn main(hart_id: usize, _fdt: usize) -> ! {
    unsafe { HART_ID = hart_id };
    println!("hart_id={hart_id}");
    panic!();
}
