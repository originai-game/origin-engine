//! What exporters read: a scene of named parts with materials, joints and clips, checked, and posed by its joints.

use super::*;

/// How a [`Material`]'s `color` and `emissive` are given (three.js's
/// colour spaces).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorSpace {
    /// sRGB components, as a person picks them (three.js
    /// `SRGBColorSpace`); written to glTF converted to linear.
    #[default]
    Srgb,
    /// Linear components, as glTF's factors are (three.js
    /// `LinearSRGBColorSpace`); written as they are. An import gives this,
    /// so a file's factors go out unchanged.
    Linear,
}

/// What a part is made of: glTF's metallic-roughness model with
/// three.js's `MeshStandardMaterial` names. The zero value is three.js's
/// default material (white, roughness 1, not metal, opaque).
///
/// Every map is sampled at the solid's [`Solid::uvs`] (glTF `texCoord`
/// 0); a part with a map and no UVs is [`ExportError::BadTexture`], as is
/// a map that is not PNG or JPEG.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Material {
    /// The material's name in the file; empty takes the part's name.
    pub name: String,
    /// Base colour, components in 0..=1 in `color_space` (three.js
    /// `color`); `None` is white.
    pub color: Option<[f64; 3]>,
    /// 0 dielectric, 1 metal (three.js `metalness`).
    pub metalness: f64,
    /// 0 mirror, 1 matte (three.js `roughness`); `None` is 1.
    pub roughness: Option<f64>,
    /// Light given off, components in 0..=1 in `color_space` (three.js
    /// `emissive`).
    pub emissive: [f64; 3],
    /// 1 opaque, 0 invisible (three.js `opacity`); `None` is 1. Below 1
    /// the material blends, unless `transparent` says otherwise.
    pub opacity: Option<f64>,
    /// Base-colour texture (three.js `map`).
    pub map: Option<Texture>,
    /// How `color` and `emissive` are given.
    pub color_space: ColorSpace,
    /// Multiplies `emissive` (three.js `emissiveIntensity`); `None` is 1.
    /// Above 1 it is written with `KHR_materials_emissive_strength`.
    pub emissive_intensity: Option<f64>,
    /// Blending (glTF `alphaMode` `BLEND`) or not (`OPAQUE`); `None`
    /// blends when `opacity` is below 1 (three.js `transparent`).
    pub transparent: Option<bool>,
    /// Texels whose alpha is below this are cut out (glTF `MASK` with
    /// this `alphaCutoff`; three.js `alphaTest`). Overrides `transparent`.
    pub alpha_test: Option<f64>,
    /// Both sides drawn (glTF `doubleSided`; three.js `side: DoubleSide`).
    pub double_sided: bool,
    /// glTF's `metallicRoughnessTexture`: roughness in green, metalness in
    /// blue, multiplied by `roughness` and `metalness` (three.js sets it
    /// as both `roughnessMap` and `metalnessMap`).
    pub metalness_map: Option<Texture>,
    /// A tangent-space normal map (three.js `normalMap`).
    pub normal_map: Option<Texture>,
    /// The normal map's strength (glTF `normalTexture.scale`, three.js
    /// `normalScale`); `None` is 1.
    pub normal_scale: Option<f64>,
    /// Emissive texture, multiplied by `emissive` (three.js
    /// `emissiveMap`).
    pub emissive_map: Option<Texture>,
    /// Ambient occlusion in red (glTF `occlusionTexture`, three.js
    /// `aoMap`).
    pub ao_map: Option<Texture>,
    /// The occlusion's strength (glTF `occlusionTexture.strength`, three.js
    /// `aoMapIntensity`); `None` is 1.
    pub ao_map_intensity: Option<f64>,
    /// A clear layer over the surface, 0 none to 1
    /// (`KHR_materials_clearcoat`'s factor; three.js `clearcoat`).
    pub clearcoat: f64,
    /// That layer's roughness, 0 mirror to 1 (three.js
    /// `clearcoatRoughness`).
    pub clearcoat_roughness: f64,
    /// The colour of the soft light cloth gives at grazing angles,
    /// linear (`KHR_materials_sheen`'s colour; three.js `sheenColor`
    /// with `sheen` 1). Black is none.
    pub sheen: [f64; 3],
    /// How wide that light spreads, 0 to 1 (three.js `sheenRoughness`).
    pub sheen_roughness: f64,
    /// How much of the light that is not reflected passes through, 0
    /// to 1 (`KHR_materials_transmission`; three.js `transmission`).
    pub transmission: f64,
    /// A thin film's colours over the reflection, 0 none to 1
    /// (`KHR_materials_iridescence`'s factor; three.js `iridescence`).
    pub iridescence: f64,
}

