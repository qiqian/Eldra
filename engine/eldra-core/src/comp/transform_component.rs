use std::any::{Any, TypeId};
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
impl Uniq for TransformComponent {
    fn is_uniq() -> bool { true }
}
impl Component for TransformComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn real_type_id(&self) -> TypeId { TypeId::of::<Self>() }
    fn is_comp_uniq(&self) -> bool { Self::is_uniq() }
    fn tick(&mut self, delta: f32, ancestor: &Option<&Components>) {
        if ancestor.is_none() {
            self.world_matrix = self.local_matrix.clone();
            return;
        }
        let p = unsafe { &(*ancestor) };
        let tr = unsafe { p.unwrap_unchecked().get_component::<TransformComponent>() };
        self.world_matrix = tr.unwrap().world_matrix * self.local_matrix;
    }
}

fn transform_component_cast<'a>(addr : &Option<*mut dyn Component>) -> &'a mut TransformComponent {
    unsafe {
        let a = addr.unwrap_unchecked();
        &mut*(a as *mut TransformComponent)
    }
}

//// exports


#[no_mangle]
pub extern "C"
fn TransformComponent_translate(me: Option<*mut dyn Component>, x : f32, y : f32, z : f32) {
    let t = transform_component_cast(&me);
    t.translate(&Vector3::new(x, y, z));
}

#[no_mangle]
pub extern "C"
fn TransformComponent_rotate(me: Option<*mut dyn Component>, x : f32, y : f32, z : f32) {
    let t = transform_component_cast(&me);
    t.rotate(&Vector3::new(x, y, z));
}

#[no_mangle]
pub extern "C"
fn TransformComponent_scale(me: Option<*mut dyn Component>, x : f32, y : f32, z : f32) {
    let t = transform_component_cast(&me);
    t.scale(&Vector3::new(x, y, z));
}