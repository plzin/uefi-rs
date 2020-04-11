//! This module implements the Block IO protocol.

use crate::{unsafe_guid, proto::Protocol};

/// This protocol is used to abstract mass storage devices to allow code 
/// running in the EFI boot services environment to access them without 
/// specific knowledge of the type of device or controller that manages 
/// the device. Functions are defined to read and write data at a block 
/// level from mass storage devices as well as to manage such devices 
/// in the EFI boot services environment.
#[repr(C)]
#[unsafe_guid("964e5b21-6459-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct BlockIO {
    /* to be implemented */
}
