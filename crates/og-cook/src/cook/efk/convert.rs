//! An Effekseer node tree as the engine's own effect: each drawn node
//! becomes an element of a `type vfx` (an emitter of `type particles`, a
//! mesh, a trail) with the `type effect` materials and meshes it needs.
//!
//! Effekseer spawns a child's instances per instance of its parent. The
//! importer walks each drawn node's chain of ancestors: ancestors with one
//! instance become the element's place, rotation, scale and start time;
//! the deepest node with many instances is the emitter's generator (its
//! count, its spawn times, where it spawns and how it moves), the drawn node
//! gives the look (its life, size, colour, texture). What that leaves out is
//! a finding, never dropped unsaid.

use super::curve;
use super::glb;
use super::project::{Draw, Node, Project, Rotation, Spawn, Uv};
use super::value::{Anim, Color, Rng, at3};
use glam::{Mat3, Quat, Vec3};
use std::fmt::Write as _;

/// Effekseer's frames per second.
pub const FPS: f32 = 60.0;
/// Elements one effect holds (og_bundle::vfx::MAX_ELEMENTS).
const MAX_ELEMENTS: usize = og_bundle::vfx::MAX_ELEMENTS;
/// Copies of a mesh element made for a node with a few instances.
const MAX_COPIES: u32 = 4;

/// Something of the source the conversion could not carry, by node.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    /// `<path> "<name>"`, or empty for the whole effect.
    pub node: String,
    /// A short category (the same words for the same loss), then details.
    pub what: String,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.node.is_empty() {
            f.write_str(&self.what)
        } else {
            write!(f, "node {}: {}", self.node, self.what)
        }
    }
}

/// What a generated file is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutKind {
    Emitter,
    Effect,
    Model,
}

/// A file the conversion writes beside the effect.
#[derive(Clone, Debug, PartialEq)]
pub struct Out {
    pub id: String,
    pub kind: OutKind,
    pub bytes: Vec<u8>,
}

/// A converted effect.
#[derive(Clone, Debug, PartialEq)]
pub struct Conversion {
    /// The `type vfx` text.
    pub vfx: String,
    pub files: Vec<Out>,
    /// The project's textures it uses: (path as the project names it, asset id).
    pub textures: Vec<(String, String)>,
    pub findings: Vec<Finding>,
}

/// How the conversion names and reads what it refers to.
pub struct Links<'a> {
    /// The asset id of a generated file by its leaf name (`n0-2.ogt`).
    pub own: &'a dyn Fn(&str) -> String,
    /// The asset id of a file the project names (relative to it).
    pub asset: &'a dyn Fn(&str) -> String,
    /// The bytes of a file the project names, when it is there.
    pub read: &'a dyn Fn(&str) -> Option<Vec<u8>>,
}

/// A number as the text formats write it: at most 4 decimals, no trailing zeros.
fn n(x: f32) -> String {
    let x = if x.abs() < 5e-5 { 0.0 } else { x };
    let s = format!("{x:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.to_string() }
}

fn ns(xs: &[f32]) -> String {
    xs.iter().map(|&x| n(x)).collect::<Vec<_>>().join(" ")
}

fn linear(c: f32) -> f32 {
    og_core::math::pow(c.clamp(0.0, 64.0), 2.2)
}

fn quat_axis(axis: Vec3, deg: f32) -> Quat {
    og_core::math::quat_from_axis_angle(axis, deg.to_radians())
}

/// Effekseer's rotation (degrees, z then x then y).
fn efk_quat(r: Vec3) -> Quat {
    quat_axis(Vec3::Y, r.y) * quat_axis(Vec3::X, r.x) * quat_axis(Vec3::Z, r.z)
}

/// A rotation as three.js's Euler XYZ, degrees (the vfx `rotate` line).
fn euler_xyz(q: Quat) -> Vec3 {
    let m = Mat3::from_quat(q);
    let m13 = m.col(2)[0].clamp(-1.0, 1.0);
    let y = og_core::math::atan2(m13, (1.0 - m13 * m13).max(0.0).sqrt());
    let (x, z) = if m13.abs() < 0.999_999 {
        (
            og_core::math::atan2(-m.col(2)[1], m.col(2)[2]),
            og_core::math::atan2(-m.col(1)[0], m.col(0)[0]),
        )
    } else {
        (og_core::math::atan2(m.col(1)[2], m.col(1)[1]), 0.0)
    };
    Vec3::new(x, y, z) * (180.0 / std::f32::consts::PI)
}

/// The rotation an Effekseer node starts with.
fn start_rotation(r: &Rotation) -> Quat {
    match r {
        Rotation::Euler(a) => efk_quat(at3(a, 0.0, 1.0)),
        Rotation::Axis { axis, angle } => quat_axis(axis.normalize_or(Vec3::Y), angle.at(0.0, 1.0)),
        Rotation::Other(_) => Quat::IDENTITY,
    }
}

/// Degrees per second a node turns about z (a billboard's roll, a ring's spin).
fn spin_z(r: &Rotation, life: f32) -> (Rng, Rng) {
    match r {
        Rotation::Euler(a) => (a[2].start(), a[2].rate(life).scaled(FPS)),
        Rotation::Axis { axis, angle } if axis.normalize_or(Vec3::Z).z.abs() > 0.9 => {
            let s = axis.z.signum();
            (angle.start().scaled(s), angle.rate(life).scaled(FPS * s))
        }
        _ => (Rng::fixed(0.0), Rng::fixed(0.0)),
    }
}

fn count(n: &Node) -> Option<u32> {
    n.max_generation
}

fn single(n: &Node) -> bool {
    n.max_generation == Some(1)
}

fn label(n: &Node) -> String {
    format!("{} \"{}\"", n.path, n.name)
}

struct Ctx<'a> {
    links: &'a Links<'a>,
    findings: Vec<Finding>,
    files: Vec<Out>,
    textures: Vec<(String, String)>,
    elements: Vec<String>,
    /// Seconds the effect lasts at least.
    end: f32,
    looping: bool,
    end_frame: f32,
    models: Vec<(String, String)>,
}

