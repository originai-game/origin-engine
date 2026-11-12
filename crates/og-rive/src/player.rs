//! Plays an [`Artboard`]: its animations advanced by a fixed step on the
//! tick, its nodes posed from their keys, its shapes drawn as UI shapes in
//! a box (placed by the artboard's fit and align).

use super::*;
use og_css::Srgba;
use og_draw::{
    UiBlend, UiGeometry, UiGradient, UiGradientKind, UiJoin, UiMode, UiPaint, UiPath, UiShape,
    UiSpread, UiStop,
};
use og_ui_model::Affine;
use og_ui_model::codec::{R, W};
use std::sync::Arc;

/// What a playing artboard draws, bottom to top, in screen space.
#[derive(Clone, Debug, PartialEq)]
pub enum Drawn {
    Shape(UiShape),
    /// A piece of a text in one font and paint.
    Text(TextDraw),
    /// Cuts what follows, until its [`Drawn::Unclip`], to this outline.
    Clip(UiPath),
    Unclip,
}

/// A run's piece of a text line in one of its style's paints, for og-ui's
/// text command (`DrawCmd::Text`, styled when it has a gradient, an outline
/// or a map).
#[derive(Clone, Debug, PartialEq)]
pub struct TextDraw {
    pub font: og_draw::FontHandle,
    pub text: String,
    /// One per character of `text` (empty: the font's advances and kerning).
    pub placements: Vec<og_draw::GlyphPlacement>,
    /// The pen at the line's baseline, CSS px.
    pub pen: Vec2,
    pub size: f32,
    /// Em after each glyph.
    pub tracking: f32,
    /// The glyphs' map about the pen (`[1, 0, 0, 1]`: none).
    pub transform: [f32; 4],
    /// Linear RGBA (clear for a stroke, whose outline draws).
    pub color: [f32; 4],
    /// A gradient fill and its box (`[x, y, width, height]`).
    pub gradient: Option<(UiGradient, [f32; 4])>,
    /// A stroke: its width, CSS px before `transform`, and colour.
    pub outline: Option<(f32, [f32; 4])>,
    /// The piece's line box on screen, `[x, y, width, height]`.
    pub bounds: [f32; 4],
}

/// A drawn object of a playing artboard, for dump_ui, describe and pick.
#[derive(Clone, Debug, PartialEq)]
pub struct RiveElement {
    /// `<artboard>/<object name>` (nested: `<outer>/<nested>/<object>`).
    pub id: String,
    /// Its node in its artboard.
    pub node: u32,
    /// Its bounds on screen, `[x, y, width, height]` CSS px.
    pub bounds: [f32; 4],
    /// Its opacity as drawn.
    pub opacity: f32,
    /// A text's words as drawn (lines apart by line feeds); else empty.
    pub text: String,
}

/// One artboard playing. Clone it to fork, compare it to check two runs.
#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub artboard: Arc<Artboard>,
    /// The animations playing (an index into [`Artboard::animations`]) and
    /// each one's time, seconds, and direction (ping-pong).
    pub playing: Vec<(u32, f32, f32)>,
    /// The players of the nested artboards, by node.
    pub nested: Vec<(u32, Player)>,
}

impl Player {
    /// Plays `animation` (by name; `None`: the artboard's default).
    pub fn new(artboard: Arc<Artboard>, animation: Option<&str>) -> Player {
        let pick = match animation {
            Some(n) => artboard.animations.iter().position(|a| a.name == n),
            None => artboard.play.map(|p| p as usize),
        };
        let mut nested = Vec::new();
        for (i, node) in artboard.nodes.iter().enumerate() {
            if let Kind::Nested {
                artboard: k,
                animations,
            } = &node.kind
            {
                let inner = Arc::new(artboard.nested[*k as usize].clone());
                let mut p = Player::new(inner, None);
                // A nested artboard plays what its node says, not its default.
                p.playing = animations
                    .iter()
                    .filter(|a| a.playing)
                    .map(|a| (a.animation, 0.0, a.speed))
                    .collect();
                nested.push((i as u32, p));
            }
        }
        Player {
            playing: pick
                .map(|i| (i as u32, artboard.animations[i].span().0, 1.0))
                .into_iter()
                .collect(),
            artboard,
            nested,
        }
    }

    /// Advances every animation by `dt` seconds (the UI tick's step).
    pub fn advance(&mut self, dt: f32) {
        for (i, t, dir) in &mut self.playing {
            let a = &self.artboard.animations[*i as usize];
            let (lo, hi) = a.span();
            *t += dt * a.speed * *dir;
            let len = hi - lo;
            match a.looping {
                Loop::OneShot => *t = t.clamp(lo, hi),
                Loop::Loop if len > 0.0 => {
                    if *t >= hi || *t < lo {
                        *t = lo + (*t - lo).rem_euclid(len);
                    }
                }
                Loop::PingPong if len > 0.0 => {
                    if *t > hi {
                        *t = hi - (*t - hi).min(len);
                        *dir = -*dir;
                    } else if *t < lo {
                        *t = lo + (lo - *t).min(len);
                        *dir = -*dir;
                    }
                }
                _ => *t = lo,
            }
        }
        for (_, p) in &mut self.nested {
            p.advance(dt);
        }
    }

    /// The bytes of what it changes as it plays (its animations' times and
    /// directions, its nested players'), for og-ui's saved state: the
    /// artboard is an asset and is not in them.
    pub fn save(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.write(&mut W(&mut out));
        out
    }

    /// Puts back what [`Player::save`] wrote, onto a player of the same
    /// artboard; refused (and left as it was) when the bytes do not fit it.
    pub fn restore(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mut r = R {
            bytes,
            at: 0,
            legacy_glyphs: false,
            affine_only: false,
        };
        let mut next = self.clone();
        if next.read(&mut r).is_none() || r.at != bytes.len() {
            return Err(format!(
                "the saved state of Rive artboard '{}' does not fit it",
                self.artboard.name
            ));
        }
        *self = next;
        Ok(())
    }

