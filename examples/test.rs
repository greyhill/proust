extern crate prust;
use self::prust::*;

fn go() -> Result<(), Error> {
    let platforms = try!(Platform::platforms());
    for p in platforms.iter() {
        println!("Platform: {} version {} from {}.  Extensions: {}", 
                 try!(p.name()), 
                 try!(p.version()),
                 try!(p.vendor()),
                 try!(p.extensions()));

        let devices = try!(p.devices());
        for d in devices.iter() {
            println!("\tIncludes device: {}.  Max work item sizes: {:?}.  Type: {:?}", 
                     try!(d.name()),
                     try!(d.max_work_item_sizes()),
                     try!(d.device_type()));
        }
        let ctx = try!(Context::new(&devices[..]));

        let mut queues: Vec<CommandQueue> = Vec::new();
        for d in devices.iter() {
            match CommandQueue::new(ctx.clone(), d.clone()) {
                Ok(q) => queues.push(q),
                Err(e) => println!("Error creating queue: {:?}", e),
            };
        }

        let program = try!(try!(Program::new_from_source(ctx, &["
        kernel void zero(global int* z) {
            z[get_global_id(0)] *= z[get_global_id(0)];
        }
        "][..])).build(&devices[..]));

        let mut buf = try!(queues[0].create_buffer(1024));
        let mut buf_host = vec!(1,2,3,4,5);

        try!(queues[0].write_buffer(&mut buf, &buf_host[..]));

        let mut k = try!(program.create_kernel("zero"));
        try!(k.bind_mut(0, &mut buf));
        try!(queues[1].run(&mut k, (1,1,1), (2,1,1)));
        try!(queues[0].read_buffer(&buf, &mut buf_host[..]));
        println!("{:?}", buf_host);
    }

    Ok(())
}

pub fn main() {
    match go() {
        Ok(()) => println!(":)"),
        Err(e) => println!("Error: {:?}", e),
    };
}

