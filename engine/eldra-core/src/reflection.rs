use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::ops::Deref;
use std::pin::Pin;
use std::ptr::addr_of_mut;
use std::rc::Rc;
use nalgebra::Matrix4;

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
    fn serialize_binary(&self, io: &mut dyn Write);
    fn deserialize_binary(&mut self, io: &mut dyn Read);
    fn serialize_yaml(&self, io: &mut dyn Write, indent: String);
    fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String);
}

#[macro_export]
macro_rules! impl_primitive_serialize {
    ( $x:ty ) => {
        impl Serializable for $x {
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

            fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String) {
                todo!()
            }
        }
    };
}
impl_primitive_serialize!(i8);
impl_primitive_serialize!(u8);
impl_primitive_serialize!(i16);
impl_primitive_serialize!(u16);
impl_primitive_serialize!(i32);
impl_primitive_serialize!(u32);
impl_primitive_serialize!(i64);
impl_primitive_serialize!(u64);
impl_primitive_serialize!(i128);
impl_primitive_serialize!(u128);

impl Serializable for Matrix4<f32> {
    fn serialize_binary(&self, io: &mut dyn Write) {
        todo!()
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        todo!()
    }

    fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
        io.write("[ ".as_bytes());
        for row in self.row_iter() {
            for e in row.iter() {
                io.write(e.to_string().as_bytes());
                io.write(", ".as_bytes());
            }
        }
        io.write("]\n".as_bytes());
    }

    fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String) {
        todo!()
    }
}
impl Serializable for String {
    fn serialize_binary(&self, io: &mut dyn Write) {
        todo!()
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        todo!()
    }

    fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
        io.write(self.as_bytes());
    }

    fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String) {
        todo!()
    }
}
impl<V> Serializable for Vec<Box<V>> where V : Serializable + ?Sized {
    fn serialize_binary(&self, io: &mut dyn Write) {
        todo!()
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        todo!()
    }

    fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
        for item in self.iter() {
            io.write(format!("{}- ", indent.clone()).as_bytes());
            item.serialize_yaml(io, indent.clone() + "  ");
        }
    }

    fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String) {
        todo!()
    }
}
impl<V> Serializable for HashMap<TypeId, Box<V>> where V : Serializable + ?Sized {
    fn serialize_binary(&self, io: &mut dyn Write) {
        todo!()
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        todo!()
    }

    fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
        for item in self.iter() {
            io.write(format!("{}- ", indent.clone()).as_bytes());
            item.1.serialize_yaml(io, indent.clone() + "  ");
        }
    }

    fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String) {
        todo!()
    }
}
impl<V> Serializable for HashMap<u64, Pin<Rc<RefCell<V>>>> where V : Serializable + ?Sized {
    fn serialize_binary(&self, io: &mut dyn Write) {
        todo!()
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        todo!()
    }

    fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
        for item in self.iter() {
            io.write(format!("{}- ", indent.clone()).as_bytes());
            item.1.borrow().serialize_yaml(io, indent.clone() + "  ");
        }
    }

    fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String) {
        todo!()
    }
}