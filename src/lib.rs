// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2022 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![no_std]

#[cfg(all(not(target_arch = "riscv64"), not(target_arch = "riscv32")))]
compile_error!("SBI is only available on RISC-V platforms");

/// Required base SBI functionality
pub mod base;
/// Hart State Management extension
pub mod hart_state_management;
/// IPI extension
pub mod ipi;
/// Legacy SBI calls
pub mod legacy;
/// Performance Monitoring Unit extension
pub mod performance_monitoring_unit;
/// RFENCE extension
pub mod rfence;
/// System Reset extension
pub mod system_reset;
/// Timer extension
pub mod timer;

pub use base::{
    impl_id, impl_version, marchid, mimpid, mvendorid, probe_extension, spec_version,
    ExtensionAvailability,
};
/// A convenience alias to the [`hart_state_management`] module.
pub use hart_state_management as hsm;

/// Error codes returned by SBI calls
///
/// note: `SBI_SUCCESS` is not represented here since this is to be used as the
/// error type in a `Result`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SbiError {
    /// The SBI call failed
    Failed,
    /// The SBI call is not implemented or the functionality is not available
    NotSupported,
    /// An invalid parameter was passed
    InvalidParameter,
    /// The SBI implementation has denied execution of the call functionality
    Denied,
    /// An invalid address was passed
    InvalidAddress,
    /// The resource is already available
    AlreadyAvailable,
}

impl SbiError {
    #[inline]
    fn new(n: isize) -> Self {
        match n {
            -1 => SbiError::Failed,
            -2 => SbiError::NotSupported,
            -3 => SbiError::InvalidParameter,
            -4 => SbiError::Denied,
            -5 => SbiError::InvalidAddress,
            -6 => SbiError::AlreadyAvailable,
            n => unreachable!("bad SBI error return value: {}", n),
        }
    }
}

impl core::fmt::Display for SbiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SbiError::AlreadyAvailable => "resource is already available",
                SbiError::Denied => "SBI implementation denied execution",
                SbiError::Failed => "call to SBI failed",
                SbiError::InvalidAddress => "invalid address passed",
                SbiError::InvalidParameter => "invalid parameter passed",
                SbiError::NotSupported => "SBI call not implemented or functionality not available",
            }
        )
    }
}

/// A SBI hart mask
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HartMask {
    base: usize,
    mask: usize,
}

impl HartMask {
    /// Create a new [`HartMask`] with the given base and no hart IDs selected
    #[inline]
    pub const fn new(base: usize) -> Self {
        Self { base, mask: 0 }
    }

    /// Create a new [`HartMask`] from the given hart ID, making it the base and
    /// selecting it
    #[inline]
    pub const fn from(hart_id: usize) -> Self {
        Self {
            base: hart_id,
            mask: 1,
        }
    }

    /// Select the given hart ID. If `hart_id` is out of the range of available
    /// selectable hart IDs, the [`HartMask`] is unchanged.
    #[inline]
    #[must_use]
    pub const fn with(mut self, hart_id: usize) -> Self {
        if hart_id >= self.base && hart_id < (self.base + usize::BITS as usize) {
            self.mask |= 1 << (hart_id - self.base);
        }

        self
    }
}

/// A convenience macro to help create a [`HartMask`] from either one or more
/// hart IDs or a base and a list of hart IDs.
///
/// Examples:
///
/// A single hart ID: `hart_mask!(my_hart_id);`
///
/// Multiple hart IDs: `hart_mask!(1, 3, 5);`
///
/// An explicit base with a list of hart IDs: `hart_mask!(base: 0, ids: 1, 3, 5);`
#[macro_export]
macro_rules! hart_mask {
    ($hart_id1:expr $(, $($hart_id:expr),+ $(,)?)?) => {{
        let mut hart_mask = $crate::HartMask::from($hart_id1);
        $($(hart_mask = hart_mask.with($hart_id);)+)?
        hart_mask
    }};
    (base: $base:literal, ids: $($hart_id:expr),* $(,)?) => {{
        let mut hart_mask = $crate::HartMask::new($base);
        $(hart_mask = hart_mask.with($hart_id);)*
        hart_mask
    }};
}

/// A zero-argument `ecall` with the given extension and function IDs.
///
/// # Safety
/// This function is only safe to call if the given function ID accepts no
/// parameters, otherwise the behavior is undefined, as the additional argument
/// registers will have undefined contents when passed to the SBI
/// implementation.
#[inline]
pub unsafe fn ecall0(extension_id: usize, function_id: usize) -> Result<usize, SbiError> {
    let error: isize;
    let value: usize;

    core::arch::asm!(
        "ecall",
        in("a6") function_id,
        in("a7") extension_id,
        lateout("a0") error,
        lateout("a1") value,
    );

    match error {
        0 => Result::Ok(value),
        e => Result::Err(SbiError::new(e)),
    }
}

