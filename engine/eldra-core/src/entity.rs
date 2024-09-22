use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::{Pin};
use std::ptr::{addr_of};
use std::rc::{Rc, Weak};
use std::marker::PhantomPinned;
use std::any::type_name;
use std::ops::{DerefMut};
use eldra_macro::{DropNotify, Reflection};
use crate::engine::{*};
use crate::reflection::{*};
use crate::comp::transform_component::TransformComponent;

#[derive(Debug)]
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

pub trait ComponentAttr {
    fn is_comp_uniq(&self) -> bool;
}
pub trait Component : Reflectable + ComponentAttr {
    fn tick(&mut self, delta: f32, ancestor: &Option<&Components>);
}
pub trait Uniq {
    fn is_uniq() -> bool;
}
#[derive(Debug,Default,Reflection)]
pub struct Components
{
    #[serialize]
    uniq_comp: HashMap<TypeId, *mut dyn Component>,
    // component pointer is leaked into entity to work around trait conversion issue
    // this is safe because they have the same lifecycle, just do cleanup when removing the component
    #[serialize]
    multi_comp: Vec<*mut dyn Component>,
}
impl Drop for Components {
    fn drop(&mut self) {
        for c in self.uniq_comp.values_mut() {
            unsafe {
                // cleanup
                let leaked = *c;
                let _ = Box::from_raw(leaked);
            };
        }
        for c in self.multi_comp.iter() {
            unsafe {
                // cleanup
                let leaked = *c;
                let _ = Box::from_raw(leaked);
            };
        }
    }
}
impl Components {
    pub fn create_component<T: Component + Uniq + Default + 'static>(&mut self) -> Option<*mut T> {
        if T::is_uniq() && self.uniq_comp.contains_key(&TypeId::of::<T>()) {
            eprintln!("can't duplicate uniq component");
            return None
        }
        let pinned = unsafe { //leak it
            Box::into_raw(Pin::into_inner_unchecked(Box::pin(T::default()))) };
        if T::is_uniq() {
            self.uniq_comp.insert(TypeId::of::<T>(), pinned);
        }
        else {
            self.multi_comp.push(pinned);
        }
        Some(pinned)
    }
    pub fn remove_component(&mut self, candidate: *mut dyn Component) -> bool {
        let c = unsafe{ &mut *candidate };
        if c.is_comp_uniq() {
            if self.uniq_comp.remove(&c.real_type_id()).is_some() {
                unsafe { let _ = Box::from_raw(candidate); }
                return true
            }
            false
        }
        else {
            let idx = 0;
            while idx < self.multi_comp.len() {
                let cc: *mut dyn Component = self.multi_comp[idx];
                if cc == candidate {
                    self.multi_comp.remove(idx);
                    // cleanup
                    unsafe { let _ = Box::from_raw(candidate); }
                    return true
                }
            }
            false
        }
    }
    pub fn get_component<T: Component + Uniq + 'static>(& self) -> Option<&T> where {
        match T::is_uniq() {
            true => {
                match self.uniq_comp.get(&TypeId::of::<T>()) {
                    Some(cc) => {
                        let c = unsafe { &**cc };
                        c.as_any().downcast_ref::<T>()
                    },
                    None => None,
                }
            },
            false => {
                for c in self.multi_comp.iter() {
                    let cc = unsafe { &**c };
                    if cc.as_any().is::<T>() {
                        return cc.as_any().downcast_ref::<T>()
                    }
                }
                None
            }
        }
    }
    pub fn get_component_mut<T: Component + Uniq + 'static>(& mut self) -> Option<&mut T> {
        match T::is_uniq() {
            true => {
                match self.uniq_comp.get_mut(&TypeId::of::<T>()) {
                    Some(cc) => {
                        let c = unsafe { &mut **cc };
                        c.as_any_mut().downcast_mut::<T>()
                    },
                    None => None,
                }
            }
            false => {
                for c in self.multi_comp.iter_mut() {
                    let cc = unsafe { &mut **c };
                    if cc.as_any().is::<T>() {
                        return cc.as_any_mut().downcast_mut::<T>()
                    }
                }
                None
            }
        }
    }
}

#[derive(Debug,Default,Reflection,DropNotify)]
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
    #[display="Children"]
    #[serialize]
    children: HashMap<u64, Pin<Rc<RefCell<Entity>>>>,
    #[display="Components"]
    #[serialize]
    components: Components,
}

pub fn entity_cast(addr : &u64) -> Rc<RefCell<Entity>> {
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

    pub fn has_parent(&self) -> bool {
        self.get_parent().is_some()
    }
    pub fn get_parent(&self) -> Option<Rc<RefCell<Entity>>> {
        self.base.parent.upgrade()
    }
    pub fn get_component<T: Component + Uniq + 'static>(& self) -> Option<&T> where {
        self.components.get_component::<T>()
    }
    pub fn get_component_mut<T: Component + Uniq + 'static>(& mut self) -> Option<&mut T> {
        self.components.get_component_mut::<T>()
    }
    pub fn tick(&mut self, delta: f32, parent: &Option<&Components>) {
        for c in self.components.uniq_comp.iter() {
            let cc = unsafe { &mut **c.1 };
            cc.tick(delta, parent);
        }
        for c in self.components.multi_comp.iter() {
            let cc = unsafe { &mut **c };
            cc.tick(delta, parent);
        }
        for c in self.children.iter() {
            c.1.borrow_mut().tick(delta, &Some(&self.components));
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

fn entity_component_to_trait<T>(opt: Option<*mut T>) -> Option<*mut dyn Component>
    where T: Component + Uniq + Default + 'static
{
    match opt {
        Some(ret) => { Some(ret as *mut dyn Component) },
        None => None,
    }
}
#[no_mangle]
pub extern "C"
fn Entity_create_transform_component(addr: u64) -> Option<*mut dyn Component> {
    let entity = entity_cast(&addr);
    let mut e = entity.borrow_mut();
    let c = e.components.create_component::<TransformComponent>();
    entity_component_to_trait(c)
}
#[no_mangle]
pub extern "C"
fn Entity_remove_component(e: u64, tc: Option<*mut dyn Component>) -> bool {
    if tc.is_none() {
        return false
    }
    let entity = entity_cast(&e);
    let mut e = entity.borrow_mut();
    unsafe {
        e.components.remove_component(tc.unwrap_unchecked())
    }
}

#[no_mangle]
pub extern "C"
fn Entity_tick(addr: u64, delta: f32) {
    let entity = entity_cast(&addr);
    if entity.borrow().has_parent() {
        eprintln!("can't tick non-root entity");
        return
    }
    let mut b = entity.borrow_mut();
    let e = b.deref_mut();
    e.tick(delta, &None);
}