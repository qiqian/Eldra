use std::any::Any;
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
impl EngineObject for Node {}

impl Drop for Node {
    fn drop(&mut self) {
        let id = self.id;
        println!("Dropping Node {id}");
    }
}

fn node_cast_const(addr : u64) -> &'static Rc<RefCell<Node>> {
    unsafe {
        &*(addr as *const Rc<RefCell<Node>>)
    }
}
// not used, but kept as this transform is interesting
fn node_transform(any : Pin<Box<dyn Any>>) -> Pin<Box<Rc<RefCell<Node>>>> {
    unsafe {
        let addr = &*(addr_of!(*any) as *const Rc<RefCell<Node>>);
        //Box::leak(Pin::into_inner_unchecked(any));
        Box::pin(addr.clone())
    }
}

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
            // need to do this before remove, otherwise c get destroyed
            let new_pin = Box::pin(c.clone());
            engine_remove(cid);
            self.children.insert(cid, new_pin);
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
    let p = node_cast_const(parent);
    let c = node_cast_const(child);
    p.borrow_mut().add_child(c)
}
#[no_mangle]
pub extern "C"
fn Node_detach_from_parent(addr: u64) -> bool {
    let node = node_cast_const(addr);
    node.borrow_mut().detach_from_parent()
}

#[no_mangle]
pub extern "C"
fn Node_destroy(addr: u64) {
    let node = node_cast_const(addr);
    node.borrow_mut().destroy();
}