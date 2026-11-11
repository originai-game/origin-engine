//! The typed wire readers (src/wire.rs over the generated types): IDL
//! defaults for what is absent, refusals that name the field's path.

use og_core::glam::Vec2;
use og_core::glam::Vec3;
use og_runtime::wire::{InjectedInput, NavGridDesc, NavMeshDesc, PathDesc, SpriteDesc, arg};

#[test]
fn absent_fields_take_the_idl_defaults() {
    let g: NavGridDesc = arg(None, "desc").unwrap();
    assert_eq!((g.width, g.height, g.cell_size), (32, 32, 1.0));
    let m: NavMeshDesc = arg(Some(r#"{"cellSize":0.5}"#), "desc").unwrap();
    assert_eq!(m.cell_size, 0.5);
    assert_eq!(m.max, Vec3::new(16.0, 1.0, 16.0));
    assert_eq!((m.agent_radius, m.tile_size), (0.4, 32));
    let p: PathDesc = arg(Some(r#"{"from":{"x":2},"smooth":null}"#), "desc").unwrap();
    assert_eq!(p.from, Vec3::new(2.0, 0.0, 0.0));
    assert!(p.smooth, "null is absent: smooth's default is true");
}

#[test]
fn wrong_values_are_refused_with_their_path() {
    let e = |text: &str| arg::<NavMeshDesc>(Some(text), "desc").unwrap_err();
    assert_eq!(
        e(r#"{"min":{"x":"1"}}"#),
        r#"desc.min.x: expected a number, got "1""#
    );
    assert_eq!(
        e(r#"{"blocked":[{},{"max":{"y":true}}]}"#),
        "desc.blocked[1].max.y: expected a number, got true"
    );
    assert_eq!(
        e(r#"{"tileSize":3.5}"#),
        "desc.tileSize: expected a whole number from 0 to 4294967295, got 3.5"
    );
    assert_eq!(e("7"), "desc: expected an object, got 7");
    let p = arg::<PathDesc>(Some(r#"{"algorithm":"dijkstra"}"#), "desc").unwrap_err();
    assert_eq!(
        p,
        "desc.algorithm: 'dijkstra' is not a NavAlgorithm: jps, astar"
    );
    let u = arg::<InjectedInput>(Some(r#"{"kind":"mouse"}"#), "event").unwrap_err();
    assert!(
        u.starts_with("event.kind: 'mouse' is not an InjectedInput: key, pointer"),
        "{u}"
    );
}

#[test]
fn a_partial_object_fills_the_rest_from_its_field_default() {
    // SpriteDesc.anchor defaults to 0.5 0.5 and .color to 1 1 1: a partial
    // value keeps the field's default for what it leaves out, not the type's zero.
    let s: SpriteDesc = arg(Some(r#"{"anchor":{"x":0},"color":{"r":0.25}}"#), "desc").unwrap();
    assert_eq!(s.anchor, Vec2::new(0.0, 0.5));
    assert_eq!((s.color.r, s.color.g, s.color.b), (0.25, 1.0, 1.0));
}
