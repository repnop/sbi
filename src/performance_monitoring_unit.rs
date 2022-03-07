// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2022 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

#![allow(missing_docs)]

use crate::{ecall0, ecall1, ecall5, SbiError};

pub const EXTENSION_ID: usize = 0x504D55;

#[inline]
pub fn num_counters() -> usize {
    unsafe { ecall0(EXTENSION_ID, 0).unwrap() }
}

#[inline]
pub fn get_counter_info(counter_idx: CounterIndex) -> Result<CounterInfo, SbiError> {
    let res = unsafe { ecall1(counter_idx.0, EXTENSION_ID, 1) }?;
    Ok(match (res as isize).is_positive() {
        // Hardware counter
        true => CounterInfo::Hardware {
            csr_number: res & 0xFFF,
            width: (res >> 12) & 0b0011_1111,
        },
        // Firmware counter
        false => CounterInfo::Firmware,
    })
}

pub use self::configure_matching_counters as counter_config_matching;

pub fn configure_matching_counters(
    counter_mask: CounterIndexMask,
    config_flags: CounterConfigurationFlags,
    event_idx: EventIdx,
    event_data: u64,
) -> Result<CounterIndex, SbiError> {
    #[cfg(target_arch = "riscv64")]
    let res = unsafe {
        ecall5(
            counter_mask.base,
            counter_mask.mask,
            config_flags.0,
            event_idx.0,
            event_data as usize,
            EXTENSION_ID,
            2,
        )
    }?;

    #[cfg(target_arch = "riscv32")]
    let res = unsafe {
        crate::ecall6(
            counter_mask.base,
            counter_mask.mask,
            config_flags.0,
            event_idx.0,
            event_data as usize,
            (event_data >> 32) as usize,
            EXTENSION_ID,
            2,
        )
    }?;

    Ok(CounterIndex(res))
}

#[derive(Debug, Clone, Copy)]
pub struct CounterConfigurationFlags(usize);

impl CounterConfigurationFlags {
    /// Skip the counter matching
    pub const SKIP_MATCH: Self = Self(1 << 0);
    /// Clear (or zero) the counter value
    pub const CLEAR_VALUE: Self = Self(1 << 1);
    /// Start the counter after configuring it
    pub const AUTO_START: Self = Self(1 << 2);

    /// Hints to the SBI implementation to inhibit event counting in VU-mode
    pub const SET_VUINH: Self = Self(1 << 3);
    /// More verbose name for [`Self::SET_VUINH`]. Hints to the SBI
    /// implementation to inhibit event counting in VU-mode.
    pub const VU_MODE_INHIBIT: Self = Self::SET_VUINH;

    /// Hints to the SBI implementation to inhibit event counting in VS-mode
    pub const SET_VSINH: Self = Self(1 << 4);
    /// More verbose name for [`Self::SET_VSINH`]. Hints to the SBI
    /// implementation to inhibit event counting in VS-mode.
    pub const VS_MODE_INHIBIT: Self = Self::SET_VSINH;

    /// Hints to the SBI implementation to inhibit event counting in U-mode
    pub const SET_UINH: Self = Self(1 << 5);
    /// More verbose name for [`Self::SET_UINH`]. Hints to the SBI
    /// implementation to inhibit event counting in U-mode.
    pub const U_MODE_INHIBIT: Self = Self::SET_UINH;

    /// Hints to the SBI implementation to inhibit event counting in S-mode
    pub const SET_SINH: Self = Self(1 << 6);
    /// More verbose name for [`Self::SET_SINH`]. Hints to the SBI
    /// implementation to inhibit event counting in S-mode.
    pub const S_MODE_INHIBIT: Self = Self::SET_SINH;

    /// Hints to the SBI implementation to inhibit event counting in M-mode
    pub const SET_MINH: Self = Self(1 << 6);
    /// More verbose name for [`Self::SET_MINH`]. Hints to the SBI
    /// implementation to inhibit event counting in M-mode.
    pub const M_MODE_INHIBIT: Self = Self::SET_MINH;
}

impl core::ops::BitOr for CounterConfigurationFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for CounterConfigurationFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

pub struct CounterIndexMask {
    base: usize,
    mask: usize,
}

