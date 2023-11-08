This crate provides a safe, pure-Rust implementation of the RISC-V [Supervisor Binary Interface](https://github.com/riscv-non-isa/riscv-sbi-doc) (SBI) to be used in S-mode software, such as operating system kernels. This crate implements the [`v2.0-rc5`](https://github.com/riscv-non-isa/riscv-sbi-doc/releases/tag/v2.0-rc5) version of the SBI specification and aims to stay up to date with it as newer versions of the specification are released.

## Important safety note

Since this crate is meant to be used by S-mode software, it is assumed that you are executing the provided functions from within S-mode with a spec compliant SBI implementation running in M-mode (or S-mode in the case of VS-mode), and considers execution from any other operating mode out of the scope of this crate, which may bring with it memory safety concerns in those environments. Consider this an implicit contract when using the crate.

## SBI extension support

Extension implementation state legend:

✅ - Fully implemented

🚧 - Partially implemented

❌ - Not implemented

### Standard SBI extensions

<details>
<summary>SBI extensions implementation state</summary>

#### Legacy ✅

| Function                      | Extension ID | Implemented |
| ----------------------------- | :----------: | :---------: |
| Set timer                     |      0       |     ✅      |
| Console putchar               |      1       |     ✅      |
| Console getchar               |      2       |     ✅      |
| Clear IPI                     |      3       |     ✅      |
| Send IPI                      |      4       |     ✅      |
| Remote `FENCE.I`              |      5       |     ✅      |
| Remote `SFENCE.VMA`           |      6       |     ✅      |
| Remote `SFENCE.VMA` with ASID |      7       |     ✅      |
| Shutdown                      |      8       |     ✅      |

#### Base ✅

| Function                       | Function ID | Implemented |
| ------------------------------ | :---------: | :---------: |
| Get SBI specification version  |      0      |     ✅      |
| Get SBI implementation ID      |      1      |     ✅      |
| Get SBI implementation version |      2      |     ✅      |
| Probe SBI extension            |      3      |     ✅      |
| Get machine vendor ID          |      4      |     ✅      |
| Get machine architecture ID    |      5      |     ✅      |
| Get machine implementation ID  |      6      |     ✅      |

#### Timer ✅

| Function  | Function ID | Implemented |
| --------- | :---------: | :---------: |
| Set timer |      0      |     ✅      |

#### Interprocessor Interrupt (IPI) ✅

| Function | Function ID | Implemented |
| -------- | :---------: | :---------: |
| Send IPI |      0      |     ✅      |

#### RFENCE ✅

| Function                       | Function ID | Implemented |
| ------------------------------ | :---------: | :---------: |
| Remote `FENCE.I`               |      0      |     ✅      |
| Remote `SFENCE.VMA`            |      1      |     ✅      |
| Remote `SFENCE.VMA` with ASID  |      2      |     ✅      |
| Remote `HFENCE.GVMA` with VMID |      3      |     ✅      |
| Remote `HFENCE.GVMA`           |      4      |     ✅      |
| Remote `HFENCE.VVMA` with ASID |      5      |     ✅      |
| Remote `HFENCE.VVMA`           |      6      |     ✅      |

#### Hart State Management ✅

| Function        | Function ID | Implemented |
| --------------- | :---------: | :---------: |
| Hart start      |      0      |     ✅      |
| Hart stop       |      1      |     ✅      |
| Get hart status |      2      |     ✅      |
| Hart suspend    |      3      |     ✅      |

#### System Reset ✅

| Function     | Function ID | Implemented |
| ------------ | :---------: | :---------: |
| System reset |      0      |     ✅      |

#### Performance Monitoring Unit ✅

| Function                    | Function ID | Implemented |
| --------------------------- | :---------: | :---------: |
| Get number of counters      |      0      |     ✅      |
| Get counter information     |      1      |     ✅      |
| Configure matching counters |      2      |     ✅      |
| Start counters              |      3      |     ✅      |
| Stop counters               |      4      |     ✅      |
| Read firmware counter       |      5      |     ✅      |

</details>

### Experimental, vendor-specific, and firmware-specific extensions

Experimental, vendor-specific, and firmware-specific SBI extensions are provided as opt-in crate features.

The currently supported non-standard SBI extensions are:

### Experimental

There are currently no supported experimental SBI extensions.

### Vendor-specific

There are currently no supported vendor-specific SBI extensions.

### Firmware-specific

There are currently no supported firmware-specific SBI extensions.

## License

`sbi` is licensed under the Mozilla Public License 2.0
