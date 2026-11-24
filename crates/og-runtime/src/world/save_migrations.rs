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
//!
//! A record that took another name is in [`RETYPED`]: its saved type takes
//! the name before any other step looks at the save, so the steps and the
//! restore see it under the name this build declares.

use og_ecs::SnapshotEdit;

/// A step's edit of the save.
type Step = fn(&mut SnapshotEdit) -> Result<(), String>;

/// An upgraded snapshot and the steps it took, for the log.
type Upgraded = (Vec<u8>, Vec<String>);

/// One engine step: `name` at version `from` becomes version `to`.
pub(crate) struct Migration {
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
        name: "og.Gamepad",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: four pads, now eight (G1-2); a session record, so the host's value replaces the saved one",
        run: None,
    },
    Migration {
        name: "og.AudioSource",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained userBus (G2-5, the bus graph); the restore takes it at 0, the source's own bus",
        run: None,
    },
    Migration {
        name: "og.AudioBusFx",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained userBus, the equaliser, the echo and the waveshaper (G2-5); the restore takes them at their defaults, all off",
        run: None,
    },
    Migration {
        name: "og.RigidBody",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained canSleep and sleeping (G2-3, sleep); the restore takes them at true and false, so a body can sleep and is awake",
        run: None,
    },
    Migration {
        name: "og.RigidBody",
        from: 3,
        to: 4,
        note: "0.7.7 and earlier: gained airDrag (the air on bodies); the restore takes it at 0, a body the wind does not move",
        run: None,
    },
    Migration {
        name: "og.Physics",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained sleepTicks and sleepSpeed (G2-3, sleep); the restore takes them at 0 and 0.1, sleep off as before",
        run: None,
    },
    Migration {
        name: "og.Joint",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained breakForce, breakTorque and broken (G2-3, breaking joints); the restore takes them at 0, 0 and false, a joint that never breaks",
        run: None,
    },
    Migration {
        name: "og.Vehicle",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained model, handbrake, handbrakeGrip, driftGrip and the drivetrain's fields (G2-3); the restore takes their defaults: model VEHICLE_ARCADE, the car it was",
        run: None,
    },
    Migration {
        name: "og.VideoPlayer",
        from: 1,
        to: 2,
        note: "G3-2: time became an f64 (an f32 sum fell behind the sound over a long stream); a save without a player drops the type, which is lazy now",
        run: Some(video_player_1_to_2),
    },
    Migration {
        name: "og.VideoPlayer",
        from: 2,
        to: 3,
        note: "0.7.7 and earlier: gained wait (a stream holding the clock until its frame is shown); the restore takes it off, a player that never waits",
        run: None,
    },
    Migration {
        name: "og.Lighting2D",
        from: 1,
        to: 2,
        note: "R11: gained shadowAngles; at its defaults (a game that never lit in 2D) it is dropped, as the record is lazy now",
        run: Some(|e| became_lazy(e, "og.Lighting2D")),
    },
    Migration {
        name: "og.Light2D",
        from: 1,
        to: 2,
        note: "R11: gained kind, innerAngle, outerAngle, height and layers; dropped from a save without lights, as the record is lazy now",
        run: Some(|e| became_lazy(e, "og.Light2D")),
    },
    Migration {
        name: "og.Occluder2D",
        from: 1,
        to: 2,
        note: "R11: gained points and layers; dropped from a save without occluders, as the record is lazy now",
        run: Some(|e| became_lazy(e, "og.Occluder2D")),
    },
    Migration {
        name: "og.Wind",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier, registered in every world: gained turbulence and gustSize (G2, the simulated field); at its defaults (a game that never set a wind) it is dropped, as the record is lazy now",
        run: Some(wind_1_to_2),
    },
    Migration {
        name: "og.Wind",
        from: 2,
        to: 3,
        note: "the world's wind: strength (the lean per metre) is gone and speed is the air's, so the saved lean becomes the speed that leans as far (strength / 0.05 m/s)",
        run: Some(wind_2_to_3),
    },
    Migration {
        name: "og.Collider",
        from: 2,
        to: 3,
        note: "one name per meaning: rotation is quaternion",
        run: Some(|e| renamed(e, "og.Collider", 3)),
    },
    Migration {
        name: "og.Light2D",
        from: 2,
        to: 3,
        note: "one name per meaning: radius is distance, as og.PointLight's",
        run: Some(|e| renamed(e, "og.Light2D", 3)),
    },
    Migration {
        name: "og.AiAgent",
        from: 1,
        to: 2,
        note: "one name per meaning: thinkEvery is thinkEveryTicks",
        run: Some(|e| renamed(e, "og.AiAgent", 2)),
    },
    Migration {
        name: "og.Perception",
        from: 1,
        to: 2,
        note: "one name per meaning: occluders is occluderMask",
        run: Some(|e| renamed(e, "og.Perception", 2)),
    },
    Migration {
        name: "og.PlatformerController",
        from: 1,
        to: 2,
        note: "one name per meaning: moveX is run, impulse is addVelocity, oneWayLayers is oneWayMask",
        run: Some(|e| renamed(e, "og.PlatformerController", 2)),
    },
    Migration {
        name: "og.Lockstep",
        from: 1,
        to: 2,
        note: "one name per meaning: inputDelay, hashEvery and maxPrediction say Ticks, stallTimeout says Ms",
        run: Some(|e| renamed(e, "og.Lockstep", 2)),
    },
    Migration {
        name: "og.VirtualCamera",
        from: 1,
        to: 2,
        note: "one name per meaning: collisionLayers is collisionMask",
        run: Some(|e| renamed(e, "og.VirtualCamera", 2)),
    },
    Migration {
        name: "og.CameraView",
        from: 1,
        to: 2,
        note: "one name per meaning: every is everyTicks",
        run: Some(|e| renamed(e, "og.CameraView", 2)),
    },
    Migration {
        name: "og.Mirror",
        from: 1,
        to: 2,
        note: "one name per meaning: every is everyTicks",
        run: Some(|e| renamed(e, "og.Mirror", 2)),
    },
    Migration {
        name: "og.Afterimage",
        from: 1,
        to: 2,
        note: "one name per meaning: interval is intervalTicks",
        run: Some(|e| renamed(e, "og.Afterimage", 2)),
    },
    Migration {
        name: "og.ReflectionProbe",
        from: 1,
        to: 2,
        note: "one name per meaning: interval is intervalTicks",
        run: Some(|e| renamed(e, "og.ReflectionProbe", 2)),
    },
    Migration {
        name: "og.Input",
        from: 2,
        to: 3,
        note: "the input API's one word per state: the key and button edges say justPressed and justReleased (pressed0-3, released0-3, buttonsPressed, buttonsReleased)",
        run: Some(|e| renamed(e, "og.Input", 3)),
    },
    Migration {
        name: "og.Pointers",
        from: 1,
        to: 2,
        note: "the input API's one word per state: pressedMask and releasedMask say justPressedMask and justReleasedMask",
        run: Some(|e| renamed(e, "og.Pointers", 2)),
    },
    Migration {
        name: "og.Gamepad",
        from: 2,
        to: 3,
        note: "the input API's one word per state: the button edges say justPressed and justReleased (pressed0-7, released0-7)",
        run: Some(|e| renamed(e, "og.Gamepad", 3)),
    },
    Migration {
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
        name: "og.VirtualCamera",
        from: 2,
        to: 3,
        note: "damping means a decay rate: the camera's time to ease back out is recoverTime",
        run: Some(|e| renamed(e, "og.VirtualCamera", 3)),
    },
    Migration {
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
        name: "og.Collider",
        from: 3,
        to: 4,
        note: "codes become enums: shape is a ColliderShape, its saved code the variant's index",
        run: Some(|e| enumerated(e, "og.Collider", 4)),
    },
    Migration {
        name: "og.Cloth",
        from: 3,
        to: 4,
        note: "codes become enums: simulation is a Simulation, its saved code the variant's index",
        run: Some(|e| enumerated(e, "og.Cloth", 4)),
    },
    Migration {
        name: "og.Garment",
        from: 3,
        to: 4,
        note: "codes become enums: simulation is a Simulation, its saved code the variant's index",
        run: Some(|e| enumerated(e, "og.Garment", 4)),
    },
    Migration {
        name: "og.SoftBody",
        from: 3,
        to: 4,
        note: "codes become enums: shape is a SoftBodyShape and simulation a Simulation, their saved codes the variants' indices",
        run: Some(|e| enumerated(e, "og.SoftBody", 4)),
    },
    Migration {
        name: "og.Fluid3D",
        from: 2,
        to: 3,
        note: "codes become enums: simulation is a Simulation, its saved code the variant's index",
        run: Some(|e| enumerated(e, "og.Fluid3D", 3)),
    },
    Migration {
        name: "og.Joint",
        from: 2,
        to: 3,
        note: "codes become enums: kind is a JointType and motor a JointMotor, their saved codes the variants' indices",
        run: Some(|e| enumerated(e, "og.Joint", 3)),
    },
    Migration {
        name: "og.Vehicle",
        from: 2,
        to: 3,
        note: "codes become enums: drive is a VehicleDrive and model a VehicleModel, their saved codes the variants' indices",
        run: Some(|e| enumerated(e, "og.Vehicle", 3)),
    },
    Migration {
        name: "og.Trail",
        from: 1,
        to: 2,
        note: "one Blend for effects: blend is a Blend, alpha 0 and additive 1 as a sprite's (StripBlend had additive 0), each saved value kept by its name",
        run: Some(|e| revariant(e, "og.Trail", 2, "blend", BLEND)),
    },
    Migration {
        name: "og.Beam",
        from: 1,
        to: 2,
        note: "one Blend for effects: blend is a Blend, alpha 0 and additive 1 as a sprite's (StripBlend had additive 0), each saved value kept by its name",
        run: Some(|e| revariant(e, "og.Beam", 2, "blend", BLEND)),
    },
    Migration {
        name: "og.Beam",
        from: 2,
        to: 3,
        note: "one tiling for strips: tile (repeats per metre) is uvTile (metres per repeat, as og.Trail's), the saved value its reciprocal",
        run: Some(beam_2_to_3),
    },
    Migration {
        name: "og.Time",
        from: 1,
        to: 2,
        note: "one time base: the clock is its tick and the integer tick rate (the saved dt's reciprocal, rounded); dt and elapsed are computed from them",
        run: Some(time_1_to_2),
    },
    Migration {
        name: "og.DayNight",
        from: 2,
        to: 3,
        note: "f64 clocks: start and speed are f64, each saved value widened as it was",
        run: Some(|e| widened(e, "og.DayNight", 3)),
    },
    Migration {
        name: "og.AnimationEvent",
        from: 1,
        to: 2,
        note: "0.9.0 and earlier: gained state, layer and dominant (where a marker came from); a pending event restores with them at 0, 0 and false",
        run: None,
    },
];

