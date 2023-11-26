// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2022 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{ecall2, RestrictedRange, SbiError};

/// System reset extension ID
pub const EXTENSION_ID: usize = 0x53525354;

/// The type of reset to perform
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ResetType {
    /// Shutdown the system
    Shutdown,
    /// Power off all hardware and perform a cold boot
    ColdReboot,
    /// Reset processors and some hardware
    WarmReboot,
    /// Platform specific reset type
    PlatformSpecific(RestrictedRange<0xF0000000, 0xFFFFFFFF>),
}

impl From<ResetType> for u32 {
    fn from(value: ResetType) -> Self {
        match value {
            ResetType::Shutdown => 0,
            ResetType::ColdReboot => 1,
            ResetType::WarmReboot => 2,
            ResetType::PlatformSpecific(n) => n.0,
        }
    }
}

/// The reason for performing the reset
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ResetReason {
    /// No reason for reset
    NoReason,
    /// System failure
    SystemFailure,
    /// SBI implementation specific reset reason
    SbiSpecific(RestrictedRange<0xE0000000, 0xEFFFFFFF>),
    /// Platform specific reset reason
    PlatformSpecific(RestrictedRange<0xF0000000, 0xFFFFFFFF>),
}

impl From<ResetReason> for u32 {
    fn from(value: ResetReason) -> Self {
        match value {
            ResetReason::NoReason => 0,
            ResetReason::SystemFailure => 1,
            ResetReason::SbiSpecific(n) => n.0,
            ResetReason::PlatformSpecific(n) => n.0,
        }
    }
}

/// Attempt to reset the system in the provided method, with a reason for the
/// reset.
///
/// ### Possible errors
///
/// [`SbiError::NOT_SUPPORTED`]: The [`ResetType`] is valid but not implemented.
///
/// [`SbiError::FAILED`]: The system reset request failed for an unknown reason.
pub fn system_reset(
    kind: ResetType,
    reason: ResetReason,
) -> Result<core::convert::Infallible, SbiError> {
    match unsafe {
        ecall2(
            u32::from(kind) as usize,
            u32::from(reason) as usize,
            EXTENSION_ID,
            0,
        )
    } {
        Ok(_) => unreachable!("SBI returned `Ok` after a system reset call"),
        Err(e) => Err(e),
    }
}
