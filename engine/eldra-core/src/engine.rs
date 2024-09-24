use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use once_cell::sync::OnceCell;
use std::ffi::CString;
use std::os::raw::c_char;
use std::rc::Rc;
use crate::comp::transform_component::TransformComponent;
use crate::entity::{Component, Entity};
use crate::reflection::{Boxed, Pinned, Serializable};

pub fn engine_init(drop_callback: ObjDropCallback) {
    unsafe {
        let mut _engine_ = engine_init_once__(drop_callback);
        let mut _clz_map_ = &_engine_.clz_map;
        //_clz_map_.insert(0, TransformComponent::boxed);
    }
}

type ObjDropCallback = unsafe extern "C" fn(clz: *const i8, id: u64);

pub struct Engine
{
    pub uid_generator : AtomicU64,
    // uid -> pointer
    pub object_registry : HashMap<u64, Pin<Rc<dyn Any>>>,

    pub on_obj_drop_callback: ObjDropCallback,

    pub clz_map : HashMap<u64, fn()>,
}
pub static mut ENGINE_ROOT: OnceCell<Engine> = OnceCell::new();
fn engine_init_once__(drop_callback: ObjDropCallback) -> &'static Engine {
    unsafe {
        ENGINE_ROOT.get_or_init(|| {
            Engine {
                uid_generator: AtomicU64::new(100),
                object_registry: HashMap::new(),
                on_obj_drop_callback: drop_callback,
                clz_map: HashMap::new(),
            }})
    }
}
pub fn engine_next_global_id() -> u64
{
    unsafe {
        ENGINE_ROOT.get_unchecked().uid_generator.fetch_add(1, Ordering::Acquire)
    }
}
pub fn engine_pin(id: u64, pin: Pin<Rc<dyn Any>>) {
    unsafe {
        ENGINE_ROOT.get_mut().unwrap().object_registry.insert(id, pin);
    }
}

pub fn engine_remove(id : u64) -> Option<Pin<Rc<dyn Any>>> {
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
        (ENGINE_ROOT.get_unchecked().on_obj_drop_callback)(c_str, id);
        drop_c_str(c_str);
    }
}