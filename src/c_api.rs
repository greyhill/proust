use platform::*;
use context::*;
use command_queue::*;

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn Proust_NumPlatforms() -> usize {
    let platforms = Platform::platforms().expect("error getting platforms");
    platforms.len()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn Proust_CreateContextWithAllDevices(platform_number: usize) -> *const Context {
    let platform = &Platform::platforms().expect("error getting platforms")[platform_number];
    let devices = platform.devices().expect("error getting devices");
    let context = Context::new(&devices[..]).expect("error creating context");
    Box::into_raw(Box::new(context))
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn Proust_DestroyContext(ctx: *mut Context) -> () {
    Box::from_raw(ctx);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn Proust_ContextNumDevices(ctx: *const Context) -> usize {
    let ctx_ref = &*ctx;
    ctx_ref.devices().expect("error getting context devices").len()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn Proust_CreateCommandQueue(ctx: *mut Context, device_num: usize) -> *const CommandQueue {
    let ctx_ref = &*ctx;
    let device = ctx_ref.devices().expect("error getting context devices")[device_num].clone();
    match CommandQueue::new(ctx_ref.clone(), device) {
        Ok(ctx) => Box::into_raw(Box::new(ctx)),
        Err(e) => {
            println!("Error in Proust_CreateCommandQueue: {:?}", e);
            panic!("Error in Proust_CreateCommandQueue");
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn Proust_DestroyCommandQueue(queue: *mut CommandQueue) -> () {
    Box::from_raw(queue);
}

