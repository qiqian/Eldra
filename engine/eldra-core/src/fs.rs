use core::ffi::c_char;
use std::ptr::addr_of;

extern "C" {
    fn fopen_s(stream: *const *const i8, filename: *const c_char, mode: *const u8) -> i32;
    fn fwrite(ptr: *const u8, size: usize, count: usize, stream: *const i8) -> usize;
    fn fclose(stream: *const i8) -> i32;
}

static CONST_WRITE_MODE: &[u8; 5] = b"w+bc\0";

pub fn write_file(path: *const c_char, data: &[u8])
{
    unsafe {
        let f :*const i8 = 0 as *const i8;
        let _fr = fopen_s(addr_of!(f), path, CONST_WRITE_MODE.as_ptr());
        let cnt = fwrite(data.as_ptr(), data.len(), 1, f);
        let r = fclose(f);
        println!("{:?}, write {} = {}/{}", f, data.len(), cnt, r);
        // sleep(time::Duration::from_secs(10));
    }
}