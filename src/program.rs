extern crate libc;
use self::libc::*;

use low_level as ll;
use context::Context;
use error::Error;
use device::Device;
use kernel::Kernel;

use std::mem::{transmute, forget};
use std::ptr;
use std::iter::repeat;
use std::ffi::CString;

/// Uncompiled OpenCL program
///
/// To produce a compiled OpenCL program, use `Program::new_from_source()`,
/// then call `build()`.
pub struct Program {
    pub id: ll::Program,
}

/// Compiled OpenCL program
///
/// To produce a compiled OpenCL program, use `Program::new_from_source()`,
/// then call `build()`.
pub struct BuiltProgram {
    pub id: ll::Program,
}

impl Program {
    pub fn new_from_source<T: AsRef<str>>(ctx: Context, 
                                           lines: &[T]) -> Result<Program, Error> {
        let num_lines = lines.len() as u32;
        let cstrings: Vec<CString> = lines.iter().map(|l| CString::new(l.as_ref()).unwrap()).collect();
        let lengths: Vec<size_t> = lines.iter().map(|l| l.as_ref().len() as size_t).collect();
        let lines_raw: Vec<*const libc::c_char> = cstrings.iter().map(|l| l.as_ptr()).collect();
        let id = unsafe {
            let mut err = 0;
            let id = ll::clCreateProgramWithSource(ctx.id,
                                                   num_lines,
                                                   transmute(&lines_raw[0]),
                                                   transmute(&lengths[0]),
                                                   &mut err);
            try!(Error::check(err));
            id
        };
        Ok(Program{id: id})
    }

    pub fn build(self: Self,
                devices: &[Device]) -> Result<BuiltProgram, Error> {
        let num_devices = devices.len() as u32;
        let device_ids: Vec<ll::DeviceID> = devices.iter().map(|d| d.id).collect();
        unsafe {
            match Error::check(ll::clBuildProgram(self.id,
                                                  num_devices,
                                                  &device_ids[0],
                                                  ptr::null(),
                                                  ptr::null(),
                                                  ptr::null_mut())) {
                Ok(()) => {},
                Err(Error::BuildProgramFailure(_)) => {
                    // need to get build log.  this is a little 
                    // unpleasant.
                    let size_req = {
                        let mut tr: size_t = 0;
                        try!(Error::check(ll::clGetProgramBuildInfo(self.id,
                                                                    devices[0].id,
                                                                    0x1183,
                                                                    0,
                                                                    ptr::null_mut(),
                                                                    &mut tr)));
                        tr as usize
                    };
                    let mut buf: Vec<u8> = repeat(0).take(size_req).collect();
                    try!(Error::check(ll::clGetProgramBuildInfo(self.id,
                                                                devices[0].id,
                                                                0x1183,
                                                                size_req as size_t,
                                                                transmute(&mut buf[0]),
                                                                ptr::null_mut())));
                    return Err(Error::BuildProgramFailure(
                            String::from_utf8(buf)
                            .ok().expect("build error but build log not valid utf8?")));
                },
                Err(e) => {
                    return Err(e);
                },
            }
        };
        let id = self.id;
        forget(self);
        Ok(BuiltProgram{id: id})
    }
}

impl Drop for Program {
    fn drop(self: &mut Self) -> () {
        unsafe {
            Error::check(ll::clReleaseProgram(self.id))
                .ok().expect("error releasing program");
        }
    }
}

impl Clone for Program {
    fn clone(self: &Self) -> Self {
        unsafe {
            Error::check(ll::clRetainProgram(self.id))
                .ok().expect("error retaining program");
        }
        Program{ id: self.id }
    }
}

impl BuiltProgram {
    /// Returns the build log from the OpenCL compiler
    pub fn log(self: &Self, 
               device: Device) -> Result<String, Error> {
        let size_req = unsafe {
            let mut tr: size_t = 0;
            try!(Error::check(ll::clGetProgramBuildInfo(self.id,
                                                        device.id,
                                                        0x1183,
                                                        0,
                                                        ptr::null_mut(),
                                                        &mut tr)));
            tr as usize
        };
        let mut buf: Vec<u8> = repeat(0).take(size_req).collect();
        unsafe {
            try!(Error::check(ll::clGetProgramBuildInfo(self.id,
                                                        device.id,
                                                        0x1183,
                                                        size_req as size_t,
                                                        transmute(&mut buf[0]),
                                                        ptr::null_mut())))
        };
        Ok(String::from_utf8(buf).ok().expect("CL returned invalid string"))
    }

    /// Create an OpenCL kernel
    pub fn create_kernel<T: AsRef<str>>(self: &Self,
                                        name: T) -> Result<Kernel, Error> {
        let mut err: i32 = 0;
        let name_str = CString::new(name.as_ref())
                .ok().expect("kernel name is invalid c string");
        let id = unsafe { 
            ll::clCreateKernel(self.id,
                               name_str.as_ptr(),
                               &mut err)
        };
        try!(Error::check(err));
        Ok(Kernel::new(id))
    }
}

impl Drop for BuiltProgram {
    fn drop(self: &mut Self) -> () {
        unsafe {
            Error::check(ll::clReleaseProgram(self.id))
                .ok().expect("error releasing program");
        }
    }
}

impl Clone for BuiltProgram {
    fn clone(self: &Self) -> Self {
        unsafe { 
            Error::check(ll::clRetainProgram(self.id))
                .ok().expect("error retaining program");
        }
        BuiltProgram{ id: self.id }
    }
}

