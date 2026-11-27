//! Checked author IR becomes native tracks before the internal blob codec runs.
use super::*;
use crate::cook::source_schemas::media::timeline as input;
use crate::cook::source_schemas::reflected::Reflected;
use og_core::{
    ease::Ease,
    glam::{Quat, Vec3},
};
use og_timeline::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) struct Prepared {
    pub timeline: Timeline,
    pub files: BTreeMap<String, (std::path::PathBuf, String)>,
}

impl Prepared {
    pub fn new(root: &Path, input: input::Timeline) -> Result<Self, String> {
        let mut out = Self {
            timeline: Timeline {
                name: input.name.unwrap_or_default(),
                duration: 0.0,
                paths: Vec::new(),
                tracks: Vec::new(),
            },
            files: BTreeMap::new(),
        };
        for (name, p) in input.paths {
            if name.is_empty() {
                return Err("a path needs a nonempty name".into());
            }
            let desc = PathDesc {
                points: p.points.into_iter().map(Vec3::from_array).collect(),
                closed: p.closed.unwrap_or(false),
                tension: p.tension.unwrap_or(0.5),
                kind: match p.kind.unwrap_or(input::PathKind::CatmullRom) {
                    input::PathKind::CatmullRom => PathKind::CatmullRom,
                    input::PathKind::Bezier => PathKind::Bezier,
                    input::PathKind::Linear => PathKind::Linear,
                },
            };
            og_timeline::Path::new(desc.clone()).map_err(|e| format!("path {name}: {e}"))?;
            out.timeline.paths.push((name, desc));
        }
        for (i, track) in input.tracks.into_iter().enumerate() {
            let track = out
                .track(root, track)
                .map_err(|e| format!("tracks/{i}: {e}"))?;
            out.timeline.tracks.push(track);
        }
        out.timeline.duration = input
            .duration
            .unwrap_or_else(|| out.timeline.tracks.iter().map(end).fold(0.0, f32::max));
        if !out.timeline.duration.is_finite() || out.timeline.duration <= 0.0 {
            return Err("a timeline needs finite positive duration or a key after 0".into());
        }
        for track in &out.timeline.tracks {
            if !end(track).is_finite() {
                return Err(format!("{} track end is not finite", track.kind()));
            }
            if let Some(time) = track
                .times()
                .into_iter()
                .find(|&t| t > out.timeline.duration)
            {
                return Err(format!(
                    "{} key at {time} is after duration {}",
                    track.kind(),
                    out.timeline.duration
                ));
            }
            if let Track::Fields(f) = track {
                check_fields(f)?;
            }
        }
        Ok(out)
    }

