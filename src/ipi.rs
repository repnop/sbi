// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2022 repnop
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{ecall2, HartMask, SbiError};

/// The IPI extension ID
pub const EXTENSION_ID: usize = 0x735049;

/// Send an inter-processor interrupt (IPI) to the harts defined in `hart_mask`.
/// The IPI is received on a hart as a supervisor software interrupt.
///
/// ### Possible errors
///
/// [`SbiError::InvalidParameter`]: The `hart_mask` base or any hart IDs
///     specified by `hart_mask` are invalid or unaccessible from supervisor
///     mode
pub fn send_ipi(hart_mask: HartMask) -> Result<(), SbiError> {
    unsafe { ecall2(hart_mask.mask, hart_mask.base, EXTENSION_ID, 0).map(drop) }
}
