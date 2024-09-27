use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap};
use std::pin::Pin;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use once_cell::sync::OnceCell;
use std::ffi::CString;
use std::os::raw::c_char;
use std::rc::Rc;
use uuid::Uuid;
use crate::comp::transform_component::TransformComponent;
use crate::entity::{Component, Entity};
use crate::reflection::{init_reflection, Reflectable, Serializable};

pub fn engine_init(drop_callback: ObjDropCallback) {
    unsafe {
        engine_init_once__(drop_callback);
        init_reflection();
    }
}

type ObjDropCallback = unsafe extern "C" fn(clz: *const c_char, id: *const c_char);

pub struct Engine
{
    pub uid_generator : AtomicI64,

    // instance-id -> pointer
    pub object_registry : HashMap<Uuid, Pin<Rc<dyn Any>>>,

    pub on_obj_drop_callback: ObjDropCallback,
}
pub static mut ENGINE_ROOT: OnceCell<Engine> = OnceCell::new();
fn engine_init_once__(drop_callback: ObjDropCallback) -> &'static Engine {
    unsafe {
        ENGINE_ROOT.get_or_init (|| {
            Engine {
                uid_generator: AtomicI64::new(100),
                object_registry: HashMap::new(),
                on_obj_drop_callback: drop_callback,
            }})
    }
}
pub fn engine_next_global_id() -> i64
{
    unsafe {
        ENGINE_ROOT.get_unchecked().uid_generator.fetch_add(1, Ordering::Acquire)
    }
}
pub fn engine_pin(id: Uuid, pin: Pin<Rc<dyn Any>>) {
    unsafe {
        ENGINE_ROOT.get_mut().unwrap().object_registry.insert(id, pin);
    }
}

pub fn engine_remove(id : &Uuid) -> Option<Pin<Rc<dyn Any>>> {
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
pub fn engine_notify_drop_object(clz: &'static str, id : &Uuid) {
    let c_str = CString::new(clz).unwrap();
    let id_str = CString::new(id.to_string()).unwrap();
    unsafe {
        (ENGINE_ROOT.get_unchecked().on_obj_drop_callback)(c_str.as_ptr(), id_str.as_ptr());
    }
}