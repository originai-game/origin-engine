//! Humanoid retargeting: a clip made for one rig played by another with
//! other proportions, rest orientations and bone names.
//!
//! - [`HumanoidMap`]: which node of a skeleton is each of the 22 standard
//!   humanoid bones ([`HUMAN_BONES`]), detected from the node names of the
//!   common naming schemes ([`SCHEMES`]: Mixamo, VRM, Unity's humanoid,
//!   Blender Rigify, Unreal's mannequin, plain Blender names) or written in
//!   a `type rig` .ogt ([`RigAsset`]), which can also name the scheme and
//!   override single bones.
//! - [`retarget`]: a clip of the source rig rewritten for the target rig.
//!   Per frame, each mapped bone's model-space rotation change from the
//!   source's rest pose is applied to the target bone, after a rest
//!   correction that turns the target's rest bone direction onto the
//!   source's (so a T-posed rig plays an A-posed rig's clip); the target
//!   keeps its own bone lengths; the hips travel by the source's travel
//!   scaled by the ratio of leg heights (root motion preserved, hip height
//!   scaled); and, with `foot_ik`, each leg is solved (two bones, the
//!   knee's bend plane kept) so the foot lands where the source's foot,
//!   scaled the same way, does: a planted source foot stays planted on the
//!   target's ground.
//!
//! The cook uses it for a model's `animation { library … }` (clips of an
//! animation library glTF written into the model's `#anim` blob) and the
//! runtime for `og.anim.retarget`. Deterministic like the rest of the crate:
//! `+ - * /` and `sqrt` on f32, in node order.

use crate::asset::{Channel, ChannelPath, ClipAsset, NONE, SkeletonAsset, bind};
use crate::pose::{Pose, sample};
use crate::xform::{Trs, rotation_arc, unit};
use og_core::glam::{Quat, Vec3};
use og_core::ogt::{Document, Entry, Value};
use og_core::{Error, ErrorKind, Result};

/// The standard humanoid bones, in the order [`HumanoidMap::nodes`] holds
/// them (a parent comes before its children). The names are VRM's and
/// Unity's humanoid names.
pub const HUMAN_BONES: [&str; HUMAN_BONE_COUNT] = [
    "hips",
    "spine",
    "chest",
    "upperChest",
    "neck",
    "head",
    "leftShoulder",
    "leftUpperArm",
    "leftLowerArm",
    "leftHand",
    "rightShoulder",
    "rightUpperArm",
    "rightLowerArm",
    "rightHand",
    "leftUpperLeg",
    "leftLowerLeg",
    "leftFoot",
    "leftToes",
    "rightUpperLeg",
    "rightLowerLeg",
    "rightFoot",
    "rightToes",
    "leftThumbMetacarpal",
    "leftThumbProximal",
    "leftThumbDistal",
    "leftIndexProximal",
    "leftIndexIntermediate",
    "leftIndexDistal",
    "leftMiddleProximal",
    "leftMiddleIntermediate",
    "leftMiddleDistal",
    "leftRingProximal",
    "leftRingIntermediate",
    "leftRingDistal",
    "leftLittleProximal",
    "leftLittleIntermediate",
    "leftLittleDistal",
    "rightThumbMetacarpal",
    "rightThumbProximal",
    "rightThumbDistal",
    "rightIndexProximal",
    "rightIndexIntermediate",
    "rightIndexDistal",
    "rightMiddleProximal",
    "rightMiddleIntermediate",
    "rightMiddleDistal",
    "rightRingProximal",
    "rightRingIntermediate",
    "rightRingDistal",
    "rightLittleProximal",
    "rightLittleIntermediate",
    "rightLittleDistal",
];

/// How many standard bones there are: 22 for the body, then 15 per hand
/// (VRM 1.0's finger bones: thumb, index, middle, ring and little, three
/// each, from the hand out).
pub const HUMAN_BONE_COUNT: usize = 52;

/// The naming schemes [`HumanoidMap::detect`] tries, in tie-break order.
pub const SCHEMES: [&str; 6] = ["mixamo", "vrm", "unity", "rigify", "ue", "blender"];

const HIPS: usize = 0;
const SPINE: usize = 1;
const CHEST: usize = 2;
const UPPER_CHEST: usize = 3;
const NECK: usize = 4;
const HEAD: usize = 5;
/// The first bone of each side's arm and leg.
const ARM: [usize; 2] = [6, 10];
const LEG: [usize; 2] = [14, 18];
/// The first finger bone of each hand (the thumb's metacarpal); a hand's
/// fingers are thumb, index, middle, ring, little, three bones each.
const FINGERS: [usize; 2] = [22, 37];

/// Which bone a standard bone hangs from (`usize::MAX` for the hips).
fn parent_bone(bone: usize) -> usize {
    match bone {
        HIPS => usize::MAX,
        SPINE | 14 | 18 => HIPS,
        CHEST..=HEAD => bone - 1,
        6 | 10 => UPPER_CHEST,
        7..=9 | 11..=13 | 15..=17 | 19..=21 => bone - 1,
        _ => {
            let side = (bone >= FINGERS[1]) as usize;
            let k = bone - FINGERS[side];
            if k.is_multiple_of(3) {
                ARM[side] + 3
            } else {
                bone - 1
            }
        }
    }
}

/// The bones a bone points at, in preference order (its rest direction is
/// towards the first one mapped on both rigs).
fn children(bone: usize) -> &'static [usize] {
    // A hand points at its middle finger (else index, ring, little).
    const HAND_L: [usize; 4] = [
        FINGERS[0] + 6,
        FINGERS[0] + 3,
        FINGERS[0] + 9,
        FINGERS[0] + 12,
    ];
    const HAND_R: [usize; 4] = [
        FINGERS[1] + 6,
        FINGERS[1] + 3,
        FINGERS[1] + 9,
        FINGERS[1] + 12,
    ];
    static NEXT: [usize; HUMAN_BONE_COUNT] = {
        let mut n = [0; HUMAN_BONE_COUNT];
        let mut i = 0;
        while i < HUMAN_BONE_COUNT {
            n[i] = i + 1;
            i += 1;
        }
        n
    };
    if bone >= FINGERS[0] && bone < HUMAN_BONE_COUNT {
        let k = (bone - FINGERS[(bone >= FINGERS[1]) as usize]) % 3;
        return if k < 2 { &NEXT[bone..bone + 1] } else { &[] };
    }
    match bone {
        9 => &HAND_L,
        13 => &HAND_R,
        SPINE => &[CHEST, UPPER_CHEST, NECK],
        CHEST => &[UPPER_CHEST, NECK],
        UPPER_CHEST => &[NECK],
        NECK => &[HEAD],
        6 => &[7],
        7 => &[8],
        8 => &[9],
        10 => &[11],
        11 => &[12],
        12 => &[13],
        14 => &[15],
        15 => &[16],
        16 => &[17],
        18 => &[19],
        19 => &[20],
        20 => &[21],
        _ => &[],
    }
}

/// The index of a standard bone by name ([`HUMAN_BONES`]).
pub fn human_bone(name: &str) -> Option<usize> {
    HUMAN_BONES.iter().position(|b| *b == name)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Centre,
    Left,
    Right,
}

