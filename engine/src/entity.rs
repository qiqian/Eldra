use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::{Pin};
use std::ptr::{addr_of, from_ref};
use std::rc::{Rc, Weak};
use std::marker::PhantomPinned;
use std::any::type_name;
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
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn tick(&mut self, delta: f32, ancestor: *const Entity);
}
#[derive(Default)]
pub struct Entity
{
    pub base: BaseObject,
    myself: Weak<RefCell<Entity>>,
    // Entity is shared in engine, we use Rc<RefCell>
    // the 1st Rc<RefCell> is used as address marker for scripting
    // so the 1st Rc<RefCell> and its address must be kept valid
    marker_address: u64,
    // to contain a weak self pointer, we must use Rc
    // but Rc is readonly, that leads to Rc<RefCell<_>>
    children: HashMap<u64, Pin<Rc<RefCell<Entity>>>>,
    // component pointer is leaked into entity to work around trait conversion issue
    // this is safe because they have the same lifecycle, just do cleanup when removing the component
    components: Vec<*mut dyn Component>,
}

impl Drop for Entity {
    fn drop(&mut self) {
        engine_notify_drop_object(type_name::<Entity>(), self.base.id);
        for c in self.components.iter() {
            unsafe {
                // cleanup
                let leaked = *c;
                let _ = Box::from_raw(leaked);
            };
        }
    }
}

fn entity_cast(addr : &u64) -> Rc<RefCell<Entity>> {
    unsafe {
        (&mut *((*addr) as *mut RefCell<Entity>)).
            borrow().myself.upgrade().unwrap_unchecked().clone()
    }
}

impl Entity {
    // caller should decide to whether engine_pin or root_entity.add_child for this new entity
    pub fn new() -> Pin<Rc<RefCell<Entity>>> {
        let entity = Rc::new(RefCell::new(Entity::default()));

        let addr = addr_of!(*entity) as u64;
        entity.borrow_mut().marker_address = addr;
        entity.borrow_mut().myself = Rc::downgrade(&entity.clone());

        unsafe { Pin::new_unchecked(entity) }
    }

    // add_child must take ownership of child, so DO NOT use reference
    pub fn add_child(&mut self, c: Pin<Rc<RefCell<Entity>>>) -> bool {
        let cid = c.borrow().base.id;
        if !c.borrow().has_parent() {
            // c.parent <- p
            c.borrow_mut().base.parent = self.myself.clone();
            // p.children <- c
            self.children.insert(cid, c);
            true
        } else {
            println!("entity:{cid} already has parent");
            false
        }
    }
    pub fn remove_child(&mut self, c: &Rc<RefCell<Entity>>) -> bool {
        let cid = c.borrow().base.id;
        let pinned = self.children.remove(&cid);
        match pinned {
            Some(p) => { // removed
                c.borrow_mut().base.parent = Weak::new();
                // keep in global
                engine_pin(cid, p);
                true
            },
            None => { // not found
                let myid = self.base.id;
                println!("entity:{cid} is not my:{myid} child");
                false
            }
        }
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
        let idx = 0;
        while idx < self.components.len() {
            let cc : *mut dyn Component = self.components[idx];
            if cc == candidate {
                self.components.remove(idx);
                // cleanup
                unsafe { let _ = Box::from_raw(candidate); }
                return true
            }
        }
        false
    }
    pub fn has_parent(&self) -> bool {
        self.get_parent().is_some()
    }
    pub fn get_parent(&self) -> Option<Rc<RefCell<Entity>>> {
        self.base.parent.upgrade()
    }
    pub fn get_component<T: Component + 'static>(& self) -> Option<&T> where {
        for c in self.components.iter() {
            let cc = unsafe { &**c };
            if cc.as_any().is::<T>() {
                return cc.as_any().downcast_ref::<T>()
            }
        }
        None
    }
    pub fn get_component_mut<T: Component + 'static>(& mut self) -> Option<&mut T> {
        for c in self.components.iter_mut() {
            let cc = unsafe { &mut **c };
            if cc.as_any().is::<T>() {
                return cc.as_any_mut().downcast_mut::<T>()
            }
        }
        None
    }

    pub fn tick(&mut self, delta: f32, parent: *const Entity) {
        for c in self.components.iter() {
            let cc = unsafe { &mut **c };
            cc.tick(delta, parent);
        }
        let me: *const Entity = from_ref(self);
        for c in self.children.iter() {
            c.1.borrow_mut().tick(delta, me);
        }
    }
}

fn entity_destroy(e: &Rc<RefCell<Entity>>) {
    let myid = e.borrow().base.id;
    match e.borrow().get_parent() {
        Some(p) => {
            // remove from parent
            p.borrow_mut().children.remove(&myid);
        },
        None => {
            // remove from global
            engine_remove(myid);
        },
    }
}

//// exports

#[no_mangle]
pub extern "C"
fn Entity_new() -> u64 {
    let entity = Entity::new();
    let id = entity.borrow().base.id;
    let addr = entity.borrow().marker_address;
    engine_pin(id, entity);
    addr
}

#[no_mangle]
pub extern "C"
fn Entity_add_child(parent: u64, child: u64) -> bool {
    let p = entity_cast(&parent);
    let c = entity_cast(&child);
    let cid = c.borrow().base.id;
    if p.borrow_mut().add_child(unsafe { Pin::new_unchecked(c) }) {
        let _ = engine_remove(cid);
        return true
    }
    false
}
#[no_mangle]
pub extern "C"
fn Entity_remove_child(parent: u64, child: u64) -> bool {
    let p = entity_cast(&parent);
    let c = entity_cast(&child);
    let mut p_ = p.borrow_mut();
    p_.remove_child(&c)
}
#[no_mangle]
pub extern "C"
fn Entity_get_parent(addr: u64) -> u64 {
    let entity = entity_cast(&addr);
    let e = entity.borrow();
    match e.get_parent() {
        Some(p) => {
            p.borrow().marker_address
        },
        None => { 0 }
    }
}

#[no_mangle]
pub extern "C"
fn Entity_destroy(addr: u64) {
    let entity = entity_cast(&addr);
    entity_destroy(&entity);
}

#[no_mangle]
pub extern "C"
fn Entity_create_transform_component(addr: u64) -> u64 {
    let c = TransformComponent::new();
    let entity = entity_cast(&addr);
    let mut e = entity.borrow_mut();
    e.add_component(c) as u64
}

#[no_mangle]
pub extern "C"
fn Entity_tick(addr: u64, delta: f32) {
    let entity = entity_cast(&addr);
    if entity.borrow().has_parent() {
        eprintln!("can't tick non-root entity");
        return
    }
    entity.borrow_mut().tick(delta, 0 as *const Entity);
}