    fn track(&mut self, root: &Path, t: input::Track) -> Result<Track, String> {
        use input::Track as T;
        Ok(match t {
            T::Camera {
                rail,
                follow,
                keys,
                blend_in,
                blend_out,
            } => {
                let mode = match (rail, follow) {
                    (Some(_), Some(_)) => return Err("rail and follow are exclusive".into()),
                    (Some(p), None) => CameraMode::Rail(p),
                    (None, Some(e)) => CameraMode::Follow(e),
                    _ => CameraMode::Keys,
                };
                Track::Camera(CameraTrack {
                    keys: map(keys, |time, ease, v| {
                        match &mode {
                            CameraMode::Rail(_) if v.position.is_some() => {
                                return Err("rail keys use at, not position".into());
                            }
                            CameraMode::Keys | CameraMode::Follow(_) if v.at.is_some() => {
                                return Err("at requires a rail".into());
                            }
                            _ => {}
                        }
                        Ok(CameraKey {
                            time,
                            ease,
                            position: v.position.map(Vec3::from_array),
                            target: v.target.map(Vec3::from_array),
                            at: v.at,
                            fov: v.fov,
                            roll: v.roll,
                            focus: v.focus,
                            focus_on: v.focus_on,
                            focus_range: v.focus_range,
                        })
                    })?,
                    mode,
                    blend_in: blend_in.unwrap_or(0.0),
                    blend_out: blend_out.unwrap_or(0.0),
                })
            }
            T::Transform { entity, keys } => Track::Transform(TransformTrack {
                entity,
                keys: map(keys, |time, ease, v| {
                    let quaternion = match (v.rotation, v.quaternion) {
                        (Some(_), Some(_)) => {
                            return Err("rotation and quaternion are exclusive".into());
                        }
                        (Some(v), None) => Some(euler_degrees(Vec3::from_array(v))),
                        (None, Some(v)) => {
                            let q = Quat::from_array(v);
                            if q.length_squared() < 1e-12 {
                                return Err("quaternion must be nonzero".into());
                            }
                            Some(q.normalize())
                        }
                        _ => None,
                    };
                    Ok(TransformKey {
                        time,
                        ease,
                        position: v.position.map(Vec3::from_array),
                        quaternion,
                        scale: v.scale.map(Vec3::from_array),
                    })
                })?,
            }),
            T::Animation {
                entity,
                model,
                keys,
            } => Track::Animation(AnimationTrack {
                entity,
                model: model.map_or_else(String::new, |r| r.id),
                keys: map(keys, |time, _, v| {
                    if v.clip.is_empty() {
                        return Err("animation key needs a clip".into());
                    }
                    Ok(AnimationKey {
                        time,
                        clip: v.clip,
                        speed: v.speed.unwrap_or(1.0),
                        blend_in: v.blend_in.unwrap_or(0.2),
                        looping: v.looping.unwrap_or(true),
                    })
                })?,
            }),
            T::Audio { keys } => Track::Audio(map(keys, |time, _, v| {
                Ok(SoundKey {
                    time,
                    sound: v.sound.id,
                    volume: v.volume.unwrap_or(1.0),
                    pitch: v.playback_rate.unwrap_or(1.0),
                    bus: bus(v.bus, 2),
                })
            })?),
            T::Music { keys } => Track::Music(map(keys, |time, _, v| {
                if v.end.is_some_and(|end| end <= time) {
                    return Err("music end must follow its time".into());
                }
                Ok(MusicKey {
                    time,
                    sound: v.sound.id,
                    volume: v.volume.unwrap_or(1.0),
                    fade_in: v.fade_in.unwrap_or(0.0),
                    fade_out: v.fade_out.unwrap_or(0.0),
                    end: v.end,
                    looping: v.looping.unwrap_or(true),
                    bus: bus(v.bus, 1),
                })
            })?),
            T::Caption {
                keys,
                imports,
                locales,
            } => {
                let mut locales = locales.unwrap_or_default();
                let explicit: BTreeSet<_> =
                    locales.keys().map(|t| t.to_ascii_lowercase()).collect();
                // Preserve adjacent translations without generating another
                // author format. New directory members are cook-index inputs.
                for import in &imports {
                    let source = crate::cook::source_path::checked(root, &import.source)
                        .map_err(|e| format!("{}: {e}", import.source))?;
                    let (Some(directory), Some(stem), Some(ext)) = (
                        source.parent(),
                        source.file_stem().and_then(|s| s.to_str()),
                        source.extension().and_then(|s| s.to_str()),
                    ) else {
                        continue;
                    };
                    let mut names = std::fs::read_dir(directory)
                        .map_err(|e| e.to_string())?
                        .map(|e| e.map(|e| e.file_name()))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| e.to_string())?;
                    names.sort();
                    for name in names {
                        let Some(name) = name.to_str() else { continue };
                        let Some(tag) = name
                            .strip_prefix(&format!("{stem}."))
                            .and_then(|s| s.strip_suffix(&format!(".{ext}")))
                        else {
                            continue;
                        };
                        let mut parts = tag.split('-');
                        let valid = parts.next().is_some_and(|s| {
                            (2..=3).contains(&s.len()) && s.bytes().all(|c| c.is_ascii_alphabetic())
                        }) && parts.all(|s| {
                            (1..=8).contains(&s.len())
                                && s.bytes().all(|c| c.is_ascii_alphanumeric())
                        });
                        if !valid || explicit.contains(&tag.to_ascii_lowercase()) {
                            continue;
                        }
                        let source = format!(
                            "{}/{name}",
                            import
                                .source
                                .rsplit_once('/')
                                .expect("canonical input directory")
                                .0
                        );
                        locales
                            .entry(tag.into())
                            .or_insert_with(|| input::CaptionCues {
                                keys: Vec::new(),
                                imports: Vec::new(),
                            })
                            .imports
                            .push(input::CaptionImport {
                                source,
                                offset: import.offset,
                            });
                    }
                }
                let keys = self.captions(root, keys, imports)?;
                let mut tags = BTreeSet::new();
                let mut translated = Vec::new();
                for (tag, cues) in locales {
                    if tag.is_empty() || !tags.insert(tag.to_ascii_lowercase()) {
                        return Err(
                            "caption locales must be nonempty and unique ignoring case".into()
                        );
                    }
                    translated.push((tag, self.captions(root, cues.keys, cues.imports)?));
                }
                Track::Caption(CaptionTrack {
                    keys,
                    locales: translated,
                })
            }
            T::Event { keys } => Track::Event(map(keys, |time, _, v| {
                let mut name = v.action.split(':').next().unwrap_or_default().chars();
                if !matches!(name.next(), Some(c) if c.is_ascii_lowercase() || c == '_')
                    || !name.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                {
                    return Err("event action must be a command-channel action name".into());
                }
                Ok(EventKey {
                    time,
                    action: v.action,
                    retroactive: v.retroactive.unwrap_or(false),
                })
            })?),
            T::Transition { keys } => Track::Transition(map(keys, |time, _, v| {
                Ok(TransitionKey {
                    time,
                    kind: match v.kind.unwrap_or(input::TransitionKind::Fade) {
                        input::TransitionKind::Cut => TransitionKind::Cut,
                        input::TransitionKind::Fade => TransitionKind::Fade,
                        input::TransitionKind::Wipe => TransitionKind::Wipe,
                        input::TransitionKind::Dissolve => TransitionKind::Dissolve,
                        input::TransitionKind::Iris => TransitionKind::Iris,
                        input::TransitionKind::Slide => TransitionKind::Slide,
                    },
                    mode: match v.mode.unwrap_or(input::TransitionMode::Through) {
                        input::TransitionMode::Through => TransitionMode::Through,
                        input::TransitionMode::In => TransitionMode::In,
                        input::TransitionMode::Out => TransitionMode::Out,
                    },
                    duration: v.duration.unwrap_or(1.0),
                    color: Vec3::from_array(v.color.unwrap_or([0.0; 3])),
                    from: match v.from.unwrap_or(input::Direction::Left) {
                        input::Direction::Left => 0,
                        input::Direction::Right => 1,
                        input::Direction::Top => 2,
                        input::Direction::Bottom => 3,
                    },
                })
            })?),
            T::Marker { keys } => Track::Marker(map(keys, |time, _, v| {
                if v.name.is_empty() || v.name.len() > 63 {
                    return Err("marker name must be 1..63 UTF-8 bytes".into());
                }
                Ok(MarkerKey { time, name: v.name })
            })?),
            T::Property {
                entity,
                field,
                keys,
            } => {
                if !field
                    .rsplit_once('.')
                    .is_some_and(|(r, n)| !r.is_empty() && !n.is_empty())
                {
                    return Err("property field must name record.field".into());
                }
                fields(
                    FieldsKind::Property,
                    entity.unwrap_or_default(),
                    0,
                    keys,
                    |v| {
                        Ok(vec![(
                            field.clone(),
                            match v {
                                input::FieldValue::Number(n) => FieldValue::Numbers(vec![n]),
                                input::FieldValue::Bool(b) => FieldValue::Word(b.to_string()),
                                input::FieldValue::Word(w) => FieldValue::Word(w),
                                input::FieldValue::Vec2(v) => FieldValue::Numbers(v.to_vec()),
                                input::FieldValue::Vec3(v) => FieldValue::Numbers(v.to_vec()),
                                input::FieldValue::Vec4(v) => FieldValue::Numbers(v.to_vec()),
                            },
                        )])
                    },
                )?
            }
            T::Light { entity, keys } => fields(FieldsKind::Light, entity, 0, keys, |v| match v {
                input::Light::Directional(v) => reflected(v),
                input::Light::Point(v) => reflected(v),
                input::Light::Spot(v) => reflected(v),
                input::Light::Rect(v) => reflected(v),
                input::Light::Planar(v) => reflected(v),
            })?,
            T::Post { keys } => fields(FieldsKind::Post, String::new(), 0, keys, reflected)?,
            T::Blendshape { entity, keys } => {
                fields(FieldsKind::Blendshape, entity, 0, keys, reflected)?
            }
            T::Visibility { entity, keys } => {
                fields(FieldsKind::Visibility, entity, 0, keys, |v| {
                    Ok(vec![("visible".into(), FieldValue::Word(v.to_string()))])
                })?
            }
            T::Material {
                entity,
                index,
                keys,
            } => fields(
                FieldsKind::Material,
                entity,
                index.unwrap_or(0),
                keys,
                |v| {
                    let mut out = match v.standard {
                        Some(v) => reflected(v)?,
                        None => Vec::new(),
                    };
                    if let Some(v) = v.map_offset {
                        out.push(("mapOffset".into(), FieldValue::Numbers(v.to_vec())));
                    }
                    if let Some(v) = v.map_repeat {
                        out.push(("mapRepeat".into(), FieldValue::Numbers(v.to_vec())));
                    }
                    if let Some(v) = v.map_rotation {
                        out.push(("mapRotation".into(), FieldValue::Numbers(vec![v])));
                    }
                    out.extend(v.params.into_iter().map(|(n, v)| {
                        (
                            n,
                            FieldValue::Numbers(match v {
                                input::Uniform::Float(v) => vec![v],
                                input::Uniform::Vec2(v) => v.to_vec(),
                                input::Uniform::Vec3(v) => v.to_vec(),
                                input::Uniform::Vec4(v) => v.to_vec(),
                            }),
                        )
                    }));
                    Ok(out)
                },
            )?,
            T::Timeline { keys } => Track::Timeline(map(keys, |time, _, v| {
                Ok(NestedKey {
                    time,
                    play: v.play.id,
                    speed: v.speed.unwrap_or(1.0),
                    looping: v.looping.unwrap_or(false),
                })
            })?),
            T::Shot { keys } => Track::Shot(map(keys, |time, ease, v| {
                Ok(ShotKey {
                    time,
                    ease,
                    camera: v.camera.unwrap_or_default(),
                    blend: v.blend.unwrap_or(-1.0),
                })
            })?),
            T::Vfx { keys } => Track::Vfx(map(keys, |time, _, v| {
                Ok(VfxKey {
                    time,
                    effect: v.effect.id,
                    at: v.at.unwrap_or_default(),
                    position: Vec3::from_array(v.position.unwrap_or([0.0; 3])),
                    scale: v.scale.unwrap_or(1.0),
                    seed: v.seed.unwrap_or(0),
                })
            })?),
            T::Video {
                entity,
                layer,
                keys,
            } => {
                if entity.is_some() && layer.is_some() {
                    return Err("layer applies only to the timeline's own video player".into());
                }
                Track::Video(VideoTrack {
                    entity: entity.unwrap_or_default(),
                    layer: match layer.unwrap_or(input::VideoLayer::Over) {
                        input::VideoLayer::Over => VideoLayer::Over,
                        input::VideoLayer::Under => VideoLayer::Under,
                    },
                    keys: map(keys, |time, _, v| {
                        Ok(VideoKey {
                            time,
                            video: v.video.id,
                            from: v.from.unwrap_or(0.0),
                            speed: v.speed.unwrap_or(1.0),
                        })
                    })?,
                })
            }
            T::Hotspot { keys } => Track::Hotspot(map(keys, |time, ease, v| {
                let off = v.off.unwrap_or(false);
                if v.id.is_empty() || v.id.len() > 63 {
                    return Err("hotspot id must be 1..63 UTF-8 bytes".into());
                }
                if v.rect.is_some() == off {
                    return Err("hotspot needs rect or off: true, not both".into());
                }
                let rect = v.rect.unwrap_or([0.0; 4]);
                let [x, y, w, h] = rect;
                if !off
                    && !(w > 0.0 && h > 0.0 && x >= 0.0 && y >= 0.0 && x + w <= 1.0 && y + h <= 1.0)
                {
                    return Err("hotspot rect must lie inside the frame".into());
                }
                Ok(HotspotKey {
                    time,
                    ease,
                    id: v.id,
                    rect,
                    off,
                    shape: match v.shape.unwrap_or(input::HotspotShape::Rect) {
                        input::HotspotShape::Rect => HotspotShape::Rect,
                        input::HotspotShape::Circle => HotspotShape::Circle,
                    },
                    kind: match v.kind.unwrap_or(input::HotspotKind::Choice) {
                        input::HotspotKind::Choice => HotspotKind::Choice,
                        input::HotspotKind::Inspect => HotspotKind::Inspect,
                    },
                    label: v.label.unwrap_or_default(),
                })
            })?),
            T::Scene { keys } => Track::Scene(map(keys, |time, _, v| {
                Ok(SceneKey {
                    time,
                    scene: v.scene.map_or_else(String::new, |r| r.id),
                })
            })?),
        })
    }

    fn captions(
        &mut self,
        root: &Path,
        mut keys: Vec<input::Key<input::Caption>>,
        imports: Vec<input::CaptionImport>,
    ) -> Result<Vec<CaptionKey>, String> {
        for import in imports {
            let file = import.source;
            if !file.starts_with("assets/") && !file.starts_with("sources/") {
                return Err(format!(
                    "{file}: caption inputs belong under module assets/ or sources/"
                ));
            }
            if !self.files.contains_key(&file) {
                let path = crate::cook::source_path::checked(root, &file)
                    .map_err(|e| format!("{file}: {e}"))?;
                let text = std::fs::read_to_string(&path).map_err(|e| format!("{file}: {e}"))?;
                self.files.insert(file.clone(), (path, text));
            }
            let text = &self.files[&file].1;
            let cues = match Path::new(&file).extension().and_then(|s| s.to_str()) {
                Some("srt") => captions::parse_srt(text)?,
                Some("vtt") => captions::parse_vtt(text)?,
                Some("json") => captions::parse_word_json(text, CAPTION_CHARS, CAPTION_GAP)?,
                _ => return Err(format!("{file}: caption file must be .srt, .vtt or .json")),
            };
            let offset = import.offset.unwrap_or(0.0);
            for cue in cues {
                if !(cue.start + offset).is_finite()
                    || !(cue.end + offset).is_finite()
                    || cue.start + offset < 0.0
                {
                    return Err(format!(
                        "{file}: caption offset must keep finite times at or after 0"
                    ));
                }
                keys.push(input::Key {
                    time: cue.start + offset,
                    ease: None,
                    value: input::Caption {
                        end: Some(cue.end + offset),
                        text: cue.text,
                        speaker: Some(cue.speaker),
                        position: match cue.position.as_str() {
                            "top" => Some(input::CaptionPosition::Top),
                            "bottom" => Some(input::CaptionPosition::Bottom),
                            _ => None,
                        },
                        spans: cue
                            .spans
                            .into_iter()
                            .map(|s| input::Span {
                                name: s.name,
                                position: s.position,
                                length: s.length,
                                color: Some(s.color),
                            })
                            .collect(),
                    },
                });
            }
        }
        keys.sort_by(|a, b| a.time.total_cmp(&b.time));
        let mut out = Vec::new();
        for (i, k) in keys.iter().enumerate() {
            let v = &k.value;
            let end = v.end.unwrap_or_else(|| {
                keys.get(i + 1).map_or(k.time + CAPTION_SECONDS, |n| {
                    n.time.min(k.time + CAPTION_SECONDS)
                })
            });
            if !end.is_finite() || end <= k.time {
                return Err(format!(
                    "caption key {i}: explicit end must be finite and follow its time"
                ));
            }
            out.push(CaptionKey {
                time: k.time,
                end,
                text: v.text.clone(),
                speaker: v.speaker.clone().unwrap_or_default(),
                position: match v.position {
                    Some(input::CaptionPosition::Top) => "top",
                    Some(input::CaptionPosition::Bottom) => "bottom",
                    None => "",
                }
                .into(),
                spans: v
                    .spans
                    .iter()
                    .map(|s| captions::CaptionSpan {
                        name: s.name.clone(),
                        position: s.position,
                        length: s.length,
                        color: s.color.clone().unwrap_or_default(),
                    })
                    .collect(),
            });
        }
        Ok(out)
    }
}