/// A part of the body, before its side.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Hips,
    Spine,
    Chest,
    UpperChest,
    Neck,
    Head,
    Shoulder,
    UpperArm,
    LowerArm,
    Hand,
    UpperLeg,
    LowerLeg,
    Foot,
    Toes,
    /// A finger (0 thumb … 4 little) and its bone from the hand out (0..3).
    Finger(usize, usize),
}

/// The hand a finger bone hangs from, `None` for a bone that is not a finger.
fn hand_of(bone: usize) -> Option<usize> {
    (bone >= FINGERS[0] && bone < HUMAN_BONE_COUNT).then(|| ARM[(bone >= FINGERS[1]) as usize] + 3)
}

/// The finger bone a squashed name is: an optional `hand`, `finger` or
/// `f` prefix (Mixamo's `HandIndex1`, Rigify's `f_index.01`), the finger
/// (`thumb`, `index`, `middle`, `ring`, `pinky` or `little`) and its bone
/// from the hand, 1 to 3 (`1` or `01`).
fn finger(squashed: &str) -> Option<Part> {
    const NAMES: [(&str, usize); 6] = [
        ("thumb", 0),
        ("index", 1),
        ("middle", 2),
        ("ring", 3),
        ("pinky", 4),
        ("little", 4),
    ];
    let named = |s: &str| NAMES.iter().find(|(n, _)| s.starts_with(n)).copied();
    let mut s = squashed;
    for prefix in ["hand", "finger", "f"] {
        if let Some(r) = s.strip_prefix(prefix)
            && named(r).is_some()
        {
            s = r;
            break;
        }
    }
    let (name, f) = named(s)?;
    let k = match &s[name.len()..] {
        "1" | "01" => 0,
        "2" | "02" => 1,
        "3" | "03" => 2,
        _ => return None,
    };
    Some(Part::Finger(f, k))
}

fn bone_of(side: Side, part: Part) -> Option<usize> {
    let centre = match part {
        Part::Hips => Some(HIPS),
        Part::Spine => Some(SPINE),
        Part::Chest => Some(CHEST),
        Part::UpperChest => Some(UPPER_CHEST),
        Part::Neck => Some(NECK),
        Part::Head => Some(HEAD),
        _ => None,
    };
    let offset = |base: [usize; 2], k: usize| match side {
        Side::Left => Some(base[0] + k),
        Side::Right => Some(base[1] + k),
        Side::Centre => None,
    };
    match (side, centre) {
        (Side::Centre, Some(b)) => Some(b),
        (_, Some(_)) => None,
        _ => match part {
            Part::Shoulder => offset(ARM, 0),
            Part::UpperArm => offset(ARM, 1),
            Part::LowerArm => offset(ARM, 2),
            Part::Hand => offset(ARM, 3),
            Part::UpperLeg => offset(LEG, 0),
            Part::LowerLeg => offset(LEG, 1),
            Part::Foot => offset(LEG, 2),
            Part::Toes => offset(LEG, 3),
            Part::Finger(f, k) => offset(FINGERS, 3 * f + k),
            _ => None,
        },
    }
}

/// `name` lower-cased without `_`, `-`, `.` and spaces.
fn squash(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '_' | '-' | '.' | ' '))
        .flat_map(char::to_lowercase)
        .collect()
}

/// The name after any namespace (`mixamorig:Hips`, `Armature|Hips`).
fn local_name(name: &str) -> &str {
    name.rsplit([':', '|']).next().unwrap_or(name)
}

/// Splits a `Left`/`Right` prefix off a squashed name.
fn side_prefix(s: &str) -> (Side, &str) {
    if let Some(r) = s.strip_prefix("left") {
        (Side::Left, r)
    } else if let Some(r) = s.strip_prefix("right") {
        (Side::Right, r)
    } else {
        (Side::Centre, s)
    }
}

/// Splits a side suffix (`.L`, `_L`, `.Left`, `_left`, ...) off a lower-cased name.
fn side_suffix(s: &str) -> (Side, &str) {
    for (suffix, side) in [
        (".left", Side::Left),
        ("_left", Side::Left),
        (".right", Side::Right),
        ("_right", Side::Right),
        (".l", Side::Left),
        ("_l", Side::Left),
        (".r", Side::Right),
        ("_r", Side::Right),
    ] {
        if let Some(r) = s.strip_suffix(suffix) {
            return (side, r);
        }
    }
    (Side::Centre, s)
}

/// The standard bone a node name is in `scheme`, if any.
fn classify(scheme: &str, name: &str) -> Option<usize> {
    let lower = local_name(name).to_lowercase();
    match scheme {
        "mixamo" => {
            // mixamorig:Hips, mixamorig1:LeftUpLeg, mixamorig_LeftArm, Hips.
            let full = name.to_lowercase();
            let s = match full.strip_prefix("mixamorig") {
                Some(r) => r
                    .trim_start_matches(|c: char| c.is_ascii_digit())
                    .trim_start_matches([':', '_']),
                None => lower.as_str(),
            };
            let (side, rest) = side_prefix(s);
            let part = match rest {
                "hips" => Part::Hips,
                "spine" => Part::Spine,
                "spine1" => Part::Chest,
                "spine2" => Part::UpperChest,
                "neck" => Part::Neck,
                "head" => Part::Head,
                "shoulder" => Part::Shoulder,
                "arm" => Part::UpperArm,
                "forearm" => Part::LowerArm,
                "hand" => Part::Hand,
                "upleg" => Part::UpperLeg,
                "leg" => Part::LowerLeg,
                "foot" => Part::Foot,
                "toebase" => Part::Toes,
                r => finger(r)?,
            };
            bone_of(side, part)
        }
        "vrm" => {
            // VRoid's J_Bip_C_Hips, J_Bip_L_UpperLeg, ...
            let s = lower.strip_prefix("j_bip_")?;
            let (side, rest) = match s.split_once('_')? {
                ("c", r) => (Side::Centre, r),
                ("l", r) => (Side::Left, r),
                ("r", r) => (Side::Right, r),
                _ => return None,
            };
            let part = match squash(rest).as_str() {
                "hips" => Part::Hips,
                "spine" => Part::Spine,
                "chest" => Part::Chest,
                "upperchest" => Part::UpperChest,
                "neck" => Part::Neck,
                "head" => Part::Head,
                "shoulder" => Part::Shoulder,
                "upperarm" => Part::UpperArm,
                "lowerarm" => Part::LowerArm,
                "hand" => Part::Hand,
                "upperleg" => Part::UpperLeg,
                "lowerleg" => Part::LowerLeg,
                "foot" => Part::Foot,
                "toebase" | "toes" => Part::Toes,
                r => finger(r)?,
            };
            bone_of(side, part)
        }
        "unity" => {
            // The standard names themselves: Hips, LeftUpperLeg, leftUpperLeg.
            let s = squash(local_name(name));
            HUMAN_BONES.iter().position(|b| b.to_lowercase() == s)
        }
        "rigify" => {
            // Rigify's deform bones: DEF-spine (the hips), DEF-thigh.L, ...
            let s = lower.strip_prefix("def-")?;
            let (side, rest) = side_suffix(s);
            let part = match rest {
                "spine" => Part::Hips,
                "spine.001" => Part::Spine,
                "spine.002" => Part::Chest,
                "spine.003" => Part::UpperChest,
                "spine.004" => Part::Neck,
                "spine.006" => Part::Head,
                "shoulder" => Part::Shoulder,
                "upper_arm" => Part::UpperArm,
                "forearm" => Part::LowerArm,
                "hand" => Part::Hand,
                "thigh" => Part::UpperLeg,
                "shin" => Part::LowerLeg,
                "foot" => Part::Foot,
                "toe" => Part::Toes,
                r => finger(&squash(r))?,
            };
            bone_of(side, part)
        }
        "ue" => {
            // Unreal's mannequin: pelvis, spine_01, thigh_l, calf_l, ball_l.
            let (side, rest) = match lower.as_str() {
                s if s.ends_with("_l") => (Side::Left, &s[..s.len() - 2]),
                s if s.ends_with("_r") => (Side::Right, &s[..s.len() - 2]),
                s => (Side::Centre, s),
            };
            let part = match rest {
                "pelvis" => Part::Hips,
                "spine_01" => Part::Spine,
                "spine_02" => Part::Chest,
                "spine_03" => Part::UpperChest,
                "neck_01" => Part::Neck,
                "head" => Part::Head,
                "clavicle" => Part::Shoulder,
                "upperarm" => Part::UpperArm,
                "lowerarm" => Part::LowerArm,
                "hand" => Part::Hand,
                "thigh" => Part::UpperLeg,
                "calf" => Part::LowerLeg,
                "foot" => Part::Foot,
                "ball" => Part::Toes,
                r => finger(&squash(r))?,
            };
            bone_of(side, part)
        }
        "blender" => {
            // Plain names with a side suffix: hips, spine, thigh.L, shin_R, upper_arm.L.
            let (mut side, rest) = side_suffix(&lower);
            let mut rest = squash(rest);
            if side == Side::Centre {
                let (s, r) = side_prefix(&rest);
                side = s;
                rest = r.to_string();
            }
            let part = match rest.as_str() {
                "hips" | "pelvis" => Part::Hips,
                "spine" => Part::Spine,
                "chest" => Part::Chest,
                "upperchest" => Part::UpperChest,
                "neck" => Part::Neck,
                "head" => Part::Head,
                "shoulder" | "clavicle" => Part::Shoulder,
                "upperarm" | "arm" => Part::UpperArm,
                "forearm" | "lowerarm" => Part::LowerArm,
                "hand" => Part::Hand,
                "thigh" | "upperleg" => Part::UpperLeg,
                "shin" | "calf" | "lowerleg" => Part::LowerLeg,
                "foot" => Part::Foot,
                "toe" | "toes" => Part::Toes,
                r => finger(r)?,
            };
            bone_of(side, part)
        }
        _ => None,
    }
}

