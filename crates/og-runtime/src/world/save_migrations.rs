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
        target: Target::Block,
        name: "og.ui",
        from: 16,
        to: 17,
        note: "og-ui foundation: a list's typeahead (the letters typed to jump and when)",
        run: Some(ui_upgrade),
    },
    Migration {
        target: Target::Block,
        name: "og.ui",
        from: 17,
        to: 18,
        note: "og-ui foundation: the Rive players documents' elements play (none before)",
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
        name: "og.RigidBody",
        from: 3,
        to: 4,
        note: "0.7.7 and earlier: gained airDrag (the air on bodies); the restore takes it at 0, a body the wind does not move",
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
        run: Some(|e| became_lazy(e, "og.Lighting2D")),
    },
    Migration {
        target: Target::Record,
        name: "og.Light2D",
        from: 1,
        to: 2,
        note: "R11: gained kind, innerAngle, outerAngle, height and layers; dropped from a save without lights, as the record is lazy now",
        run: Some(|e| became_lazy(e, "og.Light2D")),
    },
    Migration {
        target: Target::Record,
        name: "og.Occluder2D",
        from: 1,
        to: 2,
        note: "R11: gained points and layers; dropped from a save without occluders, as the record is lazy now",
        run: Some(|e| became_lazy(e, "og.Occluder2D")),
    },
    Migration {
        target: Target::Record,
        name: "og.Wind",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier, registered in every world: gained turbulence and gustSize (G2, the simulated field); at its defaults (a game that never set a wind) it is dropped, as the record is lazy now",
        run: Some(wind_1_to_2),
    },
    Migration {
        target: Target::Record,
        name: "og.Wind",
        from: 2,
        to: 3,
        note: "the world's wind: strength (the lean per metre) is gone and speed is the air's, so the saved lean becomes the speed that leans as far (strength / 0.05 m/s)",
        run: Some(wind_2_to_3),
    },
    Migration {
        target: Target::Record,
        name: "og.Collider",
        from: 2,
        to: 3,
        note: "one name per meaning: rotation is quaternion",
        run: Some(|e| renamed(e, "og.Collider", 3)),
    },
    Migration {
        target: Target::Record,
        name: "og.Light2D",
        from: 2,
        to: 3,
        note: "one name per meaning: radius is distance, as og.PointLight's",
        run: Some(|e| renamed(e, "og.Light2D", 3)),
    },
    Migration {
        target: Target::Record,
        name: "og.AiAgent",
        from: 1,
        to: 2,
        note: "one name per meaning: thinkEvery is thinkEveryTicks",
        run: Some(|e| renamed(e, "og.AiAgent", 2)),
    },
    Migration {
        target: Target::Record,
        name: "og.Perception",
        from: 1,
        to: 2,
        note: "one name per meaning: occluders is occluderMask",
        run: Some(|e| renamed(e, "og.Perception", 2)),
    },
    Migration {
        target: Target::Record,
        name: "og.PlatformerController",
        from: 1,
        to: 2,
        note: "one name per meaning: moveX is run, impulse is addVelocity, oneWayLayers is oneWayMask",
        run: Some(|e| renamed(e, "og.PlatformerController", 2)),
    },
    Migration {
        target: Target::Record,
        name: "og.Lockstep",
        from: 1,
        to: 2,
        note: "one name per meaning: inputDelay, hashEvery and maxPrediction say Ticks, stallTimeout says Ms",
        run: Some(|e| renamed(e, "og.Lockstep", 2)),
    },
    Migration {
        target: Target::Record,
        name: "og.VirtualCamera",
        from: 1,
        to: 2,
        note: "one name per meaning: collisionLayers is collisionMask",
        run: Some(|e| renamed(e, "og.VirtualCamera", 2)),
    },
    Migration {
        target: Target::Record,
        name: "og.CameraView",
        from: 1,
        to: 2,
        note: "one name per meaning: every is everyTicks",
        run: Some(|e| renamed(e, "og.CameraView", 2)),
    },
    Migration {
        target: Target::Record,
        name: "og.Mirror",
        from: 1,
        to: 2,
        note: "one name per meaning: every is everyTicks",
        run: Some(|e| renamed(e, "og.Mirror", 2)),
    },
    Migration {
        target: Target::Record,
        name: "og.Afterimage",
        from: 1,
        to: 2,
        note: "one name per meaning: interval is intervalTicks",
        run: Some(|e| renamed(e, "og.Afterimage", 2)),
    },
    Migration {
        target: Target::Record,
        name: "og.ReflectionProbe",
        from: 1,
        to: 2,
        note: "one name per meaning: interval is intervalTicks",
        run: Some(|e| renamed(e, "og.ReflectionProbe", 2)),
    },
    Migration {
        target: Target::Record,
        name: "og.Input",
        from: 2,
        to: 3,
        note: "the input API's one word per state: the key and button edges say justPressed and justReleased (pressed0-3, released0-3, buttonsPressed, buttonsReleased)",
        run: Some(|e| renamed(e, "og.Input", 3)),
    },
    Migration {
        target: Target::Record,
        name: "og.Pointers",
        from: 1,
        to: 2,
        note: "the input API's one word per state: pressedMask and releasedMask say justPressedMask and justReleasedMask",
        run: Some(|e| renamed(e, "og.Pointers", 2)),
    },
    Migration {
        target: Target::Record,
        name: "og.Gamepad",
        from: 2,
        to: 3,
        note: "the input API's one word per state: the button edges say justPressed and justReleased (pressed0-7, released0-7)",
        run: Some(|e| renamed(e, "og.Gamepad", 3)),
    },
];

