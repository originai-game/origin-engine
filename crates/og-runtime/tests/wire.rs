//! The typed wire readers (src/wire.rs over the generated types): IDL
//! defaults for what is absent, refusals that name the field's path.

use og_core::glam::Vec2;
use og_core::glam::Vec3;
use og_runtime::wire::{
    InjectedInput, NavGridDesc, NavMeshDesc, OneOf, PathDesc, ShapeGeometryDesc, SpriteDesc,
    TweenDesc, arg,
};

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

#[test]
fn one_of_reads_the_alternative_the_value_has_the_shape_of() {
    use OneOf::{A, B};
    // og.Shape crosses as `{ shape, holes }`, which is no Vector2.
    let d: ShapeGeometryDesc = arg(
        Some(r#"{"shapes":[{"shape":[{"x":1,"y":2}],"holes":[]}]}"#),
        "desc",
    )
    .unwrap();
    match d.shapes {
        Some(B(B(B(list)))) => assert_eq!(list[0].shape, [Vec2::new(1.0, 2.0)]),
        other => panic!("read as {other:?}"),
    }
    let d: ShapeGeometryDesc = arg(Some(r#"{"shapes":[{"x":1,"y":2}]}"#), "desc").unwrap();
    assert!(matches!(d.shapes, Some(B(B(A(_))))), "{:?}", d.shapes);
    // A value of one alternative's shape is refused inside it.
    let e = arg::<ShapeGeometryDesc>(Some(r#"{"shapes":[{"x":"1"}]}"#), "desc").unwrap_err();
    assert_eq!(e, r#"desc.shapes[0].x: expected a number, got "1""#);
    // A lane value: a bare number or a list.
    let t: TweenDesc = arg(Some(r#"{"to":2,"from":[1,2]}"#), "desc").unwrap();
    assert_eq!((t.to, t.from), (Some(A(2.0)), Some(B(vec![1.0, 2.0]))));
    let e = arg::<TweenDesc>(Some(r#"{"to":[1,"a"]}"#), "desc").unwrap_err();
    assert_eq!(e, r#"desc.to[1]: expected a number, got "a""#);
    let e = arg::<TweenDesc>(Some(r#"{"to":"a"}"#), "desc").unwrap_err();
    assert_eq!(
        e,
        r#"desc.to: expected a number, got "a"; or expected a list, got "a""#
    );
}

#[test]
fn a_system_desc_the_idl_refuses_refuses_the_engine() {
    use og_runtime::{og_engine_create, og_game_game, og_host_begin_bundle};
    og_host_begin_bundle();
    // Read by hand, a `when` the IDL does not name ran as `during`.
    og_runtime::og_system_define_system("late".into(), r#"{"when":"after"}"#.into(), 0);
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let answer = og_engine_create(game, r#"{"profile":"3d"}"#.into());
    assert!(
        answer.contains("og.system 'late': desc.when: 'after' is not a StateWhen"),
        "{answer}"
    );
}
