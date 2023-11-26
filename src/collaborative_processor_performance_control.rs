// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2023 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{ecall1, SbiError};

/// Collaborative Processor Performance Control extension ID
pub const EXTENSION_ID: usize = 0x43505043;

#[doc(hidden)]
pub trait CastRegisterValue: Sized + Copy {
    fn cast(from: usize) -> Self;
    fn reverse_cast(self) -> usize;
    fn hi_lo(self) -> (usize, usize);
}

impl CastRegisterValue for u64 {
    fn cast(from: usize) -> Self {
        from as u64
    }

    fn reverse_cast(self) -> usize {
        self as usize
    }

    fn hi_lo(self) -> (usize, usize) {
        ((self >> 32) as usize, (self & 0xFFFF_FFFF) as usize)
    }
}

impl CastRegisterValue for u32 {
    fn cast(from: usize) -> Self {
        from as u32
    }

    fn reverse_cast(self) -> usize {
        self as usize
    }

    fn hi_lo(self) -> (usize, usize) {
        (0, self as usize)
    }
}

/// A CPPC register
pub trait Register {
    /// Register ID
    const ID: u32;
    /// Register value width
    type Width: CastRegisterValue;
}

/// A register that can be read from
pub trait Readable: Register {}
/// A register that can be written to
pub trait Writable: Register {}

/// CPPC registers defined by the SBI specification
pub mod registers {
    use super::{Readable, Register, Writable};

    /// ACPI Specification 6.5; 8.4.6.1.1 Highest Performance
    ///
    /// Highest performance is the absolute maximum performance an individual
    /// processor may reach, assuming ideal conditions. This performance level
    /// may not be sustainable for long durations, and may only be achievable if
    /// other platform components are in a specific state; for example, it may
    /// require other processors be in an idle state.
    ///
    /// Notify events of type 0x85 to the processor device object cause OSPM to
    /// re-evaluate the Highest Performance Register, but only when it is
    /// encoded as a buffer. Note: OSPM will not re-evaluate the _CPC object as
    /// a result of the notification.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct HighestPerformance;

    impl Readable for HighestPerformance {}
    impl Register for HighestPerformance {
        const ID: u32 = 0x00000000;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.1.2 Nominal Performance
    ///
    /// Nominal Performance is the maximum sustained performance level of the
    /// processor, assuming ideal operating conditions. In absence of an
    /// external constraint (power, thermal, etc.) this is the performance level
    /// the platform is expected to be able to maintain continuously. All
    /// processors are expected to be able to sustain their nominal performance
    /// state simultaneously.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NominalPerformance;

    impl Readable for NominalPerformance {}
    impl Register for NominalPerformance {
        const ID: u32 = 0x00000001;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.1.4 Lowest Nonlinear Performance
    ///
    /// Lowest Nonlinear Performance is the lowest performance level at which
    /// nonlinear power savings are achieved, for example, due to the combined
    /// effects of voltage and frequency scaling. Above this threshold, lower
    /// performance levels should be generally more energy efficient than higher
    /// performance levels. In traditional terms, this represents the P-state
    /// range of performance levels.
    ///
    /// This register effectively conveys the most efficient performance level
    /// to OSPM.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LowestNonlinearPerformance;

    impl Readable for LowestNonlinearPerformance {}
    impl Register for LowestNonlinearPerformance {
        const ID: u32 = 0x00000002;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.1.5 Lowest Performance
    ///
    /// Lowest Performance is the absolute lowest performance level of the
    /// platform. Selecting a performance level lower than the lowest nonlinear
    /// performance level may actually cause an efficiency penalty, but should
    /// reduce the instantaneous power consumption of the processor. In
    /// traditional terms, this represents the T-state range of performance
    /// levels.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LowestPerformance;