    fn write(&self, w: &mut W) {
        w.u32(self.playing.len() as u32);
        for (i, t, dir) in &self.playing {
            w.u32(*i);
            w.f32(*t);
            w.f32(*dir);
        }
        // The nested players are the artboard's nested nodes, in order.
        for (_, p) in &self.nested {
            p.write(w);
        }
    }

    fn read(&mut self, r: &mut R) -> Option<()> {
        let n = r.u32()? as usize;
        if n > self.artboard.animations.len() {
            return None;
        }
        self.playing = (0..n)
            .map(|_| {
                let (i, t, dir) = (r.u32()?, r.f32()?, r.f32()?);
                ((i as usize) < self.artboard.animations.len()).then_some((i, t, dir))
            })
            .collect::<Option<_>>()?;
        for (_, p) in &mut self.nested {
            p.read(r)?;
        }
        Some(())
    }

    /// Every node's values at the current times.
    pub fn pose(&self) -> Vec<Values> {
        let mut v: Vec<Values> = self.artboard.nodes.iter().map(|n| n.values).collect();
        for (i, t, _) in &self.playing {
            for tr in &self.artboard.animations[*i as usize].tracks {
                let node = &mut v[tr.node as usize];
                if tr.prop == Prop::Color {
                    if let Some(c) = sample_color(&tr.keys, *t) {
                        node.color = c;
                    }
                } else if let Some(x) = sample(&tr.keys, *t) {
                    node.set(tr.prop, x);
                }
            }
        }
        v
    }

    /// The map from the artboard's content to the screen when it is drawn
    /// in `rect` (`[x, y, width, height]`).
    pub fn frame(&self, rect: [f32; 4]) -> Affine {
        let a = &*self.artboard;
        let size = a.size.max(Vec2::splat(1e-3));
        let (bw, bh) = (rect[2], rect[3]);
        let (sx, sy) = match a.fit {
            Fit::Fill => (bw / size.x, bh / size.y),
            Fit::Contain => {
                let s = (bw / size.x).min(bh / size.y);
                (s, s)
            }
            Fit::Cover => {
                let s = (bw / size.x).max(bh / size.y);
                (s, s)
            }
            Fit::FitWidth => (bw / size.x, bw / size.x),
            Fit::FitHeight => (bh / size.y, bh / size.y),
            Fit::None => (1.0, 1.0),
            Fit::ScaleDown => {
                let s = (bw / size.x).min(bh / size.y).min(1.0);
                (s, s)
            }
        };
        let free = Vec2::new(bw - size.x * sx, bh - size.y * sy);
        let at = Vec2::new(rect[0], rect[1]) + free * a.align;
        Affine::translate(at.x, at.y)
            .mul(&Affine::scale(sx, sy))
            .mul(&Affine::translate(a.origin.x * size.x, a.origin.y * size.y))
    }

    /// What it draws in `rect`, bottom to top, at `opacity`.
    pub fn draw(&self, rect: [f32; 4], opacity: f32) -> Vec<Drawn> {
        let mut out = Vec::new();
        self.walk(&self.frame(rect), opacity, "", &mut |s, _| {
            if let Some(s) = s {
                out.push(s);
            }
        });
        out
    }

    /// Its drawn objects in `rect`, bottom to top.
    pub fn elements(&self, rect: [f32; 4]) -> Vec<RiveElement> {
        let mut out = Vec::new();
        self.walk(&self.frame(rect), 1.0, "", &mut |_, e| {
            if let Some(e) = e {
                out.push(e);
            }
        });
        out
    }