fn map<T, K>(
    mut keys: Vec<input::Key<T>>,
    mut f: impl FnMut(f32, Ease, T) -> Result<K, String>,
) -> Result<Vec<K>, String> {
    keys.sort_by(|a, b| a.time.total_cmp(&b.time));
    keys.into_iter()
        .enumerate()
        .map(|(i, k)| {
            let ease = k
                .ease
                .as_deref()
                .map(Ease::parse_text)
                .transpose()?
                .unwrap_or_default();
            f(k.time, ease, k.value).map_err(|e| format!("keys/{i}: {e}"))
        })
        .collect()
}

fn bus(bus: Option<input::Bus>, default: u32) -> u32 {
    match bus {
        Some(input::Bus::Master) => 0,
        Some(input::Bus::Music) => 1,
        Some(input::Bus::Sfx) => 2,
        Some(input::Bus::Ui) => 3,
        None => default,
    }
}

fn reflected<M>(v: Reflected<M>) -> Result<Vec<(String, FieldValue)>, String> {
    v.into_fields()
        .into_iter()
        .map(|(name, v)| {
            let v = match v {
                serde_json::Value::Bool(v) => FieldValue::Word(v.to_string()),
                serde_json::Value::String(v) if v.starts_with('#') => FieldValue::Numbers(
                    crate::look::hex_to_linear(&v)
                        .ok_or_else(|| format!("{name}: expected sRGB #rrggbb"))?
                        .map(|n| n as f32)
                        .to_vec(),
                ),
                serde_json::Value::String(v) => FieldValue::Word(v),
                serde_json::Value::Number(v) => {
                    FieldValue::Numbers(vec![v.as_f64().ok_or("invalid field number")? as f32])
                }
                serde_json::Value::Array(v) => FieldValue::Numbers(
                    v.into_iter()
                        .map(|v| {
                            v.as_f64()
                                .map(|v| v as f32)
                                .ok_or_else(|| "field vector needs numbers".to_string())
                        })
                        .collect::<Result<_, _>>()?,
                ),
                _ => return Err(format!("{name}: timeline cannot drive this field")),
            };
            Ok((name, v))
        })
        .collect()
}

fn fields<T>(
    kind: FieldsKind,
    entity: String,
    index: u32,
    keys: Vec<input::Key<T>>,
    mut f: impl FnMut(T) -> Result<Vec<(String, FieldValue)>, String>,
) -> Result<Track, String> {
    Ok(Track::Fields(FieldsTrack {
        kind,
        entity,
        index,
        keys: map(keys, |time, ease, v| {
            let values = f(v)?;
            if values.is_empty() {
                return Err("key must set at least one field".into());
            }
            Ok(FieldKey { time, ease, values })
        })?,
    }))
}

fn end(t: &Track) -> f32 {
    match t {
        Track::Music(keys) => keys
            .iter()
            .map(|k| k.end.unwrap_or(k.time))
            .fold(0.0, f32::max),
        Track::Caption(c) => c
            .keys
            .iter()
            .chain(c.locales.iter().flat_map(|(_, keys)| keys))
            .map(|k| k.end)
            .fold(0.0, f32::max),
        Track::Transition(keys) => keys.iter().map(|k| k.time + k.duration).fold(0.0, f32::max),
        _ => t.times().into_iter().fold(0.0, f32::max),
    }
}
