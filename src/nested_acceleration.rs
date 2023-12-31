// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2023 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{ecall1, ecall3, PhysicalAddress, SbiError};

/// Nested Acceleration extension ID
pub const EXTENSION_ID: usize = 0x4E41434C;

mod sealed {
    pub trait Sealed {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct CsrAddress(u16);

impl CsrAddress {
    pub const fn new(raw: u16) -> Option<Self> {
        match raw {
            0x200..=0x2FF
            | 0x600..=0x67F
            | 0x680..=0x6BF
            | 0x6C0..=0x6FF
            | 0xA00..=0xA7F
            | 0xA80..=0xABF
            | 0xAC0..=0xAFF
            | 0xE00..=0xE7F
            | 0xE80..=0xEBF
            | 0xEC0..=0xEFF => Some(Self(raw)),
            _ => None,
        }
    }

    pub const fn new_unchecked(raw: u16) -> Self {
        Self(raw)
    }
}

pub trait HExtensionCsr: Sized + Copy {
    const ADDRESS: CsrAddress;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct UpdateCsrAddress(u16);

impl From<CsrAddress> for UpdateCsrAddress {
    fn from(value: CsrAddress) -> Self {
        Self(value.0)
    }
}

pub const UPDATE_ALL_CSRS: UpdateCsrAddress = UpdateCsrAddress(u16::MAX);

pub mod csrs {
    use super::CsrAddress;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Hstatus(usize);
    impl super::HExtensionCsr for Hstatus {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x600);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Hedeleg(usize);
    impl super::HExtensionCsr for Hedeleg {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x602);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Hideleg(usize);
    impl super::HExtensionCsr for Hideleg {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x603);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Hie(usize);
    impl super::HExtensionCsr for Hie {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x604);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Hcounteren(usize);
    impl super::HExtensionCsr for Hcounteren {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x606);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Hgeie(usize);
    impl super::HExtensionCsr for Hgeie {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x607);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Htval(usize);
    impl super::HExtensionCsr for Htval {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x643);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Hip(usize);
    impl super::HExtensionCsr for Hip {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x644);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Hvip(usize);
    impl super::HExtensionCsr for Hvip {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x645);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Htinst(usize);
    impl super::HExtensionCsr for Htinst {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x64A);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Hgeip(usize);
    impl super::HExtensionCsr for Hgeip {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0xE12);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Henvcfg(usize);
    impl super::HExtensionCsr for Henvcfg {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x60A);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Henvcfgh(usize);
    impl super::HExtensionCsr for Henvcfgh {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x61A);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Hgatp(usize);
    impl super::HExtensionCsr for Hgatp {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x680);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Hcontext(usize);
    impl super::HExtensionCsr for Hcontext {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x6A8);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Htimedelta(usize);
    impl super::HExtensionCsr for Htimedelta {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x605);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Htimedeltah(usize);
    impl super::HExtensionCsr for Htimedeltah {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x615);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Vsstatus(usize);
    impl super::HExtensionCsr for Vsstatus {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x200);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Vsie(usize);
    impl super::HExtensionCsr for Vsie {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x204);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Vstvec(usize);
    impl super::HExtensionCsr for Vstvec {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x205);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Vsscratch(usize);
    impl super::HExtensionCsr for Vsscratch {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x240);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Vsepc(usize);
    impl super::HExtensionCsr for Vsepc {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x241);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Vscause(usize);
    impl super::HExtensionCsr for Vscause {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x242);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Vstval(usize);
    impl super::HExtensionCsr for Vstval {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x243);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Vsip(usize);
    impl super::HExtensionCsr for Vsip {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x244);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Vsatp(usize);
    impl super::HExtensionCsr for Vsatp {
        const ADDRESS: CsrAddress = CsrAddress::new_unchecked(0x280);
    }
}

#[repr(transparent)]
pub struct Volatile<T: Copy>(T);

pub trait VolatileOps<T: Copy> {
    unsafe fn volatile_read(self) -> T;
    unsafe fn volatile_write(self, value: T);
}

impl<T: Copy> VolatileOps<T> for *mut Volatile<T> {
    unsafe fn volatile_read(self) -> T {
        unsafe { core::ptr::addr_of_mut!((*self).0).read_volatile() }
    }

