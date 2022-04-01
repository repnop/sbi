// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2022 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{ecall2, ecall4, ecall5, HartMask, SbiError};

/// The RFENCE extension ID
pub const EXTENSION_ID: usize = 0x52464E43;

/// Instructs the given harts to execute a `FENCE.I` instruction.
#[inline]
#[doc(alias = "sbi_remote_fence_i")]
pub fn remote_fence_i(hart_mask: HartMask) -> Result<(), SbiError> {
    unsafe { ecall2(hart_mask.mask, hart_mask.base, EXTENSION_ID, 0).map(drop) }
}

/// Instructs the given harts to execute a `SFENCE.VMA` for the region contained
/// by `start_addr` and `size`. `size` is the size in bytes of the memory region
/// for which an `SFENCE.VMA` will be executed.
#[inline]
#[doc(alias = "sbi_remote_sfence_vma")]
pub fn remote_sfence_vma(
    hart_mask: HartMask,
    start_addr: usize,
    size: usize,
) -> Result<(), SbiError> {
    unsafe {
        ecall4(
            hart_mask.mask,
            hart_mask.base,
            start_addr,
            size,
            EXTENSION_ID,
            1,
        )
        .map(drop)
    }
}

/// Instructs the given harts to execute a `SFENCE.VMA` for the region contained
/// by `start_addr` and `size`, only covering the provided ASID. `size` is the
/// size in bytes of the memory region for which an `SFENCE.VMA` will be
/// executed.
#[inline]
#[doc(alias = "sbi_remote_sfence_vma_asid")]
pub fn remote_sfence_vma_asid(
    hart_mask: HartMask,
    start_addr: usize,
    size: usize,
    asid: usize,
) -> Result<(), SbiError> {
    unsafe {
        ecall5(
            hart_mask.mask,
            hart_mask.base,
            start_addr,
            size,
            asid,
            EXTENSION_ID,
            2,
        )
        .map(drop)
    }
}

/// Instructs the given harts to execute a `HFENCE.GVMA` for the region
/// contained by `start_addr` and `size`, only covering the provided VMID. Only
/// valid on harts which support the hypervisor extension. `size` is the size in
/// bytes of the memory region for which an `HFENCE.GVMA` will be executed.
///
/// ### Possible errors
///
/// [`SbiError::NotSupported`]: The function is either unimplemented or the
///     target harts do not implement the hypervisor extension.
#[inline]
#[doc(alias = "sbi_remote_hfence_gvma_vmid")]
pub fn remote_hfence_gvma_vmid(
    hart_mask: HartMask,
    start_addr: usize,
    size: usize,
    vmid: usize,
) -> Result<(), SbiError> {
    unsafe {
        ecall5(
            hart_mask.mask,
            hart_mask.base,
            start_addr,
            size,
            vmid,
            EXTENSION_ID,
            3,
        )
        .map(drop)
    }
}

/// Instructs the given harts to execute a `HFENCE.GVMA` for the region
/// contained by `start_addr` and `size`. Only valid on harts which support the
/// hypervisor extension. `size` is the size in bytes of the memory region for
/// which an `HFENCE.GVMA` will be executed.
///
/// ### Possible errors
///
/// [`SbiError::NotSupported`]: The function is either unimplemented or the
///     target harts do not implement the hypervisor extension.
#[inline]
#[doc(alias = "sbi_remote_hfence_gvma")]
pub fn remote_hfence_gvma(
    hart_mask: HartMask,
    start_addr: usize,
    size: usize,
) -> Result<(), SbiError> {
    unsafe {
        ecall4(
            hart_mask.mask,
            hart_mask.base,
            start_addr,
            size,
            EXTENSION_ID,
            4,
        )
        .map(drop)
    }
}

/// Instructs the given harts to execute a `HFENCE.VVMA` for the region
/// contained by `start_addr` and `size` for the current VMID of the calling
/// hart, and the given ASID. Only valid on harts which support the hypervisor
/// extension. `size` is the size in bytes of the memory region for which an
/// `HFENCE.VVMA` will be executed.
///
/// ### Possible errors
///
/// [`SbiError::NotSupported`]: The function is either unimplemented or the
///     target harts do not implement the hypervisor extension.
#[inline]
#[doc(alias = "sbi_remote_hfence_vvma_asid")]
pub fn remote_hfence_vvma_asid(
    hart_mask: HartMask,
    start_addr: usize,
    size: usize,
    asid: usize,
) -> Result<(), SbiError> {
    unsafe {
        ecall5(
            hart_mask.mask,
            hart_mask.base,
            start_addr,
            size,
            asid,
            EXTENSION_ID,
            5,
        )
        .map(drop)
    }
}

/// Instructs the given harts to execute a `HFENCE.VVMA` for the region
/// contained by `start_addr` and `size` for the current VMID of the calling
/// hart. Only valid on harts which support the hypervisor extension.`size` is
/// the size in bytes of the memory region for which an `HFENCE.VVMA` will be
/// executed.
///
/// ### Possible errors
///
/// [`SbiError::NotSupported`]: The function is either unimplemented or the
///     target harts do not implement the hypervisor extension.
#[inline]
#[doc(alias = "sbi_remote_hfence_vvma")]
pub fn remote_hfence_vvma(
    hart_mask: HartMask,
    start_addr: usize,
    size: usize,
) -> Result<(), SbiError> {
    unsafe {
        ecall4(
            hart_mask.mask,
            hart_mask.base,
            start_addr,
            size,
            EXTENSION_ID,
            6,
        )
        .map(drop)
    }
}