/// Which node of a skeleton each standard humanoid bone is.
#[derive(Clone, Debug, PartialEq)]
pub struct HumanoidMap {
    /// The scheme the names were read in (one of [`SCHEMES`]), `custom`
    /// when a rig file named every bone it maps, `none` when nothing matched.
    pub scheme: String,
    /// Per standard bone ([`HUMAN_BONES`] order): the node index, or [`NONE`].
    pub nodes: [u32; HUMAN_BONE_COUNT],
    /// The height of the left and right foot joints above the ground when
    /// the character stands (model space, the ground at y = 0), from a rig
    /// file's `footHeight`; `None` takes the rest pose's foot heights.
    pub foot_height: Option<[f32; 2]>,
}

fn invalid(reason: String) -> Error {
    Error::new(ErrorKind::Invalid, reason)
}

/// Unity's legacy (and VRM 0.x's) thumb is Proximal, Intermediate, Distal,
/// its Proximal the metacarpal: on a side whose thumb has an `Intermediate`
/// node and no `Metacarpal`, the names are read so, Proximal as the
/// metacarpal and Intermediate as the proximal (VRM 1.0's names, which
/// have no thumb Intermediate, stay as they are).
fn unity_legacy_thumbs(skeleton: &SkeletonAsset, nodes: &mut [u32; HUMAN_BONE_COUNT]) {
    for (side, base) in [("left", FINGERS[0]), ("right", FINGERS[1])] {
        let intermediate = format!("{side}thumbintermediate");
        let Some(i) = skeleton
            .nodes
            .iter()
            .position(|n| squash(local_name(&n.name)) == intermediate)
        else {
            continue;
        };
        // The thumb's bones are base (metacarpal), base + 1, base + 2.
        if nodes[base] == NONE {
            nodes[base] = nodes[base + 1];
            nodes[base + 1] = i as u32;
        }
    }
}

impl HumanoidMap {
    /// The map `scheme` reads from the node names (the first node of each
    /// bone, in node order); an unknown scheme is refused.
    pub fn with_scheme(skeleton: &SkeletonAsset, scheme: &str) -> Result<HumanoidMap> {
        if !SCHEMES.contains(&scheme) {
            return Err(invalid(format!(
                "'{scheme}' is not a rig naming scheme ({})",
                SCHEMES.join(", ")
            )));
        }
        let mut nodes = [NONE; HUMAN_BONE_COUNT];
        for (i, n) in skeleton.nodes.iter().enumerate() {
            if let Some(b) = classify(scheme, &n.name)
                && nodes[b] == NONE
            {
                nodes[b] = i as u32;
            }
        }
        if scheme == "unity" {
            unity_legacy_thumbs(skeleton, &mut nodes);
        }
        Ok(HumanoidMap {
            scheme: scheme.to_string(),
            nodes,
            foot_height: None,
        })
    }

    /// The map of the scheme that recognises the most bones (ties go to the
    /// earlier of [`SCHEMES`]); scheme `none` and no bones when none does.
    pub fn detect(skeleton: &SkeletonAsset) -> HumanoidMap {
        let mut best = HumanoidMap {
            scheme: "none".into(),
            nodes: [NONE; HUMAN_BONE_COUNT],
            foot_height: None,
        };
        for s in SCHEMES {
            let m = HumanoidMap::with_scheme(skeleton, s).expect("a known scheme");
            if m.count() > best.count() {
                best = m;
            }
        }
        best
    }

    /// The node of the standard bone `bone` (a [`HUMAN_BONES`] name).
    pub fn node(&self, bone: &str) -> Option<u32> {
        human_bone(bone)
            .map(|b| self.nodes[b])
            .filter(|&n| n != NONE)
    }

    /// How many standard bones are mapped.
    pub fn count(&self) -> usize {
        self.nodes.iter().filter(|&&n| n != NONE).count()
    }

    /// (standard bone, node name) of every mapped bone, in [`HUMAN_BONES`] order.
    pub fn bones<'a>(&self, skeleton: &'a SkeletonAsset) -> Vec<(&'static str, &'a str)> {
        HUMAN_BONES
            .iter()
            .zip(self.nodes)
            .filter(|(_, n)| *n != NONE)
            .filter_map(|(b, n)| Some((*b, skeleton.nodes.get(n as usize)?.name.as_str())))
            .collect()
    }
}

/// A `type rig` .ogt: the humanoid map of one skeleton, by the scheme its
/// names follow (`scheme auto` detects it) and bones named outright.
///
/// ```text
/// type rig
/// version 1
/// ---
/// scheme auto                 # auto (default) | mixamo | vrm | unity | rigify | ue | blender
/// hips "Root_Pelvis"          # a standard bone and the node it is (overrides the scheme)
/// leftToes none               # this rig has no such bone
/// footHeight 0.08             # the ankles above the ground standing (or `footHeight <left> <right>`)
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RigAsset {
    /// The scheme, or `None` for `auto`.
    pub scheme: Option<String>,
    /// `footHeight`: the left and right foot joints' height above the
    /// ground (y = 0 of model space) when the character stands, for a rig
    /// whose rest pose does not stand on the ground; `None`: the rest pose's.
    pub foot_height: Option<[f32; 2]>,
    /// Bones named outright: (standard bone index, node name or `None` for absent, line).
    pub bones: Vec<(usize, Option<String>, usize)>,
}