/// The kinds of map a [`Material`] carries, in the order reports list
/// them: three.js's names.
pub const MAP_KINDS: [&str; 5] = [
    "map",
    "metalness_map",
    "normal_map",
    "emissive_map",
    "ao_map",
];

impl Material {
    /// Each map the material has, with its kind ([`MAP_KINDS`]).
    pub fn maps(&self) -> Vec<(&'static str, &Texture)> {
        self.map_slots()
            .into_iter()
            .filter_map(|(k, t)| Some((k, t.as_ref()?)))
            .collect()
    }

    /// Each map slot, filled or not, with its kind.
    pub fn map_slots(&self) -> [(&'static str, &Option<Texture>); 5] {
        [
            (MAP_KINDS[0], &self.map),
            (MAP_KINDS[1], &self.metalness_map),
            (MAP_KINDS[2], &self.normal_map),
            (MAP_KINDS[3], &self.emissive_map),
            (MAP_KINDS[4], &self.ao_map),
        ]
    }

    /// Each map slot, mutably.
    pub fn maps_mut(&mut self) -> [(&'static str, &mut Option<Texture>); 5] {
        [
            (MAP_KINDS[0], &mut self.map),
            (MAP_KINDS[1], &mut self.metalness_map),
            (MAP_KINDS[2], &mut self.normal_map),
            (MAP_KINDS[3], &mut self.emissive_map),
            (MAP_KINDS[4], &mut self.ao_map),
        ]
    }

    /// Whether a map of this kind holds colour (sRGB: base colour and
    /// emissive) rather than data (metalness-roughness, normals,
    /// occlusion).
    pub fn is_color_map(kind: &str) -> bool {
        kind == "map" || kind == "emissive_map"
    }
}

/// How a joint moves its part (`docs/research.md` §1.5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JointKind {
    /// Turns about the axis within a range; with no range it is written
    /// `continuous`.
    #[default]
    Revolute,
    /// Turns about the axis without end.
    Continuous,
    /// Slides along the axis.
    Prismatic,
    /// Moves every way: no axis, no range.
    Floating,
    /// Does not move: a named mount point, written `fixed`, with no axis,
    /// range or mimic (URDF's `fixed`).
    Fixed,
}

/// A joint's value follows another's: `multiplier` × the leader's value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mimic {
    /// The leading joint's name. It must be declared before this one.
    pub joint: String,
    /// This joint's value per unit of the leader's, in kernel units
    /// (radians or millimetres).
    pub multiplier: f64,
}

/// How a part moves on its parent.
///
/// Kernel units: millimetres and radians. The GLB carries SI units.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Joint {
    /// Unique among the scene's joints.
    pub name: String,
    /// How it moves.
    pub kind: JointKind,
    /// The pivot, in the world. The joint's node sits here.
    pub pivot: Frame,
    /// The axis in the pivot's own coordinates; zero is the pivot's +Z.
    pub axis: DVec3,
    /// The joint's pose, from rest (0): the part is written moved by it
    /// from where it was built. A follower ignores it and takes its
    /// leader's value × its multiplier.
    pub value: f64,
    /// `(lower, upper)`, containing 0. A follower with none takes its
    /// leader's range × its multiplier.
    pub range: Option<(f64, f64)>,
    /// The joint this one follows.
    pub mimic: Option<Mimic>,
}

