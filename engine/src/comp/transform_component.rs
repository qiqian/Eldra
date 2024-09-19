use std::cell::RefCell;
use std::rc::Weak;
use std::pin::{Pin};
use std::ops::{Deref, DerefMut};
use nalgebra::{*};
use crate::comp::uarg::UArg;
use crate::entity;
use crate::entity::{*};

pub struct TransformComponent
{
    pub base: BaseObject,
    pub local_matrix: Matrix4<f32>,
}

impl Default for TransformComponent {
    fn default() -> Self {
        TransformComponent {
            base : BaseObject::default(),
            local_matrix : Matrix4::identity(),
        }
    }
}
impl TransformComponent {
    pub fn new() -> Pin<Box<RefCell<TransformComponent>>> {
        let t = TransformComponent{..Default::default()};
        Box::pin(RefCell::new(t))
    }
    pub fn translate(&mut self, v: &Vector3<f32>) {
        self.local_matrix.append_translation(v);
    }

    pub fn rotate(&mut self, angles: &Vector3<f32>) {
        let r = Matrix4::from_scaled_axis(*angles);
        self.local_matrix = r * self.local_matrix;
    }

    pub fn scale(&mut self, scale: &Vector3<f32>) {
        self.local_matrix.append_nonuniform_scaling_mut(scale);
    }
}
impl Component for TransformComponent {
    fn tick(&mut self, delta: f32, parent: &Option<&&mut Entity>) {
        if parent.is_none() {
            return;
        }

    }
}

fn transform_component_cast(addr : u64) -> &'static RefCell<TransformComponent> {
    unsafe {
        &*(addr as *const RefCell<TransformComponent>)
    }
}

//// exports


#[no_mangle]
pub extern "C"
fn TransformComponent_translate(addr: u64, x : f32, y : f32, z : f32) {
    let t = transform_component_cast(addr);
    t.borrow_mut().translate(&Vector3::new(x, y, z));
}

#[no_mangle]
pub extern "C"
fn TransformComponent_rotate(addr: u64, x : f32, y : f32, z : f32) {
    let t = transform_component_cast(addr);
    t.borrow_mut().rotate(&Vector3::new(x, y, z));
}

#[no_mangle]
pub extern "C"
fn TransformComponent_scale(addr: u64, x : f32, y : f32, z : f32) {
    let t = transform_component_cast(addr);
    t.borrow_mut().scale(&Vector3::new(x, y, z));
}