use std::any::Any;
use std::boxed::Box;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::ptr::addr_of;
use std::sync::atomic::{AtomicU64, Ordering};
use super::node::Node;
use once_cell::sync::OnceCell;

pub struct Engine
{
    // uid -> pointer
    pub object_registry : HashMap<u64, Pin<Box<Node>>>,
}

pub static mut ENGINE_ROOT: OnceCell<Engine> = OnceCell::new();
pub static mut UID_GENERATOR : AtomicU64 = AtomicU64::new(0);

pub fn engine_init() {
    unsafe {
        ENGINE_ROOT.get_or_init(|| {Engine{object_registry:HashMap::new()}});
    }
}

pub fn engine_next_global_id() -> u64
{
    unsafe {
        UID_GENERATOR.fetch_add(1, Ordering::Acquire)
    }
}
pub fn engine_pin(id : u64, obj: Node) -> *const dyn Any {
    let pin = Box::pin(obj);
    let addr = addr_of!(*pin);
    unsafe {
        ENGINE_ROOT.get_mut().unwrap().object_registry.insert(id, pin);
    }
    addr
}

pub fn engine_cast_mut(addr : *const dyn Any) -> &'static mut Node {
    unsafe {
        &mut *(addr as *mut Node)
    }
}

pub fn engine_remove(addr : *const dyn Any) -> Pin<Box<Node>> {
    unsafe {
        let c = &*(addr as *const Node);
        let cid = c.id;
        ENGINE_ROOT.get_mut().unwrap().object_registry.remove(&cid).unwrap()
    }
}
