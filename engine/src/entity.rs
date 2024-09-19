use std::any::Any;
use std::cell::{RefCell};
use std::collections::HashMap;
use std::pin::{Pin};
use std::ptr::addr_of;
use std::rc::{Rc, Weak};
use std::marker::PhantomPinned;
use std::ops::DerefMut;
use std::any::TypeId;
use crate::engine::{*};
use crate::comp::transform_component::TransformComponent;

pub struct BaseObject
{
    pub id: u64,
    pub parent: Weak<RefCell<Entity>>,
    _marker_: PhantomPinned,
}
impl Default for BaseObject {
    fn default() -> Self {
        BaseObject {
            id: engine_next_global_id(),
            parent: Weak::new(),
            _marker_: PhantomPinned,
        }
    }
}

pub trait Component {
    fn as_any(&mut self) -> &mut dyn Any;
    fn tick(&mut self, delta: f32, parent: &Option<Rc<RefCell<Entity>>>);
}
#[derive(Default)]
pub struct Entity
{
    pub base: BaseObject,
    myself: Weak<RefCell<Entity>>,
    // to contain a weak self pointer, we must use Rc
    // but Rc is readonly, that leads to Rc<RefCell<_>>
    children: HashMap<u64, Pin<Box<Rc<RefCell<Entity>>>>>,
    // component pointer is leaked into entity to work around trait conversion issue
    // this is safe because they have the same lifecycle, just do cleanup when removing the component
    components: Vec<*mut dyn Component>,
}

impl Drop for Entity {
    fn drop(&mut self) {
        for mut c in self.components.iter_mut() {
            unsafe {
                // cleanup
                let leaked : *mut dyn Component = *c;
                let _ = Box::from_raw(leaked);
            };
        }
        let id = self.base.id;
        println!("Dropping Entity {id}");
    }
}

fn entity_add_component(addr: u64, c: Rc<RefCell<dyn Component>>) {
}
fn entity_cast_const<'a>(addr : &'a u64) -> &'a Rc<RefCell<Entity>> {
    unsafe {
        &*(*addr as *const Rc<RefCell<Entity>>)
    }
}

impl Entity {
    pub fn new() -> Rc<RefCell<Entity>> {
        let entity = Rc::new(RefCell::new(Entity::default()));
        let myself = entity.clone();
        entity.borrow_mut().myself = Rc::downgrade(&myself);
        entity
    }

    pub fn add_child(&mut self, c: Rc<RefCell<Entity>>) -> bool {
        if c.borrow().base.parent.upgrade().is_none() {
            // p.children <- c
            let cid = c.borrow().base.id;
            // c.parent <- p
            c.borrow_mut().base.parent = self.myself.clone();
            // need to do this before remove, otherwise c get destroyed
            let new_pin = Box::pin(c);
            engine_remove(cid);
            self.children.insert(cid, new_pin);
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

    pub fn add_component<T: Component + 'static>(&mut self, c: T) -> *mut T {
        unsafe {
            //leak it
            let pinned = Box::into_raw(Pin::into_inner_unchecked(Box::pin(c)));
            self.components.push(pinned);
            pinned
        }
    }
    pub fn remove_component(&mut self, candidate: *mut dyn Component) -> bool {
        for idx in 0..self.components.len() {
            let mut cc : *mut dyn Component = self.components[idx];
            if cc == candidate {
                self.components.remove(idx);
                unsafe { let _ = Box::from_raw(cc); }
                return true
            }
        }
        false
    }
    pub fn get_component<T: Component + 'static>(&'static mut self)
        -> Option<&mut T> where {
        for mut c in self.components.iter_mut() {
            let mut cc = unsafe { &mut **c };
            if cc.type_id() == TypeId::of::<T>() {
                return cc.as_any().downcast_mut::<T>()
            }
        }
        None
    }

    pub fn tick(&mut self, delta: f32, parent: &Option<Rc<RefCell<Entity>>>) {
        for c in self.components.iter_mut() {
            let mut cc = unsafe { &mut **c };
            cc.tick(delta, parent);
        }
        let opt = self.myself.upgrade();
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
    let cid = entity.borrow().base.id;
    let pinned = Box::pin(entity);
    // take addr after pinned
    let addr = addr_of!(*pinned) as u64;
    engine_pin(cid, pinned);
    addr
}

#[no_mangle]
pub extern "C"
fn Entity_add_child(parent: u64, child: u64) -> bool {
    let p = entity_cast_const(&parent);
    let c = entity_cast_const(&child);
    p.borrow_mut().add_child(c.clone())
}
#[no_mangle]
pub extern "C"
fn Entity_remove_child(parent: u64, child: u64) -> bool {
    let p = entity_cast_const(&parent);
    let c = entity_cast_const(&child);
    p.borrow_mut().remove_child(c)
}
#[no_mangle]
pub extern "C"
fn Entity_get_parent(addr: u64) -> u64 {
    let entity = entity_cast_const(&addr);
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
    let entity = entity_cast_const(&addr);
    entity_destroy(entity.borrow_mut().deref_mut());
}

#[no_mangle]
pub extern "C"
fn Entity_create_transform_component(addr: u64) -> u64 {
    let c = TransformComponent::new();
    let entity = entity_cast_const(&addr);
    entity.borrow_mut().add_component(c) as u64
}

#[no_mangle]
pub extern "C"
fn Entity_tick(addr: u64, delta: f32, parent_addr: u64) {
    let entity = entity_cast_const(&addr);
    let mut parent = entity_cast_const(&parent_addr);
    let opt = if parent_addr != 0 { Some(parent.clone()) } else { None } ;
    entity.borrow_mut().tick(delta, &opt);
}