/// One part: a solid, in the world, with its material and how it hangs.
#[derive(Clone, Debug)]
pub struct Part<'a> {
    /// Unique among the scene's parts: the node's and the mesh's name.
    pub name: String,
    /// Its geometry in the world at rest, every joint at 0 (millimetres,
    /// Z up).
    pub solid: &'a Solid,
    /// What it is made of.
    pub material: Material,
    /// The part it hangs on, declared before it; empty for the scene's
    /// root.
    pub parent: String,
    /// How it moves on its parent; `None` is welded to it.
    pub joint: Option<Joint>,
    /// Write `TANGENT` (on a solid with UVs), as [`crate::tools::PlainGlb`]
    /// does: the frame a baked normal map was baked in.
    pub tangents: bool,
}

impl<'a> Part<'a> {
    /// A part with three.js's default material, welded to the root.
    pub fn new(name: &str, solid: &'a Solid) -> Part<'a> {
        Part {
            name: name.to_string(),
            solid,
            material: Material::default(),
            parent: String::new(),
            joint: None,
            tangents: false,
        }
    }
}

/// The parts to export or preview.
#[derive(Clone, Debug, Default)]
pub struct Scene<'a> {
    /// The scene's name in the file.
    pub name: String,
    /// Parents before children.
    pub parts: Vec<Part<'a>>,
}

/// Why a scene could not be written.
#[derive(Clone, Debug, PartialEq)]
pub enum ExportError {
    /// Two parts share a name.
    DuplicatePart(String),
    /// Two joints share a name.
    DuplicateJoint(String),
    /// A part names a parent that is not declared before it.
    UnknownParent {
        /// The part.
        part: String,
        /// The parent it names.
        parent: String,
    },
    /// A joint follows one that is not declared before it.
    UnknownLeader {
        /// The following joint.
        joint: String,
        /// The leader it names.
        leader: String,
    },
    /// A value was NaN or infinite (where, named).
    NotFinite(String),
    /// A joint's range is empty or does not contain 0.
    BadRange(String),
    /// A joint's axis or pivot is degenerate.
    BadAxis(String),
    /// A preview asked for more than it draws: the limit that was passed
    /// and what was asked.
    TooLarge(String),
    /// A clip cannot be written: which, and why.
    BadClip {
        /// The clip's name.
        clip: String,
        /// What is wrong with it.
        why: String,
    },
    /// A floating joint was given an axis, a range or a mimic.
    FloatingConstrained(String),
    /// A fixed joint was given an axis, a range, a mimic or a value.
    FixedConstrained(String),
    /// A part's skin cannot be written.
    BadSkin {
        /// The part.
        part: String,
        /// Why.
        why: String,
    },
    /// A part's texture cannot be written: not PNG or JPEG, or the part
    /// has no UVs to sample it with.
    BadTexture {
        /// The part.
        part: String,
        /// Why.
        why: String,
    },
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::DuplicatePart(n) => write!(f, "two parts are named {n:?}"),
            ExportError::DuplicateJoint(n) => write!(f, "two joints are named {n:?}"),
            ExportError::UnknownParent { part, parent } => write!(
                f,
                "part {part:?} hangs on {parent:?}, which is not a part declared before it"
            ),
            ExportError::UnknownLeader { joint, leader } => write!(
                f,
                "joint {joint:?} follows {leader:?}, which is not a joint declared before it"
            ),
            ExportError::NotFinite(w) => write!(f, "{w} is not finite"),
            ExportError::BadRange(j) => write!(
                f,
                "joint {j:?}: its range must run from low to high and contain 0 (the rest pose)"
            ),
            ExportError::BadClip { clip, why } => write!(f, "clip {clip:?}: {why}"),
            ExportError::TooLarge(why) => write!(f, "the preview is too large: {why}"),
            ExportError::BadAxis(j) => {
                write!(f, "joint {j:?}: its axis or pivot frame is degenerate")
            }
            ExportError::FloatingConstrained(j) => write!(
                f,
                "joint {j:?}: a floating joint moves every way: it takes no axis, range or mimic"
            ),
            ExportError::FixedConstrained(j) => write!(
                f,
                "joint {j:?}: a fixed joint does not move: it takes no axis, range, mimic or value"
            ),
            ExportError::BadSkin { part, why } => write!(f, "part {part:?}: its skin: {why}"),
            ExportError::BadTexture { part, why } => {
                write!(f, "part {part:?}: its texture: {why}")
            }
        }
    }
}

