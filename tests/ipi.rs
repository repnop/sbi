#![feature(asm_sym, naked_functions, fn_align)]
#![no_std]
#![no_main]

mod common;

use sbi::hart_state_management::HartStatus;

#[no_mangle]
extern "C" fn main(hart_id: usize, _fdt: usize) -> ! {
    let target_hart = if hart_id == 0 { 1 } else { 0 };
    common::start_other_hart(other_main);
    while !matches!(
        sbi::hart_state_management::hart_status(target_hart).expect("hart_status"),
        HartStatus::Started
    ) {}
    common::wait(1000);
    sbi::ipi::send_ipi(sbi::HartMask::from(target_hart)).expect("send_ipi");
    common::wait(1000);
    println!("❌ Other hart did not trigger an exit in time");
    common::exit(1);
}

#[no_mangle]
extern "C" fn other_main(_: usize) -> ! {
    println!("🆗 Hart started");
    common::set_stvec(success);
    common::enable_interrupts();
    loop {}
}

const SUPERVISOR_SOFTWARE_INTERRUPT: usize = (1 << (usize::BITS - 1)) | 1;
#[repr(align(4))]
extern "C" fn success() -> ! {
    assert_eq!(
        common::scause(),
        SUPERVISOR_SOFTWARE_INTERRUPT,
        "not an IPI"
    );
    println!("🆗 IPI received -- success");
    common::exit(0);
}
