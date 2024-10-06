use std::any::{Any, TypeId};
use eldra_macro::{*};
use crate::data::ExtSerializable;

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