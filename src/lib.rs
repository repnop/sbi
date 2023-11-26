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
/// Collaborative Processor Performance Control
pub mod collaborative_processor_performance_control;
/// Debug Console extension
pub mod debug_console;
/// Hart State Management extension
pub mod hart_state_management;
/// IPI extension
pub mod ipi;
/// Legacy SBI calls
pub mod legacy;
/// Nested Acceleration extension
pub mod nested_acceleration;
/// Performance Monitoring Unit extension
pub mod performance_monitoring_unit;
/// RFENCE extension
pub mod rfence;
/// System Reset extension
pub mod system_reset;
/// System Suspend extension
pub mod system_suspend;
/// Timer extension
pub mod timer;

use core::{num::NonZeroIsize, ptr::NonNull};

/// A convenience alias to the [`collaborative_processor_performance_control`] module.
pub use collaborative_processor_performance_control as cbbc;
/// A convenience alias to the [`hart_state_management`] module.
pub use hart_state_management as hsm;
/// A convenience alias to the [`performance_monitoring_unit`] module;
pub use performance_monitoring_unit as pmu;

/// Error codes returned by SBI calls
///
/// For all of the various error codes, see the associated constants on this type, such as [`SbiError::FAILED`]
///
/// Implementation note: This error type is not represented by a proper `enum`
/// so that constructing it based on the returned integer code does not require
/// panicking in the event that new error codes are added to the specification.
/// Using associated constants also works to emulate `#[non_exhaustive]` since
/// it is not possible to publically construct this type, so that any new errors
/// won't cause compilation errors in code attempting to handle all errors.
/// (though that should be pretty uncommon)
///
/// note: `SBI_SUCCESS` is not represented here since this is to be used as the
/// error type in a `Result`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SbiError(Option<NonZeroIsize>);

impl SbiError {
    /// The SBI call failed
    pub const FAILED: Self = Self(unsafe { Some(NonZeroIsize::new_unchecked(-1)) });
    /// The SBI call is not implemented or the functionality is not available
    pub const NOT_SUPPORTED: Self = Self(unsafe { Some(NonZeroIsize::new_unchecked(-2)) });
    /// An invalid parameter was passed
    pub const INVALID_PARAMETER: Self = Self(unsafe { Some(NonZeroIsize::new_unchecked(-3)) });
    /// The SBI implementation has denied execution of the call functionality
    pub const DENIED: Self = Self(unsafe { Some(NonZeroIsize::new_unchecked(-4)) });
    /// An invalid address was passed
    pub const INVALID_ADDRESS: Self = Self(unsafe { Some(NonZeroIsize::new_unchecked(-5)) });
    /// The resource is already available
    pub const ALREADY_AVAILABLE: Self = Self(unsafe { Some(NonZeroIsize::new_unchecked(-6)) });
    /// The resource was previously started
    pub const ALREADY_STARTED: Self = Self(unsafe { Some(NonZeroIsize::new_unchecked(-7)) });
    /// The resource was previously stopped
    pub const ALREADY_STOPPED: Self = Self(unsafe { Some(NonZeroIsize::new_unchecked(-8)) });
    /// Shared memory is unavailable
    pub const SHARED_MEMORY_UNAVAILABLE: Self =
        Self(unsafe { Some(NonZeroIsize::new_unchecked(-9)) });
}

impl SbiError {
    #[inline]
    fn new(n: isize) -> Self {
        match n {
            n if n.is_negative() => Self(Some(unsafe { NonZeroIsize::new_unchecked(n) })),
            _ => Self(None),
        }
    }
}

impl core::fmt::Display for SbiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                SbiError::ALREADY_AVAILABLE => "resource is already available",
                SbiError::DENIED => "SBI implementation denied execution",
                SbiError::FAILED => "call to SBI failed",
                SbiError::INVALID_ADDRESS => "invalid address passed",
                SbiError::INVALID_PARAMETER => "invalid parameter passed",
                SbiError::NOT_SUPPORTED =>
                    "SBI call not implemented or functionality not available",
                SbiError::ALREADY_STARTED => "resource was already started",
                SbiError::ALREADY_STOPPED => "resource was already stopped",
                _ => "unknown error",
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

/// A value restricted to a given range
#[repr(transparent)]
pub struct RestrictedRange<const MIN: u32, const MAX: u32>(u32);

impl<const MIN: u32, const MAX: u32> RestrictedRange<MIN, MAX> {
    /// Create a new [`RestrictedRange`] value
    ///
    /// ## Panics
    ///
    /// This function will panic if the provided value is outside of the range
    /// of the type.
    pub const fn new(value: u32) -> Self {
        if value < MIN || value > MAX {
            panic!("invalid value supplied to `PlatformSpecific::new`")
        }

        Self(value)
    }
}

impl<const MIN: u32, const MAX: u32> From<RestrictedRange<MIN, MAX>> for u32 {
    fn from(value: RestrictedRange<MIN, MAX>) -> Self {
        value.0
    }
}

impl<const MIN: u32, const MAX: u32> Clone for RestrictedRange<MIN, MAX> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<const MIN: u32, const MAX: u32> Copy for RestrictedRange<MIN, MAX> {}

impl<const MIN: u32, const MAX: u32> core::fmt::Debug for RestrictedRange<MIN, MAX> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "RestrictedRange<MIN={MIN:#X}, MAX={MAX:#X}>({:#X})",
            self.0
        )
    }
}

/// Representation of a physical address
#[repr(transparent)]
pub struct PhysicalAddress<T>(usize, core::marker::PhantomData<*mut T>);

impl<T> PhysicalAddress<T> {
    /// Create a new [`PhysicalAddress`] from the raw integer value
    pub fn new(value: usize) -> Self {
        Self(value, core::marker::PhantomData)
    }

    /// Create a new [`PhysicalAddress`] from a pointer value
    pub fn from_ptr(ptr: *mut T) -> Self {
        Self(ptr as usize, core::marker::PhantomData)
    }
}

impl<T> From<*mut T> for PhysicalAddress<T> {
    fn from(value: *mut T) -> Self {
        Self::from_ptr(value)
    }
}

impl<T> From<NonNull<T>> for PhysicalAddress<T> {
    fn from(value: NonNull<T>) -> Self {
        Self::from_ptr(value.as_ptr())
    }
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
