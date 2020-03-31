//! This module provides the Device Path protocol functionalities.

use crate::{Char16, CStr16, unsafe_guid, proto::Protocol};

/// Represents a device path.
#[repr(C)]
#[unsafe_guid("09576e91-6d3f-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct DevicePath {
    path_type: u8,
    sub_type: u8,
    length: u16,
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
    /// This function is unsafe because it is the callers responsibility to free the buffer with
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
    /// This function is unsafe because it is the callers responsibility to free the buffer with
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
    ) -> *mut DevicePath,
    
    convert_text_to_device_path: unsafe extern "efiapi" fn(
        text_device_path: *const Char16
    ) -> *mut DevicePath,
}

impl DevicePathFromText {
    /// Converts a textual representation of a device node to the device node.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is the callers responsibility to free the buffer with
    /// boot_services.free_pool.
    pub unsafe fn convert_text_to_device_node(&self, text_device_node: &CStr16) -> Option<&DevicePath> {
        (self.convert_text_to_device_node)(text_device_node.as_ptr()).as_ref()
    }

    /// Converts a textual representation of a device path to a device path.
    ///
    /// # Safety
    /// 
    /// This function is unsafe because it is the callers responsibility to free the buffer with
    /// boot_services.free_pool.
    pub unsafe fn convert_text_to_device_path(&self, text_device_path: &CStr16) -> Option<&DevicePath> {
        (self.convert_text_to_device_path)(text_device_path.as_ptr()).as_ref()
    }
}

/// Implements the device path utilities protocol.
#[repr(C)]
#[unsafe_guid("0379BE4E-D706-437d-B037-EDB82FB772A4")]
#[derive(Protocol)]
pub struct DevicePathUtilities {
    get_device_path_size: unsafe extern "efiapi" fn(
        device_path: *const DevicePath
    ) -> usize,

    duplicate_device_path: unsafe extern "efiapi" fn(
        device_path: *const DevicePath
    ) -> *mut DevicePath,

    append_device_path: unsafe extern "efiapi" fn(
        src1: *const DevicePath,
        src2: *const DevicePath
    ) -> *mut DevicePath,

    append_device_node: unsafe extern "efiapi" fn(
        device_path: *const DevicePath,
        device_node: *const DevicePath
    ) -> *mut DevicePath,

    append_device_path_instance: unsafe extern "efiapi" fn(
        device_path: *const DevicePath,
        device_path_instance: *const DevicePath
    ) -> *mut DevicePath,

    get_next_device_path_instance: unsafe extern "efiapi" fn(
        device_path_instance: *const *mut DevicePath,
        device_path_instance_size: *mut usize
    ) -> *mut DevicePath,

    is_device_path_multi_instance: unsafe extern "efiapi" fn(
        device_path: *const DevicePath
    ) -> bool,

    create_device_node: unsafe extern "efiapi" fn(
        node_type: u8,
        node_sub_type: u8,
        node_length: u16
    ) -> *mut DevicePath
}

impl DevicePathUtilities {
    /// Creates a new device path by appending the second device path to the first.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is the callers responsibility to free the buffer with
    /// boot_services.free_pool.
    pub unsafe fn append_device_path(&self, src1: &DevicePath, src2: &DevicePath) -> Option<&DevicePath> {
        (self.append_device_path)(src1 as *const _, src2 as *const _).as_ref()
    }

    /// Creates a new device path by appending the device node to the device path.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is the callers responsibility to free the buffer with
    /// boot_services.free_pool.
    pub unsafe fn append_device_node(&self, device_path: &DevicePath, device_node: &DevicePath) -> Option<&DevicePath> {
        (self.append_device_node)(device_path as *const _, device_node as *const _).as_ref()
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
    ) -> Option<&DevicePath> {
        (self.append_device_path_instance)(device_path as *const _, device_path_instance as *const _).as_ref()
    }
}
