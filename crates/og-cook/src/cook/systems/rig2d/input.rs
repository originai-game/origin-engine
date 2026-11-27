//! Checked author data resolves names once into the runtime's native rig.
use super::*;
use crate::cook::source_schemas::media::{clips::Filter, rig2d as a};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) struct Prepared {
    pub rig: Rig2dAsset,
    pub conform: Option<crate::cook::source_schemas::media::tiles::TilesetConform>,
    pub atlas: Option<super::super::texture::Image>,
    pub notes: Vec<String>,
    pub files: Vec<PathBuf>,
    pub hash: Vec<u8>,
}

fn filter(value: Option<Filter>) -> SheetFilter {
    match value {
        Some(Filter::Nearest) => SheetFilter::Nearest,
        _ => SheetFilter::Linear,
    }
}

impl Prepared {
    pub fn new(input: a::Rig2d, root: &Path, id: &str) -> Result<Self, String> {
        let mut notes = Vec::new();
        let mut atlas = None;
        let mut files = Vec::new();
        let mut hash = Vec::new();
        let (rig, conform) = match input {
            a::Rig2d::Authored(input) => {
                let names = input.armatures.keys().cloned().collect::<Vec<_>>();
                let top = armature(input.rig, &names)?;
                let rig = Rig2dAsset {
                    texture: input.texture.id,
                    size: [0, 0],
                    filter: filter(input.filter),
                    unit: input.unit.unwrap_or(1.0),
                    bones: top.bones,
                    slots: top.slots,
                    attachments: top.attachments,
                    iks: top.iks,
                    transforms: top.transforms,
                    paths: top.paths,
                    clips: top.clips,
                    skins: top.skins,
                    armatures: input
                        .armatures
                        .into_iter()
                        .map(|(name, input)| {
                            let mut a = armature(input, &names)?;
                            a.name = name;
                            Ok(a)
                        })
                        .collect::<Result<_, String>>()?,
                };
                (rig, input.conform)
            }
            a::Rig2d::Dragonbones(input) => {
                let ske = serde_json::from_str(&input.dragonbones)
                    .map_err(|e| format!("dragonbones: {e}"))?;
                let atlas =
                    serde_json::from_str(&input.atlas).map_err(|e| format!("atlas: {e}"))?;
                let mut rig = dragonbones(&ske, &atlas, &mut notes)?;
                rig.texture = input.texture.id;
                rig.unit = input.unit.unwrap_or(1.0);
                rig.filter = filter(input.filter);
                (rig, input.conform)
            }
            a::Rig2d::Parts(input) => {
                if !input.parts.starts_with("assets/") && !input.parts.starts_with("sources/") {
                    return Err("parts must name a canonical path under assets/ or sources/".into());
                }
                let path = crate::cook::source_path::checked(root, &input.parts)
                    .map_err(|e| format!("parts '{}': {e}", input.parts))?;
                let inputs = if path.is_dir() {
                    files.push(path.clone());
                    autorig::part_files(&path)
                        .into_iter()
                        .map(|f| format!("{}/{f}", input.parts))
                        .collect::<Vec<_>>()
                } else {
                    vec![input.parts.clone()]
                };
                let mut captured = BTreeMap::new();
                for file in inputs {
                    let path = crate::cook::source_path::checked(root, &file)
                        .map_err(|e| format!("parts '{file}': {e}"))?;
                    let bytes = std::fs::read(&path).map_err(|e| format!("{file}: {e}"))?;
                    hash.extend_from_slice(file.as_bytes());
                    hash.push(0);
                    hash.extend_from_slice(&super::super::hash_bytes(&bytes).to_le_bytes());
                    files.push(path);
                    captured.insert(file, bytes);
                }
                let (rig, image, made) = autorig::build(id, &input, &captured)?;
                notes.extend(made);
                atlas = Some(image);
                (rig, input.conform)
            }
        };
        rig.validate().map_err(|e| e.to_string())?;
        Ok(Self {
            rig,
            conform,
            atlas,
            notes,
            files,
            hash,
        })
    }
}

fn index(names: &[String], name: &str, kind: &str) -> Result<usize, String> {
    names
        .iter()
        .position(|s| s == name)
        .ok_or_else(|| format!("no {kind} '{name}'"))
}