/// og.Time 1 -> 2: `tick`, `dt` and `elapsed` become `tick` and `rate`,
/// the rate the saved `dt` was the rounded reciprocal of (1/60 s is 60).
fn time_1_to_2(edit: &mut SnapshotEdit) -> Result<(), String> {
    use og_core::reflect::FieldType;
    let name = "og.Time";
    let Some(old) = edit.schema(name).cloned() else {
        return Ok(());
    };
    let at = |field: &str, ty: FieldType| -> Result<usize, String> {
        let i = old
            .fields
            .iter()
            .position(|f| f.name == field && f.ty == ty)
            .ok_or_else(|| format!("the saved og.Time has no {} {field}", ty.name()))?;
        Ok(old.fields[..i].iter().map(|f| f.ty.size()).sum())
    };
    let (tick, dt) = (at("tick", FieldType::U64)?, at("dt", FieldType::F32)?);
    let schema = <og_core::Time as og_core::Reflect>::schema();
    edit.map_type(name, schema, &mut |v| {
        let dt = f32::from_le_bytes(v[dt..dt + 4].try_into().expect("an f32"));
        let rate = if dt.is_finite() && dt > 0.0 {
            (1.0 / f64::from(dt))
                .round()
                .clamp(1.0, f64::from(u32::MAX)) as u32
        } else {
            60
        };
        let mut out = v[tick..tick + 8].to_vec();
        out.extend_from_slice(&rate.to_le_bytes());
        Ok(out)
    })
    .map_err(|e| e.reason().to_string())
}

