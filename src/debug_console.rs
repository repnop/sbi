// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2023 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{ecall1, ecall3, SbiError};
use core::ptr::NonNull;

/// The Debug Console extension ID
pub const EXTENSION_ID: usize = 0x4442434E;

/// Perform a write to the debug console of size `num_bytes` to the given
/// *physical* address specified by `physical_base_addr_lo` and
/// `physical_base_addr_hi`. The return value is the number of bytes written to
/// the debug console.
///
/// This call is non-blocking and may only perform partial or no writes to the
/// debug console if it is unable to accept more data.
///
/// ### Safety
///
/// This function is marked unsafe as it allows arbitrary reads to physical
/// memory which can cause undefined behavior if misused.
///
/// ### Possible errors
///
/// [`SbiError::InvalidParameter`]: The memory region described by the given
///     parameters is not accessible to supervisor mode.
///
/// [`SbiError::Denied`]: Writing to the debug console is not allowed.
///
/// [`SbiError::Failed`]: Writing failed due to I/O errors.
#[inline]
#[doc(alias = "sbi_debug_console_write")]
pub unsafe fn write(
    num_bytes: usize,
    physical_base_addr_lo: usize,
    physical_base_addr_hi: usize,
) -> Result<usize, SbiError> {
    unsafe {
        ecall3(
            num_bytes,
            physical_base_addr_lo,
            physical_base_addr_hi,
            EXTENSION_ID,
            0,
        )
    }
}

/// A convenience wrapper for `debug_console_write` which takes a single
/// non-null slice instead of the manual length and address parameters. This
/// slice ***MUST*** point into physical memory, and any pointers which are
/// virtual pointers that overlap with the physical address space can cause
/// undefined behavior.
///
/// This function is not appropriate to call for platforms where the amount of
/// physical memory can exceed the pointer size.
///
/// ### Safety
///
/// This function is marked unsafe as it allows arbitrary reads to physical
/// memory which can cause undefined behavior if misused.
///
/// ### Possible errors
///
/// [`SbiError::InvalidParameter`]: The memory region described by the given
///     pointer is not accessible to supervisor mode.
///
/// [`SbiError::Denied`]: Writing to the debug console is not allowed.
///
/// [`SbiError::Failed`]: Writing failed due to I/O errors.
#[inline]
#[doc(alias = "sbi_debug_console_write")]
pub unsafe fn write_ptr(data: NonNull<[u8]>) -> Result<usize, SbiError> {
    unsafe { write(data.len(), data.as_ptr().cast::<u8>() as usize, 0) }
}

/// Perform a read from the debug console of size `num_bytes` to the given
/// *physical* address specified by `physical_base_addr_lo` and
/// `physical_base_addr_hi`. The return value is the number of bytes read from
/// the debug console.
///
/// This call is non-blocking and will not perform any writes to memory if there
/// is no data waiting to be read on the debug console.
///
/// ### Safety
///
/// This function is marked unsafe as it allows arbitrary writes to physical
/// memory which can cause undefined behavior if misused.
///
/// ### Possible errors
///
/// [`SbiError::InvalidParameter`]: The memory region described by the given
///     parameters is not accessible to supervisor mode.
///
/// [`SbiError::Denied`]: Reads from the debug console is not allowed.
///
/// [`SbiError::Failed`]: Reading failed due to I/O errors.
#[inline]
#[doc(alias = "sbi_debug_console_read")]
pub unsafe fn read(
    num_bytes: usize,
    physical_base_addr_lo: usize,
    physical_base_addr_hi: usize,
) -> Result<usize, SbiError> {
    unsafe {
        ecall3(
            num_bytes,
            physical_base_addr_lo,
            physical_base_addr_hi,
            EXTENSION_ID,
            1,
        )
    }
}

/// A convenience wrapper for `debug_console_read` which takes a single non-null
/// slice instead of the manual length and address parameters. This slice
/// ***MUST*** point into physical memory, and any pointers which are virtual
/// pointers that overlap with the physical address space can cause undefined
/// behavior.
///
/// This function is not appropriate to call for platforms where the amount of
/// physical memory can exceed the pointer size.
///
/// ### Safety
///
/// This function is marked unsafe as it allows arbitrary writes to physical
/// memory which can cause undefined behavior if misused.
///
/// ### Possible errors
///
/// [`SbiError::InvalidParameter`]: The memory region described by the given
///     pointer is not accessible to supervisor mode.
///
/// [`SbiError::Denied`]: Writing to the debug console is not allowed.
///
/// [`SbiError::Failed`]: Writing failed due to I/O errors.
#[inline]
#[doc(alias = "sbi_debug_console_read")]
pub unsafe fn read_ptr(buffer: NonNull<[u8]>) -> Result<usize, SbiError> {
    unsafe { read(buffer.len(), buffer.as_ptr().cast::<u8>() as usize, 0) }
}

/// Write a single byte to the debug console. This call is blocking and will
/// only return after either successfully writing the byte to the debug console
/// or an I/O error occurs.
///
/// ### Possible errors
///
/// [`SbiError::Denied`]: Writing to the debug console is not allowed.
///
/// [`SbiError::Failed`]: Writing failed due to I/O errors.
#[inline]
#[doc(alias = "sbi_debug_console_write_byte")]
pub fn write_byte(byte: u8) -> Result<usize, SbiError> {
    unsafe { ecall1(usize::from(byte), EXTENSION_ID, 2) }
}
