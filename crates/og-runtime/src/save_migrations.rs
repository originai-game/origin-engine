//! Engine-owned save migrations: what an engine upgrade does to a player's
//! save so it restores on the new build instead of being refused or reset.
//!
//! A save is the whole world, so engine state sits in it beside the game's:
//! engine components and resources (og.Transform, og.Gamepad, ...) with
//! their schema versions, and the modules' state blocks (og.ui, physics,
//! ...) with their own byte versions. When a release moves one of them, it
//! registers here, by what moved and the version it moved from, the step
//! that turns the old saved form into the new one. [`upgrade`] runs every
//! step a save needs, in table order (so a block two versions behind takes
//! both of its steps), on every restore and every save a game opens for
//! its own migrations, before the restore checks the save against the
//! world: the check then sees the save as this build writes it.
//!
//! The table is the record of every engine state change since saves
//! existed; `og-api`'s layout test pins each engine
//! record's version, so a record's fields cannot change without a bump,
//! and a bump without a step here leaves old saves to the field-by-name
//! mapping (`migrate`). A record that only gained fields needs no step: a
//! restore takes it with the added fields at their declared defaults
//! (`save::engine_owned`); a field removed, renamed or retyped does.

use og_ecs::SnapshotEdit;

/// What a migration upgrades.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    /// A state block, versioned by the `u32` its bytes start with.
    Block,
    /// A component, resource or event, versioned by its schema version.
    Record,
}

/// A step's edit of the save.
type Step = fn(&mut SnapshotEdit) -> Result<(), String>;

/// An upgraded snapshot and the steps it took, for the log.
type Upgraded = (Vec<u8>, Vec<String>);

/// One engine step: `name` at version `from` becomes version `to`.
pub(crate) struct Migration {
    pub target: Target,
    pub name: &'static str,
    pub from: u32,
    pub to: u32,
    /// The engine release whose saves it upgrades, and what changed.
    pub note: &'static str,
    /// The step; `None` when the restore already takes the old form (a
    /// session record the host's value replaces).
    pub run: Option<Step>,
}

/// Every engine step, oldest first.
pub(crate) const MIGRATIONS: &[Migration] = &[
    Migration {
        target: Target::Block,
        name: "og.ui",
        from: 9,
        to: 11,
        note: "0.7.7 and earlier: hit transforms (U1), motion channels (U2), feedback sets, the layer stack (U3)",
        run: Some(ui_9_to_11),
    },
    Migration {
        target: Target::Block,
        name: "og.ui",
        from: 11,
        to: 12,
        note: "U6b: the focus returns as a stack, one per nested scope",
        run: Some(ui_upgrade),
    },
    Migration {
        target: Target::Block,
        name: "og.ui",
        from: 12,
        to: 13,
        note: "U6d: each hit's focusable owner (a press on a grid's item focuses the grid)",
        run: Some(ui_upgrade),
    },
    Migration {
        target: Target::Block,
        name: "og.ui",
        from: 13,
        to: 14,
        note: "ui-gaps B1: focus groups, explicit focus moves, tap, double tap and long press recognisers",
        run: Some(ui_upgrade),
    },
    Migration {
        target: Target::Block,
        name: "og.ui",
        from: 14,
        to: 15,
        note: "ui-gaps B2: a pad's virtual cursor, the players' HUD layouts and the edit mode",
        run: Some(ui_upgrade),
    },
    Migration {
        target: Target::Block,
        name: "og.ui",
        from: 15,
        to: 16,
        note: "UI phase 1: the finger each touch control holds (a stick keeps the touch that grabbed it)",
        run: Some(ui_upgrade),
    },
    Migration {
        target: Target::Record,
        name: "og.Gamepad",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: four pads, now eight (G1-2); a session record, so the host's value replaces the saved one",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.AudioSource",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained userBus (G2-5, the bus graph); the restore takes it at 0, the source's own bus",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.AudioBusFx",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained userBus, the equaliser, the echo and the waveshaper (G2-5); the restore takes them at their defaults, all off",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.RigidBody",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained canSleep and sleeping (G2-3, sleep); the restore takes them at true and false, so a body can sleep and is awake",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.Physics",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained sleepTicks and sleepSpeed (G2-3, sleep); the restore takes them at 0 and 0.1, sleep off as before",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.Joint",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained breakForce, breakTorque and broken (G2-3, breaking joints); the restore takes them at 0, 0 and false, a joint that never breaks",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.Vehicle",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained model, handbrake, handbrakeGrip, driftGrip and the drivetrain's fields (G2-3); the restore takes their defaults: model VEHICLE_ARCADE, the car it was",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.VideoPlayer",
        from: 1,
        to: 2,
        note: "G3-2: time became an f64 (an f32 sum fell behind the sound over a long stream); a save without a player drops the type, which is lazy now",
        run: Some(video_player_1_to_2),
    },
    Migration {
        target: Target::Record,
        name: "og.Lighting2D",
        from: 1,
        to: 2,
        note: "R11: gained shadowAngles; at its defaults (a game that never lit in 2D) it is dropped, as the record is lazy now",
        run: Some(|e| lazy_2d(e, "og.Lighting2D")),
    },
    Migration {
        target: Target::Record,
        name: "og.Light2D",
        from: 1,
        to: 2,
        note: "R11: gained kind, innerAngle, outerAngle, height and layers; dropped from a save without lights, as the record is lazy now",
        run: Some(|e| lazy_2d(e, "og.Light2D")),
    },
    Migration {
        target: Target::Record,
        name: "og.Occluder2D",
        from: 1,
        to: 2,
        note: "R11: gained points and layers; dropped from a save without occluders, as the record is lazy now",
        run: Some(|e| lazy_2d(e, "og.Occluder2D")),
    },
];

