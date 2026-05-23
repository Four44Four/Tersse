use tersse::pure::element_placement::{
    default_child_location, descendant_ids, rectangles_overlap, ElementBounds, ElementPlacement,
    ParentSide,
};
use tersse::Location;

#[test]
fn right_side_places_child_to_the_right_of_parent() {
    let parent = ElementBounds {
        x: 0,
        y: 5,
        width: 12,
        height: 1,
    };
    let child = ElementBounds {
        x: 0,
        y: 0,
        width: 14,
        height: 1,
    };
    let loc = default_child_location(parent, child, ParentSide::Right);
    assert_eq!(loc, Location { x: 12, y: 5 });
}

#[test]
fn top_side_places_child_above_parent() {
    let parent = ElementBounds {
        x: 2,
        y: 4,
        width: 20,
        height: 2,
    };
    let child = ElementBounds {
        x: 0,
        y: 0,
        width: 20,
        height: 1,
    };
    let loc = default_child_location(parent, child, ParentSide::Top);
    assert_eq!(loc, Location { x: 2, y: 3 });
}

#[test]
fn descendant_ids_collects_nested_children() {
    let placements = vec![
        (
            "a".to_string(),
            ElementPlacement::absolute(Location::default()),
        ),
        (
            "b".to_string(),
            ElementPlacement::relative_to("a", ParentSide::Bottom, Location::default()),
        ),
        (
            "c".to_string(),
            ElementPlacement::relative_to("b", ParentSide::Bottom, Location::default()),
        ),
    ];
    let ids = descendant_ids("a", &placements);
    assert_eq!(ids, vec!["b".to_string(), "c".to_string()]);
}

#[test]
fn rectangles_overlap_detects_shared_cells() {
    let a = ElementBounds {
        x: 0,
        y: 3,
        width: 5,
        height: 1,
    };
    let b = ElementBounds {
        x: 3,
        y: 3,
        width: 5,
        height: 1,
    };
    assert!(rectangles_overlap(a, b));
}
