use std::any::{Any, TypeId};
use std::fs::File;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write, BufReader, BufWriter};
use std::ptr::addr_of_mut;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use nalgebra::{ArrayStorage, Dim, Matrix, RawStorageMut, Vector2, Vector3, Vector4, U1, U3};
use once_cell::sync::OnceCell;
use uuid::Uuid;
use yaml_rust2::{Yaml, YamlLoader};
use std::mem::MaybeUninit;
use crate::data::*;
use crate::comp::render_component::RenderComponent;
use crate::comp::transform_component::TransformComponent;
use crate::data::material::Material;
use crate::data::render_object::{BufferView, Primitive, RenderObject, SkinDataVec4};
use crate::entity::{Component, Entity};
use crate::shader::register_shader_graph_components;

#[macro_export]
macro_rules! register_serializable_type {
    ( $x:ident,$y:ident ) => {
        $x.insert($y::type_uuid().unwrap(), $y::dyn_box);
    }
}
#[macro_export]
macro_rules! impl_serializable_dyn_type {
    ( $x:ident,$y:ident ) => {
        impl $x {
            pub fn dyn_box() -> Box<dyn $y> { Box::new($x::default()) }
        }
    }
}

pub unsafe fn init_reflection() {
    DYN_NEW_REG.get_or_init (|| { DynNewReg::default() });

    let reg = &mut DYN_NEW_REG.get_mut().unwrap_unchecked().Component;
    register_serializable_type!(reg, TransformComponent);
    register_serializable_type!(reg, RenderComponent);
    register_shader_graph_components(reg);
}

#[derive(Default)]
struct DynNewReg {
    Component: HashMap<Uuid, fn()->Box<dyn Component>>,
}
static mut DYN_NEW_REG : OnceCell<DynNewReg> = OnceCell::new();
#[derive(Debug,Default)]
pub struct ReflectVarInfo
{
    pub display: &'static str,
    pub serialize : bool,
    pub readonly : bool,
    pub offset : u32,
    pub size : u32,
}
pub trait Reflectable {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn real_type_id(&self) -> TypeId;
    // used for UI
    fn reflect_info(&self) -> Vec<ReflectVarInfo>;
}
pub trait Serializable {
    fn is_multi_line(&self) -> bool;
    // used for dyn trait serialization
    fn get_type_uuid(&self) -> Option<uuid::Uuid>;
    fn serialize_binary(&self, io: &mut dyn Write);
    fn deserialize_binary(&mut self, io: &mut dyn Read);
    fn serialize_text(&self, io: &mut SerializeTextWriter, indent: String);
    fn deserialize_text(&mut self, yaml: &Yaml);
}
pub struct SerializeTextWriter {
    writer: BufWriter<File>,
    newline: bool,
}
impl SerializeTextWriter {
    pub fn new(filepath: &str) -> SerializeTextWriter {
        let file = File::create(filepath).unwrap();
        SerializeTextWriter {
            writer: BufWriter::new(file),
            newline: false,
        }
    }
    pub fn write_all(&mut self, mut buf: &[u8]) -> std::io::Result<()> {
        self.newline = false;
        self.writer.write_all(buf)
    }
    pub fn newline(&mut self) {
        if !self.newline {
            self.newline = true;
            let _ = self.writer.write_all("\n".as_bytes());
        }
    }
}
impl Drop for SerializeTextWriter {
    fn drop(&mut self) {
        let _ = self.writer.flush();
    }
}
pub trait Uniq {
    fn is_uniq() -> bool { true}
}
impl Serializable for bool {
    fn is_multi_line(&self) -> bool { false }
    fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
    fn serialize_binary(&self, io: &mut dyn Write) {
        let d: [u8; 1] = [if *self { 1 } else { 0 } ];
        let _ = io.write_all(&d);
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        let mut d: [u8; 1] = [0];
        let _ = io.read_exact(&mut d);
        unsafe { *addr_of_mut!(*self) = d[0] == 0; };
    }

