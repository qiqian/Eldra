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
/*
impl Drop for Node {
    fn drop(&mut self) {
        let id = self.id;
        println!("Dropping Node {id}");
    }
}
*/

impl Node {

    pub fn new() -> Pin<Box<Rc<RefCell<Node>>>> {
        let node = Box::pin(Rc::new(RefCell::new(Node {
            id: engine_next_global_id(),
            myself: Weak::new(),
            parent: Weak::new(),
            children: HashMap::new()
        })));
        let myself = node.clone();
        node.borrow_mut().myself = Rc::downgrade(&myself);
        node
    }

    pub fn add_child(&mut self, c: &Rc<RefCell<Node>>) -> bool {
        if c.borrow().parent.upgrade().is_none() {
            // p.children <- c
            let cid = c.borrow().id;
            let c_obj = engine_remove(cid);
            self.children.insert(c.borrow_mut().id, c_obj);
            // c.parent <- p
            c.borrow_mut().parent = self.myself.clone();
            true
        }
        else {
            println!("child already has parent");
            false
        }
    }

    pub fn detach_from_parent(&mut self) -> bool {
        let par = self.parent.upgrade();
        if par.is_none() {
            println!("child has no parent");
            false
        }
        else {
            let parent = par.unwrap();
            let pinned = parent.borrow_mut().children.remove(&self.id);
            self.parent = Weak::new();
            // keep in global
            engine_pin(self.id, pinned.unwrap());
            true
        }
    }

    pub fn destroy(&mut self) {
        let par = self.parent.upgrade();
        if par.is_none() {
            // remove from global
            engine_remove(self.id);
        }
        else {
            // remove from parent
            let parent = par.unwrap();
            parent.borrow_mut().children.remove(&self.id);
        }
    }

}

//// exports

#[no_mangle]
pub extern "C"
fn Node_new() -> u64 {
    let node = Node::new();
    let addr = addr_of!(*node) as u64;
    let cid = node.borrow().id;
    engine_pin(cid, node);
    addr
}

#[no_mangle]
pub extern "C"
fn Node_add_child(parent: u64, child: u64) -> bool {
    let p = engine_cast_mut(parent);
    let c = engine_cast_const(child);
    p.borrow_mut().add_child(c)
}
#[no_mangle]
pub extern "C"
fn Node_detach_from_parent(addr: u64) -> bool {
    let node = engine_cast_mut(addr);
    node.borrow_mut().detach_from_parent()
}

#[no_mangle]
pub extern "C"
fn Node_destroy(addr: u64) {
    let node = engine_cast_mut(addr);
    node.borrow_mut().destroy();
}