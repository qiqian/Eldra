use std::cell::RefCell;
use std::rc::Weak;
use crate::entity::{*};
use super::uarg::{UArg};

pub struct ScriptComponent
{
    base: BaseObject,
    myself: Weak<RefCell<ScriptComponent>>,

    script: String,
    args: Vec<UArg>,
}

