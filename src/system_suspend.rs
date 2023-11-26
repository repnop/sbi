// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2023 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{ecall3, PhysicalAddress, RestrictedRange, SbiError};
use core::convert::Infallible;

/// System suspend extension ID
pub const EXTENSION_ID: usize = 0x53555350;

/// A set of values describing possible sleep states to enter
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum SleepType {
    /// Sleep type similar to ACPI S2 or S3 modes. This mode requires all harts
    /// except for the calling hart to be in the [`HartState::Stopped`][0] state
    /// and all hart registers and CSR values must be saved to RAM.
    ///
    /// [0]: crate::hart_state_management::HartState::Stopped
    SuspendToRam,
    /// Platform specific sleep type value
    PlatformSpecific(RestrictedRange<0x80000000, 0xFFFFFFFF>),
}

impl From<SleepType> for u32 {
    fn from(value: SleepType) -> Self {
        match value {
            SleepType::SuspendToRam => 0,
            SleepType::PlatformSpecific(n) => n.0,
        }
    }
}

/// Attempt to suspend the system in a way specified by the given [`SleepType`].
/// After a successful suspension, the calling hart will be resumed in S-mode
/// with `satp` and `sstatus.SIE` both initialized to `0` (thus, no memory
/// protection is enabled and interrupts are disabled) at the given
/// `resume_addr` with the hart ID in register `a0` and the `opaque` value in
/// register `a1`.
///
/// ### Safety
///
/// This function is marked as unsafe as it relies on resuming to a valid
/// physical address that is properly set up to handle execution with no memory
/// protection and an undefined register state (except `a0` and `a1`)
///
/// ### Possible errors
///
/// [`SbiError::INVALID_PARAMETER`]: The provided [`SleepType`] is reserved or
///     is platform-specific and unimplemented.
///
/// [`SbiError::NOT_SUPPORTED`]: The provided [`SleepType`] is implemented and
///     not reserved, but the platform does not support it due to one or more
///     missing dependencies.
///
/// [`SbiError::INVALID_ADDRESS`]: The provided `resume_addr` is not valid for
///     one or more reasons, potentially relating to memory protection or the
///     validity of the physical address.
///
/// [`SbiError::DENIED`]: The request failed due to unsatisfied entry criteria.
///
/// [`SbiError::FAILED`]: The request failed for unspecified or unknown reasons.
#[doc(alias = "sbi_system_suspend")]
pub unsafe fn system_suspend(
    sleep_type: SleepType,
    resume_addr: PhysicalAddress<()>,
    opaque: usize,
) -> Result<Infallible, SbiError> {
    let ret = unsafe {
        ecall3(
            usize::try_from(u32::from(sleep_type)).unwrap(),
            resume_addr.0,
            opaque,
            EXTENSION_ID,
            0,
        )
    };

    match ret {
        Ok(_) => unreachable!(),
        Err(e) => Err(e),
    }
}
