// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2022 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{ecall0, ecall1, ecall3, PhysicalAddress, RestrictedRange, SbiError};

/// Hart state management extension ID
pub const EXTENSION_ID: usize = 0x48534D;

/// Start the specific hart ID at the given physical address along with a
/// user-defined value. On success, the hart begins execution at the physical
/// address with the following register states:
///
/// `satp` is reset to a value of `0` (virtual memory protection disabled)
///
/// `sstatus.SIE` is reset to a value of `0` (supervisor interrupts disabled)
///
/// `a0` contains the current hart ID
///
/// `a1` contains the value of the `opaque` parameter
///
/// All other registers are in an undefined state
///
/// ### Safety
///
/// This function is marked unsafe as it allows arbitrary execution at a given
/// physical address, which can cause undefined behavior if used incorrectly.
///
/// ### Possible errors
///
/// [`SbiError::INVALID_ADDRESS`]: `start_address` is an invalid address because
///     it is either an invalid physical address or execution is prohibited by
///     physical memory protection.
///
/// [`SbiError::INVALID_PARAMETER`]: The specified hart ID is either not valid
///     or cannot be started in S-mode.
///
/// [`SbiError::ALREADY_AVAILABLE`]: The specified hart ID is already started.
///
/// [`SbiError::FAILED`]: Start request failed for unknown reasons.
pub unsafe fn hart_start(
    hart_id: usize,
    start_addr: PhysicalAddress<()>,
    private: usize,
) -> Result<(), SbiError> {
    unsafe { ecall3(hart_id, start_addr.0 as usize, private, EXTENSION_ID, 0).map(drop) }
}

/// This SBI call stops S-mode execution on the current hart and yields
/// execution back to the SBI implementation. Note: **this function must be
/// called with supervisor and user interrupts disabled.**
///
/// ### Possible errors
///
/// [`SbiError::FAILED`]: The request failed for an unknown reason.
pub fn hart_stop() -> Result<core::convert::Infallible, SbiError> {
    match unsafe { ecall0(EXTENSION_ID, 1) } {
        Ok(_) => unreachable!("SBI returned `Ok` when stopping the current hart"),
        Err(e) => Err(e),
    }
}

/// Retrieve the state of the specified hart ID.
///
/// ### Possible errors
///
/// [`SbiError::INVALID_PARAMETER`]: The specified hart ID is not valid.
pub fn hart_state(hart_id: usize) -> Result<HartState, SbiError> {
    unsafe { ecall1(hart_id, EXTENSION_ID, 2).map(HartState::from_usize) }
}

/// Places the current hart into a suspended or low power state specified by the
/// `suspend_type` parameter. The hart will resume normal execution after an
/// interrupt or platform-specific hardware event. The resume behavior depends
/// on the type of suspension passed to this call: retentive suspend types will
/// save all supervisor register and CSR states, and the hart will continue the
/// execution path after being woken (as in, from the supervisor point-of-view,
/// the SBI call returned normally with nothing changing inbetween).
/// Non-retentive suspend types will **not** save any supervisor register or CSR
/// state, and will resume execution at the given `resume_address` with:
///
/// `satp` is reset to a value of `0` (virtual memory protection disabled)
///
/// `sstatus.SIE` is reset to a value of `0` (supervisor interrupts disabled)
///
/// `a0` contains the current hart ID
///
/// `a1` contains the value of the `opaque` parameter
///
/// All other register states are undefined
///
/// ### Safety
///
/// This function is unsafe as it allows arbitrary execution at a given physical
/// address, which may cause undefined behavior if used incorrectly.
///
/// ### Possible errors
///
/// [`SbiError::INVALID_ADDRESS`]: An invalid address was given for
///     `resume_address` because it was either: an invalid physical address, or
///     the resume address is probited by Physical Memory Protection (PMP) to
///     run in supervisor mode.
///
/// [`SbiError::NOT_SUPPORTED`]: The given `suspend_type` is valid but not
///     implemented.
///
/// [`SbiError::FAILED`]: The suspension request failed for an unknown reason.
pub unsafe fn hart_suspend(suspend_type: SuspendType) -> Result<(), SbiError> {
    let (value, resume_addr, opaque) = suspend_type.to_values();
    unsafe { ecall3(value as usize, resume_addr, opaque, EXTENSION_ID, 3).map(drop) }
}

/// The type of suspension to be executed whe ncalling [`hart_suspend`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuspendType {
    /// Default retentive suspension which saves register and CSR state and
    /// restores those states upon hart resume.
    DefaultRetentive,
    /// A platform specific retentive suspend type, which saves register and CSR
    /// state and restores those states upon hart resume. The value is within
    /// the range of `0x10000000..=0x7FFFFFFF`.
    PlatformSpecificRetentive(RestrictedRange<0x10000000, 0x7FFFFFFF>),
    /// Default non-retentive suspension which does not save any register or CSR
    /// state. The hart will resume execution at `resume_address` with only
    /// registers `a0` and `a1`, and CSRs `satp` and `sstatus.SIE` in a defined
    /// state.
    DefaultNonRetentive {
        /// The address to resume execution at.
        resume_address: PhysicalAddress<()>,
        /// User-defined opaque value passed to `resume_address` in `a1` upon
        /// resumption.
        opaque: usize,
    },
    /// A platform specific non-retentive suspend type, which does not save any
    /// register or CSR state. The variant `value` is a value in the range
    /// `0x90000000..=0xFFFFFFFF`. The hart will resume execution at the
    /// `resume_address` with only registers `a0` and `a1`, and CSRs `satp` and
    /// `sstatus.SIE` in a defined state.
    PlatformSpecificNonRetentive {
        /// The platform-specific suspend value, in the range
        /// `0x90000000..=0xFFFFFFFF`.
        value: RestrictedRange<0x90000000, 0xFFFFFFFF>,
        /// The address to resume execution at.
        resume_address: PhysicalAddress<()>,
        /// User-defined opaque value passed to `resume_address` in `a1` upon
        /// resumption.
        opaque: usize,
    },
}

impl SuspendType {
    fn to_values(self) -> (u32, usize, usize) {
        match self {
            Self::DefaultRetentive => (0x00000000, 0, 0),
            Self::PlatformSpecificRetentive(n) => (n.0, 0, 0),
            Self::DefaultNonRetentive {
                resume_address,
                opaque,
            } => (0x80000000, resume_address.as_ptr() as usize, opaque),
            Self::PlatformSpecificNonRetentive {
                value,
                resume_address,
                opaque,
            } => (value.0, resume_address.as_ptr() as usize, opaque),
        }
    }
}

/// Execution state for a hart
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum HartState {
    /// The hart is powered on and executing normally
    Started,
    /// The hart is currently not executing in supervisor mode or any less
    /// privileged execution mode
    Stopped,
    /// A start request is pending for the hart
    StartRequestPending,
    /// A stop request is pending for the hart
    StopRequestPending,
    /// The hart is in a platform specific suspend (or low power) state
    Suspended,
    /// A suspend request is pending for the hart
    SuspendPending,
    /// An event has caused the hart to begin resuming normal execution
    /// ([`HartState::Started`])
    ResumePending,
}

impl HartState {
    fn from_usize(n: usize) -> Self {
        match n {
            0 => HartState::Started,
            1 => HartState::Stopped,
            2 => HartState::StartRequestPending,
            3 => HartState::StopRequestPending,
            4 => HartState::Suspended,
            5 => HartState::SuspendPending,
            6 => HartState::ResumePending,
            n => unreachable!("invalid hart state returned by SBI: {}", n),
        }
    }
}
