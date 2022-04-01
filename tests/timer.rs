#![feature(asm_sym, naked_functions, fn_align)]
#![no_std]
#![no_main]

mod common;

#[no_mangle]
extern "C" fn main(hart_id: usize, _fdt: usize) -> ! {
    common::set_stvec(success);
    common::enable_interrupts();
    sbi::timer::set_timer(0);
    common::wait(100);
    common::exit(1);
}

const SUPERVISOR_TIMER_INTERRUPT: usize = (1 << 63) | 5;
fn success() -> ! {
    assert_eq!(
        common::scause(),
        SUPERVISOR_TIMER_INTERRUPT,
        "was not a timer interrupt!"
    );
    println!("🆗 Timer interrupt received -- success");
    common::exit(0);
}
