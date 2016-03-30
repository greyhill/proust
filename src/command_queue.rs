extern crate libc;
use self::libc::*;

use low_level as ll;
use error::Error;
use device::Device;
use context::Context;
use event::{Event, EventStatus, EventLike};
use kernel::Kernel;
use mem::Mem;

use std::ptr;
use std::mem::{size_of, transmute};

pub struct CommandQueue {
    pub id: ll::CommandQueue,
}

pub struct ReadLock<'a, T: Sized + 'a> {
    #[allow(dead_code)]
    sl: &'a [T],
    evt: Event,
}

pub struct WriteLock<'a, T: Sized + 'a> {
    #[allow(dead_code)]
    sl: &'a mut [T],
    evt: Event,
}

impl<'a, T: Sized> Drop for ReadLock<'a, T> {
    fn drop(self: &mut Self) -> () {
        self.evt.wait().ok().expect(
            "error waiting to finish write for ReadLock");
    }
}

impl<'a, T: Sized> EventLike for ReadLock<'a, T> {
    fn status(self: &Self) -> Result<EventStatus, Error> {
        self.evt.status()
    }

    fn wait(self: &Self) -> Result<(), Error> {
        self.evt.wait()
    }
}

impl<'a, T: Sized> Drop for WriteLock<'a, T> {
    fn drop(self: &mut Self) -> () {
        self.evt.wait().ok().expect(
            "error waiting to finish write for ReadLock");
    }
}

impl<'a, T: Sized> EventLike for WriteLock<'a, T> {
    fn status(self: &Self) -> Result<EventStatus, Error> {
        self.evt.status()
    }

    fn wait(self: &Self) -> Result<(), Error> {
        self.evt.wait()
    }
}

unsafe impl Send for CommandQueue { }

impl CommandQueue {
    pub fn new(context: Context,
               device: Device) -> Result<CommandQueue, Error> {
        let props: c_ulong = if try!(device.out_of_order_supported()) { 
            1 
        } else { 
            0 
        }; 
        let id = unsafe {
            let mut err: i32 = 0;
            let id = ll::clCreateCommandQueue(context.id,
                                              device.id,
                                              props,
                                              &mut err);
            try!(Error::check(err));
            id
        };
        Ok(CommandQueue{ id: id })
    }

    fn get_info(self: &Self, param: u32) -> Result<*mut c_void, Error> {
        let mut tr: *mut c_void = ptr::null_mut();
        unsafe {
            try!(Error::check(ll::clGetCommandQueueInfo(self.id,
                                                   param,
                                                   size_of::<*mut c_void>() as size_t,
                                                   transmute(&mut tr),
                                                   ptr::null_mut())));
        }
        Ok(tr)
    }

    pub fn context(self: &Self) -> Result<Context, Error> {
        let val = try!(self.get_info(0x1090));
        let id = unsafe { transmute(val) };
        unsafe {
            try!(Error::check(ll::clRetainContext(id)));
        }
        Ok(Context{id: id})
    }

    pub fn device(self: &Self) -> Result<Device, Error> {
        let val = try!(self.get_info(0x1091));
        let id = unsafe { transmute(val) };
        Ok(Device{id: id})
    }

    fn alloc_ll_buffer(self: &Self, 
                    flags: c_ulong,
                    size_bytes: usize) -> Result<ll::Mem, Error> {
        let context = try!(self.context());
        let id = unsafe {
            let mut err: i32 = 0;
            let id = ll::clCreateBuffer(context.id,
                                        flags,
                                        size_bytes as size_t,
                                        ptr::null_mut(),
                                        &mut err);
            try!(Error::check(err));
            id
        };
        Ok(id)
    }

    fn alloc_ll_buffer_with_data(self: &Self,
                                 flags: c_ulong,
                                 size_bytes: usize,
                                 data: *const c_void) -> Result<ll::Mem, Error> {
        let context = try!(self.context());
        let id = unsafe {
            let mut err: i32 = 0;
            let id = ll::clCreateBuffer(context.id,
                                        flags | (1 << 5),
                                        size_bytes as size_t,
                                        data,
                                        &mut err);
            try!(Error::check(err));
            id
        };
        Ok(id)
    }

    fn write_buffer_raw<T: Sized>(self: &Self, 
                       buf: &mut Mem,
                       offset: usize,
                       size: usize,
                       data: &[T]) -> Result<Event, Error> {
        let mut event_id: ll::Event = ptr::null_mut();
        let host_ptr = &data[0] as *const T;
        let wait_list: Vec<ll::Event> = buf.write_events();
        let wait_size = wait_list.len() as u32;

        let wait_list_ptr: *const ll::Event = match wait_size {
            0 => ptr::null(),
            _ => &wait_list[0],
        };

        unsafe {
            try!(Error::check(ll::clEnqueueWriteBuffer(self.id,
                                                       buf.id,
                                                       0, // don't block
                                                       offset as size_t,
                                                       size as size_t,
                                                       transmute(host_ptr),
                                                       wait_size,
                                                       wait_list_ptr,
                                                       &mut event_id)));
        }

        let event = Event{id: event_id};
        buf.register_write(event.clone());
        Ok(event)
    }