/// The f32 fields that became f64 with a record's version.
const WIDENED: &[(&str, u32, &[&str])] = &[
    ("og.VideoPlayer", 2, &["time"]),
    ("og.DayNight", 3, &["start", "speed"]),
];

/// A record whose f32 fields became f64 with its version `to`
/// ([`WIDENED`]): each saved value widened, every other field as it was.
fn widened(edit: &mut SnapshotEdit, name: &str, to: u32) -> Result<(), String> {
    use og_core::reflect::FieldType;
    let (_, _, fields) = WIDENED
        .iter()
        .find(|r| r.0 == name && r.1 == to)
        .ok_or_else(|| format!("{name} has no widened fields at version {to}"))?;
    let Some(old) = edit.schema(name).cloned() else {
        return Ok(());
    };
    let wide: Vec<(bool, usize)> = old
        .fields
        .iter()
        .map(|f| {
            (
                fields.contains(&f.name.as_str()) && f.ty == FieldType::F32,
                f.ty.size(),
            )
        })
        .collect();
    if let Some(missing) = fields.iter().find(|n| {
        !old.fields
            .iter()
            .any(|f| f.name == **n && f.ty == FieldType::F32)
    }) {
        return Err(format!("the saved {name} has no f32 {missing}"));
    }
    let mut schema = old;
    schema.version = to;
    for f in &mut schema.fields {
        if fields.contains(&f.name.as_str()) {
            f.ty = FieldType::F64;
        }
    }
    edit.map_type(name, schema, &mut |v| {
        let mut out = Vec::with_capacity(v.len() + 4 * fields.len());
        let mut at = 0;
        for &(wide, n) in &wide {
            let b = &v[at..at + n];
            if wide {
                let x = f32::from_le_bytes(b.try_into().expect("an f32 is 4 bytes"));
                out.extend_from_slice(&f64::from(x).to_le_bytes());
            } else {
                out.extend_from_slice(b);
            }
            at += n;
        }
        Ok(out)
    })
    .map_err(|e| e.reason().to_string())
}

