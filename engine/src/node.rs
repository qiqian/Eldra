use std::cell::{RefCell};
use std::collections::HashMap;
use std::pin::{Pin};
use std::ptr::addr_of;
use std::rc::{Rc, Weak};
use super::engine::{*};

pub struct Node
{
    pub id: u64,
    myself: Weak<RefCell<Node>>,
    parent: Weak<RefCell<Node>>,
    children: HashMap<u64, Pin<Box<Rc<RefCell<Node>>>>>,
}

#[no_mangle]
pub extern "C"
fn Node_new() -> u64 {
    let node = Box::pin(Rc::new(RefCell::new(Node {
        id: engine_next_global_id(),
        myself: Weak::new(),
        parent: Weak::new(),
        children: HashMap::new()
    })));
    let myself = node.clone();
    node.borrow_mut().myself = Rc::downgrade(&myself);
    let addr = addr_of!(*node) as u64;
    engine_pin(node);
    addr
}

#[no_mangle]
pub extern "C"
fn Node_add_child(parent: u64, child: u64) {
    let p = engine_cast(parent);
    let c = engine_cast(child);
    if c.borrow().parent.upgrade().is_none() {
        // p.children <- c
        let c_obj = engine_remove(child);
        p.borrow_mut().children.insert(c.borrow_mut().id, c_obj);
        // c.parent <- p
        c.borrow_mut().parent = Rc::downgrade(&p);
    }
    else {
        println!("child already has parent")
    }
}
