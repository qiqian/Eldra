use std::ptr::addr_of;
use std::any::{Any, TypeId};
use std::str::FromStr;
use nalgebra::{*};
use eldra_macro::{*};
use crate::{decode_component, impl_serializable_dyn_type};
use crate::entity::{*};
use crate::reflection::{*};

#[derive(Reflection,ComponentAttr)]
#[uuid="bd122d2f-cc3e-4d99-8bf2-ba1b23015e46"]
pub struct TransformComponent
{
    pub base: BaseObject,

    #[display="Local Matrix"]
    #[serialize]
    pub local_matrix: Matrix4<f32>,

    #[display="World Matrix"]
    #[serialize]
    #[readonly]
    pub world_matrix: Matrix4<f32>,
}
impl_serializable_dyn_type!(TransformComponent, Component);

impl Default for TransformComponent {
    fn default() -> TransformComponent {
        TransformComponent {
            base:BaseObject::default(),
            local_matrix: Matrix4::identity(),
            world_matrix: Matrix4::identity(),
        }
    }
}
impl TransformComponent {
    pub fn translate(&mut self, v: &Vector3<f32>) {
        let _ = self.local_matrix.append_translation(v);
    }

    pub fn rotate(&mut self, angles: &Vector3<f32>) {
        let r = Matrix4::from_scaled_axis(*angles);
        self.local_matrix = r * self.local_matrix;
    }

    pub fn scale(&mut self, scale: &Vector3<f32>) {
        self.local_matrix.append_nonuniform_scaling_mut(scale);
    }
}
impl Uniq for TransformComponent {}
impl Component for TransformComponent {
    fn tick(&mut self, _delta: f32, ancestor: &Option<&Components>) {
        if ancestor.is_none() {
            self.world_matrix = self.local_matrix.clone();
            return;
        }
        let p = &(*ancestor);
        let tr = unsafe { p.unwrap_unchecked().get_component::<TransformComponent>() };
        self.world_matrix = tr.unwrap().world_matrix * self.local_matrix;
    }
}

//// exports

fn transform_component_update<F: Fn(&mut TransformComponent)>(me: u64, f: F) -> bool
{
    match decode_component!(me) {
        Some(c) => {
            match c.as_any_mut().downcast_mut::<TransformComponent>() {
                Some(tr) => {
                    f(tr);
                    true
                },
                None => false
            }
        },
        None => false
    }
}


#[no_mangle]
pub extern "C"
fn TransformComponent_translate(me: u64, x : f32, y : f32, z : f32) -> bool {
    transform_component_update(me, |tr| {
        tr.translate(&Vector3::new(x, y, z));
    })
}

#[no_mangle]
pub extern "C"
fn TransformComponent_rotate(me: u64, x : f32, y : f32, z : f32) -> bool {
    transform_component_update(me, |tr| {
        tr.rotate(&Vector3::new(x, y, z));
    })
}

#[no_mangle]
pub extern "C"
fn TransformComponent_scale(me: u64, x : f32, y : f32, z : f32) -> bool{
    transform_component_update(me, |tr| {
        tr.scale(&Vector3::new(x, y, z));
    })
}