    /// Draws through `emit`, `screen` its content's map to the screen;
    /// `emit` gets each shape and, once per drawn object, its element.
    fn walk(
        &self,
        screen: &Affine,
        opacity: f32,
        prefix: &str,
        emit: &mut dyn FnMut(Option<Drawn>, Option<RiveElement>),
    ) {
        let a = &*self.artboard;
        let pose = self.pose();
        let n = a.nodes.len();
        let mut world = vec![Affine::IDENTITY; n];
        let mut alpha = vec![1.0f32; n];
        for i in 0..n {
            let v = &pose[i];
            let local = Affine::translate(v.x, v.y)
                .mul(&Affine::rotate(v.rotation))
                .mul(&Affine::scale(v.scale_x, v.scale_y));
            let p = a.nodes[i].parent as usize;
            if i == 0 {
                alpha[0] = v.opacity * opacity;
            } else {
                world[i] = world[p].mul(&local);
                alpha[i] = alpha[p] * v.opacity;
            }
        }
        let base = if prefix.is_empty() {
            a.name.clone()
        } else {
            format!("{prefix}/{}", a.name)
        };
        // The artboard's background: its frame, in content space.
        if let Kind::Artboard { paints } = &a.nodes[0].kind {
            let mut frame = UiPath::new();
            let o = -a.origin * a.size;
            frame
                .move_to(o)
                .line_to(o + Vec2::new(a.size.x, 0.0))
                .line_to(o + a.size)
                .line_to(o + Vec2::new(0.0, a.size.y))
                .close();
            for &p in paints {
                for s in paint_path(a, &pose, &frame, p, screen, &Affine::IDENTITY, alpha[0]) {
                    emit(Some(Drawn::Shape(s)), None);
                }
            }
        }
        for &d in &a.draws {
            let node = &a.nodes[d as usize];
            let id = format!("{base}/{}", node.name);
            match &node.kind {
                Kind::Shape {
                    paths,
                    paints,
                    hidden,
                    blend,
                } => {
                    if *hidden || alpha[d as usize] <= 0.0 {
                        continue;
                    }
                    // The shape's paths in its own space, so a gradient and
                    // a stroke that the transform does not scale read it.
                    let to_shape = world[d as usize].invert().unwrap_or(Affine::IDENTITY);
                    let mut path = UiPath::new();
                    for &p in paths {
                        let m = to_shape.mul(&world[p as usize]);
                        let one = outline(a, &pose, p);
                        let one = one.map(|q| {
                            let (x, y) = m.apply(q.x, q.y);
                            Vec2::new(x, y)
                        });
                        path.verbs.extend(one.verbs);
                        path.points.extend(one.points);
                    }
                    if path.verbs.is_empty() {
                        continue;
                    }
                    let to_screen = screen.mul(&world[d as usize]);
                    let mut bounds: Option<(Vec2, Vec2)> = None;
                    for &p in paints {
                        for mut s in paint_path(
                            a,
                            &pose,
                            &path,
                            p,
                            &to_screen,
                            &world[d as usize],
                            alpha[d as usize],
                        ) {
                            s.blend = *blend;
                            let (lo, hi) = (s.center - s.half, s.center + s.half);
                            bounds = Some(bounds.map_or((lo, hi), |(a, b)| (a.min(lo), b.max(hi))));
                            emit(Some(Drawn::Shape(s)), None);
                        }
                    }
                    if let Some((lo, hi)) = bounds {
                        emit(
                            None,
                            Some(RiveElement {
                                id,
                                node: d,
                                bounds: [lo.x, lo.y, hi.x - lo.x, hi.y - lo.y],
                                opacity: alpha[d as usize],
                                text: String::new(),
                            }),
                        );
                    }
                }
                Kind::Image { image } => {
                    let alpha = alpha[d as usize];
                    if alpha <= 0.0 {
                        continue;
                    }
                    let img = &a.images[*image as usize];
                    // Rive draws the bitmap one pixel a unit, its origin on
                    // the node (Image::draw).
                    let m = screen.mul(&world[d as usize]);
                    let lo = -pose[d as usize].origin * img.size;
                    let corners = [
                        lo,
                        lo + img.size.with_y(0.0),
                        lo + img.size,
                        lo + img.size.with_x(0.0),
                    ]
                    .map(|c| {
                        let (x, y) = m.apply(c.x, c.y);
                        Vec2::new(x, y)
                    });
                    let (x, y) = m.apply(lo.x + img.size.x / 2.0, lo.y + img.size.y / 2.0);
                    if let Some(texture) = img.texture {
                        emit(
                            Some(Drawn::Shape(UiShape {
                                geometry: UiGeometry::Rect {
                                    radii: [0.0; 4],
                                    corner: og_draw::UiCorner::Round,
                                },
                                center: Vec2::new(x, y),
                                half: img.size / 2.0,
                                transform: [m.a, m.b, m.c, m.d],
                                paint: UiPaint::Image(og_draw::UiImage {
                                    texture,
                                    uv: [0.0, 0.0, 1.0, 1.0],
                                    size: img.size,
                                    density: 1.0,
                                    tint: [1.0, 1.0, 1.0, alpha],
                                    fit: og_draw::UiFit::Fill,
                                    focal: Vec2::splat(0.5),
                                    luminance: false,
                                    offset: Default::default(),
                                    nearest: false,
                                }),
                                mode: UiMode::Fill,
                                opacity: 1.0,
                                blend: UiBlend::Normal,
                                layer: 0,
                                clip: None,
                            })),
                            None,
                        );
                    }
                    let lo = corners.iter().fold(Vec2::MAX, |m, c| m.min(*c));
                    let hi = corners.iter().fold(Vec2::MIN, |m, c| m.max(*c));
                    emit(
                        None,
                        Some(RiveElement {
                            id,
                            node: d,
                            bounds: [lo.x, lo.y, hi.x - lo.x, hi.y - lo.y],
                            opacity: alpha,
                            text: String::new(),
                        }),
                    );
                }
                Kind::Nested { .. } => {
                    let Some((_, p)) = self.nested.iter().find(|(k, _)| *k == d) else {
                        continue;
                    };
                    // A nested artboard draws without its frame origin: its
                    // content's (0, 0) on the node (NestedArtboard::draw).
                    let inner = screen.mul(&world[d as usize]);
                    let corners = [
                        Vec2::ZERO,
                        p.artboard.size.with_y(0.0),
                        p.artboard.size,
                        p.artboard.size.with_x(0.0),
                    ]
                    .map(|c| {
                        let c = c - p.artboard.origin * p.artboard.size;
                        let (x, y) = inner.apply(c.x, c.y);
                        Vec2::new(x, y)
                    });
                    let lo = corners.iter().fold(Vec2::MAX, |m, c| m.min(*c));
                    let hi = corners.iter().fold(Vec2::MIN, |m, c| m.max(*c));
                    emit(
                        None,
                        Some(RiveElement {
                            id: id.clone(),
                            node: d,
                            bounds: [lo.x, lo.y, hi.x - lo.x, hi.y - lo.y],
                            opacity: alpha[d as usize],
                            text: String::new(),
                        }),
                    );
                    p.walk(&inner, alpha[d as usize], &id, emit);
                }
                Kind::Text { .. } => {
                    let alpha = alpha[d as usize];
                    if alpha <= 0.0 {
                        continue;
                    }
                    draw_text(
                        a,
                        &pose,
                        d,
                        &screen.mul(&world[d as usize]),
                        alpha,
                        id,
                        emit,
                    );
                }
                _ => {}
            }
        }
    }
}