impl std::error::Error for ExportError {}

#[doc(hidden)]
pub fn finite(v: f64, what: impl Fn() -> String) -> Result<f64, ExportError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(ExportError::NotFinite(what()))
    }
}

/// A joint after checks, in SI units and glTF space.
pub struct Checked {
    /// The joint's name.
    pub name: String,
    /// Its glTF kind (`revolute`, `prismatic`, ...).
    pub kind: &'static str,
    /// The joint's node: the pivot after the joint's motion.
    pub node: Rigid,
    /// The part's node under it: the pivot's inverse.
    pub inverse: Rigid,
    /// Its unit axis in glTF space (none for fixed and floating).
    pub axis: Option<DVec3>,
    /// Its value, in radians or metres.
    pub value: f64,
    /// Its range, in SI units.
    pub span: Option<(f64, f64)>,
    /// The joint it mimics, and the multiplier.
    pub mimic: Option<(String, f64)>,
    /// For clips, in kernel units: the pivot, the unit axis in the
    /// pivot's own coordinates (none for fixed and floating), whether it
    /// slides, its range, and the leader it follows with the multiplier.
    pub pivot: Rigid,
    /// The unit axis in the pivot's coordinates.
    pub local: Option<DVec3>,
    /// Whether it slides.
    pub slides: bool,
    /// Its range in kernel units.
    pub kernel_span: Option<(f64, f64)>,
    /// The leader it follows, and the multiplier.
    pub follows: Option<(String, f64)>,
}

/// One keyframe of a [`Clip`]: a time and the joints' values then.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClipKey {
    /// Seconds from the clip's start; keys are in rising order.
    pub time: f64,
    /// Joint name and value (radians, or millimetres for a slide), as
    /// [`Joint::value`]. A joint a key leaves out keeps its own track:
    /// only the keys that name it move it.
    pub pose: Vec<(String, f64)>,
}

/// A named motion of a scene's joints over time, written to the GLB as a
/// glTF animation ([`glb_clips`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Clip {
    /// Unique among the clips.
    pub name: String,
    /// The keyframes, by time.
    pub keys: Vec<ClipKey>,
}

/// Metres per kernel unit of a joint's value: millimetres for a slide,
/// radians otherwise.
pub fn si(kind: JointKind) -> f64 {
    if kind == JointKind::Prismatic {
        METRES
    } else {
        1.0
    }
}

/// A rigid motion in kernel space: `p ↦ x·p.x + y·p.y + z·p.z + t`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rigid {
    /// Where the x axis goes.
    pub x: DVec3,
    /// Where the y axis goes.
    pub y: DVec3,
    /// Where the z axis goes.
    pub z: DVec3,
    /// The translation.
    pub t: DVec3,
}

impl Rigid {
    #[doc(hidden)]
    pub const IDENTITY: Rigid = Rigid {
        x: DVec3::X,
        y: DVec3::Y,
        z: DVec3::Z,
        t: DVec3::ZERO,
    };

    #[doc(hidden)]
    pub fn of(f: &Frame) -> Rigid {
        Rigid {
            x: f.x,
            y: f.y,
            z: f.z,
            t: f.origin,
        }
    }