impl Ctx<'_> {
    fn find(&mut self, node: &Node, what: impl Into<String>) {
        let f = Finding {
            node: label(node),
            what: what.into(),
        };
        if !self.findings.contains(&f) {
            self.findings.push(f);
        }
    }

    /// The texture's asset id and its size in pixels, or a finding.
    fn texture(&mut self, node: &Node) -> Option<(String, Option<[f32; 2]>)> {
        let p = node.render.texture.clone()?;
        let Some(bytes) = (self.links.read)(&p) else {
            self.find(node, format!("texture not found: {p}"));
            return None;
        };
        let id = (self.links.asset)(&p);
        if !self.textures.iter().any(|t| t.1 == id) {
            self.textures.push((p, id.clone()));
        }
        Some((id, png_size(&bytes)))
    }

    fn write(&mut self, leaf: &str, kind: OutKind, bytes: Vec<u8>) -> String {
        let id = (self.links.own)(leaf);
        self.files.push(Out {
            id: id.clone(),
            kind,
            bytes,
        });
        id
    }
}

fn png_size(b: &[u8]) -> Option<[f32; 2]> {
    if b.len() >= 24 && b.starts_with(b"\x89PNG") {
        let w = u32::from_be_bytes(b[16..20].try_into().ok()?);
        let h = u32::from_be_bytes(b[20..24].try_into().ok()?);
        return Some([w as f32, h as f32]);
    }
    None
}

/// Converts a read project.
pub fn convert(project: &Project, links: &Links<'_>) -> Conversion {
    let mut cx = Ctx {
        links,
        findings: Vec::new(),
        files: Vec::new(),
        textures: Vec::new(),
        elements: Vec::new(),
        end: 0.0,
        looping: false,
        end_frame: project.end_frame,
        models: Vec::new(),
    };
    let mut chain = Vec::new();
    for n in &project.nodes {
        walk(&mut cx, n, &mut chain);
    }
    if cx.elements.len() > MAX_ELEMENTS {
        let over = cx.elements.len() - MAX_ELEMENTS;
        cx.findings.push(Finding {
            node: String::new(),
            what: format!("more than {MAX_ELEMENTS} elements: the last {over} left out"),
        });
        cx.elements.truncate(MAX_ELEMENTS);
    }
    if cx.elements.is_empty() {
        cx.findings.push(Finding {
            node: String::new(),
            what: "nothing drawn: the effect converts to no elements".into(),
        });
    }
    let duration = if cx.looping {
        (cx.end_frame / FPS).max(cx.end.min(10.0))
    } else {
        cx.end
    }
    .max(1.0 / FPS);
    let mut vfx = String::from("type vfx\nversion 1\n---\n");
    let _ = writeln!(vfx, "# Converted from Effekseer by og vfx import.");
    let _ = writeln!(vfx, "duration {}", n(duration));
    if cx.looping {
        vfx.push_str("loop true\n");
    }
    for e in &cx.elements {
        vfx.push_str(e);
    }
    Conversion {
        vfx,
        files: cx.files,
        textures: cx.textures,
        findings: cx.findings,
    }
}

fn walk<'a>(cx: &mut Ctx<'_>, node: &'a Node, chain: &mut Vec<&'a Node>) {
    chain.push(node);
    for u in &node.unread {
        cx.find(node, u.clone());
    }
    if node.rendered {
        match &node.draw {
            Draw::None => {}
            Draw::Sprite { .. } | Draw::Model { .. } | Draw::Ring(_) => mesh_or_emitter(cx, chain),
            Draw::Ribbon { .. } | Draw::Track { .. } => trail(cx, chain),
            Draw::Other(t) => cx.find(node, format!("renderer type {t}")),
        }
    }
    for c in &node.children {
        walk(cx, c, chain);
    }
    chain.pop();
}

/// Where a drawn node's element sits: its single-instance ancestors folded
/// into a start time, a transform and a scale, and its generator.
#[derive(Clone, Copy)]
struct Place {
    /// Frames from the effect's start to the generator's first instance.
    delay: f32,
    offset: Vec3,
    rot: Quat,
    scale: f32,
    /// The deepest node of the chain with many instances (or the node).
    generator: usize,
    /// Instances above the generator per one of it (nested generation).
    nested: f32,
    /// Frames after which no ancestor is left to spawn it (absolute).
    deadline: Option<f32>,
}

fn place(cx: &mut Ctx<'_>, chain: &[&Node]) -> Place {
    let me = chain.len() - 1;
    let generator = (0..chain.len())
        .rev()
        .find(|&i| !single(chain[i]))
        .unwrap_or(me);
    let mut p = Place {
        delay: 0.0,
        offset: Vec3::ZERO,
        rot: Quat::IDENTITY,
        scale: 1.0,
        generator,
        nested: 1.0,
        deadline: None,
    };
    for (i, a) in chain.iter().enumerate() {
        p.delay += a.generation_offset.c().max(0.0);
        if i < generator {
            if !single(a) {
                p.nested *= count(a).map_or(8.0, |c| c as f32);
            }
            let life = a.life.c();
            if a.remove_when_life_ends {
                let end = p.delay + life;
                p.deadline = Some(p.deadline.map_or(end, |d: f32| d.min(end)));
            }
            // This ancestor's start transform, as its child binds it.
            let child = chain[i + 1];
            let loc = at3(&a.location, 0.0, life) + spawn_offset(&a.spawn);
            if a.location
                .iter()
                .any(|l| !l.is_fixed() && l.rate(life).c().abs() > 1e-4)
            {
                cx.find(
                    chain[me],
                    "a moving ancestor: its descendants are placed where it starts",
                );
            }
            if child.bind[0] != 0 && child.bind[0] != 3 {
                p.offset += p.rot * (loc * p.scale);
            }
            if child.bind[1] != 0 && child.bind[1] != 3 {
                p.rot *= start_rotation(&a.rotation);
            }
            if child.bind[2] != 0 && child.bind[2] != 3 {
                let s = at3(&a.scale, 0.0, life);
                p.scale *= ((s.x.abs() + s.y.abs() + s.z.abs()) / 3.0).max(0.0);
            }
        }
    }
    if p.nested > 1.0 {
        cx.find(
            chain[me],
            format!(
                "nested generation: {} instances above its generator folded into one emitter",
                n(p.nested)
            ),
        );
    }
    p
}

fn spawn_offset(s: &Spawn) -> Vec3 {
    match s {
        Spawn::Point(p) => Vec3::new(p[0].c(), p[1].c(), p[2].c()),
        _ => Vec3::ZERO,
    }
}