    fn serialize_text(&self, io: &mut SerializeTextWriter, _indent: String) {
        let _ = io.write_all(self.to_string().as_bytes());
    }

    fn deserialize_text(&mut self, yaml: &Yaml) {
        match yaml.as_bool() {
            Some(v) => { *self = v; },
            None => {},
        }
    }
}
#[macro_export]
macro_rules! impl_enum_serialize {
    ( $x:ty ) => {
        impl Serializable for $x {
            fn is_multi_line(&self) -> bool { false }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                let _ = io.write_all((self.value() as u8).to_le_bytes().as_ref());
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                let mut bytes = [0u8; 1];
                let _ = io.read_exact(me);
                unsafe { *addr_of_mut!(*self) = <$x>::from_le_bytes(bytes); };
            }

            fn serialize_text(&self, io: &mut crate::reflection::SerializeTextWriter, _indent: String) {
                let _ = io.write_all((*self as u8).to_string().as_bytes());
            }

            fn deserialize_text(&mut self, yaml: &Yaml) {
                match yaml.$yamlconv() {
                    Some(v) => { *self = v as $x; },
                    None => {},
                }
            }
        }
    };
}
#[macro_export]
macro_rules! impl_primitive_serialize {
    ( $x:ty,$yamlconv:ident ) => {
        impl Serializable for $x {
            fn is_multi_line(&self) -> bool { false }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                let _ = io.write_all(self.to_le_bytes().as_ref());
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                let mut bytes = self.to_le_bytes();
                let me = bytes.as_mut();
                let _ = io.read_exact(me);
                unsafe { *addr_of_mut!(*self) = <$x>::from_le_bytes(bytes); };
            }

            fn serialize_text(&self, io: &mut crate::reflection::SerializeTextWriter, _indent: String) {
                let _ = io.write_all(self.to_string().as_bytes());
            }

            fn deserialize_text(&mut self, yaml: &Yaml) {
                match yaml.$yamlconv() {
                    Some(v) => { *self = v as $x; },
                    None => {},
                }
            }
        }
    };
}
impl_primitive_serialize!(i8,as_i64);
impl_primitive_serialize!(u8,as_i64);
impl_primitive_serialize!(i16,as_i64);
impl_primitive_serialize!(u16,as_i64);
impl_primitive_serialize!(i32,as_i64);
impl_primitive_serialize!(u32,as_i64);
impl_primitive_serialize!(i64,as_i64);
impl_primitive_serialize!(f32,as_f64);
impl_primitive_serialize!(f64,as_f64);
impl<T, R, C, S> Serializable for Matrix<T, R, C, S> where T: Serializable + Default + ToString, R: Dim, C: Dim, S : RawStorageMut<T, R, C> {
    fn is_multi_line(&self) -> bool { false }
    fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
    fn serialize_binary(&self, io: &mut dyn Write) {
        self.iter().for_each(|e| {
            e.serialize_binary(io);
        });
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        self.iter_mut().for_each(|e| {
            e.deserialize_binary(io);
        });
    }

    fn serialize_text(&self, io: &mut SerializeTextWriter, _indent: String) {
        let _ = io.write_all("[ ".as_bytes());
        for col in self.column_iter() {
            for e in col.iter() {
                let _ = io.write_all(e.to_string().as_bytes());
                let _ = io.write_all(", ".as_bytes());
            }
        }
        let _ = io.write_all("]".as_bytes());
    }

    fn deserialize_text(&mut self, yaml: &Yaml) {
        let mut yiter = yaml.as_vec().unwrap().iter();
        self.iter_mut().for_each(|e| {
            e.deserialize_text(yiter.next().unwrap());
        });
    }
}
impl Serializable for String {
    fn is_multi_line(&self) -> bool { false }
    fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
    fn serialize_binary(&self, io: &mut dyn Write) {
        let data = self.as_bytes();
        let len = data.len() as i64;
        len.serialize_binary(io);
        let _ = io.write_all(data);
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        let mut len: i64 = 0;
        len.deserialize_binary(io);
        let len = len as usize;
        let mut str = Vec::<u8>::with_capacity(len);
        unsafe { str.set_len(len); }
        let _ = io.read_exact(str.as_mut());
        *self = String::from_utf8(str).expect("Malformed string");
    }

