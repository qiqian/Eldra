use std::any::Any;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::pin::{pin, Pin};
use std::ptr::addr_of;
use std::rc::{Rc, Weak};
use std::sync::atomic::Ordering;
use super::engine::{*};

pub struct Node
{
    pub id: u64,
    parent: Weak<RefCell<Node>>,
    children: HashMap<u64, Pin<RefCell<Box<Node>>>>,
}

#[no_mangle]
pub extern "C"
fn Node_new() -> *const dyn Any {
    let node_id = engine_next_global_id();
    let node = Node {
        id: node_id,
        parent: Weak::new(),
        children: HashMap::new()
    };
    engine_pin(node_id, node)
}

#[no_mangle]
pub extern "C"
fn Node_add_child(parent: *const dyn Any, child: *const dyn Any) {
    unsafe {
        let p = engine_cast_mut(parent);
        let c = engine_cast_mut(child);
        if c.parent.upgrade().is_none() {
            let c_obj = engine_remove(child);
            //let x = Rc::downgrade(&Rc::new(Box::new(p)));
            p.children.insert(c_obj.id, c_obj);
        }
        println!();
    }
}
