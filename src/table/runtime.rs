//! UEFI services available at runtime, even after the OS boots.

use super::Header;
use crate::table::boot::MemoryDescriptor;
use crate::{Result, Status, Guid};
use crate::data_types::Char16;
use bitflags::bitflags;
use core::mem::MaybeUninit;
use core::ptr;
use core::ffi::c_void;
#[cfg(feature = "exts")]
use crate::alloc_api::{string::String, vec, vec::Vec};

/// Contains pointers to all of the runtime services.
///
/// This table, and the function pointers it contains are valid
/// even after the UEFI OS loader and OS have taken control of the platform.
#[repr(C)]
pub struct RuntimeServices {
    header: Header,
    get_time:
        unsafe extern "efiapi" fn(time: *mut Time, capabilities: *mut TimeCapabilities) -> Status,
    set_time: unsafe extern "efiapi" fn(time: &Time) -> Status,
    get_wakeup_time: usize,
    set_wakeup_time: usize,
    set_virtual_address_map: unsafe extern "efiapi" fn(
        map_size: usize,
        desc_size: usize,
        desc_version: u32,
        virtual_map: *mut MemoryDescriptor,
    ) -> Status,
    convert_pointer: extern "efiapi" fn(
        debug_disposition: usize,
        address: &mut *const u8
    ) -> Status,
    get_variable: extern "efiapi" fn(
        variable_name: *const Char16,
        vendor_guid: &Guid,
        attributes: *mut u32,
        data_size: &mut usize,
        data: *mut c_void
    ) -> Status,
    get_next_variable_name: extern "efiapi" fn(
        variable_name_size: &mut usize,
        variable_name: *mut Char16,
        vendor_guid: &mut Guid
    ) -> Status,
    set_variable: extern "efiapi" fn(
        variable_name: *const Char16,
        vendor_guid: &Guid,
        attributes: u32,
        data_size: usize,
        data: *const c_void
    ) -> Status,
    get_next_high_monotonic_count: usize,
    reset: unsafe extern "efiapi" fn(
        rt: ResetType,

        status: Status,
        data_size: usize,
        data: *const u8,
    ) -> !,
}

impl RuntimeServices {
    /// Query the current time and date information
    pub fn get_time(&self) -> Result<Time> {
        let mut time = MaybeUninit::<Time>::uninit();
        unsafe { (self.get_time)(time.as_mut_ptr(), ptr::null_mut()) }
            .into_with_val(|| unsafe { time.assume_init() })
    }

    /// Query the current time and date information and the RTC capabilities
    pub fn get_time_and_caps(&self) -> Result<(Time, TimeCapabilities)> {
        let mut time = MaybeUninit::<Time>::uninit();
        let mut caps = MaybeUninit::<TimeCapabilities>::uninit();
        unsafe { (self.get_time)(time.as_mut_ptr(), caps.as_mut_ptr()) }
            .into_with_val(|| unsafe { (time.assume_init(), caps.assume_init()) })
    }

    /// Sets the current local time and date information
    ///
    /// During runtime, if a PC-AT CMOS device is present in the platform, the
    /// caller must synchronize access to the device before calling `set_time`.
    ///
    /// # Safety
    ///
    /// Undefined behavior could happen if multiple tasks try to
    /// use this function at the same time without synchronisation.
    pub unsafe fn set_time(&mut self, time: &Time) -> Result {
        (self.set_time)(time).into()
    }

    /// Changes the runtime addressing mode of EFI firmware from physical to virtual.
    ///
    /// # Safety
    ///
    /// Setting new virtual memory map is unsafe and may cause undefined behaviors.
    pub unsafe fn set_virtual_address_map(&self, map: &mut [MemoryDescriptor]) -> Result {
        // Unsafe Code Guidelines guarantees that there is no padding in an array or a slice
        // between its elements if the element type is `repr(C)`, which is our case.
        //
        // See https://rust-lang.github.io/unsafe-code-guidelines/layout/arrays-and-slices.html
        let map_size = core::mem::size_of_val(map);
        let entry_size = core::mem::size_of::<MemoryDescriptor>();
        let entry_version = crate::table::boot::MEMORY_DESCRIPTOR_VERSION;
        let map_ptr = map.as_mut_ptr();
        (self.set_virtual_address_map)(map_size, entry_size, entry_version, map_ptr).into()
    }

