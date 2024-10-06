use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::RwLock;
use once_cell::sync::OnceCell;
use uuid::Uuid;
use yaml_rust2::{Yaml, YamlLoader};
use crate::data::material::Material;
use crate::data::render_object::RenderObject;
use crate::data::skeleton::Skeleton;
use crate::data::texture::Texture;
use crate::reflection::{Serializable};

pub mod skeleton;
pub mod material;
pub mod render_object;
pub mod texture;

#[derive(Default)]
pub struct ResourceMgr
{
    pub RenderObject: RwLock<HashMap<String, Rc<RenderObject>>>,
    pub Material: RwLock<HashMap<String, Rc<Material>>>,
    pub Texture: RwLock<HashMap<String, Rc<Texture>>>,
    pub Skeleton: RwLock<HashMap<String, Rc<Skeleton>>>,
}
static mut RESOURCE_MGR : OnceCell<ResourceMgr> = OnceCell::new();
#[inline]
pub fn res_mgr() -> &'static ResourceMgr {
    unsafe { RESOURCE_MGR.get_unchecked() }
}
pub unsafe fn init_resource_mgr() {
    RESOURCE_MGR.get_or_init (|| { ResourceMgr::default() });
}
pub trait ExtSerializable<T> where T : Serializable + Sized {
    fn text_ext() -> &'static str { "yaml" }
    fn deserialize_from_text_file(res: &mut T, respath: &String) {
        let yaml_str = fs::read_to_string(respath).unwrap();
        let docs = YamlLoader::load_from_str(&yaml_str).unwrap();
        let doc = &docs[0];
        res.deserialize_text(doc);
    }
}

#[derive(Default)]
pub struct ExtRes<T> {
    path: String,
    value: Rc<T>,
}
macro_rules! impl_ext_ref {
    ( $t:ident ) => {
        impl Serializable for ExtRes<$t> {
            fn is_multi_line(&self) -> bool { false }
            fn get_type_uuid(&self) -> Option<Uuid> { None }
            fn serialize_binary(&self, io: &mut dyn Write) {
                self.path.serialize_binary(io);
            }
            fn deserialize_binary(&mut self, io: &mut dyn Read) {
                self.path.deserialize_binary(io);
                self.value = self.load_ext_res(true, &res_mgr().$t);
            }
            fn serialize_text(&self, io: &mut crate::reflection::SerializeTextWriter, indent: String) {
                self.path.serialize_text(io, indent.clone());
            }
            fn deserialize_text(&mut self, yaml: &Yaml) {
                self.path.deserialize_text(yaml);
                self.value = self.load_ext_res(false, &res_mgr().$t);
            }
        }
    }
}
impl_ext_ref!(RenderObject);
impl_ext_ref!(Material);
impl_ext_ref!(Texture);
impl_ext_ref!(Skeleton);
impl<T> ExtRes<T> {
    pub fn load_ext_res(&mut self, bin:bool, resmap_rw: &RwLock<HashMap<String, Rc<T>>>) -> Rc<T>
    where T : Default + Serializable + ExtSerializable<T>
    {
        {
            let resmap = resmap_rw.read().unwrap();
            let res_opt = resmap.get(&self.path);
            if res_opt.is_some() {
                return res_opt.unwrap().clone()
            }
        }
        // resource not found, load
        let mut obj = T::default();
        let mut resmap = resmap_rw.write().unwrap();
        // w-locked, try again
        let res_opt = resmap.get(&self.path);
        if res_opt.is_some() {
            return res_opt.unwrap().clone()
        }
        {
            let respath = if bin { self.path.clone() + T::text_ext() } else { self.path.clone() + ".bin" };
            if bin {
                let mut file = BufReader::new(File::open(respath).unwrap());
                obj.deserialize_binary(&mut file);
            } else {
                T::deserialize_from_text_file(&mut obj, &respath);
            }
        }
        let refer: Rc<T> = Rc::from(obj);
        resmap.insert(self.path.clone(), refer.clone());
        refer
    }
}
impl<T> Deref for ExtRes<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value.as_ref()
    }
}
