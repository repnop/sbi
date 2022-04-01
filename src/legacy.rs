// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2022 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use core::arch::asm;

/// `sbi_set_timer` extension ID
pub const SET_TIMER_EID: usize = 0x00;

/// Schedule an interrupt for `time` in the future. To clear the timer interrupt
/// without scheduling another timer event, set a time infinitely far into the
/// future (`u64::MAX`) or mask the `STIE` bit of the `sie` CSR. This function
/// will clear the pending timer interrupt bit.
///
/// Note: `time` is an absolute time, not an offset from when the call is made.
/// This means that if you want to set a time that is _n_ ticks in the future,
/// you will need to read the `time` CSR first, then add the ticks to that. How
/// you determine the number of time each tick represents is platform-dependent,
/// and the frequency of the clock should be expressed in the
/// `timebase-frequency` property of the CPU nodes in the devicetree, if you
/// have one available.
#[inline]
#[doc(alias = "sbi_set_timer")]
pub fn set_timer(stime: u64) {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!(
            "ecall",
            inout("a0") stime => _,
            in("a7") SET_TIMER_EID,
        );
    }

    #[cfg(target_arch = "riscv32")]
    unsafe {
        asm!(
            "ecall",
            inout ("a0") stime as usize => _,
            inout ("a1") (stime >> 32) as usize => _,
            in("a7") SET_TIMER_EID,
        );
    }
}

/// `sbi_console_putchar` extension ID
pub const CONSOLE_PUTCHAR_EID: usize = 0x01;

/// Write a character to the debug console. This call will block if there is
/// still pending console output. If no console exists, no action is taken.
#[inline]
#[doc(alias = "sbi_console_putchar")]
pub fn console_putchar(c: u8) {
    unsafe {
        asm!(
            "ecall",
            inout("a0") c as usize => _,
            in("a7") CONSOLE_PUTCHAR_EID,
        );
    }
}

/// `sbi_console_getchar` extension ID
pub const CONSOLE_GETCHAR_EID: usize = 0x02;

/// Attempt to retrieve a character from the debug console. If there is no
/// character waiting to be read, or if there is no debug console device, this
/// function will return [`None`].
#[inline]
#[doc(alias = "sbi_console_getchar")]
pub fn console_getchar() -> Option<u8> {
    let mut ret: i8;

    unsafe {
        asm!(
            "ecall",
            lateout("a0") ret,
            in("a7") CONSOLE_GETCHAR_EID,
        );
    }

    match ret {
        -1 => None,
        _ => Some(ret as u8),
    }
}

/// `sbi_clear_ipi` extension ID
pub const CLEAR_IPI_EID: usize = 0x03;

/// Clears any pending interprocessor interrupts (IPIs) for the hart this
/// function is called from.
#[inline]
#[doc(alias = "sbi_clear_ipi")]
#[deprecated = "S-mode can clear the `sip.SSIP` CSR bit directly, it is not necessary to call this function"]
pub fn clear_ipi() {
    unsafe {
        asm!(
            "ecall",
            in("a7") CLEAR_IPI_EID,
            lateout("a0") _,
        );
    }
}

/// `sbi_send_ipi` extension ID
pub const SEND_IPI_EID: usize = 0x04;

/// Send an interprocessor interrupt (IPI) to all of the harts specified by
/// the `hart_mask` bitmask. Received IPIs are represented as Supervisor
/// Software Interrupts.
///
/// `hart_mask` is a bit vector of length `n_harts / size_of::<usize>()`,
/// rounded up to the next `usize`.
#[inline]
#[doc(alias = "sbi_send_ipi")]
pub fn send_ipi(hart_mask: &[usize]) {
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") hart_mask.as_ptr() => _,
            in("a7") SEND_IPI_EID,
        );
    }
}

/// `sbi_remote_fence_i` extension ID
pub const REMOTE_FENCE_I_EID: usize = 0x05;

/// Execute a `FENCE.I` instruction on the harts specified by `hart_mask`
/// bitmask.
///
/// `hart_mask` is a bit vector of length `n_harts / size_of::<usize>()`,
/// rounded up to the next `usize`.
#[inline]
#[doc(alias = "sbi_remote_fence_i")]
pub fn remote_fence_i(hart_mask: &[usize]) {
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") hart_mask.as_ptr() => _,
            in("a7") REMOTE_FENCE_I_EID,
        );
    }
}

/// `sbi_remote_sfence_vma` extension ID
pub const REMOTE_SFENCE_VMA_EID: usize = 0x06;

/// Execute a `SFENCE.VMA` instruction on the harts specified by `hart_mask`
/// bitmask for the virtual memory range specified by `start` and `size`.
///
/// `hart_mask` is a bit vector of length `n_harts / size_of::<usize>()`,
/// rounded up to the next `usize`.
///
/// `start` is the starting virtual address to execute the `SFENCE.VMA` on.
///
/// `size` is the size of the region to `SFENCE.VMA` in bytes. For example, to
/// invalidate a region of 2 4-KiB pages, you would pass `8192` for `size`.
///
/// If `start` and `size` are both `0`, or if `size` is [`usize::MAX`], a full
/// `SFENCE.VMA` will be executed instead of one or more page-sized
/// `SFENCE.VMA`s.
#[inline]
#[doc(alias = "sbi_remote_sfence_vma")]
pub fn remote_sfence_vma(hart_mask: &[usize], start: usize, size: usize) {
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") hart_mask.as_ptr() => _,
            in("a1") start,
            in("a2") size,
            in("a7") REMOTE_SFENCE_VMA_EID,
        );
    }
}

/// `sbi_remote_sfence_vma_asid` extension ID
pub const REMOTE_SFENCE_VMA_ASID_EID: usize = 0x07;

/// Execute a `SFENCE.VMA` instruction on the harts specified by `hart_mask`
/// bitmask for the virtual memory range specified by `start` and `size` for the
/// given Address Space ID (ASID) only.
///
/// `hart_mask` is a bit vector of length `n_harts / size_of::<usize>()`,
/// rounded up to the next `usize`.
///
/// `start` is the starting virtual address to execute the `SFENCE.VMA` on.
///
/// `size` is the size of the region to `SFENCE.VMA` in bytes. For example, to
/// invalidate a region of 2 4-KiB pages, you would pass `8192` for `size`.
///
/// If `start` and `size` are both `0`, or if `size` is [`usize::MAX`], a full
/// `SFENCE.VMA` will be executed instead of one or more page-sized
/// `SFENCE.VMA`s.
#[inline]
#[doc(alias = "sbi_remote_sfence_vma_asid")]
pub fn remote_sfence_vma_asid(hart_mask: &[usize], start: usize, size: usize, asid: usize) {
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") hart_mask.as_ptr() => _,
            in("a1") start,
            in("a2") size,
            in("a3") asid,
            in("a7") REMOTE_SFENCE_VMA_ASID_EID,
        );
    }
}

/// `sbi_shutdown` extension ID
pub const SHUTDOWN_EID: usize = 0x08;

/// Puts all harts into a shutdown state wherein the execution mode of the
/// processors is more privileged than the current supervisor mode. This call
/// does not return.
#[inline]
#[doc(alias = "sbi_shutdown")]
pub fn shutdown() -> ! {
    unsafe {
        asm!(
            "ecall",
            in("a7") SHUTDOWN_EID,
            options(noreturn)
        );
    }
}
