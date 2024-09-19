use eldra;
use eldra::engine::{*};
use eldra::entity::{*};

#[test]
fn test_add() {
    engine_init();
    let parent = Entity_new();
    let child = Entity_new();
    assert_eq!(Entity_add_child(parent, child), true);
    assert_eq!(Entity_add_child(parent, child), false);
    //assert_eq!(Entity_detach_from_parent(child), true);
    //assert_eq!(Entity_detach_from_parent(child), false);
    assert_eq!(Entity_remove_child(Entity_get_parent(child), child), true);
    assert_eq!(Entity_get_parent(child), 0);
    assert_eq!(Entity_remove_child(parent, child), false);
    assert_eq!(Entity_add_child(parent, child), true);
    Entity_destroy(child);
    Entity_destroy(parent);
}