/// The value of a number track at `t`.
fn sample(keys: &[Key], t: f32) -> Option<f32> {
    let (i, f) = span(keys, t)?;
    let (a, b) = (keys[i], keys[(i + 1).min(keys.len() - 1)]);
    Some(match a.ease {
        _ if f <= 0.0 || i + 1 >= keys.len() => a.value,
        Ease::Hold => a.value,
        Ease::Linear => a.value + (b.value - a.value) * f,
        Ease::Cubic { x1, y1, x2, y2 } => {
            a.value + (b.value - a.value) * bezier(solve(f, x1, x2), y1, y2)
        }
        Ease::CubicValue { x1, y1, x2, y2 } => {
            // y(t) through the values: from a, controls y1 and y2, to b.
            let s = solve(f, x1, x2);
            let (y0, y3) = (a.value, b.value);
            let ca = y3 + 3.0 * (y1 - y2) - y0;
            let cb = 3.0 * (y2 - y1 * 2.0 + y0);
            let cc = 3.0 * (y1 - y0);
            ((ca * s + cb) * s + cc) * s + y0
        }
    })
}

/// The colour of a colour track at `t` (channels lerped, as Rive does).
fn sample_color(keys: &[Key], t: f32) -> Option<u32> {
    let (i, f) = span(keys, t)?;
    let (a, b) = (keys[i], keys[(i + 1).min(keys.len() - 1)]);
    let f = match a.ease {
        _ if f <= 0.0 || i + 1 >= keys.len() => return Some(a.color),
        Ease::Hold => return Some(a.color),
        Ease::Linear => f,
        Ease::Cubic { x1, y1, x2, y2 } | Ease::CubicValue { x1, y1, x2, y2 } => {
            bezier(solve(f, x1, x2), y1, y2)
        }
    };
    let ch = |c: u32, s: u32| ((c >> s) & 0xFF) as f32;
    let mut out = 0u32;
    for s in [24, 16, 8, 0] {
        let v = ch(a.color, s) + (ch(b.color, s) - ch(a.color, s)) * f;
        out |= (v.round().clamp(0.0, 255.0) as u32) << s;
    }
    Some(out)
}

/// The key `t` falls after and the fraction of the way to the next (0
/// before the first key, which holds; the last key holds after the end).
fn span(keys: &[Key], t: f32) -> Option<(usize, f32)> {
    let first = keys.first()?;
    if t <= first.seconds {
        return Some((0, 0.0));
    }
    let i = keys.partition_point(|k| k.seconds <= t).saturating_sub(1);
    if i + 1 >= keys.len() {
        return Some((keys.len() - 1, 0.0));
    }
    let (a, b) = (keys[i].seconds, keys[i + 1].seconds);
    Some((i, if b > a { (t - a) / (b - a) } else { 1.0 }))
}

/// x(t) or y(t) of the unit cubic through `a1`, `a2`.
fn bezier(t: f32, a1: f32, a2: f32) -> f32 {
    (((1.0 - 3.0 * a2 + 3.0 * a1) * t + (3.0 * a2 - 6.0 * a1)) * t + 3.0 * a1) * t
}

fn slope(t: f32, a1: f32, a2: f32) -> f32 {
    3.0 * (1.0 - 3.0 * a2 + 3.0 * a1) * t * t + 2.0 * (3.0 * a2 - 6.0 * a1) * t + 3.0 * a1
}

/// The t where x(t) = `x` (rive-runtime's `CubicInterpolatorSolver`: an
/// 11-sample table, then Newton or bisection).
fn solve(x: f32, x1: f32, x2: f32) -> f32 {
    const N: usize = 11;
    let step = 1.0 / (N as f32 - 1.0);
    let table: [f32; N] = std::array::from_fn(|i| bezier(i as f32 * step, x1, x2));
    let mut start = 0.0;
    let mut k = 1;
    while k != N - 1 && table[k] <= x {
        start += step;
        k += 1;
    }
    k -= 1;
    let dist = (x - table[k]) / (table[k + 1] - table[k]);
    let mut t = start + dist * step;
    let s0 = slope(t, x1, x2);
    if s0 >= 0.001 {
        for _ in 0..4 {
            let s = slope(t, x1, x2);
            if s == 0.0 {
                return t;
            }
            t -= (bezier(t, x1, x2) - x) / s;
        }
        t
    } else if s0 == 0.0 {
        t
    } else {
        let (mut lo, mut hi) = (start, start + step);
        for _ in 0..10 {
            t = lo + (hi - lo) / 2.0;
            let d = bezier(t, x1, x2) - x;
            if d.abs() <= 1e-7 {
                break;
            }
            if d > 0.0 {
                hi = t;
            } else {
                lo = t;
            }
        }
        t
    }
}

