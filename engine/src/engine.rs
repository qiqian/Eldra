use std::any::Any;
use std::boxed::Box;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use super::node::Node;
use once_cell::sync::OnceCell;

pub struct Engine
{
    // uid -> pointer
    pub object_registry : HashMap<u64, RefCell<Pin<Box<Node>>>>,
}

pub static mut ENGINE_ROOT: OnceCell<Engine> = OnceCell::new();
pub static mut UID_GENERATOR : AtomicU64 = AtomicU64::new(0);

pub fn engine_init() {
    unsafe {
        ENGINE_ROOT.get_or_init(|| {Engine{object_registry:HashMap::new()}});
    }
}

pub fn next_global_id() -> u64
{
    unsafe {
        UID_GENERATOR.fetch_add(1, Ordering::Acquire)
    }
}
pub fn pin_object(id : u64, pin: RefCell<Pin<Box<Node>>>) {
    unsafe {
        ENGINE_ROOT.get_mut().unwrap().object_registry.insert(id, pin);
    }
}