fn parse_err(e: &Entry, reason: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Parse, format!("line {}: {reason}", e.line))
}

impl RigAsset {
    /// Parses `type rig` text; errors name the line.
    pub fn parse(text: &str) -> Result<RigAsset> {
        let doc = Document::parse(text)?;
        if doc.ty != "rig" || doc.version != 1 {
            return Err(Error::new(
                ErrorKind::Parse,
                format!(
                    "a rig is `type rig` `version 1`, not `type {}` `version {}`",
                    doc.ty, doc.version
                ),
            ));
        }
        let mut rig = RigAsset::default();
        for e in &doc.body.entries {
            if e.key == "footHeight" {
                if rig.foot_height.is_some() {
                    return Err(parse_err(e, "'footHeight' is given twice"));
                }
                if !(1..=2).contains(&e.values.len()) || e.block.is_some() {
                    return Err(parse_err(
                        e,
                        "'footHeight' takes one height, or the left's and the right's",
                    ));
                }
                let mut h = [0f32; 2];
                for (i, v) in h.iter_mut().enumerate() {
                    *v = e
                        .f32(i.min(e.values.len() - 1))
                        .map_err(|x| parse_err(e, x.reason()))?;
                    if !(v.is_finite() && (-10.0..=10.0).contains(v)) {
                        return Err(parse_err(e, "a foot height is metres, -10..=10"));
                    }
                }
                rig.foot_height = Some(h);
                continue;
            }
            if e.values.len() != 1 || e.block.is_some() {
                return Err(parse_err(e, format!("'{}' takes one value", e.key)));
            }
            if e.key == "scheme" {
                let w = e.word(0).map_err(|x| parse_err(e, x.reason()))?;
                rig.scheme = match w {
                    "auto" => None,
                    s if SCHEMES.contains(&s) => Some(s.to_string()),
                    s => {
                        return Err(parse_err(
                            e,
                            format!("scheme '{s}' is not auto or one of {}", SCHEMES.join(", ")),
                        ));
                    }
                };
                continue;
            }
            let Some(b) = human_bone(&e.key) else {
                return Err(parse_err(
                    e,
                    format!(
                        "'{}' is not `scheme`, `footHeight` or a humanoid bone ({})",
                        e.key,
                        HUMAN_BONES.join(", ")
                    ),
                ));
            };
            if rig.bones.iter().any(|(x, _, _)| *x == b) {
                return Err(parse_err(e, format!("'{}' is named twice", e.key)));
            }
            let node = match &e.values[0] {
                Value::String(s) => Some(s.clone()),
                Value::Word(w) if w == "none" => None,
                _ => {
                    return Err(parse_err(
                        e,
                        format!("'{}' takes a node name in quotes or `none`", e.key),
                    ));
                }
            };
            rig.bones.push((b, node, e.line));
        }
        Ok(rig)
    }

    /// The map of `skeleton`: its scheme's (or the detected one), then the
    /// bones named outright. A named node the skeleton lacks is refused.
    pub fn map(&self, skeleton: &SkeletonAsset) -> Result<HumanoidMap> {
        let mut m = match &self.scheme {
            Some(s) => HumanoidMap::with_scheme(skeleton, s)?,
            None => HumanoidMap::detect(skeleton),
        };
        for (b, node, line) in &self.bones {
            m.nodes[*b] = match node {
                None => NONE,
                Some(name) => skeleton.node(name).ok_or_else(|| {
                    invalid(format!(
                        "line {line}: {} is '{name}', which the skeleton does not have",
                        HUMAN_BONES[*b]
                    ))
                })?,
            };
        }
        if m.count() > 0 && self.scheme.is_none() && m.scheme == "none" {
            m.scheme = "custom".into();
        }
        m.foot_height = self.foot_height;
        Ok(m)
    }

    /// The canonical text (parses back equal, lines aside).
    pub fn to_text(&self) -> String {
        let mut s = String::from("type rig\nversion 1\n---\n");
        s.push_str(&format!(
            "scheme {}\n",
            self.scheme.as_deref().unwrap_or("auto")
        ));
        for (b, node, _) in &self.bones {
            match node {
                Some(n) => s.push_str(&format!("{} {n:?}\n", HUMAN_BONES[*b])),
                None => s.push_str(&format!("{} none\n", HUMAN_BONES[*b])),
            }
        }
        match self.foot_height {
            Some([l, r]) if l == r => s.push_str(&format!("footHeight {l}\n")),
            Some([l, r]) => s.push_str(&format!("footHeight {l} {r}\n")),
            None => {}
        }
        s
    }
}

/// The node name on the other side: the side tokens the naming schemes
/// read swapped (a `Left`/`Right` prefix after any namespace, as Mixamo,
/// Unity and plain names write it; VRM's `_L_`/`_R_`; a `.L`, `_R`, `.left`,
/// `_l` suffix, as Blender, Rigify and Unreal write it), keeping the case;
/// `None` for a name with no side.
pub fn mirror_name(name: &str) -> Option<String> {
    let cut = name.rfind([':', '|']).map_or(0, |i| i + 1);
    let (ns, local) = name.split_at(cut);
    for (a, b) in [("Left", "Right"), ("left", "right"), ("LEFT", "RIGHT")] {
        for (from, to) in [(a, b), (b, a)] {
            if let Some(rest) = local.strip_prefix(from) {
                return Some(format!("{ns}{to}{rest}"));
            }
        }
    }
    for (from, to) in [
        ("_L_", "_R_"),
        ("_R_", "_L_"),
        ("_l_", "_r_"),
        ("_r_", "_l_"),
    ] {
        if let Some(i) = local.find(from) {
            return Some(format!(
                "{ns}{}{to}{}",
                &local[..i],
                &local[i + from.len()..]
            ));
        }
    }
    for sep in ['.', '_', '-', ' '] {
        for (a, b) in [
            ("Left", "Right"),
            ("left", "right"),
            ("LEFT", "RIGHT"),
            ("L", "R"),
            ("l", "r"),
        ] {
            for (from, to) in [(a, b), (b, a)] {
                if let Some(rest) = local.strip_suffix(from)
                    && rest.ends_with(sep)
                {
                    return Some(format!("{ns}{rest}{to}"));
                }
            }
        }
    }
    None
}

/// Each node's counterpart on the other side ([`mirror_name`]): itself for
/// a node without a side, [`NONE`] for a sided node whose counterpart the
/// skeleton lacks (or whose counterpart does not name it back).
pub fn mirror_pairs(skeleton: &SkeletonAsset) -> Vec<u32> {
    (0..skeleton.nodes.len())
        .map(|i| match mirror_name(&skeleton.nodes[i].name) {
            None => i as u32,
            Some(other) => skeleton
                .node(&other)
                .filter(|&k| {
                    mirror_name(&skeleton.nodes[k as usize].name).as_deref()
                        == Some(skeleton.nodes[i].name.as_str())
                })
                .unwrap_or(NONE),
        })
        .collect()
}

/// `v` reflected in the model's x = 0 plane.
fn reflect(v: Vec3) -> Vec3 {
    Vec3::new(-v.x, v.y, v.z)
}

