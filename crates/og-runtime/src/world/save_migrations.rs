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
    Migration {
        target: Target::Record,
        name: "og.Rope",
        from: 1,
        to: 2,
        note: "damping is a rate: the share kept per tick becomes the decay per second that keeps as much at the save's tick rate, (1 / keep - 1) / dt",
        run: Some(|e| {
            revalued(
                e,
                "og.Rope",
                2,
                &[("damping", "damping", rate_of_keep_per_tick)],
            )
        }),
    },
    Migration {
        target: Target::Record,
        name: "og.Cloth",
        from: 2,
        to: 3,
        note: "damping is a rate: the share kept per tick becomes the decay per second that keeps as much at the save's tick rate",
        run: Some(|e| {
            revalued(
                e,
                "og.Cloth",
                3,
                &[("damping", "damping", rate_of_keep_per_tick)],
            )
        }),
    },
    Migration {
        target: Target::Record,
        name: "og.Garment",
        from: 2,
        to: 3,
        note: "damping is a rate: the share kept per tick becomes the decay per second that keeps as much at the save's tick rate",
        run: Some(|e| {
            revalued(
                e,
                "og.Garment",
                3,
                &[("damping", "damping", rate_of_keep_per_tick)],
            )
        }),
    },
    Migration {
        target: Target::Record,
        name: "og.SoftBody",
        from: 2,
        to: 3,
        note: "damping is a rate: the share kept per tick becomes the decay per second that keeps as much at the save's tick rate",
        run: Some(|e| {
            revalued(
                e,
                "og.SoftBody",
                3,
                &[("damping", "damping", rate_of_keep_per_tick)],
            )
        }),
    },
    Migration {
        target: Target::Record,
        name: "og.WaterSurface",
        from: 1,
        to: 2,
        note: "damping is a rate: the share of a wave left after a second becomes the decay per second that keeps as much each tick",
        run: Some(|e| {
            revalued(
                e,
                "og.WaterSurface",
                2,
                &[("damping", "damping", rate_of_keep_per_second)],
            )
        }),
    },
    Migration {
        target: Target::Record,
        name: "og.VirtualCamera",
        from: 2,
        to: 3,
        note: "damping means a decay rate: the camera's time to ease back out is recoverTime",
        run: Some(|e| renamed(e, "og.VirtualCamera", 3)),
    },
    Migration {
        target: Target::Record,
        name: "og.AudioSource",
        from: 3,
        to: 4,
        note: "one angle rule: the cone is innerAngle and outerAngle, half-angles in radians (coneInnerAngle and coneOuterAngle were full angles in degrees), and coneOuterGain is outerGain",
        run: Some(|e| {
            revalued(
                e,
                "og.AudioSource",
                4,
                &[
                    ("coneInnerAngle", "innerAngle", half_angle_of_degrees),
                    ("coneOuterAngle", "outerAngle", half_angle_of_degrees),
                    ("coneOuterGain", "outerGain", same),
                ],
            )
        }),
    },
    Migration {
        target: Target::Record,
        name: "og.Environment",
        from: 1,
        to: 2,
        note: "one angle rule: rotation is in radians, as every other rotation (it was degrees)",
        run: Some(|e| {
            revalued(
                e,
                "og.Environment",
                2,
                &[("rotation", "rotation", radians_of_degrees)],
            )
        }),
    },
    Migration {
        target: Target::Record,
        name: "og.DayNight",
        from: 1,
        to: 2,
        note: "one angle rule: azimuth is a heading, from +z towards +x (it counted from +x), so the saved angle gains a quarter turn and the sun rises where it did",
        run: Some(|e| {
            revalued(
                e,
                "og.DayNight",
                2,
                &[("azimuth", "azimuth", heading_of_from_x)],
            )
        }),
    },
    Migration {
        target: Target::Record,
        name: "og.Aurora",
        from: 1,
        to: 2,
        note: "one angle rule: direction is a heading, from +z towards +x (it counted from +x), so the saved angle gains a quarter turn and the arcs lie where they did",
        run: Some(|e| {
            revalued(
                e,
                "og.Aurora",
                2,
                &[("direction", "direction", heading_of_from_x)],
            )
        }),
    },
    Migration {
        target: Target::Record,
        name: "og.Collider",
        from: 3,
        to: 4,
        note: "codes become enums: shape is a ColliderShape, its saved code the variant's index",
        run: Some(|e| enumerated(e, "og.Collider", 4)),
    },
    Migration {
        target: Target::Record,
        name: "og.Cloth",
        from: 3,
        to: 4,
        note: "codes become enums: simulation is a Simulation, its saved code the variant's index",
        run: Some(|e| enumerated(e, "og.Cloth", 4)),
    },
    Migration {
        target: Target::Record,
        name: "og.Garment",
        from: 3,
        to: 4,
        note: "codes become enums: simulation is a Simulation, its saved code the variant's index",
        run: Some(|e| enumerated(e, "og.Garment", 4)),
    },
    Migration {
        target: Target::Record,
        name: "og.SoftBody",
        from: 3,
        to: 4,
        note: "codes become enums: shape is a SoftBodyShape and simulation a Simulation, their saved codes the variants' indices",
        run: Some(|e| enumerated(e, "og.SoftBody", 4)),
    },
    Migration {
        target: Target::Record,
        name: "og.Fluid3D",
        from: 2,
        to: 3,
        note: "codes become enums: simulation is a Simulation, its saved code the variant's index",
        run: Some(|e| enumerated(e, "og.Fluid3D", 3)),
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
    ("og.VirtualCamera", 3, &[("damping", "recoverTime")]),
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

/// A record's `u32` code fields that became enums with its version `.1`:
/// each field and its variants in code order, as that version declared
/// them.
type Enumerated = (
    &'static str,
    u32,
    &'static [(&'static str, &'static [&'static str])],
);

