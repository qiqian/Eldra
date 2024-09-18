use std::boxed::Box;
use std::cell::{RefCell};
use std::collections::{HashMap};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use once_cell::sync::OnceCell;
use super::node::Node;

pub struct Engine
{
    pub uid_generator : AtomicU64,
    // uid -> pointer
    pub object_registry : HashMap<u64, Pin<Box<Rc<RefCell<Node>>>>>,
}
pub static mut ENGINE_ROOT: OnceCell<Engine> = OnceCell::new();

pub fn engine_init() {
    unsafe {
        ENGINE_ROOT.get_or_init(|| {
            Engine{
                uid_generator : AtomicU64::new(0),
                object_registry:HashMap::new()
            }});
    }
}

pub fn engine_next_global_id() -> u64
{
    unsafe {
        ENGINE_ROOT.get_mut().unwrap().uid_generator.fetch_add(1, Ordering::Acquire)
    }
}
pub fn engine_pin(pin: Pin<Box<Rc<RefCell<Node>>>>) {
    unsafe {
        let id = pin.borrow().id;
        ENGINE_ROOT.get_mut().unwrap().object_registry.insert(id, pin);
    }
}

pub fn engine_cast(addr : u64) -> &'static mut Rc<RefCell<Node>> {
    unsafe {
        &mut *(addr as *mut Rc<RefCell<Node>>)
    }
}

pub fn engine_remove(addr : u64) -> Pin<Box<Rc<RefCell<Node>>>> {
    unsafe {
        let cid = engine_cast(addr).borrow().id;
        ENGINE_ROOT.get_mut().unwrap().object_registry.remove(&cid).unwrap()
    }
}