    /// Returns an iterator over EFI variables.
    #[cfg(feature = "exts")]
    pub fn variables<'a>(&'a self) -> VariablesIterator {
        VariablesIterator::new(self)
    }

    /// Used internally by VariablesIterator.
    fn get_next_variable_name(&self, variable_name: &mut [u16], vendor_guid: &mut Guid) -> (Status, usize) {
        let mut buf_len = variable_name.len();
        let status = (self.get_next_variable_name)(&mut buf_len, variable_name as *mut _ as *mut Char16, vendor_guid);

        (status, buf_len)
    }

    /// Get the data stored with the variable.
    #[cfg(feature = "exts")]
    pub fn get_variable(&self, variable: &Variable) -> Result<(Vec<u8>, u32)> {
        let mut attributes = 0;
        let name_len = variable.name.as_str().chars().count();
        let mut name = vec![0; name_len + 1];

        ucs2::encode(variable.name.as_str(), &mut name[..name_len])
            .map_err(|_| Status::INVALID_PARAMETER)?;

        // use some default size to try to avoid a second call because of buffer size
        let mut data = vec![0; 64];
        let mut data_size = data.len();

        // this might be called when multiple processors are running, so the data could get bigger
        // even after resizing the buffer, so this should be called in a loop
        loop {
            match (self.get_variable)(name.as_ptr() as *const Char16, &variable.vendor, 
                &mut attributes, &mut data_size, data.as_mut_ptr() as *mut c_void) {
                Status::SUCCESS => {
                    // resize the vector to reflect the size of the data returned
                    data.truncate(data_size);
                    return Ok(crate::Completion::new(Status::SUCCESS, (data, attributes)));
                },
                Status::BUFFER_TOO_SMALL => {
                    // resize the buffer 
                    data.resize(data_size, 0);
                },
                s => return Err(s.into()),
            }
        }
    }

    /// Sets the value of a variable. This service can be used to create a new variable, 
    /// modify the value of an existing variable, or to delete an existing variable.
    pub fn set_variable(&self, variable: &Variable, attributes: u32, data: &[u8]) -> Result {
        let name_len = variable.name.as_str().chars().count();
        let mut name = vec![0; name_len + 1];

        ucs2::encode(variable.name.as_str(), &mut name[..name_len])
            .map_err(|_| Status::INVALID_PARAMETER)?;

        (self.set_variable)(name.as_ptr() as *const Char16, &variable.vendor, 
            attributes, data.len(), data.as_ptr() as *const c_void).into()
    }

    /// Resets the computer.
    pub fn reset(&self, rt: ResetType, status: Status, data: Option<&[u8]>) -> ! {
        let (size, data) = match data {
            // FIXME: The UEFI spec states that the data must start with a NUL-
            //        terminated string, which we should check... but it does not
            //        specify if that string should be Latin-1 or UCS-2!
            //
            //        PlatformSpecific resets should also insert a GUID after the
            //        NUL-terminated string.
            Some(data) => (data.len(), data.as_ptr()),
            None => (0, ptr::null()),
        };

        unsafe { (self.reset)(rt, status, size, data) }
    }
}

impl super::Table for RuntimeServices {
    const SIGNATURE: u64 = 0x5652_4553_544e_5552;
}

/// An EFI Variable
#[cfg(feature = "exts")]
#[derive(Debug, Clone)]
pub struct Variable {
    /// The variable name.
    pub name: String,

    /// The vendor's guid.
    pub vendor: Guid,
}

#[cfg(feature = "exts")]
impl Variable {
    /// Creates an EFI Variable from a variable name and the vendor guid.
    pub fn new(name: String, vendor: Guid) -> Variable {
        Variable {
            name,
            vendor
        }
    }
}

/// An iterator for EFI variables.
#[cfg(feature = "exts")]
pub struct VariablesIterator<'a> {
    rt: &'a RuntimeServices,
    buffer: Vec<u16>,
    guid: Guid,
}

#[cfg(feature = "exts")]
impl<'a> VariablesIterator<'a> {
    /// Creates an iterator over EFI Variables.
    pub fn new(runtime_services: &'a RuntimeServices) -> VariablesIterator {
        VariablesIterator {
            rt: runtime_services,
            buffer: vec![0; 32], // try to avoid BUFFER_TOO_SMALL
            guid: Guid::from_values(0, 0, 0, 0, [0, 0, 0, 0, 0, 0]),
        }
    }
}