/// The element's common lines.
fn element_head(out: &mut String, at: f32, place: &Place) {
    let _ = writeln!(out, "  at {}", n(at));
    if place.offset.length() > 1e-4 {
        let o = place.offset;
        let _ = writeln!(out, "  offset {}", ns(&[o.x, o.y, o.z]));
    }
    let r = euler_xyz(place.rot);
    if r.length() > 1e-3 {
        let _ = writeln!(out, "  rotate {}", ns(&[r.x, r.y, r.z]));
    }
}

/// The node's colour over its life (linear rgb, alpha with its fades), as keys.
fn color_keys(node: &Node, life: f32) -> Vec<(f32, [f32; 4])> {
    let l = life.max(1.0);
    let fin = node.render.fade_in;
    let fout = node.render.fade_out.filter(|_| !node.render.fade_out_after);
    let f = |t: f32| {
        let fr = t * l;
        let c = node.color.at(fr, l);
        let mut a = c[3];
        if let Some((frames, ease)) = fin
            && frames > 0.0
            && fr < frames
        {
            a *= ease.eval(fr / frames).clamp(0.0, 1.0);
        }
        if let Some((frames, ease)) = fout
            && frames > 0.0
            && fr > l - frames
        {
            a *= (1.0 - ease.eval((fr - (l - frames)) / frames)).clamp(0.0, 1.0);
        }
        [linear(c[0]), linear(c[1]), linear(c[2]), a.clamp(0.0, 1.0)]
    };
    let mut keep = Vec::new();
    if let Some((fr, _)) = fin {
        keep.push(fr / l);
    }
    if let Some((fr, _)) = fout {
        keep.push(1.0 - fr / l);
    }
    curve::keys(f, &keep, og_render::MAX_CURVE_KEYS, 0.01)
}

/// A uniform scale over the node's life: (its largest value, keys of it over that).
fn scale_keys(node: &Node, life: f32) -> (f32, Vec<(f32, [f32; 1])>, bool) {
    let l = life.max(1.0);
    let s = |t: f32| {
        let v = at3(&node.scale, t * l, l);
        [v.x.abs(), v.y.abs(), v.z.abs()]
    };
    let mut peak: f32 = 0.0;
    let mut square = true;
    for i in 0..=16 {
        let v = s(i as f32 / 16.0);
        peak = peak.max(v[0]).max(v[1]);
        if (v[0] - v[1]).abs() > 0.25 * v[0].max(v[1]).max(1e-4) {
            square = false;
        }
    }
    let peak = peak.max(1e-4);
    let keys = curve::keys(
        |t| {
            let v = s(t);
            [((v[0] * v[1]).sqrt() / peak).max(0.0)]
        },
        &[],
        og_render::MAX_CURVE_KEYS,
        0.01,
    );
    (peak, keys, square)
}

fn curve_line(keys: &[(f32, [f32; 1])]) -> String {
    keys.iter()
        .map(|(t, v)| format!("{} {}", n(*t), n(v[0])))
        .collect::<Vec<_>>()
        .join(" ")
}

