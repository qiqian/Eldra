use eldra;
use eldra::engine::{*};
use eldra::entity::{*};
use eldra::comp::transform_component::{*};

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
    let c2 = Entity_new();
    let tr2 = Entity_create_transform_component(c2);
    Entity_add_child(c1, c2);
    for i in 0..3 {
        TransformComponent_scale(tr1, 2., 2., 2.);
        TransformComponent_translate(tr1, 1., 1., 1.);
        // update c2
        Entity_tick(c1, 0.1, 0);
        // check
    }
}

#[test]
fn main() {
    engine_init();
    test_entity_create();
    test_transform_component();
}