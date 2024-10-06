use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::io::Read;
use std::fs;
use std::fs::File;
use std::any::{Any, TypeId};
use std::rc::{Rc, Weak};
use nalgebra::{Matrix2, Matrix3, Matrix4, Vector2, Vector3, Vector4};
use eldra_macro::*;
use yaml_rust2::{Yaml, YamlLoader};
use crate::{impl_map_concrete_serialize, impl_vec_concrete_serialize, impl_vec_embed_serialize};
use crate::entity::{Component, DummyComponent};
use crate::reflection::Serializable;
use crate::shader::{*};

#[derive(Default,Reflection)]
struct InputPin
{
    #[serialize]
    id: u32,

    #[serialize]
    display_name: String,

    var_type: ShaderVar,

    pub parent: Weak<RefCell<ShaderNode>>,
    pub from: Weak<RefCell<OutputPin>>,
}
impl InputPin {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(InputPin::default()))
    }
}
impl_vec_concrete_serialize!(Rc, RefCell, InputPin, new, borrow, borrow_mut);

#[derive(Default,Reflection)]
struct OutLink
{
    #[serialize]
    node_id: u32,
    #[serialize]
    pin_id: u32,
    pin: Weak<RefCell<InputPin>>,
}
impl_vec_embed_serialize!(OutLink);

#[derive(Default,Reflection)]
struct OutputPin
{
    #[serialize]
    id: u32,
    #[serialize]
    pub to: Vec<OutLink>,

    var_type: ShaderVar,
    pub parent: Weak<RefCell<ShaderNode>>,
}
impl OutputPin {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(OutputPin::default()))
    }
}
impl_vec_concrete_serialize!(Rc, RefCell, OutputPin, new, borrow, borrow_mut);

#[derive(Reflection)]
struct ShaderNode
{
    #[serialize]
    id: u32,
    myself: Weak<RefCell<ShaderNode>>,
    #[serialize]
    pub pos: Vector2<f32>,
    #[serialize]
    pub input_pin: Vec<Rc<RefCell<InputPin>>>,
    #[serialize]
    pub output_pin: Vec<Rc<RefCell<OutputPin>>>,
    #[serialize]
    pub validator: Box<dyn Component>,
    #[serialize]
    pub generator: Box<dyn Component>,
}
impl Default for ShaderNode {
    fn default() -> ShaderNode {
        ShaderNode {
            validator: Box::new(DummyComponent::default()),
            generator: Box::new(DummyComponent::default()),
            ..Default::default()
        }
    }
}
impl ShaderNode {
    pub fn new() -> Rc<RefCell<ShaderNode>> {
        let n = Rc::new(RefCell::new(ShaderNode::default()));
        n.borrow_mut().myself = Rc::downgrade(&n);
        n
    }
    pub fn get_input_pin(&self, id: u32) -> Weak<RefCell<InputPin>> {
        for pin in self.input_pin.iter() {
            if pin.borrow().id == id {
                return Rc::downgrade(&pin)
            }
        }
        panic!("pin not found")
    }
    pub fn id(&self) -> u32 { self.id }
}
impl_map_concrete_serialize!(u32, id, Rc, RefCell, ShaderNode, new, borrow, borrow_mut);

#[derive(Default,Reflection)]
struct ShaderGraph
{
    #[serialize]
    pub zoom_scale: f32,
    #[serialize]
    pub center: Vector2<f32>,

    #[serialize]
    seq: u32,
    #[serialize]
    pub nodes: HashMap<u32, Rc<RefCell<ShaderNode>>>,
}

impl ShaderGraph {
    pub fn new() -> ShaderGraph {
        ShaderGraph::default()
    }
    pub fn load_from_file(&mut self, yaml_path: &str) {
        let yaml_str = fs::read_to_string(yaml_path).unwrap();
        let docs = YamlLoader::load_from_str(&yaml_str).unwrap();
        let doc = &docs[0];
        self.deserialize_text(doc);

        for elem in self.nodes.values() {
            let e = elem.borrow_mut();
            // fix pin parent
            for pin in e.input_pin.iter() {
                pin.borrow_mut().parent = e.myself.clone();
            }
            for pin in e.output_pin.iter() {
                let mut out_pin = pin.borrow_mut();
                out_pin.parent = e.myself.clone();
                // fix connection
                for link in out_pin.to.iter_mut() {
                    let link_node = self.nodes.get(&link.node_id).unwrap();
                    let linked_input_pin = link_node.borrow().get_input_pin(link.pin_id);
                    // setup connection
                    linked_input_pin.upgrade().unwrap().borrow_mut().from = Rc::downgrade(&pin);
                    link.pin = linked_input_pin;
                }
            }
        }
    }
}