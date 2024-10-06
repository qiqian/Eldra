use std::any::{Any, TypeId};
use std::io::Write;
use std::str::FromStr;
use eldra_macro::{*};
use crate::data::*;
use crate::data::skeleton::Skeleton;
use crate::data::render_object::{RenderObject};
use crate::impl_serializable_dyn_type;
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
impl Uniq for RenderComponent {}
impl Component for RenderComponent {}
impl_serializable_dyn_type!(RenderComponent, Component);