    fn read_buffer_raw<T: Sized>(self: &Self,
                                 buf: &Mem,
                                 offset: usize,
                                 size: usize,
                                 data: &mut [T]) -> Result<Event, Error> {
        let mut event_id: ll::Event = ptr::null_mut();
        let host_ptr = &mut data[0] as *mut T;
        let wait_list: Vec<ll::Event> = buf.read_events();
        let wait_size = wait_list.len() as u32;

        let wait_list_ptr: *const ll::Event = match wait_size {
            0 => ptr::null(),
            _ => &wait_list[0],
        };

        unsafe {
            try!(Error::check(ll::clEnqueueReadBuffer(self.id,
                                                      buf.id,
                                                      0, // don't block,
                                                      offset as size_t,
                                                      size as size_t,
                                                      transmute(host_ptr),
                                                      wait_size,
                                                      wait_list_ptr,
                                                      &mut event_id)));
        }

        let event = Event{id: event_id};
        buf.register_read(event.clone());
        Ok(event)
    }

    pub fn create_buffer(self: &Self, 
                               size_bytes: usize) -> Result<Mem, Error> {
        let id = try!(self.alloc_ll_buffer(0, size_bytes));
        let tr = Mem::new(id);
        Ok(tr)
    }

    pub fn create_buffer_from_slice<T: Sized>(self: &Self,
                                    slice: &[T]) -> Result<Mem, Error> {
        let num_bytes = slice.len() * size_of::<T>();
        let id = try!(self.alloc_ll_buffer_with_data(0, num_bytes, unsafe { transmute(&slice[0]) } ));
        let tr = Mem::new(id);
        Ok(tr)
    }

    pub fn write_buffer<'a, 'b, T: Sized + 'a>(self: &Self,
                                  mem: &'b mut Mem,
                                  slice: &'a [T]) -> Result<ReadLock<'a, T>, Error> {
        let size_bytes = size_of::<T>()*slice.len();
        let evt = try!(self.write_buffer_raw(mem,
                                             0,
                                             size_bytes,
                                             slice));
        Ok(ReadLock{ evt: evt, sl: slice })
    }

    pub fn read_buffer<'a, T: Sized>(self: &Self,
                                 mem: &Mem,
                                 slice: &'a mut [T]) -> Result<WriteLock<'a, T>, Error> {
        let size_bytes = size_of::<T>()*slice.len();
        let evt = try!(self.read_buffer_raw(mem,
                                            0,
                                            size_bytes,
                                            slice));
        Ok(WriteLock{ evt: evt, sl: slice })
    }

    fn next_mul(self: &Self, global: usize, local: usize) -> usize {
        if global % local == 0 {
            global
        } else {
            ((global/local)+1)*local
        }
    }

    pub fn run( self: &Self, 
                kernel: &mut Kernel, 
                local_size: (usize, usize, usize),
                global_size: (usize, usize, usize)) -> Result<Event, Error> {
        let events = kernel.run_events();
        let global_size_fixed = (self.next_mul(global_size.0, local_size.0),
                                self.next_mul(global_size.1, local_size.1),
                                self.next_mul(global_size.2, local_size.2));

        let local_vec = vec!(local_size.0 as size_t, 
                             local_size.1 as size_t, 
                             local_size.2 as size_t);
        let global_vec = vec!(global_size_fixed.0 as size_t, 
                              global_size_fixed.1 as size_t,  
                              global_size_fixed.2 as size_t);

        let mut event_id: ll::Event = ptr::null_mut();
        unsafe {
            try!(Error::check(ll::clEnqueueNDRangeKernel(self.id,
                                                         kernel.id,
                                                         3,
                                                         ptr::null(),
                                                         &global_vec[0],
                                                         &local_vec[0],
                                                         events.len() as u32,
                                                         &events[0],
                                                         &mut event_id)));
        }
        let event = Event{id: event_id};

        try!(kernel.register_run(event.clone()));
        Ok(event)
    }
}

impl Drop for CommandQueue {
    fn drop(self: &mut Self) -> () {
        unsafe { 
            Error::check(ll::clReleaseCommandQueue(self.id))
                .ok().expect("Error releasing command queue");
        }
    }
}

impl Clone for CommandQueue {
    fn clone(self: &Self) -> Self {
        unsafe { 
            Error::check(ll::clRetainCommandQueue(self.id))
                .ok().expect("Error retaining command queue");
        }
        CommandQueue{ id: self.id }
    }
}

