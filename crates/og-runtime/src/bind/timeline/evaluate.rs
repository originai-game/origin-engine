//! Timeline tracks evaluated into world and UI state.

use super::*;

/// One-shot voices that ended go; while paused every voice goes (a
/// resume starts the music again at its offset).
pub(super) fn drop_ended_voices(
    ctx: &mut SystemContext<'_>,
    r: &Rec,
    run: &mut Run,
    playing: bool,
) -> og_core::Result<()> {
    let mut keep = Vec::new();
    for (tr, k, v) in std::mem::take(&mut run.voices) {
        let e = Entity::from_bits(v);
        let alive = ctx.is_alive(e)
            && ctx.get_bytes(e, r.source.0).is_ok_and(|b| {
                r.source
                    .1
                    .read::<record::AudioSource>(&b)
                    .is_ok_and(|s| s.playing)
            });
        if alive && playing {
            keep.push((tr, k, v));
        } else if ctx.is_alive(e) {
            ctx.despawn(e)?;
        }
    }
    run.voices = keep;
    Ok(())
}

/// The track ids of a nested timeline's tracks: this tag, a hash of where
/// it nests (its parent's track id and the key), then its own track index
/// (below [`NESTED_TRACKS`]). A timeline's own tracks are their indices,
/// as they always were, so the state block of one without nesting is
/// unchanged.
pub(super) const NESTED: u32 = 0x8000_0000;
pub(super) const NESTED_TRACKS: u32 = 0x400;

pub(super) fn nest_id(parent: u32, key: usize) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in parent
        .to_le_bytes()
        .into_iter()
        .chain((key as u32).to_le_bytes())
    {
        h = (h ^ u32::from(b)).wrapping_mul(0x0100_0193);
    }
    NESTED | (h & !(NESTED | (NESTED_TRACKS - 1)))
}

