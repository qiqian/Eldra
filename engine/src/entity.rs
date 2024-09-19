use std::any::Any;
use std::cell::{RefCell};
use std::collections::HashMap;
use std::pin::{Pin};
use std::ptr::addr_of;
use std::rc::{Rc, Weak};
use std::ops::Deref;
use std::ops::DerefMut;
use std::any::TypeId;
use crate::engine::{*};
use crate::comp::transform_component::TransformComponent;

pub struct BaseObject
{
    pub id: u64,
    pub parent: Weak<RefCell<Entity>>,
}
impl Default for BaseObject {
    fn default() -> Self {
        BaseObject {
            id: engine_next_global_id(),
            parent: Weak::new(),
        }
    }
}
pub struct Entity
{
    pub base: BaseObject,
    myself: Weak<RefCell<Entity>>,
    // to contain a weak self pointer, we must use Rc
    // but Rc is readonly, that leads to Rc<RefCell<_>>
    children: HashMap<u64, Pin<Box<Rc<RefCell<Entity>>>>>,
    components: Vec<Pin<Box<RefCell<dyn Component>>>>,
}
impl EngineObject for Entity {}

pub trait Component {
    fn tick(&mut self, delta: f32, parent: &Option<&&mut Entity>);
}

impl Drop for Entity {
    fn drop(&mut self) {
        let id = self.base.id;
        println!("Dropping Entity {id}");
    }
}

fn Entity_create_component(addr: u64, c: Pin<Box<RefCell<dyn Component>>>) {
    let entity = entity_cast_const(addr);
    entity.borrow_mut().add_component(c);
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

impl Default for Entity {
    fn default() -> Self {
        Entity {
            base : BaseObject::default(),
            myself: Weak::new(),
            children: HashMap::new(),
            components: Vec::new(),
        }
    }
}
impl Entity {
    pub fn new() -> Pin<Box<Rc<RefCell<Entity>>>> {
        let entity = Box::pin(Rc::new(RefCell::new(Entity::default())));
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
            let cid = c.borrow().base.id;
            println!("entity:{cid} already has parent");
            false
        }
    }
    pub fn remove_child(&mut self, c: &Rc<RefCell<Entity>>) -> bool {
        let pinned = self.children.remove(&c.borrow().base.id);
        if !pinned.is_none() {
            c.borrow_mut().base.parent = Weak::new();
            // keep in global
            engine_pin(c.borrow().base.id, pinned.unwrap());
            return true;
        }
        let myid = self.base.id;
        let cid = c.borrow().base.id;
        println!("entity:{cid} is not my:{myid} child");
        false
    }

    pub fn add_component(&mut self, c: Pin<Box<RefCell<dyn Component>>>) {
        self.components.push(c);
    }
    pub fn get_component<T: Component + 'static>(&'static mut self)
        -> Option<RefCell<T>> {
        for c in self.components.iter() {
            if c.borrow().type_id() == TypeId::of::<T>() {
                return Some(**c)
            }
        }
        return None
    }

    pub fn tick(&mut self, delta: f32, parent: &Option<&&mut Entity>) {
        for c in self.components.iter_mut() {
            c.borrow_mut().tick(delta, parent);
        }
        let opt = Some(&self);
        for c in self.children.iter() {
            c.1.borrow_mut().tick(delta, &opt);
        }
    }
}

pub fn entity_destroy(e: &mut Entity) {
    let par = e.base.parent.upgrade();
    if par.is_none() {
        // remove from global
        engine_remove(e.base.id);
    }
    else {
        // remove from parent
        let parent = par.unwrap();
        parent.borrow_mut().children.remove(&e.base.id);
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

#[no_mangle]
pub extern "C"
fn Entity_destroy(addr: u64) {
    let entity = entity_cast_const(addr);
    entity_destroy(entity.borrow_mut().deref_mut());
}

#[no_mangle]
pub extern "C"
fn Entity_create_transform_component(addr: u64) -> u64 {
    let c = TransformComponent::new();
    let c_addr = addr_of!(*c);
    Entity_create_component(addr, c);
    c_addr as u64
}

#[no_mangle]
pub extern "C"
fn Entity_tick(addr: u64, delta: f32, parent_addr: u64) {
    let entity = entity_cast_const(addr);
    let mut parent = entity_cast_const(parent_addr).borrow_mut();
    let mut parent_ref = parent.deref_mut();
    let opt = if parent_addr != 0 { Some(&(parent_ref)) } else { None } ;
    entity.borrow_mut().tick(delta, &opt);
}