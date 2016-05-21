fn main() {
    // special case: sometimes the OpenCL library is squirreled away in C:\Windows\system32
    if cfg!(windows) {
        println!("cargo:rustc-link-search=C:\\Windows\\system32")
    }
}
