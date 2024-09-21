use std::any::Any;
use std::any::type_name;
use nalgebra::{*};
use crate::entity::{*};
use crate::engine::{*};

#[derive(Default)]
pub struct TransformComponent
{
    pub base: BaseObject,
    pub local_matrix: Matrix4<f32>,
    pub world_matrix: Matrix4<f32>,
}

impl Drop for TransformComponent {
    fn drop(&mut self) {
        engine_notify_drop_object(type_name::<TransformComponent>(), self.base.id);
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
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn tick(&mut self, delta: f32, ancestor: *const Entity) {
        if ancestor == (0 as *const Entity) {
            self.world_matrix = self.local_matrix.clone();
            return;
        }
        let p = unsafe { &(*ancestor) };
        let tr = p.get_component::<TransformComponent>();
        self.world_matrix = tr.unwrap().world_matrix * self.local_matrix;
    }
}

fn transform_component_cast<'a>(addr : &'a u64) -> &'a mut TransformComponent {
    unsafe {
        &mut*(*addr as *mut TransformComponent)
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