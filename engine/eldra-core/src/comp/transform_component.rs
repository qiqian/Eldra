use std::ptr::addr_of;
use std::any::{Any, TypeId};
use std::any::type_name;
use std::io::Write;
use nalgebra::{*};
use eldra_macro::{*};
use crate::decode_component;
use crate::entity::{*};
use crate::engine::{*};
use crate::reflection::{*};

#[derive(Reflection,DropNotify,ComponentAttr)]
pub struct TransformComponent
{
    #[serialize]
    pub base: BaseObject,

    #[display="Local Matrix"]
    #[serialize]
    pub local_matrix: Matrix4<f32>,

    #[display="World Matrix"]
    #[serialize]
    #[readonly]
    pub world_matrix: Matrix4<f32>,
}

impl Default for TransformComponent {
    fn default() -> TransformComponent {
        let mut t = TransformComponent {
            base:BaseObject::default(),
            local_matrix: Default::default(),
            world_matrix: Default::default(),
        };
        t.local_matrix.fill_with_identity();
        t.world_matrix.fill_with_identity();
        t
    }
}
impl Boxed<TransformComponent> for TransformComponent {
    fn boxed() -> Box<TransformComponent> {
        Box::new(TransformComponent::default())
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
impl Uniq for TransformComponent {
    fn is_uniq() -> bool { true }
}
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

fn transform_component_cast(addr : &Option<*mut dyn Component>) -> &mut TransformComponent {
    unsafe {
        let a = addr.unwrap_unchecked();
        &mut*(a as *mut TransformComponent)
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