/// A record's fields under the name each had before and the name it has
/// from the record's version `.1` on.
type Rename = (&'static str, u32, &'static [(&'static str, &'static str)]);

/// The fields that took another name with a record's version, their order,
/// types and values as they were.
const RENAMES: &[Rename] = &[
    ("og.Collider", 3, &[("rotation", "quaternion")]),
    ("og.Light2D", 3, &[("radius", "distance")]),
    ("og.AiAgent", 2, &[("thinkEvery", "thinkEveryTicks")]),
    ("og.Perception", 2, &[("occluders", "occluderMask")]),
    (
        "og.PlatformerController",
        2,
        &[
            ("moveX", "run"),
            ("impulse", "addVelocity"),
            ("oneWayLayers", "oneWayMask"),
        ],
    ),
    (
        "og.Lockstep",
        2,
        &[
            ("inputDelay", "inputDelayTicks"),
            ("hashEvery", "hashEveryTicks"),
            ("stallTimeout", "stallTimeoutMs"),
            ("maxPrediction", "maxPredictionTicks"),
        ],
    ),
    (
        "og.VirtualCamera",
        2,
        &[("collisionLayers", "collisionMask")],
    ),
    ("og.CameraView", 2, &[("every", "everyTicks")]),
    ("og.Mirror", 2, &[("every", "everyTicks")]),
    ("og.Afterimage", 2, &[("interval", "intervalTicks")]),
    ("og.ReflectionProbe", 2, &[("interval", "intervalTicks")]),
    (
        "og.Input",
        3,
        &[
            ("buttonsPressed", "buttonsJustPressed"),
            ("buttonsReleased", "buttonsJustReleased"),
            ("pressed0", "justPressed0"),
            ("pressed1", "justPressed1"),
            ("pressed2", "justPressed2"),
            ("pressed3", "justPressed3"),
            ("released0", "justReleased0"),
            ("released1", "justReleased1"),
            ("released2", "justReleased2"),
            ("released3", "justReleased3"),
        ],
    ),
    (
        "og.Pointers",
        2,
        &[
            ("pressedMask", "justPressedMask"),
            ("releasedMask", "justReleasedMask"),
        ],
    ),
    (
        "og.Gamepad",
        3,
        &[
            ("pressed0", "justPressed0"),
            ("pressed1", "justPressed1"),
            ("pressed2", "justPressed2"),
            ("pressed3", "justPressed3"),
            ("pressed4", "justPressed4"),
            ("pressed5", "justPressed5"),
            ("pressed6", "justPressed6"),
            ("pressed7", "justPressed7"),
            ("released0", "justReleased0"),
            ("released1", "justReleased1"),
            ("released2", "justReleased2"),
            ("released3", "justReleased3"),
            ("released4", "justReleased4"),
            ("released5", "justReleased5"),
            ("released6", "justReleased6"),
            ("released7", "justReleased7"),
        ],
    ),
];

