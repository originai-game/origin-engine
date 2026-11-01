//! Timeline tracks evaluated into world and UI state.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn evaluate(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    r: &Rec,
    ui: &og_ui::SharedUi,
    lt: &mut Loaded,
    run: &mut Run,
    t: f32,
    playing: bool,
    spans: &[(f64, f64)],
    retro: Option<(f64, f64)>,
    dt: f32,
    speed: f32,
) -> og_core::Result<()> {
    let duration = lt.timeline.duration;
    // One-shot voices that ended go; while paused every voice goes (a
    // resume starts the music again at its offset).
    let mut keep = Vec::new();
    for (tr, k, v) in std::mem::take(&mut run.voices) {
        let e = Entity::from_bits(v);
        let alive = ctx.is_alive(e)
            && ctx
                .get_bytes(e, r.source.0)
                .is_ok_and(|b| r.source.1.decode(&b)["playing"].as_bool().unwrap_or(false));
        if alive && playing {
            keep.push((tr, k, v));
        } else if ctx.is_alive(e) {
            ctx.despawn(e)?;
        }
    }
    run.voices = keep;
    let tracks = lt.timeline.tracks.clone();
    for (ti, tr) in tracks.iter().enumerate() {
        let ti32 = ti as u32;
        match tr {
            Track::Camera(c) => camera(ctx, engine_id, r, lt, run, c, t)?,
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
                        "sound": handle as f64, "bus": k.bus, "volume": k.volume, "pitch": k.pitch,
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
                    caption(ui, ti, k, c);
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
    let asset = r.model.1.decode(&b)["asset"].as_f64().unwrap_or(0.0);
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
    if run.camera.is_none() {
        let p = v3(&cam["position"]);
        let q = quat(&cam["quaternion"]);
        run.camera = Some([
            p.x,
            p.y,
            p.z,
            q.x,
            q.y,
            q.z,
            q.w,
            cam["fov"].as_f64().unwrap_or(50.0) as f32,
        ]);
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
            let p = v3(&r.transform.1.decode(&b)["position"]);
            (
                Some(p + at.position.unwrap_or(Vec3::new(0.0, 2.0, 6.0))),
                Some(p + at.target.unwrap_or(Vec3::new(0.0, 1.0, 0.0))),
            )
        }
    };
    if let Some(p) = pos {
        cam["position"] = jv3(p);
    }
    if let Some(target) = target {
        let eye = pos.unwrap_or_else(|| v3(&cam["position"]));
        cam["quaternion"] = jq(super::feel::look_rotation(eye, target));
    }
    if let Some(f) = at.fov {
        cam["fov"] = json!(f);
    }
    let out = r
        .camera
        .1
        .encode(Some(&bytes), Some(&cam))
        .map_err(invalid)?;
    ctx.set_resource_bytes(r.camera.0, &out)
}

fn caption(
    ui: &og_ui::SharedUi,
    ti: usize,
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