/// Text node `d` drawn through `emit`, `m` its node's map to the screen:
/// each piece of each line in each paint of its run's style, cut to its
/// box when it clips, and its element (its box and words).
fn draw_text(
    a: &Artboard,
    pose: &[Values],
    d: u32,
    m: &Affine,
    alpha: f32,
    id: String,
    emit: &mut dyn FnMut(Option<Drawn>, Option<RiveElement>),
) {
    let Some(laid) = super::text::layout(a, pose, d) else {
        return;
    };
    let to_screen = m.mul(&laid.transform);
    let map = |p: Vec2| {
        let (x, y) = to_screen.apply(p.x, p.y);
        Vec2::new(x, y)
    };
    let quad = |r: [f32; 4]| {
        [
            Vec2::new(r[0], r[1]),
            Vec2::new(r[2], r[1]),
            Vec2::new(r[2], r[3]),
            Vec2::new(r[0], r[3]),
        ]
        .map(map)
    };
    let screen_box = |r: [f32; 4]| {
        let q = quad(r);
        let lo = q.iter().fold(Vec2::MAX, |m, c| m.min(*c));
        let hi = q.iter().fold(Vec2::MIN, |m, c| m.max(*c));
        [lo.x, lo.y, hi.x - lo.x, hi.y - lo.y]
    };
    let linear_map = [to_screen.a, to_screen.b, to_screen.c, to_screen.d];
    let unit = to_screen.determinant().abs().sqrt().max(1e-6);
    if let Some(c) = laid.clip {
        let q = quad(c);
        let mut path = UiPath::new();
        path.move_to(q[0])
            .line_to(q[1])
            .line_to(q[2])
            .line_to(q[3])
            .close();
        emit(Some(Drawn::Clip(path)), None);
    }
    for (piece, pen) in &laid.drawn {
        let r = &laid.runs[piece.run];
        let Some(font) = r.handle else { continue };
        let Kind::TextStyle { paints } = &a.nodes[r.style as usize].kind else {
            continue;
        };
        let (text, placements) = super::text::shaped(r, &piece.text);
        let line_box = screen_box([
            pen.x,
            pen.y - r.ascent * r.size,
            pen.x + piece.width,
            pen.y + r.descent * r.size,
        ]);
        for &p in paints {
            let (shader, visible, stroke) = match &a.nodes[p as usize].kind {
                Kind::Fill {
                    shader, visible, ..
                } => (*shader, *visible, None),
                Kind::Stroke {
                    shader, visible, ..
                } => (*shader, *visible, Some(pose[p as usize].thickness)),
                _ => continue,
            };
            if !visible {
                continue;
            }
            let v = &pose[shader as usize];
            let (color, gradient) = match &a.nodes[shader as usize].kind {
                Kind::Solid => (linear(v.color, alpha), None),
                Kind::Gradient { radial, stops } => {
                    // The gradient's ends in the text's space, as fractions
                    // of the whole text's box on screen.
                    let fill_box = screen_box(laid.bounds);
                    let size = Vec2::new(fill_box[2], fill_box[3]).max(Vec2::splat(1e-3));
                    let lo = Vec2::new(fill_box[0], fill_box[1]);
                    let frac = |q: Vec2| (map(q) - lo) / size;
                    let mut st: Vec<UiStop> = stops
                        .iter()
                        .map(|s| UiStop {
                            offset: pose[*s as usize].position,
                            color: linear(pose[*s as usize].color, alpha * v.opacity),
                        })
                        .collect();
                    st.sort_by(|x, y| x.offset.total_cmp(&y.offset));
                    let to = if *radial {
                        Vec2::splat((v.end - v.start).length() * unit) / size
                    } else {
                        frac(v.end)
                    };
                    (
                        [1.0; 4],
                        Some((
                            UiGradient {
                                kind: if *radial {
                                    UiGradientKind::Radial
                                } else {
                                    UiGradientKind::Linear
                                },
                                from: frac(v.start),
                                to,
                                stops: st,
                                spread: UiSpread::Pad,
                                dither: true,
                            },
                            fill_box,
                        )),
                    )
                }
                _ => continue,
            };
            let (color, outline) = match stroke {
                // A stroke draws its outline; the glyphs' fill stays clear.
                Some(t) if t > 0.0 => ([color[0], color[1], color[2], 0.0], Some((t, color))),
                Some(_) => continue,
                None => (color, None),
            };
            emit(
                Some(Drawn::Text(TextDraw {
                    font,
                    text: text.clone(),
                    placements: placements.clone(),
                    pen: map(*pen),
                    size: r.size,
                    tracking: r.tracking,
                    transform: linear_map,
                    color,
                    gradient: if outline.is_some() { None } else { gradient },
                    outline,
                    bounds: line_box,
                })),
                None,
            );
        }
    }
    if laid.clip.is_some() {
        emit(Some(Drawn::Unclip), None);
    }
    emit(
        None,
        Some(RiveElement {
            id,
            node: d,
            bounds: screen_box(laid.bounds),
            opacity: alpha,
            text: laid.words,
        }),
    );
}

/// Linear RGBA of a Rive colour, its alpha times `alpha`.
fn linear(c: u32, alpha: f32) -> [f32; 4] {
    let mut l = Srgba {
        rgb: [(c >> 16) as u8, (c >> 8) as u8, c as u8],
        alpha: 1.0,
    }
    .linear();
    l[3] = ((c >> 24) & 0xFF) as f32 / 255.0 * alpha;
    l
}