/// A record whose fields took other names with its version `to`
/// ([`RENAMES`]): the saved type takes the new names, and every component,
/// resource and event of it keeps its bytes.
fn renamed(edit: &mut SnapshotEdit, name: &str, to: u32) -> Result<(), String> {
    let (_, _, fields) = RENAMES
        .iter()
        .find(|r| r.0 == name && r.1 == to)
        .ok_or_else(|| format!("{name} has no renames at version {to}"))?;
    let Some(mut schema) = edit.schema(name).cloned() else {
        return Ok(());
    };
    schema.version = to;
    for f in &mut schema.fields {
        if let Some((_, new)) = fields.iter().find(|(old, _)| f.name == *old) {
            f.name = (*new).to_string();
        }
    }
    edit.map_type(name, schema, &mut |v| Ok(v.to_vec()))
        .map_err(|e| e.reason().to_string())
}

/// A record that became lazy with its version 1 -> 2 (R11's 2D lighting):
/// dropped where it holds nothing a game set (no component,
/// or the resource at its defaults), so the save restores as a world that
/// never had it; else each saved field kept by name and the new ones at
/// their defaults. Keyed on the version, so a save this build writes keeps
/// a record its game named, whatever it holds.
fn became_lazy(edit: &mut SnapshotEdit, name: &str) -> Result<(), String> {
    let err = |e: og_core::Error| e.reason().to_string();
    let Some(old) = edit.schema(name).cloned() else {
        return Ok(());
    };
    let decl = og_api::host::RECORDS
        .iter()
        .find(|d| d.schema == name)
        .ok_or_else(|| format!("{name} is not an engine record"))?;
    let layout = crate::host::codec::Layout::from_decl(decl)?;
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

/// The f32s a schema's fields hold in `v`, by name.
fn f32s<'a>(schema: &'a og_core::reflect::Schema, v: &'a [u8]) -> Vec<(&'a str, Vec<f32>)> {
    let mut at = 0;
    schema
        .fields
        .iter()
        .map(|f| {
            let n = f.ty.size();
            at += n;
            let (words, _) = v[at - n..at].as_chunks::<4>();
            let values = words.iter().map(|b| f32::from_le_bytes(*b)).collect();
            (f.name.as_str(), values)
        })
        .collect()
}

/// og.Wind 1 -> 2: `turbulence` and `gustSize` at the defaults version 2
/// declared (0.3 and 14 m), after the saved fields. Version 1 was in every
/// world, so one still at that version's defaults is a game that never set
/// a wind: dropped, as the record is lazy now. The fields and defaults are
/// those releases', written here: the record has moved on since.
fn wind_1_to_2(edit: &mut SnapshotEdit) -> Result<(), String> {
    use og_core::reflect::{FieldSchema, FieldType};
    const V1: [(&str, &[f32]); 4] = [
        ("strength", &[0.0]),
        ("direction", &[1.0, 0.0, 0.0]),
        ("speed", &[3.0]),
        ("gust", &[0.6]),
    ];
    let name = "og.Wind";
    let err = |e: og_core::Error| e.reason().to_string();
    let Some(old) = edit.schema(name).cloned() else {
        return Ok(());
    };
    let unused = match edit.resource(name) {
        Some(v) => f32s(&old, v) == V1.map(|(n, d)| (n, d.to_vec())),
        None => edit.components(name).is_empty() && !edit.has_events(name),
    };
    if unused {
        return edit.replace_types(&[name], Vec::new()).map_err(err);
    }
    let mut schema = old;
    schema.version = 2;
    for added in ["turbulence", "gustSize"] {
        schema.fields.push(FieldSchema::new(added, FieldType::F32));
    }
    edit.map_type(name, schema, &mut |v| {
        let mut out = v.to_vec();
        out.extend_from_slice(&0.3f32.to_le_bytes());
        out.extend_from_slice(&14.0f32.to_le_bytes());
        Ok(out)
    })
    .map_err(err)
}

/// What og.Wind's version 2 leaned a copy by per m/s of the wind version 3
/// blows at (`og_render::Wind::LEAN_PER_MPS` when the step was written).
const WIND_LEAN_PER_MPS: f32 = 0.05;

