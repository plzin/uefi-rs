//! This module provides the Device Path protocol functionalities.

use crate::{Char16, CStr16, unsafe_guid, newtype_enum, proto::Protocol};

/// Represents a device path.
#[repr(C)]
#[unsafe_guid("09576e91-6d3f-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct DevicePath {
    /// The type of the node.
    pub path_type: DevicePathType,

    /// The sub type of the node.
    pub sub_type: u8, // the data type of the sub type depends on the value of path_type

    /// The length of the node in bytes.
    pub length: u16,
}

impl DevicePath {
    /// Determines whether this device path node marks the end of a device path.
    pub fn is_end(&self) -> bool {
        self.path_type == DevicePathType::END_OF_HARDWARE && self.sub_type == EndSubType::END_ENTIRE.0
    }

    /// Returns the next device path node in the path.
    pub fn next_node(&self) -> &DevicePath {
        // TODO: not really safe since the node might not be aligned
        unsafe {
            &*((self as *const _ as *const u8).add(self.length as usize) as *const DevicePath)
        }
    }
}

newtype_enum! {
    /// Device path type
    pub enum DevicePathType: u8 => #[allow(missing_docs)] {
        HARDWARE = 0x1,
        ACPI = 0x2,
        MESSAGING = 0x3,
        MEDIA = 0x4,
        BIOS = 0x5,
        END_OF_HARDWARE = 0x7f,
    }
}

newtype_enum! {
    /// Device subtype of a device of type Media.
    pub enum MediaSubType: u8 => #[allow(missing_docs)] {
        HARD_DRIVE = 0x1,
        CD_ROM = 0x2,
        VENDOR = 0x3,
        FILE_PATH = 0x4,
        MEDIA_PROTOCOL = 0x5,
        PIWG_FIRMWARE_FILE = 0x6,
        PIWG_FIRMWARE_VOLUME = 0x7,
        RELATIVE_OFFSET_RANGE = 0x8,
        RAM_DISK = 0x9,
    }
}

newtype_enum! {
    /// Subtype of a device path that has type END_OF_HARDWARE
    pub enum EndSubType: u8 => #[allow(missing_docs)] {
        END_INSTANCE = 0x01,
        END_ENTIRE = 0xff,
    }
}

/// Device path of a hard drive media
#[repr(C)]
pub struct HardDriveMediaDevicePath {
    /// Device node header.
    pub header: DevicePath,

    /// Entry in the partition table (1-indexed).
    pub partition_number: u32,

    /// LBA of the partition start.
    pub partition_start: u64,

    /// Size of the parition in logical blocks.
    pub partition_size: u64,

    /// Signature unique to this partition.
    pub signature: u128,

    /// Partition format. (MBR, GPT, ...)
    pub partition_format: u8,

    /// The type of the signature.
    pub signature_type: SignatureType,
}

newtype_enum! {
    /// Signature type of a partition
    pub enum SignatureType: u8 => {
        /// No signature (signature field must be 0).
        NONE = 0x00,

        /// 32-bit signature from address 0x1b8 of the type 0x01 MBR.
        MBR_SIGNATURE = 0x01,

        /// 128-bit GUID signature.
        GUID = 0x02,
    }
}

/// Implements the protocol to convert from device paths/nodes to text.
#[repr(C)]
#[unsafe_guid("8b843e20-8132-4852-90cc-551a4e4a7f1c")]
#[derive(Protocol)]
pub struct DevicePathToText {
    convert_device_node_to_text: unsafe extern "efiapi" fn(
        device_node: *const DevicePath, 
        display_only: bool, 
        allow_shortcuts: bool
    ) -> *const Char16,

    convert_device_path_to_text: unsafe extern "efiapi" fn(
        device_path: *const DevicePath,
        display_only: bool,
        allow_shortcuts: bool
    ) -> *const Char16,
}

impl DevicePathToText {
    /// Converts a device node to a textual representation.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is the callers responsibility to free the string with
    /// boot_services.free_pool.
    pub unsafe fn convert_device_node_to_text(
        &self,
        device_node: &DevicePath,
        display_only: bool,
        allow_shortcuts: bool
    ) -> Option<&CStr16> {
        let ptr = (self.convert_device_node_to_text)(device_node as *const _, display_only, allow_shortcuts);
        if ptr.is_null() {
            None
        } else {
            Some(CStr16::from_ptr(ptr))
        }
    }

    /// Converts a device path to a textual representation.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is the callers responsibility to free the string with
    /// boot_services.free_pool.
    pub unsafe fn convert_device_path_to_text(
        &self,
        device_path: &DevicePath,
        display_only: bool,
        allow_shortcuts: bool
    ) -> Option<&CStr16> {
        let ptr = (self.convert_device_path_to_text)(device_path as *const _, display_only, allow_shortcuts);
        if ptr.is_null() {
            None
        } else {
            Some(CStr16::from_ptr(ptr))
        }
    }
}

/// Implements the protocol to convert from text to device paths/nodes.
#[repr(C)]
#[unsafe_guid("05c99a21-c70f-4ad2-8a5f-35df3343f51e")]
#[derive(Protocol)]
pub struct DevicePathFromText {
    convert_text_to_device_node: unsafe extern "efiapi" fn(
        text_device_node: *const Char16
    ) -> *const DevicePath,
    
