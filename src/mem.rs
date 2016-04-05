use low_level as ll;
use error::Error;

pub struct Mem {
    pub id: ll::Mem,
}

impl Mem {
    pub fn new(id: ll::Mem) -> Mem {
        Mem{
            id: id,
        }
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
        Mem{ id: self.id }
    }
}

