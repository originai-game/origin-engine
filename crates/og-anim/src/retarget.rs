//! Humanoid retargeting: a clip made for one rig played by another with
//! other proportions, rest orientations and bone names.
//!
//! - [`HumanoidMap`]: which node of a skeleton is each of the 22 standard
//!   humanoid bones ([`HUMAN_BONES`]), detected from the node names of the
//!   common naming schemes ([`SCHEMES`]: Mixamo, VRM, Unity's humanoid,
//!   Blender Rigify, Unreal's mannequin, plain Blender names, NVIDIA's SOMA,
//!   the skeleton og.motion generates) or written in
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

mod motion;

pub use motion::{
    FootPlants, PLANT_HEIGHT, RetargetOptions, Retargeted, mirror, mirror_name, mirror_pairs,
    retarget,
};
pub(crate) use motion::{TwistBone, share_twist, twist_bones};

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
pub const SCHEMES: [&str; 7] = ["mixamo", "vrm", "unity", "rigify", "ue", "blender", "soma"];

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
        "soma" => {
            // NVIDIA's SOMA (MotionBricks, og.motion): Spine1, Spine2, Chest,
            // Neck1, LeftLeg (the thigh), LeftShin, LeftToeBase; a finger's 1
            // is its metacarpal (the thumb's is the standard one).
            let (side, rest) = side_prefix(&lower);
            let part = match rest {
                "hips" => Part::Hips,
                "spine1" => Part::Spine,
                "spine2" => Part::Chest,
                "chest" => Part::UpperChest,
                "neck1" => Part::Neck,
                "head" => Part::Head,
                "shoulder" => Part::Shoulder,
                "arm" => Part::UpperArm,
                "forearm" => Part::LowerArm,
                "hand" => Part::Hand,
                "leg" => Part::UpperLeg,
                "shin" => Part::LowerLeg,
                "foot" => Part::Foot,
                "toebase" => Part::Toes,
                r => {
                    let r = r.strip_prefix("hand")?;
                    let digit = r.find(|c: char| c.is_ascii_digit())?;
                    let (name, k) = (&r[..digit], r[digit..].parse::<usize>().ok()?);
                    let k = if name == "thumb" {
                        k
                    } else {
                        k.checked_sub(1)?
                    };
                    finger(&format!("{name}{k}"))?
                }
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
/// scheme auto                 # auto (default) | mixamo | vrm | unity | rigify | ue | blender | soma
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
