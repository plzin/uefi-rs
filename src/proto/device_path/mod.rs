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
