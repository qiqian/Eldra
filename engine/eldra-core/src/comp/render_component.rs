use std::ptr::addr_of;
use std::any::{Any, TypeId};
use std::io::{Read, Write};
use std::str::FromStr;
use std::rc::{Rc, Weak};
use nalgebra::{*};
use uuid::Uuid;
use yaml_rust2::Yaml;
use eldra_macro::{*};
use crate::comp::transform_component::TransformComponent;
use crate::data::*;
use crate::data::material::Material;
use crate::data::skeleton::Skeleton;
use crate::data::render_object::{RenderObject};
use crate::decode_component;
use crate::entity::{*};
use crate::reflection::{*};


#[derive(Reflection,ComponentAttr,Default)]
#[uuid="f8128f7a-685e-4436-a831-3a2adab3b0dc"]
pub struct RenderComponent {
    pub base: BaseObject,
    #[serialize]
    pub object: ExtRes<RenderObject>,
    #[serialize]
    pub skeleton: ExtRes<Skeleton>,
}
impl Uniq for RenderComponent {
    fn is_uniq() -> bool { true }
}
impl Component for RenderComponent {
    fn tick(&mut self, _delta: f32, ancestor: &Option<&Components>) {
        // todo ?
    }
}