impl CounterIndexMask {
    /// Creates a new [`CounterIndexMask`] with a base value of `0` and no
    /// counter indices selected.
    #[inline]
    pub const fn empty() -> Self {
        Self { base: 0, mask: 0 }
    }

    /// Create a new [`CounterIndexMask`] with the given base and no counter
    /// indices selected
    #[inline]
    pub const fn new(base: CounterIndex) -> Self {
        Self {
            base: base.0,
            mask: 0,
        }
    }

    /// Create a new [`CounterIndexMask`] from the given [`CounterIndex`],
    /// making it the base and selecting it
    #[inline]
    pub const fn from(counter_idx: CounterIndex) -> Self {
        Self {
            base: counter_idx.0,
            mask: 1,
        }
    }

    /// Select the given counter index. If `counter_idx` is out of the range of available
    /// selectable counter indices, the [`CounterIndexMask`] is unchanged.
    #[inline]
    #[must_use]
    pub const fn with(mut self, counter_idx: CounterIndex) -> Self {
        if counter_idx.0 >= self.base && counter_idx.0 < (self.base + usize::BITS as usize) {
            self.mask |= 1 << (counter_idx.0 - self.base);
        }

        self
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CounterIndex(usize);

impl CounterIndex {
    pub fn new(idx: usize) -> Self {
        Self(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CounterInfo {
    Hardware { csr_number: usize, width: usize },
    Firmware,
}

mod sealed {
    pub trait Sealed {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct EventIdx(usize);

impl EventIdx {
    pub fn new<T: EventType>(
        #[allow(unused_variables)] event_type: T,
        event_code: <T as EventType>::EventCode,
    ) -> Self {
        Self(((T::TYPE_VALUE & 0b1111) << 16) | (event_code.to_code() as usize))
    }
}

pub trait EventType: sealed::Sealed {
    const TYPE_VALUE: usize;
    type EventCode: EventCode;
}

pub trait EventCode: Sized + sealed::Sealed {
    fn to_code(self) -> u16;
}

#[derive(Debug, Clone, Copy)]
pub struct HardwareGeneralEvent;

impl sealed::Sealed for HardwareGeneralEvent {}
impl EventType for HardwareGeneralEvent {
    const TYPE_VALUE: usize = 0;
    type EventCode = HardwareGeneralEventCode;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum HardwareGeneralEventCode {
    CpuCycles = 1,
    Instructions = 2,
    CacheReferences = 3,
    CacheMisses = 4,
    BranchInstructions = 5,
    BranchMisses = 6,
    BusCycles = 7,
    StalledCyclesFrontend = 8,
    StalledCyclesBackend = 9,
    ReferenceCpuCycles = 10,
}

impl sealed::Sealed for HardwareGeneralEventCode {}
impl EventCode for HardwareGeneralEventCode {
    fn to_code(self) -> u16 {
        self as u16
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HardwareCacheEvent;

impl sealed::Sealed for HardwareCacheEvent {}
impl EventType for HardwareCacheEvent {
    const TYPE_VALUE: usize = 1;
    type EventCode = HardwareCacheEventCode;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct HardwareCacheEventCode(u16);

#[rustfmt::skip]
impl HardwareCacheEventCode {
    pub const LEVEL_1_DATA_READ_ACCESS: Self = Self::new(HardwareCacheEventCodeId::Level1Data, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Access);
    pub const LEVEL_1_DATA_READ_MISS: Self = Self::new(HardwareCacheEventCodeId::Level1Data, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Miss);
    pub const LEVEL_1_DATA_WRITE_ACCESS: Self = Self::new(HardwareCacheEventCodeId::Level1Data, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Access);
    pub const LEVEL_1_DATA_WRITE_MISS: Self = Self::new(HardwareCacheEventCodeId::Level1Data, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Miss);
    pub const LEVEL_1_DATA_PREFETCH_ACCESS: Self = Self::new(HardwareCacheEventCodeId::Level1Data, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Access);
    pub const LEVEL_1_DATA_PREFETCH_MISS: Self = Self::new(HardwareCacheEventCodeId::Level1Data, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Miss);
    
    pub const LEVEL_1_INSTRUCTION_READ_ACCESS: Self = Self::new(HardwareCacheEventCodeId::Level1Instruction, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Access);
    pub const LEVEL_1_INSTRUCTION_READ_MISS: Self = Self::new(HardwareCacheEventCodeId::Level1Instruction, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Miss);
    pub const LEVEL_1_INSTRUCTION_WRITE_ACCESS: Self = Self::new(HardwareCacheEventCodeId::Level1Instruction, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Access);
    pub const LEVEL_1_INSTRUCTION_WRITE_MISS: Self = Self::new(HardwareCacheEventCodeId::Level1Instruction, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Miss);
    pub const LEVEL_1_INSTRUCTION_PREFETCH_ACCESS: Self = Self::new(HardwareCacheEventCodeId::Level1Instruction, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Access);
    pub const LEVEL_1_INSTRUCTION_PREFETCH_MISS: Self = Self::new(HardwareCacheEventCodeId::Level1Instruction, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Miss);

    pub const LAST_LEVEL_READ_ACCESS: Self = Self::new(HardwareCacheEventCodeId::LastLevel, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Access);
    pub const LAST_LEVEL_READ_MISS: Self = Self::new(HardwareCacheEventCodeId::LastLevel, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Miss);
    pub const LAST_LEVEL_WRITE_ACCESS: Self = Self::new(HardwareCacheEventCodeId::LastLevel, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Access);
    pub const LAST_LEVEL_WRITE_MISS: Self = Self::new(HardwareCacheEventCodeId::LastLevel, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Miss);
    pub const LAST_LEVEL_PREFETCH_ACCESS: Self = Self::new(HardwareCacheEventCodeId::LastLevel, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Access);
    pub const LAST_LEVEL_PREFETCH_MISS: Self = Self::new(HardwareCacheEventCodeId::LastLevel, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Miss);

    pub const DATA_TLB_READ_ACCESS: Self = Self::new(HardwareCacheEventCodeId::DataTlb, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Access);
    pub const DATA_TLB_READ_MISS: Self = Self::new(HardwareCacheEventCodeId::DataTlb, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Miss);
    pub const DATA_TLB_WRITE_ACCESS: Self = Self::new(HardwareCacheEventCodeId::DataTlb, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Access);
    pub const DATA_TLB_WRITE_MISS: Self = Self::new(HardwareCacheEventCodeId::DataTlb, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Miss);
    pub const DATA_TLB_PREFETCH_ACCESS: Self = Self::new(HardwareCacheEventCodeId::DataTlb, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Access);
    pub const DATA_TLB_PREFETCH_MISS: Self = Self::new(HardwareCacheEventCodeId::DataTlb, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Miss);

    pub const INSTRUCTION_TLB_READ_ACCESS: Self = Self::new(HardwareCacheEventCodeId::InstructionTlb, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Access);
    pub const INSTRUCTION_TLB_READ_MISS: Self = Self::new(HardwareCacheEventCodeId::InstructionTlb, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Miss);
    pub const INSTRUCTION_TLB_WRITE_ACCESS: Self = Self::new(HardwareCacheEventCodeId::InstructionTlb, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Access);
    pub const INSTRUCTION_TLB_WRITE_MISS: Self = Self::new(HardwareCacheEventCodeId::InstructionTlb, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Miss);
    pub const INSTRUCTION_TLB_PREFETCH_ACCESS: Self = Self::new(HardwareCacheEventCodeId::InstructionTlb, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Access);
    pub const INSTRUCTION_TLB_PREFETCH_MISS: Self = Self::new(HardwareCacheEventCodeId::InstructionTlb, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Miss);

    pub const BRANCH_PREDICTOR_UNIT_READ_ACCESS: Self = Self::new(HardwareCacheEventCodeId::BranchPredictorUnit, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Access);
    pub const BRANCH_PREDICTOR_UNIT_READ_MISS: Self = Self::new(HardwareCacheEventCodeId::BranchPredictorUnit, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Miss);
    pub const BRANCH_PREDICTOR_UNIT_WRITE_ACCESS: Self = Self::new(HardwareCacheEventCodeId::BranchPredictorUnit, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Access);
    pub const BRANCH_PREDICTOR_UNIT_WRITE_MISS: Self = Self::new(HardwareCacheEventCodeId::BranchPredictorUnit, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Miss);
    pub const BRANCH_PREDICTOR_UNIT_PREFETCH_ACCESS: Self = Self::new(HardwareCacheEventCodeId::BranchPredictorUnit, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Access);
    pub const BRANCH_PREDICTOR_UNIT_PREFETCH_MISS: Self = Self::new(HardwareCacheEventCodeId::BranchPredictorUnit, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Miss);

    pub const NUMA_NODE_READ_ACCESS: Self = Self::new(HardwareCacheEventCodeId::NumaNode, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Access);
    pub const NUMA_NODE_READ_MISS: Self = Self::new(HardwareCacheEventCodeId::NumaNode, HardwareCacheEventCodeOperationId::Read, HardwareCacheEventCodeResultId::Miss);
    pub const NUMA_NODE_WRITE_ACCESS: Self = Self::new(HardwareCacheEventCodeId::NumaNode, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Access);
    pub const NUMA_NODE_WRITE_MISS: Self = Self::new(HardwareCacheEventCodeId::NumaNode, HardwareCacheEventCodeOperationId::Write, HardwareCacheEventCodeResultId::Miss);
    pub const NUMA_NODE_PREFETCH_ACCESS: Self = Self::new(HardwareCacheEventCodeId::NumaNode, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Access);
    pub const NUMA_NODE_PREFETCH_MISS: Self = Self::new(HardwareCacheEventCodeId::NumaNode, HardwareCacheEventCodeOperationId::Prefetch, HardwareCacheEventCodeResultId::Miss);

    pub const fn new(
        id: HardwareCacheEventCodeId,
        op: HardwareCacheEventCodeOperationId,
        result: HardwareCacheEventCodeResultId,
    ) -> Self {
        Self(((id as u16) << 3) | ((op as u16) << 1) | (result as u16))
    }
}

impl sealed::Sealed for HardwareCacheEventCode {}
impl EventCode for HardwareCacheEventCode {
    fn to_code(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum HardwareCacheEventCodeId {
    Level1Data = 0,
    Level1Instruction = 1,
    LastLevel = 2,
    DataTlb = 3,
    InstructionTlb = 4,
    BranchPredictorUnit = 5,
    NumaNode = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum HardwareCacheEventCodeOperationId {
    Read = 0,
    Write = 1,
    Prefetch = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum HardwareCacheEventCodeResultId {
    Access = 0,
    Miss = 1,
}

#[derive(Debug, Clone, Copy)]
pub struct HardwareRawEvent;

impl sealed::Sealed for HardwareRawEvent {}
impl EventType for HardwareRawEvent {
    const TYPE_VALUE: usize = 2;
    type EventCode = HardwareRawEventCode;
}

#[derive(Debug, Clone, Copy)]
pub struct HardwareRawEventCode;

impl sealed::Sealed for HardwareRawEventCode {}
impl EventCode for HardwareRawEventCode {
    fn to_code(self) -> u16 {
        0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FirmwareEvent;

impl sealed::Sealed for FirmwareEvent {}
impl EventType for FirmwareEvent {
    const TYPE_VALUE: usize = 0xF;
    type EventCode = FirmwareEventCode;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum FirmwareEventCode {
    MisalignedLoad = 0,
    MisalignedStore = 1,
    AccessLoad = 2,
    AccessStore = 3,
    IllegalInstruction = 4,
    SetTimer = 5,
    IpiSent = 6,
    IpiReceived = 7,
    FenceISent = 8,
    FenceIReceived = 9,
    SfenceVmaSent = 10,
    SfenceVmaReceived = 11,
    SfenceVmaAsidSent = 12,
    SfenceVmaAsidReceived = 13,
    HfenceGvmaSent = 14,
    HfenceGvmaReceived = 15,
    HfenceGvmaVmidSent = 16,
    HfenceGvmaVmidReceived = 17,
    HfenceVvmaSent = 18,
    HfenceVvmaReceived = 19,
    HfenceVvmaAsidSent = 20,
    HfenceVvmaAsidReceived = 21,
}

impl sealed::Sealed for FirmwareEventCode {}
impl EventCode for FirmwareEventCode {
    fn to_code(self) -> u16 {
        self as u16
    }
}
