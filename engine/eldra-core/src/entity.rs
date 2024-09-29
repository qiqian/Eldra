use std::io::BufWriter;
use std::os::raw::c_char;
use std::ffi::CStr;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::{Pin};
use std::ptr::{addr_of};
use std::rc::{Rc, Weak};
use std::marker::PhantomPinned;
use std::any::type_name;
use std::fs::File;
use std::str::FromStr;
use uuid::Uuid;
use eldra_macro::{DropNotify, Reflection};
use crate::engine::{*};
use crate::reflection::{*};
use crate::comp::transform_component::TransformComponent;

#[derive(Debug,Reflection)]
pub struct BaseObject
{
    #[display="Name"]
    #[serialize]
    pub name : String,
    #[display="UUID"]
    #[serialize]
    pub instance_id : Uuid,

    pub parent: Weak<RefCell<Entity>>,

    _marker_: PhantomPinned,
}
impl Default for BaseObject {
    fn default() -> Self {
        let myid = engine_next_global_id();
        BaseObject {
            name: myid.to_string(),
            instance_id: Uuid::new_v4(),
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
        let boxed = Box::new(T::default());
        if T::is_uniq() {
            self.uniq_comp.insert(TypeId::of::<T>(), boxed);
            self.uniq_comp.get(&TypeId::of::<T>())
        }
        else {
            self.multi_comp.push(boxed);
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
                        cc.as_any().downcast_ref::<T>()
                    },
                    None => None,
                }
            },
            false => {
                for c in self.multi_comp.iter() {
                    if c.as_any().is::<T>() {
                        return c.as_any().downcast_ref::<T>()
                    }
                }
                None
            }
        }
    }
}

#[derive(Default,Reflection,DropNotify)]
#[uuid="1d9f39bc-ed1b-4868-8475-67b8d3caf88c"]
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
    pub children: Vec<Rc<RefCell<Entity>>>,
    #[display="Components"]
    #[serialize]
    components: Components,
}
impl Entity {
    // caller should decide to whether engine_pin or root_entity.add_child for this new entity
    pub fn pinned() -> Rc<RefCell<Entity>> {
        let entity = Entity::new();
        engine_pin(entity.borrow().base.instance_id, unsafe { Pin::new_unchecked(entity.clone()) });
        entity
    }
    pub fn new() -> Rc<RefCell<Entity>> {
        let entity = Rc::new(RefCell::new(Entity::default()));

        let addr = addr_of!(*entity) as u64;
        entity.borrow_mut().marker_address = addr;
        entity.borrow_mut().myself = Rc::downgrade(&entity.clone());

        entity
    }
    pub fn add_child(&mut self, c: Rc<RefCell<Entity>>) -> bool {
        let iid = c.borrow().base.instance_id;
        if !c.borrow().has_parent() {
            // c.parent <- p
            c.borrow_mut().base.parent = self.myself.clone();
            // p.children <- c
            self.children.push(c);
            true
        } else {
            println!("entity:{iid} already has parent");
            false
        }
    }
    pub fn remove_child(&mut self, c: &Rc<RefCell<Entity>>) -> bool {
        let instance_id = c.borrow().base.instance_id;
        if !c.borrow().has_parent() {
            println!("entity:{instance_id} has no parent");
            false
        }
        else {
            if (c.borrow().base.parent.as_ptr() as u64) != self.marker_address {
                let myid = self.base.instance_id;
                println!("entity:{instance_id} is not my:{myid} child");
                false
            }
            else {
                for i in 0..self.children.len() {
                    if self.children[i].borrow().base.instance_id == instance_id {
                        c.borrow_mut().base.parent = Weak::new();
                        self.children.remove(i);
                        return true
                    }
                }
                let myid = self.base.instance_id;
                println!("entity:{instance_id} not found in my:{myid} child");
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
        for c in self.components.uniq_comp.iter_mut() {
            c.1.tick(delta, parent);
        }
        for c in self.components.multi_comp.iter_mut() {
            c.tick(delta, parent);
        }
        for c in self.children.iter_mut() {
            c.borrow_mut().tick(delta, &Some(&self.components));
        }
    }
}

fn entity_destroy(e: &Rc<RefCell<Entity>>) {
    let p = e.borrow().get_parent();
    if p.is_some() {
        unsafe { p.unwrap_unchecked() }.borrow_mut().remove_child(e);
    }
    engine_remove(&e.borrow().base.instance_id);
}

//// exports

#[no_mangle]
pub extern "C"
fn Entity_new() -> u64 {
    let entity = Entity::pinned();
    let addr = entity.borrow().marker_address;
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
            p.borrow_mut().add_child(c)
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
fn Entity_serialize_binary(addr: u64, path: *const c_char) {
    entity_update(&addr, |entity| {
        let p = unsafe { CStr::from_ptr(path) }.to_str().unwrap();
        let file = File::create(p).unwrap();
        let mut writer = BufWriter::new(file);
        entity.borrow().serialize_binary(&mut writer);
    });
}
#[no_mangle]
pub extern "C"
fn Entity_serialize_yaml(addr: u64, path: *const c_char) {
    entity_update(&addr, |entity| {
        let p = unsafe { CStr::from_ptr(path) }.to_str().unwrap();
        let file = File::create(p).unwrap();
        let mut writer = BufWriter::new(file);
        entity.borrow().serialize_yaml(&mut writer, String::new());
    });
}