/// A rotation reflected in the model's x = 0 plane (`M q M`).
fn reflect_rotation(q: Quat) -> Quat {
    Quat::from_xyzw(q.x, -q.y, -q.z, q.w)
}

/// `clip` (made for `skeleton`) played on the other side: every frame
/// reflected in the model's x = 0 plane, each node taking its
/// counterpart's motion ([`mirror_pairs`]). Rotations move in model space
/// (a node's model rotation is its counterpart's change from rest,
/// reflected, applied to its own rest), so left and right bone frames need
/// not mirror each other; translations are the counterpart's travel from
/// rest, reflected (root motion's x and yaw flip); scales and blendshape
/// weights are copied, visibility channels move to the counterpart.
/// Assumes a symmetric rest pose. A node gets a channel
/// when it or its counterpart is animated; a sided node without a
/// counterpart gets none (it keeps its rest pose). Mirroring twice gives
/// the clip back. Same name, duration, rate and markers.
pub fn mirror(skeleton: &SkeletonAsset, clip: &ClipAsset) -> Result<ClipAsset> {
    clip.validate()?;
    let pairs = mirror_pairs(skeleton);
    let binding = bind(skeleton, clip);
    let n = skeleton.nodes.len();
    let animated = |node: u32, path: ChannelPath| {
        node != NONE
            && clip
                .channels
                .iter()
                .zip(&binding)
                .any(|(c, &b)| b == node && c.path == path)
    };
    let wants = |path: ChannelPath| -> Vec<bool> {
        (0..n)
            .map(|i| pairs[i] != NONE && (animated(i as u32, path) || animated(pairs[i], path)))
            .collect()
    };
    let (rot, pos, scl) = (
        wants(ChannelPath::Rotation),
        wants(ChannelPath::Translation),
        wants(ChannelPath::Scale),
    );
    let rest_pose = skeleton.rest_pose();
    let rest = model(skeleton, &rest_pose);
    let mut samples: Vec<[Vec<f32>; 3]> = vec![Default::default(); n];
    for i in 0..clip.frames {
        let t = (i as f32 / clip.rate).min(clip.duration);
        let mut src = Pose::rest(skeleton);
        sample(skeleton, clip, &binding, t, NONE, &mut src);
        let ms = model(skeleton, &src.nodes);
        let mut pose = rest_pose.clone();
        let mut mt: Vec<Trs> = Vec::with_capacity(n);
        for j in 0..n {
            let parent = parent_model(skeleton, &mt, j);
            let k = pairs[j] as usize;
            if rot[j] {
                let change = ms[k].rotation * rest[k].rotation.conjugate();
                let want = unit(reflect_rotation(change) * rest[j].rotation);
                pose[j].rotation = unit(parent.rotation.conjugate() * want);
                samples[j][1].extend_from_slice(&pose[j].rotation.to_array());
            }
            if pos[j] {
                let at = rest[j].translation + reflect(ms[k].translation - rest[k].translation);
                pose[j].translation = parent.inverse().transform_point(at);
                samples[j][0].extend_from_slice(&pose[j].translation.to_array());
            }
            if scl[j] {
                pose[j].scale = src.nodes[k].scale;
                samples[j][2].extend_from_slice(&pose[j].scale.to_array());
            }
            mt.push(parent.mul(&pose[j]));
        }
    }
    let mut channels = Vec::new();
    for (j, s) in samples.into_iter().enumerate() {
        for (path, lanes, data) in [
            (ChannelPath::Translation, 3, &s[0]),
            (ChannelPath::Rotation, 4, &s[1]),
            (ChannelPath::Scale, 3, &s[2]),
        ] {
            if !data.is_empty() {
                channels.push(Channel {
                    node: j as u32,
                    node_name: skeleton.nodes[j].name.clone(),
                    path,
                    step: false,
                    lanes,
                    samples: data.clone().into(),
                });
            }
        }
    }
    for (c, &b) in clip.channels.iter().zip(&binding) {
        if c.path == ChannelPath::Weights && b != NONE {
            channels.push(Channel {
                node: b,
                node_name: skeleton.nodes[b as usize].name.clone(),
                ..c.clone()
            });
        }
    }
    channels.extend(crate::visibility::mirrored(
        skeleton, clip, &binding, &pairs,
    ));
    let out = ClipAsset {
        channels,
        ..clip.clone()
    };
    out.validate()?;
    Ok(out)
}

/// How far above its lowest a foot may be and still be planted, in the
/// skeleton's model units: 5 mm.
pub const PLANT_HEIGHT: f32 = 0.005;

/// Where a foot meets the ground over a clip: the one notion of "planted"
/// the engine has (retargeting's contact checks, `og.FootLock`). The foot
/// node's model-space height (the clip sampled over the rest pose, root
/// motion not removed) at each of the clip's frames; a frame is planted when
/// the foot is within [`PLANT_HEIGHT`] of its lowest over the clip.
#[derive(Clone, Debug, PartialEq)]
pub struct FootPlants {
    /// The foot's model-space height at each frame.
    pub heights: Vec<f32>,
    /// The lowest of them (the clip's ground for this foot).
    pub lowest: f32,
}

impl FootPlants {
    /// The plants of node `foot` of `skeleton` over `clip` (made for it).
    pub fn new(skeleton: &SkeletonAsset, clip: &ClipAsset, foot: u32) -> FootPlants {
        let binding = bind(skeleton, clip);
        let heights: Vec<f32> = (0..clip.frames)
            .map(|i| {
                let t = (i as f32 / clip.rate).min(clip.duration);
                let mut pose = Pose::rest(skeleton);
                sample(skeleton, clip, &binding, t, NONE, &mut pose);
                model(skeleton, &pose.nodes)[foot as usize].translation.y
            })
            .collect();
        let lowest = heights.iter().copied().fold(f32::MAX, f32::min);
        FootPlants { heights, lowest }
    }

    /// Whether the foot is planted at frame `frame`.
    pub fn planted(&self, frame: usize) -> bool {
        self.heights
            .get(frame)
            .is_some_and(|&h| h <= self.lowest + PLANT_HEIGHT)
    }

    /// Whether the foot is planted at clip time `t` of `clip` (the clip the
    /// plants were made from): its height interpolated between the frames
    /// around `t`, as the frames are.
    pub fn planted_at(&self, clip: &ClipAsset, t: f32) -> bool {
        let p = clip.frame_pos(t);
        let (Some(&a), Some(&b)) = (self.heights.get(p.i0), self.heights.get(p.i1)) else {
            return false;
        };
        a + (b - a) * p.frac <= self.lowest + PLANT_HEIGHT
    }
}

/// How [`retarget`] treats the legs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RetargetOptions {
    /// Solve each leg so the foot lands where the source's foot does,
    /// scaled by the leg ratio (planted feet stay on the target's ground).
    pub foot_ik: bool,
}

impl Default for RetargetOptions {
    fn default() -> Self {
        RetargetOptions { foot_ik: true }
    }
}

/// What [`retarget`] made.
#[derive(Clone, Debug, PartialEq)]
pub struct Retargeted {
    /// The clip for the target skeleton (same name, duration, rate and markers).
    pub clip: ClipAsset,
    /// The leg-height ratio target / source the hips' travel was scaled by.
    pub scale: f32,
    /// Standard bones mapped on both rigs.
    pub bones: u32,
    /// Legs solved for the feet (0, 1 or 2).
    pub legs: u32,
    /// Twist bones of the target driven from their limb segments.
    pub twists: u32,
}

