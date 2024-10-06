use std::any::{Any, TypeId};
use eldra_macro::Reflection;
use crate::data::ExtSerializable;

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