    #[doc(hidden)]
    pub fn rotate(&self, v: DVec3) -> DVec3 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    #[doc(hidden)]
    pub fn apply(&self, p: DVec3) -> DVec3 {
        self.rotate(p) + self.t
    }

    /// `self ∘ other`: `other` first.
    pub fn then(&self, other: &Rigid) -> Rigid {
        Rigid {
            x: self.rotate(other.x),
            y: self.rotate(other.y),
            z: self.rotate(other.z),
            t: self.apply(other.t),
        }
    }

    #[doc(hidden)]
    pub fn inverse(&self) -> Rigid {
        // (R, t)⁻¹ = (Rᵀ, −Rᵀ t).
        let tr = |v: DVec3| DVec3::new(self.x.dot(v), self.y.dot(v), self.z.dot(v));
        Rigid {
            x: tr(DVec3::X),
            y: tr(DVec3::Y),
            z: tr(DVec3::Z),
            t: -tr(self.t),
        }
    }

    /// Turn by `angle` radians about the unit axis `k` through the origin.
    pub fn turn(k: DVec3, angle: f64) -> Rigid {
        let (sn, cs) = trig::sin_cos(angle);
        let r = |v: DVec3| v * cs + k.cross(v) * sn + k * (k.dot(v) * (1.0 - cs));
        Rigid {
            x: r(DVec3::X),
            y: r(DVec3::Y),
            z: r(DVec3::Z),
            t: DVec3::ZERO,
        }
    }

    /// The same motion in glTF space (metres, Y up): translation and
    /// rotation (x, y, z, w).
    pub fn gltf(&self) -> (DVec3, [f64; 4]) {
        let down = |v: DVec3| DVec3::new(v.x, -v.z, v.y);
        let col = |e: DVec3| y_up(self.rotate(down(e)));
        (
            y_up(self.t) * METRES,
            quaternion(col(DVec3::X), col(DVec3::Y), col(DVec3::Z)),
        )
    }
}

/// The rotation whose columns are `x`, `y`, `z` as a unit quaternion
/// (x, y, z, w), by Shepperd's method: `+ - * /` and `sqrt` only.
pub fn quaternion(x: DVec3, y: DVec3, z: DVec3) -> [f64; 4] {
    let (m00, m11, m22) = (x.x, y.y, z.z);
    let trace = m00 + m11 + m22;
    let q = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [(y.z - z.y) / s, (z.x - x.z) / s, (x.y - y.x) / s, 0.25 * s]
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        [0.25 * s, (y.x + x.y) / s, (z.x + x.z) / s, (y.z - z.y) / s]
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        [(y.x + x.y) / s, 0.25 * s, (z.y + y.z) / s, (z.x - x.z) / s]
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        [(z.x + x.z) / s, (z.y + y.z) / s, 0.25 * s, (x.y - y.x) / s]
    };
    // One sign for every host: w >= 0.
    let q = if q[3] < 0.0 { q.map(|c| -c) } else { q };
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    q.map(|c| c / len)
}

/// Checks a scene as every tool does before using it, and places each
/// part: its joints' motions, composed from the root, in kernel space.
pub fn check(scene: &Scene) -> Result<Vec<Rigid>, ExportError> {
    Ok(check_joints(scene)?.into_iter().map(|(_, w)| w).collect())
}

#[doc(hidden)]
pub type Leader = (JointKind, Option<(f64, f64)>, f64);