/// The height of the foot of `leg` (0 left, 1 right) above the ground the
/// character stands on: the rig's `footHeight`, else the rest pose's.
fn foot_y(rest: &[Trs], map: &HumanoidMap, leg: usize) -> f32 {
    map.foot_height
        .map_or(rest[map.nodes[LEG[leg] + 2] as usize].translation.y, |h| {
            h[leg]
        })
}

/// A twist bone of the target (a node no standard bone maps whose name
/// says `twist`, or a Rigify deform segment `….001`) on the limb segment
/// from standard bone `a` to the next one, `c`.
#[derive(Clone, Debug)]
pub(crate) struct TwistBone {
    pub(crate) node: usize,
    /// The segment's bone and the next bone's node; `a`'s parent node.
    pub(crate) a: usize,
    pub(crate) c: usize,
    parent: Option<usize>,
    /// The segment's direction in `a`'s frame (towards `c` at rest).
    axis: Vec3,
    /// How far along the segment the twist bone sits, 0 at `a` to 1 at `c`.
    at: f32,
}

/// Whether a node name is a twist bone's.
fn twist_name(name: &str) -> bool {
    let lower = local_name(name).to_lowercase();
    lower.contains("twist")
        || (lower.starts_with("def-")
            && lower.len() > 4
            && lower[lower.len() - 4..].starts_with('.')
            && lower[lower.len() - 3..].bytes().all(|b| b.is_ascii_digit()))
}

/// The twist of rotation `q` about unit `axis` (swing-twist), in radians.
fn twist_angle(q: Quat, axis: Vec3) -> f32 {
    let p = Vec3::new(q.x, q.y, q.z).dot(axis);
    let (p, w) = if q.w < 0.0 { (-p, -q.w) } else { (p, q.w) };
    2.0 * og_core::math::atan2(p, w)
}

/// The twist bones of `skeleton` (mapped by `map`), on the upper and
/// lower arm and leg segments whose next bone is mapped.
pub(crate) fn twist_bones(
    skeleton: &SkeletonAsset,
    map: &HumanoidMap,
    rest: &[Trs],
) -> Vec<TwistBone> {
    let mapped = |n: usize| map.nodes.iter().position(|&m| m as usize == n);
    let mut out = Vec::new();
    for (n, node) in skeleton.nodes.iter().enumerate() {
        if mapped(n).is_some() || !twist_name(&node.name) {
            continue;
        }
        // The nearest mapped ancestor is the segment.
        let mut p = node.parent;
        let a_bone = loop {
            if p == NONE {
                break None;
            }
            if let Some(b) = mapped(p as usize) {
                break Some(b);
            }
            p = skeleton.nodes[p as usize].parent;
        };
        let Some(ab) = a_bone else { continue };
        let segment = [
            ARM[0] + 1,
            ARM[0] + 2,
            ARM[1] + 1,
            ARM[1] + 2,
            LEG[0],
            LEG[0] + 1,
        ]
        .into_iter()
        .chain([LEG[1], LEG[1] + 1])
        .any(|b| b == ab);
        if !segment || map.nodes[ab + 1] == NONE {
            continue;
        }
        let (a, c) = (map.nodes[ab] as usize, map.nodes[ab + 1] as usize);
        let along = rest[c].translation - rest[a].translation;
        let len2 = along.length_squared();
        if len2 < 1e-12 {
            continue;
        }
        let local = rest[a].inverse().transform_point(rest[c].translation);
        let axis = local / local.length();
        let at = ((rest[n].translation - rest[a].translation).dot(along) / len2).clamp(0.0, 1.0);
        let parent = match skeleton.nodes[a].parent {
            NONE => None,
            q => Some(q as usize),
        };
        out.push(TwistBone {
            node: n,
            a,
            c,
            parent,
            axis,
            at,
        });
    }
    out
}

/// Turns each of `twists` in `pose` (whose model space is `fk`; `rest` is
/// the rest pose's) to a linear share of its segment's twist, from none
/// relative to the segment's parent at its head to the next bone's at its
/// end. Every node `keep` names keeps its model rotation from `fk`; the
/// others keep their local rotation.
pub(crate) fn share_twist(
    skeleton: &SkeletonAsset,
    twists: &[&TwistBone],
    rest: &[Trs],
    fk: &[Trs],
    pose: &mut [Trs],
    keep: impl Fn(usize) -> bool,
) {
    let mut m2: Vec<Trs> = Vec::with_capacity(pose.len());
    for n in 0..pose.len() {
        let p = parent_model(skeleton, &m2, n);
        if let Some(tw) = twists.iter().find(|t| t.node == n) {
            let rel = |of: &[Trs], x: usize, y: Option<usize>| -> Quat {
                let base = y.map_or(Quat::IDENTITY, |y| of[y].rotation);
                base.conjugate() * of[x].rotation
            };
            let tau_a = twist_angle(
                rel(rest, tw.a, tw.parent).conjugate() * rel(fk, tw.a, tw.parent),
                tw.axis,
            );
            let tau_c = twist_angle(
                rel(fk, tw.c, Some(tw.a)) * rel(rest, tw.c, Some(tw.a)).conjugate(),
                tw.axis,
            );
            let theta = tw.at * tau_c - (1.0 - tw.at) * tau_a;
            let q = og_core::math::quat_from_axis_angle(tw.axis, theta);
            let want = unit(m2[tw.a].rotation * q * rel(rest, n, Some(tw.a)));
            pose[n].rotation = unit(p.rotation.conjugate() * want);
        } else if keep(n) {
            pose[n].rotation = unit(p.rotation.conjugate() * fk[n].rotation);
        }
        m2.push(p.mul(&pose[n]));
    }
}

/// The height of the hips above the feet at rest (model space), or of the
/// hips alone when no foot is mapped.
fn leg_height(rest: &[Trs], map: &HumanoidMap) -> f32 {
    let hips = rest[map.nodes[HIPS] as usize].translation.y;
    let feet: Vec<f32> = [0, 1]
        .into_iter()
        .filter(|&l| map.nodes[LEG[l] + 2] != NONE)
        .map(|l| foot_y(rest, map, l))
        .collect();
    if feet.is_empty() {
        hips
    } else {
        hips - feet.iter().sum::<f32>() / feet.len() as f32
    }
}

/// Model space of a pose (the skeleton's own composition).
fn model(skeleton: &SkeletonAsset, pose: &[Trs]) -> Vec<Trs> {
    skeleton.model_space(pose)
}

/// The parent's model transform of node `n` (identity at a root).
fn parent_model(skeleton: &SkeletonAsset, m: &[Trs], n: usize) -> Trs {
    match skeleton.nodes[n].parent {
        NONE => Trs::IDENTITY,
        p => m[p as usize],
    }
}

/// Turns node `n` (in `pose`) so that its model rotation becomes `rotation`.
fn set_model_rotation(
    skeleton: &SkeletonAsset,
    pose: &mut [Trs],
    m: &[Trs],
    n: usize,
    rotation: Quat,
) {
    let p = parent_model(skeleton, m, n);
    pose[n].rotation = unit(p.rotation.conjugate() * rotation);
}

