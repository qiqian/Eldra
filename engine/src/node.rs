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
    id: u64,
    parent: RefCell<Weak<Node>>,
    children: HashMap<u64, RefCell<Pin<Box<Node>>>>,
}

#[no_mangle]
pub extern "C"
fn Node_new() -> u64 {
    let node_id = next_global_id();
    let node = Node {
        id: node_id,
        parent: RefCell::new(Weak::new()),
        children: HashMap::new()
    };
    let add0 = addr_of!(node);

    let pin = RefCell::new(Box::pin(node));

    let ptr = addr_of!(pin);
    pin_object(node_id, pin);

    let add1 = ptr as u64;
    println!("test");
    add1
}

#[no_mangle]
pub extern "C"
fn Node_add_child(parent: u64, child: u64) {
    unsafe {
        let p = &*(parent as *const RefCell<Pin<Box<Node>>>);
        let c = ENGINE_ROOT.get_mut().unwrap().object_registry.remove(&child).unwrap();
        p.borrow_mut().children.insert(child, c);
    }
}