#[doc(hidden)]
pub fn check_joints(scene: &Scene) -> Result<Vec<(Option<Checked>, Rigid)>, ExportError> {
    // Each part's placement so far, by name.
    let mut parts: BTreeMap<&str, Rigid> = BTreeMap::new();
    let mut joints: BTreeMap<String, Leader> = BTreeMap::new();
    let mut out = Vec::with_capacity(scene.parts.len());
    for p in &scene.parts {
        if !p.parent.is_empty() && !parts.contains_key(p.parent.as_str()) {
            return Err(ExportError::UnknownParent {
                part: p.name.clone(),
                parent: p.parent.clone(),
            });
        }
        if parts.contains_key(p.name.as_str()) {
            return Err(ExportError::DuplicatePart(p.name.clone()));
        }
        let above = if p.parent.is_empty() {
            Rigid::IDENTITY
        } else {
            parts[p.parent.as_str()]
        };
        for v in &p.solid.positions {
            for c in [v.x, v.y, v.z] {
                finite(c, || format!("a position of part {:?}", p.name))?;
            }
        }
        let Some(j) = &p.joint else {
            parts.insert(p.name.as_str(), above);
            out.push((None, above));
            continue;
        };
        let jn = || j.name.clone();
        if joints.contains_key(&j.name) {
            return Err(ExportError::DuplicateJoint(j.name.clone()));
        }
        let f = &j.pivot;
        for c in [f.origin, f.x, f.y, f.z]
            .iter()
            .flat_map(|v| [v.x, v.y, v.z])
            .chain([j.axis.x, j.axis.y, j.axis.z, j.value])
        {
            finite(c, || format!("joint {:?}", j.name))?;
        }
        // The pivot must be a rotation: orthonormal and right-handed.
        let ortho = (f.x.length() - 1.0).abs() < 1e-9
            && (f.y.length() - 1.0).abs() < 1e-9
            && (f.z.length() - 1.0).abs() < 1e-9
            && f.x.dot(f.y).abs() < 1e-9
            && f.x.cross(f.y).dot(f.z) > 0.999_999;
        if !ortho {
            return Err(ExportError::BadAxis(jn()));
        }
        let floating = j.kind == JointKind::Floating;
        if floating
            && (j.axis != DVec3::ZERO || j.range.is_some() || j.mimic.is_some() || j.value != 0.0)
        {
            return Err(ExportError::FloatingConstrained(jn()));
        }
        let fixed = j.kind == JointKind::Fixed;
        if fixed
            && (j.axis != DVec3::ZERO || j.range.is_some() || j.mimic.is_some() || j.value != 0.0)
        {
            return Err(ExportError::FixedConstrained(jn()));
        }
        let local = if floating || fixed {
            None
        } else {
            let a = if j.axis == DVec3::ZERO {
                DVec3::Z
            } else {
                j.axis
            };
            let len = a.length();
            if len == 0.0 || !len.is_finite() {
                return Err(ExportError::BadAxis(jn()));
            }
            Some(a / len)
        };
        let axis = local.map(y_up);
        if let Some((lo, hi)) = j.range {
            finite(lo, || format!("joint {:?}'s range", j.name))?;
            finite(hi, || format!("joint {:?}'s range", j.name))?;
            if !(lo < hi && lo <= 0.0 && 0.0 <= hi) {
                return Err(ExportError::BadRange(jn()));
            }
        }
        let mut span = j.range;
        let mut value = j.value;
        let mut mimic = None;
        if let Some(m) = &j.mimic {
            finite(m.multiplier, || format!("joint {:?}'s multiplier", j.name))?;
            let Some(&(lead_kind, lead_span, lead_value)) = joints.get(&m.joint) else {
                return Err(ExportError::UnknownLeader {
                    joint: j.name.clone(),
                    leader: m.joint.clone(),
                });
            };
            if span.is_none() {
                span = lead_span.map(|(lo, hi)| {
                    let (a, b) = (lo * m.multiplier, hi * m.multiplier);
                    if a <= b { (a, b) } else { (b, a) }
                });
            }
            value = lead_value * m.multiplier;
            mimic = Some((m.joint.clone(), m.multiplier * si(j.kind) / si(lead_kind)));
        }
        joints.insert(j.name.clone(), (j.kind, span, value));
        let kind = match j.kind {
            JointKind::Revolute if span.is_none() => "continuous",
            JointKind::Revolute => "revolute",
            JointKind::Continuous => "continuous",
            JointKind::Prismatic => "prismatic",
            JointKind::Floating => "floating",
            JointKind::Fixed => "fixed",
        };
        // A continuous joint has no bounds.
        let span = if kind == "continuous" { None } else { span };
        // The motion, in the pivot's own coordinates.
        let motion = match (j.kind, local) {
            (JointKind::Prismatic, Some(a)) => Rigid {
                t: a * value,
                ..Rigid::IDENTITY
            },
            (_, Some(a)) => Rigid::turn(a, value),
            (_, None) => Rigid::IDENTITY,
        };
        let pivot = Rigid::of(f);
        let node = pivot.then(&motion);
        let inverse = pivot.inverse();
        let world = above.then(&node).then(&inverse);
        parts.insert(p.name.as_str(), world);
        let k = si(j.kind);
        out.push((
            Some(Checked {
                name: j.name.clone(),
                kind,
                node,
                inverse,
                axis,
                value: value * k,
                span: span.map(|(lo, hi)| (lo * k, hi * k)),
                mimic,
                pivot,
                local,
                slides: j.kind == JointKind::Prismatic,
                kernel_span: span,
                follows: j.mimic.as_ref().map(|m| (m.joint.clone(), m.multiplier)),
            }),
            world,
        ));
    }
    Ok(out)
}