/// The uvTile a saved og.Beam `tile` of 0 becomes: version 2 repeated the
/// texture every 1e6 m (its first column along the whole beam), where a
/// uvTile of 0 now stretches it once.
const BEAM_UNTILED: f32 = 1.0e6;

/// og.Beam 2 -> 3: `tile`, repeats per metre, is `uvTile`, metres per
/// repeat; the saved value becomes its reciprocal, so the beam draws as it
/// did.
fn beam_2_to_3(edit: &mut SnapshotEdit) -> Result<(), String> {
    use og_core::reflect::FieldType;
    let name = "og.Beam";
    let Some(old) = edit.schema(name).cloned() else {
        return Ok(());
    };
    let i = old
        .fields
        .iter()
        .position(|f| f.name == "tile" && f.ty == FieldType::F32)
        .ok_or("the saved og.Beam has no f32 tile")?;
    let start: usize = old.fields[..i].iter().map(|f| f.ty.size()).sum();
    let mut schema = old;
    schema.version = 3;
    schema.fields[i].name = "uvTile".to_string();
    edit.map_type(name, schema, &mut |v| {
        let mut out = v.to_vec();
        let at = start..start + 4;
        let tile = f32::from_le_bytes(out[at.clone()].try_into().expect("an f32"));
        let length = if tile > 0.0 { 1.0 / tile } else { BEAM_UNTILED };
        out[at].copy_from_slice(&length.to_le_bytes());
        Ok(out)
    })
    .map_err(|e| e.reason().to_string())
}

/// Blend's variants in code order (og.Trail, og.Beam 2).
const BLEND: &[&str] = &["alpha", "additive"];

/// A record whose enum `field` took `variants`, in this order, with its
/// version `to`: each saved value moves to its variant's index there,
/// found by name, so a saved alpha trail restores alpha.
fn revariant(
    edit: &mut SnapshotEdit,
    name: &str,
    to: u32,
    field: &str,
    variants: &[&str],
) -> Result<(), String> {
    use og_core::reflect::FieldType;
    let Some(old) = edit.schema(name).cloned() else {
        return Ok(());
    };
    let i = old
        .fields
        .iter()
        .position(|f| f.name == field && f.ty == FieldType::Enum)
        .ok_or_else(|| format!("the saved {name} has no enum {field}"))?;
    let start: usize = old.fields[..i].iter().map(|f| f.ty.size()).sum();
    let codes = old.fields[i]
        .variants
        .iter()
        .map(|v| {
            variants
                .iter()
                .position(|w| w == v)
                .map(|j| j as u32)
                .ok_or_else(|| format!("the saved {name}.{field} has {v}, which {to} has not"))
        })
        .collect::<Result<Vec<u32>, String>>()?;
    let mut schema = old;
    schema.version = to;
    schema.fields[i].variants = variants.iter().map(|v| (*v).to_string()).collect();
    edit.map_type(name, schema, &mut |v| {
        let mut out = v.to_vec();
        let at = start..start + 4;
        let code = u32::from_le_bytes(out[at.clone()].try_into().expect("an enum is a u32"));
        let new = codes.get(code as usize).copied().unwrap_or(0);
        out[at].copy_from_slice(&new.to_le_bytes());
        Ok(out)
    })
    .map_err(|e| e.reason().to_string())
}

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
    (
        "og.Joint",
        3,
        &[
            (
                "kind",
                &["fixed", "revolute", "prismatic", "spring", "rope", "ball"],
            ),
            ("motor", &["none", "velocity", "position"]),
        ],
    ),
    (
        "og.Vehicle",
        3,
        &[
            ("drive", &["rear", "front", "all"]),
            ("model", &["arcade", "drivetrain"]),
        ],
    ),
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

