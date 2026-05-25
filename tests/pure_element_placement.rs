use tersse::pure::element_placement::{
    default_child_location, descendant_ids, rectangles_overlap, resolve_absolute_location,
    resolve_overlap_location, ElementBounds, ElementPlacement, ParentSide,
};
use tersse::test_api::{create_text_element, Element, ElementStore};
use tersse::Location;

fn bounds(x: u16, y: u16, width: usize, height: usize) -> ElementBounds {
    ElementBounds {
        x,
        y,
        width,
        height,
    }
}

fn field() -> Element {
    create_text_element(1, "")
}

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
    let mut store = ElementStore::new();
    let root = store.insert(0.0, field());
    let child = store.insert(0.0, field());
    let grandchild = store.insert(0.0, field());

    let placements = vec![
        (0, ElementPlacement::absolute(Location::default())),
        (
            1,
            ElementPlacement::relative_to(root, ParentSide::Bottom, Location::default()),
        ),
        (
            2,
            ElementPlacement::relative_to(child, ParentSide::Bottom, Location::default()),
        ),
    ];
    let ids = descendant_ids(0, &placements);
    assert_eq!(ids, vec![1, 2]);
    let _ = grandchild;
}

#[test]
fn bottom_side_places_child_below_parent() {
    let parent = bounds(0, 2, 5, 1);
    let child = bounds(0, 0, 8, 1);
    let loc = default_child_location(parent, child, ParentSide::Bottom);
    assert_eq!(loc, Location { x: 0, y: 3 });
}

#[test]
fn left_side_places_child_to_the_left() {
    let parent = bounds(10, 2, 5, 1);
    let child = bounds(0, 0, 4, 1);
    let loc = default_child_location(parent, child, ParentSide::Left);
    assert_eq!(loc, Location { x: 6, y: 2 });
}

#[test]
fn absolute_placement_uses_terminal_origin_plus_offset() {
    let placement = ElementPlacement::absolute(Location { x: 1, y: 2 });
    let loc = resolve_absolute_location(&placement, None, bounds(0, 0, 5, 1)).unwrap();
    assert_eq!(loc, Location { x: 1, y: 2 });
}

#[test]
fn relative_placement_uses_parent_bounds() {
    let mut store = ElementStore::new();
    let parent = store.insert(0.0, field());
    let placement = ElementPlacement::relative_to(parent, ParentSide::Bottom, Location::default());
    let loc =
        resolve_absolute_location(&placement, Some(bounds(0, 4, 20, 1)), bounds(0, 0, 80, 12))
            .unwrap();
    assert_eq!(loc, Location { x: 0, y: 5 });
}

#[test]
fn overlap_resolution_pushes_existing_lower_element() {
    let mut store = ElementStore::new();
    let _ = store.insert(0.0, field());
    let _ = store.insert(0.0, field());
    let _ = store.insert(0.0, field());
    let _ = store.insert(0.0, field());

    let candidate = bounds(0, 3, 8, 1);
    let others = vec![(2, bounds(0, 3, 5, 1), None)];
    let (loc, shifts) = resolve_overlap_location(candidate, &others, 3, Some(1));
    assert_eq!(loc, Location { x: 0, y: 3 });
    assert_eq!(shifts, vec![(3, 1)]);
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