/// Simulation's variants (og.Cloth, og.Garment, og.SoftBody, og.Fluid3D).
const SIMULATION: &[&str] = &["gameplay", "visual"];

/// The code fields that became enums, the stored code the variant's index.
const ENUMS: &[Enumerated] = &[
    (
        "og.Collider",
        4,
        &[(
            "shape",
            &[
                "box",
                "sphere",
                "capsule",
                "cylinder",
                "convex",
                "trimesh",
                "heightfield",
                "diamond",
                "hexagon",
            ],
        )],
    ),
    ("og.Cloth", 4, &[("simulation", SIMULATION)]),
    ("og.Garment", 4, &[("simulation", SIMULATION)]),
    (
        "og.SoftBody",
        4,
        &[
            ("shape", &["sphere", "box", "mesh"]),
            ("simulation", SIMULATION),
        ],
    ),
    ("og.Fluid3D", 3, &[("simulation", SIMULATION)]),
];

/// A record whose code fields became enums with its version `to`
/// ([`ENUMS`]): the saved type takes the enums, and every value keeps its
/// bytes, a code being its variant's index.
fn enumerated(edit: &mut SnapshotEdit, name: &str, to: u32) -> Result<(), String> {
    use og_core::reflect::FieldType;
    let (_, _, fields) = ENUMS
        .iter()
        .find(|r| r.0 == name && r.1 == to)
        .ok_or_else(|| format!("{name} has no enums at version {to}"))?;
    let Some(mut schema) = edit.schema(name).cloned() else {
        return Ok(());
    };
    schema.version = to;
    for (field, variants) in *fields {
        let f = schema
            .fields
            .iter_mut()
            .find(|f| f.name == *field && f.ty == FieldType::U32)
            .ok_or_else(|| format!("the saved {name} has no u32 {field}"))?;
        f.ty = FieldType::Enum;
        f.variants = variants.iter().map(|v| (*v).to_string()).collect();
    }
    edit.map_type(name, schema, &mut |v| Ok(v.to_vec()))
        .map_err(|e| e.reason().to_string())
}

