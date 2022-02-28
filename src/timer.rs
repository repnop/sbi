// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2022 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::SbiError;

/// Timer extension ID
pub const EXTENSION_ID: usize = 0x54494D45;

/// Schedule an interrupt for `time` in the future. To clear the timer interrupt
/// without scheduling another timer event, a time infinitely far into the
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
#[rustfmt::skip]
pub fn set_timer(time: u64) -> Result<(), SbiError> {
    #[cfg(target_arch = "riscv64")]
    unsafe { crate::ecall1(time as usize, EXTENSION_ID, 0).map(drop) }

    // Since `time` is always a `u64`, we need to split it up into two arguments
    // on the 32-bit targets, with the low 32-bits in `a0` and the high 32-bits
    // in `a1`
    #[cfg(target_arch = "riscv32")]
    unsafe { crate::ecall2(time as usize, (time >> 32) as usize, EXTENSION_ID, 0).map(drop) }
}