/// R11's 2D lighting records, version 1 -> 2: dropped where they hold
/// nothing a game set (no component, or the resource at its version 1
/// defaults), so the save restores as a world that never had them; else
/// each saved field kept by name and the new ones at their defaults.
fn lazy_2d(edit: &mut SnapshotEdit, name: &str) -> Result<(), String> {
    let err = |e: og_core::Error| e.reason().to_string();
    let Some(old) = edit.schema(name).cloned() else {
        return Ok(());
    };
    let decl = og_api::host::RECORDS
        .iter()
        .find(|d| d.schema == name)
        .ok_or_else(|| format!("{name} is not an engine record"))?;
    let layout = crate::codec::Layout::from_decl(decl)?;
    let defaults = layout.encode(None, None)?;
    let mut schema = layout.schema();
    schema.version = 2;
    // Each new field's default bytes, by name.
    let mut at = 0;
    let new: Vec<(String, Vec<u8>)> = schema
        .fields
        .iter()
        .map(|f| {
            let n = f.ty.size();
            at += n;
            (f.name.clone(), defaults[at - n..at].to_vec())
        })
        .collect();
    // The saved fields' byte ranges, by name, where the type is unchanged.
    let mut at = 0;
    let saved: Vec<(String, std::ops::Range<usize>)> = old
        .fields
        .iter()
        .map(|f| {
            let n = f.ty.size();
            at += n;
            let same = schema
                .fields
                .iter()
                .any(|g| g.name == f.name && g.ty == f.ty);
            (
                if same { f.name.clone() } else { String::new() },
                at - n..at,
            )
        })
        .collect();
    let upgrade = |v: &[u8]| -> Vec<u8> {
        let mut out = Vec::with_capacity(defaults.len());
        for (field, default) in &new {
            match saved.iter().find(|(n, _)| n == field) {
                Some((_, r)) => out.extend_from_slice(&v[r.clone()]),
                None => out.extend_from_slice(default),
            }
        }
        out
    };
    let unused = match edit.resource(name) {
        Some(v) => upgrade(v) == defaults,
        None => edit.components(name).is_empty() && !edit.has_events(name),
    };
    if unused {
        return edit.replace_types(&[name], Vec::new()).map_err(err);
    }
    edit.map_type(name, schema, &mut |v| Ok(upgrade(v)))
        .map_err(err)
}

