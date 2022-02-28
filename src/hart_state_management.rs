// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2022 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{ecall0, ecall1, ecall3, SbiError};

/// Hart state management extension ID
pub const EXTENSION_ID: usize = 0x48534D;

/// Start the specific hart ID at the given physical address along with a
/// user-defined value. On success, the hart begins execution at the physical
/// address with the hart ID in `a0` and the user-defined value in `a1`, all
/// other register values are in an undefined state.
///
/// ### Possible errors
///
/// [`SbiError::InvalidAddress`]: `start_address` is an invalid address because it is either
/// an invalid physical address or execution is prohibited by physical memory
/// protection
///
/// [`SbiError::InvalidParameter`]: The specified hart ID is either not valid or cannot be
/// started in S-mode
///
/// [`SbiError::AlreadyAvailable`]: The specified hart ID is already started
///
/// [`SbiError::Failed`]: Start request failed for unknown reasons
pub fn hart_start(hart_id: usize, start_addr: usize, private: usize) -> Result<(), SbiError> {
    unsafe { ecall3(hart_id, start_addr, private, EXTENSION_ID, 0).map(drop) }
}

/// This SBI call stops S-mode execution on the current hart and yields
/// execution back to the SBI implementation. Note that this function must be
/// called with supervisor and user interrupts disabled.
///
/// ### Possible errors
///
/// [`SbiError::Failed`]: The request failed for an unknown reason
pub fn hart_stop() -> Result<core::convert::Infallible, SbiError> {
    match unsafe { ecall0(EXTENSION_ID, 1) } {
        Ok(_) => unreachable!("SBI returned `Ok` when stopping the current hart"),
        Err(e) => Err(e),
    }
}

/// Retrieve the status of the specified hart ID.
///
/// ### Possible errors
///
/// [`SbiError::InvalidParameter`]: The specified hart ID is not valid
pub fn hart_status(hart_id: usize) -> Result<HartStatus, SbiError> {
    unsafe { ecall1(hart_id, EXTENSION_ID, 2).map(HartStatus::from_usize) }
}

/// Places the current hart into a suspended or low power state specified by the
/// `suspend_type` parameter. The hart will resume normal execution after an
/// interrupt or platform-specific hardware event. The resume behavior depends
/// on the type of suspension passed to this call: retentive suspend types will
/// save all supervisor register and CSR states, and the hart will continue the
/// execution path after being woken (as in, from the supervisor point-of-view,
/// the SBI call returned normally with nothing changing inbetween).
/// Non-retentive suspend types will **not** save any supervisor register or CSR
/// state, and will resume execution at the given `resume_address` with only the
/// following states being defined:
///
/// `satp` is reset to a value of `0` (virtual memory protection disabled)
///
/// `sstatus.SIE` is reset to a value of `0` (supervisor interrupts disabled)
///
/// `a0` contains the current hart ID
///
/// `a1` contains the value of the `opaque` parameter
///
/// ### Possible errors
///
/// [`SbiError::InvalidAddress`]: An invalid address was given for
///     `resume_address` because it was either: an invalid physical address, or
///     the resume address is probited by Physical Memory Protection (PMP) to
///     run in supervisor mode
///
/// [`SbiError::NotSupported`]: The given `suspend_type` is valid but not
///     implemented
///
/// [`SbiError::Failed`]: The suspension request failed for an unknown reason
pub fn hart_suspend(suspend_type: SuspendType) -> Result<(), SbiError> {
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
    /// state and restores those states upon hart resume. The variant value is a
    /// value in the range `0x00000000..=0x6FFFFFFF`. This value will be clamped
    /// to the maximum possible valid value for this suspension type if the
    /// contained value exceeds it.
    PlatformSpecificRetentive(u32),
    /// Default non-retentive suspension which does not save any register or CSR
    /// state. The hart will resume execution at `resume_address` with only
    /// registers `a0` and `a1`, and CSRs `satp` and `sstatus.SIE` in a defined
    /// state.
    DefaultNonRetentive {
        /// The address to resume execution at.
        resume_address: usize,
        /// User-defined opaque value passed to `resume_address` in `a1` upon
        /// resumption.
        opaque: usize,
    },
    /// A platform specific non-retentive suspend type, which does not save any
    /// register or CSR state. The variant `value` is a value in the range
    /// `0x00000000..=0x6FFFFFFF`. This value will be clamped to the maximum
    /// possible valid value for this suspension type if the contained value
    /// exceeds it. The hart will resume execution at the `resume_address` with
    /// only registers `a0` and `a1`, and CSRs `satp` and `sstatus.SIE` in a
    /// defined state.
    PlatformSpecificNonRetentive {
        /// The platform-specific suspend value, in the range
        /// `0x00000000..=0x6FFFFFFF`.
        value: u32,
        /// The address to resume execution at.
        resume_address: usize,
        /// User-defined opaque value passed to `resume_address` in `a1` upon
        /// resumption.
        opaque: usize,
    },
}

impl SuspendType {
    fn to_values(self) -> (u32, usize, usize) {
        match self {
            Self::DefaultRetentive => (0x00000000, 0, 0),
            Self::PlatformSpecificRetentive(n) => (n.min(0x6FFFFFFF) + 0x10000000, 0, 0),
            Self::DefaultNonRetentive {
                resume_address,
                opaque,
            } => (0x80000000, resume_address, opaque),
            Self::PlatformSpecificNonRetentive {
                value,
                resume_address,
                opaque,
            } => (value.min(0x6FFFFFFF) + 0x90000000, resume_address, opaque),
        }
    }
}

/// Execution status for a hart
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HartStatus {
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
    /// ([`HartStatus::Started`])
    ResumePending,
}

impl HartStatus {
    fn from_usize(n: usize) -> Self {
        match n {
            0 => HartStatus::Started,
            1 => HartStatus::Stopped,
            2 => HartStatus::StartRequestPending,
            3 => HartStatus::StopRequestPending,
            4 => HartStatus::Suspended,
            5 => HartStatus::SuspendPending,
            6 => HartStatus::ResumePending,
            n => unreachable!("invalid hart status returned by SBI: {}", n),
        }
    }
}
