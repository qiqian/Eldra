use std::any::{Any, TypeId};
use std::io::*;
use eldra_macro::{*};
use crate::data::{ExtRes, ExtSerializable};
use crate::data::texture::Texture;
use crate::reflection::Serializable;
use crate::impl_vec_embed_serialize;
use yaml_rust2::Yaml;

#[derive(Reflection,Default)]
pub struct Material
{
    #[serialize]
    pub name: String,
    #[serialize]
    pub tex: ExtRes<Texture>,
}
impl_vec_embed_serialize!(Material);
impl ExtSerializable<Material> for Material {}