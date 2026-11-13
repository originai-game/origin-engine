//! Transform, random, audio, input and camera binding calls.

use super::*;
use crate::host::wire::{FromJson, record};

// ── transform ───────────────────────────────────────────────────────────────

fn vec3_json(v: glam::Vec3) -> Value {
    json!({ "x": f64::from(v.x), "y": f64::from(v.y), "z": f64::from(v.z) })
}

fn quat_json(q: glam::Quat) -> Value {
    json!({ "x": f64::from(q.x), "y": f64::from(q.y), "z": f64::from(q.z), "w": f64::from(q.w) })
}

pub fn og_transform_compose(position: String, quaternion: String, scale: String) -> glam::Mat4 {
    let p: glam::Vec3 = wire::arg(Some(&position), "position").unwrap_or_default();
    let q: glam::Quat = wire::arg(Some(&quaternion), "quaternion").unwrap_or_default();
    let s: glam::Vec3 = wire::arg(Some(&scale), "scale").unwrap_or_default();
    glam::Mat4::from_scale_rotation_translation(s, q, p)
}

pub fn og_transform_decompose(matrix: String) -> Value {
    let m: glam::Mat4 = wire::arg(Some(&matrix), "matrix").unwrap_or_default();
    let (s, q, p) = m.to_scale_rotation_translation();
    json!({ "position": vec3_json(p), "quaternion": quat_json(q), "scale": vec3_json(s) })
}

/// The rotation that points -Z from `eye` towards `target`, +Y up by default.
pub fn og_transform_look_at(eye: String, target: String, up: Option<String>) -> glam::Quat {
    let eye: glam::Vec3 = wire::arg(Some(&eye), "eye").unwrap_or_default();
    let target: glam::Vec3 = wire::arg(Some(&target), "target").unwrap_or_default();
    let up: glam::Vec3 = wire::arg(up.as_deref(), "up").unwrap_or_default();
    og_core::facing::camera_look(eye, target, up)
}

/// The rotation that points an object's +Z from `position` towards `target`.
pub fn og_transform_face(position: String, target: String, up: Option<String>) -> glam::Quat {
    let position: glam::Vec3 = wire::arg(Some(&position), "position").unwrap_or_default();
    let target: glam::Vec3 = wire::arg(Some(&target), "target").unwrap_or_default();
    let up: glam::Vec3 = wire::arg(up.as_deref(), "up").unwrap_or_default();
    og_core::facing::look(position, target, up)
}

/// A rotation argument as a unit quaternion (one that is not a rotation:
/// the identity).
pub(crate) fn rotation_arg(quaternion: &str, name: &str) -> glam::Quat {
    let q: glam::Quat = wire::arg(Some(quaternion), name).unwrap_or_default();
    if q.is_finite() && q.length_squared() > 0.0 {
        q.normalize()
    } else {
        glam::Quat::IDENTITY
    }
}

pub fn og_transform_forward(quaternion: String) -> glam::Vec3 {
    og_core::facing::forward(rotation_arg(&quaternion, "quaternion"))
}

pub fn og_transform_right(quaternion: String) -> glam::Vec3 {
    og_core::facing::right(rotation_arg(&quaternion, "quaternion"))
}

pub fn og_transform_up(quaternion: String) -> glam::Vec3 {
    og_core::facing::up(rotation_arg(&quaternion, "quaternion"))
}

pub fn og_transform_camera_forward(quaternion: String) -> glam::Vec3 {
    og_core::facing::camera_forward(rotation_arg(&quaternion, "quaternion"))
}

pub fn og_transform_camera_right(quaternion: String) -> glam::Vec3 {
    og_core::facing::camera_right(rotation_arg(&quaternion, "quaternion"))
}

pub fn og_transform_from_axis_angle(axis: String, angle: f64) -> glam::Quat {
    let axis: glam::Vec3 = wire::arg(Some(&axis), "axis").unwrap_or_default();
    og_core::math::quat_from_axis_angle(axis, angle as f32)
}

// ── rng ─────────────────────────────────────────────────────────────────────

