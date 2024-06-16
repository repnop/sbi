#![feature(naked_functions, fn_align)]
#![no_std]
#![no_main]

mod common;

extern "C" fn main(_hart_id: usize, _fdt: usize) -> ! {
    assert_eq!(sbi::base::impl_id(), sbi::base::SbiImplId::OpenSbi);
    assert_eq!(
        sbi::base::spec_version(),
        sbi::base::SbiSpecVersion { major: 2, minor: 0 }
    );
    assert_eq!(sbi::base::marchid(), 0);
    assert_eq!(sbi::base::mvendorid(), 0);
    assert_eq!(sbi::base::mimpid(), 0);
    assert!(sbi::base::probe_extension(sbi::hsm::EXTENSION_ID).is_available());
    assert!(sbi::base::probe_extension(sbi::ipi::EXTENSION_ID).is_available());
    assert!(sbi::base::probe_extension(sbi::pmu::EXTENSION_ID).is_available());
    assert!(sbi::base::probe_extension(sbi::rfence::EXTENSION_ID).is_available());
    assert!(sbi::base::probe_extension(sbi::system_reset::EXTENSION_ID).is_available());
    assert!(sbi::base::probe_extension(sbi::timer::EXTENSION_ID).is_available());
    assert!(sbi::base::probe_extension(sbi::legacy::CLEAR_IPI_EID).is_available());
    assert!(sbi::base::probe_extension(sbi::legacy::CONSOLE_GETCHAR_EID).is_available());
    assert!(sbi::base::probe_extension(sbi::legacy::CONSOLE_PUTCHAR_EID).is_available());
    assert!(sbi::base::probe_extension(sbi::legacy::REMOTE_FENCE_I_EID).is_available());
    assert!(sbi::base::probe_extension(sbi::legacy::REMOTE_SFENCE_VMA_ASID_EID).is_available());
    assert!(sbi::base::probe_extension(sbi::legacy::REMOTE_SFENCE_VMA_EID).is_available());
    assert!(sbi::base::probe_extension(sbi::legacy::SEND_IPI_EID).is_available());
    assert!(sbi::base::probe_extension(sbi::legacy::SET_TIMER_EID).is_available());
    assert!(sbi::base::probe_extension(sbi::legacy::SHUTDOWN_EID).is_available());
    println!("🆗 extensions successfully probed");
    common::exit(0);
}