#[cfg(feature = "exts")]
impl<'a> Iterator for VariablesIterator<'a> {
    type Item = Variable;
    fn next(&mut self) -> Option<Self::Item> {
        match self.rt.get_next_variable_name(self.buffer.as_mut_slice(), &mut self.guid) {
            (Status::SUCCESS, _) => (),
            (Status::BUFFER_TOO_SMALL, len) => {
                // Note that if EFI_BUFFER_TOO_SMALL is returned,
                // the VariableName buffer was too small for the next variable. 
                // When such an error occurs, the VariableNameSize is updated 
                // to reflect the size of buffer needed.
                self.buffer.resize(len, 0);

                let (status, _) = self.rt.get_next_variable_name(self.buffer.as_mut_slice(), &mut self.guid);
                if status.is_error() {
                    panic!("get_next_variable failed after resizing the buffer ({:?})", status);
                }
            }
            _ => return None,
        }

        let name_len = self.buffer.as_slice().iter().take_while(|&&e| e != 0).count();

        let mut utf8_buffer = vec![0; name_len * 3];

        // every ucs2 string has a utf8 encoding so this should never fail
        let name_len = ucs2::decode(&self.buffer[..name_len], &mut utf8_buffer[..])
            .expect("get_next_variable_name returned a string that couldn't be converted to utf8");

        // truncate the vector so that we can give it to the String constructor
        utf8_buffer.truncate(name_len);

        // ucs2::decode only returns valid utf8 encodings
        let name = unsafe {
            String::from_utf8_unchecked(utf8_buffer)
        };

        return Some(Variable::new(name, self.guid));
    }
}

/// The current time information
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Time {
    year: u16,  // 1900 - 9999
    month: u8,  // 1 - 12
    day: u8,    // 1 - 31
    hour: u8,   // 0 - 23
    minute: u8, // 0 - 59
    second: u8, // 0 - 59
    _pad1: u8,
    nanosecond: u32, // 0 - 999_999_999
    time_zone: i16,  // -1440 to 1440, or 2047 if unspecified
    daylight: Daylight,
    _pad2: u8,
}

bitflags! {
    /// Flags describing the capabilities of a memory range.
    pub struct Daylight: u8 {
        /// Time is affected by daylight savings time
        const ADJUST_DAYLIGHT = 0x01;
        /// Time has been adjusted for daylight savings time
        const IN_DAYLIGHT = 0x02;
    }
}

impl Time {
    /// Build an UEFI time struct
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
        time_zone: i16,
        daylight: Daylight,
    ) -> Self {
        assert!(year >= 1900 && year <= 9999);
        assert!(month >= 1 && month <= 12);
        assert!(day >= 1 && day <= 31);
        assert!(hour <= 23);
        assert!(minute <= 59);
        assert!(second <= 59);
        assert!(nanosecond <= 999_999_999);
        assert!((time_zone >= -1440 && time_zone <= 1440) || time_zone == 2047);
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            _pad1: 0,
            nanosecond,
            time_zone,
            daylight,
            _pad2: 0,
        }
    }

    /// Query the year
    pub fn year(&self) -> u16 {
        self.year
    }

    /// Query the month
    pub fn month(&self) -> u8 {
        self.month
    }

    /// Query the day
    pub fn day(&self) -> u8 {
        self.day
    }

    /// Query the hour
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// Query the minute
    pub fn minute(&self) -> u8 {
        self.minute
    }

    /// Query the second
    pub fn second(&self) -> u8 {
        self.second
    }

    /// Query the nanosecond
    pub fn nanosecond(&self) -> u32 {
        self.nanosecond
    }

    /// Query the time offset in minutes from UTC, or None if using local time
    pub fn time_zone(&self) -> Option<i16> {
        if self.time_zone == 2047 {
            None
        } else {
            Some(self.time_zone)
        }
    }

    /// Query the daylight savings time information
    pub fn daylight(&self) -> Daylight {
        self.daylight
    }
}

/// Real time clock capabilities
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct TimeCapabilities {
    /// Reporting resolution of the clock in counts per second. 1 for a normal
    /// PC-AT CMOS RTC device, which reports the time with 1-second resolution.
    pub resolution: u32,

    /// Timekeeping accuracy in units of 1e-6 parts per million.
    pub accuracy: u32,

    /// Whether a time set operation clears the device's time below the
    /// "resolution" reporting level. False for normal PC-AT CMOS RTC devices.
    pub sets_to_zero: bool,
}

/// The type of system reset.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum ResetType {
    /// Resets all the internal circuitry to its initial state.
    ///
    /// This is analogous to power cycling the device.
    Cold = 0,
    /// The processor is reset to its initial state.
    Warm,
    /// The components are powered off.
    Shutdown,
    /// A platform-specific reset type.
    ///
    /// The additional data must be a pointer to
    /// a null-terminated string followed by an UUID.
    PlatformSpecific,
    // SAFETY: This enum is never exposed to the user, but only fed as input to
    //         the firmware. Therefore, unexpected values can never come from
    //         the firmware, and modeling this as a Rust enum seems safe.
}