fn armature(input: a::Armature, children: &[String]) -> Result<Armature2d, String> {
    let bones = input
        .bones
        .iter()
        .map(|b| b.name.clone())
        .collect::<Vec<_>>();
    let slots = input
        .slots
        .iter()
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    let attachments = input.attachments.keys().cloned().collect::<Vec<_>>();
    let iks = input.iks.iter().map(|s| s.name.clone()).collect::<Vec<_>>();
    let bone = |n: &str| index(&bones, n, "bone");
    let slot = |n: &str| index(&slots, n, "slot");
    let attachment = |n: &str| index(&attachments, n, "attachment");
    Ok(Armature2d {
        name: String::new(),
        bones: input
            .bones
            .into_iter()
            .map(|b| {
                let [x, y] = b.at.unwrap_or([0.0; 2]);
                Ok(Bone2d {
                    parent: b.parent.as_deref().map(bone).transpose()?,
                    name: b.name,
                    x,
                    y,
                    rotation: b.rotation.unwrap_or(0.0),
                    skew: b.skew.unwrap_or(0.0),
                    scale: b.scale.unwrap_or([1.0; 2]),
                    length: b.length.unwrap_or(0.0),
                })
            })
            .collect::<Result<_, String>>()?,
        slots: input
            .slots
            .into_iter()
            .map(|s| {
                Ok(Slot2d {
                    bone: bone(&s.bone)?,
                    name: s.name,
                    attachment: s.attachment.as_deref().map(attachment).transpose()?,
                    color: s.color.unwrap_or([1.0; 4]),
                    offset: s.offset.unwrap_or([0.0; 4]),
                })
            })
            .collect::<Result<_, String>>()?,
        attachments: input
            .attachments
            .into_iter()
            .map(|(name, a)| {
                let (s, kind) = match a {
                    a::Attachment::Region {
                        slot,
                        rect,
                        at,
                        rotation,
                        skew,
                        size,
                    } => (
                        slot,
                        AttachmentKind::Region {
                            rect,
                            at: at.unwrap_or([0.0; 2]),
                            rotation: rotation.unwrap_or(0.0),
                            skew: skew.unwrap_or(0.0),
                            size: size.unwrap_or([rect[2], rect[3]]),
                        },
                    ),
                    a::Attachment::Mesh {
                        slot,
                        vertices,
                        triangles,
                    } => {
                        let vertices = vertices
                            .into_iter()
                            .map(|v| {
                                Ok(MeshVertex2d {
                                    uv: v.uv,
                                    influences: influences(v.influences, &bone)?,
                                })
                            })
                            .collect::<Result<_, String>>()?;
                        (
                            slot,
                            AttachmentKind::Mesh {
                                vertices,
                                triangles,
                            },
                        )
                    }
                    a::Attachment::Box { slot, polygon } => (slot, AttachmentKind::Box { polygon }),
                    a::Attachment::Path {
                        slot,
                        points,
                        closed,
                        constant,
                        lengths,
                    } => {
                        let points = points
                            .into_iter()
                            .map(|p| influences(p, &bone))
                            .collect::<Result<_, _>>()?;
                        (
                            slot,
                            AttachmentKind::Path {
                                points,
                                closed,
                                constant,
                                lengths,
                            },
                        )
                    }
                    a::Attachment::Child {
                        slot,
                        armature,
                        clip,
                        at,
                        rotation,
                        skew,
                        scale,
                    } => (
                        slot,
                        AttachmentKind::Child {
                            armature: index(children, &armature, "armature")?,
                            clip,
                            at: at.unwrap_or([0.0; 2]),
                            rotation: rotation.unwrap_or(0.0),
                            skew: skew.unwrap_or(0.0),
                            scale: scale.unwrap_or([1.0; 2]),
                        },
                    ),
                };
                Ok(Attachment2d {
                    name,
                    slot: slot(&s)?,
                    kind,
                })
            })
            .collect::<Result<_, String>>()?,
        iks: input
            .iks
            .into_iter()
            .map(|i| {
                Ok(Ik2d {
                    bones: i.bones.iter().map(|n| bone(n)).collect::<Result<_, _>>()?,
                    target: bone(&i.target)?,
                    name: i.name,
                    counterclockwise: i.counterclockwise.unwrap_or(false),
                    weight: i.weight.unwrap_or(1.0),
                })
            })
            .collect::<Result<_, String>>()?,
        transforms: input
            .transforms
            .into_iter()
            .map(|t| {
                Ok(og_anim::TransformConstraint2d {
                    bones: t.bones.iter().map(|n| bone(n)).collect::<Result<_, _>>()?,
                    target: bone(&t.target)?,
                    name: t.name,
                    rotation: t.rotation.unwrap_or(0.0),
                    at: t.at.unwrap_or([0.0; 2]),
                    scale: t.scale.unwrap_or([0.0; 2]),
                    shear: t.shear.unwrap_or(0.0),
                    mix: t.mix.unwrap_or([1.0; 4]),
                })
            })
            .collect::<Result<_, String>>()?,
        paths: input
            .paths
            .into_iter()
            .map(|p| {
                Ok(PathConstraint2d {
                    path: attachment(&p.path)?,
                    bones: p.bones.iter().map(|n| bone(n)).collect::<Result<_, _>>()?,
                    name: p.name,
                    position: p.position.unwrap_or(0.0),
                    position_unit: match p.position_unit {
                        Some(a::PathUnit::Fixed) => PathUnit::Fixed,
                        _ => PathUnit::Percent,
                    },
                    spacing: p.spacing.unwrap_or(0.0),
                    spacing_mode: match p.spacing_mode {
                        Some(a::PathSpacing::Fixed) => PathSpacing::Fixed,
                        Some(a::PathSpacing::Percent) => PathSpacing::Percent,
                        _ => PathSpacing::Length,
                    },
                    rotate: match p.rotate {
                        Some(a::PathRotate::Chain) => PathRotate::Chain,
                        Some(a::PathRotate::ChainScale) => PathRotate::ChainScale,
                        _ => PathRotate::Tangent,
                    },
                    rotation: p.rotation.unwrap_or(0.0),
                    mix: p.mix.unwrap_or([1.0; 2]),
                })
            })
            .collect::<Result<_, String>>()?,
        clips: input
            .clips
            .into_iter()
            .map(|c| {
                let mut events = c
                    .events
                    .into_iter()
                    .map(|k| (k.time, k.value))
                    .collect::<Vec<_>>();
                og_core::sort::sort_by(&mut events, |a, b| a.0.total_cmp(&b.0));
                Ok(Clip2d {
                    name: c.name,
                    duration: c.duration,
                    looping: c.looping.unwrap_or(true),
                    bones: c
                        .bones
                        .into_iter()
                        .map(|t| {
                            Ok(BoneTimeline {
                                bone: bone(&t.bone)?,
                                rotate: keys(t.rotate, |r| {
                                    Ok(Rotation {
                                        degrees: r.degrees,
                                        turn: match r.turn {
                                            None | Some(a::Turn::Shorter) => Turn::Shorter,
                                            Some(a::Turn::Clockwise { turns }) => {
                                                Turn::Clockwise(turns)
                                            }
                                            Some(a::Turn::Counterclockwise { turns }) => {
                                                Turn::Counterclockwise(turns)
                                            }
                                        },
                                    })
                                })?,
                                skew: keys(t.skew, Ok)?,
                                translate: keys(t.translate, Ok)?,
                                scale: keys(t.scale, Ok)?,
                            })
                        })
                        .collect::<Result<_, String>>()?,
                    slots: c
                        .slots
                        .into_iter()
                        .map(|t| {
                            Ok(SlotTimeline {
                                slot: slot(&t.slot)?,
                                attachment: t
                                    .attachment
                                    .into_iter()
                                    .map(|k| {
                                        Ok((
                                            k.time,
                                            k.value.as_deref().map(attachment).transpose()?,
                                        ))
                                    })
                                    .collect::<Result<_, String>>()?,
                                color: keys(t.color, Ok)?,
                                offset: keys(t.offset, Ok)?,
                            })
                        })
                        .collect::<Result<_, String>>()?,
                    deforms: c
                        .deforms
                        .into_iter()
                        .map(|(n, k)| {
                            Ok(DeformTimeline {
                                attachment: attachment(&n)?,
                                keys: keys(k, Ok)?,
                            })
                        })
                        .collect::<Result<_, String>>()?,
                    order: c
                        .order
                        .into_iter()
                        .map(|k| {
                            Ok((
                                k.time,
                                k.value.iter().map(|n| slot(n)).collect::<Result<_, _>>()?,
                            ))
                        })
                        .collect::<Result<_, String>>()?,
                    iks: c
                        .iks
                        .into_iter()
                        .map(|(n, k)| {
                            Ok(IkTimeline {
                                ik: index(&iks, &n, "IK")?,
                                keys: keys(k, |v| {
                                    Ok(IkKey {
                                        weight: v.weight,
                                        counterclockwise: v.counterclockwise,
                                    })
                                })?,
                            })
                        })
                        .collect::<Result<_, String>>()?,
                    events,
                })
            })
            .collect::<Result<_, String>>()?,
        skins: input
            .skins
            .into_iter()
            .map(|(name, swaps)| {
                Ok(Skin2d {
                    name,
                    swaps: swaps
                        .into_iter()
                        .map(|(from, to)| {
                            Ok((
                                attachment(&from)?,
                                to.as_deref().map(attachment).transpose()?,
                            ))
                        })
                        .collect::<Result<_, String>>()?,
                })
            })
            .collect::<Result<_, String>>()?,
    })
}

fn influences(
    input: Vec<a::Influence>,
    bone: &impl Fn(&str) -> Result<usize, String>,
) -> Result<Vec<Influence>, String> {
    input
        .into_iter()
        .map(|v| {
            Ok(Influence {
                bone: bone(&v.bone)?,
                x: v.at[0],
                y: v.at[1],
                weight: v.weight,
            })
        })
        .collect()
}

fn keys<T, U>(
    input: Vec<a::Key<T>>,
    value: impl Fn(T) -> Result<U, String>,
) -> Result<Vec<Key<U>>, String> {
    input
        .into_iter()
        .map(|k| {
            Ok(Key {
                time: k.time,
                value: value(k.value)?,
                ease: match k.ease {
                    None | Some(a::Ease::Linear) => Ease::Linear,
                    Some(a::Ease::Step) => Ease::Step,
                    Some(a::Ease::Bezier { points }) => Ease::Bezier(points),
                },
            })
        })
        .collect()
}
