use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::{Pin};
use std::ptr::{addr_of};
use std::rc::{Rc, Weak};
use std::marker::PhantomPinned;
use std::any::type_name;
use std::ffi::CString;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::{Deref, DerefMut};
use uuid::Uuid;
use eldra_macro::{DropNotify, Reflection};
use crate::engine::{*};
use crate::reflection::{*};
use crate::comp::transform_component::TransformComponent;

#[derive(Debug,Reflection)]
pub struct BaseObject
{
    #[serialize]
    pub id: u64,
    #[display="Name"]
    #[serialize]
    pub name : String,
    #[display="UUID"]
    #[serialize]
    pub uid : Uuid,

    pub parent: Weak<RefCell<Entity>>,

    _marker_: PhantomPinned,
}
impl Default for BaseObject {
    fn default() -> Self {
        let myid = engine_next_global_id();
        BaseObject {
            id: myid,
            name: myid.to_string(),
            uid: Uuid::new_v4(),
            parent: Weak::new(),
            _marker_: PhantomPinned,
        }
    }
}

pub trait ComponentAttr {
    fn is_comp_uniq(&self) -> bool;
}
pub trait Component : Reflectable + ComponentAttr + Serializable {
    fn tick(&mut self, delta: f32, ancestor: &Option<&Components>);
}
#[derive(Default,Reflection)]
pub struct Components
{
    #[serialize]
    uniq_comp: HashMap<TypeId, Box<dyn Component>>,
    // component pointer is leaked into entity to work around trait conversion issue
    // this is safe because they have the same lifecycle, just do cleanup when removing the component
    #[serialize]
    multi_comp: Vec<Box<dyn Component>>,
}
impl Components {
    pub fn create_component<T>(&mut self) -> Option<&Box<dyn Component>>
        where T: Component + Uniq + Default + 'static
    {
        if T::is_uniq() && self.uniq_comp.contains_key(&TypeId::of::<T>()) {
            eprintln!("can't duplicate uniq component");
            return None
        }
        let pinned = Box::new(T::default());
        if T::is_uniq() {
            self.uniq_comp.insert(TypeId::of::<T>(), pinned);
            self.uniq_comp.get(&TypeId::of::<T>())
        }
        else {
            self.multi_comp.push(pinned);
            Some(&self.multi_comp[self.multi_comp.len() - 1])
        }
    }
    pub fn remove_component(&mut self, candidate: &Box<dyn Component>) -> bool {
        if candidate.is_comp_uniq() {
            if self.uniq_comp.remove(&candidate.real_type_id()).is_some() {
                return true
            }
            false
        }
        else {
            let idx = 0;
            while idx < self.multi_comp.len() {
                let cc = &self.multi_comp[idx];
                if addr_of!(cc) == addr_of!(candidate) {
                    self.multi_comp.remove(idx);
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
}

#[derive(Default,Reflection,DropNotify)]
pub struct Entity
{
    #[serialize]
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
    pub fn tick(&mut self, delta: f32, parent: &Option<&Components>) {
        self.components.uniq_comp.iter_mut().map(|c| {
            c.1.tick(delta, parent);
        });
        for c in self.components.multi_comp.iter_mut() {
            c.tick(delta, parent);
        }
        for c in self.children.iter_mut() {
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

pub fn entity_cast(addr : &u64) -> Option<Rc<RefCell<Entity>>> {
    unsafe {
        (&mut *((*addr) as *mut RefCell<Entity>)).borrow().myself.upgrade()
    }
}
fn entity_update<T, F: Fn(Rc<RefCell<Entity>>) -> T>(me: &u64, f: F) -> T where T: Default
{
    let entity = entity_cast(me);
    match entity {
        Some(t) => f(t),
        None => T::default(),
    }
}
#[no_mangle]
pub extern "C"
fn Entity_add_child(parent: u64, child: u64) -> bool {
    entity_update(&parent, |p| {
        entity_update(&child, |c| {
            let cid = c.borrow().base.id;
            if p.borrow_mut().add_child(unsafe { Pin::new_unchecked(c) }) {
                let _ = engine_remove(cid);
                true
            }
            else {
                false
            }
        })
    })
}
#[no_mangle]
pub extern "C"
fn Entity_remove_child(parent: u64, child: u64) -> bool {
    entity_update(&parent, |p| {
        entity_update(&child, |c| {
            let mut p_ = p.borrow_mut();
            p_.remove_child(&c)
        })
    })
}
#[no_mangle]
pub extern "C"
fn Entity_get_parent(addr: u64) -> u64 {
    entity_update(&addr, |entity| {
        let e = entity.borrow();
        match e.get_parent() {
            Some(p) => {
                p.borrow().marker_address
            },
            None => 0
        }
    })
}

#[no_mangle]
pub extern "C"
fn Entity_destroy(addr: u64) {
    entity_update(&addr, |entity| {
        entity_destroy(&entity);
    });
}
#[no_mangle]
pub extern "C"
fn Entity_create_transform_component(addr: u64) -> u64 {
    entity_update(&addr, |entity| {
        let mut e = entity.borrow_mut();
        let opt = e.components.create_component::<TransformComponent>();
        unsafe { *(addr_of!(opt) as *const u64) }
    })
}

#[macro_export]
macro_rules! decode_component {
    ( $x:expr ) => {
        {
            unsafe { (&mut*(addr_of!($x) as *mut u64 as *mut Option<&mut Box<dyn Component>>)) }
        }
    };
}
#[no_mangle]
pub extern "C"
fn Entity_remove_component(e: u64, c: u64) -> bool {
    let tc = decode_component!(c);
    if tc.is_none() {
        return false
    }
    entity_update(&e, |entity| {
        let mut e = entity.borrow_mut();
        match decode_component!(c) {
            Some(comp) => {
                e.components.remove_component(comp);
                true
            },
            None => false
        }
    })
}

#[no_mangle]
pub extern "C"
fn Entity_tick(addr: u64, delta: f32) {
    entity_update(&addr, |entity| {
        if entity.borrow().has_parent() {
            eprintln!("can't tick non-root entity");
            return
        }
        let mut b = entity.borrow_mut();
        b.tick(delta, &None);
    })
}

#[no_mangle]
pub extern "C"
fn Entity_serialize(addr: u64, path: CString) {
    entity_update(&addr, |entity| {
        let p = path.to_str().unwrap().to_string();
        let mut file = File::create(p).unwrap();;
        entity.borrow().serialize_yaml(&mut file, String::new());
    });
}