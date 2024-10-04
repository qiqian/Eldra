use std::ptr::addr_of;
use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::ops::Deref;
use std::str::FromStr;
use std::rc::{Rc, Weak};
use std::sync::RwLock;
use gltf::{Accessor, Document, Gltf, Semantic};
use nalgebra::{*};
use base64::prelude::*;
use gltf::accessor::Dimensions;
use gltf::buffer::Source;
use gltf::mesh::Mode;
use gltf_json::accessor::Type;
use eldra_macro::{Reflection};
use crate::data::{res_mgr, ExtRes, ExtSerializable};
use crate::data::material::Material;
use crate::{impl_option_embed_serialize, impl_vec_embed_serialize};
use crate::reflection::{Serializable};
use yaml_rust2::{Yaml, YamlLoader};
use yaml_rust2::Yaml::Hash;
use crate::data::render_object::BufferType::{INDEX, VERTEX};

#[derive(Reflection,Default)]
pub enum BufferType
{
    #[default]
    VERTEX = 0,
    INDEX = 1,
}
impl BufferType {
    fn to_gltf(&self) -> i32 {
        match self {
            BufferType::VERTEX => 34962,// ARRAY_BUFFER
            BufferType::INDEX => 34963, // ELEMENT_ARRAY_BUFFER
        }
    }
    fn decode_gltf(&mut self, v: i32) {
        match v {
            34962 => { *self = BufferType::VERTEX; },
            34963 => { *self = BufferType::INDEX; },
            _ => panic!("invalid enum value for BufferType"),
        }
    }
}
#[derive(Reflection,Default)]
pub enum DataType {
    #[default]
    SCALA = 0,
    VEC2 = 1, VEC3 = 2, VEC4 = 3,
    MAT2 = 4, MAT3 = 5, MAT4 = 6,
}
impl DataType {
    fn decode_gltf(&mut self, v: Dimensions) {
        match v {
            Type::Scalar => { *self = DataType::SCALA; },
            Type::Vec2 => { *self = DataType::VEC2; },
            Type::Vec3 => { *self = DataType::VEC3; },
            Type::Vec4 => { *self = DataType::VEC4; },
            Type::Mat2 => { *self = DataType::MAT2; },
            Type::Mat3 => { *self = DataType::MAT3; },
            Type::Mat4 => { *self = DataType::MAT4; },
        }
    }
}
#[derive(Reflection,Default)]
pub enum ComponentType {
    #[default]
    F32 = 0,
    U32 = 1,
    S8 = 2, U8 = 3,
    S16 = 4, U16 = 5,
}
impl ComponentType {
    fn to_gltf(&self) -> i32 {
        match self {
            ComponentType::S8 => 5120,
            ComponentType::U8 => 5121,
            ComponentType::S16 => 5122,
            ComponentType::U16 => 5123,
            ComponentType::U32 => 5125,
            ComponentType::F32 => 5126,
        }
    }
    fn decode_gltf(&mut self, v: gltf_json::accessor::ComponentType) {
        match v {
            gltf_json::accessor::ComponentType::I8 => { *self = ComponentType::S8; },
            gltf_json::accessor::ComponentType::U8 => { *self = ComponentType::U8; },
            gltf_json::accessor::ComponentType::I16 => { *self = ComponentType::S16; },
            gltf_json::accessor::ComponentType::U16 => { *self = ComponentType::U16; },
            gltf_json::accessor::ComponentType::U32 => { *self = ComponentType::U32; },
            gltf_json::accessor::ComponentType::F32 => { *self = ComponentType::F32; },
            _ => panic!("invalid enum value for BufferType"),
        }
    }
}
#[derive(Reflection,Default)]
pub enum PrimitiveMode { // value same as gltf spec
    POINTS = 0,
    LINES = 1,
    LINE_LOOP = 2,
    LINE_STRIP = 3,
    #[default]
    TRIANGLES = 4,
    TRIANGLE_STRIP = 5,
    TRIANGLE_FAN = 6,
}
impl PrimitiveMode {
    fn decode_gltf(&mut self, mode: Mode) {
        match mode {
            Mode::Points => { *self = PrimitiveMode::POINTS; },
            Mode::Lines => { *self = PrimitiveMode::LINES; },
            Mode::LineLoop => { *self = PrimitiveMode::LINE_LOOP; },
            Mode::LineStrip => { *self = PrimitiveMode::LINE_STRIP; },
            Mode::Triangles => { *self = PrimitiveMode::TRIANGLES; },
            Mode::TriangleStrip => { *self = PrimitiveMode::TRIANGLE_STRIP; },
            Mode::TriangleFan => { *self = PrimitiveMode::TRIANGLE_FAN; },
        }
    }
}
#[derive(Reflection,Default)]
pub struct BufferView
{
    #[serialize]
    pub offset: u32,
    #[serialize]
    pub byte_size: u32,
    #[serialize]
    pub vertex_stride: u32, // only useful if buffer_type is VERTEX
    #[serialize]
    pub count: u32,
    #[serialize]
    pub buffer_type: BufferType,
    #[serialize]
    pub data_type: DataType,
    #[serialize]
    pub component_type: ComponentType,
    #[serialize]
    pub normalized: bool,
}
impl_vec_embed_serialize!(BufferView);
impl_option_embed_serialize!(BufferView);
#[derive(Reflection,Default)]
pub struct SkinDataVec4 {
    #[serialize]
    pub joints: BufferView,
    #[serialize]
    pub weights: BufferView,
}
impl_vec_embed_serialize!(SkinDataVec4);
#[derive(Reflection,Default)]
pub struct Primitive {
    #[serialize]
    pub position: BufferView,
    #[serialize]
    pub normal: Option<BufferView>,
    #[serialize]
    pub tangent: Option<BufferView>,
    #[serialize]
    pub texcoord: Vec<BufferView>,
    #[serialize]
    pub color: Vec<BufferView>,
    #[serialize]
    pub skin: Vec<SkinDataVec4>,

