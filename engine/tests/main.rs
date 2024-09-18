use eldra;
use eldra::engine::{*};
use eldra::node::{*};

#[test]
fn test_add() {
    engine_init();
    let parent = Node_new();
    let child = Node_new();
    Node_add_child(parent, child);
    Node_detach_from_parent(child);
    Node_add_child(parent, child);
    Node_destroy(child);
    Node_destroy(parent);
}