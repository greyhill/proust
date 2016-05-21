extern crate libc;
use self::libc::*;

use low_level as ll;
use error::Error;
use platform::Platform;

use std::iter::repeat;
use std::ptr;
use std::mem::{forget, size_of, transmute};
use std::slice;

pub struct Device {
    pub id: ll::DeviceID,
}

#[derive(Debug)]
pub enum DeviceType {
    CPU,
    GPU,
    Accelerator,
}

impl Clone for Device {
    fn clone(self: &Self) -> Self {
        Device{ id: self.id }
    }
}

unsafe impl Send for Device { }

impl Device {
    fn get_info(self: &Self, info: ll::DeviceInfo) -> Result<Vec<u8>, Error> {
        let size_req = unsafe {
            let mut tr: size_t = 0;
            try!(Error::check(ll::clGetDeviceInfo(self.id,
                                                    info,
                                                    0,
                                                    ptr::null_mut(),
                                                    &mut tr)));
            tr as usize
        };

        let mut bf: Vec<u8> = repeat(0).take(size_req).collect();

        unsafe {
            try!(Error::check(ll::clGetDeviceInfo(self.id,
                                                   info,
                                                   size_req as size_t,
                                                   transmute(&mut bf[0]),
                                                   ptr::null_mut())))
        };

        Ok(bf)
    }

    fn get_info_string(self: &Self, info: ll::DeviceInfo) -> Result<String, Error> {
        Ok(String::from_utf8(try!(self.get_info(info)))
            .ok().expect("CL returned invalid string"))
    }

    fn get_info_scalar<T: Sized + Copy>(self: &Self, info: ll::DeviceInfo) -> Result<T, Error> {
        let buf = try!(self.get_info(info));
        assert_eq!(buf.len(), size_of::<T>());
        let scalar: T = unsafe {
            let b_ptr: *const T = transmute(&buf[0]);
            let slice = slice::from_raw_parts(b_ptr, 1);
            let tr = slice[0].clone();
            forget(slice);
            tr
        };
        Ok(scalar)
    }

    pub fn device_type(self: &Self) -> Result<DeviceType, Error> {
        let cl: usize = try!(self.get_info_scalar(0x1000));
        match cl {
            2 => Ok(DeviceType::CPU),
            4 => Ok(DeviceType::GPU),
            8 => Ok(DeviceType::Accelerator),
            _ => panic!("Unknown device type"),
        }
    }

    pub fn vendor_id(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x1001)
    }

    pub fn max_compute_units(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x1002)
    }

    pub fn max_work_item_dimensions(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x1003)
    }

    pub fn max_work_group_size(self: &Self) -> Result<usize, Error> {
        self.get_info_scalar(0x1004)
    }

    pub fn max_work_item_sizes(self: &Self) -> Result<Vec<usize>, Error> {
        let b = try!(self.get_info(0x1005));
        let p0: *const u8 = &b[0];
        let tr: Vec<usize> = unsafe {
            let sl = slice::from_raw_parts(transmute(p0),
                                           b.len() / size_of::<usize>());
            sl.to_vec()
        };
        Ok(tr)
    }

    pub fn preferred_vector_width_char(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x1006)
    }

    pub fn preferred_vector_width_short(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x1007)
    }

    pub fn preferred_vector_width_int(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x1008)
    }

    pub fn preferred_vector_width_long(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x1009)
    }

    pub fn preferred_vector_width_float(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x100A)
    }

    pub fn preferred_vector_width_double(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x100B)
    }

    pub fn max_clock_frequency(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x100C)
    }

    pub fn address_bits(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x100D)
    }

    pub fn max_read_image_args(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x100E)
    }

    pub fn max_write_image_args(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x100F)
    }

    pub fn max_mem_alloc_size(self: &Self) -> Result<usize, Error> {
        let cl: c_ulong = try!(self.get_info_scalar(0x1010));
        Ok(cl as usize)
    }

    pub fn image2d_max_width(self: &Self) -> Result<usize, Error> {
        self.get_info_scalar(0x1011)
    }

    pub fn image2d_max_height(self: &Self) -> Result<usize, Error> {
        self.get_info_scalar(0x1012)
    }

    pub fn image3d_max_width(self: &Self) -> Result<usize, Error> {
        self.get_info_scalar(0x1013)
    }

    pub fn image3d_max_height(self: &Self) -> Result<usize, Error> {
        self.get_info_scalar(0x1014)
    }

    pub fn image3d_max_depth(self: &Self) -> Result<usize, Error> {
        self.get_info_scalar(0x1015)
    }

    pub fn image_support(self: &Self) -> Result<bool, Error> {
        Ok(try!(self.get_info_scalar::<u32>(0x1016)) == 1)
    }

    pub fn max_parameter_size(self: &Self) -> Result<usize, Error> {
        self.get_info_scalar(0x1017)
    }

    pub fn max_samplers(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x1018)
    }

    pub fn mem_base_addr_align(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x1019)
    }

    pub fn min_data_type_align_size(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x101A)
    }

    pub fn global_mem_cacheline_size(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x101D)
    }

    pub fn global_mem_cache_size(self: &Self) -> Result<usize, Error> {
        Ok(try!(self.get_info_scalar::<c_ulong>(0x101E)) as usize)
    }

    pub fn global_mem_size(self: &Self) -> Result<usize, Error> {
        Ok(try!(self.get_info_scalar::<c_ulong>(0x101F)) as usize)
    }

    pub fn max_constant_buffer_size(self: &Self) -> Result<usize, Error> {
        Ok(try!(self.get_info_scalar::<c_ulong>(0x1020)) as usize)
    }

    pub fn max_constant_args(self: &Self) -> Result<u32, Error> {
        self.get_info_scalar(0x1021)
    }

    pub fn local_mem_size(self: &Self) -> Result<usize, Error> {
        Ok(try!(self.get_info_scalar::<c_ulong>(0x1023)) as usize)
    }

    pub fn endian_little(self: &Self) -> Result<bool, Error> {
        Ok(try!(self.get_info_scalar::<u32>(0x1026)) == 1)
    }

    pub fn available(self: &Self) -> Result<bool, Error> {
        Ok(try!(self.get_info_scalar::<u32>(0x1027)) == 1)
    }

    pub fn compiler_available(self: &Self) -> Result<bool, Error> {
        Ok(try!(self.get_info_scalar::<u32>(0x1028)) == 1)
    }

    pub fn out_of_order_supported(self: &Self) -> Result<bool, Error> {
        let v: u64 = try!(self.get_info_scalar(0x102A));
        let r: u64 = v as u64 & 1u64;
        Ok(r > 0)
    }

    pub fn name(self: &Self) -> Result<String, Error> {
        self.get_info_string(0x102B)
    }

    pub fn vendor(self: &Self) -> Result<String, Error> {
        self.get_info_string(0x102C)
    }

    pub fn profile(self: &Self) -> Result<String, Error> {
        self.get_info_string(0x102E)
    }

    pub fn driver_version(self: &Self) -> Result<String, Error> {
        self.get_info_string(0x102D)
    }

    pub fn version(self: &Self) -> Result<String, Error> {
        self.get_info_string(0x102F)
    }

    pub fn extensions(self: &Self) -> Result<String, Error> {
        self.get_info_string(0x102E)
    }

    pub fn platform(self: &Self) -> Result<Platform, Error> {
        let id = try!(self.get_info_scalar::<ll::PlatformID>(0x1031));
        Ok(Platform{id:id})
    }
}