/// og.Wind 2 -> 3: `strength` (the lean per metre of height) is gone;
/// `speed` is the air's speed, which the lean follows. The save keeps what
/// it showed: the speed that leans as far as its strength did, so a wind
/// saved still (strength 0) restores still.
fn wind_2_to_3(edit: &mut SnapshotEdit) -> Result<(), String> {
    let name = "og.Wind";
    let Some(old) = edit.schema(name).cloned() else {
        return Ok(());
    };
    let at = |field: &str| -> Result<std::ops::Range<usize>, String> {
        let i = old
            .fields
            .iter()
            .position(|f| f.name == field)
            .ok_or_else(|| format!("the saved og.Wind has no {field}"))?;
        let start: usize = old.fields[..i].iter().map(|f| f.ty.size()).sum();
        Ok(start..start + old.fields[i].ty.size())
    };
    let (strength, speed) = (at("strength")?, at("speed")?);
    let mut schema = old.clone();
    schema.version = 3;
    schema.fields.retain(|f| f.name != "strength");
    edit.map_type(name, schema, &mut |v| {
        let lean = f32::from_le_bytes(v[strength.clone()].try_into().expect("an f32"));
        let mut out = v.to_vec();
        out[speed.clone()].copy_from_slice(&(lean.max(0.0) / WIND_LEAN_PER_MPS).to_le_bytes());
        out.drain(strength.clone());
        Ok(out)
    })
    .map_err(|e| e.reason().to_string())
}