    fn serialize_text(&self, io: &mut SerializeTextWriter, _indent: String) {
        let _ = io.write_all(format!("\"{}\"", self).as_bytes());
    }

    fn deserialize_text(&mut self, yaml: &Yaml) {
        *self = yaml.as_str().unwrap().to_string();
    }
}
impl Serializable for Uuid {
    fn is_multi_line(&self) -> bool { false }
    fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
    fn serialize_binary(&self, io: &mut dyn Write) {
        let _ = io.write_all(self.as_bytes());
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        let bytes: MaybeUninit<[u8; 16]> = MaybeUninit::uninit();
        let mut buf = unsafe { bytes.assume_init() };
        let _ = io.read_exact(&mut buf);
        *self = Uuid::from_bytes(buf);
    }

    fn serialize_text(&self, io: &mut SerializeTextWriter, _indent: String) {
        let _ = io.write_all(format!("\"{}\"", self.to_string()).as_bytes());
    }

    fn deserialize_text(&mut self, yaml: &Yaml) {
        *self = Uuid::from_str(yaml.as_str().unwrap()).unwrap();
    }
}

#[macro_export]
macro_rules! impl_option_embed_serialize {
    ( $x:ident ) => {
        impl Serializable for Option<$x> {
            fn is_multi_line(&self) -> bool {
                match self {
                    Some(v) => v.is_multi_line(),
                    None => false,
                }
            }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                match self {
                    Some(v) => {
                        true.serialize_binary(io);
                        v.serialize_binary(io);
                    },
                    None => { false.serialize_binary(io); },
                }
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                let mut opt: bool = false;
                opt.deserialize_binary(io);
                if opt {
                    let mut v = $x::default();
                    v.deserialize_binary(io);
                    *self = Some(v);
                }
            }

            fn serialize_text(&self, io: &mut crate::reflection::SerializeTextWriter, indent: String) {
                match self {
                    Some(v) => {
                        let _ = io.write_all(format!("{}- array_item :", indent.clone()).as_bytes());
                        io.newline();
                        v.serialize_text(io, indent.clone() + "  ");
                        io.newline();
                    },
                    None => {
                        let _ = io.write_all("[]".as_bytes());
                    },
                }
            }

            fn deserialize_text(&mut self, data: &Yaml) {
                let arr = data.as_vec().unwrap();
                if arr.is_empty() {
                    return
                }
                let mut item = $x::default();
                item.deserialize_text(&arr[0]);
                *self = Some(item);
            }
        }
    }
}
#[macro_export]
macro_rules! impl_vec_embed_serialize {
    ( $x:ident ) => {
        impl crate::reflection::Serializable for Vec<$x> {
            fn is_multi_line(&self) -> bool { !self.is_empty() }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                (self.len() as i64).serialize_binary(io);
                for v in self.iter() {
                    v.serialize_binary(io);
                }
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                let mut len: i64 = 0;
                len.deserialize_binary(io);
                self.reserve(len as usize);
                for _i in 0..len {
                    let mut item = $x::default();
                    item.deserialize_binary(io);
                    self.push(item);
                }
            }

            fn serialize_text(&self, io: &mut crate::reflection::SerializeTextWriter, indent: String) {
                if self.is_empty() {
                    let _ = io.write_all("[]".as_bytes());
                }
                else {
                    for item in self.iter() {
                        let _ = io.write_all(format!("{}- array_item :", indent.clone()).as_bytes());
                        io.newline();
                        item.serialize_text(io, indent.clone() + "  ");
                        io.newline();
                    }
                }
            }

            fn deserialize_text(&mut self, data: &Yaml) {
                let arr = data.as_vec().unwrap();
                self.reserve(arr.len());
                for yaml in arr {
                    let mut item = $x::default();
                    item.deserialize_text(yaml);
                    self.push(item);
                }
            }
        }
    }
}
impl_vec_embed_serialize!(u8);
impl_vec_embed_serialize!(i8);
impl_vec_embed_serialize!(u16);
impl_vec_embed_serialize!(i16);
impl_vec_embed_serialize!(u32);
impl_vec_embed_serialize!(i32);
impl_vec_embed_serialize!(i64);
impl_vec_embed_serialize!(f32);
impl_vec_embed_serialize!(f64);
pub type Vec2f = Vector2<f32>;
pub type Vec3f = Vector3<f32>;
pub type Vec4f = Vector4<f32>;
impl_vec_embed_serialize!(Vec2f);
impl_vec_embed_serialize!(Vec3f);
impl_vec_embed_serialize!(Vec4f);