/// Draws from the world's Rng resource. Inside a system the resource must
/// be among its writes (a draw moves its state); otherwise the draw is
/// refused, logged once (`undeclared`) and answers the refused value (NaN).
fn with_rng<T>(world: u32, refused: T, f: impl FnOnce(&mut Rng) -> T) -> T {
    if let Some(frame) = running(world) {
        // SAFETY: the frame is published while its system runs.
        let ctx = unsafe { frame.ctx() };
        return match ctx.resource_mut::<Rng>() {
            Ok(mut r) => f(&mut r),
            Err(_) => {
                if let Ok(eng) = engine(world) {
                    undeclared(&eng, frame.system, "write", Rng::NAME);
                }
                refused
            }
        };
    }
    match engine(world).and_then(|e| {
        e.world()
            .and_then(|w| w.resource_mut::<Rng>().map_err(err))
            .map(f)
    }) {
        Ok(v) => v,
        Err(_) => refused,
    }
}

/// Reads the world's Rng resource without moving it: a fork derives a
/// stream from the seed alone, so a system with og.Rng among its reads may
/// fork (the parent is untouched and the child is the same on every host).
pub(crate) fn read_rng<T>(world: u32, f: impl FnOnce(&Rng) -> T) -> Option<T> {
    if let Some(frame) = running(world) {
        // SAFETY: the frame is published while its system runs.
        let ctx = unsafe { frame.ctx() };
        return match ctx.resource::<Rng>() {
            Ok(r) => Some(f(&r)),
            Err(_) => {
                if let Ok(eng) = engine(world) {
                    undeclared(&eng, frame.system, "read", Rng::NAME);
                }
                None
            }
        };
    }
    engine(world)
        .and_then(|e| {
            e.world()
                .and_then(|w| w.resource::<Rng>().map_err(err))
                .map(|r| f(&r))
        })
        .ok()
}

fn with_stream<T>(stream: u32, refused: T, f: impl FnOnce(&mut Rng) -> T) -> T {
    HANDLES.with(|t| {
        let mut t = t.borrow_mut();
        let base = t.streams.base;
        match stream
            .checked_sub(base)
            .and_then(|i| t.streams.items.get_mut(i as usize))
        {
            Some((_, r)) => f(r),
            None => refused,
        }
    })
}

fn chance(r: &mut Rng, p: f64) -> bool {
    r.next_f64() < p
}

fn integer(r: &mut Rng, low: i32, high: i32) -> i32 {
    if high < low {
        return low;
    }
    r.integer(i64::from(low), i64::from(high)) as i32
}

fn new_stream(world: u32, rng: Rng) -> u32 {
    HANDLES.with(|t| t.borrow_mut().streams.push((world, rng)))
}

pub fn og_rng_stream_get_seed(stream: u32) -> u32 {
    with_stream(stream, 0, |r| r.seed())
}
pub fn og_rng_stream_next(stream: u32) -> f64 {
    with_stream(stream, f64::NAN, Rng::next_f64)
}
pub fn og_rng_stream_range(stream: u32, low: f64, high: f64) -> f64 {
    with_stream(stream, f64::NAN, |r| r.range(low, high))
}
pub fn og_rng_stream_integer(stream: u32, low: i32, high: i32) -> i32 {
    with_stream(stream, 0, |r| integer(r, low, high))
}
pub fn og_rng_stream_chance(stream: u32, p: f64) -> bool {
    with_stream(stream, false, |r| chance(r, p))
}
pub fn og_rng_stream_fork(stream: u32, label: String) -> u32 {
    let owner = HANDLES.with(|t| {
        let t = t.borrow();
        t.streams.get(stream).map(|(w, r)| (*w, r.fork(&label)))
    });
    match owner {
        Some((w, r)) => new_stream(w, r),
        None => 0,
    }
}
/// The weights a `weighted` call passes (a JSON list); none on bad JSON.
fn weights(text: &str) -> Vec<f64> {
    serde_json::from_str::<Vec<Option<f64>>>(text)
        .map(|w| w.into_iter().map(|x| x.unwrap_or(f64::NAN)).collect())
        .unwrap_or_default()
}

fn weighted(r: &mut Rng, text: &str) -> i32 {
    og_core::procgen::weighted(r, &weights(text)).map_or(-1, |i| i as i32)
}

