use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::ops::Deref;
use std::pin::Pin;
use std::ptr::addr_of_mut;
use std::rc::Rc;
use std::sync::Arc;
use nalgebra::Matrix4;
use uuid::Uuid;

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
    fn serialize_binary(&self, io: &mut dyn Write);
    fn deserialize_binary(&mut self, io: &mut dyn Read);
    fn serialize_yaml(&self, io: &mut dyn Write, indent: String);
    fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String);
}
pub trait Uniq {
    fn is_uniq() -> bool;
}
pub trait Boxed<V> {
    fn boxed() -> Box<V>;
}
pub trait Pinned<V> {
    fn pinned() -> Pin<Rc<RefCell<V>>>;
}
#[macro_export]
macro_rules! impl_primitive_serialize {
    ( $x:ty ) => {
        impl Serializable for $x {
            fn is_multi_line(&self) -> bool { false }

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
    fn is_multi_line(&self) -> bool { false }

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
    fn is_multi_line(&self) -> bool { false }

    fn serialize_binary(&self, io: &mut dyn Write) {
        todo!()
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        todo!()
    }

    fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
        io.write(format!("\"{}\"", self).as_bytes());
    }

    fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String) {
        todo!()
    }
}
impl Serializable for Uuid {
    fn is_multi_line(&self) -> bool { false }

    fn serialize_binary(&self, io: &mut dyn Write) {
        todo!()
    }

    fn deserialize_binary(&mut self, io: &mut dyn Read) {
        todo!()
    }

    fn serialize_yaml(&self, io: &mut dyn Write, indent: String) {
        io.write(format!("\"{}\"", self.to_string()).as_bytes());
    }

    fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String) {
        todo!()
    }
}
#[macro_export]
macro_rules! impl_vec_ptr_serialize {
    ( $x:ident ) => {
        impl<V> Serializable for Vec<$x<V>> where V : Serializable + ?Sized {
            fn is_multi_line(&self) -> bool { !self.is_empty() }

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

            fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String) {
                todo!()
            }
        }
    }
}
impl_vec_ptr_serialize!(Box);
impl_vec_ptr_serialize!(Rc);
impl_vec_ptr_serialize!(Arc);
#[macro_export]
macro_rules! impl_map_ptr_serialize {
    ( $x:ident,$y:ident ) => {
        impl<V> Serializable for HashMap<$x, $y<V>> where V : Serializable + ?Sized {
            fn is_multi_line(&self) -> bool { !self.is_empty() }

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

            fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String) {
                todo!()
            }
        }
    }
}
impl_map_ptr_serialize!(TypeId, Box);
impl_map_ptr_serialize!(TypeId, Rc);
impl_map_ptr_serialize!(TypeId, Arc);
impl<V> Serializable for HashMap<u64, Pin<Rc<RefCell<V>>>> where V : Serializable + ?Sized {
    fn is_multi_line(&self) -> bool { !self.is_empty() }

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

    fn deserialize_yaml(&mut self, io: &mut dyn Read, indent: String) {
        todo!()
    }
}