/// What a tick's evaluation of one timeline is given.
pub(super) struct Step<'a> {
    pub t: f32,
    pub playing: bool,
    pub spans: &'a [(f64, f64)],
    pub retro: Option<(f64, f64)>,
    pub dt: f32,
    pub speed: f32,
    /// Added to each track's index for its id (0 for the timeline itself).
    pub base: u32,
    pub depth: usize,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn evaluate(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    r: &Rec,
    ui: &og_ui::SharedUi,
    lt: &mut Loaded,
    run: &mut Run,
    step: &Step<'_>,
    touched: &mut Vec<u32>,
) -> og_core::Result<()> {
    let Step {
        t,
        playing,
        spans,
        retro,
        dt,
        speed,
        base,
        depth,
    } = *step;
    let duration = lt.timeline.duration;
    let tracks = lt.timeline.tracks.clone();
    for (ti, tr) in tracks.iter().enumerate() {
        let ti32 = base | ti as u32;
        match tr {
            Track::Camera(c) => camera(ctx, engine_id, r, lt, run, c, t)?,
            Track::Shot(keys) => shot(ctx, engine_id, r, lt, run, keys, t)?,
            Track::Fields(f) if f.kind != og_timeline::FieldsKind::Material => {
                fields(ctx, engine_id, &lt.id, f, t)?;
            }
            Track::Timeline(keys) => {
                let Some((ki, until, _)) = eval::nested_at(keys, t, duration, |_| 0.0) else {
                    continue;
                };
                let k = &keys[ki];
                let Some(child) = lt.nested.get(&k.play).cloned() else {
                    continue;
                };
                if depth + 1 > og_timeline::NEST_DEPTH {
                    continue;
                }
                let mut cl = lock(&child);
                let d = cl.timeline.duration;
                let cspans: Vec<(f64, f64)> = spans
                    .iter()
                    .flat_map(|(a, b)| eval::child_spans(k, until, d, *a, *b))
                    .collect();
                let cretro =
                    retro.and_then(|(a, b)| eval::child_spans(k, until, d, a, b).first().copied());
                let cbase = nest_id(ti32, ki);
                touched.push(cbase);
                evaluate(
                    ctx,
                    engine_id,
                    r,
                    ui,
                    &mut cl,
                    run,
                    &Step {
                        t: eval::child_time(k, t, d),
                        playing,
                        spans: &cspans,
                        retro: cretro,
                        dt,
                        speed: speed * k.speed,
                        base: cbase,
                        depth: depth + 1,
                    },
                    touched,
                )?;
            }
            Track::Transform(tt) => {
                let Some(e) = ctx.find(&tt.entity) else {
                    note(
                        engine_id,
                        format!(
                            "timeline {}: no entity '{}' for its transform track",
                            lt.id, tt.entity
                        ),
                    );
                    continue;
                };
                let at = eval::transform_at(tt, t);
                let bytes = ctx.get_bytes(e, r.transform.0)?;
                let mut v = r.transform.1.decode(&bytes);
                if let Some(p) = at.position {
                    v["position"] = jv3(p);
                }
                if let Some(q) = at.quaternion {
                    v["quaternion"] = jq(q);
                }
                if let Some(s) = at.scale {
                    v["scale"] = jv3(s);
                }
                let out = r
                    .transform
                    .1
                    .encode(Some(&bytes), Some(&v))
                    .map_err(invalid)?;
                ctx.set_bytes(e, r.transform.0, &out)?;
            }
            Track::Animation(a) => {
                let Some((ki, ct)) = eval::clip_at(a, t) else {
                    continue;
                };
                let Some(e) = ctx.find(&a.entity) else {
                    note(
                        engine_id,
                        format!(
                            "timeline {}: no entity '{}' for its animation track",
                            lt.id, a.entity
                        ),
                    );
                    continue;
                };
                if !lt.clips.contains_key(&ti) {
                    match model_clips(ctx, engine_id, r, e) {
                        Ok(c) => {
                            lt.clips.insert(ti, c);
                        }
                        Err(m) => {
                            note(
                                engine_id,
                                format!(
                                    "timeline {}: animation track of '{}': {m}",
                                    lt.id, a.entity
                                ),
                            );
                            continue;
                        }
                    }
                }
                let (skeleton, clips) = &lt.clips[&ti];
                let key = &a.keys[ki];
                let Some(&clip) = clips.get(&key.clip) else {
                    note(
                        engine_id,
                        format!(
                            "timeline {}: '{}' has no clip '{}' (its clips: {})",
                            lt.id,
                            a.entity,
                            key.clip,
                            clips.keys().cloned().collect::<Vec<_>>().join(", ")
                        ),
                    );
                    continue;
                };
                if ctx.get_bytes(e, r.skeleton.0).is_err() {
                    let s = r
                        .skeleton
                        .1
                        .encode(None, Some(&json!({ "asset": *skeleton as f64 })))
                        .map_err(invalid)?;
                    ctx.insert_bytes(e, r.skeleton.0, &s)?;
                }
                let changed = run.anim.iter().find(|x| x.0 == ti32).map(|x| x.1) != Some(ki as u32);
                run.anim.retain(|x| x.0 != ti32);
                run.anim.push((ti32, ki as u32));
                // og.anim advances the player by its speed after this
                // system: write the time it lands on `ct` from.
                let eff = if playing { key.speed * speed } else { 0.0 };
                let mut v = match ctx.get_bytes(e, r.player.0) {
                    Ok(b) => r.player.1.decode(&b),
                    Err(_) => json!({}),
                };
                v["clip"] = json!(clip as f64);
                v["speed"] = json!(eff);
                v["loop"] = json!(key.looping);
                v["time"] = json!((ct - dt * eff).max(0.0));
                if changed {
                    v["fade"] = json!(if playing
                        && spans.iter().any(|(a, b)| crossed(key.time, *a, *b))
                    {
                        key.blend_in
                    } else {
                        0.0
                    });
                }
                let base = ctx.get_bytes(e, r.player.0).ok();
                let out = r
                    .player
                    .1
                    .encode(base.as_deref(), Some(&v))
                    .map_err(invalid)?;
                if base.is_some() {
                    ctx.set_bytes(e, r.player.0, &out)?;
                } else {
                    ctx.insert_bytes(e, r.player.0, &out)?;
                }
            }
            Track::Audio(ks) if playing => {
                for (ki, k) in ks.iter().enumerate() {
                    let (handle, seconds) = lt.sounds.get(&k.sound).copied().unwrap_or((0, 0.0));
                    let started = run.voices.iter().any(|v| v.0 == ti32 && v.1 == ki as u32);
                    let fire = spans.iter().any(|(a, b)| crossed(k.time, *a, *b));
                    // Joined mid-sound (a start or a seek past its key): from its offset.
                    let mid = spans.is_empty() || spans.iter().all(|(a, _)| *a > f64::from(k.time));
                    let within = t >= k.time && t < k.time + seconds / k.pitch.max(1e-3);
                    if started || handle == 0 || !(fire || (mid && within)) {
                        continue;
                    }
                    let offset = if fire { 0.0 } else { (t - k.time) * k.pitch };
                    let v = json!({
                        "sound": handle as f64, "bus": k.bus, "volume": k.volume, "playbackRate": k.pitch,
                        "playing": true, "startOffset": offset,
                    });
                    let bytes = r.source.1.encode(None, Some(&v)).map_err(invalid)?;
                    let e = ctx.spawn_bytes(vec![(r.source.0, bytes)])?;
                    run.voices.push((ti32, ki as u32, e.to_bits()));
                }
            }
            Track::Music(ks) => {
                for (ki, k) in ks.iter().enumerate() {
                    let gain = if playing {
                        eval::music_gain(k, t, duration)
                    } else {
                        None
                    };
                    let at = run
                        .voices
                        .iter()
                        .position(|v| v.0 == ti32 && v.1 == ki as u32);
                    match (gain, at) {
                        (Some(g), Some(i)) => {
                            let e = Entity::from_bits(run.voices[i].2);
                            let b = ctx.get_bytes(e, r.source.0)?;
                            let mut v = r.source.1.decode(&b);
                            v["volume"] = json!(g);
                            let out = r.source.1.encode(Some(&b), Some(&v)).map_err(invalid)?;
                            ctx.set_bytes(e, r.source.0, &out)?;
                        }
                        (Some(g), None) => {
                            let Some(&(handle, _)) = lt.sounds.get(&k.sound) else {
                                continue;
                            };
                            let v = json!({
                                "sound": handle as f64, "bus": k.bus, "volume": g, "loop": k.looping,
                                "playing": true, "startOffset": t - k.time,
                            });
                            let bytes = r.source.1.encode(None, Some(&v)).map_err(invalid)?;
                            let e = ctx.spawn_bytes(vec![(r.source.0, bytes)])?;
                            run.voices.push((ti32, ki as u32, e.to_bits()));
                        }
                        (None, Some(i)) => {
                            let (_, _, v) = run.voices.remove(i);
                            let e = Entity::from_bits(v);
                            if ctx.is_alive(e) {
                                ctx.despawn(e)?;
                            }
                        }
                        (None, None) => {}
                    }
                }
            }
            Track::Caption(c) => {
                if let Some(k) = eval::caption_at(c, t) {
                    caption(ui, ti32, k, c);
                }
            }
            Track::Event(ks) => {
                for k in ks {
                    let run_it = spans.iter().any(|(a, b)| crossed(k.time, *a, *b))
                        || (k.retroactive && retro.is_some_and(|(a, b)| crossed(k.time, a, b)));
                    if run_it {
                        let mut parts = k.action.split(':').map(str::to_string);
                        let action = parts.next().unwrap_or_default();
                        if let Some(inbox) = lock(&registry(engine_id)).inbox.clone() {
                            inbox.push(action, parts.collect());
                        }
                    }
                }
            }
            Track::Transition(ks) => {
                if let Some(c) = eval::cover_at(ks, t) {
                    let kind = match c.kind {
                        TransitionKind::Cut | TransitionKind::Fade => og_ui::ScreenWipe::Fade,
                        TransitionKind::Wipe => og_ui::ScreenWipe::Wipe,
                        TransitionKind::Dissolve => og_ui::ScreenWipe::Dissolve,
                        TransitionKind::Iris => og_ui::ScreenWipe::Iris,
                        TransitionKind::Slide => og_ui::ScreenWipe::Slide,
                    };
                    let side = [
                        og_ui::Side::Left,
                        og_ui::Side::Right,
                        og_ui::Side::Top,
                        og_ui::Side::Bottom,
                    ];
                    let byte = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
                    let desc = og_ui::ScreenTransitionDesc {
                        kind,
                        color: Some(og_ui::Srgba {
                            rgb: [byte(c.color.x), byte(c.color.y), byte(c.color.z)],
                            alpha: 1.0,
                        }),
                        from: side[(c.from as usize).min(3)],
                        ..Default::default()
                    };
                    ui.lock().cover(&desc, c.amount, c.revealing);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// The skeleton and clips of the model an entity draws (its og.Model's glTF).
fn model_clips(
    ctx: &SystemContext<'_>,
    engine_id: u32,
    r: &Rec,
    e: Entity,
) -> Result<(u64, BTreeMap<String, u64>), String> {
    let b = ctx.get_bytes(e, r.model.0).map_err(|_| {
        "it has no og.Model; name the glTF with `model \"<path>\"` in the track".to_string()
    })?;
    let asset = r.model.1.read::<record::Model>(&b)?.asset as f64;
    let eng = engine(engine_id)?;
    let path = {
        let a = eng.assets.borrow();
        super::assets::index_of(asset)
            .and_then(|i| a.entries.get(i))
            .map(|x| x.path.clone())
            .ok_or("its og.Model.asset is not a loaded glTF")?
    };
    clip_set(&eng, &path)
}

/// A shot track at time `t`: the key at or before it names
/// og.CameraBrain.solo (its blend and ease the brain's soloBlend and
/// soloEase); a `none` key, and the time before the first key, hand the
/// solo the run found back. Written only when it changes, so a game's own
/// solo set meanwhile stands until the next key.
fn shot(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    r: &Rec,
    lt: &Loaded,
    run: &mut Run,
    keys: &[og_timeline::ShotKey],
    t: f32,
) -> og_core::Result<()> {
    let Some(k) = keys.iter().rev().find(|k| k.time <= t) else {
        return Ok(());
    };
    let want = if k.camera.is_empty() {
        None
    } else {
        match ctx.find(&k.camera) {
            Some(e) => Some(e.to_bits()),
            None => {
                note(
                    engine_id,
                    format!(
                        "timeline {}: no entity '{}' for its shot track (a virtual camera's name)",
                        lt.id, k.camera
                    ),
                );
                None
            }
        }
    };
    let Some(cur) = solo_now(ctx, r)? else {
        return Ok(());
    };
    match (run.solo, want) {
        (None, Some(e)) => {
            run.solo = Some((cur, e));
            set_solo(ctx, r, e, k.blend, k.ease)?;
        }
        (Some((found, wrote)), Some(e)) if wrote != e => {
            run.solo = Some((found, e));
            set_solo(ctx, r, e, k.blend, k.ease)?;
        }
        (Some((found, _)), None) => {
            run.solo = None;
            set_solo(ctx, r, found, k.blend, k.ease)?;
        }
        _ => {}
    }
    Ok(())
}

/// og.CameraBrain.solo now, if the brain is registered.
fn solo_now(ctx: &SystemContext<'_>, r: &Rec) -> og_core::Result<Option<u64>> {
    let Ok(id) = ctx.id_by_name(crate::present::feel::vcam::BRAIN) else {
        return Ok(None);
    };
    if !ctx.has_resource_id(id)? {
        return Ok(None);
    }
    let Some(l) = &r.brain else {
        return Ok(None);
    };
    let v: record::CameraBrain = l.read(&ctx.resource_bytes(id)?).map_err(invalid)?;
    Ok(Some(v.solo))
}

/// Writes og.CameraBrain's solo and its blend (`blend` below 0: the
/// camera's own).
pub(super) fn set_solo(
    ctx: &mut SystemContext<'_>,
    r: &Rec,
    solo: u64,
    blend: f32,
    ease: og_tween::Ease,
) -> og_core::Result<()> {
    let (Ok(id), Some(l)) = (ctx.id_by_name(crate::present::feel::vcam::BRAIN), &r.brain) else {
        return Ok(());
    };
    if !ctx.has_resource_id(id)? {
        return Ok(());
    }
    let bytes = ctx.resource_bytes(id)?;
    let mut v = l.decode(&bytes);
    v["solo"] = json!(solo);
    v["soloBlend"] = json!(blend);
    v["soloEase"] = json!(ease.to_text());
    let out = l.encode(Some(&bytes), Some(&v)).map_err(invalid)?;
    if out != bytes {
        ctx.set_resource_bytes(id, &out)?;
    }
    Ok(())
}

fn camera(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    r: &Rec,
    lt: &Loaded,
    run: &mut Run,
    c: &og_timeline::CameraTrack,
    t: f32,
) -> og_core::Result<()> {
    let at = eval::camera_at(c, t);
    let bytes = ctx.resource_bytes(r.camera.0)?;
    let mut cam = r.camera.1.decode(&bytes);
    let row = record::Camera::from_json(&cam).map_err(invalid)?;
    if run.camera.is_none() {
        let (p, q) = (row.position, row.quaternion);
        run.camera = Some([p.x, p.y, p.z, q.x, q.y, q.z, q.w, row.fov]);
    }
    let (pos, target) = match &c.mode {
        CameraMode::Keys => (at.position, at.target),
        CameraMode::Rail(name) => {
            let p = lt.paths.get(name).cloned().or_else(|| {
                let reg = registry(engine_id);
                let r = lock(&reg);
                r.named.get(name).and_then(|h| r.path(*h))
            });
            let Some(p) = p else {
                note(
                    engine_id,
                    format!(
                        "timeline {}: the camera's rail '{name}' is neither a path of the timeline nor of a spawned scene",
                        lt.id
                    ),
                );
                return Ok(());
            };
            let s = p.sample(at.at.unwrap_or(0.0));
            (
                Some(s.position),
                Some(at.target.unwrap_or(s.position + s.tangent)),
            )
        }
        CameraMode::Follow(name) => {
            let Some(e) = ctx.find(name) else {
                note(
                    engine_id,
                    format!(
                        "timeline {}: no entity '{name}' for the camera to follow",
                        lt.id
                    ),
                );
                return Ok(());
            };
            let b = ctx.get_bytes(e, r.transform.0)?;
            let p = r
                .transform
                .1
                .read::<record::Transform>(&b)
                .map_err(invalid)?
                .position;
            (
                Some(p + at.position.unwrap_or(Vec3::new(0.0, 2.0, 6.0))),
                Some(p + at.target.unwrap_or(Vec3::new(0.0, 1.0, 0.0))),
            )
        }
    };
    if let Some(p) = pos {
        cam["position"] = jv3(p);
    }
    let eye = pos.unwrap_or(row.position);
    let look = target.map(|target| super::feel::look_rotation(eye, target));
    if let Some(q) = look {
        cam["quaternion"] = jq(q);
    }
    if let Some(roll) = at.roll {
        // About the view direction, from the look (or the camera it found):
        // never accumulated over ticks.
        let base = look.unwrap_or_else(|| {
            let c = run
                .camera
                .unwrap_or([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 50.0]);
            Quat::from_xyzw(c[3], c[4], c[5], c[6])
        });
        let (s, c) = og_core::math::sin_cos(roll * (std::f32::consts::PI / 360.0));
        cam["quaternion"] = jq((base * Quat::from_xyzw(0.0, 0.0, s, c)).normalize());
    }
    if let Some(f) = at.fov {
        cam["fov"] = json!(f);
    }
    let out = r
        .camera
        .1
        .encode(Some(&bytes), Some(&cam))
        .map_err(invalid)?;
    ctx.set_resource_bytes(r.camera.0, &out)?;
    // Focus: og.PostEffects' depth of field while the track owns the camera.
    let on = eval::focus_on_at(c, t);
    let distance = match on {
        Some(name) => match ctx.find(name) {
            Some(e) => {
                let b = ctx.get_bytes(e, r.transform.0)?;
                let focus: record::Transform = r.transform.1.read(&b).map_err(invalid)?;
                Some((focus.position - eye).length())
            }
            None => {
                note(
                    engine_id,
                    format!(
                        "timeline {}: no entity '{name}' for the camera to focus on",
                        lt.id
                    ),
                );
                None
            }
        },
        None => at.focus,
    };
    if distance.is_none() && at.focus_range.is_none() {
        return Ok(());
    }
    let Some(post) = r
        .post
        .as_ref()
        .filter(|_| super::granted(engine_id, "og.PostEffects"))
    else {
        return Ok(());
    };
    let (base, mut pv) = match ctx.resource_bytes(post.0) {
        Ok(b) => {
            let v = post.1.decode(&b);
            (Some(b), v)
        }
        Err(_) => (
            None,
            post.1.decode(&post.1.encode(None, None).map_err(invalid)?),
        ),
    };
    if run.focus.is_none() {
        let p = record::PostEffects::from_json(&pv).map_err(invalid)?;
        run.focus = Some([
            if p.depth_of_field { 1.0 } else { 0.0 },
            p.dof_focus_distance,
            p.dof_focus_range,
        ]);
    }
    pv["depthOfField"] = json!(true);
    if let Some(d) = distance {
        pv["dofFocusDistance"] = json!(d.max(0.01));
    }
    if let Some(range) = at.focus_range {
        pv["dofFocusRange"] = json!(range);
    }
    let out = post.1.encode(base.as_deref(), Some(&pv)).map_err(invalid)?;
    ctx.set_resource_bytes(post.0, &out)
}

/// The layout of a world read's record (`og.PointLight.intensity`) and the
/// field after it: the longest record name with a layout.
pub(super) fn record_of<'a>(
    layouts: &BTreeMap<String, Layout>,
    path: &'a str,
) -> Option<(&'a str, &'a str)> {
    path.rmatch_indices('.')
        .map(|(i, _)| (&path[..i], &path[i + 1..]))
        .find(|(rec, _)| layouts.contains_key(*rec))
}

/// The engine records a light, visibility or blendshape track may write
/// on its entity, in the order they are tried.
pub(super) fn track_records(kind: og_timeline::FieldsKind) -> &'static [&'static str] {
    use og_timeline::FieldsKind as K;
    match kind {
        K::Light => &[
            "og.DirectionalLight",
            "og.PointLight",
            "og.SpotLight",
            "og.RectLight",
            "og.Light2D",
        ],
        K::Visibility => &[
            "og.Mesh",
            "og.Model",
            "og.Sprite",
            "og.ParticleSystem",
            "og.Decal",
            "og.Lines",
            "og.Points",
            "og.Tilemap",
            "og.Rope",
            "og.Cloth",
            "og.SoftBody",
            "og.Terrain",
            "og.Fluid2D",
            "og.Fluid3D",
        ],
        K::Blendshape => &["og.Blendshapes"],
        K::Post => &["og.PostEffects"],
        K::Property | K::Material => &[],
    }
}

/// A value as the JSON its field's layout encodes: lanes for a vector,
/// colour or quaternion, a bool, an enum's variant, a number.
fn field_json(f: &crate::host::codec::FieldLayout, v: &og_timeline::FieldValue) -> Option<Value> {
    use crate::host::codec::Kind;
    use og_timeline::FieldValue as V;
    let lanes = f.kind.keys();
    Some(match (v, f.kind) {
        (V::Word(w), Kind::Bool) => json!(w == "true"),
        (V::Numbers(n), Kind::Bool) => json!(n.first().is_some_and(|x| *x != 0.0)),
        (V::Word(w), Kind::Enum | Kind::Ease) => json!(w),
        (V::Numbers(n), _) if !lanes.is_empty() => {
            let mut m = serde_json::Map::new();
            for (k, x) in lanes.iter().zip(n) {
                m.insert((*k).to_string(), json!(x));
            }
            Value::Object(m)
        }
        (V::Numbers(n), Kind::U32 | Kind::I32) => json!(n.first()?.round()),
        (V::Numbers(n), Kind::F32 | Kind::F64) => json!(n.first()?),
        _ => return None,
    })
}

/// How a field of `kind` moves between keys.
fn mix_of(kind: crate::host::codec::Kind) -> eval::Mix {
    use crate::host::codec::Kind;
    match kind {
        Kind::Quaternion => eval::Mix::Slerp,
        Kind::F32 | Kind::F64 | Kind::Vector2 | Kind::Vector3 | Kind::Color => eval::Mix::Lerp,
        _ => eval::Mix::Step,
    }
}

/// A property, light, visibility, post or blendshape track at `t`: each
/// field it sets, written into its record on its entity (or the resource).
fn fields(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    id: &str,
    f: &og_timeline::FieldsTrack,
    t: f32,
) -> og_core::Result<()> {
    use og_timeline::FieldsKind as K;
    let Ok(eng) = engine(engine_id) else {
        return Ok(());
    };
    let layouts = eng.state.borrow().layouts.clone();
    let entity = if f.entity.is_empty() {
        None
    } else {
        match ctx.find(&f.entity) {
            Some(e) => Some(e),
            None => {
                note(
                    engine_id,
                    format!(
                        "timeline {id}: no entity '{}' for its {} track",
                        f.entity,
                        f.kind.name()
                    ),
                );
                return Ok(());
            }
        }
    };
    // (record, field) for each field the keys set.
    let mut targets: Vec<(String, String, String)> = Vec::new();
    for name in f.fields() {
        match f.kind {
            K::Property => match record_of(&layouts, name) {
                Some((rec, field)) => targets.push((rec.into(), field.into(), name.into())),
                None => note(
                    engine_id,
                    format!("timeline {id}: '{name}' names no component or resource this game has"),
                ),
            },
            K::Post => targets.push(("og.PostEffects".into(), name.into(), name.into())),
            _ => {
                let Some(e) = entity else { continue };
                let mut any = false;
                for rec in track_records(f.kind) {
                    let Ok(sid) = ctx.id_by_name(rec) else {
                        continue;
                    };
                    if ctx.get_bytes(e, sid).is_ok()
                        && layouts.get(*rec).is_some_and(|l| l.field(name).is_some())
                    {
                        targets.push(((*rec).into(), name.into(), name.into()));
                        any = true;
                        if f.kind == K::Light {
                            break;
                        }
                    }
                }
                if !any && f.kind == K::Blendshape {
                    targets.push(("og.Blendshapes".into(), name.into(), name.into()));
                } else if !any {
                    note(
                        engine_id,
                        format!(
                            "timeline {id}: '{}' has no {} with '{name}' for its {} track",
                            f.entity,
                            track_records(f.kind).join(" or "),
                            f.kind.name()
                        ),
                    );
                }
            }
        }
    }
    // One write per record.
    let mut recs: Vec<&str> = targets.iter().map(|x| x.0.as_str()).collect();
    recs.dedup();
    for rec in recs {
        if !super::granted(engine_id, rec) {
            note(
                engine_id,
                format!(
                    "timeline {id}: {rec} was not registered when the timeline loaded; its {} track writes nothing to it",
                    f.kind.name()
                ),
            );
            continue;
        }
        let Some(layout) = layouts.get(rec) else {
            continue;
        };
        let Ok(sid) = ctx.id_by_name(rec) else {
            continue;
        };
        let resource = layout.kind == crate::host::codec::RecordKind::Resource;
        let base = match (resource, entity) {
            (true, _) => ctx.resource_bytes(sid).ok(),
            (false, Some(e)) => ctx.get_bytes(e, sid).ok(),
            (false, None) => {
                note(
                    engine_id,
                    format!(
                        "timeline {id}: {rec} is a component: name the entity, `track {} \"crate\" {{`",
                        f.kind.name()
                    ),
                );
                continue;
            }
        };
        let mut v = match &base {
            Some(b) => layout.decode(b),
            None => json!({}),
        };
        if f.kind == K::Blendshape {
            v["source"] = json!("component");
        }
        for (_, field, name) in targets.iter().filter(|x| x.0 == rec) {
            let Some((_, fl)) = layout.field(field) else {
                note(
                    engine_id,
                    format!("timeline {id}: {rec} has no field '{field}'"),
                );
                continue;
            };
            let Some(value) = eval::field_at(f, name, t, mix_of(fl.kind)) else {
                continue;
            };
            match field_json(fl, &value) {
                Some(j) => v[field.as_str()] = j,
                None => note(
                    engine_id,
                    format!("timeline {id}: {rec}.{field} cannot take {value:?}"),
                ),
            }
        }
        let out = layout.encode(base.as_deref(), Some(&v)).map_err(invalid)?;
        match (resource, entity, base.is_some()) {
            (true, _, _) => ctx.set_resource_bytes(sid, &out)?,
            (false, Some(e), true) => ctx.set_bytes(e, sid, &out)?,
            (false, Some(e), false) => ctx.insert_bytes(e, sid, &out)?,
            _ => {}
        }
    }
    Ok(())
}

fn caption(
    ui: &og_ui::SharedUi,
    ti: u32,
    k: &og_timeline::CaptionKey,
    c: &og_timeline::CaptionTrack,
) {
    let mut ui = ui.lock();
    let theme = *ui.theme();
    let style = k.style.as_deref().unwrap_or(&c.style);
    let anchor = k.anchor.as_deref().unwrap_or(&c.anchor);
    let id = format!("timeline-caption-{ti}");
    ui.open_anchor(
        id.as_str(),
        og_ui::AnchorDesc {
            anchor: super::modules::anchor_of(&json!(anchor)).unwrap_or(og_ui::Anchor::Bottom),
            padding: og_ui::Insets::xy(16.0, 8.0),
            background: Some(og_ui::Srgba {
                rgb: [0, 0, 0],
                alpha: 0.62,
            }),
            radius: 8.0,
            over: true,
            instant: true,
            ..Default::default()
        },
    );
    ui.label(
        format!("{id}-text").as_str(),
        &k.text,
        og_ui::LabelDesc {
            step: super::modules::step(&theme, style),
            color: Some(og_ui::Srgba {
                rgb: [255, 255, 255],
                alpha: 1.0,
            }),
            align: og_ui::TextAlign::Center,
            ..Default::default()
        },
    );
    ui.close();
}