fn permutation(r: &mut Rng, n: u32) -> Vec<u32> {
    og_core::procgen::permutation(r, n.min(1 << 24))
}

pub fn og_rng_stream_weighted(stream: u32, weights: String) -> i32 {
    with_stream(stream, -1, |r| weighted(r, &weights))
}
pub fn og_rng_stream_permutation(stream: u32, n: u32) -> Vec<u32> {
    with_stream(stream, Vec::new(), |r| permutation(r, n))
}
pub fn og_rng_weighted(world: u32, weights: String) -> i32 {
    with_rng(world, -1, |r| weighted(r, &weights))
}
pub fn og_rng_permutation(world: u32, n: u32) -> Vec<u32> {
    with_rng(world, Vec::new(), |r| permutation(r, n))
}
pub fn og_rng_next(world: u32) -> f64 {
    with_rng(world, f64::NAN, Rng::next_f64)
}
pub fn og_rng_range(world: u32, low: f64, high: f64) -> f64 {
    with_rng(world, f64::NAN, |r| r.range(low, high))
}
pub fn og_rng_integer(world: u32, low: i32, high: i32) -> i32 {
    with_rng(world, 0, |r| integer(r, low, high))
}
pub fn og_rng_chance(world: u32, p: f64) -> bool {
    with_rng(world, false, |r| chance(r, p))
}
pub fn og_rng_fork(world: u32, label: String) -> u32 {
    match read_rng(world, |r| r.fork(&label)) {
        Some(r) => new_stream(world, r),
        None => 0,
    }
}

// ── audio ───────────────────────────────────────────────────────────────────

pub fn og_audio_play_feedback(
    world: u32,
    sound: f64,
    desc: Option<String>,
) -> Result<Value, String> {
    (|| {
        let eng = engine(world)?;
        assets::sound_index(&eng, sound).ok_or("feedback sound is not loaded")?;
        let tick = if let Some(frame) = running(world) {
            // SAFETY: the frame is published while its system runs.
            unsafe { frame.ctx() }.time().tick
        } else {
            eng.world()?.time().tick
        };
        let desc: wire::AudioFeedbackDesc = wire::arg(desc.as_deref(), "desc")?;
        let link = og_audio::link(world).ok_or("world has no audio sink")?;
        link.play_feedback(tick, sound as u64, desc.volume, desc.playback_rate)?;
        Ok(Value::Null)
    })()
}

/// `og.audio.outputLatency`: og.Audio's `outputLatency`, which the host's
/// sink reports between ticks (`set_audio_latency`). Inside a system og.Audio
/// must be among its reads or writes (refused and logged once otherwise: 0).
pub fn og_audio_output_latency(world: u32) -> f64 {
    if let Some(frame) = running(world) {
        // SAFETY: the frame is published while its system runs.
        return match unsafe { frame.ctx() }.resource::<og_audio::Audio>() {
            Ok(a) => a.output_latency,
            Err(_) => {
                if let Ok(eng) = engine(world) {
                    undeclared(&eng, frame.system, "read", "og.Audio");
                }
                0.0
            }
        };
    }
    engine(world)
        .and_then(|e| {
            e.world().and_then(|w| {
                w.resource::<og_audio::Audio>()
                    .map_err(err)
                    .map(|a| a.output_latency)
            })
        })
        .unwrap_or(0.0)
}

