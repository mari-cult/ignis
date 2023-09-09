//! Interact with EFI variables.
//!
//! # References
//!
//! - [systemd](https://systemd.io/BOOT_LOADER_INTERFACE/)
//! - [UEFI 2.3.1](https://uefi.org/sites/default/files/resources/UEFI_2_3_1_C.pdf)
//! - [`MdePkg/Include/Uefi/UefiSpec.h#L1778`](https://github.com/tianocore/edk2/blob/master/MdePkg/Include/Uefi/UefiSpec.h#L1778)
//! - [`include/linux/efi.h#L922`](https://github.com/torvalds/linux/blob/master/include/linux/efi.h#L922)

use {
    bytemuck::{NoUninit, Pod, Zeroable},
    std::{
        fs::{self, File},
        io::{self, Write},
        path::Path,
    },
};

bitflags::bitflags! {
    /// EFI variable attributes.
    #[derive(Clone, Copy, Debug, Eq, Pod, PartialEq, Zeroable)]
    #[repr(transparent)]
    pub struct Attribute: u32 {
        /// Permanently stored.
        const NON_VOLATILE = 1 << 0;
        /// Boot services can access it.
        const BOOT_SERVICE_ACCESS = 1 << 1;
        /// Runtime can access it.
        const RUNTIME_ACCESS = 1 << 2;
        /// Hardware errors are recorded.
        const HARDWARE_ERROR_RECORD = 1 << 3;
        /// Requires authentication to write.
        const AUTHENTICATED_WRITE_ACCESS = 1 << 4;
        /// Time-based authentication to write.
        const TIME_BASED_AUTHENTICATED_WRITE_ACCESS = 1 << 5;
        /// Appends rather than truncating on write.
        const APPEND_WRITE = 1 << 6;
    }
}

bitflags::bitflags! {
    /// OS indications.
    #[derive(Clone, Copy, Debug, Eq, Pod, PartialEq, Zeroable)]
    #[repr(transparent)]
    pub struct Indication: u64 {
        /// Boot to firmware UI.
        const BOOT_TO_FW_UI = 0x0000000000000001;
        #[allow(missing_docs)]
        const TIMESTAMP_REVOCATION = 0x0000000000000002;
        #[allow(missing_docs)]
        const FILE_CAPSULE_DELIVERY_SUPPORTED = 0x0000000000000004;
        #[allow(missing_docs)]
        const FMP_CAPSULE_SUPPORTED = 0x0000000000000008;
        #[allow(missing_docs)]
        const CAPSULE_RESULT_VAR_SUPPORTED = 0x0000000000000010;
        #[allow(missing_docs)]
        const START_PLATFORM_RECOVERY = 0x0000000000000040;
        #[allow(missing_docs)]
        const JSON_CONFIG_DATA_REFRESH = 0x0000000000000080;
    }
}

/// OS indication data.
#[derive(Clone, Copy, NoUninit)]
#[repr(C, packed)]
pub struct OsIndications {
    attributes: Attribute,
    indications: Indication,
}

/// Determine whether OS indications are supported.
pub fn os_indications_supported() -> bool {
    const OS_INDICATIONS_SUPPORTED: &str =
        "/sys/firmware/efi/efivars/OsIndicationsSupported-8be4df61-93ca-11d2-aa0d-00e098032b8c";

    Path::new(OS_INDICATIONS_SUPPORTED).exists()
}

/// Set the "Boot into firmware UI" OS indicator.
pub fn set_boot_to_firmware(enabled: bool) -> io::Result<()> {
    const OS_INDICATIONS: &str =
        "/sys/firmware/efi/efivars/OsIndications-8be4df61-93ca-11d2-aa0d-00e098032b8c";

    if enabled {
        let os_indications = OsIndications {
            attributes: Attribute::NON_VOLATILE
                | Attribute::BOOT_SERVICE_ACCESS
                | Attribute::RUNTIME_ACCESS,
            indications: Indication::BOOT_TO_FW_UI,
        };

        let mut variable = File::create(OS_INDICATIONS)?;

        // Write only once, as that is what the kernel expects.
        let _amount = variable.write(bytemuck::bytes_of(&os_indications))?;

        Ok(())
    } else {
        match fs::remove_file(OS_INDICATIONS) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}