fn blend(cx: &mut Ctx<'_>, node: &Node) -> &'static str {
    match node.render.blend {
        2 => "additive",
        0 | 1 => "alpha",
        3 => {
            cx.find(node, "subtractive blend: drawn alpha blended");
            "alpha"
        }
        4 => {
            cx.find(node, "multiplicative blend: drawn alpha blended");
            "alpha"
        }
        b => {
            cx.find(node, format!("blend {b}: drawn alpha blended"));
            "alpha"
        }
    }
}

/// The main-texture UV lines of an effect material for the node's UV
/// (a region, a scroll), and whether a flipbook took it instead.
fn uv_block(cx: &mut Ctx<'_>, node: &Node, size: Option<[f32; 2]>) -> Option<String> {
    let region = |start: [f32; 2], sz: [f32; 2], scroll: [f32; 2]| -> Option<String> {
        let [w, h] = size?;
        let tile = [sz[0] / w, sz[1] / h];
        let off = [start[0] / w, start[1] / h];
        let scroll = [scroll[0] * FPS / w, scroll[1] * FPS / h];
        let mut s = String::new();
        if (tile[0] - 1.0).abs() > 1e-3 || (tile[1] - 1.0).abs() > 1e-3 {
            let _ = writeln!(s, "  tile {}", ns(&tile));
        }
        if off[0].abs() > 1e-4 || off[1].abs() > 1e-4 {
            let _ = writeln!(s, "  offset {}", ns(&off));
        }
        if scroll[0].abs() > 1e-5 || scroll[1].abs() > 1e-5 {
            let _ = writeln!(s, "  scroll {}", ns(&scroll));
        }
        Some(s)
    };
    match &node.render.uv {
        Uv::Default | Uv::Animation { .. } => None,
        Uv::Fixed { start, size: sz } => {
            let s = region(*start, *sz, [0.0; 2]);
            if s.is_none() {
                cx.find(
                    node,
                    "a UV region of a texture whose size is not known (not a PNG)",
                );
            }
            s.filter(|s| !s.is_empty())
        }
        Uv::Scroll {
            start,
            size: sz,
            speed,
        } => {
            if speed.iter().any(|r| r.h().abs() > 1e-4) {
                cx.find(node, "random UV scroll speed: its mean used");
            }
            let s = region(
                [start[0].c(), start[1].c()],
                [sz[0].c(), sz[1].c()],
                [speed[0].c(), speed[1].c()],
            );
            if s.is_none() {
                cx.find(
                    node,
                    "a UV region of a texture whose size is not known (not a PNG)",
                );
            }
            s.filter(|s| !s.is_empty())
        }
        Uv::Curve => None,
    }
}

/// A flipbook `sheet` line for an animated UV that covers the texture's grid.
fn sheet_line(cx: &mut Ctx<'_>, node: &Node, size: Option<[f32; 2]>, life: f32) -> Option<String> {
    let Uv::Animation {
        start,
        size: cell,
        frame_length,
        count,
        loop_type,
    } = &node.render.uv
    else {
        return None;
    };
    let frames = count[0] * count[1];
    let Some([w, h]) = size else {
        cx.find(
            node,
            "UV animation of a texture whose size is not known (not a PNG)",
        );
        return None;
    };
    let cols = (w / cell[0].max(1.0)).round().max(1.0) as i32;
    let rows = (h / cell[1].max(1.0)).round().max(1.0) as i32;
    if start[0] > 0.5 || start[1] > 0.5 || cols < count[0] {
        cx.find(node, "UV animation over a part of its texture: drawn whole");
        return None;
    }
    let mode = match (frame_length, loop_type) {
        (None, _) => "off".to_string(),
        (Some(len), 0) => format!("life {}", n((life / (len * frames as f32)).min(1.0))),
        (Some(len), lt) => {
            if *lt == 2 {
                cx.find(node, "ping-pong UV animation: looped");
            }
            format!("fps {}", n(FPS / len))
        }
    };
    Some(format!("sheet {cols} {rows} {frames} {mode}\n"))
}

/// An effect material file for a mesh or an emitter, or `None` when the
/// emitter needs none.
#[allow(clippy::too_many_arguments)]
fn effect_file(
    cx: &mut Ctx<'_>,
    node: &Node,
    leaf: &str,
    texture: Option<&(String, Option<[f32; 2]>)>,
    mesh: bool,
    alpha: Option<&[(f32, [f32; 4])]>,
    blend: &str,
    polar: bool,
) -> Option<String> {
    let uv = uv_block(cx, node, texture.and_then(|t| t.1));
    if !mesh && uv.is_none() {
        return None;
    }
    let mut s = String::from("type effect\nversion 1\n---\n");
    let _ = writeln!(s, "# Effekseer node {}", label(node));
    let name = texture.map(|t| format!(" \"{}\"", t.0)).unwrap_or_default();
    let body = uv.unwrap_or_default();
    let polar_line = if polar { "  polar true\n" } else { "" };
    if !name.is_empty() || !body.is_empty() || !polar_line.is_empty() {
        let _ = writeln!(s, "main{name} rgba {{\n{body}{polar_line}}}");
    }
    if let Some(keys) = alpha {
        let a = keys.iter().map(|k| k.1[3]).fold(0.0, f32::max).max(1e-4);
        let line: Vec<String> = keys
            .iter()
            .map(|(t, c)| format!("{} {}", n(*t), n((c[3] / a).clamp(0.0, 1.0))))
            .collect();
        if keys.iter().any(|k| (k.1[3] - keys[0].1[3]).abs() > 1e-3) {
            let _ = writeln!(s, "alpha curve {}", line.join(" "));
        }
    }
    let _ = writeln!(s, "blend {blend}");
    if mesh {
        s.push_str("doubleSided true\n");
    }
    Some(cx.write(leaf, OutKind::Effect, s.into_bytes()))
}

fn leaf(node: &Node, what: &str) -> String {
    format!("n{}.{what}", node.path.replace('.', "-"))
}

/// The emitted count and spawn times of a generator, its instances' spans.
struct Schedule {
    /// `None`: generates as long as it lives.
    count: Option<u32>,
    /// Frames between instances.
    interval: f32,
}

fn schedule(g: &Node) -> Schedule {
    Schedule {
        count: count(g),
        interval: g.generation_time.c().max(0.0),
    }
}

/// A drawn sprite, ring or model node: a mesh element (one instance, or a
/// few), else an emitter.
fn mesh_or_emitter(cx: &mut Ctx<'_>, chain: &[&Node]) {
    let node = chain[chain.len() - 1];
    let p = place(cx, chain);
    let sch = schedule(chain[p.generator]);
    let few = sch.count.is_some_and(|c| c <= MAX_COPIES) && p.nested <= 1.0;
    let mesh_like = match &node.draw {
        Draw::Ring(_) => true,
        Draw::Model { model, .. } => {
            if model.is_none() {
                return; // Effekseer draws nothing for a model node without one.
            }
            true
        }
        Draw::Sprite { billboard } => *billboard == 2 && few,
        _ => false,
    };
    if mesh_like {
        if !few {
            cx.find(
                node,
                format!(
                    "{} {} instances: the first {MAX_COPIES} drawn",
                    node.draw.name(),
                    sch.count.map_or("unlimited".into(), |c| c.to_string())
                ),
            );
        }
        let copies = sch.count.unwrap_or(MAX_COPIES).clamp(1, MAX_COPIES);
        mesh(cx, chain, &p, &sch, copies);
    } else {
        emitter(cx, chain, &p, &sch);
    }
}

fn mesh(cx: &mut Ctx<'_>, chain: &[&Node], p: &Place, sch: &Schedule, copies: u32) {
    let node = chain[chain.len() - 1];
    let life = node.life.c().max(1.0);
    let (peak, mut scale, square) = scale_keys(node, life);
    let mut colors = color_keys(node, life);
    let blend = blend(cx, node);
    let texture = cx.texture(node);
    let mut base = peak * p.scale;
    // A ring's own shape: its lines in the element.
    let mut ring = String::new();
    let shape = match &node.draw {
        Draw::Ring(r) => {
            if r.billboard != 2 {
                cx.find(node, "a camera-facing ring: drawn in its own plane");
            }
            let outer = |t: f32| r.outer.at(t * life, life).abs();
            let o_peak = (0..=16)
                .map(|i| outer(i as f32 / 16.0))
                .fold(0.0, f32::max)
                .max(1e-4);
            let mid = 0.5;
            let inner = (r.inner.at(mid * life, life) / outer(mid).max(1e-4)).clamp(0.0, 16.0);
            // Its edges' heights along its axis, in its outer radius.
            let height = |t: f32| {
                let o = outer(t).max(1e-4);
                [
                    r.outer_height.at(t * life, life) / o,
                    r.inner_height.at(t * life, life) / o,
                ]
            };
            let h = height(mid);
            let inner_moves = (0..=8).any(|i| {
                let t = i as f32 / 8.0;
                ((r.inner.at(t * life, life) / outer(t).max(1e-4)) - inner).abs() > 0.15
            });
            let height_moves = (0..=8).any(|i| {
                let hh = height(i as f32 / 8.0);
                (hh[0] - h[0]).abs() > 0.15 || (hh[1] - h[1]).abs() > 0.15
            });
            if height_moves {
                cx.find(
                    node,
                    "a ring whose height changes over its life: its middle height kept",
                );
            }
            if inner_moves {
                cx.find(
                    node,
                    "a ring whose thickness changes over its life: its middle thickness kept",
                );
            }
            let fr = colors.len() / 2;
            let at = colors.get(fr).map_or(0.0, |k| k.0) * life;
            let lin = |c: &Color| {
                let c = c.at(at, life);
                [linear(c[0]), linear(c[1]), linear(c[2]), c[3]]
            };
            // Effekseer's arc is centred on +y.
            let start = (360.0 - r.viewing_angle) * 0.5 + 90.0;
            let cs = [
                lin(&r.outer_color),
                lin(&r.center_color),
                lin(&r.inner_color),
            ];
            let _ = writeln!(ring, "  inner {}", n(inner));
            let _ = writeln!(ring, "  band {}", n(r.center_ratio));
            if h[0].abs() > 1e-4 || h[1].abs() > 1e-4 {
                let _ = writeln!(ring, "  height {}", ns(&h));
            }
            if r.viewing_angle < 360.0 {
                let _ = writeln!(ring, "  arc {}", ns(&[start, r.viewing_angle]));
            }
            let _ = writeln!(ring, "  colors {}", ns(&cs.concat()));
            // The ring's radius over life times the node's scale.
            let keys = curve::keys(
                |t| {
                    let s = at3(&node.scale, t * life, life);
                    [outer(t) * (s.x.abs() * s.y.abs()).sqrt()]
                },
                &[],
                og_render::MAX_CURVE_KEYS,
                0.01 * o_peak,
            );
            let pk = keys.iter().map(|k| k.1[0]).fold(0.0, f32::max).max(1e-4);
            scale = keys.iter().map(|(t, v)| (*t, [v[0] / pk])).collect();
            base = pk * p.scale;
            "ring".into()
        }
        Draw::Model { model: Some(m), .. } => {
            if !square {
                cx.find(node, "a model scaled unevenly: drawn at its mean scale");
            }
            match model_id(cx, node, m) {
                Some(id) => format!("\"{id}\""),
                None => "sphere".into(),
            }
        }
        _ => {
            if !square {
                cx.find(node, "a sprite scaled unevenly: drawn square at its mean");
            }
            "quad".into()
        }
    };
    // A node not removed at the end of its life stays, as its life left
    // it, until the effect ends: its curves end at its life's share.
    let shown = if node.remove_when_life_ends {
        life
    } else {
        life.max(cx.end_frame - p.delay)
    };
    let share = life / shown;
    for k in &mut colors {
        k.0 *= share;
    }
    for k in &mut scale {
        k.0 *= share;
    }
    let life = shown;
    let effect = effect_file(
        cx,
        node,
        &leaf(node, "effect.ogt"),
        texture.as_ref(),
        true,
        Some(&colors),
        blend,
        false,
    )
    .expect("a mesh's effect");
    let (_, spin) = spin_z(&node.rotation, life * share);
    let c0 = colors.iter().fold([0.0f32; 4], |m, k| {
        // The brightest key: the colour the alpha curve scales.
        if k.1[3] > m[3] { k.1 } else { m }
    });
    let rgb_moves = colors
        .iter()
        .any(|k| (0..3).any(|i| (k.1[i] - c0[i]).abs() > 0.15));
    if rgb_moves {
        cx.find(
            node,
            "a mesh's colour over its life: its brightest kept, alpha over life kept",
        );
    }
    let mut col = [c0[0], c0[1], c0[2], c0[3].clamp(1e-3, 1.0)];
    // Copies spawned within a frame of each other differ only by their
    // random start: one each of its rotation's range, else one.
    let rot_range = match &node.rotation {
        Rotation::Euler(a) => Some([a[0].start(), a[1].start(), a[2].start()]),
        _ => None,
    };
    let varied = rot_range.is_some_and(|r| r.iter().any(|r| r.h() > 1.0));
    let together = sch.interval * (copies.saturating_sub(1) as f32) < 1.0;
    let copies = if !together || varied { copies } else { 1 };
    // Instances drawn together over one another: the copies drawn carry
    // the light of those left out (Effekseer stacks faint rings by the
    // hundred).
    let stacked = sch.count.unwrap_or(copies) as f32 / copies as f32;
    if together && stacked > 1.0 {
        if blend == "additive" {
            for c in &mut col[..3] {
                *c *= stacked;
            }
        } else {
            col[3] = 1.0 - og_core::math::pow(1.0 - col[3].min(0.999), stacked);
        }
    }
    let own_life = life * share;
    if node
        .location
        .iter()
        .any(|l| l.rate(own_life).c().abs() > 1e-3)
    {
        cx.find(node, "a moving mesh: drawn where it starts");
    }
    let start = at3(&node.location, 0.0, own_life) + spawn_offset(&node.spawn);
    for k in 0..copies {
        let at = (p.delay + k as f32 * sch.interval) / FPS;
        // Its own start: where it is and how it is turned, in its parents' place.
        let u = (k as f32 + 0.5) / copies as f32;
        let own = match (&node.rotation, rot_range) {
            (Rotation::Euler(_), Some(r)) => efk_quat(Vec3::new(
                r[0].min + (r[0].max - r[0].min) * u,
                r[1].min + (r[1].max - r[1].min) * u,
                r[2].min + (r[2].max - r[2].min) * u,
            )),
            (r, _) => start_rotation(r),
        };
        let me = Place {
            offset: p.offset + p.rot * (start * p.scale),
            rot: p.rot * own,
            ..*p
        };
        let mut s = format!("mesh {shape} {{\n");
        element_head(&mut s, at, &me);
        s.push_str(&ring);
        let _ = writeln!(s, "  life {}", n(life / FPS));
        let _ = writeln!(s, "  effect \"{effect}\"");
        if scale.len() > 1 && !curve::flat(&scale, 1e-3) {
            let _ = writeln!(
                s,
                "  scale curve {}",
                scale
                    .iter()
                    .map(|(t, v)| format!("{} {}", n(*t), n(v[0] * base)))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        } else {
            let _ = writeln!(s, "  scale {}", n(base * scale[0].1[0]));
        }
        if spin.c().abs() > 1e-3 {
            let _ = writeln!(s, "  spin 0 0 {}", n(spin.c()));
        }
        let _ = writeln!(s, "  color {}", ns(&col));
        let _ = writeln!(s, "  blend {blend}");
        s.push_str("}\n");
        cx.elements.push(s);
        cx.end = cx.end.max(at + life / FPS);
    }
}

/// A model the project names, converted once (an `.efkmodel`), or a finding.
fn model_id(cx: &mut Ctx<'_>, node: &Node, path: &str) -> Option<String> {
    if let Some((_, id)) = cx.models.iter().find(|m| m.0 == path) {
        return Some(id.clone());
    }
    if !path.to_ascii_lowercase().ends_with(".efkmodel") {
        cx.find(
            node,
            format!("a model that is not an .efkmodel ({path}): drawn as a sphere"),
        );
        return None;
    }
    let Some(bytes) = (cx.links.read)(path) else {
        cx.find(node, format!("model not found: {path} (drawn as a sphere)"));
        return None;
    };
    match glb::efkmodel(&bytes) {
        Ok(g) => {
            let stem = path.rsplit('/').next().unwrap_or(path);
            let stem = stem.rsplit_once('.').map_or(stem, |s| s.0);
            let id = cx.write(&format!("model-{stem}.glb"), OutKind::Model, g);
            cx.models.push((path.to_string(), id.clone()));
            Some(id)
        }
        Err(e) => {
            cx.find(
                node,
                format!("a model that does not read ({e}): drawn as a sphere"),
            );
            None
        }
    }
}

/// The generator's motion relative to where it spawns, plus the drawn
/// node's own (when it is not the generator and follows it): per frame.
struct Motion {
    vel: [Rng; 3],
    accel: Vec3,
    /// Its speed over its life against its mean (1 a steady speed).
    speed_keys: Vec<(f32, [f32; 1])>,
    /// Where it starts (the centre, and the box of its random start).
    start: Vec3,
    start_half: Vec3,
}

fn motion(cx: &mut Ctx<'_>, chain: &[&Node], g: usize, life: f32) -> Motion {
    let me = chain.len() - 1;
    let movers: Vec<&Node> = (g..=me)
        .filter(|&i| i == g || chain[i].bind[0] == 2)
        .map(|i| chain[i])
        .collect();
    let mut vel = [Rng::fixed(0.0); 3];
    let mut accel = Vec3::ZERO;
    let mut start = Vec3::ZERO;
    let mut half = Vec3::ZERO;
    for m in &movers {
        let ml = m.life.c().max(1.0);
        for k in 0..3 {
            let a = &m.location[k];
            let r = match a {
                Anim::Pva { v, .. } => *v,
                _ => a.rate(ml),
            };
            vel[k] = Rng {
                min: vel[k].min + r.min,
                max: vel[k].max + r.max,
            };
            let s = a.start();
            start[k] += s.c();
            half[k] += s.h().abs();
        }
        accel += Vec3::new(
            m.location[0].accel().c(),
            m.location[1].accel().c(),
            m.location[2].accel().c(),
        ) + m.gravity;
        if m.location_type == 3 {
            cx.find(chain[me], "location over an f-curve: straightened");
        }
    }
    // The speed's shape over the drawn node's life (an easing that slows).
    let l = life.max(1.0);
    let pos = |t: f32| -> Vec3 {
        movers
            .iter()
            .map(|m| {
                let mut p = Vec3::ZERO;
                for k in 0..3 {
                    p[k] = match &m.location[k] {
                        Anim::Pva { p: p0, v, .. } => p0.c() + v.c() * t * l,
                        a => a.at(t * l, m.life.c().max(1.0)),
                    };
                }
                p
            })
            .sum()
    };
    let total = (pos(1.0) - pos(0.0)).length();
    let speed_keys = if total > 1e-4 {
        let dt = 1.0 / 64.0;
        curve::keys(
            |t| {
                let a = (t - dt * 0.5).max(0.0);
                let b = (t + dt * 0.5).min(1.0);
                [((pos(b) - pos(a)).length() / (b - a).max(1e-6)) / total]
            },
            &[],
            og_render::MAX_CURVE_KEYS,
            0.05,
        )
    } else {
        vec![(0.0, [1.0]), (1.0, [1.0])]
    };
    Motion {
        vel,
        accel,
        speed_keys,
        start,
        start_half: half,
    }
}

/// Speed (units per second), spread (degrees) and the direction of a
/// velocity box (per frame).
fn direction(v: &[Rng; 3]) -> (Vec3, [f32; 2], f32) {
    let mean = Vec3::new(v[0].c(), v[1].c(), v[2].c()) * FPS;
    let half = Vec3::new(v[0].h(), v[1].h(), v[2].h()) * FPS;
    let corners = (0..8).map(|i| {
        mean + Vec3::new(
            if i & 1 == 0 { -half.x } else { half.x },
            if i & 2 == 0 { -half.y } else { half.y },
            if i & 4 == 0 { -half.z } else { half.z },
        )
    });
    let m = mean.length();
    if m < 1e-3 {
        let h = half.length();
        return (Vec3::Y, [h * 0.25, h], if h > 1e-3 { 180.0 } else { 0.0 });
    }
    let dir = mean / m;
    let (mut lo, mut hi, mut spread) = (f32::MAX, 0.0f32, 0.0f32);
    for c in corners {
        let l = c.length();
        lo = lo.min(l);
        hi = hi.max(l);
        if l > 1e-6 {
            let cos = (c.dot(dir) / l).clamp(-1.0, 1.0);
            spread = spread.max(og_core::math::acos(cos).to_degrees());
        }
    }
    (dir, [lo.min(hi), hi], spread.min(180.0))
}

fn emitter(cx: &mut Ctx<'_>, chain: &[&Node], p: &Place, sch: &Schedule) {
    let node = chain[chain.len() - 1];
    let g = chain[p.generator];
    let life = node.life;
    let lc = life.c().max(1.0);
    let s = p.scale;
    if let Draw::Sprite { billboard } = node.draw
        && billboard != 0
        && billboard != 3
    {
        cx.find(
            node,
            match billboard {
                1 => "a y-axis billboard: drawn facing the camera",
                2 => "a fixed sprite with many instances: drawn facing the camera",
                _ => "a directional billboard: drawn facing the camera",
            },
        );
    }
    if node.color.random_spread() > 0.1 {
        cx.find(node, "a random colour per particle: its mean used");
    }
    let mo = motion(cx, chain, p.generator, lc);
    let (dir, speed, mut spread) = direction(&mo.vel);
    let (peak, size_keys, square) = scale_keys(node, lc);
    if !square {
        cx.find(node, "a sprite scaled unevenly: drawn square at its mean");
    }
    let start_scale = node.scale[0].start();
    let rel = if start_scale.c().abs() > 1e-4 {
        Rng {
            min: start_scale.min / start_scale.c(),
            max: start_scale.max / start_scale.c(),
        }
    } else {
        Rng::fixed(1.0)
    };
    let colors = color_keys(node, lc);
    let blend = blend(cx, node);
    let texture = cx.texture(node);

    let mut e = String::from("type particles\nversion 1\n---\n");
    let _ = writeln!(e, "# Effekseer node {}", label(node));
    // How many and when.
    let peak_alive = match sch.count {
        None => lc / sch.interval.max(0.1) + 1.0,
        Some(c) => (c as f32).min(lc / sch.interval.max(1e-3) + 1.0),
    } * p.nested;
    let max = (peak_alive * 1.25).ceil().clamp(4.0, 4096.0) as u32;
    let _ = writeln!(e, "maxParticles {max}");
    // Where: the generator's spawn shape, rotated so +y is the direction.
    let mut rot = Quat::from_rotation_arc(Vec3::Y, dir);
    let mut half = mo.start_half;
    let mut center = mo.start;
    match &g.spawn {
        Spawn::Point(pt) => {
            half += Vec3::new(pt[0].h(), pt[1].h(), pt[2].h()).abs();
        }
        Spawn::Sphere { radius, .. } => {
            if g.spawn_rotates {
                // Moving out from the centre: the sphere's own direction.
                rot = Quat::IDENTITY;
                spread = 0.0;
            }
            let _ = writeln!(e, "shape sphere {}", n(radius.max * s));
        }
        Spawn::Circle {
            radius,
            axis,
            start,
            end,
            ..
        } => {
            let r = radius.c() * s;
            let (a0, a1) = (start.c(), end.c());
            let pts: Vec<String> = (0..8)
                .map(|i| {
                    let a = (a0 + (a1 - a0) * i as f32 / 8.0).to_radians();
                    let (sn, cs) = og_core::math::sin_cos(a);
                    let v = match axis {
                        0 => Vec3::new(0.0, sn * r, cs * r),
                        1 => Vec3::new(cs * r, 0.0, -sn * r),
                        _ => Vec3::new(-sn * r, cs * r, 0.0),
                    };
                    // The spline is in the emitter's space, which `rot` turns.
                    let v = rot.inverse() * v;
                    ns(&[v.x, v.y, v.z])
                })
                .collect();
            let closed = (a1 - a0).abs() >= 359.0;
            let _ = writeln!(
                e,
                "shape spline {}{}",
                pts.join("  "),
                if closed { " closed" } else { "" }
            );
            if g.spawn_rotates {
                cx.find(
                    node,
                    "motion out from a circle: approximated along one direction",
                );
            }
        }
        Spawn::Line { from, to } => {
            let a = Vec3::new(from[0].c(), from[1].c(), from[2].c());
            let b = Vec3::new(to[0].c(), to[1].c(), to[2].c());
            center += (a + b) * 0.5;
            half += ((b - a) * 0.5).abs();
        }
        Spawn::Model => {}
    }
    if !matches!(g.spawn, Spawn::Sphere { .. } | Spawn::Circle { .. }) {
        if half.length() > 1e-4 {
            let h = (rot.inverse() * half).abs() * s;
            let _ = writeln!(
                e,
                "shape box {}",
                ns(&[h.x.max(1e-3), h.y.max(1e-3), h.z.max(1e-3)])
            );
        } else {
            e.push_str("shape point\n");
        }
    }
    let _ = writeln!(e, "spread {}", n(spread));
    match sch.count {
        None => {
            let _ = writeln!(e, "rate {}", n(FPS / sch.interval.max(0.1) * p.nested));
            e.push_str("duration 1\nlooping true\n");
            cx.looping = true;
        }
        Some(c) => {
            let per = p.nested.round().max(1.0) as u32;
            let span = c.saturating_sub(1) as f32 * sch.interval;
            if span < 1.0 {
                // Within a frame: at once.
                let _ = writeln!(e, "rate 0\nburst 0 {}", c * per);
                let _ = writeln!(e, "duration {}", n(1.0 / FPS));
            } else if sch.interval < 1.0 {
                // Several a frame: a stream over the span.
                let _ = writeln!(e, "rate {}", n(FPS / sch.interval * per as f32));
                let _ = writeln!(e, "duration {}", n(c as f32 * sch.interval / FPS));
            } else {
                let _ = writeln!(e, "rate 0\nburst 0 {per} {c} {}", n(sch.interval / FPS));
                let _ = writeln!(e, "duration {}", n((span + 1.0) / FPS));
            }
            e.push_str("looping false\n");
        }
    }
    let _ = writeln!(e, "lifetime {}", ns(&[life.min / FPS, life.max / FPS]));
    let speed = [speed[0] * s, speed[1] * s];
    let _ = writeln!(e, "speed {}", ns(&speed));
    if mo.speed_keys.len() > 1 && !curve::flat(&mo.speed_keys, 0.05) {
        let _ = writeln!(e, "speedOverLife {}", curve_line(&mo.speed_keys));
    }
    let size = peak * s;
    let _ = writeln!(e, "size {}", ns(&[size * rel.min, size * rel.max]));
    if !curve::flat(&size_keys, 1e-3) {
        let _ = writeln!(e, "sizeOverLife {}", curve_line(&size_keys));
    }
    e.push_str("colorOverLife {\n");
    for (t, c) in &colors {
        let _ = writeln!(e, "  key {} {}", n(*t), ns(c));
    }
    e.push_str("}\n");
    let (roll, spin) = spin_z(&node.rotation, lc);
    if roll.max.abs() > 1e-3 || roll.min.abs() > 1e-3 {
        let _ = writeln!(e, "rotation {}", ns(&[roll.min, roll.max]));
    }
    if spin.max.abs() > 1e-3 || spin.min.abs() > 1e-3 {
        let _ = writeln!(e, "angularVelocity {}", ns(&[spin.min, spin.max]));
    }
    let gravity = mo.accel * (FPS * FPS * s);
    if gravity.length() > 1e-3 {
        // In the simulation's (world) space: turned as the node's parents are.
        let gl = p.rot * gravity;
        let _ = writeln!(e, "gravity {}", ns(&[gl.x, gl.y, gl.z]));
    }
    if let Some((id, size)) = &texture {
        let _ = writeln!(e, "texture \"{id}\"");
        if let Some(sheet) = sheet_line(cx, node, *size, lc) {
            e.push_str(&sheet);
        }
    }
    let _ = writeln!(e, "blend {blend}");
    if let Some(d) = node.render.distortion {
        let _ = writeln!(e, "distortion {}", n((d * 0.02).clamp(0.002, 0.1)));
    }
    if let Some(fx) = effect_file(
        cx,
        node,
        &leaf(node, "effect.ogt"),
        texture.as_ref(),
        false,
        None,
        blend,
        false,
    ) {
        let _ = writeln!(e, "effect \"{fx}\"");
    }
    let id = cx.write(&leaf(node, "emitter.ogt"), OutKind::Emitter, e.into_bytes());

    // The element: the place, turned so the emitter's +y is the motion.
    let mut place = Place {
        offset: p.offset + p.rot * (center * s),
        rot: p.rot * rot,
        ..*p
    };
    let at = place.delay / FPS;
    let mut el = format!("emitter \"{id}\" {{\n");
    element_head(&mut el, at, &place);
    let span = match sch.count {
        Some(c) => c.saturating_sub(1) as f32 * sch.interval,
        None => place
            .deadline
            .map_or(cx.end_frame, |d| (d - place.delay).max(0.0)),
    };
    if let Some(d) = place.deadline.take()
        && sch.count.is_none()
    {
        let _ = writeln!(el, "  until {}", n((d / FPS).max(at)));
    }
    el.push_str("}\n");
    cx.elements.push(el);
    cx.end = cx.end.max(at + span / FPS + life.max / FPS);
}

/// A ribbon or track: a trail where its node's instances go.
fn trail(cx: &mut Ctx<'_>, chain: &[&Node]) {
    let node = chain[chain.len() - 1];
    let p = place(cx, chain);
    let g = chain[p.generator];
    let sch = schedule(g);
    let life = node.life.c().max(1.0);
    let colors = color_keys(node, life);
    let blend = blend(cx, node);
    let texture = cx.texture(node);
    let (peak, _, _) = scale_keys(node, life);
    let width = match &node.draw {
        Draw::Ribbon { width } => [width * peak, width * peak * 0.25],
        Draw::Track { width } => [width[0] * peak, width[2] * peak],
        _ => [peak, 0.0],
    };
    if matches!(node.draw, Draw::Track { .. }) && !single(node) {
        cx.find(node, "a track of many instances: one trail drawn");
    }
    let effect = effect_file(
        cx,
        node,
        &leaf(node, "effect.ogt"),
        texture.as_ref(),
        true,
        None,
        blend,
        false,
    );
    // How long it lays points: its generator's span, else its life.
    let span = match sch.count {
        Some(c) if c > 1 && !single(node) => c.saturating_sub(1) as f32 * sch.interval,
        _ => g.life.c(),
    }
    .max(1.0);
    // Its motion: the fastest turn of an ancestor (or its own) at the
    // radius its place puts it, and its rise up that turn's axis (a
    // sword's swing turns about z).
    let mut orbit = [0.0f32, 0.0];
    let mut axis = 1;
    let mut arm = Vec3::ZERO;
    for (i, a) in chain.iter().enumerate() {
        let al = a.life.c().max(1.0);
        if let Rotation::Euler(r) = &a.rotation {
            for (j, r) in r.iter().enumerate() {
                let w = r.rate(al).c() * FPS / 360.0;
                if w.abs() > orbit[1].abs().max(1e-3) {
                    let loc: Vec3 = chain[i..]
                        .iter()
                        .skip(1)
                        .map(|c| at3(&c.location, 0.0, c.life.c().max(1.0)))
                        .sum();
                    let mut flat = loc;
                    flat[j] = 0.0;
                    orbit = [flat.length() * p.scale, w];
                    axis = j;
                    arm = flat;
                }
            }
        }
    }
    let rise: f32 = chain[p.generator.min(chain.len() - 1)..]
        .iter()
        .map(|a| a.location[axis].rate(a.life.c().max(1.0)).c() * FPS * p.scale)
        .sum();
    // Centred on the turn, its y turned onto the turn's axis: y onto x (a
    // turn of -90° about z) or onto z (90° about x).
    // (Its place leaves out its own location, which the arm counts.)
    let own = at3(&node.location, 0.0, life);
    let p = Place {
        offset: p.offset + p.rot * ((own - arm) * p.scale),
        rot: match axis {
            0 => p.rot * quat_axis(Vec3::Z, -90.0),
            2 => p.rot * quat_axis(Vec3::X, 90.0),
            _ => p.rot,
        },
        ..p
    };
    if orbit[1] == 0.0
        && chain[p.generator..].iter().any(|a| {
            a.location[0].rate(a.life.c()).c().abs() + a.location[2].rate(a.life.c()).c().abs()
                > 1e-3
        })
    {
        cx.find(
            node,
            "a ribbon or track's sideways motion: only its rise and turn are kept",
        );
    }
    let alpha = colors.iter().map(|k| k.1[3]).fold(0.0, f32::max);
    let c = colors
        .iter()
        .find(|k| k.1[3] >= alpha - 1e-4)
        .map_or([1.0; 4], |k| k.1);
    let at = p.delay / FPS;
    let mut s = String::from("trail {\n");
    element_head(&mut s, at, &p);
    let _ = writeln!(s, "  life {}", n(span / FPS));
    if let Some(fx) = effect {
        let _ = writeln!(s, "  effect \"{fx}\"");
    }
    let _ = writeln!(
        s,
        "  width {}",
        ns(&[width[0] * p.scale, width[1] * p.scale])
    );
    let _ = writeln!(s, "  lifetime {}", n(life / FPS));
    let _ = writeln!(s, "  color {}", ns(&[c[0], c[1], c[2], alpha]));
    if orbit[1] != 0.0 {
        let _ = writeln!(s, "  orbit {}", ns(&orbit));
    }
    if rise.abs() > 1e-3 {
        let _ = writeln!(s, "  rise {}", n(rise));
    }
    s.push_str("}\n");
    cx.elements.push(s);
    cx.end = cx.end.max(at + (span + life) / FPS);
}
