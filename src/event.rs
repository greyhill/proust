extern crate libc;
use self::libc::*;

use low_level as ll;
use error::Error;

use std::ptr;
use std::mem::{size_of, transmute};

/// OpenCL event
pub struct Event {
    pub id: ll::Event,
}

/// OpenCL event status
pub enum EventStatus {
    Queued,
    Submitted,
    Running,
    Complete
}

pub trait EventLike {
    fn status(self: &Self) -> Result<EventStatus, Error>;
    fn wait(self: &Self) -> Result<(), Error>;
    // TODO: add callbacks
    // fn then(self: &Self) -> Result<(), Error>;
}

unsafe impl Send for Event { }

impl EventStatus {
    pub fn from_code(code: i32) -> EventStatus {
        match code {
            0 => EventStatus::Complete,
            1 => EventStatus::Running,
            2 => EventStatus::Submitted,
            3 => EventStatus::Queued,
            _ => panic!("CL returned unknown event status code"),
        }
    }
}

impl EventLike for Event {
    fn status(self: &Self) -> Result<EventStatus, Error> {
        let mut code: i32 = 0;
        unsafe {
            try!(Error::check(ll::clGetEventInfo(self.id,
                                                0x11D3,
                                                size_of::<i32>() as size_t,
                                                transmute(&mut code),
                                                ptr::null_mut())));
        }
        Ok(EventStatus::from_code(code))
    }

    fn wait(self: &Self) -> Result<(), Error> {
        unsafe {
            try!(Error::check(ll::clWaitForEvents(1,
                                                  &self.id)));
        }
        Ok(())
    }
}

impl Drop for Event {
    fn drop(self: &mut Self) -> () {
        unsafe {
            Error::check(ll::clReleaseEvent(self.id))
                .ok().expect("error releasing event");
        }
    }
}

impl Clone for Event {
    fn clone(self: &Self) -> Self {
        unsafe {
            Error::check(ll::clRetainEvent(self.id))
                .ok().expect("error retaining event");
        }
        Event{ id: self.id }
    }
}

