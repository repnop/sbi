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
                j main
        ",
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