    impl Readable for LowestPerformance {}
    impl Register for LowestPerformance {
        const ID: u32 = 0x00000003;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.1.6 Guaranteed Performance
    ///
    /// Guaranteed Performance Register conveys to OSPM a Guaranteed Performance
    /// level, which is the current maximum sustained performance level of a
    /// processor, taking into account all known external constraints (power
    /// budgeting, thermal constraints, AC vs DC power source, etc.). All
    /// processors are expected to be able to sustain their guaranteed
    /// performance levels simultaneously. The guaranteed performance level is
    /// required to fall in the range \[Lowest Performance, Nominal
    /// performance], inclusive.
    ///
    /// If this register is not implemented, OSPM assumes guaranteed performance
    /// is always equal to nominal performance.
    ///
    /// Notify events of type 0x83 to the processor device object will cause
    /// OSPM to re-evaluate the Guaranteed Performance Register. Changes to
    /// guaranteed performance should not be more frequent than once per second.
    /// If the platform is not able to guarantee a given performance level for a
    /// sustained period of time (greater than one second), it should guarantee
    /// a lower performance level and opportunistically enter the higher
    /// performance level as requested by OSPM and allowed by current operating
    /// conditions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct GuaranteedPerformance;

    impl Readable for GuaranteedPerformance {}
    impl Register for GuaranteedPerformance {
        const ID: u32 = 0x00000004;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.2.3 Desired Performance
    ///
    /// When Autonomous Selection is disabled, the Desired Performance Register
    /// is required and conveys the performance level OSPM is requesting from
    /// the platform. Desired performance may be set to any performance value in
    /// the range \[Minimum Performance, Maximum Performance], inclusive.
    /// Desired performance may take one of two meanings, depending on whether
    /// the desired performance is above or below the guaranteed performance
    /// level.
    ///
    /// - Below the guaranteed performance level, desired performance expresses
    /// the average performance level the platform must provide subject to the
    /// Performance Reduction Tolerance.
    /// - Above the guaranteed performance level, the platform must provide the
    /// guaranteed performance level. The platform should attempt to provide up
    /// to the desired performance level, if current operating conditions allow
    /// for it, but it is not required to do so
    ///
    /// When Autonomous Selection is enabled, it is not necessary for OSPM to
    /// assess processor workload performance demand and convey a corresponding
    /// performance delivery request to the platform via the Desired Register.
    /// If the Desired Performance Register exists, OSPM may provide an explicit
    /// performance requirement hint to the platform by writing a non-zero
    /// value. In this case, the delivered performance is not bounded by the
    /// Performance Reduction Tolerance Register, however, OSPM can influence
    /// the delivered performance by writing appropriate values to the Energy
    /// Performance Preference Register. Writing a zero value to the Desired
    /// Performance Register or the non-existence of the Desired Performance
    /// Register causes the platform to autonomously select a performance level
    /// appropriate to the current workload.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DesiredPerformance;

    impl Readable for DesiredPerformance {}
    impl Writable for DesiredPerformance {}
    impl Register for DesiredPerformance {
        const ID: u32 = 0x00000005;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.2.2 Minimum Performance
    ///
    /// The Minimum Performance Register allows OSPM to convey the minimum
    /// performance level at which the platform may run. Minimum performance may
    /// be set to any performance value in the range \[Lowest Performance,
    /// Highest Performance], inclusive but must be set to a value that is less
    /// than or equal to that specified by the Maximum Performance Register.
    ///
    /// In the presence of a physical constraint, for example a thermal
    /// excursion, the platform may not be able to successfully maintain minimum
    /// performance in accordance with that set via the Minimum Performance
    /// Register. In this case, the platform issues a Notify event of type 0x84
    /// to the processor device object and sets the Minimum_Excursion bit within
    /// the Performance Limited Register.
    ///
    /// The platform must implement either both the Minimum Performance and
    /// Maximum Performance registers or neither register. If neither register
    /// is implemented and Autonomous Selection is disabled, the platform must
    /// always deliver the desired performance.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MinimumPerformance;

