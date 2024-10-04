use std::rc::Rc;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::RwLock;
use eldra_macro::{*};
use crate::data::{res_mgr, ExtSerializable};
use crate::data::material::Material;

#[derive(Reflection,Default)]
pub struct Texture
{
    #[serialize]
    pub pixel_format: u8,
}
impl ExtSerializable<Texture> for Texture {}
impl Texture {
    pub fn new() -> Box<Texture> {
        Box::new(Texture::default())
    }
}