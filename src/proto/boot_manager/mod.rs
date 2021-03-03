//! Boot Manager Protocols

use crate::{Status, Result, Guid, unsafe_guid};
use crate::proto::{Protocol, loaded_image::DevicePath};

/// Boot Manager Policy Protocol
#[repr(C)]
#[unsafe_guid("FEDF8E0C-E147-11E3-9903-B8E8562CBAFA")]
#[derive(Protocol)]
pub struct BootManagerPolicy {
    revision: u64,
    connect_device_path: extern "efiapi" fn(
        this: *const BootManagerPolicy,
        device_path: *const DevicePath,
        recursive: bool
    ) -> Status,
    connect_device_class: extern "efiapi" fn(
        this: *const BootManagerPolicy,
        class: *const Guid,
    ) -> Status,
}

impl BootManagerPolicy {
    /// Returns the structure's revision.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Connect a device path following the platform's EFI Boot Manager policy.
    pub fn connect_device_path(&self, device_path: &DevicePath, recursive: bool) -> Result {
        (self.connect_device_path)(self as *const _, device_path as *const _, recursive).into()
    }

    /// Connect a class of devices using the platform Boot Manager policy.
    pub fn connect_device_class(&self, class: &Guid) -> Result {
        (self.connect_device_class)(self as *const _, class as *const _).into()
    }

    /// Connects all controllers using the platform's EFI Boot Manager policy.
    pub fn connect_all_controllers(&self) -> Result {
        (self.connect_device_path)(self as *const _, core::ptr::null(), false).into()
    }
}
