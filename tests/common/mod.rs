static mut BOOT_HART_ID: usize = 0;

#[naked]
#[no_mangle]
#[rustfmt::skip]
#[link_section = ".text.init"]
unsafe extern "C" fn _start(hart_id: usize, fdt: usize) -> ! {
    core::arch::asm!(
        "
            .option push
            .option norelax
            lla gp, __global_pointer$
            .option pop

            lla sp, __stack_start

            lla t0, __bss_start
            lla t1, __bss_end

            // Clear BSS
            1:
                beq t0, t1, 2f
                sw zero, (t0)
                addi t0, t0, 4
                j 1b
            2:
                lla t2, {boot_hart_id}
                sw a0, 0(t2)
                lla t2, {fail}
                csrw stvec, t2
                j main
        ",
        boot_hart_id = sym BOOT_HART_ID,
        fail = sym fail,
        options(noreturn),
    );
}

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    crate::println!("{}", panic_info);
    exit(1);
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($($arg:tt)*) => ($crate::print!("{}\r\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::common::_print(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    Uart.write_fmt(args).unwrap();
}

struct Uart;

impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let uart = unsafe { &*(0x10000000 as *mut u8 as *mut core::sync::atomic::AtomicU8) };
        for byte in s.bytes() {
            uart.store(byte, core::sync::atomic::Ordering::Relaxed);
        }

        Ok(())
    }
}

pub fn exit(status: u16) -> ! {
    let test_device = 0x10_0000 as *mut u32;
    let exit_status: u32 = match status {
        0 => 0x5555,
        n => ((n as u32) << 16) | 0x3333,
    };

    unsafe { core::ptr::write_volatile(test_device, exit_status) };

    unreachable!()
}

pub fn wait(millis: u32) {
    let mut time = time();

    // QEMU has a 10 MHz clock

    let micros = millis as u64 * 1000;
    let hundred_nanos = micros * 10;
    let target_time = time + hundred_nanos;

    while time < target_time {
        time = self::time();
    }
}

pub fn time() -> u64 {
    let time: u64;
    #[cfg(target_arch = "riscv64")]
    unsafe {
        core::arch::asm!("csrr {}, time", out(reg) time)
    };

    #[cfg(target_arch = "riscv32")]
    unsafe {
        let timeh: u32;
        let timel: u32;
        core::arch::asm!("csrr {}, timeh", out(reg) timeh);
        core::arch::asm!("csrr {}, time", out(reg) timel);
        time = (u64::from(timeh) << 32) | u64::from(timel);
    };

    time
}

pub fn set_stvec(f: extern "C" fn() -> !) {
    unsafe { core::arch::asm!("csrw stvec, {}", in(reg) f) };
}

#[naked]
#[rustfmt::skip]
unsafe extern "C" fn other_entry() -> ! {
    core::arch::asm!(
        "lla sp, __stack_end2",
        "jr a1",
        options(noreturn),
    )
}

pub fn start_other_hart(f: extern "C" fn(usize) -> !) {
    let target_hart = if unsafe { BOOT_HART_ID } == 0 { 1 } else { 0 };
    sbi::hart_state_management::hart_start(target_hart, other_entry as usize, f as usize)
        .expect("start_hart");
}

pub fn scause() -> usize {
    let mut scause: usize;
    unsafe { core::arch::asm!("csrr {}, scause", out(reg) scause) };
    scause
}

pub fn enable_interrupts() {
    unsafe { core::arch::asm!("csrs sie, {}", in(reg) (1 << 1) | (1 << 5) | (1 << 9)) };
    unsafe { core::arch::asm!("csrsi sstatus, 1 << 1") };
}

#[repr(align(4))]
extern "C" fn fail() -> ! {
    crate::println!("Unexpected trap: {:#X}", scause());
    exit(1);
}