    impl Readable for MinimumPerformance {}
    impl Writable for MinimumPerformance {}
    impl Register for MinimumPerformance {
        const ID: u32 = 0x00000006;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.2.1 Maximum Performance
    ///
    /// Maximum Performance Register conveys the maximum performance level at
    /// which the platform may run. Maximum performance may be set to any
    /// performance value in the range \[Lowest Performance, Highest
    /// Performance], inclusive.
    ///
    /// The value written to the Maximum Performance Register conveys a request
    /// to limit maximum performance for the purpose of energy efficiency or
    /// thermal control and the platform limits its performance accordingly as
    /// possible. However, the platform may exceed the requested limit in the
    /// event it is necessitated by internal package optimization. For example,
    /// hardware coordination among multiple logical processors with
    /// interdependencies.
    ///
    /// OSPM’s use of this register to limit performance for the purpose of
    /// thermal control must comprehend multiple logical processors with
    /// interdependencies. i.e. the same value must be written to all processors
    /// within a domain to achieve the desired result.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MaximumPerformance;

    impl Readable for MaximumPerformance {}
    impl Writable for MaximumPerformance {}
    impl Register for MaximumPerformance {
        const ID: u32 = 0x00000007;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.2.4 Performance Reduction Tolerance
    ///
    /// The Performance Reduction Tolerance Register is used by OSPM to convey
    /// the deviation below the Desired Performance that is tolerable. It is
    /// expressed by OSPM as an absolute value on the performance scale.
    /// Performance Tolerance must be less than or equal to the Desired
    /// Performance. If the platform supports the Time Window Register, the
    /// Performance Reduction Tolerance conveys the minimal performance value
    /// that may be delivered on average over the Time Window. If this register
    /// is not implemented, the platform must assume Performance Reduction
    /// Tolerance = Desired Performance.
    ///
    /// When Autonomous Selection is enabled, values written to the Performance
    /// Reduction Tolerance Register are ignored.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PerformanceReductionTolerance;

    impl Readable for PerformanceReductionTolerance {}
    impl Writable for PerformanceReductionTolerance {}
    impl Register for PerformanceReductionTolerance {
        const ID: u32 = 0x00000008;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.2.5 Time Window
    ///
    /// When Autonomous Selection is not enabled, OSPM may write a value to the
    /// Time Window Register to indicate a time window over which the platform
    /// must provide the desired performance level (subject to the Performance
    /// Reduction Tolerance). OSPM sets the time window when electing a new
    /// desired performance The time window represents the minimum time duration
    /// for OSPM’s evaluation of the platform’s delivered performance (see
    /// Performance Counters “Performance Counters” for details on how OSPM
    /// computes delivered performance). If OSPM evaluates delivered performance
    /// over an interval smaller than the specified time window, it has no
    /// expectations of the performance delivered by the platform. For any
    /// evaluation interval equal to or greater than the time window, the
    /// platform must deliver the OSPM desired performance within the specified
    /// tolerance bound.
    ///
    /// If OSPM specifies a time window of zero or if the platform does not
    /// support the time window register, the platform must deliver performance
    /// within the bounds of Performance Reduction Tolerance irrespective of the
    /// duration of the evaluation interval.
    ///
    /// When Autonomous Selection is enabled, values written to the Time Window
    /// Register are ignored. Reads of the Time Window register indicate minimum
    /// length of time (in ms) between successive reads of the platform’s
    /// performance counters. If the Time Window register is not supported then
    /// there is no minimum time requirement between successive reads of the
    /// platform’s performance counters.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TimeWindow;

    impl Readable for TimeWindow {}
    impl Writable for TimeWindow {}
    impl Register for TimeWindow {
        const ID: u32 = 0x00000009;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.3.1 Performance Counters; Counter Wraparound Time
    ///
    /// Counter Wraparound Time provides a means for the platform to specify a
    /// rollover time for the Reference/Delivered performance counters. If
    /// greater than this time period elapses between OSPM querying the feedback
    /// counters, the counters may wrap without OSPM being able to detect that
    /// they have done so. If not implemented (or zero), the performance
    /// counters are assumed to never wrap during the lifetime of the platform.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CounterWraparoundTime;

    impl Readable for CounterWraparoundTime {}
    impl Register for CounterWraparoundTime {
        const ID: u32 = 0x0000000A;
        type Width = u64;
    }

