// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2023 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{ecall1, ecall3, PhysicalAddress, SbiError};

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
/// [`SbiError::INVALID_PARAMETER`]: The memory region described by the given
///     parameters is not accessible to supervisor mode.
///
/// [`SbiError::DENIED`]: Writing to the debug console is not allowed.
///
/// [`SbiError::FAILED`]: Writing failed due to I/O errors.
#[inline]
#[doc(alias = "sbi_debug_console_write")]
pub unsafe fn write(
    base_addr_lo: PhysicalAddress<u8>,
    base_addr_hi: PhysicalAddress<u8>,
    num_bytes: usize,
) -> Result<usize, SbiError> {
    unsafe {
        ecall3(
            num_bytes,
            base_addr_lo.0 as usize,
            base_addr_hi.0 as usize,
            EXTENSION_ID,
            0,
        )
    }
}

/// A convenience wrapper for `debug_console_write` which takes a single
/// physical slice pointer instead of the manual length and address parameters.
/// This slice ***MUST*** point into physical memory, and any pointers which are
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
/// [`SbiError::INVALID_PARAMETER`]: The memory region described by the given
///     pointer is not accessible to supervisor mode.
///
/// [`SbiError::DENIED`]: Writing to the debug console is not allowed.
///
/// [`SbiError::FAILED`]: Writing failed due to I/O errors.
#[inline]
#[doc(alias = "sbi_debug_console_write")]
pub unsafe fn write_ptr(data: PhysicalAddress<[u8]>) -> Result<usize, SbiError> {
    unsafe {
        write(
            PhysicalAddress::from_ptr(data.as_ptr()),
            PhysicalAddress::new(0),
            data.len(),
        )
    }
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
/// [`SbiError::INVALID_PARAMETER`]: The memory region described by the given
///     parameters is not accessible to supervisor mode.
///
/// [`SbiError::DENIED`]: Reads from the debug console is not allowed.
///
/// [`SbiError::FAILED`]: Reading failed due to I/O errors.
#[inline]
#[doc(alias = "sbi_debug_console_read")]
pub unsafe fn read(
    physical_base_addr_lo: PhysicalAddress<u8>,
    physical_base_addr_hi: PhysicalAddress<u8>,
    num_bytes: usize,
) -> Result<usize, SbiError> {
    unsafe {
        ecall3(
            num_bytes,
            physical_base_addr_lo.0 as usize,
            physical_base_addr_hi.0 as usize,
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
/// [`SbiError::INVALID_PARAMETER`]: The memory region described by the given
///     pointer is not accessible to supervisor mode.
///
/// [`SbiError::DENIED`]: Writing to the debug console is not allowed.
///
/// [`SbiError::FAILED`]: Writing failed due to I/O errors.
#[inline]
#[doc(alias = "sbi_debug_console_read")]
pub unsafe fn read_ptr(buffer: PhysicalAddress<[u8]>) -> Result<usize, SbiError> {
    unsafe {
        read(
            PhysicalAddress::from_ptr(buffer.as_ptr()),
            PhysicalAddress::new(0),
            buffer.len(),
        )
    }
}

/// Write a single byte to the debug console. This call is blocking and will
/// only return after either successfully writing the byte to the debug console
/// or an I/O error occurs.
///
/// ### Possible errors
///
/// [`SbiError::DENIED`]: Writing to the debug console is not allowed.
///
/// [`SbiError::FAILED`]: Writing failed due to I/O errors.
#[inline]
#[doc(alias = "sbi_debug_console_write_byte")]
pub fn write_byte(byte: u8) -> Result<usize, SbiError> {
    unsafe { ecall1(usize::from(byte), EXTENSION_ID, 2) }
}
