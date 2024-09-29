use std::ptr::addr_of;
use eldra;
use eldra::{*};
use eldra::engine::{*};
use eldra::entity::{*};
use eldra::comp::transform_component::{*};
use eldra::reflection::{*};
use std::ffi::{CStr, CString};
use std::fs;
use std::ops::DerefMut;
use std::os::raw::c_char;
use std::rc::Rc;
use nalgebra::{*};
use std::env::current_dir;

fn test_entity_create() {
    let parent = Entity_new();
    let parent2 = Entity_new();
    let child = Entity_new();
    assert_eq!(Entity_add_child(parent, child), true);
    assert_eq!(Entity_add_child(parent, child), false);
    assert_eq!(Entity_add_child(parent2, child), false);
    //assert_eq!(Entity_detach_from_parent(child), true);
    //assert_eq!(Entity_detach_from_parent(child), false);
    assert_eq!(Entity_get_parent(child), parent);
    assert_eq!(Entity_remove_child(Entity_get_parent(child), child), true);
    assert_eq!(Entity_get_parent(child), 0);
    assert_eq!(Entity_remove_child(parent, child), false);
    assert_eq!(Entity_remove_child(parent2, child), false);
    assert_eq!(Entity_add_child(parent, child), true);
    Entity_destroy(child);
    Entity_destroy(parent);
    Entity_destroy(parent2);
}

fn test_transform_component() -> u64 {
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
            let comp = decode_component!(tr2).as_deref_mut().unwrap_unchecked();
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

    c1
}

fn test_serialize(entity_uuid: u64) { 
    // serialize
    let output_path = "../bin/test.yaml";
    let curdir = current_dir().unwrap();
    let yaml_path = curdir.as_path().join(output_path);
    let yaml_path = yaml_path.as_path().to_str().unwrap();
    println!("serialize to {}", yaml_path);
    let output_path_c = convert_c_str(output_path);
    Entity_serialize_yaml(entity_uuid, output_path_c);
    drop_c_str(output_path_c);
    // deserialize
    let yaml_str = fs::read_to_string(yaml_path).unwrap();
    let e = Entity::pinned();
    load_from_yaml(e.borrow_mut().deref_mut(), &yaml_str);
    let entity = e.borrow();
    let c = entity.children.first().unwrap();
    println!("deserialize done {}/{}", Rc::strong_count(c), Rc::weak_count(c));
}
pub fn cstr_to_str(c_buf: *const c_char) -> &'static str {
    unsafe {
        let cstr = CStr::from_ptr(c_buf);
        cstr.to_str().unwrap()
    }
}
#[no_mangle]
extern "C"
fn entity_drop_callback(clz: *const c_char, id: *const c_char) {
    println!("{} {} dropped", cstr_to_str(clz), cstr_to_str(id))
}

pub(crate) fn convert_c_str(input: &str) -> *mut c_char {
    let c_str = CString::new(input).unwrap().into_raw();
    return c_str;
}
pub(crate) fn drop_c_str(c_str: *mut c_char) {
    drop(unsafe { CString::from_raw(c_str) });
}

#[test]
fn main() {
    engine_init(entity_drop_callback);

    println!("test entity");
    test_entity_create();

    println!("test transform");
    let entity = test_transform_component();

    println!("test serialize");
    test_serialize(entity);

    println!("test cleanup");
    Entity_destroy(entity);
}