macro_rules! impl_ptr_serialize {
    ( $x:ident,$y:ident ) => {
        impl Serializable for $x<dyn $y> {
            fn is_multi_line(&self) -> bool { true }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { self.as_ref().get_type_uuid() }
            fn serialize_binary(&self, io: &mut dyn Write) {
                self.as_ref().get_type_uuid().unwrap().serialize_binary(io);
                self.as_ref().serialize_binary(io);
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                let mut uuid: MaybeUninit<Uuid> = MaybeUninit::uninit();
                let uuid_ref = unsafe { uuid.assume_init_mut() };
                uuid_ref.deserialize_binary(io);

                let constructor = unsafe { &(DYN_NEW_REG.get_unchecked().$y) };
                let mut item = (constructor.get(uuid_ref).unwrap())();
                item.as_mut().deserialize_binary(io);
                *self = $x::from(item);
            }

            fn serialize_text(&self, io: &mut crate::reflection::SerializeTextWriter, indent: String) {
                let _ = io.write_all(format!("{}type_uuid : \"{}\"", indent.clone(), self.as_ref().get_type_uuid().unwrap()).as_bytes());
                io.newline();
                let _ = io.write_all(format!("{}value : ", indent.clone()).as_bytes());
                if self.is_multi_line() {
                    io.newline();
                }
                self.as_ref().serialize_text(io, indent.clone() + "  ");
                io.newline();
            }

            fn deserialize_text(&mut self, data: &Yaml) {
                // println!("deserialize dyn array-item {:?}", data);
                let uuid_str = data["type_uuid"].as_str().unwrap();
                let uuid = Uuid::from_str(uuid_str).unwrap();
                let constructor = unsafe { &(DYN_NEW_REG.get_unchecked().$y) };
                let mut item = (constructor.get(&uuid).unwrap())();
                item.as_mut().deserialize_text(&data["value"]);
                *self = $x::from(item);
            }
        }
    }
}
impl_ptr_serialize!(Box, Component);
impl_ptr_serialize!(Rc, Component);
impl_ptr_serialize!(Arc, Component);
macro_rules! impl_vec_ptr_serialize {
    ( $x:ident,$y:ident ) => {
        impl Serializable for Vec<$x<dyn $y>> {
            fn is_multi_line(&self) -> bool { !self.is_empty() }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                (self.len() as i64).serialize_binary(io);
                for v in self.iter() {
                    v.as_ref().get_type_uuid().unwrap().serialize_binary(io);
                    v.as_ref().serialize_binary(io);
                }
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                let mut len: i64 = 0;
                len.deserialize_binary(io);
                let mut uuid: MaybeUninit<Uuid> = MaybeUninit::uninit();
                let uuid_ref = unsafe { uuid.assume_init_mut() };
                let constructor = unsafe { &(DYN_NEW_REG.get_unchecked().$y) };
                self.reserve(len as usize);
                for _i in 0..len {
                    uuid_ref.deserialize_binary(io);
                    let mut item = (constructor.get(uuid_ref).unwrap())();
                    item.as_mut().deserialize_binary(io);
                    let item_ : $x<dyn $y> = $x::from(item);
                    self.push(item_);
                }
            }

            fn serialize_text(&self, io: &mut crate::reflection::SerializeTextWriter, indent: String) {
                if self.is_empty() {
                    let _ = io.write_all("[]".as_bytes());
                }
                else {
                    for item in self.iter() {
                        let _ = io.write_all(format!("{}- array_item :", indent.clone()).as_bytes());
                        io.newline();
                        let _ = io.write_all(format!("{}  type_uuid : \"{}\"", indent.clone(), item.as_ref().get_type_uuid().unwrap()).as_bytes());
                        io.newline();
                        item.as_ref().serialize_text(io, indent.clone() + "  ");
                        io.newline();
                    }
                }
            }

            fn deserialize_text(&mut self, data: &Yaml) {
                // println!("deserialize dyn array-item {:?}", data);
                let constructor = unsafe { &(DYN_NEW_REG.get_unchecked().$y) };
                let arr = data.as_vec().unwrap();
                self.reserve(arr.len());
                for yaml in arr {
                    let uuid_str = yaml["type_uuid"].as_str().unwrap();
                    let uuid = Uuid::from_str(uuid_str).unwrap();
                    let mut item = (constructor.get(&uuid).unwrap())();
                    item.as_mut().deserialize_text(yaml);
                    let item_ : $x<dyn $y> = $x::from(item);
                    self.push(item_);
                }
            }
        }
    }
}
impl_vec_ptr_serialize!(Box, Component);
impl_vec_ptr_serialize!(Rc, Component);
impl_vec_ptr_serialize!(Arc, Component);
#[macro_export]
macro_rules! impl_vec_concrete_serialize {
    ( $x:ident,$c:ident,$y:ident,$cons:ident,$ref:ident,$mut:ident ) => {
        impl crate::reflection::Serializable for Vec<$x<$c<$y>>> {
            fn is_multi_line(&self) -> bool { !self.is_empty() }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                (self.len() as i64).serialize_binary(io);
                for v in self.iter() {
                    v.$ref().serialize_binary(io);
                }
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                let mut len: i64 = 0;
                len.deserialize_binary(io);
                self.reserve(len as usize);
                for _i in 0..len {
                    let item = $y::$cons();
                    item.$mut().deserialize_binary(io);
                    self.push(item);
                }
            }

            fn serialize_text(&self, io: &mut crate::reflection::SerializeTextWriter, indent: String) {
                if self.is_empty() {
                    let _ = io.write_all("[]".as_bytes());
                }
                else {
                    for item in self.iter() {
                        let _ = io.write_all(format!("{}- array_item :", indent.clone()).as_bytes());
                        io.newline();
                        item.$ref().serialize_text(io, indent.clone() + "  ");
                        io.newline();
                    }
                }
            }

            fn deserialize_text(&mut self, data: &Yaml) {
                let arr = data.as_vec().unwrap();
                self.reserve(arr.len());
                for yaml in arr {
                    // println!("deserialize concrete {:?}", yaml);
                    let item = $y::$cons();
                    item.$mut().deserialize_text(yaml);
                    self.push(item);
                }
            }
        }
    }
}

