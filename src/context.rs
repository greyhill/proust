use low_level as ll;
use error::Error;
use device::Device;

use std::ptr;
use std::mem::transmute;

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
            let props: Vec<usize> = vec!(
                             0x1084, 
                             transmute(try!(devices[0].platform())));
            let id = ll::clCreateContext(transmute(&props[0]),
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