/// The tick's length the save ran at (its og.Time's `dt`), 1/60 s where it
/// holds none (a selection).
fn tick_dt(edit: &SnapshotEdit) -> f32 {
    let saved = edit.schema("og.Time").zip(edit.resource("og.Time"));
    saved
        .and_then(|(schema, v)| {
            let (_, dt) = f32s(schema, v).into_iter().find(|(n, _)| *n == "dt")?;
            dt.first().copied()
        })
        .filter(|dt| dt.is_finite() && *dt > 0.0)
        .unwrap_or(1.0 / 60.0)
}

/// The most a converted `damping` is: a point that kept nothing.
const MAX_DAMPING: f32 = 1.0e6;

/// The decay rate (`og_core::math::damping_keep`'s) that keeps `keep` of a
/// velocity over a tick of `dt`: what a rope's, a cloth's, a garment's and
/// a soft body's `damping` was before it became a rate.
fn rate_of_keep_per_tick(keep: f32, dt: f32) -> f32 {
    let keep = keep.clamp(0.0, 1.0);
    if keep <= 0.0 {
        return MAX_DAMPING;
    }
    ((1.0 / keep - 1.0) / dt).min(MAX_DAMPING)
}

/// The decay rate that keeps each tick what og.WaterSurface's `damping`
/// kept when it was the share of a wave left after a second.
fn rate_of_keep_per_second(left: f32, dt: f32) -> f32 {
    rate_of_keep_per_tick(og_core::math::pow(left.clamp(0.0, 1.0), dt), dt)
}

/// One f32 field of a record that took another meaning: its name before,
/// its name now, and its saved value (with the tick's length the save ran
/// at) as the value that means the same now.
type Revalue = (&'static str, &'static str, fn(f32, f32) -> f32);

/// A record whose f32 fields took another meaning (and perhaps another
/// name) with its version `to`: each saved value through its conversion,
/// every other field where it was.
fn revalued(
    edit: &mut SnapshotEdit,
    name: &str,
    to: u32,
    fields: &[Revalue],
) -> Result<(), String> {
    let Some(mut schema) = edit.schema(name).cloned() else {
        return Ok(());
    };
    let mut moves = Vec::with_capacity(fields.len());
    for &(old, new, convert) in fields {
        let i = schema
            .fields
            .iter()
            .position(|f| f.name == old && f.ty == og_core::reflect::FieldType::F32)
            .ok_or_else(|| format!("the saved {name} has no f32 {old}"))?;
        let at: usize = schema.fields[..i].iter().map(|f| f.ty.size()).sum();
        schema.fields[i].name = new.to_string();
        moves.push((at, convert));
    }
    let dt = tick_dt(edit);
    schema.version = to;
    edit.map_type(name, schema, &mut |v| {
        let mut out = v.to_vec();
        for &(at, convert) in &moves {
            let old = f32::from_le_bytes(v[at..at + 4].try_into().expect("an f32"));
            out[at..at + 4].copy_from_slice(&convert(old, dt).to_le_bytes());
        }
        Ok(out)
    })
    .map_err(|e| e.reason().to_string())
}

/// A cone's full angle in degrees (Web Audio's) as its half-angle in
/// radians; all round (360) is exactly pi, og.AudioSource's default.
fn half_angle_of_degrees(full: f32, _dt: f32) -> f32 {
    if full >= 360.0 {
        std::f32::consts::PI
    } else {
        (full.max(0.0) * 0.5).to_radians()
    }
}

/// A value the step keeps as it is (a field that only took another name).
fn same(v: f32, _dt: f32) -> f32 {
    v
}

/// Degrees as radians (og.Environment.rotation).
fn radians_of_degrees(v: f32, _dt: f32) -> f32 {
    v.to_radians()
}