/// og.VideoPlayer 1 -> 2: `time` from f32 to f64, every other field as it
/// was. A save that holds no player (every world's save before the type
/// became lazy) drops it, so restoring it registers nothing.
fn video_player_1_to_2(edit: &mut SnapshotEdit) -> Result<(), String> {
    use og_core::reflect::FieldType;
    let name = crate::video::PLAYER;
    let Some(old) = edit.schema(name).cloned() else {
        return Ok(());
    };
    let err = |e: og_core::Error| e.reason().to_string();
    if edit.components(name).is_empty() && edit.resource(name).is_none() && !edit.has_events(name) {
        return edit.replace_types(&[name], Vec::new()).map_err(err);
    }
    let mut schema = old.clone();
    schema.version = 2;
    for f in &mut schema.fields {
        if f.name == "time" {
            f.ty = FieldType::F64;
        }
    }
    let fields: Vec<(bool, usize)> = old
        .fields
        .iter()
        .map(|f| (f.name == "time" && f.ty == FieldType::F32, f.ty.size()))
        .collect();
    edit.map_type(name, schema, &mut |v| {
        let mut out = Vec::with_capacity(v.len() + 4);
        let mut at = 0;
        for &(time, n) in &fields {
            let b = &v[at..at + n];
            if time {
                let t = f32::from_le_bytes(b.try_into().expect("an f32 is 4 bytes"));
                out.extend_from_slice(&f64::from(t).to_le_bytes());
            } else {
                out.extend_from_slice(b);
            }
            at += n;
        }
        Ok(out)
    })
    .map_err(err)
}

fn ui_9_to_11(edit: &mut SnapshotEdit) -> Result<(), String> {
    let bytes = edit.block("og.ui").ok_or("no og.ui block")?;
    let upgraded = og_ui::Ui::upgrade_state(bytes).map_err(|e| e.reason().to_string())?;
    edit.set_block("og.ui", upgraded)
        .map_err(|e| e.reason().to_string())
}

fn ui_upgrade(edit: &mut SnapshotEdit) -> Result<(), String> {
    let bytes = edit.block("og.ui").ok_or("no og.ui block")?;
    let upgraded = og_ui::Ui::upgrade_state(bytes).map_err(|e| e.reason().to_string())?;
    edit.set_block("og.ui", upgraded)
        .map_err(|e| e.reason().to_string())
}

/// A state block's version: the `u32` its bytes start with.
fn block_version(edit: &SnapshotEdit, name: &str) -> Option<u32> {
    let b = edit.block(name)?;
    Some(u32::from_le_bytes(b.get(..4)?.try_into().ok()?))
}

