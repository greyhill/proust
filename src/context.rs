extern crate libc;
use self::libc::*;

use low_level as ll;
use error::Error;
use device::Device;

use std::ptr;
use std::mem;

/// OpenCL context
///
/// To create a new context:
///
/// 1. Get a list of platforms with `Platform::platforms()` and select one.
///
/// 2. Get a list of devices from the selected platform with `.devices()`
///
/// 3. Pass the list of devices to `Context::new()`.
pub struct Context {
    pub id: ll::Context,
}

impl Context {
    pub fn new(devices: &[Device]) -> Result<Context, Error> {
        let device_ids: Vec<ll::DeviceID> = devices.iter().map(|d| d.id).collect();
        let num_devices = device_ids.len() as u32;
        let callback = ptr::null(); // TODO ?
        let user_data = ptr::null(); // TODO ?
        let id = unsafe {
            let mut err: i32 = 0;
            let id = ll::clCreateContext(ptr::null_mut(),
                                         num_devices,
                                         &device_ids[0],
                                         callback,
                                         user_data,
                                         &mut err);
            try!(Error::check(err));
            id
        };
        Ok(Context{id:id})
    }

    pub fn devices(self: &Self) -> Result<Vec<Device>, Error> {
        let num_devices = unsafe {
            let mut tr: size_t = 0;
            try!(Error::check(ll::clGetContextInfo(self.id,
                                                   0x1081,
                                                   0,
                                                   ptr::null_mut(),
                                                   &mut tr)));
            tr as usize / mem::size_of::<ll::DeviceID>()
        };

        let mut device_ids: Vec<ll::DeviceID> = 
            (0..num_devices).map(|_| ptr::null_mut()).collect();

        unsafe {
            try!(Error::check(ll::clGetContextInfo(self.id,
                                                   0x1081,
                                                   num_devices * mem::size_of::<ll::DeviceID>(),
                                                   mem::transmute(&mut device_ids[0]),
                                                   ptr::null_mut())));
        }

        Ok(device_ids.iter().map(|&id| Device{id:id}).collect())
    }
}

impl Drop for Context {
    fn drop(self: &mut Self) -> () {
        unsafe { 
            Error::check(ll::clReleaseContext(self.id))
                .ok().expect("Error releasing context");
        }
    }
}

impl Clone for Context {
    fn clone(self: &Self) -> Self {
        unsafe { 
            Error::check(ll::clRetainContext(self.id))
                .ok().expect("Error retaining context");
        }
        Context{ id: self.id }
    }
}

unsafe impl Send for Context {
}