    convert_text_to_device_path: unsafe extern "efiapi" fn(
        text_device_path: *const Char16
    ) -> *const DevicePath,
}

impl DevicePathFromText {
    /// Converts a textual representation of a device node to the device node.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is the callers responsibility to free the buffer with
    /// boot_services.free_pool.
    pub unsafe fn convert_text_to_device_node(&self, text_device_node: &CStr16) -> *const DevicePath {
        (self.convert_text_to_device_node)(text_device_node.as_ptr())
    }

    /// Converts a textual representation of a device path to a device path.
    ///
    /// # Safety
    /// 
    /// This function is unsafe because it is the callers responsibility to free the buffer with
    /// boot_services.free_pool.
    pub unsafe fn convert_text_to_device_path(&self, text_device_path: &CStr16) -> *const DevicePath {
        (self.convert_text_to_device_path)(text_device_path.as_ptr())
    }
}

/// Implements the device path utilities protocol.
#[repr(C)]
#[unsafe_guid("0379BE4E-D706-437d-B037-EDB82FB772A4")]
#[derive(Protocol)]
pub struct DevicePathUtilities {
    get_device_path_size: extern "efiapi" fn(
        device_path: *const DevicePath
    ) -> usize,

    duplicate_device_path: unsafe extern "efiapi" fn(
        device_path: *const DevicePath
    ) -> *const DevicePath,

    append_device_path: unsafe extern "efiapi" fn(
        src1: *const DevicePath,
        src2: *const DevicePath
    ) -> *const DevicePath,

    append_device_node: unsafe extern "efiapi" fn(
        device_path: *const DevicePath,
        device_node: *const DevicePath
    ) -> *const DevicePath,

    append_device_path_instance: unsafe extern "efiapi" fn(
        device_path: *const DevicePath,
        device_path_instance: *const DevicePath
    ) -> *const DevicePath,

    get_next_device_path_instance: unsafe extern "efiapi" fn(
        device_path_instance: &mut *const DevicePath,
        device_path_instance_size: *mut usize
    ) -> *const DevicePath,

    is_device_path_multi_instance: extern "efiapi" fn(
        device_path: *const DevicePath
    ) -> bool,

    create_device_node: unsafe extern "efiapi" fn(
        node_type: u8,
        node_sub_type: u8,
        node_length: u16
    ) -> *const DevicePath
}

impl DevicePathUtilities {
    /// This function returns the size of the specified device path, 
    /// in bytes, including the end-of-path tag.
    pub fn get_device_path_size(&self, device_path: &DevicePath) -> usize {
        (self.get_device_path_size)(device_path as *const _)
    }

    /// Create a duplicate of the specified path.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is the callers responsibility to free the
    /// the returned path with boot_services.free_pool.
    pub unsafe fn duplicate_device_path(&self, device_path: &DevicePath) -> *const DevicePath {
        (self.duplicate_device_path)(device_path as *const _)
    }

    /// Creates a new device path by appending the second device path to the first.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is the callers responsibility to free the buffer with
    /// boot_services.free_pool.
    pub unsafe fn append_device_path(&self, src1: &DevicePath, src2: &DevicePath) -> *const DevicePath {
        (self.append_device_path)(src1 as *const _, src2 as *const _)
    }

    /// Creates a new device path by appending the device node to the device path.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is the callers responsibility to free the buffer with
    /// boot_services.free_pool.
    pub unsafe fn append_device_node(&self, device_path: &DevicePath, device_node: &DevicePath) -> *const DevicePath {
        (self.append_device_node)(device_path as *const _, device_node as *const _)
    }

    /// Creates a new path by appending the device path instance to the device path.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is the callers responsibility to free the buffer with
    /// boot_services.free_pool.
    pub unsafe fn append_device_path_instance(
        &self, 
        device_path: &DevicePath, 
        device_path_instance: &DevicePath
    ) -> *const DevicePath {
        (self.append_device_path_instance)(device_path as *const _, device_path_instance as *const _)
    }

    /// Returns a copy of the current instance that has to be freed and the next instance.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is the callers responsibility to free the copy of the
    /// instace with boot_services.free_pool.
    pub unsafe fn get_next_device_path_instance(&self, device_path_instance: &DevicePath) 
        -> (*const DevicePath, *const DevicePath) {
        let mut ptr = device_path_instance as *const _;
        let copy = (self.get_next_device_path_instance)(&mut ptr, core::ptr::null_mut());
        (copy, ptr)
    }

    /// Creates a device node.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is the callers responsibility to free the newly created
    /// devide node.
    pub unsafe fn create_device_node(&self, node_type: u8, node_sub_type: u8, node_length: u16) -> *const DevicePath {
        (self.create_device_node)(node_type, node_sub_type, node_length)
    }

    /// Returns whether a device path is multi-instance.
    pub fn is_device_path_multi_instance(&self, device_path: &DevicePath) -> bool {
        (self.is_device_path_multi_instance)(device_path as *const _)
    }
}