    /// ACPI Specification 6.5; 8.4.6.1.3.1 Performance Counters; Reference Performance Counter
    ///
    /// The Reference Performance Counter Register counts at a fixed rate any
    /// time the processor is active. It is not affected by changes to Desired
    /// Performance, processor throttling, etc. If Reference Performance is
    /// supported, the Reference Performance Counter accumulates at a rate
    /// corresponding to the Reference Performance level. Otherwise, the
    /// Reference Performance Counter accumulates at the Nominal performance
    /// level.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ReferencePerformanceCounter;

    impl Readable for ReferencePerformanceCounter {}
    impl Register for ReferencePerformanceCounter {
        const ID: u32 = 0x0000000B;
        type Width = u64;
    }

    /// ACPI Specification 6.5; 8.4.6.1.3.1 Performance Counters; Delivered Performance Counter
    ///
    /// The Delivered Performance Counter Register increments any time the
    /// processor is active, at a rate proportional to the current performance
    /// level, taking into account changes to Desired Performance. When the
    /// processor is operating at its reference performance level, the delivered
    /// performance counter must increment at the same rate as the reference
    /// performance counter.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DeliveredPerformanceCounter;

    impl Readable for DeliveredPerformanceCounter {}
    impl Register for DeliveredPerformanceCounter {
        const ID: u32 = 0x0000000C;
        type Width = u64;
    }

    /// ACPI Specification 6.5; 8.4.6.1.3.2 Performance Limited Register
    ///
    /// In the event that the platform constrains the delivered performance to
    /// less than the minimum performance or the desired performance (or, less
    /// than the guaranteed performance, if desired performance is greater than
    /// guaranteed performance) due to an unpredictable event, the platform sets
    /// the performance limited indicator to a non-zero value. This indicates to
    /// OSPM that an unpredictable event has limited processor performance, and
    /// the delivered performance may be less than desired / minimum
    /// performance. If the platform does not support signaling performance
    /// limited events, this register is permitted to always return zero when
    /// read.
    ///
    /// | Bit | Name              | Description                                                                                                                                                                                                                                                      |
    /// |-----|-------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
    /// | 0   | Desired_Excursion | Set when Delivered Performance has been constrained to less than Desired Performance (or, less than the guaranteed performance, if desired performance is greater than guaranteed performance). This bit is not utilized when Autonomous Selection is enabled. |
    /// | 1   | Minimum_Excursion | Set when Delivered Performance has been constrained to less than Minimum Performance                                                                                                                                                                             |
    /// | 2-n | Reserved          | Reserved                                                                                                                                                                                                                                                         |
    ///
    /// Bits within the Performance Limited Register are sticky, and will remain
    /// non-zero until OSPM clears the bit. The platform should only issue a
    /// Notify when Minimum Excursion transitions from 0 to 1 to avoid repeated
    /// events when there is sustained or recurring limiting but OSPM has not
    /// cleared the previous indication.
    ///
    /// The performance limited register should only be used to report short
    /// term, unpredictable events (e.g., PROCHOT being asserted). If the
    /// platform is capable of identifying longer term, predictable events that
    /// limit processor performance, it should use the guaranteed performance
    /// register to notify OSPM of this limitation. Changes to guaranteed
    /// performance should not be more frequent than once per second. If the
    /// platform is not able to guarantee a given performance level for a
    /// sustained period of time (greater than one second), it should guarantee
    /// a lower performance level and opportunistically enter the higher
    /// performance level as requested by OSPM and allowed by current operating
    /// conditions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PerformanceLimited;

    impl Readable for PerformanceLimited {}
    impl Writable for PerformanceLimited {}
    impl Register for PerformanceLimited {
        const ID: u32 = 0x0000000D;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.4 CPPC Enable Register
    ///
    /// If supported by the platform, OSPM writes a one to this register to
    /// enable CPPC on this processor.
    ///
    /// If not implemented, OSPM assumes the platform always has CPPC enabled.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CppcEnable;

