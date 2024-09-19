use eldra;
use eldra::engine::{*};
use eldra::node::{*};

#[test]
fn test_add() {
    engine_init();
    let parent = Node_new();
    let child = Node_new();
    assert_eq!(Node_add_child(parent, child), true);
    assert_eq!(Node_add_child(parent, child), false);
    //assert_eq!(Node_detach_from_parent(child), true);
    //assert_eq!(Node_detach_from_parent(child), false);
    assert_eq!(Node_remove_child(Node_get_parent(child), child), true);
    assert_eq!(Node_get_parent(child), 0);
    assert_eq!(Node_remove_child(parent, child), false);
    assert_eq!(Node_add_child(parent, child), true);
    Node_destroy(child);
    Node_destroy(parent);
}