/// A one-argument `ecall` with the given extension and function IDs.
///
/// # Safety
/// This function is only safe to call if the given function ID accepts one
/// parameter, otherwise the behavior is undefined, as the additional argument
/// registers will have undefined contents when passed to the SBI
/// implementation.
#[inline]
pub unsafe fn ecall1(
    arg: usize,
    extension_id: usize,
    function_id: usize,
) -> Result<usize, SbiError> {
    let error: isize;
    let value: usize;

    core::arch::asm!(
        "ecall",
        inlateout("a0") arg => error,
        in("a6") function_id,
        in("a7") extension_id,
        lateout("a1") value,
    );

    match error {
        0 => Result::Ok(value),
        e => Result::Err(SbiError::new(e)),
    }
}

/// A two-argument `ecall` with the given extension and function IDs.
///
/// # Safety
/// This function is only safe to call if the given function ID accepts two
/// parameters, otherwise the behavior is undefined, as the additional argument
/// registers will have undefined contents when passed to the SBI
/// implementation.
#[inline]
pub unsafe fn ecall2(
    arg0: usize,
    arg1: usize,
    extension_id: usize,
    function_id: usize,
) -> Result<usize, SbiError> {
    let error: isize;
    let value: usize;

    core::arch::asm!(
        "ecall",
        inlateout("a0") arg0 => error,
        inlateout("a1") arg1 => value,
        in("a6") function_id,
        in("a7") extension_id,
    );

    match error {
        0 => Result::Ok(value),
        e => Result::Err(SbiError::new(e)),
    }
}

/// A three-argument `ecall` with the given extension and function IDs.
///
/// # Safety
/// This function is only safe to call if the given function ID accepts three
/// parameters, otherwise the behavior is undefined, as the additional argument
/// registers will have undefined contents when passed to the SBI
/// implementation.
#[inline]
pub unsafe fn ecall3(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    extension_id: usize,
    function_id: usize,
) -> Result<usize, SbiError> {
    let error: isize;
    let value: usize;

    core::arch::asm!(
        "ecall",
        inlateout("a0") arg0 => error,
        inlateout("a1") arg1 => value,
        in("a2") arg2,
        in("a6") function_id,
        in("a7") extension_id,
    );

    match error {
        0 => Result::Ok(value),
        e => Result::Err(SbiError::new(e)),
    }
}

/// A four-argument `ecall` with the given extension and function IDs.
///
/// # Safety
/// This function is only safe to call if the given function ID accepts four
/// parameters, otherwise the behavior is undefined, as the additional argument
/// registers will have undefined contents when passed to the SBI
/// implementation.
#[inline]
pub unsafe fn ecall4(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    extension_id: usize,
    function_id: usize,
) -> Result<usize, SbiError> {
    let error: isize;
    let value: usize;

    core::arch::asm!(
        "ecall",
        inlateout("a0") arg0 => error,
        inlateout("a1") arg1 => value,
        in("a2") arg2,
        in("a3") arg3,
        in("a6") function_id,
        in("a7") extension_id,
    );

    match error {
        0 => Result::Ok(value),
        e => Result::Err(SbiError::new(e)),
    }
}

/// A five-argument `ecall` with the given extension and function IDs.
///
/// # Safety
/// This function is only safe to call if the given function ID accepts five
/// parameters, otherwise the behavior is undefined, as the additional argument
/// registers will have undefined contents when passed to the SBI
/// implementation.
#[inline]
pub unsafe fn ecall5(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    extension_id: usize,
    function_id: usize,
) -> Result<usize, SbiError> {
    let error: isize;
    let value: usize;

    core::arch::asm!(
        "ecall",
        inlateout("a0") arg0 => error,
        inlateout("a1") arg1 => value,
        in("a2") arg2,
        in("a3") arg3,
        in("a4") arg4,
        in("a6") function_id,
        in("a7") extension_id,
    );

    match error {
        0 => Result::Ok(value),
        e => Result::Err(SbiError::new(e)),
    }
}

/// A six-argument `ecall` with the given extension and function IDs.
///
/// # Safety
/// This function is only safe to call if the given function ID accepts six
/// parameters, otherwise the behavior is undefined, as the additional argument
/// registers will have undefined contents when passed to the SBI
/// implementation.
#[inline]
#[allow(clippy::too_many_arguments)]
pub unsafe fn ecall6(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    extension_id: usize,
    function_id: usize,
) -> Result<usize, SbiError> {
    let error: isize;
    let value: usize;

    core::arch::asm!(
        "ecall",
        inlateout("a0") arg0 => error,
        inlateout("a1") arg1 => value,
        in("a2") arg2,
        in("a3") arg3,
        in("a4") arg4,
        in("a5") arg5,
        in("a6") function_id,
        in("a7") extension_id,
    );

    match error {
        0 => Result::Ok(value),
        e => Result::Err(SbiError::new(e)),
    }
}