    #[serialize]
    pub indice: Option<BufferView>,
    #[serialize]
    pub material: ExtRes<Material>,
    #[serialize]
    pub mode: PrimitiveMode,
}
impl_vec_embed_serialize!(Primitive);
#[derive(Reflection,Default)]
pub struct RenderPart
{
    #[serialize]
    pub name: String,
    #[serialize]
    pub primitives: Vec<Primitive>,
}
impl_vec_embed_serialize!(RenderPart);
#[derive(Reflection,Default)]
pub struct RenderObject
{
    #[serialize]
    pub buffer: Vec<u8>,
    #[serialize]
    pub parts: Vec<RenderPart>,
}
impl_vec_embed_serialize!(RenderObject);

#[derive(Clone,Copy,PartialEq)]
enum BufferLoopOrder
{
    POS, NORMAL, TANGENT, TEXCOORD, COLOR, SKIN
}
#[derive(Clone)]
struct BViewInfo<'a> {
    pub buffer_view: &'a BufferView,
    pub attr_name: String,
}
struct BViewIterator<'a> {
    pub curr_mesh: &'a Primitive,
    pub buffer_loop_order: BufferLoopOrder, // pos(0)->normal(1)->tangent(2)->texcoord(3)->color(4)->skin(5)
    pub buffer_vec_index: isize, // index in texcoord_n(3)/color_n(4)/skin_n(5)
    pub skin_index: isize,
}
impl<'a> BViewIterator<'a> {
    fn next_skin(&mut self) -> Option<BViewInfo<'a>> {
        if self.buffer_loop_order == BufferLoopOrder::SKIN {
            if self.curr_mesh.skin.is_empty() ||
                self.buffer_vec_index >= self.curr_mesh.skin.len() as isize ||
                (self.buffer_vec_index == (self.curr_mesh.skin.len() as isize - 1) && self.skin_index >= 1)
            {
                None
            }
            else {
                let skin = &self.curr_mesh.skin[self.buffer_vec_index as usize];
                self.skin_index += 1;
                if self.skin_index <= 1 {
                    Some(BViewInfo{
                        buffer_view: if self.skin_index == 0 { &skin.joints } else { &skin.weights },
                        attr_name:
                            if self.skin_index == 0 {
                                format!("JOINTS_{}", self.buffer_vec_index).to_string()
                            } else {
                                format!("WEIGHTS_{}", self.buffer_vec_index).to_string()
                            },
                    })
                }
                else { // next skin
                    self.buffer_vec_index += 1;
                    self.skin_index = -1;
                    self.next_skin()
                }
            }
        }
        else {
            panic!("invalid primitive loop order");
        }
    }
}
impl<'a> Iterator for BViewIterator<'a> {
    type Item = BViewInfo<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        // pos
        if self.buffer_loop_order == BufferLoopOrder::POS {
            if self.buffer_vec_index < 0 {
                self.buffer_vec_index = 0;
                return Some(BViewInfo{
                    buffer_view: &self.curr_mesh.position,
                    attr_name: "POSITION".to_string(),
                })
            } else { // move to normal
                self.buffer_loop_order = BufferLoopOrder::NORMAL;
                self.buffer_vec_index = -1;
            }
        }
        // normal
        if self.buffer_loop_order == BufferLoopOrder::NORMAL {
            if self.curr_mesh.normal.is_some() && self.buffer_vec_index < 0 {
                self.buffer_vec_index = 0;
                return Some(BViewInfo{
                    buffer_view: &self.curr_mesh.normal.as_ref().unwrap(),
                    attr_name: "NORMAL".to_string(),
                });
            } else { // move to tangent
                self.buffer_loop_order = BufferLoopOrder::TANGENT;
                self.buffer_vec_index = -1;
            }
        }
        // tangent
        if self.buffer_loop_order == BufferLoopOrder::TANGENT {
            if self.curr_mesh.tangent.is_some() && self.buffer_vec_index < 0 {
                self.buffer_vec_index = 0;
                return Some(BViewInfo{
                    buffer_view: &self.curr_mesh.tangent.as_ref().unwrap(),
                    attr_name: "TANGENT".to_string(),
                });
            } else { // move to texcoord
                self.buffer_loop_order = BufferLoopOrder::TEXCOORD;
                self.buffer_vec_index = -1;
            }
        }
        if self.buffer_loop_order == BufferLoopOrder::TEXCOORD {
            if self.curr_mesh.texcoord.is_empty() ||
                self.buffer_vec_index >= (self.curr_mesh.texcoord.len() as isize - 1) {
                // move to color
                self.buffer_loop_order = BufferLoopOrder::COLOR;
                self.buffer_vec_index = -1;
            } else { // next texcoord
                self.buffer_vec_index += 1;
                return Some(BViewInfo{
                    buffer_view: &self.curr_mesh.texcoord[self.buffer_vec_index as usize],
                    attr_name: format!("TEXCOORD_{}", self.buffer_vec_index),
                })
            }
        }
        if self.buffer_loop_order == BufferLoopOrder::COLOR {
            if self.curr_mesh.color.is_empty() ||
                self.buffer_vec_index >= (self.curr_mesh.color.len() as isize - 1) {
                // move to skin
                self.buffer_loop_order = BufferLoopOrder::SKIN;
                self.buffer_vec_index = 0; // continue
                self.skin_index = -1;
            } else { // next color
                self.buffer_vec_index += 1;
                return Some(BViewInfo{
                    buffer_view: &self.curr_mesh.color[self.buffer_vec_index as usize],
                    attr_name: format!("COLOR_{}", self.buffer_vec_index),
                })
            }
        }
        self.next_skin()
    }
}
impl Primitive {
    fn vertex_iter(&self) -> BViewIterator {
        BViewIterator {
            curr_mesh: self,
            buffer_loop_order: BufferLoopOrder::POS,
            buffer_vec_index: -1,
            skin_index: -1,
        }
    }
}

