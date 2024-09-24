use std::ptr::{addr_of, addr_of_mut};
use eldra;
use eldra::{*};
use eldra::engine::{*};
use eldra::entity::{*};
use eldra::comp::transform_component::{*};
use eldra::reflection::{*};
use std::ffi::{CStr, CString};
use std::fs;
use std::io::{BufReader, Read};
use std::ops::{Deref, DerefMut};
use nalgebra::{*};
use yaml_rust2::{YamlLoader, YamlEmitter};


fn test_entity_create() {
    let parent = Entity_new();
    let parent2 = Entity_new();
    let child = Entity_new();
    assert_eq!(Entity_add_child(parent, child), true);
    assert_eq!(Entity_add_child(parent, child), false);
    assert_eq!(Entity_add_child(parent2, child), false);
    //assert_eq!(Entity_detach_from_parent(child), true);
    //assert_eq!(Entity_detach_from_parent(child), false);
    assert_eq!(Entity_remove_child(Entity_get_parent(child), child), true);
    assert_eq!(Entity_get_parent(child), 0);
    assert_eq!(Entity_remove_child(parent, child), false);
    assert_eq!(Entity_remove_child(parent2, child), false);
    assert_eq!(Entity_add_child(parent, child), true);
    Entity_destroy(child);
    Entity_destroy(parent);
    Entity_destroy(parent2);
}

fn test_transform_component() {
    let c1 = Entity_new();
    let tr1 = Entity_create_transform_component(c1);
    assert_eq!(Entity_create_transform_component(c1), 0);
    assert_eq!(Entity_remove_component(c1, tr1), true);
    assert_ne!(Entity_create_transform_component(c1), 0);
    let c2 = Entity_new();
    let tr2 = Entity_create_transform_component(c2);
    Entity_add_child(c1, c2);

    let _info1 = entity_cast(&c1).unwrap().borrow().reflect_info();
    let _info2 = unsafe {
            let mut comp = decode_component!(tr2).as_deref_mut().unwrap_unchecked();
            comp.as_any_mut().downcast_mut::<TransformComponent>().unwrap_unchecked().reflect_info() };

    let mut t1 = Matrix4::<f32>::default();
    let mut t2 = Matrix4::<f32>::default();
    for _i in 0..3 {
        TransformComponent_scale(tr1, 2., 2., 2.);
        TransformComponent_translate(tr1, 1., 1., 1.);
        // update c2
        Entity_tick(c1, 0.1);
        // check
        t1.append_nonuniform_scaling_mut(&Vector3::new(2., 2., 2.));
        let _ = t1.append_translation(&Vector3::new(1., 1., 1.));
        t2 = t1 * t2;
    }

    // serialize
    {
        let yaml_path = "../../bin/test.yaml";
        Entity_serialize(c1, CString::new(yaml_path).unwrap());
        let yaml_str = fs::read_to_string(yaml_path).unwrap();
        let docs = YamlLoader::load_from_str(yaml_str.as_ref());
        let yaml_obj = docs.unwrap();
        let cnt = yaml_obj.len();
        for d in yaml_obj.iter() {

        }
    }

    Entity_destroy(c1);
}


pub fn cstr_to_str(c_buf: *const i8) -> &'static str {
    unsafe {
        let cstr = CStr::from_ptr(c_buf);
        cstr.to_str().unwrap()
    }
}
#[no_mangle]
extern "C"
fn entity_drop_callback(clz: *const i8, id: u64) {
    let result = cstr_to_str(clz);
    println!("{result} {id} dropped")
}

trait XX {
    fn test(&mut self);
    fn test2(&mut self);
}
impl XX for u32 {
    fn test(&mut self) {
        unsafe { *addr_of_mut!(*self) += 1; };
    }
    fn test2(&mut self) {
        let mut bytes = self.to_le_bytes();
        let mut me = bytes.as_mut();

        let mut b = (8815466 as u32).to_le_bytes();
        let mut b0 = b.as_ref();
        b0.read(me);

        unsafe { *addr_of_mut!(*self) = <u32>::from_le_bytes(bytes); };
    }
}

#[test]
fn main() {
    let mut v = 5 as u32;
    v.test();
    println!("{}", v);
    v.test2();
    println!("{}", v);

    engine_init(entity_drop_callback);
    test_entity_create();
    test_transform_component();
}