    impl Readable for CppcEnable {}
    impl Writable for CppcEnable {}
    impl Register for CppcEnable {
        const ID: u32 = 0x0000000E;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.5 Autonomous Selection Enable Register
    ///
    /// If supported by the platform, OSPM writes a one to this register to
    /// enable Autonomous Performance Level Selection on this processor. CPPC
    /// must be enabled via the CPPC Enable Register to enable Autonomous
    /// Performance Level Selection. Platforms that exclusively support
    /// Autonomous Selection must populate this field as an Integer with a value
    /// of 1.
    ///
    /// When Autonomous Selection is enabled, the platform is responsible for
    /// selecting performance states. OSPM is not required to assess processor
    /// workload performance demand and convey a corresponding performance
    /// delivery request to the platform via the Desired Performance Register.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AutonomousSelectionEnable;

    impl Readable for AutonomousSelectionEnable {}
    impl Writable for AutonomousSelectionEnable {}
    impl Register for AutonomousSelectionEnable {
        const ID: u32 = 0x0000000F;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.6 Autonomous Activity Window Register
    ///
    /// If supported by the platform, OSPM may write a time value (10^3-bit exp
    /// * 7-bit mantissa in 1μsec units: 1us to 1270 sec) to this field to
    /// indicate a moving utilization sensitivity window to the platform’s
    /// autonomous selection policy. Combined with the Energy Performance
    /// Preference Register value, the Activity Window influences the rate of
    /// performance increase / decrease of the platform’s autonomous selection
    /// policy. OSPM writes a zero value to this register to enable the platform
    /// to determine an appropriate Activity Window depending on the workload.
    ///
    /// Writes to this register only have meaning when Autonomous Selection is
    /// enabled.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AutonomousAcivityWindow;

    impl Readable for AutonomousAcivityWindow {}
    impl Writable for AutonomousAcivityWindow {}
    impl Register for AutonomousAcivityWindow {
        const ID: u32 = 0x00000010;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.7 Energy Performance Preference Register
    ///
    /// If supported by the platform, OSPM may write a range of values from 0
    /// (performance preference) to 0xFF (energy efficiency preference) that
    /// influences the rate of performance increase /decrease and the result of
    /// the hardware’s energy efficiency and performance optimization
    /// policies.This provides a means for OSPM to limit the energy efficiency
    /// impact of the platform’s performance-related optimizations / control
    /// policy and the performance impact of the platform’s energy
    /// efficiency-related optimizations / control policy.
    ///
    /// Writes to this register only have meaning when Autonomous Selection is
    /// enabled.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EnergyPerformancePreference;

    impl Readable for EnergyPerformancePreference {}
    impl Writable for EnergyPerformancePreference {}
    impl Register for EnergyPerformancePreference {
        const ID: u32 = 0x00000011;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.1.3 Reference Performance
    ///
    /// If supported by the platform, Reference Performance is the rate at which
    /// the Reference Performance Counter increments. If not implemented (or
    /// zero), the Reference Performance Counter increments at a rate
    /// corresponding to the Nominal Performance level.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ReferencePerformance;

    impl Readable for ReferencePerformance {}
    impl Register for ReferencePerformance {
        const ID: u32 = 0x00000012;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.1.5 Lowest Performance
    ///
    /// Lowest Performance is the absolute lowest performance level of the
    /// platform. Selecting a performance level lower than the lowest nonlinear
    /// performance level may actually cause an efficiency penalty, but should
    /// reduce the instantaneous power consumption of the processor. In
    /// traditional terms, this represents the T-state range of performance
    /// levels.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LowestFrequency;

    impl Readable for LowestFrequency {}
    impl Register for LowestFrequency {
        const ID: u32 = 0x00000013;
        type Width = u32;
    }

    /// ACPI Specification 6.5; 8.4.6.1.1.2 Nominal Performance
    ///
    /// Nominal Performance is the maximum sustained performance level of the
    /// processor, assuming ideal operating conditions. In absence of an
    /// external constraint (power, thermal, etc.) this is the performance level
    /// the platform is expected to be able to maintain continuously. All
    /// processors are expected to be able to sustain their nominal performance
    /// state simultaneously.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NominalFrequency;