impl ExtSerializable<RenderObject> for RenderObject {
    fn text_ext() -> &'static str { "gltf" }
    fn deserialize_from_text_file(res: &mut RenderObject, respath: &String) {
        let result = gltf::import(respath);
        let (document, buffers, images) = result.unwrap();
        res.read_gltf(&document, &buffers, &images);
    }
}
impl RenderObject {
    pub fn new() -> Box<Self> {
        Box::new(RenderObject::default())
    }

    pub fn write_gltf(&self, io: &mut dyn Write) {
        // header
        {
            let _ = io.write(r#"{
    "asset": {
        "generator": "Eldra",
        "version": "2.0"
    },"#.as_bytes());
        }

        // buffer
        {
            let _ = io.write(format!(r#"
    "buffers": [
        {{
            "byteLength": {},
            "uri": "data:application/octet-stream;base64,{}"
        }}
    ],"#, self.buffer.len(), BASE64_STANDARD.encode(self.buffer.as_slice())).as_bytes());
        }

        // buffer view
        {
            let _ = io.write(format!(r#"
    "bufferViews": ["#).as_bytes());
            for part in self.parts.iter() {
                for prim in part.primitives.iter() {
                    match prim.indice.as_ref() {
                        Some(view) => {
                            let _ = io.write(format!(r#"
        "{{
            "buffer": 0,
            "byteOffset": {},
            "byteLength": {},
            "byteStride": {},
            "target": {},
        }},"#,              view.offset, view.byte_size, view.vertex_stride,
                            view.buffer_type.to_gltf()).as_bytes());
                        }
                        None => {}
                    }
                    for info in prim.vertex_iter() {
                        let view = info.buffer_view;
                        let _ = io.write(format!(r#"
        "{{
            "buffer": 0,
            "byteOffset": {},
            "byteLength": {},
            "byteStride": {},
            "target": {},
        }},"#,          view.offset, view.byte_size, view.vertex_stride,
                        view.buffer_type.to_gltf()).as_bytes());
                    }
                }
            }
            let _ = io.write(format!(r#"
    "],"#).as_bytes());
        }

        // accessors
        {
            let mut view_index = 0;
            let _ = io.write(format!(r#"
    "accessors": ["#).as_bytes());
            for part in self.parts.iter() {
                for prim in part.primitives.iter() {
                    match prim.indice.as_ref() {
                        Some(view) => {
                            let _ = io.write(format!(r#"
        "{{
            "bufferView": {},
            "byteOffset": 0,
            "componentType": {},
            "count": {},
            "type": "{}",
        }},"#,              view_index, view.component_type.to_gltf(),
                            view.count, view.data_type.to_string()).as_bytes());
                            view_index += 1;
                        },
                        None => {},
                    };
                    for info in prim.vertex_iter() {
                        let view = info.buffer_view;
                        let _ = io.write(format!(r#"
        "{{
            "bufferView": {},
            "byteOffset": 0,
            "componentType": {},
            "count": {},
            "type": "{}",
        }},"#,          view_index, view.component_type.to_gltf(),
                        view.count, view.data_type.to_string()).as_bytes());
                        view_index += 1;
                    }
                }
            }
            let _ = io.write(format!(r#"
    "],"#).as_bytes());
        }

        // meshes
        {
            let mut view_index = 0;
            let _ = io.write(format!(r#"
    "meshes": ["#).as_bytes());
            for part in self.parts.iter() {
                let _ = io.write(format!(r#"
    {{
        "name": "{}",
        "primitives": ["#, part.name).as_bytes());
                for prim in part.primitives.iter() {
                    let _ = io.write(r#"
        {{"#.as_bytes());
                    match prim.indice.as_ref() {
                        Some(view) => {
                            let _ = io.write(format!(r#"
            "indices": {},
            "mode": {},
            "attributes": {{
            "#,             view_index, prim.mode.to_i32()).as_bytes());
                            view_index += 1;
                        },
                        None => {},
                    }
                    for info in prim.vertex_iter() {
                        let view = info.buffer_view;
                        let _ = io.write(format!(r#"
                "{}": {},"#, info.attr_name, view_index).as_bytes());
                        view_index += 1;
                    }
                    let _ = io.write(r#"
            }},
        }},"#.as_bytes());
                }
                let _ = io.write(format!(r#"
        "],"#).as_bytes());
            }
            let _ = io.write(format!(r#"
    "],"#).as_bytes());
        }
    }

    pub fn read_gltf(&mut self, doc:&Document,
             buffers:&Vec<gltf::buffer::Data>, images:&Vec<gltf::image::Data>) {
        let mut buffer_offset_fix:Vec<u32> = Vec::new();
        {
            let mut offset = 0_u32;
            for buffer in doc.buffers() {
                buffer_offset_fix.push(offset);
                offset += buffer.length() as u32;
                // merge buffer
                let data = buffers[buffer.index()].deref();
                self.buffer.extend_from_slice(data);
            }
        }
        let fn_view_from_accessor = |accessor:Accessor, indice:bool| -> BufferView {
            let gltf_view = accessor.view().unwrap();
            let mut view = BufferView::default();
            view.count = accessor.count() as u32;
            view.offset = (gltf_view.offset() + accessor.offset()) as u32 + buffer_offset_fix[gltf_view.index()];
            view.component_type.decode_gltf(accessor.data_type());
            view.normalized = accessor.normalized();
            view.vertex_stride = gltf_view.stride().unwrap_or(0) as u32;
            view.byte_size = gltf_view.length() as u32;
            view.data_type.decode_gltf(accessor.dimensions());
            view.buffer_type = if indice { INDEX } else { VERTEX };
            let _ = accessor.size();
            let _ = accessor.name();
            view
        };
        for gltf_mesh in doc.meshes() {
            let mut part = RenderPart::default();
            for gltf_prim in gltf_mesh.primitives() {
                let mut primitive = Primitive::default();
                primitive.mode.decode_gltf(gltf_prim.mode());
                // indice
                match gltf_prim.indices() {
                    Some(accessor) => { primitive.indice = Some(fn_view_from_accessor(accessor, true)); },
                    None => {},
                }
                // vertex stream
                let mut color_map = HashMap::<u32,BufferView>::new();
                let mut texcoord_map = HashMap::<u32,BufferView>::new();
                let mut joint_map = HashMap::<u32,BufferView>::new();
                let mut weight_map = HashMap::<u32,BufferView>::new();
                for gltf_attr in gltf_prim.attributes() {
                    let accessor = gltf_attr.1;
                    let semantic = gltf_attr.0;
                    match semantic {
                        Semantic::Positions => { primitive.position = fn_view_from_accessor(accessor, false); },
                        Semantic::Normals => { primitive.normal = Some(fn_view_from_accessor(accessor, false)); },
                        Semantic::Tangents => { primitive.tangent = Some(fn_view_from_accessor(accessor, false)); },
                        Semantic::Colors(i) => { color_map.insert(i, fn_view_from_accessor(accessor, false)); },
                        Semantic::TexCoords(i) => { texcoord_map.insert(i, fn_view_from_accessor(accessor, false)); },
                        Semantic::Joints(i) => { joint_map.insert(i, fn_view_from_accessor(accessor, false)); },
                        Semantic::Weights(i) => { weight_map.insert(i, fn_view_from_accessor(accessor, false)); },
                        _ => {},
                    }
                }
                // sort vectors
                for i in 0_u32..color_map.len() as u32 {
                    primitive.color.push(color_map.remove(&i).unwrap());
                }
                for i in 0_u32..texcoord_map.len() as u32 {
                    primitive.texcoord.push(texcoord_map.remove(&i).unwrap());
                }
                for i in 0_u32..joint_map.len() as u32 {
                    primitive.skin.push(SkinDataVec4 {
                        joints: joint_map.remove(&i).unwrap(),
                        weights: weight_map.remove(&i).unwrap(),
                    });
                }
                // gen part
                part.primitives.push(primitive);
            }
            self.parts.push(part);
        }
        for skin in doc.skins() {

        }
        {
            // this is just for debug purpose of standalone viewer
            for mat in doc.materials() {
                let metallic_roughness = mat.pbr_metallic_roughness();
                let specular_glossiness = mat.pbr_specular_glossiness();
            }
            for tex in doc.images() {}
        }
    }
}