use std::rc::Rc;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::RwLock;
use eldra_macro::{*};
use eldra_macro::Reflection;
use crate::data::{res_mgr, ExtSerializable, ResourceMgr};
use crate::data::texture::Texture;

#[derive(Reflection,Default)]
pub struct Skeleton {
    #[serialize]
    pub bone_count: u16,
}
impl ExtSerializable<Skeleton> for Skeleton {}

impl Skeleton {
    pub fn new() -> Box<Skeleton> {
        Box::new(Skeleton::default())
    }
}