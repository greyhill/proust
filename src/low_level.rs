extern crate libc;
use self::libc::*;

pub type PlatformID = *mut c_void;
pub type DeviceID = *mut c_void;
pub type Context = *mut c_void;
pub type CommandQueue = *mut c_void;
pub type Mem = *mut c_void;
pub type Program = *mut c_void;
pub type Kernel = *mut c_void;
pub type Event = *mut c_void;
pub type Sampler = *mut c_void;

pub type PlatformInfo = u32;
pub type DeviceInfo = u32;

#[cfg(target_os = "macos")]
#[link(name = "OpenCL", kind = "framework")]
extern { }

#[cfg(target_os = "linux")]
#[link(name = "OpenCL")]
extern { }

#[cfg(target_os = "windows")]
#[link(name = "OpenCL")]
extern { }

extern {
    pub fn clGetPlatformIDs(num_entries: u32,
                            platforms: *mut PlatformID,
                            num_platforms: *mut u32) -> i32;

    pub fn clGetPlatformInfo(platform: PlatformID,
                             param_name: PlatformInfo,
                             value_size: size_t,
                             value: *mut c_void,
                             size_ret: *mut size_t) -> i32;

    pub fn clGetDeviceIDs(platform: PlatformID,
                          device_type: c_ulong,
                          num_devices_in: c_uint,
                          devices: *mut DeviceID,
                          num_devices_out: *mut c_uint) -> i32;

    pub fn clGetDeviceInfo(device: DeviceID,
                           param_name: DeviceInfo,
                           param_size_in: size_t,
                           value: *mut c_void,
                           param_size_out: *mut size_t) -> i32;

    pub fn clCreateContext(properties: *mut *mut c_void,
                           num_devices: u32,
                           devices: *const DeviceID,
                           callback: *const extern fn (*mut char, *const c_void, size_t, *const c_void) -> (),
                           user_data: *const c_void,
                           err_ret: *mut i32) -> Context;

    pub fn clRetainContext(ctx: Context) -> i32;

    pub fn clReleaseContext(ctx: Context) -> i32;

    pub fn clCreateCommandQueue(context: Context,
                                device: DeviceID,
                                props: c_ulong,
                                err: *mut i32) -> CommandQueue;

    pub fn clGetCommandQueueInfo(queue: CommandQueue,
                                 param_name: u32,
                                 param_size: size_t,
                                 param_value: *mut c_void,
                                 size_ret: *mut size_t) -> i32;

    pub fn clRetainCommandQueue(queue: CommandQueue) -> i32;

    pub fn clReleaseCommandQueue(queue: CommandQueue) -> i32;

    pub fn clCreateProgramWithSource(context: Context,
                                     num_lines: u32,
                                     lines: *const *const u8,
                                     lengths: *const size_t,
                                     err: *mut i32) -> Program;

    pub fn clBuildProgram(program: Program,
                          num_devices: u32,
                          devices: *const DeviceID,
                          options: *const u8,
                          callback: *const extern fn (Program, *mut c_void) -> (),
                          user_data: *mut c_void) -> i32;

    pub fn clRetainProgram(program: Program) -> i32;

    pub fn clReleaseProgram(program: Program) -> i32;

    pub fn clGetProgramBuildInfo(program: Program,
                                 device: DeviceID,
                                 parameter: u32,
                                 param_size: size_t,
                                 param_value: *mut c_void,
                                 param_size_ret: *mut size_t) -> i32;

    pub fn clCreateKernel(program: Program,
                          name: *const i8,
                          err: *mut i32) -> Kernel;

    pub fn clSetKernelArg(kernel: Kernel,
                          index: u32,
                          size: size_t,
                          value: *const c_void) -> i32;

    pub fn clRetainKernel(kernel: Kernel) -> i32;

    pub fn clReleaseKernel(kernel: Kernel) -> i32;

    pub fn clRetainEvent(event: Event) -> i32;

    pub fn clReleaseEvent(event: Event) -> i32;

    pub fn clGetEventInfo(event: Event,
                          param: u32,
                          param_size: size_t,
                          param_val: *mut c_void,
                          size_ret: *mut size_t) -> i32;

    pub fn clWaitForEvents(num_events: u32,
                           events: *const Event) -> i32;

    pub fn clRetainMemObject(mem: Mem) -> i32;

    pub fn clReleaseMemObject(mem: Mem) -> i32;

    pub fn clCreateBuffer(ctx: Context,
                          flags: c_ulong,
                          size: size_t,
                          host_ptr: *mut c_void,
                          error: *mut i32) -> Mem;

    pub fn clEnqueueWriteBuffer(queue: CommandQueue,
                                mem: Mem,
                                blocking: u32,
                                offset: size_t,
                                size: size_t,
                                host: *const c_void,
                                wait_size: u32,
                                wait_list: *const Event,
                                event: *mut Event) -> i32;

    pub fn clEnqueueReadBuffer(queue: CommandQueue,
                               mem: Mem,
                               blocking: u32,
                               offset: size_t,
                               size: size_t,
                               host: *mut c_void,
                               wait_size: u32,
                               wait_list: *const Event,
                               event: *mut Event) -> i32;

    pub fn clEnqueueNDRangeKernel(queue: CommandQueue,
                                  kernel: Kernel,
                                  dim: u32,
                                  global_offset: *const size_t,
                                  global_size: *const size_t,
                                  local_size: *const size_t,
                                  num_events: u32,
                                  events: *const Event,
                                  event: *mut Event) -> i32;
}