    impl Readable for NominalFrequency {}
    impl Register for NominalFrequency {
        const ID: u32 = 0x00000014;
        type Width = u32;
    }

    /// Provides the maximum (worst-case) performance state transition latency
    /// in nanoseconds.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TransitionLatency;

    impl Readable for TransitionLatency {}
    impl Register for TransitionLatency {
        const ID: u32 = 0x80000000;
        type Width = u32;
    }
}

/// Probe whether the given CPPC register is supported. On success, this
/// function returns the width of the register in bits, if the register is
/// implemented.
///
/// ### Possible errors
///
/// [`SbiError::INVALID_PARAMETER`]: The register ID is a reserved ID.
///
/// [`SbiError::FAILED`]: The probe request failed for unspecified or unknown
///     reasons.
#[doc(alias = "sbi_cppc_probe")]
pub fn probe_register<R: Register>(
    #[allow(unused_variables)] register: R,
) -> Result<Option<usize>, SbiError> {
    let ret = unsafe { ecall1(R::ID as usize, EXTENSION_ID, 0) }?;

    match ret {
        0 => Ok(None),
        _ => Ok(Some(ret)),
    }
}

/// Read the value of a CPPC register. When `XLEN` is 32, this value only
/// contains the lower 32 bits of the full register value, and a subsequent call
/// to [`read_register_hi`] is required to read the full value if the register
/// size is >32 bits. When `XLEN` is >= 64, no further calls are required.
///
/// ### Possible errors
///
/// [`SbiError::INVALID_PARAMETER`]: The register ID is a reserved ID.
///
/// [`SbiError::NOT_SUPPORTED`]: The register is not implemented by the platform.
///
/// [`SbiError::DENIED`]: The register is write-only.
///
/// [`SbiError::FAILED`]: The read request failed for unspecified or unknown
///     reasons.
#[doc(alias = "sbi_cppc_read")]
pub fn read_register<R: Readable>(
    #[allow(unused_variables)] register: R,
) -> Result<R::Width, SbiError> {
    unsafe { ecall1(R::ID as usize, EXTENSION_ID, 1) }.map(<R::Width as CastRegisterValue>::cast)
}

/// Read the upper 32 bits of the register value. When `XLEN` >= 64, this
/// function will always return `0` for valid register IDs.
///
/// ### Possible errors
///
/// [`SbiError::INVALID_PARAMETER`]: The register ID is a reserved ID.
///
/// [`SbiError::NOT_SUPPORTED`]: The register is not implemented by the platform.
///
/// [`SbiError::DENIED`]: The register is write-only.
///
/// [`SbiError::FAILED`]: The read request failed for unspecified or unknown
///     reasons.
#[doc(alias = "sbi_cppc_read_hi")]
pub fn read_register_hi<R: Readable>(
    #[allow(unused_variables)] register: R,
) -> Result<R::Width, SbiError> {
    unsafe { ecall1(R::ID as usize, EXTENSION_ID, 2) }.map(<R::Width as CastRegisterValue>::cast)
}

/// Write a value to the specified CPPC register.
///
/// ### Possible errors
///
/// [`SbiError::INVALID_PARAMETER`]: The register ID is a reserved ID.
///
/// [`SbiError::NOT_SUPPORTED`]: The register is not implemented by the platform.
///
/// [`SbiError::DENIED`]: The register is read-only.
///
/// [`SbiError::FAILED`]: The write request failed for unspecified or unknown
///     reasons.
#[doc(alias = "sbi_cppc_write")]
pub fn write_register<R: Readable>(
    #[allow(unused_variables)] register: R,
    value: R::Width,
) -> Result<(), SbiError> {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        crate::ecall2(
            R::ID as usize,
            <R::Width as CastRegisterValue>::reverse_cast(value),
            EXTENSION_ID,
            3,
        )?;
    };

    #[cfg(target_arch = "riscv32")]
    unsafe {
        let (high, low) = <R::Width as CastRegisterValue>::hi_lo(value);
        crate::ecall3(R::ID as usize, low, high, EXTENSION_ID, 3)?;
    };

    Ok(())
}