/// The tick's length the save ran at, 1/60 s where it holds no og.Time (a
/// selection): its og.Time's `rate` (version 2), or the `dt` version 1
/// held, which a save from before one time base has until og.Time's own
/// step, the table's last, runs.
fn tick_dt(edit: &SnapshotEdit) -> f32 {
    use og_core::reflect::FieldType;
    let Some((schema, v)) = edit.schema("og.Time").zip(edit.resource("og.Time")) else {
        return 1.0 / 60.0;
    };
    let mut at = 0;
    for f in &schema.fields {
        let b = &v[at..at + f.ty.size()];
        at += f.ty.size();
        match (f.name.as_str(), f.ty) {
            ("rate", FieldType::U32) => {
                let rate = u32::from_le_bytes(b.try_into().expect("a u32"));
                return og_core::Time::from_rate(rate).dt();
            }
            ("dt", FieldType::F32) => {
                let dt = f32::from_le_bytes(b.try_into().expect("an f32"));
                if dt.is_finite() && dt > 0.0 {
                    return dt;
                }
            }
            _ => {}
        }
    }
    1.0 / 60.0
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
    let name = crate::bind::video::PLAYER;
    if edit.schema(name).is_none() {
        return Ok(());
    }
    if edit.components(name).is_empty() && edit.resource(name).is_none() && !edit.has_events(name) {
        return edit
            .replace_types(&[name], Vec::new())
            .map_err(|e| e.reason().to_string());
    }
    widened(edit, name, 2)
}

/// A record that took another name: the name it was saved under, its name
/// now, and its fields that took another name with it.
type Retyped = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
);

/// The records that took another name, oldest first.
const RETYPED: &[Retyped] = &[
    // og.film became og.flow; the field naming a deck's or an event's flow
    // went with it.
    ("og.Film", "og.Flow", &[]),
    ("og.FilmDeck", "og.FlowDeck", &[("film", "flow")]),
    ("og.FilmSegment", "og.FlowSegment", &[("film", "flow")]),
    ("og.FilmChoice", "og.FlowChoice", &[("film", "flow")]),
    ("og.FilmChapter", "og.FlowChapter", &[("film", "flow")]),
    ("og.FilmEnded", "og.FlowEnded", &[("film", "flow")]),
];

/// Gives every saved type in [`RETYPED`] its name now: its components,
/// resource and events keep their bytes and its version. Returns what it
/// renamed, for the log.
fn retype(edit: &mut SnapshotEdit) -> Result<Vec<String>, String> {
    let mut done = Vec::new();
    for (saved, now, fields) in RETYPED {
        let Some(mut schema) = edit.schema(saved).cloned() else {
            continue;
        };
        schema.name = (*now).to_string();
        for f in &mut schema.fields {
            if let Some((_, new)) = fields.iter().find(|(old, _)| f.name == *old) {
                f.name = (*new).to_string();
            }
        }
        edit.map_type(saved, schema, &mut |v| Ok(v.to_vec()))
            .map_err(|e| {
                format!(
                    "the engine's rename of {saved} to {now} failed: {}",
                    e.reason()
                )
            })?;
        done.push(format!("{saved} is {now}"));
    }
    Ok(done)
}

/// A state block's version: the `u32` its bytes start with.
/// Upgrades a world snapshot (or a selection) taken by an older engine:
/// `Ok(None)` when nothing in it needed a step (the bytes restore as they
/// are), `Ok(Some(bytes))` the upgraded snapshot and the steps it took.
/// A step that fails refuses the save, naming the step. A snapshot this
/// module cannot read is left to the restore, which says why.
pub(crate) fn upgrade(snapshot: &[u8]) -> Result<Option<Upgraded>, String> {
    let Ok(mut edit) = SnapshotEdit::from_bytes(snapshot) else {
        return Ok(None);
    };
    // Names first: the lazy check and the steps below go by this build's.
    let mut done = retype(&mut edit)?;
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
        let at = edit.schema(m.name).map(|s| s.version);
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