/// An angle about +y counted from +x (og.DayNight.azimuth and
/// og.Aurora.direction before) as the heading of the same direction,
/// counted from +z: a quarter turn more, in degrees.
fn heading_of_from_x(degrees: f32, _dt: f32) -> f32 {
    degrees + 90.0
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

    /// `now` as the build that registered it at `version` laid it out:
    /// every later rename and enum step undone, newest first. A step that
    /// only changed what a value means left the layout alone.
    fn laid_out_at(now: &og_core::reflect::Schema, version: u32) -> og_core::reflect::Schema {
        let mut old = now.clone();
        let name = now.name.as_str();
        for v in (version + 1..=now.version).rev() {
            if let Some((_, _, fields)) = RENAMES.iter().find(|r| r.0 == name && r.1 == v) {
                for (before, after) in *fields {
                    let f = old
                        .fields
                        .iter_mut()
                        .find(|f| f.name == *after)
                        .unwrap_or_else(|| panic!("{name} has no field {after}"));
                    f.name = (*before).to_string();
                }
            }
            if let Some((_, _, fields)) = ENUMS.iter().find(|r| r.0 == name && r.1 == v) {
                for (field, _) in *fields {
                    let f = old
                        .fields
                        .iter_mut()
                        .find(|f| f.name == *field)
                        .unwrap_or_else(|| panic!("{name} has no field {field}"));
                    f.ty = og_core::reflect::FieldType::U32;
                    f.variants.clear();
                }
            }
        }
        old.version = version;
        old
    }

    /// How many steps the table runs on the record `name` saved at `version`.
    fn steps_from(name: &str, version: u32) -> usize {
        MIGRATIONS
            .iter()
            .filter(|m| {
                m.target == Target::Record && m.name == name && m.from >= version && m.run.is_some()
            })
            .count()
    }

    /// Each record of [`ENUMS`] as the build before wrote it (plain codes)
    /// upgrades to the layout this build registers, each code where it
    /// was: the variant's index.
    #[test]
    fn a_saved_code_restores_as_the_enum_variant_of_that_index() {
        for &(name, to, fields) in ENUMS {
            let decl = og_api::host::RECORDS
                .iter()
                .find(|d| d.schema == name)
                .unwrap_or_else(|| panic!("{name} is not an engine record"));
            let layout = crate::host::codec::Layout::from_decl(decl).unwrap();
            let now = crate::engine::versioned(layout.schema());
            let old = laid_out_at(&now, to - 1);
            let mut bytes = layout.encode(None, None).unwrap();
            for (field, variants) in fields {
                let declared = now.fields.iter().find(|f| f.name == *field).unwrap();
                assert_eq!(declared.variants, *variants, "{name}.{field}");
                // The last code: the last variant.
                let at: usize = now
                    .fields
                    .iter()
                    .take_while(|f| f.name != *field)
                    .map(|f| f.ty.size())
                    .sum();
                let code = variants.len() as u32 - 1;
                bytes[at..at + 4].copy_from_slice(&code.to_le_bytes());
            }
            let mut world = og_ecs::World::new(&og_ecs::WorldDesc::default()).unwrap();
            let id = world.register_schema(old).unwrap();
            let e = world.spawn_bytes(vec![(id, bytes.clone())]).unwrap();
            let (upgraded, steps) = upgrade(&world.snapshot())
                .unwrap()
                .unwrap_or_else(|| panic!("{name}: no step ran"));
            assert_eq!(steps.len(), steps_from(name, to - 1), "{name}: {steps:?}");
            let edit = SnapshotEdit::from_bytes(&upgraded).unwrap();
            let saved = edit.schema(name).unwrap();
            assert_eq!(saved.layout_hash(), now.layout_hash(), "{name}: {saved:?}");
            assert_eq!(edit.components(name), vec![(e, bytes)], "{name}");
            assert!(upgrade(&upgraded).unwrap().is_none(), "{name}: twice");
        }
    }

    /// Each record of [`RENAMES`] as the build before wrote it (the old
    /// names, the version before) upgrades to the layout this build
    /// registers, so the restore takes it, with its values where they were.
    #[test]
    fn a_renamed_records_save_upgrades_to_this_builds_layout_with_its_values() {
        use crate::host::codec::RecordKind;
        for &(name, to, _) in RENAMES {
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
            // Later steps of the record run after this one: the save as the
            // build before this step wrote it undoes them too.
            let old = laid_out_at(&now, to - 1);
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
            assert_eq!(steps.len(), steps_from(name, to - 1), "{name}: {steps:?}");
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

    /// A save from before `damping` was a rate: each soft thing's share
    /// kept per tick, and the water surface's share left after a second,
    /// restore as the rate that keeps as much a tick at the save's tick
    /// rate (30 Hz here), the other fields where they were.
    #[test]
    fn a_saved_damping_becomes_the_rate_that_keeps_as_much_a_tick() {
        use og_core::Reflect;
        let dt = 1.0 / 30.0;
        // Each with the version it had before its damping became a rate.
        for (now, before, kept_per_tick) in [
            (og_physics::Rope::schema(), 1, 0.99f32),
            (og_physics::Cloth::schema(), 2, 0.985),
            (og_physics::Garment::schema(), 2, 0.97),
            (og_physics::SoftBody::schema(), 2, 0.995),
            (og_physics::WaterSurface::schema(), 1, 0.0),
        ] {
            let name = now.name.clone();
            let water = name == "og.WaterSurface";
            // The value the old build saved: the water's is per second.
            let (saved, keep) = if water {
                (0.4f32, og_core::math::pow(0.4, dt))
            } else {
                (kept_per_tick, kept_per_tick)
            };
            let decl = og_api::host::RECORDS
                .iter()
                .find(|d| d.schema == name)
                .unwrap();
            let layout = crate::host::codec::Layout::from_decl(decl).unwrap();
            assert_eq!(
                crate::engine::versioned(layout.schema()).version,
                now.version,
                "{name}"
            );
            let old = laid_out_at(&crate::engine::versioned(layout.schema()), before);
            let at: usize = old
                .fields
                .iter()
                .take_while(|f| f.name != "damping")
                .map(|f| f.ty.size())
                .sum();
            let mut bytes = layout.encode(None, None).unwrap();
            bytes[at..at + 4].copy_from_slice(&saved.to_le_bytes());
            let mut world = og_ecs::World::new(&og_ecs::WorldDesc {
                tick_rate: 30,
                seed: 1,
            })
            .unwrap();
            let id = world.register_schema(old).unwrap();
            let e = world.spawn_bytes(vec![(id, bytes.clone())]).unwrap();
            let (upgraded, steps) = upgrade(&world.snapshot())
                .unwrap()
                .unwrap_or_else(|| panic!("{name}: no step ran"));
            assert_eq!(steps.len(), steps_from(&name, before), "{name}: {steps:?}");
            let edit = SnapshotEdit::from_bytes(&upgraded).unwrap();
            assert_eq!(edit.schema(&name).unwrap().version, now.version, "{name}");
            let rows = edit.components(&name);
            let (entity, row) = &rows[0];
            assert_eq!(*entity, e);
            let rate = f32::from_le_bytes(row[at..at + 4].try_into().unwrap());
            let kept = og_core::math::damping_keep(rate, dt);
            assert!((kept - keep).abs() < 1e-5, "{name}: {rate} keeps {kept}");
            assert_eq!(row[..at], bytes[..at], "{name}");
            assert_eq!(row[at + 4..], bytes[at + 4..], "{name}");
            assert!(upgrade(&upgraded).unwrap().is_none(), "{name}: twice");
        }
    }

    /// A record as the build before its last step saved it (`fields`: a
    /// field's name now, its name then, the value saved), upgraded: the
    /// values of those fields now, in order. Every other field holds its
    /// default and must come through untouched.
    fn upgraded(name: &str, fields: &[(&str, &str, f32)]) -> Vec<f32> {
        use crate::host::codec::RecordKind;
        let decl = og_api::host::RECORDS
            .iter()
            .find(|d| d.schema == name)
            .unwrap_or_else(|| panic!("{name} is not an engine record"));
        let layout = crate::host::codec::Layout::from_decl(decl).unwrap();
        let now = crate::engine::versioned(layout.schema());
        let at = |field: &str| -> usize {
            now.fields
                .iter()
                .take_while(|f| f.name != field)
                .map(|f| f.ty.size())
                .sum()
        };
        let mut old = now.clone();
        old.version -= 1;
        let mut bytes = layout.encode(None, None).unwrap();
        let defaults = bytes.clone();
        for &(field, before, saved) in fields {
            let f = old.fields.iter_mut().find(|f| f.name == field).unwrap();
            f.name = before.to_string();
            bytes[at(field)..at(field) + 4].copy_from_slice(&saved.to_le_bytes());
        }
        let mut world = og_ecs::World::new(&og_ecs::WorldDesc::default()).unwrap();
        let id = world.register_schema(old).unwrap();
        let resource = layout.kind == RecordKind::Resource;
        if resource {
            world.insert_resource_bytes(id, &bytes).unwrap();
        } else {
            world.spawn_bytes(vec![(id, bytes)]).unwrap();
        }
        let (upgraded, steps) = upgrade(&world.snapshot())
            .unwrap()
            .unwrap_or_else(|| panic!("{name}: no step ran"));
        assert_eq!(steps.len(), 1, "{name}: {steps:?}");
        let edit = SnapshotEdit::from_bytes(&upgraded).unwrap();
        let saved = edit.schema(name).unwrap();
        assert_eq!(saved.layout_hash(), now.layout_hash(), "{name}: {saved:?}");
        let row = if resource {
            edit.resource(name).unwrap().to_vec()
        } else {
            edit.components(name).remove(0).1
        };
        let mut rest = row.clone();
        let mut values = Vec::new();
        for &(field, _, _) in fields {
            let i = at(field);
            values.push(f32::from_le_bytes(row[i..i + 4].try_into().unwrap()));
            rest[i..i + 4].copy_from_slice(&defaults[i..i + 4]);
        }
        assert_eq!(rest, defaults, "{name}: another field moved");
        assert!(upgrade(&upgraded).unwrap().is_none(), "{name}: twice");
        values
    }

    /// Saves from before the one angle rule mean what they did: a sound's
    /// cone (full angles in degrees) as half-angles in radians, all round
    /// exactly pi; an environment's turn in radians; the sun's rise and
    /// the aurora's arcs, counted from +x, as headings.
    #[test]
    fn saved_angles_take_the_one_rule_and_point_where_they_did() {
        use std::f32::consts::PI;
        let cone = upgraded(
            "og.AudioSource",
            &[
                ("innerAngle", "coneInnerAngle", 90.0),
                ("outerAngle", "coneOuterAngle", 240.0),
                ("outerGain", "coneOuterGain", 0.25),
            ],
        );
        assert!((cone[0] - PI / 4.0).abs() < 1e-6, "{cone:?}");
        assert!((cone[1] - 2.0 * PI / 3.0).abs() < 1e-6, "{cone:?}");
        assert_eq!(cone[2], 0.25);
        let all_round = upgraded(
            "og.AudioSource",
            &[
                ("innerAngle", "coneInnerAngle", 360.0),
                ("outerAngle", "coneOuterAngle", 360.0),
                ("outerGain", "coneOuterGain", 0.0),
            ],
        );
        assert_eq!(
            all_round,
            [PI, PI, 0.0],
            "the default stays omnidirectional"
        );
        let turn = upgraded("og.Environment", &[("rotation", "rotation", 90.0)]);
        assert!((turn[0] - PI / 2.0).abs() < 1e-6, "{turn:?}");
        // The old default rose in +x (0 from +x): the heading 90.
        assert_eq!(
            upgraded("og.DayNight", &[("azimuth", "azimuth", 0.0)]),
            [90.0]
        );
        assert_eq!(
            upgraded("og.Aurora", &[("direction", "direction", 90.0)]),
            [180.0]
        );
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