/// Upgrades a world snapshot (or a selection) taken by an older engine:
/// `Ok(None)` when nothing in it needed a step (the bytes restore as they
/// are), `Ok(Some(bytes))` the upgraded snapshot and the steps it took.
/// A step that fails refuses the save, naming the step. A snapshot this
/// module cannot read is left to the restore, which says why.
pub(crate) fn upgrade(snapshot: &[u8]) -> Result<Option<Upgraded>, String> {
    let Ok(mut edit) = SnapshotEdit::from_bytes(snapshot) else {
        return Ok(None);
    };
    let mut done = Vec::new();
    // Older engines registered some now-lazy records eagerly. An unused
    // schema must not opt the restored world into those modules again.
    // An empty archetype still uses its schemas: a later spawn reuses its
    // position in storage order (for example when a persisted cell reloads).
    // Event declarations activate their producers before the first emit
    // (nav links, AI actions); an empty queue does not make them unused.
    // Saves hold no separate emit-grant marker, so even an old eager event
    // declaration must survive: it is indistinguishable from activation.
    let unused: Vec<String> = edit
        .schemas()
        .iter()
        .filter(|s| {
            crate::lazy::is_lazy(&s.name)
                && !og_api::host::RECORDS
                    .iter()
                    .any(|r| r.schema == s.name && r.kind == "event")
                && !edit.has_archetype(&s.name)
                && edit.resource(&s.name).is_none()
                && !edit.has_events(&s.name)
        })
        .map(|s| s.name.clone())
        .collect();
    for name in unused {
        edit.replace_types(&[&name], Vec::new())
            .map_err(|e| e.reason().to_string())?;
        done.push(format!("{name}: dropped the unused lazy type"));
    }
    for m in MIGRATIONS {
        let at = match m.target {
            Target::Block => block_version(&edit, m.name),
            Target::Record => edit.schema(m.name).map(|s| s.version),
        };
        if at != Some(m.from) {
            continue;
        }
        let Some(run) = m.run else { continue };
        run(&mut edit).map_err(|e| {
            format!(
                "the engine's migration of {} from version {} to {} failed: {e}",
                m.name, m.from, m.to
            )
        })?;
        done.push(format!("{} {} -> {} ({})", m.name, m.from, m.to, m.note));
    }
    Ok((!done.is_empty()).then(|| (edit.to_bytes(), done)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unused_lazy_types_drop_but_saved_state_survives() {
        for name in ["og.Water", "og.Clouds", "og.WorldOrigin", "og.BodySlept"] {
            let decl = og_api::host::RECORDS
                .iter()
                .find(|d| d.schema == name)
                .unwrap();
            let layout = crate::codec::Layout::from_decl(decl).unwrap();
            for legacy in [true, false] {
                for used in [false, true] {
                    let mut schema = layout.schema();
                    if legacy && name == "og.Water" {
                        // R5 added flowMap and the sea fields when Water became lazy,
                        // without changing its schema version.
                        let end = schema
                            .fields
                            .iter()
                            .position(|f| f.name == "flowMap")
                            .unwrap();
                        schema.fields.truncate(end);
                    }
                    let mut world = og_ecs::World::new(&og_ecs::WorldDesc::default()).unwrap();
                    let id = world.register_schema(schema.clone()).unwrap();
                    if used {
                        let bytes: Vec<u8> = schema
                            .fields
                            .iter()
                            .flat_map(|f| vec![0; f.ty.size()])
                            .collect();
                        match layout.kind {
                            crate::codec::RecordKind::Resource => {
                                world.insert_resource_bytes(id, &bytes).unwrap();
                            }
                            crate::codec::RecordKind::Event => {
                                world.send_bytes(id, &bytes).unwrap();
                            }
                            crate::codec::RecordKind::Component => {
                                let entity = world.spawn_bytes(vec![(id, bytes)]).unwrap();
                                assert!(upgrade(&world.snapshot()).unwrap().is_none());
                                world.despawn(entity).unwrap();
                            }
                        }
                    }
                    let saved = world.snapshot();
                    let upgraded = upgrade(&saved).unwrap();
                    if !used && layout.kind != crate::codec::RecordKind::Event {
                        let (bytes, _) = upgraded.expect("the unused eager type is removed");
                        let edit = SnapshotEdit::from_bytes(&bytes).unwrap();
                        assert!(edit.schema(name).is_none());
                        assert!(upgrade(&bytes).unwrap().is_none());
                    } else {
                        assert!(upgraded.is_none(), "legacy={legacy}, used={used}");
                    }
                }
            }
        }
    }

    #[test]
    fn every_step_moves_a_version_forward_and_each_name_chains() {
        for m in MIGRATIONS {
            assert!(m.to > m.from, "{} {} -> {}", m.name, m.from, m.to);
        }
        for (i, m) in MIGRATIONS.iter().enumerate() {
            // A later step of the same name starts where an earlier ended.
            for later in &MIGRATIONS[i + 1..] {
                if later.name == m.name && later.target == m.target {
                    assert!(later.from >= m.to, "{} steps overlap", m.name);
                }
            }
        }
    }
}
