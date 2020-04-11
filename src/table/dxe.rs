//! UEFI Driver Execution Environment Services Table

use super::Header;
use crate::{Status, Result, Guid};

/// Contains pointers to all of the DXE Services
#[repr(C)]
pub struct DXEServices {
    header: Header,

    _reserved1: [usize; 13],

    dispatch: extern "efiapi" fn() -> Status,

    _reserved2: [usize; 4],
}

impl DXEServices {
    /// A pointer to the table can be found in System Table's Config Table
    /// with this associated guid.
    pub const GUID: Guid = Guid::from_values(0x5ad34ba, 0x6f02, 0x4214, 0x952e, 
        [0x4d, 0xa0, 0x39, 0x8e, 0x2b, 0xb9]);

    /// Loads and executes DXE drivers from firmware volumes.
    pub fn dispatch(&self) -> Result {
        (self.dispatch)().into()
    }
}

impl super::Table for DXEServices {
    const SIGNATURE: u64 = 0x5652_4553_5f45_5844;
}
