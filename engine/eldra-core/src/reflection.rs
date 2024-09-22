use std::any::{Any, TypeId};
use std::io::{BufReader, BufWriter, Read, Write};

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
pub trait Serializable : Reflectable {
    fn serialize<W>(&self, io: &mut BufWriter<W>) where W: ?Sized + Write;
    fn deserialize<R>(&mut self, io: &mut BufReader<R>) where R: ?Sized + Read;
}