/// `og.audio.createSound`: PCM from the game, clamped to 16 bits and put in
/// the bank resident, and listed among the loaded sounds so soundInfo,
/// timelines and themes find it by handle.
pub fn og_audio_create_sound(
    world: u32,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    (|| {
        let eng = engine(world)?;
        let d: wire::SoundDesc = wire::arg(desc.as_deref(), "desc")?;
        let (rate, channels) = (d.rate, d.channels);
        if !(8_000..=192_000).contains(&rate) {
            return Err(format!("rate {rate} is out of range: 8000 to 192000"));
        }
        if !(1..=2).contains(&channels) {
            return Err(format!("channels {channels} is out of range: 1 or 2"));
        }
        let mut pcm = Vec::with_capacity(d.samples.len());
        for x in d.samples {
            if !x.is_finite() {
                return Err("samples holds something that is not a finite number".into());
            }
            pcm.push((f64::from(x).clamp(-1.0, 1.0) * 32767.0).round() as i16);
        }
        if pcm.is_empty() || !pcm.len().is_multiple_of(channels as usize) {
            return Err(format!(
                "samples holds {} values: at least one frame of {channels} channel(s), whole frames",
                pcm.len()
            ));
        }
        let link = og_audio::link(eng.id).ok_or("this engine has no audio")?;
        let id = format!("og.audio.createSound#{}", eng.assets.borrow().sounds.len());
        let handle = link
            .bank()
            .insert_pcm(&id, rate, channels as usize, &pcm)
            .map_err(|e| e.reason().to_string())?;
        let mut a = eng.assets.borrow_mut();
        a.sounds.push(og_bundle::runtime::CookedSound {
            id,
            meta: og_bundle::format::SoundMeta {
                sample_rate: rate,
                channels,
                frames: (pcm.len() / channels as usize) as u32,
                chunk_frames: 0,
                chunk_count: 0,
                codec: og_bundle::format::sound_codec::QOA,
            },
            qoa: Vec::new(),
            opus: Vec::new(),
        });
        a.sound_handles.push(handle);
        Ok(json!(handle as f64))
    })()
}

/// The host's sink reports its output latency (seconds from a sample's world
/// time to the speaker: the device's own and the buffering in front of it),
/// between ticks; it is host input, written to og.Audio like an input event.
pub fn set_audio_latency(engine_id: u32, seconds: f64) -> Result<(), String> {
    if running(engine_id).is_some() {
        return Err("the audio latency is host input: report it between ticks".into());
    }
    let seconds = if seconds.is_finite() {
        seconds.clamp(0.0, 10.0)
    } else {
        0.0
    };
    let eng = engine(engine_id)?;
    let w = eng.world()?;
    let a = w.resource_mut::<og_audio::Audio>().map_err(err)?;
    if a.output_latency != seconds {
        a.output_latency = seconds;
    }
    Ok(())
}

// ── input ───────────────────────────────────────────────────────────────────

/// The `og.Input` resource as of this tick (inside a system, through its
/// context; the resource is og-probe's input layer until og-input owns it).
fn input<T>(world: u32, f: impl FnOnce(&og_probe::Input) -> T, refused: T) -> T {
    if let Some(fr) = running(world) {
        // SAFETY: the frame is published while its system runs.
        return match unsafe { fr.ctx() }.resource::<og_probe::Input>() {
            Ok(i) => f(&i),
            Err(_) => refused,
        };
    }
    match engine(world).and_then(|eng| {
        eng.world()?
            .resource::<og_probe::Input>()
            .map(|i| *i)
            .map_err(err)
    }) {
        Ok(i) => f(&i),
        Err(_) => refused,
    }
}

fn button(name: &str) -> og_probe::input::PointerButton {
    use og_probe::input::PointerButton as B;
    match name {
        "left" => B::Left,
        "middle" => B::Middle,
        "right" => B::Right,
        _ => B::None,
    }
}

pub fn og_input_down(world: u32, key: String) -> bool {
    input(world, |i| i.down(&key), false)
}
pub fn og_input_pressed(world: u32, key: String) -> bool {
    input(world, |i| i.pressed(&key), false)
}
pub fn og_input_released(world: u32, key: String) -> bool {
    input(world, |i| i.released(&key), false)
}
pub fn og_input_axis(world: u32, negative: String, positive: String) -> f64 {
    input(world, |i| f64::from(i.axis(&negative, &positive)), 0.0)
}
pub fn og_input_axis2(
    world: u32,
    left: String,
    right: String,
    down: String,
    up: String,
) -> glam::Vec2 {
    input(
        world,
        |i| i.axis2(&left, &right, &down, &up),
        og_core::glam::Vec2::ZERO,
    )
}
pub fn og_input_button_down(world: u32, b: String) -> bool {
    input(world, |i| i.button_down(button(&b)), false)
}
pub fn og_input_button_pressed(world: u32, b: String) -> bool {
    input(world, |i| i.button_pressed(button(&b)), false)
}
pub fn og_input_button_released(world: u32, b: String) -> bool {
    input(world, |i| i.button_released(button(&b)), false)
}

