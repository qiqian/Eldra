use std::any::Any;
use std::boxed::Box;
use std::collections::{HashMap};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use once_cell::sync::OnceCell;

pub struct Engine
{
    pub uid_generator : AtomicU64,
    // uid -> pointer
    pub object_registry : HashMap<u64, Pin<Box<dyn Any>>>,
}
pub static mut ENGINE_ROOT: OnceCell<Engine> = OnceCell::new();

pub fn engine_init() {
    unsafe {
        ENGINE_ROOT.get_or_init(|| {
            Engine{
                uid_generator : AtomicU64::new(100),
                object_registry:HashMap::new()
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
