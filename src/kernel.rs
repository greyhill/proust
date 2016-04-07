extern crate libc;
use self::libc::*;

use low_level as ll;
use error::Error;
use mem::Mem;

use std::ptr;
use std::mem::{transmute, size_of};

pub struct Kernel {
    pub id: ll::Kernel,
}

impl Drop for Kernel {
    fn drop(self: &mut Self) -> () {
        unsafe {
            Error::check(ll::clReleaseKernel(self.id))
                .ok().expect("error releasing kernel")
        };
    }
}

impl Clone for Kernel {
    fn clone(self: &Self) -> Self {
        unsafe {
            Error::check(ll::clRetainKernel(self.id))
                .ok().expect("error retaining kernel")
        };
        Kernel{ 
            id: self.id, 
        }
    }
}

unsafe impl Send for Kernel { }

impl Kernel {
    pub fn new(id: ll::Kernel) -> Kernel {
        Kernel{
            id: id,
        }
    }

    pub fn bind_null(self: &mut Self, index: u32) -> Result<(), Error> {
        unsafe {
            try!(Error::check(ll::clSetKernelArg(self.id, index, size_of::<ll::Mem>() as size_t, ptr::null_mut())));
        }
        Ok(())
    }

    pub fn bind_scalar<T: Sized>(self: &mut Self, index: u32, val: &T) -> Result<(), Error> {
        unsafe {
            try!(Error::check(ll::clSetKernelArg(self.id, index, size_of::<T>() as size_t, transmute(val))));
        }
        Ok(())
    }

    pub fn bind(self: &mut Self, index: u32, buf: &Mem) -> Result<(), Error> {
        unsafe {
            try!(Error::check(ll::clSetKernelArg(self.id, index, size_of::<ll::Mem>() as size_t, transmute(&buf.id))));
        }
        Ok(())
    }

    pub fn bind_mut(self: &mut Self, index: u32, buf: &mut Mem) -> Result<(), Error> {
        unsafe {
            try!(Error::check(ll::clSetKernelArg(self.id, index, size_of::<ll::Mem>() as size_t, transmute(&buf.id))));
        }
        Ok(())
    }
}