/// Where [`obj_at`] and [`stl_at`] put jointed parts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Placement {
    /// As built: every joint at 0, whatever its value (the zero value).
    #[default]
    Rest,
    /// Moved by the joints' values (followers by their leaders'), as the
    /// GLB's nodes and the preview place them. The file's header says
    /// `posed`.
    Pose,
}

/// A part's name, its positions in metres (kernel axes) and its triangles.
pub type Metres<'s> = (&'s str, Vec<DVec3>, &'s [[u32; 3]]);

/// Each part's motion from rest at `at`, after the scene's checks: `None`
/// where no non-zero joint value moves it (at rest, every part).
pub fn motions(scene: &Scene, at: Placement) -> Result<Vec<Option<Rigid>>, ExportError> {
    let checked = check_joints(scene)?;
    let mut moved: BTreeMap<&str, Option<Rigid>> = BTreeMap::new();
    let mut out = Vec::with_capacity(scene.parts.len());
    for (p, (c, _)) in scene.parts.iter().zip(&checked) {
        let above = moved.get(p.parent.as_str()).copied().flatten();
        let mine = match c {
            Some(c) if at == Placement::Pose && c.value != 0.0 => Some(
                above
                    .unwrap_or(Rigid::IDENTITY)
                    .then(&c.node)
                    .then(&c.inverse),
            ),
            _ => above,
        };
        moved.insert(p.name.as_str(), mine);
        out.push(mine);
    }
    Ok(out)
}

/// Each part's solid at the scene's pose (millimetres, kernel axes), as
/// [`obj_at`] with [`Placement::Pose`] places it: a part that no non-zero
/// joint value moves is its solid, bit for bit; the others are moved as
/// the preview moves them (names, surfaces and tags kept).
///
/// Errors as [`glb`] does on an invalid scene.
pub fn posed(scene: &Scene) -> Result<Vec<Solid>, ExportError> {
    Ok(scene
        .parts
        .iter()
        .zip(motions(scene, Placement::Pose)?)
        .map(|(p, m)| {
            let mut s = p.solid.clone();
            if let Some(m) = m {
                s.map_positions(|v| m.apply(v), false);
            }
            s
        })
        .collect())
}

/// Every part's triangles in metres, after the scene's checks, at `at`.
pub fn meshes<'s>(scene: &'s Scene, at: Placement) -> Result<Vec<Metres<'s>>, ExportError> {
    Ok(scene
        .parts
        .iter()
        .zip(motions(scene, at)?)
        .map(|(p, m)| {
            let positions = p
                .solid
                .positions
                .iter()
                .map(|&v| m.map_or(v, |m| m.apply(v)) * METRES)
                .collect();
            (p.name.as_str(), positions, p.solid.triangles.as_slice())
        })
        .collect())
}
