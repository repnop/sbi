#![feature(naked_functions, fn_align)]
#![no_std]
#![no_main]

mod common;

extern "C" fn main(_hart_id: usize, _fdt: usize) -> ! {
    common::set_stvec(success);
    common::enable_interrupts();
    sbi::timer::set_timer(common::time() + 100).expect("set_timer");
    common::wait(100);
    common::exit(1);
}

const SUPERVISOR_TIMER_INTERRUPT: usize = (1 << (usize::BITS - 1)) | 5;

#[repr(align(4))]
extern "C" fn success() -> ! {
    assert_eq!(
        common::scause(),
        SUPERVISOR_TIMER_INTERRUPT,
        "was not a timer interrupt!"
    );
    println!("🆗 Timer interrupt received -- success");
    common::exit(0);
}
