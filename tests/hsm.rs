#![feature(naked_functions, fn_align)]
#![no_std]
#![no_main]

mod common;

use core::sync::atomic::AtomicBool;
use sbi::hart_state_management::{hart_state, hart_stop, HartState};

static HART_BOOTED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
extern "C" fn main(hart_id: usize, _fdt: usize) -> ! {
    let target_hart = if hart_id == 0 { 1 } else { 0 };
    common::start_other_hart(other_main);
    common::wait(10);

    if !HART_BOOTED.load(core::sync::atomic::Ordering::Acquire) {
        panic!("❌ Hart {target_hart} did not start");
    }

    assert_eq!(
        hart_state(target_hart).expect("hart_status"),
        HartState::Started,
    );

    println!("🆗 Hart {target_hart} started");

    common::wait(150);

    assert_eq!(
        hart_state(target_hart).expect("hart_status"),
        HartState::Stopped,
        "❌ Hart {target_hart} did not stop in time",
    );

    println!("🆗 Hart {target_hart} stopped");

    println!("🆗 Success");
    common::exit(0);
}

#[no_mangle]
extern "C" fn other_main(_: usize) -> ! {
    HART_BOOTED.store(true, core::sync::atomic::Ordering::Release);
    common::wait(100);
    #[allow(unreachable_code)]
    match hart_stop().expect("hart_stop") {}
}