/// Shapes for one paint of `path` (in the paint's space, `to_screen` its
/// map to the screen; `world` the paint's to the artboard, for a stroke
/// whose width the transform does not scale).
fn paint_path(
    a: &Artboard,
    pose: &[Values],
    path: &UiPath,
    paint: u32,
    to_screen: &Affine,
    world: &Affine,
    alpha: f32,
) -> Vec<UiShape> {
    let node = &a.nodes[paint as usize];
    let (shader, visible) = match &node.kind {
        Kind::Fill {
            shader, visible, ..
        }
        | Kind::Stroke {
            shader, visible, ..
        } => (*shader, *visible),
        _ => return Vec::new(),
    };
    if !visible {
        return Vec::new();
    }
    let map = |p: Vec2| {
        let (x, y) = to_screen.apply(p.x, p.y);
        Vec2::new(x, y)
    };
    let screen = path.map(map);
    let paint_of = |lo: Vec2, hi: Vec2| -> Option<UiPaint> {
        let sh = &a.nodes[shader as usize];
        let v = &pose[shader as usize];
        Some(match &sh.kind {
            Kind::Solid => UiPaint::Solid(linear(v.color, alpha)),
            Kind::Gradient { radial, stops } => {
                let size = (hi - lo).max(Vec2::splat(1e-3));
                let frac = |p: Vec2| (map(p) - lo) / size;
                let mut st: Vec<UiStop> = stops
                    .iter()
                    .map(|s| UiStop {
                        offset: pose[*s as usize].position,
                        color: linear(pose[*s as usize].color, alpha * v.opacity),
                    })
                    .collect();
                st.sort_by(|x, y| x.offset.total_cmp(&y.offset));
                let from = frac(v.start);
                let to = if *radial {
                    // The radius |end − start|, scaled as the map scales
                    // area, in fractions of each side.
                    let r = (v.end - v.start).length() * (to_screen.determinant().abs()).sqrt();
                    Vec2::splat(r) / size
                } else {
                    frac(v.end)
                };
                UiPaint::Gradient(UiGradient {
                    kind: if *radial {
                        UiGradientKind::Radial
                    } else {
                        UiGradientKind::Linear
                    },
                    from,
                    to,
                    stops: st,
                    spread: UiSpread::Pad,
                    dither: true,
                })
            }
            _ => return None,
        })
    };
    let shape = |geometry: UiGeometry, lo: Vec2, hi: Vec2, paint: UiPaint| UiShape {
        geometry,
        center: (lo + hi) * 0.5,
        half: (hi - lo) * 0.5,
        transform: [1.0, 0.0, 0.0, 1.0],
        paint,
        mode: UiMode::Fill,
        opacity: 1.0,
        blend: UiBlend::Normal,
        layer: 0,
        clip: None,
    };
    match &node.kind {
        Kind::Fill { rule, .. } => {
            let Some((lo, hi)) = screen.bounds() else {
                return Vec::new();
            };
            let Some(p) = paint_of(lo, hi) else {
                return Vec::new();
            };
            let c = (lo + hi) * 0.5;
            let mut local = screen.map(|q| q - c);
            local.fill_rule = *rule;
            vec![shape(UiGeometry::Path(Arc::new(local)), lo, hi, p)]
        }
        Kind::Stroke {
            cap, join, scaled, ..
        } => {
            let v = &pose[paint as usize];
            // Rive's width is in the shape's space when the transform
            // affects it, else in the artboard's; either way the frame's
            // scale applies.
            let unit = if *scaled {
                to_screen.determinant().abs().sqrt()
            } else {
                let frame = to_screen.mul(&world.invert().unwrap_or(Affine::IDENTITY));
                frame.determinant().abs().sqrt()
            };
            let width = v.thickness * unit;
            if width <= 0.0 {
                return Vec::new();
            }
            let join = match join {
                Join::Miter => UiJoin::Miter(4.0),
                Join::Round => UiJoin::Round,
                Join::Bevel => UiJoin::Bevel,
            };
            let mut out = Vec::new();
            for (line, closed) in screen.flatten(0.25) {
                let lo = line.iter().fold(Vec2::MAX, |m, p| m.min(*p)) - Vec2::splat(width / 2.0);
                let hi = line.iter().fold(Vec2::MIN, |m, p| m.max(*p)) + Vec2::splat(width / 2.0);
                let Some(p) = paint_of(lo, hi) else { continue };
                let c = (lo + hi) * 0.5;
                out.push(shape(
                    UiGeometry::Polyline {
                        points: line.iter().map(|q| *q - c).collect(),
                        closed,
                        filled: false,
                        width,
                        cap: *cap,
                        join,
                    },
                    lo,
                    hi,
                    p,
                ));
            }
            out
        }
        _ => Vec::new(),
    }
}

/// A vertex as `build` takes it: its point, a cubic's in and out
/// controls, a straight one's corner radius.
type PathPoint = (Vec2, Option<(Vec2, Vec2)>, f32);

