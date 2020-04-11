//! Loaded image protocol.

use crate::{
    data_types::{CStr16, Char16},
    proto::{Protocol, device_path::DevicePath},
    table::boot::MemoryType,
    unsafe_guid, Handle, Status,
};
use core::ffi::c_void;

/// The Loaded Image protocol. This can be opened on any image handle using the `HandleProtocol` boot service.
#[repr(C)]
#[unsafe_guid("5b1b31a1-9562-11d2-8e3f-00a0c969723b")]
#[derive(Protocol)]
pub struct LoadedImage {
    revision: u32,
    parent_handle: Handle,
    system_table: *const c_void,

    // Source location of the image
    device_handle: Handle,
    file_path: *const DevicePath,
    _reserved: *const c_void,

    // Image load options
    load_options_size: u32,
    load_options: *const c_void,

    // Location where image was loaded
    image_base: usize,
    image_size: u64,
    image_code_type: MemoryType,
    image_data_type: MemoryType,
    /// This is a callback that a loaded image can use to do cleanup. It is called by the
    /// UnloadImage boot service.
    unload: extern "efiapi" fn(image_handle: Handle) -> Status,
}

/// Errors that can be raised during parsing of the load options.
#[derive(Debug)]
pub enum LoadOptionsError {
    /// The passed buffer is not large enough to contain the load options.
    BufferTooSmall,
    /// The load options are not valid UTF-8.
    NotValidUtf8,
}

impl LoadedImage {
    /// Get the parent handle.
    pub fn parent_handle(&self) -> Handle {
        self.parent_handle
    }

    /// Get the device handle that the image was loaded from.
    pub fn device_handle(&self) -> Handle {
        self.device_handle
    }

    /// Get the file path that the image was loaded from.
    pub fn file_path(&self) -> Option<&DevicePath> {
        unsafe { 
            self.file_path.as_ref()
        }
    }

    /// Get the load options of the given image. If the image was executed from the EFI shell, or from a boot
    /// option, this is the command line that was used to execute it as a string.
    pub fn load_options<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a str, LoadOptionsError> {
        let ucs2_slice = unsafe { CStr16::from_ptr(self.load_options as *const Char16).to_u16_slice() };
        let length =
            ucs2::decode(ucs2_slice, buffer).map_err(|_| LoadOptionsError::BufferTooSmall)?;
        core::str::from_utf8(&buffer[0..length]).map_err(|_| LoadOptionsError::NotValidUtf8)
    }

    /// Get the address that the image was loaded at.
    pub fn image_base(&self) -> usize {
        self.image_base
    }

    /// Get the size of the image.
    pub fn image_size(&self) -> u64 {
        self.image_size
    }

    /// Get the memory type of the image's code.
    pub fn image_code_type(&self) -> MemoryType {
        self.image_code_type
    }

    /// Get the memory type of the image's data.
    pub fn image_data_type(&self) -> MemoryType {
        self.image_code_type
    }

    /// Overwrite the parent handle.
    pub fn overwrite_parent_handle(&mut self, parent_handle: Handle) {
        self.parent_handle = parent_handle;
    }

    /// Overwrite the load options.
    pub fn overwrite_load_options<'a>(&'a mut self, load_options: &'a [u8]) {
        self.load_options_size = load_options.len() as u32;
        self.load_options = load_options as *const _ as *const c_void
    }
}
