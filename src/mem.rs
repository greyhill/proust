use low_level as ll;
use error::Error;
use event::{Event, EventStatus, EventLike};

use std::sync::Arc;
use std::cell::RefCell;

struct MemEvents {
    last_write: RefCell<Option<Event>>,
    last_read: RefCell<Vec<Event>>,
}

pub struct Mem {
    pub id: ll::Mem,
    events: Arc<MemEvents>,
}

impl Mem {
    pub fn new(id: ll::Mem) -> Mem {
        let events = MemEvents{
            last_write: RefCell::new(None),
            last_read: RefCell::new(Vec::new()),
        };
        Mem{
            id: id,
            events: Arc::new(events),
        }
    }

    pub fn write_events(self: &Self) -> Vec<ll::Event> {
        let mut tr: Vec<ll::Event> = Vec::with_capacity(2);
        if let Some(ref evt) = *self.events.last_write.borrow() {
            tr.push(evt.id);
        }
        tr.extend(self.events.last_read.borrow().iter().map(|e| e.id));
        tr
    }

    pub fn read_events(self: &Self) -> Vec<ll::Event> {
        if let Some(ref evt) = *self.events.last_write.borrow() {
            vec!(evt.id)
        } else {
            vec!()
        }
    }

    pub fn register_write(self: &mut Self, evt: Event) -> () {
        *self.events.last_write.borrow_mut() = Some(evt);
    }

    pub fn register_read(self: &Self, evt: Event) -> () {
        let mut r = self.events.last_read.borrow_mut();
        r.retain(|e| 
                 match e.status().ok().expect("error getting event status") {
                     EventStatus::Complete => false,
                     _ => true,
        });
        println!("buffer {:?} has {} events in its read list", self.id, r.len());
        r.push(evt);
    }
}

impl Drop for Mem {
    fn drop(self: &mut Self) -> () {
        unsafe {
            Error::check(ll::clReleaseMemObject(self.id))
                .ok().expect("error releasing mem object");
        }
    }
}

impl Clone for Mem {
    fn clone(self: &Self) -> Self {
        unsafe {
            Error::check(ll::clRetainMemObject(self.id))
                .ok().expect("error retaining mem object");
        }
        Mem{ id: self.id, events: self.events.clone() }
    }
}