/// A path node's outline in its own space.
fn outline(a: &Artboard, pose: &[Values], p: u32) -> UiPath {
    let Kind::Path { outline, closed } = &a.nodes[p as usize].kind else {
        return UiPath::new();
    };
    let v = &pose[p as usize];
    let o = Vec2::new(-v.origin.x * v.width, -v.origin.y * v.height);
    let corner = |pos: Vec2, r: f32| (pos, Some(r));
    // Each vertex: its point, and for a cubic its in and out controls; a
    // straight one its corner radius.
    let mut verts: Vec<PathPoint> = Vec::new();
    let polygon = |n: u32, inner: Option<f32>, r: f32| -> Vec<PathPoint> {
        let half = Vec2::new(v.width, v.height) / 2.0;
        let c = o + half;
        let count = if inner.is_some() { n * 2 } else { n };
        let inc = std::f32::consts::TAU / count.max(1) as f32;
        (0..count)
            .map(|i| {
                let ang = -std::f32::consts::FRAC_PI_2 + i as f32 * inc;
                let (s, co) = og_core::math::sin_cos(ang);
                let k = if inner.is_some() && i % 2 == 1 {
                    inner.unwrap_or(1.0)
                } else {
                    1.0
                };
                (c + Vec2::new(co, s) * half * k, None, r)
            })
            .collect()
    };
    match outline {
        Outline::Points { vertices } => {
            for &i in vertices {
                let Kind::Vertex(kind) = a.nodes[i as usize].kind else {
                    continue;
                };
                let w = &pose[i as usize];
                let at = Vec2::new(w.x, w.y);
                let dir = |r: f32| {
                    let (s, c) = og_core::math::sin_cos(r);
                    Vec2::new(c, s)
                };
                verts.push(match kind {
                    Vertex::Straight => (at, None, w.radius),
                    Vertex::Detached => (
                        at,
                        Some((
                            at + dir(w.in_rotation) * w.in_distance,
                            at + dir(w.rotation) * w.out_distance,
                        )),
                        0.0,
                    ),
                    Vertex::Mirrored => {
                        let d = dir(w.rotation) * w.in_distance;
                        (at, Some((at - d, at + d)), 0.0)
                    }
                    Vertex::Asymmetric => (
                        at,
                        Some((
                            at - dir(w.rotation) * w.in_distance,
                            at + dir(w.rotation) * w.out_distance,
                        )),
                        0.0,
                    ),
                });
            }
        }
        Outline::Rect => {
            let (w, h) = (v.width, v.height);
            for (k, q) in [
                o,
                o + Vec2::new(w, 0.0),
                o + Vec2::new(w, h),
                o + Vec2::new(0.0, h),
            ]
            .into_iter()
            .enumerate()
            {
                let (pos, r) = corner(q, v.corners[k]);
                verts.push((pos, None, r.unwrap_or(0.0)));
            }
        }
        Outline::Ellipse => {
            let r = Vec2::new(v.width, v.height) / 2.0;
            let c = o + r;
            // rive-runtime's circleConstant.
            let k = 0.552_284_8;
            verts = vec![
                (
                    c - Vec2::new(0.0, r.y),
                    Some((c + Vec2::new(-r.x * k, -r.y), c + Vec2::new(r.x * k, -r.y))),
                    0.0,
                ),
                (
                    c + Vec2::new(r.x, 0.0),
                    Some((c + Vec2::new(r.x, -r.y * k), c + Vec2::new(r.x, r.y * k))),
                    0.0,
                ),
                (
                    c + Vec2::new(0.0, r.y),
                    Some((c + Vec2::new(r.x * k, r.y), c + Vec2::new(-r.x * k, r.y))),
                    0.0,
                ),
                (
                    c - Vec2::new(r.x, 0.0),
                    Some((c + Vec2::new(-r.x, r.y * k), c + Vec2::new(-r.x, -r.y * k))),
                    0.0,
                ),
            ];
        }
        Outline::Triangle => {
            verts = vec![
                (o + Vec2::new(v.width / 2.0, 0.0), None, 0.0),
                (o + Vec2::new(v.width, v.height), None, 0.0),
                (o + Vec2::new(0.0, v.height), None, 0.0),
            ];
        }
        Outline::Polygon { points } => verts = polygon(*points, None, v.corners[0]),
        Outline::Star { points } => verts = polygon(*points, Some(v.inner), v.corners[0]),
    }
    build(
        &verts,
        *closed || !matches!(outline, Outline::Points { .. }),
    )
}

/// rive-runtime's `computeIdealControlPointDistance`.
fn ideal(to_prev: Vec2, to_next: Vec2, radius: f32) -> f32 {
    let angle = og_core::math::atan2(to_prev.perp_dot(to_next), to_prev.dot(to_next)).abs();
    let pi = std::f32::consts::PI;
    let (s, c) = og_core::math::sin_cos(angle);
    let q = pi / (2.0 * ((2.0 * pi) / angle.max(1e-6)));
    let (qs, qc) = og_core::math::sin_cos(q);
    radius.min(4.0 / 3.0 * (qs / qc) * radius * if angle < pi / 2.0 { 1.0 + c } else { 2.0 - s })
}

