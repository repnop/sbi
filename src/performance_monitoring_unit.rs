// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2022 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{ecall0, ecall1, SbiError};

/// Performance Monitoring Unit extension ID
pub const EXTENSION_ID: usize = 0x504D55;

/// Returns the number of available performance counters, both hardware and
/// firmware
#[inline]
pub fn num_counters() -> usize {
    unsafe { ecall0(EXTENSION_ID, 0).unwrap() }
}

/// Retreive the information associated with a given performance counter.
///
/// ### Possible errors
///
/// [`SbiError::InvalidParameter`]: The given [`CounterIndex`] is not valid.
#[inline]
#[doc(alias = "counter_get_info", alias = "sbi_pmu_counter_get_info")]
pub fn counter_info(counter_idx: CounterIndex) -> Result<CounterInfo, SbiError> {
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

/// Configure a set of matching performance counters described by the given
/// [`CounterIndexMask`].
///
/// ### Possible errors
///
/// [`SbiError::InvalidParameter`]: One or more of the given counter indices was
///     not valid.
///
/// [`SbiError::NotSupported`]: None of the given counters can monitor the
///     specified event.
#[inline]
#[doc(
    alias = "counter_config_matching",
    alias = "sbi_pmu_counter_config_matching"
)]
pub fn configure_matching_counters(
    counter_mask: CounterIndexMask,
    config_flags: CounterConfigurationFlags,
    event_idx: EventIndex,
    event_data: u64,
) -> Result<CounterIndex, SbiError> {
    #[cfg(target_arch = "riscv64")]
    let res = unsafe {
        crate::ecall5(
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

/// Start the performance counters described by the given [`CounterIndexMask`].
///
/// ### Possible errors
///
/// [`SbiError::InvalidParameter`]: One or more of the counters specified are
///     not valid.
///
/// [`SbiError::AlreadyStarted`]: One or more of the counters specified have
///     already been started.
#[inline]
#[doc(alias = "counter_start", alias = "sbi_pmu_counter_start")]
pub fn start_counters(
    counter_mask: CounterIndexMask,
    start_flags: CounterStartFlags,
    initial_value: u64,
) -> Result<(), SbiError> {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        crate::ecall4(
            counter_mask.base,
            counter_mask.mask,
            start_flags.0,
            initial_value as usize,
            EXTENSION_ID,
            3,
        )
    }?;

    #[cfg(target_arch = "riscv32")]
    unsafe {
        crate::ecall5(
            counter_mask.base,
            counter_mask.mask,
            start_flags.0,
            initial_value as usize,
            (initial_value >> 32) as usize,
            EXTENSION_ID,
            3,
        )
    }?;

    Ok(())
}

/// Stop the performance counters described by the given [`CounterIndexMask`].
///
/// ### Possible errors
///
/// [`SbiError::InvalidParameter`]: One or more of the counters specified are
///     not valid.
///
/// [`SbiError::AlreadyStopped`]: One or more of the counters specified have
///     already been stopped.
#[inline]
#[doc(alias = "counter_stop", alias = "sbi_pmu_counter_stop")]
pub fn stop_counters(
    counter_mask: CounterIndexMask,
    stop_flags: CounterStopFlags,
) -> Result<(), SbiError> {
    unsafe {
        crate::ecall3(
            counter_mask.base,
            counter_mask.mask,
            stop_flags.0,
            EXTENSION_ID,
            4,
        )
        .map(drop)
    }
}

/// Read the current value of the specified [`CounterIndex`].
///
/// ### Possible errors
///
/// [`SbiError::InvalidParameter`]: One or more of the counters specified are
///     not valid.
#[inline]
#[doc(alias = "counter_fw_read", alias = "sbi_pmu_counter_fw_read")]
pub fn read_firmware_counter(counter_idx: CounterIndex) -> Result<usize, SbiError> {
    unsafe { ecall1(counter_idx.0, EXTENSION_ID, 5) }
}

/// Counter configuration flags
#[derive(Debug, Clone, Copy)]
pub struct CounterConfigurationFlags(usize);

impl CounterConfigurationFlags {
    /// No flags
    pub const NONE: Self = Self(0);

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
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for CounterConfigurationFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Default for CounterConfigurationFlags {
    #[inline]
    fn default() -> Self {
        Self::NONE
    }
}

/// Counter start flags
pub struct CounterStartFlags(usize);

impl CounterStartFlags {
    /// No flags
    pub const NONE: Self = Self(0);
    /// Set the initial counter value
    pub const SET_INIT_VALUE: Self = Self(1);
}

impl core::ops::BitOr for CounterStartFlags {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for CounterStartFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Default for CounterStartFlags {
    #[inline]
    fn default() -> Self {
        Self::NONE
    }
}

/// Counter stop flags
pub struct CounterStopFlags(usize);

impl CounterStopFlags {
    /// No flags
    pub const NONE: Self = Self(0);
    /// Reset the counter to event mapping
    pub const RESET: Self = Self(1);
}

impl core::ops::BitOr for CounterStopFlags {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for CounterStopFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Default for CounterStopFlags {
    #[inline]
    fn default() -> Self {
        Self::NONE
    }
}

/// A bitmask of counter indices to be acted upon
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

    /// Select the given counter index. If `counter_idx` is out of the range of
    /// available selectable counter indices, the [`CounterIndexMask`] is
    /// unchanged.
    #[inline]
    #[must_use]
    pub const fn with(mut self, counter_idx: CounterIndex) -> Self {
        if counter_idx.0 >= self.base && counter_idx.0 < (self.base + usize::BITS as usize) {
            self.mask |= 1 << (counter_idx.0 - self.base);
        }

        self
    }
}

/// A logical index assigned to a specific performance counter
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CounterIndex(usize);

impl CounterIndex {
    /// Create a new [`CounterIndex`]
    #[inline]
    pub fn new(idx: usize) -> Self {
        Self(idx)
    }
}

/// Information about a specific performance counter
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CounterInfo {
    /// The counter is a hardware performance counter
    Hardware {
        /// The underlying CSR number backing the performance counter
        csr_number: usize,
        /// The CSR width. Equal to one less than the number of the bits used by
        /// the CSR.
        width: usize,
    },
    /// The counter is a firmware provided performance counter
    Firmware,
}

mod sealed {
    pub trait Sealed {}
}

/// A hardware or firmware event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct EventIndex(usize);

impl EventIndex {
    /// Create a new [`EventIndex`] from the given [`EventType`] and
    /// [`EventCode`]
    #[inline]
    pub fn new<T: EventType>(
        #[allow(unused_variables)] event_type: T,
        event_code: <T as EventType>::EventCode,
    ) -> Self {
        Self(((T::TYPE_VALUE & 0b1111) << 16) | (event_code.to_code() as usize))
    }
}

/// A type of performance monitoring event
#[allow(missing_docs)]
pub trait EventType: sealed::Sealed {
    const TYPE_VALUE: usize;
    type EventCode: EventCode;
}

/// A specific performance monitoring event in an [`EventType`]
#[allow(missing_docs)]
pub trait EventCode: Sized + sealed::Sealed {
    fn to_code(self) -> u16;
}

/// A general hardware performance monitoring event type
#[derive(Debug, Clone, Copy)]
pub struct HardwareGeneralEvent;

impl sealed::Sealed for HardwareGeneralEvent {}
impl EventType for HardwareGeneralEvent {
    const TYPE_VALUE: usize = 0;
    type EventCode = HardwareGeneralEventCode;
}

/// A general hardware performance monitoring event code
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(missing_docs)]
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
    #[inline]
    fn to_code(self) -> u16 {
        self as u16
    }
}

/// A hardware cache performance monitoring event type
#[derive(Debug, Clone, Copy)]
pub struct HardwareCacheEvent;

impl sealed::Sealed for HardwareCacheEvent {}
impl EventType for HardwareCacheEvent {
    const TYPE_VALUE: usize = 1;
    type EventCode = HardwareCacheEventCode;
}

/// A hardware cache performance monitoring event code
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct HardwareCacheEventCode(u16);

#[rustfmt::skip]
#[allow(missing_docs)]
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

    /// Create a new [`HardwareCacheEventCode`] from the cache unit, operation,
    /// and result to monitor
    #[inline]
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
    #[inline]
    fn to_code(self) -> u16 {
        self.0
    }
}

/// The hardware cache unit to monitor
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum HardwareCacheEventCodeId {
    /// First level data cache
    Level1Data = 0,
    /// First level instruction cache
    Level1Instruction = 1,
    /// Last level cache
    LastLevel = 2,
    /// Data translation lookaside buffer cache
    DataTlb = 3,
    /// Instruction translation lookaside buffer cache
    InstructionTlb = 4,
    #[allow(missing_docs)]
    BranchPredictorUnit = 5,
    /// Non-uniform memory access node cache
    NumaNode = 6,
}

/// The cache operation to monitor
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(missing_docs)]
#[repr(u16)]
pub enum HardwareCacheEventCodeOperationId {
    Read = 0,
    Write = 1,
    Prefetch = 2,
}

/// The result of the caching operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(missing_docs)]
#[repr(u16)]
pub enum HardwareCacheEventCodeResultId {
    Access = 0,
    Miss = 1,
}

/// A raw hardware performance monitoring event
#[derive(Debug, Clone, Copy, Default)]
pub struct HardwareRawEvent;

impl sealed::Sealed for HardwareRawEvent {}
impl EventType for HardwareRawEvent {
    const TYPE_VALUE: usize = 2;
    type EventCode = HardwareRawEventCode;
}

/// A raw hardware performance monitoring event code
#[derive(Debug, Clone, Copy, Default)]
pub struct HardwareRawEventCode;

impl sealed::Sealed for HardwareRawEventCode {}
impl EventCode for HardwareRawEventCode {
    #[inline]
    fn to_code(self) -> u16 {
        0
    }
}

/// A firmware performance monitoring event type
#[derive(Debug, Clone, Copy)]
pub struct FirmwareEvent;

impl sealed::Sealed for FirmwareEvent {}
impl EventType for FirmwareEvent {
    const TYPE_VALUE: usize = 0xF;
    type EventCode = FirmwareEventCode;
}

/// Firmware performance monitoring event metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
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
    #[inline]
    fn to_code(self) -> u16 {
        self as u16
    }
}
