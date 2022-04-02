/*
    SPDX-License-Identifier: MPL-2.0
    SPDX-FileCopyrightText: 2022 repnop
    
    This Source Code Form is subject to the terms of the Mozilla Public License,
    v. 2.0. If a copy of the MPL was not distributed with this file, You can
    obtain one at https://mozilla.org/MPL/2.0/.
*/

ENTRY(_start);

SECTIONS {
    . = 0x80200000;
    .text : {
        PROVIDE(__text_start = .);
        *(.text.init)
        *(.text .text.*)
        PROVIDE(__text_end = .);
    }

    .data : {
        *(.data .data.* .rodata .rodata.*)
        . = ALIGN(8);
        PROVIDE(__global_pointer$ = .);
        *(.sdata .sdata.*)
        . = ALIGN(4);
        PROVIDE(__bss_start = .);
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        . = ALIGN(4);
        PROVIDE(__bss_end = .);
    }

    .tmp_stack : ALIGN(8) {
        PROVIDE(__stack_end = .);
        . += 1024 * 8;
        PROVIDE(__stack_start = .);
    }

    .tmp_stack2 : ALIGN(8) {
        PROVIDE(__stack_end2 = .);
        . += 1024 * 8;
        PROVIDE(__stack_start2 = .);
    }

    /DISCARD/ : { *(.eh_frame_hdr .eh_frame) }
}