/// rive-runtime's `Path::buildPath` over `verts` (point, cubic controls,
/// straight radius).
fn build(verts: &[PathPoint], closed: bool) -> UiPath {
    let mut path = UiPath::new();
    let n = verts.len();
    if n < 2 {
        return path;
    }
    let in_of = |i: usize| verts[i].1.map_or(verts[i].0, |c| c.0);
    let out_of = |i: usize| verts[i].1.map_or(verts[i].0, |c| c.1);
    let rounded = |i: usize, prev: usize, next: usize| {
        let pos = verts[i].0;
        let radius = verts[i].2;
        let tp = out_of(prev) - pos;
        let tn = in_of(next) - pos;
        let (lp, ln) = (tp.length(), tn.length());
        let (tp, tn) = (tp / lp.max(1e-6), tn / ln.max(1e-6));
        let r = (lp / 2.0).min(ln / 2.0).min(radius.abs());
        let d = ideal(tp, tn, r);
        let enter = pos + tp * r;
        let mut c1 = pos + tp * (r - d);
        let mut c2 = pos + tn * (r - d);
        let leave = pos + tn * r;
        if radius < 0.0 {
            // Cut inwards: the controls turned about the ends.
            let v1 = enter - leave;
            let v2 = pos - leave;
            let ang = og_core::math::atan2(v1.perp_dot(v2), v1.dot(v2));
            let rot = |p: Vec2, about: Vec2, a: f32| {
                let (s, c) = og_core::math::sin_cos(a);
                let q = p - about;
                about + Vec2::new(q.x * c - q.y * s, q.x * s + q.y * c)
            };
            c1 = rot(c1, enter, ang * 2.0);
            c2 = rot(c2, leave, -ang * 2.0);
        }
        (enter, c1, c2, leave)
    };
    let (start, start_in, mut out, mut prev_cubic, start_cubic);
    if let Some((cin, cout)) = verts[0].1 {
        start = verts[0].0;
        start_in = cin;
        out = cout;
        prev_cubic = true;
        start_cubic = true;
        path.move_to(start);
    } else if verts[0].2 != 0.0 {
        let (enter, c1, c2, leave) = rounded(0, n - 1, 1);
        start = enter;
        start_in = enter;
        path.move_to(enter);
        path.cubic_to(c1, c2, leave);
        out = leave;
        prev_cubic = false;
        start_cubic = false;
    } else {
        start = verts[0].0;
        start_in = start;
        out = start;
        prev_cubic = false;
        start_cubic = false;
        path.move_to(start);
    }
    for (i, &(pos, controls, radius)) in verts.iter().enumerate().skip(1) {
        if let Some((cin, cout)) = controls {
            path.cubic_to(out, cin, pos);
            prev_cubic = true;
            out = cout;
        } else if radius != 0.0 {
            let (enter, c1, c2, leave) = rounded(i, i - 1, (i + 1) % n);
            if prev_cubic {
                path.cubic_to(out, enter, enter);
            } else {
                path.line_to(enter);
            }
            path.cubic_to(c1, c2, leave);
            prev_cubic = false;
            out = leave;
        } else if prev_cubic {
            path.cubic_to(out, pos, pos);
            prev_cubic = false;
            out = pos;
        } else {
            out = pos;
            path.line_to(pos);
        }
    }
    if closed {
        if prev_cubic || start_cubic {
            path.cubic_to(out, start_in, start);
        } else {
            path.line_to(start);
        }
        path.close();
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(seconds: f32, value: f32, ease: Ease) -> Key {
        Key {
            seconds,
            value,
            color: 0,
            ease,
        }
    }

    #[test]
    fn keys_hold_ease_and_clamp_at_their_ends() {
        let keys = [
            key(1.0, 10.0, Ease::Linear),
            key(2.0, 20.0, Ease::Hold),
            key(3.0, 40.0, Ease::Linear),
        ];
        assert_eq!(sample(&keys, 0.0), Some(10.0), "before the first: it holds");
        assert_eq!(sample(&keys, 1.25), Some(12.5));
        assert_eq!(sample(&keys, 2.9), Some(20.0), "a hold key keeps its value");
        assert_eq!(sample(&keys, 9.0), Some(40.0), "after the last: it holds");
        // ease-in-out (0.42, 0, 0.58, 1) is symmetric: half way at t = ½,
        // slower than linear at a quarter.
        let e = Ease::Cubic {
            x1: 0.42,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        };
        let k = [key(0.0, 0.0, e), key(1.0, 100.0, e)];
        assert!((sample(&k, 0.5).unwrap() - 50.0).abs() < 0.05);
        let q = sample(&k, 0.25).unwrap();
        assert!(q < 20.0 && q > 5.0, "{q}");
    }

    /// A ping-ponging animation on a nested artboard inside a looping one:
    /// saved mid-way, restored onto a fresh player and run on, it matches
    /// the run that was never interrupted, bit for bit.
    #[test]
    fn a_restored_player_runs_on_as_the_unbroken_one() {
        let anim = |looping, tracks| Animation {
            name: "a".into(),
            fps: 60,
            duration: 60,
            speed: 1.0,
            looping,
            work: None,
            tracks,
        };
        let x_track = |node| Track {
            node,
            prop: Prop::X,
            keys: vec![key(0.0, 0.0, Ease::Linear), key(1.0, 100.0, Ease::Linear)],
        };
        let node = |kind, parent| Node {
            name: "n".into(),
            parent,
            kind,
            values: Values::default(),
        };
        let inner = Artboard {
            name: "inner".into(),
            nodes: vec![
                node(Kind::Artboard { paints: vec![] }, 0),
                node(Kind::Group, 0),
            ],
            animations: vec![anim(Loop::PingPong, vec![x_track(1)])],
            ..Artboard::default()
        };
        let outer = Arc::new(Artboard {
            name: "outer".into(),
            nodes: vec![
                node(Kind::Artboard { paints: vec![] }, 0),
                node(Kind::Group, 0),
                node(
                    Kind::Nested {
                        artboard: 0,
                        animations: vec![NestedAnimation {
                            animation: 0,
                            speed: 1.0,
                            playing: true,
                        }],
                    },
                    0,
                ),
            ],
            animations: vec![anim(Loop::Loop, vec![x_track(1)])],
            play: Some(0),
            nested: vec![inner],
            ..Artboard::default()
        });
        let step = 1.0 / 60.0;
        let mut whole = Player::new(outer.clone(), None);
        // 77 ticks: past the outer loop's end and the inner one's turn.
        for _ in 0..77 {
            whole.advance(step);
        }
        let saved = whole.save();
        for _ in 0..50 {
            whole.advance(step);
        }
        let mut restored = Player::new(outer.clone(), None);
        restored.restore(&saved).expect("fits");
        // Saved while the inner ping-pong was on its way back.
        assert!(
            restored.nested[0].1.playing[0].2 < 0.0,
            "the inner one runs back"
        );
        for _ in 0..50 {
            restored.advance(step);
        }
        assert_eq!(restored, whole);
        assert_eq!(restored.pose(), whole.pose());
        // Bytes of another artboard's player do not fit.
        assert!(
            Player::new(Arc::new(Artboard::default()), None)
                .restore(&saved)
                .is_err()
        );
    }

    #[test]
    fn colours_lerp_by_channel() {
        let k = [
            Key {
                seconds: 0.0,
                value: 0.0,
                color: 0xFF00_00FF,
                ease: Ease::Linear,
            },
            Key {
                seconds: 1.0,
                value: 0.0,
                color: 0x00FF_0000,
                ease: Ease::Linear,
            },
        ];
        assert_eq!(sample_color(&k, 0.5), Some(0x8080_0080));
    }

    #[test]
    fn a_rounded_corner_starts_and_ends_on_its_edges() {
        // A 100 px square, its first corner rounded by 20.
        let v = [
            (Vec2::new(0.0, 0.0), None, 20.0),
            (Vec2::new(100.0, 0.0), None, 0.0),
            (Vec2::new(100.0, 100.0), None, 0.0),
            (Vec2::new(0.0, 100.0), None, 0.0),
        ];
        let p = build(&v, true);
        // Starts 20 down the left edge (towards the previous vertex) and
        // its first curve ends 20 along the top.
        assert_eq!(p.points[0], Vec2::new(0.0, 20.0));
        assert_eq!(p.points[3], Vec2::new(20.0, 0.0));
        assert_eq!(p.verbs.last(), Some(&og_draw::UiVerb::Close));
    }
}