#[macro_export]
macro_rules! impl_map_ptr_serialize {
    ( $K:ident,$C:ident,$t:ident,$key:ident ) => {
        impl crate::reflection::Serializable for HashMap<$K, $C<dyn $t>> where dyn $t : Serializable {
            fn is_multi_line(&self) -> bool { !self.is_empty() }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                (self.len() as i64).serialize_binary(io);
                for v in self.iter() {
                    v.1.as_ref().get_type_uuid().unwrap().serialize_binary(io);
                    v.1.as_ref().serialize_binary(io);
                }
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                let mut len: i64 = 0;
                len.deserialize_binary(io);
                let mut uuid: MaybeUninit<Uuid> = MaybeUninit::uninit();
                let uuid_ref = unsafe { uuid.assume_init_mut() };
                let constructor = unsafe { &(DYN_NEW_REG.get_unchecked().$t) };
                for _i in 0..len {
                    uuid_ref.deserialize_binary(io);
                    let mut item = (constructor.get(uuid_ref).unwrap())();
                    item.as_mut().deserialize_binary(io);
                    let tt = item.$key();
                    let item_ : $C<dyn $t> = $C::from(item);
                    self.insert(tt, item_);
                }
            }

            fn serialize_text(&self, io: &mut crate::reflection::SerializeTextWriter, indent: String) {
                if self.is_empty() {
                    let _ = io.write_all("[]".as_bytes());
                }
                else {
                    for item in self.iter() {
                        let _ = io.write_all(format!("{}- map_item :", indent.clone()).as_bytes());
                        io.newline();
                        let _ = io.write_all(format!("{}  type_uuid : \"{}\"", indent.clone(), item.1.as_ref().get_type_uuid().unwrap()).as_bytes());
                        io.newline();
                        item.1.as_ref().serialize_text(io, indent.clone() + "  ");
                        io.newline();
                    }
                }
            }

            fn deserialize_text(&mut self, yaml: &Yaml) {
                let constructor = unsafe { &(DYN_NEW_REG.get_unchecked().$t) };
                yaml.as_vec().unwrap().iter().for_each(|e| {
                    // println!("desrialze dyn map item {:?}", e);
                    let uuid_str = e["type_uuid"].as_str().unwrap();
                    let uuid = Uuid::from_str(uuid_str).unwrap();
                    let mut item = (constructor.get(&uuid).unwrap())();
                    item.as_mut().deserialize_text(e);
                    let tt = item.$key();
                    let item_ : $C<dyn $t> = $C::from(item);
                    self.insert(tt, item_);
                });
            }
        }
    }
}
impl_map_ptr_serialize!(TypeId, Box, Component, real_type_id);
impl_map_ptr_serialize!(TypeId, Rc, Component, real_type_id);
impl_map_ptr_serialize!(TypeId, Arc, Component, real_type_id);
#[macro_export]
macro_rules! impl_map_concrete_serialize {
    ( $K:ident,$key:ident,$x:ident,$c:ident,$y:ident,$cons:ident,$ref:ident,$mut:ident ) => {
        impl crate::reflection::Serializable for HashMap<$K, $x<$c<$y>>> {
            fn is_multi_line(&self) -> bool { !self.is_empty() }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                (self.len() as i64).serialize_binary(io);
                for v in self.iter() {
                    v.1.$ref().serialize_binary(io);
                }
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                let mut len: i64 = 0;
                len.deserialize_binary(io);
                self.reserve(len as usize);
                for _i in 0..len {
                    let item = $y::$cons();
                    item.$mut().deserialize_binary(io);
                    let tt = item.$ref().$key();
                    self.insert(tt, item);
                }
            }

            fn serialize_text(&self, io: &mut crate::reflection::SerializeTextWriter, indent: String) {
                if self.is_empty() {
                    let _ = io.write_all("[]".as_bytes());
                }
                else {
                    for item in self.iter() {
                        let _ = io.write_all(format!("{}- array_item :", indent.clone()).as_bytes());
                        io.newline();
                        item.1.$ref().serialize_text(io, indent.clone() + "  ");
                        io.newline();
                    }
                }
            }

            fn deserialize_text(&mut self, data: &Yaml) {
                let arr = data.as_vec().unwrap();
                self.reserve(arr.len());
                for yaml in arr {
                    // println!("deserialize concrete {:?}", yaml);
                    let item = $y::$cons();
                    item.$mut().deserialize_text(yaml);
                    let tt = item.$ref().$key();
                    self.insert(tt, item);
                }
            }
        }
    }
}
// yaml loader
pub(crate) fn load_from_yaml(root: &mut dyn Serializable, data: &String) {
    let docs = YamlLoader::load_from_str(data.as_ref()).unwrap();
    let doc = &docs[0];
    root.deserialize_text(doc);
}