/// og.VideoPlayer 1 -> 2: `time` from f32 to f64, every other field as it
/// was. A save that holds no player (every world's save before the type
/// became lazy) drops it, so restoring it registers nothing.
fn video_player_1_to_2(edit: &mut SnapshotEdit) -> Result<(), String> {
    use og_core::reflect::FieldType;
    let name = crate::bind::video::PLAYER;
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
            crate::engine::lazy::is_lazy(&s.name)
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
            let layout = crate::host::codec::Layout::from_decl(decl).unwrap();
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
                            crate::host::codec::RecordKind::Resource => {
                                world.insert_resource_bytes(id, &bytes).unwrap();
                            }
                            crate::host::codec::RecordKind::Event => {
                                world.send_bytes(id, &bytes).unwrap();
                            }
                            crate::host::codec::RecordKind::Component => {
                                let entity = world.spawn_bytes(vec![(id, bytes)]).unwrap();
                                assert!(upgrade(&world.snapshot()).unwrap().is_none());
                                world.despawn(entity).unwrap();
                            }
                        }
                    }
                    let saved = world.snapshot();
                    let upgraded = upgrade(&saved).unwrap();
                    if !used && layout.kind != crate::host::codec::RecordKind::Event {
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

    /// A world's save holding og.Wind as version 1 or 2 declared it.
    fn saved_wind(version: u32, strength: f32, speed: f32) -> Vec<u8> {
        use og_core::reflect::{FieldSchema, FieldType, Schema};
        let mut fields = vec![
            FieldSchema::new("strength", FieldType::F32),
            FieldSchema::new("direction", FieldType::Vec3),
            FieldSchema::new("speed", FieldType::F32),
            FieldSchema::new("gust", FieldType::F32),
        ];
        let mut values = vec![strength, 1.0, 0.0, 0.0, speed, 0.6];
        if version == 2 {
            fields.push(FieldSchema::new("turbulence", FieldType::F32));
            fields.push(FieldSchema::new("gustSize", FieldType::F32));
            values.extend([0.5, 20.0]);
        }
        let mut world = og_ecs::World::new(&og_ecs::WorldDesc::default()).unwrap();
        let id = world
            .register_schema(Schema {
                name: "og.Wind".into(),
                version,
                doc: String::new(),
                fields,
            })
            .unwrap();
        let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        world.insert_resource_bytes(id, &bytes).unwrap();
        world.snapshot()
    }

    type WindFields = Vec<(String, Vec<f32>)>;

    /// The saved og.Wind after the engine's steps: its version and its
    /// fields' f32s by name, `None` when the steps dropped it.
    fn upgraded_wind(saved: &[u8]) -> Option<(u32, WindFields)> {
        let (bytes, _) = upgrade(saved).unwrap().expect("a step ran");
        let edit = SnapshotEdit::from_bytes(&bytes).unwrap();
        let schema = edit.schema("og.Wind")?;
        let fields = f32s(schema, edit.resource("og.Wind").unwrap())
            .into_iter()
            .map(|(n, v)| (n.to_string(), v))
            .collect();
        Some((schema.version, fields))
    }

    #[test]
    fn a_saved_wind_keeps_its_lean_as_the_airs_speed() {
        let wind = |speed: f32, turbulence: f32, gust_size: f32| {
            vec![
                ("direction".to_string(), vec![1.0, 0.0, 0.0]),
                ("speed".to_string(), vec![speed]),
                ("gust".to_string(), vec![0.6]),
                ("turbulence".to_string(), vec![turbulence]),
                ("gustSize".to_string(), vec![gust_size]),
            ]
        };
        // Version 1 was in every world: at its defaults, no game set it.
        assert_eq!(upgraded_wind(&saved_wind(1, 0.0, 3.0)), None);
        // A lean of 0.3 per metre is what 6 m/s leans; the gusts' old
        // travel speed (9) is not the air's.
        assert_eq!(
            upgraded_wind(&saved_wind(1, 0.3, 9.0)),
            Some((3, wind(6.0, 0.3, 14.0)))
        );
        assert_eq!(
            upgraded_wind(&saved_wind(2, 0.1, 3.0)),
            Some((3, wind(2.0, 0.5, 20.0)))
        );
        // Version 2 was lazy: a wind a game named stays, still as it was.
        assert_eq!(
            upgraded_wind(&saved_wind(2, 0.0, 3.0)),
            Some((3, wind(0.0, 0.5, 20.0)))
        );
        // What this build registers for og.Wind is what the steps end at.
        let decl = og_api::host::RECORDS
            .iter()
            .find(|d| d.schema == "og.Wind")
            .unwrap();
        let now = crate::host::codec::Layout::from_decl(decl)
            .unwrap()
            .schema();
        let names: Vec<&str> = now.fields.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(
            names,
            ["direction", "speed", "gust", "turbulence", "gustSize"]
        );
    }

    /// Each record of [`RENAMES`] as the build before wrote it (the old
    /// names, the version before) upgrades to the layout this build
    /// registers, so the restore takes it, with its values where they were.
    #[test]
    fn a_renamed_records_save_upgrades_to_this_builds_layout_with_its_values() {
        use crate::host::codec::RecordKind;
        for &(name, to, fields) in RENAMES {
            let decl = og_api::host::RECORDS
                .iter()
                .find(|d| d.schema == name)
                .unwrap_or_else(|| panic!("{name} is not an engine record"));
            let layout = crate::host::codec::Layout::from_decl(decl).unwrap();
            // og-input registers its own records; the runtime the rest.
            let now = match name {
                "og.Input" => <og_input::Input as og_core::Reflect>::schema(),
                "og.Pointers" => <og_input::Pointers as og_core::Reflect>::schema(),
                "og.Gamepad" => <og_input::Gamepad as og_core::Reflect>::schema(),
                _ => crate::engine::versioned(layout.schema()),
            };
            assert_eq!(now.version, to, "{name}: the version this build registers");
            let mut old = now.clone();
            old.version = to - 1;
            for (before, after) in fields {
                let f = old
                    .fields
                    .iter_mut()
                    .find(|f| f.name == *after)
                    .unwrap_or_else(|| panic!("{name} has no field {after}"));
                f.name = (*before).to_string();
            }
            // The record at its declared defaults: bytes every field type takes.
            let bytes = layout.encode(None, None).unwrap();
            let mut world = og_ecs::World::new(&og_ecs::WorldDesc::default()).unwrap();
            let id = world.register_schema(old).unwrap();
            let entity = match layout.kind {
                RecordKind::Resource => {
                    world.insert_resource_bytes(id, &bytes).unwrap();
                    None
                }
                _ => Some(world.spawn_bytes(vec![(id, bytes.clone())]).unwrap()),
            };
            let (upgraded, steps) = upgrade(&world.snapshot())
                .unwrap()
                .unwrap_or_else(|| panic!("{name}: no step ran"));
            assert_eq!(steps.len(), 1, "{name}: {steps:?}");
            let edit = SnapshotEdit::from_bytes(&upgraded).unwrap();
            let saved = edit.schema(name).unwrap();
            assert_eq!(saved.layout_hash(), now.layout_hash(), "{name}: {saved:?}");
            match entity {
                None => assert_eq!(edit.resource(name), Some(&bytes[..]), "{name}"),
                Some(e) => assert_eq!(edit.components(name), vec![(e, bytes)], "{name}"),
            }
            assert!(
                upgrade(&upgraded).unwrap().is_none(),
                "{name}: upgraded twice"
            );
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
