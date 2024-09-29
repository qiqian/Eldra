use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::ptr::addr_of_mut;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use nalgebra::{Dim, Matrix, RawStorageMut};
use once_cell::sync::OnceCell;
use uuid::Uuid;
use yaml_rust2::{Yaml, YamlLoader};
use std::mem::MaybeUninit;
use crate::comp::transform_component::TransformComponent;
use crate::entity::{Component, Entity};

macro_rules! register_serializable_type {
    ( $x:ident,$y:ident ) => {
        $x.insert($y::type_uuid().unwrap(), $y::dyn_box);
    }
}
macro_rules! impl_serializable_dyn_type {
    ( $x:ident,$y:ident ) => {
        impl $x {
            pub fn dyn_box() -> Box<dyn $y> { Box::new($x::default()) }
        }
    }
}

impl_serializable_dyn_type!(TransformComponent, Component);
pub unsafe fn init_reflection() {
    DYN_NEW_REG.get_or_init (|| { DynNewReg::default() });

    let reg = &mut DYN_NEW_REG.get_mut().unwrap_unchecked().Component;
    register_serializable_type!(reg, TransformComponent);
    // let yy = x.downcast::<dyn Component>();
}

#[derive(Default)]
struct DynNewReg {
    Component: HashMap<Uuid, fn()->Box<dyn Component>>,
}
static mut DYN_NEW_REG : OnceCell<DynNewReg> = OnceCell::new();
#[derive(Debug,Default)]
pub struct ReflectVarInfo
{
    pub serialize : bool,
    pub readonly : bool,
    pub offset : u32,
    pub size : u32,
}
pub trait Reflectable {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn real_type_id(&self) -> TypeId;
    fn reflect_info(&self) -> Vec<ReflectVarInfo>;
}
pub trait Serializable {
    fn is_multi_line(&self) -> bool;
    fn get_type_uuid(&self) -> Option<uuid::Uuid>;
    fn serialize_binary(&self, io: &mut dyn Write);
    fn deserialize_binary(&mut self, io: &mut dyn Read);
    fn serialize_yaml(&self, io: &mut dyn Write, indent: String);
    fn deserialize_yaml(&mut self, yaml: &Yaml);
}
pub trait Uniq {
    fn is_uniq() -> bool;
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

    fn serialize_yaml(&self, io: &mut dyn Write, _indent: String) {
        let _ = io.write_all(self.to_string().as_bytes());
    }

    fn deserialize_yaml(&mut self, yaml: &Yaml) {
        match yaml.as_bool() {
            Some(v) => { *self = v; },
            None => {},
        }
    }
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

            fn serialize_yaml(&self, io: &mut dyn Write, _indent: String) {
                let _ = io.write_all(self.to_string().as_bytes());
            }

            fn deserialize_yaml(&mut self, yaml: &Yaml) {
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

    fn serialize_yaml(&self, io: &mut dyn Write, _indent: String) {
        let _ = io.write_all("[ ".as_bytes());
        for col in self.column_iter() {
            for e in col.iter() {
                let _ = io.write_all(e.to_string().as_bytes());
                let _ = io.write_all(", ".as_bytes());
            }
        }
        let _ = io.write_all("]\n".as_bytes());
    }

    fn deserialize_yaml(&mut self, yaml: &Yaml) {
        let mut yiter = yaml.as_vec().unwrap().iter();
        self.iter_mut().for_each(|e| {
            e.deserialize_yaml(yiter.next().unwrap());
        });
    }
}
impl Serializable for String {
    fn is_multi_line(&self) -> bool { false }
    fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
    fn serialize_binary(&self, io: &mut dyn Write) {
        let data = self.as_bytes();
        let _ = io.write_all(&(data.len() as u64).to_le_bytes());
        let _ = io.write_all(data);
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        let mut len_bytes : MaybeUninit<[u8; 8]> = MaybeUninit::uninit();
        unsafe {
            let _ = io.read_exact(len_bytes.assume_init_mut());
            let len = u64::from_le_bytes(len_bytes.assume_init()) as usize;
            let mut str = Vec::with_capacity(len);
            str.set_len(len);
            let _ = io.read_exact(str.as_mut());
            *self = String::from_raw_parts(str.as_mut_ptr(), len, len);
        }
    }

