use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::mem::transmute_copy;
use std::ops::Deref;
use std::pin::Pin;
use std::ptr::addr_of_mut;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use nalgebra::{Dim, Matrix, Matrix4, RawStorage, RawStorageMut, Scalar};
use nalgebra::iter::RowIter;
use once_cell::sync::OnceCell;
use uuid::Uuid;
use yaml_rust2::Yaml::Hash;
use yaml_rust2::{Yaml, YamlLoader};
use crate::comp::transform_component::TransformComponent;
use crate::engine::ENGINE_ROOT;
use crate::entity::{Component, Entity};

#[derive(Debug,Default)]
pub struct ReflectVarInfo
{
    pub display : Option<&'static str>,
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
        io.write(&d);
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        let mut d: [u8; 1] = [0];
        io.read(&mut d);
        unsafe { *addr_of_mut!(*self) = (d[0] == 0); };
    }

    fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
        io.write(self.to_string().as_bytes());
    }

    fn deserialize_yaml(&mut self, yaml: &Yaml) {
        match yaml.as_bool() {
            Some(v) => unsafe { *self = v; },
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
                io.write(self.to_le_bytes().as_ref());
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                unsafe {
                    let mut bytes = self.to_le_bytes();
                    let me = bytes.as_mut();
                    io.read(me);
                    unsafe { *addr_of_mut!(*self) = <$x>::from_le_bytes(bytes); };
                };
            }

            fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
                io.write(self.to_string().as_bytes());
            }

            fn deserialize_yaml(&mut self, yaml: &Yaml) {
                match yaml.$yamlconv() {
                    Some(v) => unsafe { *self = v as $x; },
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
        todo!()
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        todo!()
    }

    fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
        io.write("[ ".as_bytes());
        for col in self.column_iter() {
            for e in col.iter() {
                io.write(e.to_string().as_bytes());
                io.write(", ".as_bytes());
            }
        }
        io.write("]\n".as_bytes());
    }

    fn deserialize_yaml(&mut self, yaml: &Yaml) {
        let mut it = self.iter_mut();
        for v in yaml.as_vec().unwrap() {
            it.next().unwrap().deserialize_yaml(v);
        }
    }
}
impl Serializable for String {
    fn is_multi_line(&self) -> bool { false }
    fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
    fn serialize_binary(&self, io: &mut dyn Write) {
        todo!()
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        todo!()
    }

    fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
        io.write(format!("\"{}\"", self).as_bytes());
    }

    fn deserialize_yaml(&mut self, yaml: &Yaml) {
        *self = yaml.as_str().unwrap().to_string();
    }
}
impl Serializable for Uuid {
    fn is_multi_line(&self) -> bool { false }
    fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
    fn serialize_binary(&self, io: &mut dyn Write) {
        todo!()
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        todo!()
    }

    fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
        io.write(format!("\"{}\"", self.to_string()).as_bytes());
    }

    fn deserialize_yaml(&mut self, yaml: &Yaml) {
        *self = Uuid::from_str(yaml.as_str().unwrap()).unwrap();
    }
}
struct DynComponentStruct {
    Box : fn()->Box<dyn Component>,
    Rc  : fn()->Rc<dyn Component>,
    Arc : fn()->Arc<dyn Component>,
}
#[derive(Default)]
struct DynNewReg {
    Component: HashMap<Uuid, DynComponentStruct>,
}
static mut DYN_NEW_REG : OnceCell<DynNewReg> = OnceCell::new();
#[macro_export]
macro_rules! register_serializable_type {
    ( $x:ident,$y:ident ) => {
        $x.insert($y::type_uuid().unwrap(), DynComponentStruct{
            Box: $y::dyn_box,
            Rc: $y::dyn_rc,
            Arc: $y::dyn_arc,
        });
    }
}
#[macro_export]
macro_rules! impl_serializable_dyn_type {
    ( $x:ident,$y:ident ) => {
        impl $x {
            pub fn dyn_box() -> Box<dyn $y> { Box::new($x::default()) }
            pub fn dyn_rc() -> Rc<dyn $y> { Rc::new($x::default()) }
            pub fn dyn_arc() -> Arc<dyn $y> { Arc::new($x::default()) }
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
pub type FARPROC = unsafe extern "system" fn() -> isize;
pub fn cast_to_function<F>(address: FARPROC, _fn: &F) -> F {
    unsafe { transmute_copy(&address) }
}
#[macro_export]
macro_rules! impl_vec_ptr_serialize {
    ( $x:ident,$y:ident ) => {
        impl Serializable for Vec<$x<dyn $y>> {
            fn is_multi_line(&self) -> bool { !self.is_empty() }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                todo!()
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                todo!()
            }

            fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
                if self.is_empty() {
                    io.write("[]".as_bytes());
                }
                else {
                    for item in self.iter() {
                        io.write(format!("{}- array_item : \n", indent.clone()).as_bytes());
                        item.serialize_yaml(io, indent.clone() + "  ");
                    }
                }
            }

            fn deserialize_yaml(&mut self, yaml: &Yaml) {
                let item = unsafe { (&DYN_NEW_REG.get_unchecked().$y.get(&Uuid::new_v4()).unwrap().$x)() };
                self.push(item);
            }
        }
    }
}
impl_vec_ptr_serialize!(Box, Component);
impl_vec_ptr_serialize!(Rc, Component);
impl_vec_ptr_serialize!(Arc, Component);
#[macro_export]
macro_rules! impl_map_ptr_serialize {
    ( $x:ident,$y:ident ) => {
        impl<V> Serializable for HashMap<$x, $y<V>> where V : Serializable + ?Sized {
            fn is_multi_line(&self) -> bool { !self.is_empty() }
            fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                todo!()
            }

            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                todo!()
            }

            fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
                if self.is_empty() {
                    io.write("[]".as_bytes());
                }
                else {
                    for item in self.iter() {
                        io.write(format!("{}- map_item : \n", indent.clone()).as_bytes());
                        item.1.serialize_yaml(io, indent.clone() + "  ");
                    }
                }
            }

            fn deserialize_yaml(&mut self, yaml: &Yaml) {
                todo!()
            }
        }
    }
}
impl_map_ptr_serialize!(TypeId, Box);
impl_map_ptr_serialize!(TypeId, Rc);
impl_map_ptr_serialize!(TypeId, Arc);
impl<V> Serializable for HashMap<i64, Pin<Rc<RefCell<V>>>> where V : Serializable + ?Sized {
    fn is_multi_line(&self) -> bool { !self.is_empty() }
    fn get_type_uuid(&self) -> Option<uuid::Uuid> { None }
    fn serialize_binary(&self, io: &mut dyn Write) {
        todo!()
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        todo!()
    }

    fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
        if self.is_empty() {
            io.write("[]".as_bytes());
        }
        else {
            for item in self.iter() {
                io.write(format!("{}- map_item : \n", indent.clone()).as_bytes());
                item.1.borrow().serialize_yaml(io, indent.clone() + "  ");
            }
        }
    }

    fn deserialize_yaml(&mut self, yaml: &Yaml) {
        todo!()
    }
}

// yaml loader
pub fn load_from_yaml(root: &mut dyn Serializable, data: &String) {
    let docs = YamlLoader::load_from_str(data.as_ref()).unwrap();
    let yaml = &docs[0];
    let base = &yaml["base"];
    let readonly = base["readonly"].as_bool().unwrap();

    let children = yaml["children"]["value"].as_vec().unwrap();
    //let components = yaml["components"]["value"].as_vec().unwrap();

    //let xx = &base["xx"].as_bool().unwrap();


}

// binary loader


