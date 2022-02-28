This crate provides a safe, pure-Rust implementation of the RISC-V SBI interface to be used in S-mode software, such as operating system kernels. This crate implements the `v1.0-rc2` version of the SBI specification and aims to stay up to date with it as newer versions of the specification are released.

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

#### Base extension ✅
|             Function           | Function ID | Implemented |
| ------------------------------ | :---------: | :---------: |
| Get SBI specification version  | 0           | ✅          |
| Get SBI implementation ID      | 1           | ✅          |
| Get SBI implementation version | 2           | ✅          |
| Probe SBI extension            | 3           | ✅          |
| Get machine vendor ID          | 4           | ✅          |
| Get machine architecture ID    | 5           | ✅          |
| Get machine implementation ID  | 6           | ✅          |


#### Timer ✅
| Function  | Function ID | Implemented |
| --------- | :---------: | :---------: |
| Set Timer | 0           | ✅          |

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