    fn serialize_yaml(&self, io: &mut dyn Write, _indent: String) {
        let _ = io.write_all(format!("\"{}\"", self).as_bytes());
    }

    fn deserialize_yaml(&mut self, yaml: &Yaml) {
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
        let mut bytes: MaybeUninit<[u8; 16]> = MaybeUninit::uninit();
        unsafe { 
            let _ = io.read_exact(bytes.assume_init_mut());
            *self = Uuid::from_bytes(bytes.assume_init());
        }
    }

    fn serialize_yaml(&self, io: &mut dyn Write, _indent: String) {
        let _ = io.write_all(format!("\"{}\"", self.to_string()).as_bytes());
    }

    fn deserialize_yaml(&mut self, yaml: &Yaml) {
        *self = Uuid::from_str(yaml.as_str().unwrap()).unwrap();
    }
}
macro_rules! impl_vec_ptr_serialize {
    ( $x:ident,$y:ident ) => {
        impl Serializable for Vec<$x<dyn $y>> {
            fn is_multi_line(&self) -> bool { !self.is_empty() }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                let _ = io.write_all(&(self.len() as u64).to_le_bytes());
                for v in self.iter() {
                    v.get_type_uuid().unwrap().serialize_binary(io);
                    v.serialize_binary(io);
                }
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                let constructor = unsafe { &(DYN_NEW_REG.get_unchecked().$y) };
                let mut len_bytes : MaybeUninit<[u8; 8]> = MaybeUninit::uninit();
                let mut uuid: MaybeUninit<Uuid> = MaybeUninit::uninit();
                unsafe {
                    let _ = io.read_exact(len_bytes.assume_init_mut());
                    let len = u64::from_le_bytes(len_bytes.assume_init()) as usize;
                    for _i in 0..len {
                        uuid.assume_init_mut().deserialize_binary(io);
                        let mut item = (constructor.get(&uuid.assume_init()).unwrap())();
                        item.deserialize_binary(io);
                        let item_ : $x<dyn $y> = $x::from(item);
                        self.push(item_);
                    }
                }
            }

            fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
                if self.is_empty() {
                    let _ = io.write_all("[]".as_bytes());
                }
                else {
                    for item in self.iter() {
                        let _ = io.write_all(format!("{}- array_item : \n", indent.clone()).as_bytes());
                        let _ = io.write_all(format!("{}  type_uuid : \"{}\"\n", indent.clone(), item.get_type_uuid().unwrap()).as_bytes());
                        item.serialize_yaml(io, indent.clone() + "  ");
                    }
                }
            }

            fn deserialize_yaml(&mut self, data: &Yaml) {
                // println!("deserialize dyn array-item {:?}", data);
                let constructor = unsafe { &(DYN_NEW_REG.get_unchecked().$y) };
                let arr = data.as_vec().unwrap();
                for yaml in arr {
                    let uuid_str = yaml["type_uuid"].as_str().unwrap();
                    let uuid = Uuid::from_str(uuid_str).unwrap();
                    let mut item = (constructor.get(&uuid).unwrap())();
                    item.deserialize_yaml(yaml);
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
macro_rules! impl_vec_concrete_serialize {
    ( $x:ident,$c:ident,$y:ident,$cons:ident,$ref:ident,$mut:ident ) => {
        impl Serializable for Vec<$x<$c<$y>>> {
            fn is_multi_line(&self) -> bool { !self.is_empty() }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                let _ = io.write_all(&(self.len() as u64).to_le_bytes());
                for v in self.iter() {
                    v.$ref().serialize_binary(io);
                }
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                let mut len_bytes : MaybeUninit<[u8; 8]> = MaybeUninit::uninit();
                unsafe {
                    let _ = io.read_exact(len_bytes.assume_init_mut());
                    let len = u64::from_le_bytes(len_bytes.assume_init()) as usize;
                    for _i in 0..len {
                        let item = $y::$cons();
                        item.$mut().deserialize_binary(io);
                        self.push(item);
                    }
                }
            }

            fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
                if self.is_empty() {
                    let _ = io.write_all("[]".as_bytes());
                }
                else {
                    for item in self.iter() {
                        let _ = io.write_all(format!("{}- array_item : \n", indent.clone()).as_bytes());
                        item.$ref().serialize_yaml(io, indent.clone() + "  ");
                    }
                }
            }

            fn deserialize_yaml(&mut self, data: &Yaml) {
                for yaml in data.as_vec().unwrap() {
                    // println!("deserialize concrete {:?}", yaml);
                    let item = $y::$cons();
                    item.$mut().deserialize_yaml(yaml);
                    self.push(item);
                }
            }
        }
    }
}
impl_vec_concrete_serialize!(Rc, RefCell, Entity, new, borrow, borrow_mut);

macro_rules! impl_map_ptr_serialize {
    ( $Key:ident,$C:ident,$t:ident,$key:ident ) => {
        impl Serializable for HashMap<$Key, $C<dyn $t>> where dyn $t : Serializable {
            fn is_multi_line(&self) -> bool { !self.is_empty() }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                let _ = io.write_all(&(self.len() as u64).to_le_bytes());
                for v in self.iter() {
                    v.1.get_type_uuid().unwrap().serialize_binary(io);
                    v.1.serialize_binary(io);
                }
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                let constructor = unsafe { &(DYN_NEW_REG.get_unchecked().$t) };
                let mut len_bytes : MaybeUninit<[u8; 8]> = MaybeUninit::uninit();
                let mut uuid: MaybeUninit<Uuid> = MaybeUninit::uninit();
                unsafe {
                    let _ = io.read_exact(len_bytes.assume_init_mut());
                    let len = u64::from_le_bytes(len_bytes.assume_init()) as usize;
                    for _i in 0..len {
                        uuid.assume_init_mut().deserialize_binary(io);
                        let mut item = (constructor.get(&uuid.assume_init()).unwrap())();
                        item.deserialize_binary(io);
                        let tt = item.$key();
                        let item_ : $C<dyn $t> = $C::from(item);
                        self.insert(tt, item_);
                    }
                }
            }

            fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
                if self.is_empty() {
                    let _ = io.write_all("[]".as_bytes());
                }
                else {
                    for item in self.iter() {
                        let _ = io.write_all(format!("{}- map_item : \n", indent.clone()).as_bytes());
                        let _ = io.write_all(format!("{}  type_uuid : \"{}\"\n", indent.clone(), item.1.get_type_uuid().unwrap()).as_bytes());
                        item.1.serialize_yaml(io, indent.clone() + "  ");
                    }
                }
            }

            fn deserialize_yaml(&mut self, yaml: &Yaml) {
                let constructor = unsafe { &(DYN_NEW_REG.get_unchecked().$t) };
                yaml.as_vec().unwrap().iter().for_each(|e| {
                    // println!("desrialze dyn map item {:?}", e);
                    let uuid_str = e["type_uuid"].as_str().unwrap();
                    let uuid = Uuid::from_str(uuid_str).unwrap();
                    let mut item = (constructor.get(&uuid).unwrap())();
                    item.deserialize_yaml(e);
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
// yaml loader
pub(crate) fn load_from_yaml(root: &mut dyn Serializable, data: &String) {
    let docs = YamlLoader::load_from_str(data.as_ref()).unwrap();
    let doc = &docs[0];
    root.deserialize_yaml(doc);
}

// binary loader


