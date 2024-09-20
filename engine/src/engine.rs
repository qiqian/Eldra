use std::any::Any;
use std::boxed::Box;
use std::collections::{HashMap};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use once_cell::sync::OnceCell;
use std::os::raw::{c_int};
use std::ffi::CString;
use std::ffi::CStr;
use std::os::raw::c_char;

type ObjDropCallback = unsafe extern "C" fn(clz: *const i8, id: u64);

pub struct Engine
{
    pub uid_generator : AtomicU64,
    // uid -> pointer
    pub object_registry : HashMap<u64, Pin<Box<dyn Any>>>,

    pub onObjDropCallback: ObjDropCallback,
}
pub static mut ENGINE_ROOT: OnceCell<Engine> = OnceCell::new();

pub fn engine_init(dropCallback: ObjDropCallback) {
    unsafe {
        ENGINE_ROOT.get_or_init(|| {
            Engine{
                uid_generator : AtomicU64::new(100),
                object_registry:HashMap::new(),
                onObjDropCallback:dropCallback,
            }});
    }
}

pub fn engine_next_global_id() -> u64
{
    unsafe {
        ENGINE_ROOT.get_unchecked().uid_generator.fetch_add(1, Ordering::Acquire)
    }
}
pub fn engine_pin(id: u64, pin: Pin<Box<dyn Any>>) {
    unsafe {
        ENGINE_ROOT.get_mut().unwrap().object_registry.insert(id, pin);
    }
}

pub fn engine_remove(id : u64) -> Option<Pin<Box<dyn Any>>> {
    unsafe {
        ENGINE_ROOT.get_mut().unwrap().object_registry.remove(&id)
    }
}

pub unsafe fn convert_c_str(input: &str) -> *mut c_char {
    let c_str = CString::new(input).unwrap().into_raw();
    return c_str;
}
pub unsafe fn drop_c_str(c_str: *mut c_char) {
    drop(CString::from_raw(c_str));
}
pub fn engine_notify_drop_object(clz: &'static str, id : u64) {
    unsafe {
        let c_str = convert_c_str(clz);
        (ENGINE_ROOT.get_unchecked().onObjDropCallback)(c_str, id);
        drop_c_str(c_str);
    }
}