/// A copy of an input resource as of this tick (inside a system, through
/// its context, which refuses it unless the system reads it).
pub(super) fn input_res<R: og_core::Reflect + Copy + Send + Sync + 'static, T>(
    world: u32,
    f: impl FnOnce(&R) -> T,
    refused: T,
) -> T {
    if let Some(fr) = running(world) {
        // SAFETY: the frame is published while its system runs.
        return match unsafe { fr.ctx() }.resource::<R>() {
            Ok(r) => f(&r),
            Err(_) => refused,
        };
    }
    match engine(world).and_then(|eng| eng.world()?.resource::<R>().map(|r| *r).map_err(err)) {
        Ok(r) => f(&r),
        Err(_) => refused,
    }
}

pub fn og_pointers_down(world: u32, id: u32) -> bool {
    input_res(world, |p: &og_input::Pointers| p.down(id), false)
}
pub fn og_pointers_pressed(world: u32, id: u32) -> bool {
    input_res(world, |p: &og_input::Pointers| p.is_pressed(id), false)
}
pub fn og_pointers_released(world: u32, id: u32) -> bool {
    input_res(world, |p: &og_input::Pointers| p.is_released(id), false)
}
pub fn og_pointers_position(world: u32, id: u32) -> glam::Vec2 {
    input_res(
        world,
        |p: &og_input::Pointers| p.position(id),
        Default::default(),
    )
}
pub fn og_pointers_start(world: u32, id: u32) -> glam::Vec2 {
    input_res(
        world,
        |p: &og_input::Pointers| p.start(id),
        Default::default(),
    )
}
pub fn og_pointers_count(world: u32) -> u32 {
    input_res(world, |p: &og_input::Pointers| p.count(), 0)
}
pub fn og_gamepad_connected(world: u32, pad: Option<u32>) -> bool {
    input_res(
        world,
        |g: &og_input::Gamepad| g.is_connected(pad.unwrap_or(0)),
        false,
    )
}
fn pad_button(button: &str) -> Option<usize> {
    og_input::gamepad_button(button)
}
pub fn og_gamepad_down(world: u32, button: String, pad: Option<u32>) -> bool {
    pad_button(&button).is_some_and(|b| {
        input_res(
            world,
            |g: &og_input::Gamepad| g.down(b, pad.unwrap_or(0)),
            false,
        )
    })
}
pub fn og_gamepad_pressed(world: u32, button: String, pad: Option<u32>) -> bool {
    pad_button(&button).is_some_and(|b| {
        input_res(
            world,
            |g: &og_input::Gamepad| g.pressed(b, pad.unwrap_or(0)),
            false,
        )
    })
}
pub fn og_gamepad_released(world: u32, button: String, pad: Option<u32>) -> bool {
    pad_button(&button).is_some_and(|b| {
        input_res(
            world,
            |g: &og_input::Gamepad| g.released(b, pad.unwrap_or(0)),
            false,
        )
    })
}
pub fn og_gamepad_axis(world: u32, axis: String, pad: Option<u32>) -> f64 {
    match og_input::GamepadAxis::from_name(&axis) {
        Some(a) => input_res(
            world,
            |g: &og_input::Gamepad| f64::from(g.axis(a, pad.unwrap_or(0))),
            0.0,
        ),
        None => 0.0,
    }
}

// ── camera ──────────────────────────────────────────────────────────────────

/// The play surface in CSS pixels (the unit of og.Input and the probe).
fn surface_size(eng: &Engine) -> (f32, f32) {
    let v = super::modules::viewport_of(eng.id);
    (v.width.max(1.0), v.height.max(1.0))
}

pub fn og_camera_look_at(world: u32, target: String) {
    let _ = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let cam = record::Camera::from_json(&resource_value(&eng, "og.Camera")?)?;
        let q = og_core::facing::camera_look(
            cam.position,
            wire::arg(Some(&target), "target")?,
            og_core::facing::UP,
        );
        let value = json!({ "quaternion": quat_json(q) });
        let answer = super::engine::write_resource(world, "og.Camera", &value.to_string());
        if answer.contains("\"ok\":false") {
            return Err(answer);
        }
        Ok(())
    })();
}

