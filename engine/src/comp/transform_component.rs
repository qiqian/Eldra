use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::pin::{Pin};
use std::ops::{Deref, DerefMut};
use nalgebra::{*};
use crate::comp::uarg::UArg;
use crate::entity;
use crate::entity::{*};

#[derive(Default)]
pub struct TransformComponent
{
    pub base: BaseObject,
    pub local_matrix: Matrix4<f32>,
}

impl Drop for TransformComponent {
    fn drop(&mut self) {
        let id = self.base.id;
        println!("Dropping TransformComponent {id}");
    }
}
impl TransformComponent {
    pub fn new() -> Self {
        TransformComponent::default()
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
    fn as_any(&mut self) -> &mut dyn Any { self }
    fn tick(&mut self, delta: f32, parent: &Option<Rc<RefCell<Entity>>>) {
        if parent.is_none() {
            return;
        }

    }
}

fn transform_component_cast<'a>(addr : &'a u64) -> &'a mut Box<TransformComponent> {
    unsafe {
        &mut*(*addr as *mut Box<TransformComponent>)
    }
}

//// exports


#[no_mangle]
pub extern "C"
fn TransformComponent_translate(addr: u64, x : f32, y : f32, z : f32) {
    let t = transform_component_cast(&addr);
    t.translate(&Vector3::new(x, y, z));
}

#[no_mangle]
pub extern "C"
fn TransformComponent_rotate(addr: u64, x : f32, y : f32, z : f32) {
    let t = transform_component_cast(&addr);
    t.rotate(&Vector3::new(x, y, z));
}

#[no_mangle]
pub extern "C"
fn TransformComponent_scale(addr: u64, x : f32, y : f32, z : f32) {
    let t = transform_component_cast(&addr);
    t.scale(&Vector3::new(x, y, z));
}