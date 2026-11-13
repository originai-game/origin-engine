use super::*;

/// `kept_rows` answers what `rows` decodes, row for row, after each
/// change it can see.
fn kept_rows_decode(world: &World, layout: &Layout, kept: &[(u64, Arc<Value>)]) {
    let fresh = rows(world, Some(layout));
    assert_eq!(kept.len(), fresh.len());
    for ((ke, kv), (fe, fv)) in kept.iter().zip(&fresh) {
        assert_eq!((ke, &**kv), (fe, fv));
    }
}

/// A row's JSON is kept while its bytes stay those of the row at its
/// place in the last read, decoded again when they change, and the kept
/// rows are only the last read's: a row gone, or a new one, never leaves
/// a row with another's JSON (engine s110).
#[test]
fn kept_rows_are_what_decoding_every_row_gives() {
    let decl = og_api::host::RECORDS
        .iter()
        .find(|d| d.schema == "og.WindMotor")
        .expect("og.WindMotor");
    let layout = Layout::from_decl(decl).unwrap();
    let mut world = World::new(&og_ecs::WorldDesc::default()).unwrap();
    let id = world.register_schema(layout.schema()).unwrap();
    let base = layout.encode(None, None).unwrap();
    let radius = |r: f64| {
        layout
            .encode(Some(&base), Some(&serde_json::json!({ "radius": r })))
            .unwrap()
    };
    let a = world.spawn_bytes(vec![(id, radius(1.0))]).unwrap();
    let b = world.spawn_bytes(vec![(id, radius(2.0))]).unwrap();
    world.spawn_bytes(vec![(id, radius(3.0))]).unwrap();
    let first = kept_rows(&world, Some(&layout));
    kept_rows_decode(&world, &layout, &first);
    assert_eq!(first.len(), 3);
    // Unchanged: every row's JSON is the one kept, by the layout's clones too.
    let again = kept_rows(&world, Some(&layout.clone()));
    kept_rows_decode(&world, &layout, &again);
    for (k, (f, g)) in first.iter().zip(&again).enumerate() {
        assert!(Arc::ptr_eq(&f.1, &g.1), "row {k} decoded again");
    }
    // One row changes: it alone is decoded again.
    world.insert_bytes(b, id, &radius(5.0)).unwrap();
    let changed = kept_rows(&world, Some(&layout));
    kept_rows_decode(&world, &layout, &changed);
    assert_eq!(changed[1].1["radius"], serde_json::json!(5.0));
    assert!(!Arc::ptr_eq(&again[1].1, &changed[1].1));
    assert!(Arc::ptr_eq(&again[0].1, &changed[0].1));
    assert!(Arc::ptr_eq(&again[2].1, &changed[2].1));
    // A row gone and a new one: the rows after them move.
    world.despawn(a).unwrap();
    let gone = kept_rows(&world, Some(&layout));
    kept_rows_decode(&world, &layout, &gone);
    world.spawn_bytes(vec![(id, radius(2.0))]).unwrap();
    let new = kept_rows(&world, Some(&layout));
    kept_rows_decode(&world, &layout, &new);
    assert_eq!(new.len(), 3);
    let kept = layout.kept_rows.lock().unwrap().len();
    assert_eq!(kept, 3, "the kept rows are the last read's");
    let again = kept_rows(&world, Some(&layout));
    assert!(Arc::ptr_eq(&new[1].1, &again[1].1));
    for (e, _) in again {
        world.despawn(Entity::from_bits(e)).unwrap();
    }
    assert!(kept_rows(&world, Some(&layout)).is_empty());
    assert!(layout.kept_rows.lock().unwrap().is_empty());
}

/// og.MaterialStyle as its row reads: `v` written over the IDL's
/// defaults, then decoded.
fn material_style(v: Value) -> record::MaterialStyle {
    let decl = og_api::host::RECORDS
        .iter()
        .find(|d| d.schema == "og.MaterialStyle")
        .expect("og.MaterialStyle");
    let layout = Layout::from_decl(decl).unwrap();
    layout
        .read(&layout.encode(None, Some(&v)).unwrap())
        .unwrap()
}

/// A look's MaterialStyle.shadowDarkness reaches the renderer's
/// material style; unset it is 0 (off), so no look moves without it.
#[test]
fn material_style_carries_the_shadow_darkness() {
    let m = material_style(serde_json::json!({ "toon": true, "shadowDarkness": 0.75 }));
    let s = stylise(None, Some(&m), &[], &[]);
    assert_eq!(s.material.shadow_darkness, 0.75);
    assert!(s.material.toon);
    let off = material_style(serde_json::json!({ "toon": true }));
    let off = stylise(None, Some(&off), &[], &[]);
    assert_eq!(off.material.shadow_darkness, 0.0);
}

#[test]
fn the_material_style_s_surface_reaches_the_renderer() {
    let m = |v: Value| stylise(None, Some(&material_style(v)), &[], &[]).material;
    // Written as the variant's index and as its name (a look's).
    let clay = m(serde_json::json!({ "surface": 1, "surfaceStrength": 0.5 }));
    assert_eq!(clay.surface, og_render::MaterialSurface::Clay);
    assert_eq!(clay.surface_strength, 0.5);
    assert!(clay.is_active());
    let paper = m(serde_json::json!({ "surface": "paper", "surfaceScale": 0.3 }));
    assert_eq!(paper.surface, og_render::MaterialSurface::Paper);
    assert_eq!((paper.surface_strength, paper.surface_scale), (1.0, 0.3));
    assert_eq!(
        m(serde_json::json!({ "surface": 3 })).surface,
        og_render::MaterialSurface::Plastic
    );
    // None is no change.
    assert!(!m(serde_json::json!({})).is_active());
}