    unsafe fn volatile_write(self, value: T) {
        unsafe { core::ptr::addr_of_mut!((*self).0).write_volatile(value) }
    }
}

pub trait NaclFeature: sealed::Sealed {
    const ID: u32;
}

#[repr(C, align(4096))]
pub struct SharedMemoryLayout {
    scratch_space: [u8; 4096],
    csr_space: [usize; 128],
}

pub trait CsrSpace {
    fn index<C: HExtensionCsr>(self, csr: C) -> *mut Volatile<C>;
}

#[repr(transparent)]
pub struct SynchronizeCsr([u8; 128]);

const _: () = assert!(core::mem::size_of::<SynchronizeCsr>() == 128);

impl sealed::Sealed for SynchronizeCsr {}
impl NaclFeature for SynchronizeCsr {
    const ID: u32 = 0x00000000;
}

pub trait SynchronizeCsrFeature {
    unsafe fn synchronize_csr(self) -> *mut SynchronizeCsr;
}

impl SynchronizeCsrFeature for *mut SharedMemoryLayout {
    unsafe fn synchronize_csr(self) -> *mut SynchronizeCsr {
        unsafe {
            core::ptr::addr_of_mut!((*self).scratch_space)
                .offset(0x0F80)
                .cast::<SynchronizeCsr>()
        }
    }
}

const NUM_HFENCE_ENTRIES: usize = 1920 / core::mem::size_of::<usize>() / 4;

#[repr(transparent)]
pub struct SynchronizeHfence([[usize; 4]; NUM_HFENCE_ENTRIES]);

const _: () = assert!(core::mem::size_of::<SynchronizeHfence>() == 1920);

impl sealed::Sealed for SynchronizeHfence {}
impl NaclFeature for SynchronizeHfence {
    const ID: u32 = 0x00000001;
}

pub trait SynchronizeHfenceFeature {
    unsafe fn synchronize_hfence(self) -> *mut SynchronizeHfence;
}

impl SynchronizeHfenceFeature for *mut SharedMemoryLayout {
    unsafe fn synchronize_hfence(self) -> *mut SynchronizeHfence {
        unsafe {
            core::ptr::addr_of_mut!((*self).scratch_space)
                .offset(0x0800)
                .cast::<SynchronizeHfence>()
        }
    }
}

const NUM_SRET_ENTRIES: usize = 512 / core::mem::size_of::<usize>();

#[repr(transparent)]
pub struct SynchronizeSret([usize; NUM_SRET_ENTRIES]);

const _: () = assert!(core::mem::size_of::<SynchronizeSret>() == 512);

impl sealed::Sealed for SynchronizeSret {}
impl NaclFeature for SynchronizeSret {
    const ID: u32 = 0x00000002;
}

pub trait SynchronizeSretFeature {
    unsafe fn synchronize_sret(self) -> *mut SynchronizeSret;
}

impl SynchronizeSretFeature for *mut SharedMemoryLayout {
    unsafe fn synchronize_sret(self) -> *mut SynchronizeSret {
        unsafe {
            core::ptr::addr_of_mut!((*self).scratch_space)
                .offset(0x0000)
                .cast::<SynchronizeSret>()
        }
    }
}

const NUM_AUTOSWAP_RESERVED_ENTRIES: usize = 128 / core::mem::size_of::<usize>() - 2;

#[repr(transparent)]
pub struct AutoswapFlags(usize);

#[repr(C)]
pub struct AutoswapCsr {
    autoswap_flags: AutoswapFlags,
    hstatus: csrs::Hstatus,
    _reserved: [usize; NUM_AUTOSWAP_RESERVED_ENTRIES],
}

const _: () = assert!(core::mem::size_of::<AutoswapCsr>() == 128);

impl sealed::Sealed for AutoswapCsr {}
impl NaclFeature for AutoswapCsr {
    const ID: u32 = 0x00000003;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct AutoswapCsrToken(());

pub trait AutoswapCsrFeature {
    unsafe fn autoswap_csr(self) -> *mut AutoswapCsr;
}

impl AutoswapCsrFeature for *mut SharedMemoryLayout {
    unsafe fn autoswap_csr(self) -> *mut AutoswapCsr {
        unsafe {
            core::ptr::addr_of_mut!((*self).scratch_space)
                .offset(0x0200)
                .cast::<AutoswapCsr>()
        }
    }
}

#[doc(alias = "sbi_nacl_probe_feature")]
pub fn probe_feature<F: NaclFeature>() -> Result<bool, SbiError> {
    let value = unsafe { ecall1(F::ID as usize, EXTENSION_ID, 0) }?;

    match value {
        1 => Ok(true),
        _ => Ok(false),
    }
}

#[repr(transparent)]
pub struct Flags(usize);

impl Flags {
    pub const NONE: Self = Self(0);
}

#[doc(alias = "sbi_nacl_set_shmem")]
pub unsafe fn set_shared_memory(
    lo: PhysicalAddress<SharedMemoryLayout>,
    hi: PhysicalAddress<SharedMemoryLayout>,
    flags: Flags,
) -> Result<(), SbiError> {
    unsafe { ecall3(lo.0, hi.0, flags.0, EXTENSION_ID, 1) }.map(drop)
}

pub unsafe fn synchronize_csr<U: Into<UpdateCsrAddress>>(address: U) -> Result<(), SbiError> {
    let UpdateCsrAddress(addr) = address.into();
    unsafe { ecall1(addr as usize, EXTENSION_ID, 2) }.map(drop)
}

pub fn foo() {
    unsafe { synchronize_csr(csrs::Hstatus::ADDRESS) };
}
