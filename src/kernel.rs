extern crate libc;
use self::libc::*;

use low_level as ll;
use error::Error;
use event::Event;
use mem::Mem;

use std::mem::{transmute, size_of};

pub struct Kernel {
    pub id: ll::Kernel,
    write_args: Vec<Mem>,
    read_args: Vec<Mem>,
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
            write_args: self.write_args.clone(),
            read_args: self.read_args.clone()
        }
    }
}

unsafe impl Send for Kernel { }

impl Kernel {
    pub fn new(id: ll::Kernel) -> Kernel {
        Kernel{
            id: id,
            write_args: Vec::new(),
            read_args: Vec::new(),
        }
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
        self.read_args.push(buf.clone());
        Ok(())
    }

    pub fn bind_mut(self: &mut Self, index: u32, buf: &mut Mem) -> Result<(), Error> {
        unsafe {
            try!(Error::check(ll::clSetKernelArg(self.id, index, size_of::<ll::Mem>() as size_t, transmute(&buf.id))));
        }
        self.write_args.push(buf.clone());
        Ok(())
    }

    pub fn run_events(self: &Self) -> Vec<ll::Event> {
        let mut tr = Vec::new();
        for b in self.read_args.iter() {
            tr.extend(b.read_events().iter())
        }
        for b in self.write_args.iter() {
            tr.extend(b.write_events().iter())
        }
        tr
    }

    pub fn register_run(self: &mut Self, event: Event) -> Result<(), Error> {
        for b in self.read_args.iter_mut() {
            b.register_read(event.clone())
        }
        self.read_args.clear();
        for b in self.write_args.iter_mut() {
            b.register_write(event.clone())
        }
        self.write_args.clear();
        Ok(())
    }
}

