//! A host's system (a TypeScript system body) works on host-declared layouts
//! through the packed-byte calls of `SystemContext`, by `SchemaId`, with the
//! same declared-access checks as the typed calls.

use og_core::{ErrorKind, Schema};
use og_ecs::{SchemaId, SystemDesc, World, WorldDesc};
use std::sync::{Arc, Mutex};

fn schema(json: &str) -> Schema {
    Schema::from_json(json).unwrap()
}

struct Ids {
    hp: SchemaId,
    tag: SchemaId,
    score: SchemaId,
    hit: SchemaId,
    other: SchemaId,
}

fn world() -> (World, Ids) {
    let mut w = World::new(&WorldDesc::default()).unwrap();
    let ids = Ids {
        hp: w
            .register_schema(schema(
                r#"{"name":"Hp","version":1,"fields":[{"name":"hp","type":"f32"},{"name":"dead","type":"bool"}]}"#,
            ))
            .unwrap(),
        tag: w
            .register_schema(schema(r#"{"name":"Tagged","version":1,"fields":[]}"#))
            .unwrap(),
        score: w
            .register_schema(schema(
                r#"{"name":"Score","version":1,"fields":[{"name":"points","type":"u32"}]}"#,
            ))
            .unwrap(),
        hit: w
            .register_schema(schema(
                r#"{"name":"Hit","version":1,"fields":[{"name":"damage","type":"f32"}]}"#,
            ))
            .unwrap(),
        other: w
            .register_schema(schema(
                r#"{"name":"Other","version":1,"fields":[{"name":"x","type":"u32"}]}"#,
            ))
            .unwrap(),
    };
    w.insert_resource_bytes(ids.score, &0u32.to_le_bytes())
        .unwrap();
    w.insert_resource_bytes(ids.other, &7u32.to_le_bytes())
        .unwrap();
    (w, ids)
}

fn hp(v: f32, dead: bool) -> Vec<u8> {
    let mut b = v.to_le_bytes().to_vec();
    b.push(u8::from(dead));
    b
}

#[test]
fn a_system_reads_and_writes_host_layouts_as_bytes() {
    let (mut w, ids) = world();
    let a = w.spawn_bytes(vec![(ids.hp, hp(10.0, false))]).unwrap();
    let b = w
        .spawn_bytes(vec![(ids.hp, hp(3.0, false)), (ids.tag, vec![])])
        .unwrap();
    w.send_bytes(ids.hit, &4.0f32.to_le_bytes()).unwrap();
    let spawned = Arc::new(Mutex::new(None));
    let out = spawned.clone();
    let (hp_id, tag, score, hit) = (ids.hp, ids.tag, ids.score, ids.hit);
    let desc = SystemDesc {
        name: "combat".into(),
        reads: vec!["Hit".into()],
        writes: vec!["Hp".into(), "Tagged".into(), "Score".into(), "Hit".into()],
        queries: vec![],
    };
    w.add_system(desc, move |ctx| {
        let hits = ctx.events_bytes(hit)?;
        let damage: f32 = hits
            .iter()
            .map(|h| f32::from_le_bytes(h[..4].try_into().unwrap()))
            .sum();
        for e in [a, b] {
            let mut v = ctx.get_bytes(e, hp_id)?;
            let left = f32::from_le_bytes(v[..4].try_into().unwrap()) - damage;
            v[..4].copy_from_slice(&left.to_le_bytes());
            v[4] = u8::from(left <= 0.0);
            ctx.set_bytes(e, hp_id, &v)?;
        }
        let mut points = u32::from_le_bytes(ctx.resource_bytes(score)?.try_into().unwrap());
        points += hits.len() as u32;
        ctx.set_resource_bytes(score, &points.to_le_bytes())?;
        if ctx.time().tick == 0 {
            ctx.remove_id(b, tag)?;
            ctx.insert_bytes(a, tag, &[])?;
            *out.lock().unwrap() = Some(ctx.spawn_bytes(vec![(hp_id, hp(1.0, false))])?);
            ctx.send_bytes(hit, &1.5f32.to_le_bytes())?;
        }
        Ok(())
    })
    .unwrap();

    let before = w.structural_changes();
    w.step().unwrap();
    assert_eq!(w.get_bytes(a, ids.hp).unwrap(), hp(6.0, false));
    assert_eq!(w.get_bytes(b, ids.hp).unwrap(), hp(-1.0, true));
    assert_eq!(w.resource_bytes(ids.score).unwrap(), 1u32.to_le_bytes());
    assert!(w.components_of(a).unwrap().contains(&ids.tag));
    assert!(!w.components_of(b).unwrap().contains(&ids.tag));
    let c = spawned.lock().unwrap().unwrap();
    assert_eq!(w.get_bytes(c, ids.hp).unwrap(), hp(1.0, false));
    assert_eq!(
        w.pending_events_bytes(ids.hit).unwrap(),
        [1.5f32.to_le_bytes().to_vec()]
    );
    assert_eq!(w.structural_changes() - before, 3, "remove, insert, spawn");

    // Next tick the system sees its own event.
    w.step().unwrap();
    assert_eq!(w.get_bytes(a, ids.hp).unwrap(), hp(4.5, false));
}

#[test]
fn undeclared_byte_access_is_refused_and_named() {
    let (mut w, ids) = world();
    let e = w.spawn_bytes(vec![(ids.hp, hp(1.0, false))]).unwrap();
    w.send_bytes(ids.hit, &1.0f32.to_le_bytes()).unwrap();
    let errors = Arc::new(Mutex::new(Vec::new()));
    let out = errors.clone();
    let (hp_id, tag, score, hit, other) = (ids.hp, ids.tag, ids.score, ids.hit, ids.other);
    // Reads Hp and Score only; writes nothing.
    let desc = SystemDesc {
        name: "reader".into(),
        reads: vec!["Hp".into(), "Score".into()],
        ..SystemDesc::default()
    };
    w.add_system(desc, move |ctx| {
        let mut errs = Vec::new();
        ctx.get_bytes(e, hp_id)?; // declared
        ctx.resource_bytes(score)?; // declared
        errs.push(ctx.set_bytes(e, hp_id, &hp(2.0, false)).unwrap_err());
        errs.push(ctx.insert_bytes(e, tag, &[]).unwrap_err());
        errs.push(ctx.remove_id(e, hp_id).unwrap_err());
        errs.push(ctx.spawn_bytes(vec![(hp_id, hp(1.0, false))]).unwrap_err());
        errs.push(ctx.resource_bytes(other).unwrap_err());
        errs.push(
            ctx.set_resource_bytes(score, &1u32.to_le_bytes())
                .unwrap_err(),
        );
        errs.push(ctx.events_bytes(hit).unwrap_err());
        errs.push(ctx.send_bytes(hit, &1.0f32.to_le_bytes()).unwrap_err());
        out.lock().unwrap().extend(errs);
        Ok(())
    })
    .unwrap();
    w.step().unwrap();
    let errors = errors.lock().unwrap();
    let expected = [
        "write access to 'Hp'",
        "write access to 'Tagged'",
        "write access to 'Hp'",
        "write access to 'Hp'",
        "read access to 'Other'",
        "write access to 'Score'",
        "read access to 'Hit'",
        "write access to 'Hit'",
    ];
    assert_eq!(errors.len(), expected.len());
    for (err, needle) in errors.iter().zip(expected) {
        assert_eq!(err.kind(), ErrorKind::Access, "{err}");
        assert!(
            err.reason().contains("system 'reader' did not declare"),
            "{err}"
        );
        assert!(err.reason().contains(needle), "{err} lacks {needle}");
    }
    assert_eq!(w.get_bytes(e, ids.hp).unwrap(), hp(1.0, false));
    assert_eq!(w.resource_bytes(ids.score).unwrap(), 0u32.to_le_bytes());
    assert_eq!(w.entity_count(), 1);
}

#[test]
fn byte_calls_check_sizes_and_values() {
    let (mut w, ids) = world();
    let e = w.spawn_bytes(vec![(ids.hp, hp(1.0, false))]).unwrap();
    let errors = Arc::new(Mutex::new(Vec::new()));
    let out = errors.clone();
    let hp_id = ids.hp;
    let score = ids.score;
    w.add_system(
        SystemDesc {
            name: "writer".into(),
            writes: vec!["Hp".into(), "Score".into(), "og.Time".into()],
            ..SystemDesc::default()
        },
        move |_| Ok(()),
    )
    .unwrap_err();
    w.add_system(
        SystemDesc {
            name: "writer".into(),
            writes: vec!["Hp".into(), "Score".into()],
            ..SystemDesc::default()
        },
        move |ctx| {
            let mut v = hp(1.0, false);
            v[4] = 7;
            out.lock()
                .unwrap()
                .push(ctx.set_bytes(e, hp_id, &v).unwrap_err());
            out.lock()
                .unwrap()
                .push(ctx.set_bytes(e, hp_id, &v[..3]).unwrap_err());
            out.lock()
                .unwrap()
                .push(ctx.set_resource_bytes(score, &[1, 2]).unwrap_err());
            out.lock()
                .unwrap()
                .push(ctx.resource_bytes(SchemaId::from_index(999)).unwrap_err());
            Ok(())
        },
    )
    .unwrap();
    w.step().unwrap();
    let errors = errors.lock().unwrap();
    assert!(
        errors[0].reason().contains("Hp.dead: bool byte 7"),
        "{}",
        errors[0]
    );
    assert!(
        errors[1].reason().contains("5 bytes packed, got 3"),
        "{}",
        errors[1]
    );
    assert!(
        errors[2].reason().contains("4 bytes packed, got 2"),
        "{}",
        errors[2]
    );
    assert_eq!(errors[3].kind(), ErrorKind::NotFound);
}