/// Rewrites `clip` (made for `source`) for `target`. Both maps need the
/// hips; bones mapped on one rig only are left out, and target nodes no
/// mapped bone covers keep their rest pose. A target bone gets a channel
/// when the clip animates its source bone (and both legs' bones when their
/// feet are solved), so a partial clip (a wave) stays partial. Visibility
/// channels follow their bone, else their node's name.
pub fn retarget(
    source: &SkeletonAsset,
    source_map: &HumanoidMap,
    target: &SkeletonAsset,
    target_map: &HumanoidMap,
    clip: &ClipAsset,
    options: &RetargetOptions,
) -> Result<Retargeted> {
    for (map, which, skeleton) in [
        (source_map, "source", source),
        (target_map, "target", target),
    ] {
        if map.nodes[HIPS] == NONE {
            return Err(invalid(format!(
                "the {which} skeleton has no hips the engine recognises (scheme '{}'); name them in a `type rig` .ogt (`hips \"<node>\"`)",
                map.scheme
            )));
        }
        if let Some(&n) = map
            .nodes
            .iter()
            .find(|&&n| n != NONE && n as usize >= skeleton.nodes.len())
        {
            return Err(invalid(format!(
                "the {which} map names node {n}, past the skeleton's {} nodes",
                skeleton.nodes.len()
            )));
        }
    }
    clip.validate()?;
    let binding = bind(source, clip);
    let animated = |node: u32, path: Option<ChannelPath>| {
        clip.channels
            .iter()
            .zip(&binding)
            .any(|(c, &b)| b == node && path.is_none_or(|p| c.path == p))
    };
    let both = |b: usize| source_map.nodes[b] != NONE && target_map.nodes[b] != NONE;

    let s_rest = model(source, &source.rest_pose());
    let t_rest_pose = target.rest_pose();
    let t_rest = model(target, &t_rest_pose);
    let (hs, ht) = (
        leg_height(&s_rest, source_map),
        leg_height(&t_rest, target_map),
    );
    let k = if hs.abs() > 1e-6 && ht.abs() > 1e-6 {
        ht / hs
    } else {
        1.0
    };

    // The rest correction of each bone: the target's rest bone direction
    // turned onto the source's; inherited by bones with no child, none for
    // the hips.
    let mut correction = [Quat::IDENTITY; HUMAN_BONE_COUNT];
    for b in 1..HUMAN_BONE_COUNT {
        if !both(b) {
            continue;
        }
        let (sb, tb) = (source_map.nodes[b] as usize, target_map.nodes[b] as usize);
        correction[b] = match children(b).iter().find(|&&c| both(c)) {
            Some(&c) => {
                let (sc, tc) = (source_map.nodes[c] as usize, target_map.nodes[c] as usize);
                rotation_arc(
                    t_rest[tc].translation - t_rest[tb].translation,
                    s_rest[sc].translation - s_rest[sb].translation,
                )
            }
            None => {
                let mut p = parent_bone(b);
                while p != usize::MAX && !both(p) {
                    p = parent_bone(p);
                }
                if p == usize::MAX {
                    Quat::IDENTITY
                } else {
                    correction[p]
                }
            }
        };
    }

    // Which bones get channels, and which legs are solved.
    let mut emit = [false; HUMAN_BONE_COUNT];
    for (b, e) in emit.iter_mut().enumerate() {
        *e = both(b) && animated(source_map.nodes[b], None);
    }
    let hips_travel = animated(source_map.nodes[HIPS], Some(ChannelPath::Translation));
    let mut legs: Vec<usize> = Vec::new();
    if options.foot_ik {
        for &l in &LEG {
            let chain = [l, l + 1, l + 2];
            if chain.iter().all(|&b| both(b)) && (emit[HIPS] || chain.iter().any(|&b| emit[b])) {
                for b in chain {
                    emit[b] = true;
                }
                legs.push(l);
            }
        }
    }
    // The hips move when the clip moves them or a solved leg may lower them.
    let moves_hips = hips_travel || !legs.is_empty();
    if moves_hips {
        emit[HIPS] = true;
    }
    let th = target_map.nodes[HIPS] as usize;
    let sh = source_map.nodes[HIPS] as usize;
    // Twist bones on segments the clip moves (the segment or its next bone).
    let twists: Vec<TwistBone> = twist_bones(target, target_map, &t_rest)
        .into_iter()
        .filter(|t| {
            emit.iter().enumerate().any(|(b, &e)| {
                e && (target_map.nodes[b] as usize == t.a || target_map.nodes[b] as usize == t.c)
            })
        })
        .collect();
    let mut twist_rotations: Vec<Vec<f32>> = vec![Vec::new(); twists.len()];

    // Per frame: the target pose, recorded into the emitted channels.
    let emitted: Vec<usize> = {
        let mut v: Vec<usize> = (0..HUMAN_BONE_COUNT).filter(|&b| emit[b]).collect();
        og_core::sort::sort_by_key(&mut v, |&b| target_map.nodes[b]);
        v
    };
    let mut rotations: Vec<Vec<f32>> = vec![Vec::new(); emitted.len()];
    let mut travel: Vec<f32> = Vec::new();
    let mut by_node: Vec<Option<usize>> = vec![None; target.nodes.len()];
    for &b in &emitted {
        by_node[target_map.nodes[b] as usize] = Some(b);
    }
    for i in 0..clip.frames {
        let t = (i as f32 / clip.rate).min(clip.duration);
        let mut ps = Pose::rest(source);
        sample(source, clip, &binding, t, NONE, &mut ps);
        let ms = model(source, &ps.nodes);
        let desired = |b: usize| -> Quat {
            let s = source_map.nodes[b] as usize;
            let tb = target_map.nodes[b] as usize;
            unit(
                ms[s].rotation
                    * s_rest[s].rotation.conjugate()
                    * correction[b]
                    * t_rest[tb].rotation,
            )
        };
        let hips_at = t_rest[th].translation + (ms[sh].translation - s_rest[sh].translation) * k;
        // Forward kinematics in node order: each emitted bone at its
        // desired model rotation, the hips at their scaled travel.
        let mut pt = t_rest_pose.clone();
        let mut mt: Vec<Trs> = Vec::with_capacity(pt.len());
        for n in 0..pt.len() {
            let p = parent_model(target, &mt, n);
            if let Some(b) = by_node[n] {
                let want = match hand_of(b) {
                    // A finger turns from its hand as the source's does from
                    // its own, so a hand pose holds on an arm the clip
                    // leaves alone (A-posed under a T-posed source's clip).
                    Some(h) if both(h) => {
                        let hand = target_map.nodes[h] as usize;
                        unit(mt[hand].rotation * desired(h).conjugate() * desired(b))
                    }
                    _ => desired(b),
                };
                pt[n].rotation = unit(p.rotation.conjugate() * want);
            }
            if n == th && moves_hips {
                pt[n].translation = p.inverse().transform_point(hips_at);
            }
            mt.push(p.mul(&pt[n]));
        }
        // A leg too short for its foot's goal (a straight source leg on a
        // rig with other proportions) lowers the hips until it reaches.
        // On the ground the rigs stand on: each foot's height above it is
        // the rig's `footHeight`, else its rest pose's.
        let goal = |l: usize| {
            let side = (l == LEG[1]) as usize;
            let sf = source_map.nodes[l + 2] as usize;
            let ft = target_map.nodes[l + 2] as usize;
            let mut base = t_rest[ft].translation;
            base.y = foot_y(&t_rest, target_map, side);
            let mut from = s_rest[sf].translation;
            from.y = foot_y(&s_rest, source_map, side);
            base + (ms[sf].translation - from) * k
        };
        let mut drop = 0f32;
        for &l in &legs {
            let (up, lo, ft) = (
                target_map.nodes[l] as usize,
                target_map.nodes[l + 1] as usize,
                target_map.nodes[l + 2] as usize,
            );
            let (a, b, c) = (mt[up].translation, mt[lo].translation, mt[ft].translation);
            let reach = (b - a).length() + (c - b).length() - 1e-4;
            let v = goal(l) - a;
            if v.length() > reach {
                // The hips at a + d·y reach the goal: |v - d·y| = reach.
                let disc = v.y * v.y - (v.length_squared() - reach * reach);
                let d = if disc >= 0.0 { v.y + disc.sqrt() } else { v.y };
                drop = drop.min(d);
            }
        }
        if drop < 0.0 {
            let p = parent_model(target, &mt, th);
            pt[th].translation = p.inverse().transform_point(hips_at + Vec3::Y * drop);
            mt = model(target, &pt);
        }
        // The legs: the knee bent in its plane so the foot reaches the
        // source's foot travel, scaled; then the foot turned back to its
        // transferred rotation.
        for &l in &legs {
            let (up, lo, ft) = (
                target_map.nodes[l] as usize,
                target_map.nodes[l + 1] as usize,
                target_map.nodes[l + 2] as usize,
            );
            let goal = goal(l);
            let (a, b, c) = (mt[up].translation, mt[lo].translation, mt[ft].translation);
            let (l1, l2) = ((b - a).length(), (c - b).length());
            let to = goal - a;
            let dist = to.length();
            if l1 < 1e-6 || l2 < 1e-6 || dist < 1e-6 {
                continue;
            }
            let dir = to / dist;
            let (reach_min, reach_max) = ((l1 - l2).abs(), l1 + l2);
            let d = dist.clamp(reach_min + 1e-6, reach_max - 1e-6).max(1e-6);
            let hint = b - a;
            let mut bend = hint - dir * hint.dot(dir);
            if bend.length_squared() < 1e-12 {
                // A straight leg: bend forward (+z), as knees do.
                bend = Vec3::Z - dir * dir.z;
            }
            let bend = bend / bend.length();
            let x = (l1 * l1 - l2 * l2 + d * d) / (2.0 * d);
            let h = (l1 * l1 - x * x).max(0.0).sqrt();
            let knee = a + dir * x + bend * h;
            let tip = a + dir * d;
            let r_up = unit(rotation_arc(b - a, knee - a) * mt[up].rotation);
            set_model_rotation(target, &mut pt, &mt, up, r_up);
            let mt2 = model(target, &pt);
            let (b2, c2) = (mt2[lo].translation, mt2[ft].translation);
            let r_lo = unit(rotation_arc(c2 - b2, tip - b2) * mt2[lo].rotation);
            set_model_rotation(target, &mut pt, &mt2, lo, r_lo);
            let mt3 = model(target, &pt);
            set_model_rotation(target, &mut pt, &mt3, ft, desired(l + 2));
            mt = model(target, &pt);
        }
        // Twist bones: a linear share of the segment's twist, from none
        // relative to the segment's parent at its head to the next bone's at
        // its end; every other node keeps its model rotation.
        if !twists.is_empty() {
            let all: Vec<&TwistBone> = twists.iter().collect();
            share_twist(target, &all, &t_rest, &mt, &mut pt, |n| {
                by_node[n].is_some()
            });
            for (j, tw) in twists.iter().enumerate() {
                let r = pt[tw.node].rotation;
                twist_rotations[j].extend_from_slice(&[r.x, r.y, r.z, r.w]);
            }
        }
        for (k, &b) in emitted.iter().enumerate() {
            let r = pt[target_map.nodes[b] as usize].rotation;
            rotations[k].extend_from_slice(&[r.x, r.y, r.z, r.w]);
        }
        if moves_hips {
            travel.extend_from_slice(&pt[th].translation.to_array());
        }
    }
    let mut channels: Vec<Channel> = Vec::new();
    for (k, &b) in emitted.iter().enumerate() {
        let n = target_map.nodes[b];
        if n as usize == th && moves_hips {
            channels.push(Channel {
                node: n,
                node_name: target.nodes[n as usize].name.clone(),
                path: ChannelPath::Translation,
                step: false,
                lanes: 3,
                samples: std::mem::take(&mut travel).into(),
            });
        }
        channels.push(Channel {
            node: n,
            node_name: target.nodes[n as usize].name.clone(),
            path: ChannelPath::Rotation,
            step: false,
            lanes: 4,
            samples: std::mem::take(&mut rotations[k]).into(),
        });
    }
    for (tw, samples) in twists.iter().zip(twist_rotations) {
        channels.push(Channel {
            node: tw.node as u32,
            node_name: target.nodes[tw.node].name.clone(),
            path: ChannelPath::Rotation,
            step: false,
            lanes: 4,
            samples: samples.into(),
        });
    }
    channels.extend(crate::visibility::retargeted(
        source, source_map, target, target_map, clip, &binding,
    ));
    let out = ClipAsset {
        name: clip.name.clone(),
        duration: clip.duration,
        rate: clip.rate,
        source_rate: clip.source_rate,
        frames: clip.frames,
        channels,
        events: clip.events.clone(),
    };
    out.validate()?;
    Ok(Retargeted {
        clip: out,
        scale: k,
        bones: (0..HUMAN_BONE_COUNT).filter(|&b| both(b)).count() as u32,
        legs: legs.len() as u32,
        twists: twists.len() as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_of_each_scheme_land_on_their_bones() {
        let cases: &[(&str, &str, &str)] = &[
            ("mixamo", "mixamorig:LeftUpLeg", "leftUpperLeg"),
            ("mixamo", "mixamorig1:Spine1", "chest"),
            ("mixamo", "mixamorig_RightToeBase", "rightToes"),
            ("vrm", "J_Bip_L_LowerArm", "leftLowerArm"),
            ("vrm", "J_Bip_C_UpperChest", "upperChest"),
            ("unity", "RightUpperLeg", "rightUpperLeg"),
            ("unity", "leftHand", "leftHand"),
            ("rigify", "DEF-spine", "hips"),
            ("rigify", "DEF-spine.006", "head"),
            ("rigify", "DEF-shin.R", "rightLowerLeg"),
            ("ue", "calf_l", "leftLowerLeg"),
            ("ue", "ball_r", "rightToes"),
            ("ue", "pelvis", "hips"),
            ("blender", "thigh.L", "leftUpperLeg"),
            ("blender", "Forearm_R", "rightLowerArm"),
            ("blender", "upper_arm.left", "leftUpperArm"),
        ];
        for (scheme, name, bone) in cases {
            assert_eq!(
                classify(scheme, name).map(|b| HUMAN_BONES[b]),
                Some(*bone),
                "{scheme} {name}"
            );
        }
        // Not bones: a twist bone, Rigify's control bones, a centre name with a side.
        for (scheme, name) in [
            ("rigify", "DEF-thigh.L.001"),
            ("rigify", "thigh.L"),
            ("ue", "thigh_twist_01_l"),
            ("blender", "head.L"),
        ] {
            assert_eq!(classify(scheme, name), None, "{scheme} {name}");
        }
    }
}
