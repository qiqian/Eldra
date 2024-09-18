use std::cell::RefCell;
use std::rc::Weak;
use crate::comp::uarg::UArg;
use crate::node::BaseObject;
use super::super::node;
pub struct TransformComponent
{
    base: BaseObject,
    myself: Weak<RefCell<TransformComponent>>,

    // pos
    // rot
    // scale
}