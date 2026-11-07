//! An Effekseer project's node tree as the importer reads it: the subset of
//! each node's parameters it maps (`Data/*Values.cs`, MIT;
//! `third_party/effekseer/`), the editor's defaults where a value is not
//! written, and what it holds that the importer does not map.

use super::value::{
    self, Anim, Anim3, Color, Ease, Kinds, Rng, anim3, boolean, f, int, rnd, v2, v2rnd, v3, v3rnd,
};
use super::xml::El;
use glam::Vec3;

/// What a node draws (`RendererValues.ParamaterType`).
#[derive(Clone, Debug, PartialEq)]
pub enum Draw {
    None,
    /// `billboard`: 0 billboard, 1 y-axis fixed, 2 fixed, 3 rotated, 4 directional.
    Sprite {
        billboard: i32,
    },
    Ribbon {
        width: f32,
    },
    Ring(Box<Ring>),
    Model {
        model: Option<String>,
        billboard: i32,
    },
    /// Widths at the head, the middle and the tail.
    Track {
        width: [f32; 3],
    },
    Other(i32),
}

impl Draw {
    pub fn name(&self) -> &'static str {
        match self {
            Draw::None => "none",
            Draw::Sprite { .. } => "sprite",
            Draw::Ribbon { .. } => "ribbon",
            Draw::Ring(_) => "ring",
            Draw::Model { .. } => "model",
            Draw::Track { .. } => "track",
            Draw::Other(_) => "renderer",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ring {
    pub billboard: i32,
    /// Outer and inner radius over the life (x of each location).
    pub outer: Anim,
    pub inner: Anim,
    /// Outer and inner height over the life (y of each location), along
    /// the ring's axis: a ring whose edges differ in height is a cone or a
    /// tube.
    pub outer_height: Anim,
    pub inner_height: Anim,
    pub center_ratio: f32,
    /// Degrees of the arc drawn (360 a whole ring).
    pub viewing_angle: f32,
    pub outer_color: Color,
    pub center_color: Color,
    pub inner_color: Color,
}

/// How an instance spreads from its parent (`GenerationLocationValues`).
#[derive(Clone, Debug, PartialEq)]
pub enum Spawn {
    Point([Rng; 3]),
    /// Radius; `RotationX`/`RotationY` ranges in degrees.
    Sphere {
        radius: Rng,
        rot_x: Rng,
        rot_y: Rng,
    },
    /// Radius in the plane across `axis` (0 x, 1 y, 2 z), its angles
    /// (degrees), its division, and whether instances take it in order.
    Circle {
        radius: Rng,
        axis: i32,
        start: Rng,
        end: Rng,
        division: i32,
        ordered: bool,
    },
    Line {
        from: [Rng; 3],
        to: [Rng; 3],
    },
    Model,
}

/// How UVs are laid over the texture (`RendererCommonValues.UVType`), in
/// the texture's pixels as the editor stores them.
#[derive(Clone, Debug, PartialEq)]
pub enum Uv {
    Default,
    Fixed {
        start: [f32; 2],
        size: [f32; 2],
    },
    Animation {
        start: [f32; 2],
        size: [f32; 2],
        /// Frames a cell shows; `None` holds it.
        frame_length: Option<f32>,
        count: [i32; 2],
        /// 0 once, 1 loop, 2 ping-pong.
        loop_type: i32,
    },
    Scroll {
        start: [Rng; 2],
        size: [Rng; 2],
        speed: [Rng; 2],
    },
    Curve,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Render {
    /// As written, relative to the project file.
    pub texture: Option<String>,
    /// 0 opaque, 1 alpha, 2 add, 3 subtract, 4 multiply.
    pub blend: i32,
    pub uv: Uv,
    /// Frames, and the easing.
    pub fade_in: Option<(f32, Ease)>,
    pub fade_out: Option<(f32, Ease)>,
    /// `FadeOutType` 2 (after the instance is removed) rather than within its life.
    pub fade_out_after: bool,
    pub distortion: Option<f32>,
    /// 0 default, 6 distortion, 7 lit, 128 a material file.
    pub material: i32,
    /// `ColorInheritType`: 0 not bound, 1 when created, 2 always.
    pub color_inherit: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Rotation {
    /// Degrees about x, y, z (Effekseer turns z, then x, then y).
    Euler(Anim3),
    /// Degrees about an axis.
    Axis {
        axis: Vec3,
        angle: Anim,
    },
    Other(i32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    /// Its place in the tree: `0`, `0.2`, ...
    pub path: String,
    pub name: String,
    pub rendered: bool,
    pub draw: Draw,
    /// `None`: infinite.
    pub max_generation: Option<u32>,
    pub life: Rng,
    pub generation_time: Rng,
    pub generation_offset: Rng,
    /// `TranslationParentEffectType`, `ParentEffectType` ×2: 0 not bound,
    /// 1 when created, 2 always, 3 not bound (root).
    pub bind: [i32; 3],
    pub remove_when_life_ends: bool,
    pub remove_with_parent: bool,
    pub location: Anim3,
    pub location_type: i32,
    pub rotation: Rotation,
    pub scale: Anim3,
    pub spawn: Spawn,
    pub spawn_rotates: bool,
    /// Units per frame².
    pub gravity: Vec3,
    pub render: Render,
    pub color: Color,
    pub children: Vec<Node>,
    /// What it sets that the importer does not map.
    pub unread: Vec<String>,
}

/// The project: its top-level nodes and the editor's frame range.
#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub nodes: Vec<Node>,
    pub end_frame: f32,
}

/// The texture path a renderer names, without the editor's leftovers (an
/// empty path saved relative to the project file is the project's own name).
fn file_path(e: &El, p: &str) -> Option<String> {
    let t = e.text_at(p)?.trim().replace('\\', "/");
    let t = t.trim().to_string();
    let lower = t.to_ascii_lowercase();
    if t.is_empty() || lower.ends_with(".efkproj") || lower.ends_with(".efkefc") {
        return None;
    }
    Some(t)
}

pub fn read(root: &El) -> Result<Project, String> {
    let r = root.child("Root").ok_or("the project has no <Root>")?;
    let nodes = children(r, "")?;
    Ok(Project {
        nodes,
        end_frame: f(root, "EndFrame", 120.0).max(1.0),
    })
}

fn children(e: &El, parent: &str) -> Result<Vec<Node>, String> {
    let Some(c) = e.child("Children") else {
        return Ok(Vec::new());
    };
    c.children
        .iter()
        .filter(|n| n.name == "Node")
        .enumerate()
        .map(|(i, n)| {
            let path = if parent.is_empty() {
                i.to_string()
            } else {
                format!("{parent}.{i}")
            };
            node(n, path)
        })
        .collect()
}

const LOCATION: Kinds<'static> = Kinds {
    fixed: ("Fixed", "Location"),
    pva: ("PVA", ["Location", "Velocity", "Acceleration"]),
    easing: "Easing",
    fcurve: "LocationFCurve",
};
const ROTATION: Kinds<'static> = Kinds {
    fixed: ("Fixed", "Rotation"),
    pva: ("PVA", ["Rotation", "Velocity", "Acceleration"]),
    easing: "Easing",
    fcurve: "RotationFCurve",
};
const SCALE: Kinds<'static> = Kinds {
    fixed: ("Fixed", "Scale"),
    pva: ("PVA", ["Scale", "Velocity", "Acceleration"]),
    easing: "Easing",
    fcurve: "FCurve",
};

fn node(e: &El, path: String) -> Result<Node, String> {
    let empty = El::default();
    let mut unread = Vec::new();
    let cv = e.child("CommonValues").unwrap_or(&empty);
    let gen_block = cv.child("Generation");
    let gen_v = |name: &str| gen_block.and_then(|g| g.child(name)).or(cv.child(name));
    let max_generation = if boolean(cv, "MaxGeneration/Infinite", false) {
        None
    } else {
        Some(int(cv, "MaxGeneration/Value", 1).max(0) as u32)
    };
    let removal = cv.child("Removal");
    let remove = |new: &str, old: &str, d: bool| {
        removal.map_or_else(
            || boolean(cv, old, d),
            |r| boolean(r, new, boolean(cv, old, d)),
        )
    };
    let bind = [
        int(cv, "LocationEffectType", 2),
        int(cv, "RotationEffectType", 2),
        int(cv, "ScaleEffectType", 2),
    ];
    if matches!(bind[0], 4 | 5) {
        unread.push("location follows its parent with steering".into());
    }
    if gen_block.is_some_and(|g| int(g, "Timing", 0) != 0)
        || cv.child("TriggerParam").is_some_and(|t| {
            ["ToStartGeneration", "ToStopGeneration", "ToRemove"]
                .iter()
                .any(|k| int(t, k, 0) != 0)
        })
    {
        unread.push("triggers".into());
    }

    // Location.
    let lv = e.child("LocationValues").unwrap_or(&empty);
    let location_type = int(lv, "Type", 0);
    let location =
        anim3(lv, location_type, &LOCATION, [0, 1, 2, 3], Vec3::ZERO).unwrap_or_else(|| {
            unread.push(match location_type {
                4 => "location along a NURBS curve".into(),
                5 => "location offset towards the view".into(),
                t => format!("location type {t}"),
            });
            value::fixed3(Vec3::ZERO)
        });
    if lv
        .at("Easing")
        .is_some_and(|b| boolean(b, "IsMiddleEnabled", false))
        && location_type == 2
    {
        unread.push("easing through a middle point".into());
    }

    // Rotation.
    let rv = e.child("RotationValues").unwrap_or(&empty);
    let rt = int(rv, "Type", 0);
    let rotation = match rt {
        0 | 1 | 2 | 5 => Rotation::Euler(
            anim3(rv, rt, &ROTATION, [0, 1, 2, 5], Vec3::ZERO).expect("a known rotation type"),
        ),
        3 => {
            let b = rv.child("AxisPVA");
            let axis = v3rnd(b.and_then(|b| b.child("Axis")), Vec3::Y);
            Rotation::Axis {
                axis: Vec3::new(axis[0].c(), axis[1].c(), axis[2].c()),
                angle: Anim::Pva {
                    p: rnd(b.and_then(|b| b.child("Rotation")), 0.0),
                    v: rnd(b.and_then(|b| b.child("Velocity")), 0.0),
                    a: rnd(b.and_then(|b| b.child("Acceleration")), 0.0),
                },
            }
        }
        4 => {
            let b = rv.child("AxisEasing");
            let axis = v3rnd(b.and_then(|b| b.child("Axis")), Vec3::Y);
            let eb = b.and_then(|b| b.child("Easing"));
            Rotation::Axis {
                axis: Vec3::new(axis[0].c(), axis[1].c(), axis[2].c()),
                angle: Anim::Easing {
                    s: rnd(eb.and_then(|b| b.child("Start")), 0.0),
                    e: rnd(eb.and_then(|b| b.child("End")), 0.0),
                    ease: Ease::read(eb),
                },
            }
        }
        t => {
            unread.push(match t {
                6 => "rotation towards the viewpoint".into(),
                7 => "rotation towards the velocity".into(),
                t => format!("rotation type {t}"),
            });
            Rotation::Other(t)
        }
    };

    // Scale.
    let sv = e.child("ScalingValues").unwrap_or(&empty);
    let st = int(sv, "Type", 0);
    let scale = match st {
        0 | 1 | 2 | 5 => {
            anim3(sv, st, &SCALE, [0, 1, 2, 5], Vec3::ONE).expect("a known scale type")
        }
        3 => {
            let b = sv.child("SinglePVA");
            let a = Anim::Pva {
                p: rnd(b.and_then(|b| b.child("Scale")), 1.0),
                v: rnd(b.and_then(|b| b.child("Velocity")), 0.0),
                a: rnd(b.and_then(|b| b.child("Acceleration")), 0.0),
            };
            [a.clone(), a.clone(), a]
        }
        4 => {
            let b = sv.child("SingleEasing");
            let a = Anim::Easing {
                s: rnd(b.and_then(|b| b.child("Start")), 1.0),
                e: rnd(b.and_then(|b| b.child("End")), 1.0),
                ease: Ease::read(b),
            };
            [a.clone(), a.clone(), a]
        }
        6 => {
            let c = value::FCurve::channels(sv.child("SingleFCurve"), &["S"], 1.0);
            let a = Anim::Curve(c.into_iter().next().unwrap_or_default());
            [a.clone(), a.clone(), a]
        }
        t => {
            unread.push(format!("scale type {t}"));
            value::fixed3(Vec3::ONE)
        }
    };

    // Where instances spawn.
    let gl = e.child("GenerationLocationValues").unwrap_or(&empty);
    let spawn = match int(gl, "Type", 0) {
        1 => {
            let s = gl.child("Sphere");
            Spawn::Sphere {
                radius: rnd(s.and_then(|s| s.child("Radius")), 0.0),
                rot_x: rnd(s.and_then(|s| s.child("RotationX")), 0.0),
                rot_y: rnd(s.and_then(|s| s.child("RotationY")), 0.0),
            }
        }
        2 => {
            unread.push("spawning on a model's surface".into());
            Spawn::Model
        }
        3 => {
            let c = gl.child("Circle");
            Spawn::Circle {
                radius: rnd(c.and_then(|c| c.child("Radius")), 0.0),
                axis: c.map_or(2, |c| int(c, "AxisDirection", 2)),
                start: rnd(c.and_then(|c| c.child("AngleStart")), 0.0),
                end: rnd(c.and_then(|c| c.child("AngleEnd")), 360.0),
                division: c.map_or(8, |c| int(c, "Division", 8)).max(1),
                ordered: c.is_some_and(|c| int(c, "Type", 0) != 0),
            }
        }
        4 => {
            let l = gl.child("Line");
            Spawn::Line {
                from: v3rnd(l.and_then(|l| l.child("PositionStart")), Vec3::ZERO),
                to: v3rnd(l.and_then(|l| l.child("PositionEnd")), Vec3::ZERO),
            }
        }
        _ => Spawn::Point(v3rnd(gl.at("Point/Location"), Vec3::ZERO)),
    };
    let spawn_rotates = boolean(gl, "EffectsRotation", false);

    // Forces in the effect's space.
    let la = e.child("LocationAbsValues").unwrap_or(&empty);
    let mut gravity = Vec3::ZERO;
    match int(la, "Type", 0) {
        0 => {}
        1 => gravity = v3(la.at("Gravity/Gravity"), Vec3::ZERO),
        2 => unread.push("attraction to the target".into()),
        t => unread.push(format!("absolute location type {t}")),
    }
    if let Some(ff) = la.child("LocalForceField1").or(la.child("LocalForceField"))
        && int(ff, "Type", 0) != 0
    {
        unread.push("local force fields".into());
    }

    // Rendering common to every renderer.
    let rc = e.child("RendererCommonValues").unwrap_or(&empty);
    let dv = e.child("DrawingValues").unwrap_or(&empty);
    let draw_type = int(dv, "Type", 2);
    // An older project keeps the texture and blend on the renderer; the
    // ribbon was the stripe before 0.6.
    let old_renderer = |name: &str| match name {
        "Ribbon" => dv.child("Ribbon").or(dv.child("Stripe")),
        _ => dv.child(name),
    };
    let texture = file_path(rc, "ColorTexture").or_else(|| {
        ["Sprite", "Ribbon", "Ring", "Model"]
            .iter()
            .find_map(|r| old_renderer(r).and_then(|r| file_path(r, "ColorTexture")))
    });
    let blend = rc
        .text_at("AlphaBlend")
        .and_then(|t| t.parse().ok())
        .or_else(|| {
            let r = match draw_type {
                2 => "Sprite",
                3 => "Ribbon",
                4 => "Ring",
                _ => return None,
            };
            old_renderer(r)
                .and_then(|r| r.text_at("AlphaBlend"))
                .and_then(|t| t.parse().ok())
        })
        .unwrap_or(1);
    let uv = match int(rc, "UV", 0) {
        1 => Uv::Fixed {
            start: v2(rc.at("UVFixed/Start"), [0.0; 2]),
            size: v2(rc.at("UVFixed/Size"), [0.0; 2]),
        },
        2 => {
            let a = rc.child("UVAnimation");
            let a = a.and_then(|a| a.child("AnimationParams")).or(a);
            let p = |n: &str| a.and_then(|a| a.child(n));
            Uv::Animation {
                start: v2(p("Start"), [0.0; 2]),
                size: v2(p("Size"), [0.0; 2]),
                frame_length: if p("FrameLength").is_some_and(|l| boolean(l, "Infinite", false)) {
                    None
                } else {
                    Some(
                        p("FrameLength")
                            .map_or(1.0, |l| {
                                value::num(l.child("Value"))
                                    .or(value::num(Some(l)))
                                    .unwrap_or(1.0)
                            })
                            .max(1.0),
                    )
                },
                count: [
                    a.map_or(1, |a| int(a, "FrameCountX", 1)).max(1),
                    a.map_or(1, |a| int(a, "FrameCountY", 1)).max(1),
                ],
                loop_type: a.map_or(0, |a| int(a, "LoopType", 0)),
            }
        }
        3 => {
            let s = rc.child("UVScroll");
            Uv::Scroll {
                start: v2rnd(s.and_then(|s| s.child("Start")), [0.0; 2]),
                size: v2rnd(s.and_then(|s| s.child("Size")), [0.0; 2]),
                speed: v2rnd(s.and_then(|s| s.child("Speed")), [0.0; 2]),
            }
        }
        4 => {
            unread.push("UV over an f-curve".into());
            Uv::Curve
        }
        _ => Uv::Default,
    };
    let fade = |ty: &str, block: &str| {
        (int(rc, ty, 0) != 0).then(|| {
            let b = rc.child(block);
            (
                b.map_or(1.0, |b| f(b, "Frame", 1.0)).max(0.0),
                Ease::read(b),
            )
        })
    };
    let material = int(rc, "Material", 0);
    let distortion = (boolean(rc, "Distortion", false) || material == 6)
        .then(|| f(rc, "DistortionIntensity", 1.0));
    if material == 128 {
        unread.push("a custom material file".into());
    }
    if let Some(a) = e
        .child("AdvancedRendererCommonValuesValues")
        .or(e.child("AdvancedRendererCommonValues"))
    {
        for (k, what) in [
            ("AlphaTextureParam", "an alpha texture"),
            ("UVDistortionParam", "UV distortion"),
            ("AlphaCutoffParam", "alpha cutoff"),
            ("FalloffParam", "falloff"),
            ("BlendTextureParams", "a blend texture"),
        ] {
            if a.child(k).is_some_and(|p| boolean(p, "Enabled", false)) {
                unread.push(what.into());
            }
        }
        if a.child("SoftParticleParams")
            .is_some_and(|p| boolean(p, "Enabled", false))
        {
            unread.push("soft particles".into());
        }
    }
    for (k, what) in [
        ("CollisionsValues", "collisions"),
        ("KillRulesValues", "kill rules"),
        ("GpuParticlesValues", "GPU particles"),
    ] {
        if e.child(k)
            .is_some_and(|c| int(c, "Type", 0) != 0 || boolean(c, "IsEnabled", false))
        {
            unread.push(what.into());
        }
    }
    if e.child("SoundValues")
        .is_some_and(|s| int(s, "Type", 0) == 1)
    {
        unread.push("a sound".into());
    }
    if rc
        .child("CustomData1")
        .is_some_and(|c| int(c, "CustomData", 0) != 0)
    {
        unread.push("custom data".into());
    }

    // The renderer.
    let rendered = boolean(e, "IsRendered", true);
    let new_color = dv.child("ColorAll");
    let draw = match draw_type {
        0 => Draw::None,
        2 => {
            let s = dv.child("Sprite").unwrap_or(&empty);
            Draw::Sprite {
                billboard: int(s, "Billboard", 0),
            }
        }
        3 => {
            let r = old_renderer("Ribbon").unwrap_or(&empty);
            let w = if int(r, "Position", 0) == 1 {
                f(r, "Position_Fixed_R", 0.5) - f(r, "Position_Fixed_L", -0.5)
            } else {
                1.0
            };
            Draw::Ribbon { width: w.abs() }
        }
        4 => {
            let r = dv.child("Ring").unwrap_or(&empty);
            let loc = |name: &str, d: f32, k: usize| -> Anim {
                match int(r, name, 0) {
                    1 => {
                        let p = r.child(&format!("{name}_PVA"));
                        Anim::Pva {
                            p: v2rnd(p.and_then(|p| p.child("Location")), [d, d])[k],
                            v: v2rnd(p.and_then(|p| p.child("Velocity")), [0.0; 2])[k],
                            a: v2rnd(p.and_then(|p| p.child("Acceleration")), [0.0; 2])[k],
                        }
                    }
                    2 => {
                        let p = r.child(&format!("{name}_Easing"));
                        Anim::Easing {
                            s: v2rnd(p.and_then(|p| p.child("Start")), [d, d])[k],
                            e: v2rnd(p.and_then(|p| p.child("End")), [d, d])[k],
                            ease: Ease::read(p),
                        }
                    }
                    _ => Anim::Fixed(v2(r.at(&format!("{name}_Fixed/Location")), [d, d])[k]),
                }
            };
            let ratio = match int(r, "CenterRatio", 0) {
                1 => rnd(r.child("CenterRatio_Random"), 0.5).c(),
                2 => {
                    let b = r.child("CenterRatio_Easing");
                    (rnd(b.and_then(|b| b.child("Start")), 0.5).c()
                        + rnd(b.and_then(|b| b.child("End")), 0.5).c())
                        * 0.5
                }
                _ => f(r, "CenterRatio_Fixed", 0.5),
            };
            let angle = match int(r, "ViewingAngle", 0) {
                1 => rnd(r.child("ViewingAngle_Random"), 360.0).c(),
                2 => {
                    unread.push("a ring's arc over its life".into());
                    rnd(r.at("ViewingAngle_Easing/End"), 360.0).c()
                }
                _ => f(r, "ViewingAngle_Fixed", 360.0),
            };
            let col = |name: &str, d: [f32; 4]| -> Color {
                // A ring's colours: 0 fixed, 1 random, 2 easing.
                match int(r, name, 0) {
                    1 => Color::Random(value::color_rnd(r.child(&format!("{name}_Random")), d)),
                    2 => {
                        let b = r.child(&format!("{name}_Easing"));
                        Color::Easing {
                            s: value::color_rnd(b.and_then(|b| b.child("Start")), d),
                            e: value::color_rnd(b.and_then(|b| b.child("End")), d),
                            ease: Ease::read(b),
                        }
                    }
                    _ => Color::Fixed(value::color(r.child(&format!("{name}_Fixed")), d)),
                }
            };
            Draw::Ring(Box::new(Ring {
                billboard: int(r, "Billboard", 2),
                outer: loc("Outer", 2.0, 0),
                inner: loc("Inner", 1.0, 0),
                outer_height: loc("Outer", 0.0, 1),
                inner_height: loc("Inner", 0.0, 1),
                center_ratio: ratio.clamp(0.0, 1.0),
                viewing_angle: angle.clamp(0.0, 360.0),
                outer_color: col("OuterColor", [255.0, 255.0, 255.0, 0.0]),
                center_color: col("CenterColor", [255.0; 4]),
                inner_color: col("InnerColor", [255.0, 255.0, 255.0, 0.0]),
            }))
        }
        5 => {
            let m = dv.child("Model").unwrap_or(&empty);
            Draw::Model {
                model: file_path(m, "Model"),
                billboard: int(m, "Billboard", 2),
            }
        }
        6 => {
            let t = dv.child("Track").unwrap_or(&empty);
            Draw::Track {
                width: [
                    f(t, "TrackSizeFor_Fixed", 1.0),
                    f(t, "TrackSizeMiddle_Fixed", 1.0),
                    f(t, "TrackSizeBack_Fixed", 1.0),
                ],
            }
        }
        t => Draw::Other(t),
    };
    let d = [255.0; 4];
    let color = match (&draw, new_color) {
        (_, Some(c)) => Color::standard(Some(c), d),
        (Draw::Sprite { .. }, None) => {
            Color::prefixed(dv.child("Sprite").unwrap_or(&empty), "ColorAll", d)
        }
        (Draw::Ribbon { .. }, None) => {
            Color::prefixed(old_renderer("Ribbon").unwrap_or(&empty), "ColorAll", d)
        }
        (Draw::Model { .. }, None) => {
            Color::prefixed(dv.child("Model").unwrap_or(&empty), "Color", d)
        }
        (Draw::Track { .. }, None) => {
            Color::prefixed(dv.child("Track").unwrap_or(&empty), "ColorCenter", d)
        }
        _ => Color::WHITE,
    };
    if matches!(color, Color::Gradient) {
        unread.push("a colour gradient".into());
    }

    Ok(Node {
        name: e.text_at("Name").unwrap_or("Node").to_string(),
        rendered,
        draw,
        max_generation,
        life: rnd(cv.child("Life"), 100.0),
        generation_time: rnd(gen_v("GenerationTime"), 1.0),
        generation_offset: rnd(gen_v("GenerationTimeOffset"), 0.0),
        bind,
        remove_when_life_ends: remove("WhenLifeIsExtinct", "RemoveWhenLifeIsExtinct", true),
        remove_with_parent: remove("WhenParentIsRemoved", "RemoveWhenParentIsRemoved", false),
        location,
        location_type,
        rotation,
        scale,
        spawn,
        spawn_rotates,
        gravity,
        render: Render {
            texture,
            blend,
            uv,
            fade_in: fade("FadeInType", "FadeIn"),
            fade_out: fade("FadeOutType", "FadeOut"),
            fade_out_after: int(rc, "FadeOutType", 0) == 2,
            distortion,
            material,
            color_inherit: int(rc, "ColorInheritType", 0),
        },
        color,
        children: children(e, &path)?,
        unread,
        path,
    })
}
