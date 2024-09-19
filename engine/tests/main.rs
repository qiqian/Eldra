use eldra;
use eldra::engine::{*};
use eldra::entity::{*};

#[test]
fn test_add() {
    engine_init();
    let parent = _Entity_new();
    let parent2 = _Entity_new();
    let child = _Entity_new();
    assert_eq!(_Entity_add_child(parent, child), true);
    assert_eq!(_Entity_add_child(parent, child), false);
    assert_eq!(_Entity_add_child(parent2, child), false);
    //assert_eq!(Entity_detach_from_parent(child), true);
    //assert_eq!(Entity_detach_from_parent(child), false);
    assert_eq!(_Entity_remove_child(_Entity_get_parent(child), child), true);
    assert_eq!(_Entity_get_parent(child), 0);
    assert_eq!(_Entity_remove_child(parent, child), false);
    assert_eq!(_Entity_remove_child(parent2, child), false);
    assert_eq!(_Entity_add_child(parent, child), true);
    _Entity_destroy(child);
    _Entity_destroy(parent);
    _Entity_destroy(parent2);
}