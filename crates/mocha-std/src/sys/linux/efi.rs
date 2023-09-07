//! Interact with EFI variables.

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
    ///
    /// # References
    ///
    ///  - [Linux](https://github.com/torvalds/linux/blob/master/include/linux/efi.h#L922)
    #[derive(Clone, Copy, Debug, Eq, Pod, PartialEq, Zeroable)]
    #[repr(transparent)]
    pub struct Attribute: u32 {
        const NON_VOLATILE = 0x0000000000000001;
        const BOOT_SERVICE_ACCESS = 0x0000000000000002;
        const RUNTIME_ACCESS = 0x0000000000000004;
        const HARDWARE_ERROR_RECORD = 0x0000000000000008;
        const AUTHENTICATED_WRITE_ACCESS = 0x0000000000000010;
        const TIME_BASED_AUTHENTICATED_WRITE_ACCESS = 0x0000000000000020;
        const APPEND_WRITE = 0x0000000000000040;
    }
}

bitflags::bitflags! {
    /// OS indications.
    ///
    /// # References
    ///
    /// - [UEFI 2.3.1](https://uefi.org/sites/default/files/resources/UEFI_2_3_1_C.pdf)
    /// - [EDK2](https://github.com/tianocore/edk2/blob/master/MdePkg/Include/Uefi/UefiSpec.h#L1778)
    #[derive(Clone, Copy, Debug, Eq, Pod, PartialEq, Zeroable)]
    #[repr(transparent)]
    pub struct Indication: u64 {
        const BOOT_TO_FW_UI = 0x0000000000000001;
        const TIMESTAMP_REVOCATION = 0x0000000000000002;
        const FILE_CAPSULE_DELIVERY_SUPPORTED = 0x0000000000000004;
        const FMP_CAPSULE_SUPPORTED = 0x0000000000000008;
        const CAPSULE_RESULT_VAR_SUPPORTED = 0x0000000000000010;
        const START_PLATFORM_RECOVERY = 0x0000000000000040;
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
        let mut variable = File::create(OS_INDICATIONS)?;
        let os_indications = OsIndications {
            attributes: Attribute::NON_VOLATILE
                | Attribute::BOOT_SERVICE_ACCESS
                | Attribute::RUNTIME_ACCESS,
            indications: Indication::BOOT_TO_FW_UI,
        };

        variable.write(bytemuck::bytes_of(&os_indications))?;

        Ok(())
    } else {
        match fs::remove_file(OS_INDICATIONS) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}
