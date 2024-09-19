use std::any::Any;
use std::cell::{RefCell};
use std::collections::HashMap;
use std::pin::{Pin};
use std::ptr::addr_of;
use std::rc::{Rc, Weak};
use super::engine::{*};

pub struct BaseObject
{
    pub id: u64,
    pub parent: Weak<RefCell<Entity>>,
}
pub struct Entity
{
    pub base: BaseObject,
    myself: Weak<RefCell<Entity>>,
    // to contain a weak self/parent pointer, we must use Rc
    // but Rc is readonly, that leads to Rc<RefCell<_>>
    children: HashMap<u64, Pin<Box<Rc<RefCell<Entity>>>>>,
    components: HashMap<u64, Pin<Box<Rc<RefCell<dyn Any>>>>>,
}
impl EngineObject for Entity {}

impl Drop for Entity {
    fn drop(&mut self) {
        let id = self.base.id;
        println!("Dropping Entity {id}");
    }
}

fn entity_cast_const(addr : u64) -> &'static Rc<RefCell<Entity>> {
    unsafe {
        &*(addr as *const Rc<RefCell<Entity>>)
    }
}
// not used, but kept as this transform is interesting
fn entity_transform(any : Pin<Box<dyn Any>>) -> Pin<Box<Rc<RefCell<Entity>>>> {
    unsafe {
        let addr = &*(addr_of!(*any) as *const Rc<RefCell<Entity>>);
        //Box::leak(Pin::into_inner_unchecked(any));
        Box::pin(addr.clone())
    }
}

impl Entity {

    pub fn new() -> Pin<Box<Rc<RefCell<Entity>>>> {
        let entity = Box::pin(Rc::new(RefCell::new(Entity {
            base : BaseObject {
                id: engine_next_global_id(),
                parent: Weak::new(),
            },
            myself: Weak::new(),
            children: HashMap::new(),
            components: HashMap::new(),
        })));
        let myself = entity.clone();
        entity.borrow_mut().myself = Rc::downgrade(&myself);
        entity
    }

    pub fn add_child(&mut self, c: &Rc<RefCell<Entity>>) -> bool {
        if c.borrow().base.parent.upgrade().is_none() {
            // p.children <- c
            let cid = c.borrow().base.id;
            // need to do this before remove, otherwise c get destroyed
            let new_pin = Box::pin(c.clone());
            engine_remove(cid);
            self.children.insert(cid, new_pin);
            // c.parent <- p
            c.borrow_mut().base.parent = self.myself.clone();
            true
        } else {
            println!("child already has parent");
            false
        }
    }
    pub fn remove_child(&mut self, c: &Rc<RefCell<Entity>>) -> bool {
        let mut c_mut = c.borrow_mut();
        let c_p = c_mut.base.parent.upgrade();
        if c_p.is_none() {// || c_p.unwrap().borrow().base.id != self.base.id {
            println!("child is not my child");
            false
        }
        else {
            let pinned = self.children.remove(&c_mut.base.id);
            c_mut.base.parent = Weak::new();
            // keep in global
            engine_pin(c_mut.base.id, pinned.unwrap());
            true
        }
    }

    /*
    pub fn detach_from_parent(&mut self) -> bool {
        let par = self.base.parent.upgrade();
        if par.is_none() {
            println!("child has no parent");
            false
        }
        else {
            let parent = par.unwrap();
            let pinned = parent.borrow_mut().children.remove(&self.base.id);
            self.base.parent = Weak::new();
            // keep in global
            engine_pin(self.base.id, pinned.unwrap());
            true
        }
    }*/

    pub fn destroy(&mut self) {
        let par = self.base.parent.upgrade();
        if par.is_none() {
            // remove from global
            engine_remove(self.base.id);
        }
        else {
            // remove from parent
            let parent = par.unwrap();
            parent.borrow_mut().children.remove(&self.base.id);
        }
    }

}

//// exports

#[no_mangle]
pub extern "C"
fn Entity_new() -> u64 {
    let entity = Entity::new();
    let addr = addr_of!(*entity) as u64;
    let cid = entity.borrow().base.id;
    engine_pin(cid, entity);
    addr
}

#[no_mangle]
pub extern "C"
fn Entity_add_child(parent: u64, child: u64) -> bool {
    let p = entity_cast_const(parent);
    let c = entity_cast_const(child);
    p.borrow_mut().add_child(c)
}
#[no_mangle]
pub extern "C"
fn Entity_remove_child(parent: u64, child: u64) -> bool {
    let p = entity_cast_const(parent);
    let c = entity_cast_const(child);
    p.borrow_mut().remove_child(c)
}
#[no_mangle]
pub extern "C"
fn Entity_get_parent(addr: u64) -> u64 {
    let entity = entity_cast_const(addr);
    let p = entity.borrow().base.parent.upgrade();
    if p.is_none() {
        0
    } else {
        let inner = p.unwrap();
        addr_of!(inner) as u64
    }
}
/*
#[no_mangle]
pub extern "C"
fn Entity_detach_from_parent(addr: u64) -> bool {
    let entity = entity_cast_const(addr);
    entity.borrow_mut().detach_from_parent()
}
*/

#[no_mangle]
pub extern "C"
fn Entity_destroy(addr: u64) {
    let entity = entity_cast_const(addr);
    entity.borrow_mut().destroy();
}