fn view_proj(eng: &Engine) -> Result<(glam::Mat4, glam::Mat4, f32, f32), String> {
    let cam = record::Camera::from_json(&resource_value(eng, "og.Camera")?)?;
    let (w, h) = surface_size(eng);
    let vp = camera_view_proj(&cam, w, h);
    Ok((vp, vp.inverse(), w, h))
}

/// [`camera_of`] of og.Camera's JSON (a value that is not one: the default
/// camera).
pub(crate) fn camera_view(cam: &Value, w: f32, h: f32) -> og_ui::WorldView {
    camera_of(&record::Camera::from_json(cam).unwrap_or_default(), w, h)
}

/// og.Camera's projection times view on a `w`×`h` surface, and its eye,
/// forward axis and whether it is orthographic: what og.camera's
/// worldToScreen projects with and the UI's world anchors follow.
pub(crate) fn camera_of(cam: &record::Camera, w: f32, h: f32) -> og_ui::WorldView {
    og_ui::WorldView {
        view_proj: camera_view_proj(cam, w, h),
        eye: cam.position,
        forward: og_core::facing::camera_forward(cam.quaternion),
        orthographic: cam.orthographic,
    }
}

fn camera_view_proj(cam: &record::Camera, w: f32, h: f32) -> glam::Mat4 {
    let view = glam::Mat4::from_rotation_translation(cam.quaternion, cam.position).inverse();
    let (near, far) = (cam.near, cam.far);
    let zoom = cam.zoom.max(0.01);
    let proj = if cam.orthographic {
        let half_h = cam.height / 2.0 / zoom;
        let half_w = half_h * w / h;
        let r = 1.0 / (near - far);
        glam::Mat4::from_cols(
            glam::Vec4::new(1.0 / half_w, 0.0, 0.0, 0.0),
            glam::Vec4::new(0.0, 1.0 / half_h, 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, r, 0.0),
            glam::Vec4::new(0.0, 0.0, r * near, 1.0),
        )
    } else {
        // perspective_rh by hand: glam's calls the host libm, og_core::math does not.
        let half = cam.fov.to_radians() / 2.0;
        let (s, c) = og_core::math::sin_cos(half);
        let f = c / s * zoom;
        let r = far / (near - far);
        glam::Mat4::from_cols(
            glam::Vec4::new(f / (w / h), 0.0, 0.0, 0.0),
            glam::Vec4::new(0.0, f, 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, r, -1.0),
            glam::Vec4::new(0.0, 0.0, r * near, 0.0),
        )
    };
    proj * view
}

pub fn og_camera_world_to_screen(world: u32, point: String) -> Option<glam::Vec2> {
    let answer = (|| -> Result<Option<glam::Vec2>, String> {
        let eng = engine(world)?;
        let (vp, _, w, h) = view_proj(&eng)?;
        let p = vp * wire::arg::<glam::Vec3>(Some(&point), "point")?.extend(1.0);
        if p.w <= 0.0 {
            return Ok(None);
        }
        let ndc = p.truncate() / p.w;
        Ok(Some(glam::Vec2::new(
            (ndc.x + 1.0) / 2.0 * w,
            (1.0 - ndc.y) / 2.0 * h,
        )))
    })();
    answer.ok().flatten()
}

pub fn og_camera_screen_to_ray(world: u32, x: f64, y: f64) -> crate::host::wire::Ray {
    let answer = (|| -> Result<crate::host::wire::Ray, String> {
        let eng = engine(world)?;
        let (_, inv, w, h) = view_proj(&eng)?;
        let nx = x as f32 / w * 2.0 - 1.0;
        let ny = 1.0 - y as f32 / h * 2.0;
        let near = inv.project_point3(glam::Vec3::new(nx, ny, 0.0));
        let far = inv.project_point3(glam::Vec3::new(nx, ny, 1.0));
        Ok(wire::Ray {
            origin: near,
            direction: (far - near).normalize_or_zero(),
        })
    })();
    answer.unwrap_or(wire::Ray {
        origin: glam::Vec3::ZERO,
        direction: og_core::facing::CAMERA_FORWARD,
    })
}
