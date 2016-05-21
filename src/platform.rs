extern crate libc;
use self::libc::*;

use low_level as ll;
use error::Error;
use device::Device;

use std::iter::repeat;
use std::ptr;
use std::mem::transmute;

/// OpenCL platform
///
/// Get a list of OpenCL platforms with `Platform::platforms()`, then query
/// devices with `devices()`.
pub struct Platform {
    pub id: ll::PlatformID,
}

impl Clone for Platform {
    fn clone(self: &Self) -> Self {
        Platform{ id: self.id }
    }
}

unsafe impl Send for Platform { }

impl Platform {
    /// Returns a vector of available platforms
    pub fn platforms() -> Result<Vec<Platform>, Error> {
        let num_platforms = unsafe {
            let mut tr: u32 = 0;
            try!(Error::check(ll::clGetPlatformIDs(0, ptr::null_mut(), &mut tr)));
            tr
        };

        let mut ids: Vec<ll::PlatformID> = repeat(ptr::null_mut())
            .take(num_platforms as usize).collect();

        unsafe {
            try!(Error::check(ll::clGetPlatformIDs(num_platforms,
                                                   &mut ids[0],
                                                   ptr::null_mut())))
        };

        // TODO use drain after stabilizes
        Ok(ids.iter().map(|&id| Platform{ id: id }).collect())
    }

    fn get_info(self: &Self, info: ll::PlatformInfo) -> Result<String, Error> {
        let size_req = unsafe {
            let mut tr: usize = 0;
            try!(Error::check(ll::clGetPlatformInfo(self.id,
                                                    info,
                                                    0,
                                                    ptr::null_mut(),
                                                    transmute(&mut tr))));
            tr
        };

        let mut buf: Vec<u8> = repeat(0).take(size_req).collect();

        unsafe {
            try!(Error::check(ll::clGetPlatformInfo(self.id,
                                                    info,
                                                    size_req as size_t,
                                                    transmute(&mut buf[0]),
                                                    ptr::null_mut())))
        };

        Ok(String::from_utf8(buf).ok().expect("CL returned invalid PlatformInfo"))
    }

    pub fn profile(self: &Self) -> Result<String, Error> {
        self.get_info(0x0900)
    }

    pub fn version(self: &Self) -> Result<String, Error> {
        self.get_info(0x0901)
    }

    pub fn name(self: &Self) -> Result<String, Error> {
        self.get_info(0x0902)
    }

    pub fn vendor(self: &Self) -> Result<String, Error> {
        self.get_info(0x0903)
    }

    pub fn extensions(self: &Self) -> Result<String, Error> {
        self.get_info(0x0904)
    }

    /// Returns a vector of devices on this platform
    pub fn devices(self: &Self) -> Result<Vec<Device>, Error> {
        let num_devices: usize = unsafe {
            let mut tr: c_uint = 0;
            try!(Error::check(ll::clGetDeviceIDs(self.id,
                                                 0xFFFFFFFF, // all types
                                                 0,
                                                 ptr::null_mut(),
                                                 &mut tr)));
            tr as usize
        };

        let mut device_ids: Vec<ll::DeviceID> = repeat(ptr::null_mut())
            .take(num_devices).collect();

        unsafe {
            try!(Error::check(ll::clGetDeviceIDs(self.id,
                                                 0xFFFFFFFF, // all types
                                                 num_devices as c_uint,
                                                 &mut device_ids[0],
                                                 ptr::null_mut())))
        };

        Ok(device_ids.iter().map(|&id| Device{id:id}).collect())
    }
}
