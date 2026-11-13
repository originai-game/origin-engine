use super::*;
use crate::{
    Dispatch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_render_sphere_geometry, og_render_standard_material, og_world_despawn, og_world_insert,
    og_world_remove, og_world_set, og_world_spawn, set_hooks,
};
use serde_json::json;

struct Quiet;
impl Hooks for Quiet {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

/// The tick's raycast record, each transform's bits, against the world
/// read afresh; and how many world matrices composing it took. The
/// tick itself composes none: only a raycast from a system needs them.
fn checked(w: u32, label: &str) -> usize {
    let eng = engine(w).unwrap();
    let mut rs = eng.render.borrow_mut();
    assert_eq!(
        rs.geo.hierarchy.composed, 0,
        "{label}: the tick composed world matrices no raycast asked for"
    );
    assert!(
        rs.geo.hierarchy.pending(),
        "{label}: the tick brought the hierarchy to the world before a raycast asked"
    );
    let GeoState {
        record, hierarchy, ..
    } = &mut rs.geo;
    let list = record.as_mut().expect("a record").1.targets(hierarchy);
    let mut fresh = Hierarchy::of(eng.world().unwrap());
    // The rows kept from earlier ticks are those read afresh.
    let mut models = Vec::new();
    let meshes = drawn(
        eng.world().unwrap(),
        &rs,
        &mut Hierarchy::default(),
        &mut Meshes::default(),
        &mut models,
    );
    let rows = |t: &[(u64, usize, bool, u32)]| t.to_vec();
    assert_eq!(
        rows(
            &list
                .iter()
                .map(|t| (t.entity, t.geometry, t.visible, t.layers))
                .collect::<Vec<_>>()
        ),
        rows(
            &meshes
                .iter()
                .chain(&models)
                .map(|d| (d.entity, d.geometry, d.visible, d.layers))
                .collect::<Vec<_>>()
        ),
        "{label}: the record's meshes are not the world's"
    );
    for t in list.iter() {
        let want = fresh.world_of(t.entity).to_cols_array().map(f32::to_bits);
        assert_eq!(
            t.transform.to_cols_array().map(f32::to_bits),
            want,
            "{label}: entity {} kept a stale world matrix",
            t.entity
        );
    }
    rs.geo.hierarchy.composed
}

/// The raycast record's hierarchy read (the probe's before_tick, every
/// tick) composes only the world matrices that moved, and stays bit for
/// bit what a fresh read composes through moves, a moved parent,
/// reparenting, a removed transform and a despawn. Before it was
/// incremental, a tick where nothing moved composed all 350; before it
/// waited for a raycast, every tick composed what moved (`checked`).
#[test]
fn the_raycast_record_composes_only_what_moved() {
    set_hooks(Rc::new(Quiet));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let w = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let geometry = value(crate::host::host_impl::answer(og_render_sphere_geometry(
        w,
        Some("{}".into()),
    )));
    let material = value(crate::host::host_impl::answer(og_render_standard_material(
        w,
        Some("{}".into()),
    )));
    let mesh = json!({ "geometry": geometry, "material": material }).to_string();
    let spawn = |x: f64, parent: Option<f64>| {
        let at = json!({ "position": { "x": x, "y": 0.5, "z": -x * 0.5 } }).to_string();
        let e = value(crate::host::host_impl::answer(og_world_spawn(w, Some(at))))
            .as_f64()
            .unwrap();
        value(crate::host::host_impl::answer(og_world_insert(
            w,
            e,
            "og.Mesh".into(),
            Some(mesh.clone()),
        )));
        if let Some(p) = parent {
            let v = json!({ "entity": p }).to_string();
            value(crate::host::host_impl::answer(og_world_insert(
                w,
                e,
                "og.Parent".into(),
                Some(v),
            )));
        }
        e
    };
    // 200 roots, a child under every other one and a grandchild under
    // every fourth: 350 meshes.
    let mut roots = Vec::new();
    let mut children = Vec::new();
    for i in 0..200 {
        let r = spawn(i as f64 * 1.5, None);
        roots.push(r);
        if i % 2 == 0 {
            let c = spawn(0.25, Some(r));
            children.push(c);
            if i % 4 == 0 {
                spawn(0.125, Some(c));
            }
        }
    }
    let frame = || value(og_engine_frame(w, 1000.0 / 60.0));
    let step = |label: &str| {
        frame();
        checked(w, label)
    };
    assert_eq!(step("first"), 350);
    assert_eq!(step("still"), 0, "nothing moved");
    // A leaf root: one matrix.
    let to = |x: f64| json!({ "position": { "x": x, "y": 1, "z": 2 } }).to_string();
    value(crate::host::host_impl::answer(og_world_set(
        w,
        roots[1],
        "og.Transform".into(),
        to(3.0),
    )));
    assert_eq!(step("one root moved"), 1);
    // A root with a child and a grandchild: three.
    value(crate::host::host_impl::answer(og_world_set(
        w,
        roots[4],
        "og.Transform".into(),
        to(5.0),
    )));
    assert_eq!(step("a parent moved"), 3);
    // A child moved: it and its own child.
    value(crate::host::host_impl::answer(og_world_set(
        w,
        children[0],
        "og.Transform".into(),
        to(0.5),
    )));
    assert_eq!(step("a child moved"), 2);
    // Reparented: everything again, once.
    let v = json!({ "entity": roots[3] }).to_string();
    value(crate::host::host_impl::answer(og_world_insert(
        w,
        children[1],
        "og.Parent".into(),
        Some(v),
    )));
    assert_eq!(step("reparented"), 350);
    assert_eq!(step("still after"), 0);
    // A parent without its transform, then despawned.
    value(crate::host::host_impl::answer(og_world_remove(
        w,
        roots[8],
        "og.Transform".into(),
    )));
    step("transform removed");
    value(crate::host::host_impl::answer(og_world_despawn(
        w, roots[8],
    )));
    step("despawned");
    assert_eq!(step("still at the end"), 0);
    // Ticks with no raycast between them: a root moved a tick before
    // one where nothing moves (which alone reads as still), then moved
    // and moved back.
    value(crate::host::host_impl::answer(og_world_set(
        w,
        roots[1],
        "og.Transform".into(),
        to(9.0),
    )));
    frame();
    assert_eq!(step("moved a tick before a still one"), 1);
    value(crate::host::host_impl::answer(og_world_set(
        w,
        roots[1],
        "og.Transform".into(),
        to(11.0),
    )));
    frame();
    value(crate::host::host_impl::answer(og_world_set(
        w,
        roots[1],
        "og.Transform".into(),
        to(9.0),
    )));
    assert_eq!(step("moved and moved back"), 0);
    // Reparented a tick before a still one: everything again.
    let v = json!({ "entity": roots[6] }).to_string();
    value(crate::host::host_impl::answer(og_world_insert(
        w,
        children[1],
        "og.Parent".into(),
        Some(v),
    )));
    frame();
    assert_eq!(step("reparented a tick before a still one"), 347);
    // A mesh hidden and one given another geometry: the kept rows are
    // read again (`checked` compares them with the world's).
    let hidden = json!({ "visible": false }).to_string();
    value(crate::host::host_impl::answer(og_world_set(
        w,
        roots[2],
        "og.Mesh".into(),
        hidden,
    )));
    let other = value(crate::host::host_impl::answer(og_render_sphere_geometry(
        w,
        Some(r#"{"radius":2}"#.into()),
    )));
    let swapped = json!({ "geometry": other }).to_string();
    value(crate::host::host_impl::answer(og_world_set(
        w,
        roots[5],
        "og.Mesh".into(),
        swapped,
    )));
    step("a mesh hidden, one swapped");
    let eng = engine(w).unwrap();
    let rs = eng.render.borrow();
    let Some((_, Record::Targets(list))) = &rs.geo.record else {
        panic!("composed by checked");
    };
    let of = |e: f64| list.iter().find(|t| t.entity == e as u64).expect("drawn");
    assert!(!of(roots[2]).visible);
    assert_ne!(of(roots[5]).geometry, of(roots[6]).geometry);
}

/// A read where nothing moved since the last one brought to the slots
/// walks no entity and no slot, at a tick's first raycast and at a
/// raycast between ticks (`perf_raycast_record_20k`: the walk was the
/// still read's own work beside comparing the columns). One where
/// something moved, at that read or at a tick with no raycast since,
/// walks them, and the record stays a fresh read (`checked`).
#[test]
fn a_still_read_walks_no_entity() {
    set_hooks(Rc::new(Quiet));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let w = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let geometry = value(crate::host::host_impl::answer(og_render_sphere_geometry(
        w,
        Some("{}".into()),
    )));
    let material = value(crate::host::host_impl::answer(og_render_standard_material(
        w,
        Some("{}".into()),
    )));
    let mesh = json!({ "geometry": geometry, "material": material }).to_string();
    let mut all = Vec::new();
    for i in 0..60 {
        let at = json!({ "position": { "x": i as f64, "y": 0, "z": 0 } }).to_string();
        let e = value(crate::host::host_impl::answer(og_world_spawn(w, Some(at))))
            .as_f64()
            .unwrap();
        value(crate::host::host_impl::answer(og_world_insert(
            w,
            e,
            "og.Mesh".into(),
            Some(mesh.clone()),
        )));
        if i % 3 == 2 {
            let v = json!({ "entity": all[i - 1] }).to_string();
            value(crate::host::host_impl::answer(og_world_insert(
                w,
                e,
                "og.Parent".into(),
                Some(v),
            )));
        }
        all.push(e);
    }
    let frame = || value(og_engine_frame(w, 1000.0 / 60.0));
    let walked = |label: &str| {
        frame();
        checked(w, label);
        engine(w).unwrap().render.borrow().geo.hierarchy.walked
    };
    assert!(walked("first") >= 60);
    assert_eq!(walked("still"), 0, "nothing moved");
    let to = |x: f64| json!({ "position": { "x": x, "y": 1, "z": 2 } }).to_string();
    value(crate::host::host_impl::answer(og_world_set(
        w,
        all[4],
        "og.Transform".into(),
        to(3.0),
    )));
    assert!(walked("one moved") >= 60);
    assert_eq!(walked("still after"), 0);
    // Moved at a tick no raycast read, still at the next.
    value(crate::host::host_impl::answer(og_world_set(
        w,
        all[4],
        "og.Transform".into(),
        to(5.0),
    )));
    frame();
    assert!(walked("moved a tick before a still one") >= 60);
    // Every parented mesh loses its transform: their archetype empties
    // and leaves the query, the others' columns and the parents are
    // unchanged, and they are no longer the slots they were.
    for &e in all.iter().skip(2).step_by(3) {
        value(crate::host::host_impl::answer(og_world_remove(
            w,
            e,
            "og.Transform".into(),
        )));
    }
    assert!(walked("the last archetype emptied") >= 40);
    assert_eq!(walked("still after it"), 0);
    // A raycast between ticks reads the world as it is.
    let ray =
        json!({ "origin": { "x": 0, "y": 10, "z": 0 }, "direction": { "x": 0, "y": -1, "z": 0 } })
            .to_string();
    let between = || {
        crate::host::host_impl::answer(og_render_raycast(w, ray.clone(), None));
        engine(w).unwrap().render.borrow().geo.now.walked
    };
    assert!(between() >= 40);
    assert_eq!(between(), 0, "nothing moved between the raycasts");
    value(crate::host::host_impl::answer(og_world_set(
        w,
        all[7],
        "og.Transform".into(),
        to(1.0),
    )));
    assert!(between() >= 40);
}
