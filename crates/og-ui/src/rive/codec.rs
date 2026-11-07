//! The bytes of an [`Artboard`]: what og-cook writes into a Rive
//! component blob and the runtime reads back. Little endian, versioned by
//! [`VERSION`]; a reader refuses another version rather than guess.

use super::*;
use crate::motion::codec::{R, W};
use og_render::UiBlend;

/// The codec's version (in the blob after its magic).
pub const VERSION: u32 = 3;
const MAGIC: &[u8; 4] = b"OGRV";

/// The bytes of `a`.
pub fn encode(a: &Artboard) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    let mut w = W(&mut out);
    w.u32(VERSION);
    artboard(&mut w, a);
    out
}

/// An artboard from [`encode`]'s bytes, checked: every index in range.
pub fn decode(bytes: &[u8]) -> Result<Artboard, String> {
    if bytes.get(..4) != Some(MAGIC.as_slice()) {
        return Err("not a Rive artboard (no OGRV magic)".into());
    }
    let mut r = R { bytes, at: 4 };
    let v = r.u32().ok_or("the artboard is cut short")?;
    if v != VERSION {
        return Err(format!(
            "a Rive artboard of codec version {v}; this engine reads {VERSION} (cook it again)"
        ));
    }
    let a = rartboard(&mut r, 0).ok_or("the artboard is cut short or malformed")?;
    check(&a)?;
    Ok(a)
}

fn list(w: &mut W, v: &[u32]) {
    w.u32(v.len() as u32);
    for x in v {
        w.u32(*x);
    }
}

fn rlist(r: &mut R) -> Option<Vec<u32>> {
    let n = r.u32()? as usize;
    if n > r.bytes.len() {
        return None;
    }
    (0..n).map(|_| r.u32()).collect()
}

fn blend(b: UiBlend) -> u8 {
    match b {
        UiBlend::Normal => 0,
        UiBlend::Multiply => 1,
        UiBlend::Screen => 2,
        UiBlend::Add => 3,
        UiBlend::Overlay => 4,
        UiBlend::SoftLight => 5,
        UiBlend::ColorDodge => 6,
    }
}

fn rblend(v: u8) -> Option<UiBlend> {
    Some(match v {
        0 => UiBlend::Normal,
        1 => UiBlend::Multiply,
        2 => UiBlend::Screen,
        3 => UiBlend::Add,
        4 => UiBlend::Overlay,
        5 => UiBlend::SoftLight,
        6 => UiBlend::ColorDodge,
        _ => return None,
    })
}

fn cap(c: UiCap) -> u8 {
    match c {
        UiCap::Butt => 0,
        UiCap::Round => 1,
        UiCap::Square => 2,
    }
}

fn rcap(v: u8) -> Option<UiCap> {
    Some(match v {
        0 => UiCap::Butt,
        1 => UiCap::Round,
        2 => UiCap::Square,
        _ => return None,
    })
}

fn values(w: &mut W, v: &Values) {
    for p in Prop::ALL {
        if p != Prop::Color {
            w.f32(v.get(p));
        }
    }
    w.u32(v.color);
}

fn rvalues(r: &mut R) -> Option<Values> {
    let mut v = Values::default();
    for p in Prop::ALL {
        if p != Prop::Color {
            v.set(p, r.f32()?);
        }
    }
    v.color = r.u32()?;
    Some(v)
}

fn artboard(w: &mut W, a: &Artboard) {
    w.str(&a.name);
    w.vec2(a.size);
    w.vec2(a.origin);
    w.u8(u8::from(a.clip));
    w.u8(Fit::ALL.iter().position(|f| f.0 == a.fit).unwrap_or(1) as u8);
    w.vec2(a.align);
    w.str(&a.role);
    w.str(&a.label);
    w.u32(a.nodes.len() as u32);
    for n in &a.nodes {
        w.str(&n.name);
        w.u32(n.parent);
        kind(w, &n.kind);
        values(w, &n.values);
    }
    list(w, &a.draws);
    w.u32(a.animations.len() as u32);
    for an in &a.animations {
        w.str(&an.name);
        w.u32(an.fps);
        w.u32(an.duration);
        w.f32(an.speed);
        w.u8(an.looping as u8);
        let (ws, we) = an.work.unwrap_or((u32::MAX, u32::MAX));
        w.u32(ws);
        w.u32(we);
        w.u32(an.tracks.len() as u32);
        for t in &an.tracks {
            w.u32(t.node);
            w.u8(Prop::ALL.iter().position(|q| *q == t.prop).unwrap_or(0) as u8);
            w.u32(t.keys.len() as u32);
            for k in &t.keys {
                w.f32(k.seconds);
                w.f32(k.value);
                w.u32(k.color);
                match k.ease {
                    Ease::Hold => w.u8(0),
                    Ease::Linear => w.u8(1),
                    Ease::Cubic { x1, y1, x2, y2 } | Ease::CubicValue { x1, y1, x2, y2 } => {
                        w.u8(if matches!(k.ease, Ease::Cubic { .. }) {
                            2
                        } else {
                            3
                        });
                        for v in [x1, y1, x2, y2] {
                            w.f32(v);
                        }
                    }
                }
            }
        }
    }
    w.u32(a.machines.len() as u32);
    for (name, play) in &a.machines {
        w.str(name);
        w.u32(play.unwrap_or(u32::MAX));
    }
    w.u32(a.play.unwrap_or(u32::MAX));
    w.u32(a.nested.len() as u32);
    for n in &a.nested {
        artboard(w, n);
    }
    w.u32(a.images.len() as u32);
    for i in &a.images {
        w.str(&i.id);
        w.vec2(i.size);
    }
    w.u32(a.fonts.len() as u32);
    for f in &a.fonts {
        w.str(&f.id);
        w.f32(f.ascent);
        w.f32(f.descent);
    }
    w.u32(a.strings.len() as u32);
    for t in &a.strings {
        w.str(t);
    }
}

fn kind(w: &mut W, k: &Kind) {
    match k {
        Kind::Artboard { paints } => {
            w.u8(0);
            list(w, paints);
        }
        Kind::Group => w.u8(1),
        Kind::Shape {
            paths,
            paints,
            hidden,
            blend: b,
        } => {
            w.u8(2);
            list(w, paths);
            list(w, paints);
            w.u8(u8::from(*hidden));
            w.u8(blend(*b));
        }
        Kind::Path { outline, closed } => {
            w.u8(3);
            w.u8(u8::from(*closed));
            match outline {
                Outline::Points { vertices } => {
                    w.u8(0);
                    list(w, vertices);
                }
                Outline::Rect => w.u8(1),
                Outline::Ellipse => w.u8(2),
                Outline::Triangle => w.u8(3),
                Outline::Polygon { points } => {
                    w.u8(4);
                    w.u32(*points);
                }
                Outline::Star { points } => {
                    w.u8(5);
                    w.u32(*points);
                }
            }
        }
        Kind::Vertex(v) => {
            w.u8(4);
            w.u8(*v as u8);
        }
        Kind::Fill {
            rule,
            shader,
            visible,
        } => {
            w.u8(5);
            w.u8(u8::from(*rule == UiFillRule::NonZero));
            w.u32(*shader);
            w.u8(u8::from(*visible));
        }
        Kind::Stroke {
            cap: c,
            join,
            scaled,
            shader,
            visible,
        } => {
            w.u8(6);
            w.u8(cap(*c));
            w.u8(*join as u8);
            w.u8(u8::from(*scaled));
            w.u32(*shader);
            w.u8(u8::from(*visible));
        }
        Kind::Solid => w.u8(7),
        Kind::Gradient { radial, stops } => {
            w.u8(8);
            w.u8(u8::from(*radial));
            list(w, stops);
        }
        Kind::Stop => w.u8(9),
        Kind::Image { image } => {
            w.u8(11);
            w.u32(*image);
        }
        Kind::Text { runs, layout: l } => {
            w.u8(12);
            list(w, runs);
            w.u8(l.sizing as u8);
            w.u8(l.overflow as u8);
            w.u8(l.align as u8);
            w.u8(l.valign as u8);
            w.u8(u8::from(l.wrap)
                | u8::from(l.break_words) << 1
                | u8::from(l.baseline) << 2
                | u8::from(l.fit_from_baseline) << 3);
        }
        Kind::TextRun { style } => {
            w.u8(13);
            w.u32(*style);
        }
        Kind::TextStyle { paints } => {
            w.u8(14);
            list(w, paints);
        }
        Kind::Nested {
            artboard,
            animations,
        } => {
            w.u8(10);
            w.u32(*artboard);
            w.u32(animations.len() as u32);
            for a in animations {
                w.u32(a.animation);
                w.f32(a.speed);
                w.u8(u8::from(a.playing));
            }
        }
    }
}

fn rkind(r: &mut R) -> Option<Kind> {
    Some(match r.u8()? {
        0 => Kind::Artboard { paints: rlist(r)? },
        1 => Kind::Group,
        2 => Kind::Shape {
            paths: rlist(r)?,
            paints: rlist(r)?,
            hidden: r.u8()? != 0,
            blend: rblend(r.u8()?)?,
        },
        3 => {
            let closed = r.u8()? != 0;
            let outline = match r.u8()? {
                0 => Outline::Points {
                    vertices: rlist(r)?,
                },
                1 => Outline::Rect,
                2 => Outline::Ellipse,
                3 => Outline::Triangle,
                4 => Outline::Polygon { points: r.u32()? },
                5 => Outline::Star { points: r.u32()? },
                _ => return None,
            };
            Kind::Path { outline, closed }
        }
        4 => Kind::Vertex(match r.u8()? {
            0 => Vertex::Straight,
            1 => Vertex::Detached,
            2 => Vertex::Mirrored,
            3 => Vertex::Asymmetric,
            _ => return None,
        }),
        5 => Kind::Fill {
            rule: if r.u8()? != 0 {
                UiFillRule::NonZero
            } else {
                UiFillRule::EvenOdd
            },
            shader: r.u32()?,
            visible: r.u8()? != 0,
        },
        6 => Kind::Stroke {
            cap: rcap(r.u8()?)?,
            join: match r.u8()? {
                0 => Join::Miter,
                1 => Join::Round,
                2 => Join::Bevel,
                _ => return None,
            },
            scaled: r.u8()? != 0,
            shader: r.u32()?,
            visible: r.u8()? != 0,
        },
        7 => Kind::Solid,
        8 => Kind::Gradient {
            radial: r.u8()? != 0,
            stops: rlist(r)?,
        },
        9 => Kind::Stop,
        10 => {
            let artboard = r.u32()?;
            let n = r.u32()? as usize;
            if n > r.bytes.len() {
                return None;
            }
            let animations = (0..n)
                .map(|_| {
                    Some(NestedAnimation {
                        animation: r.u32()?,
                        speed: r.f32()?,
                        playing: r.u8()? != 0,
                    })
                })
                .collect::<Option<Vec<_>>>()?;
            Kind::Nested {
                artboard,
                animations,
            }
        }
        11 => Kind::Image { image: r.u32()? },
        12 => {
            let runs = rlist(r)?;
            let sizing = match r.u8()? {
                0 => Sizing::AutoWidth,
                1 => Sizing::AutoHeight,
                2 => Sizing::Fixed,
                _ => return None,
            };
            let overflow = match r.u8()? {
                0 => Overflow::Visible,
                1 => Overflow::Hidden,
                2 => Overflow::Clipped,
                3 => Overflow::Ellipsis,
                4 => Overflow::Fit,
                5 => Overflow::FitFontSize,
                _ => return None,
            };
            let align = match r.u8()? {
                0 => HAlign::Left,
                1 => HAlign::Right,
                2 => HAlign::Center,
                _ => return None,
            };
            let valign = match r.u8()? {
                0 => VAlign::Top,
                1 => VAlign::Bottom,
                2 => VAlign::Middle,
                _ => return None,
            };
            let flags = r.u8()?;
            Kind::Text {
                runs,
                layout: TextLayout {
                    sizing,
                    overflow,
                    align,
                    valign,
                    wrap: flags & 1 != 0,
                    break_words: flags & 2 != 0,
                    baseline: flags & 4 != 0,
                    fit_from_baseline: flags & 8 != 0,
                },
            }
        }
        13 => Kind::TextRun { style: r.u32()? },
        14 => Kind::TextStyle { paints: rlist(r)? },
        _ => return None,
    })
}

fn rartboard(r: &mut R, depth: u32) -> Option<Artboard> {
    if depth > 16 {
        return None;
    }
    let mut a = Artboard {
        name: r.str()?,
        ..Artboard::default()
    };
    a.size = Vec2::new(r.f32()?, r.f32()?);
    a.origin = Vec2::new(r.f32()?, r.f32()?);
    a.clip = r.u8()? != 0;
    a.fit = Fit::ALL.get(usize::from(r.u8()?))?.0;
    a.align = Vec2::new(r.f32()?, r.f32()?);
    a.role = r.str()?;
    a.label = r.str()?;
    let n = r.u32()? as usize;
    if n > r.bytes.len() {
        return None;
    }
    for _ in 0..n {
        a.nodes.push(Node {
            name: r.str()?,
            parent: r.u32()?,
            kind: rkind(r)?,
            values: rvalues(r)?,
        });
    }
    a.draws = rlist(r)?;
    let n = r.u32()? as usize;
    if n > r.bytes.len() {
        return None;
    }
    for _ in 0..n {
        let name = r.str()?;
        let fps = r.u32()?;
        let duration = r.u32()?;
        let speed = r.f32()?;
        let looping = match r.u8()? {
            0 => Loop::OneShot,
            1 => Loop::Loop,
            2 => Loop::PingPong,
            _ => return None,
        };
        let (ws, we) = (r.u32()?, r.u32()?);
        let tn = r.u32()? as usize;
        if tn > r.bytes.len() {
            return None;
        }
        let mut tracks = Vec::with_capacity(tn);
        for _ in 0..tn {
            let node = r.u32()?;
            let prop = *Prop::ALL.get(usize::from(r.u8()?))?;
            let kn = r.u32()? as usize;
            if kn > r.bytes.len() {
                return None;
            }
            let mut keys = Vec::with_capacity(kn);
            for _ in 0..kn {
                let seconds = r.f32()?;
                let value = r.f32()?;
                let color = r.u32()?;
                let ease = match r.u8()? {
                    0 => Ease::Hold,
                    1 => Ease::Linear,
                    t @ (2 | 3) => {
                        let (x1, y1, x2, y2) = (r.f32()?, r.f32()?, r.f32()?, r.f32()?);
                        if t == 2 {
                            Ease::Cubic { x1, y1, x2, y2 }
                        } else {
                            Ease::CubicValue { x1, y1, x2, y2 }
                        }
                    }
                    _ => return None,
                };
                keys.push(Key {
                    seconds,
                    value,
                    color,
                    ease,
                });
            }
            tracks.push(Track { node, prop, keys });
        }
        a.animations.push(Animation {
            name,
            fps,
            duration,
            speed,
            looping,
            work: (ws != u32::MAX).then_some((ws, we)),
            tracks,
        });
    }
    let n = r.u32()? as usize;
    if n > r.bytes.len() {
        return None;
    }
    for _ in 0..n {
        let name = r.str()?;
        let play = r.u32()?;
        a.machines.push((name, (play != u32::MAX).then_some(play)));
    }
    let play = r.u32()?;
    a.play = (play != u32::MAX).then_some(play);
    let n = r.u32()? as usize;
    if n > r.bytes.len() {
        return None;
    }
    for _ in 0..n {
        a.nested.push(rartboard(r, depth + 1)?);
    }
    let n = r.u32()? as usize;
    if n > r.bytes.len() {
        return None;
    }
    for _ in 0..n {
        a.images.push(Image {
            id: r.str()?,
            size: Vec2::new(r.f32()?, r.f32()?),
            texture: None,
        });
    }
    let n = r.u32()? as usize;
    if n > r.bytes.len() {
        return None;
    }
    for _ in 0..n {
        a.fonts.push(Font {
            id: r.str()?,
            ascent: r.f32()?,
            descent: r.f32()?,
            font: None,
        });
    }
    let n = r.u32()? as usize;
    if n > r.bytes.len() {
        return None;
    }
    for _ in 0..n {
        a.strings.push(r.str()?);
    }
    Some(a)
}

/// Whether every index `a` holds is in range, parents come first and the
/// nesting has no cycle (an artboard's nested ones are its own copies).
pub fn check(a: &Artboard) -> Result<(), String> {
    let n = a.nodes.len() as u32;
    let bad = |what: &str| Err(format!("artboard '{}': {what} out of range", a.name));
    if !matches!(
        a.nodes.first().map(|n| &n.kind),
        Some(Kind::Artboard { .. })
    ) {
        return Err(format!("artboard '{}': node 0 is not the artboard", a.name));
    }
    for (i, node) in a.nodes.iter().enumerate() {
        if i > 0 && node.parent >= i as u32 {
            return bad("a parent");
        }
        let idx: Vec<u32> = match &node.kind {
            Kind::Artboard { paints } => paints.clone(),
            Kind::Shape { paths, paints, .. } => paths.iter().chain(paints).copied().collect(),
            Kind::Path {
                outline: Outline::Points { vertices },
                ..
            } => vertices.clone(),
            Kind::Fill { shader, .. } | Kind::Stroke { shader, .. } => vec![*shader],
            Kind::Gradient { stops, .. } => stops.clone(),
            Kind::Text { runs, .. } => runs.clone(),
            Kind::TextRun { style } => {
                if node.values.text as usize >= a.strings.len() {
                    return bad("a run's words");
                }
                vec![*style]
            }
            Kind::TextStyle { paints } => {
                if node.values.font != NO_FONT && node.values.font as usize >= a.fonts.len() {
                    return bad("a style's font");
                }
                paints.clone()
            }
            Kind::Image { image } => {
                if *image as usize >= a.images.len() {
                    return bad("an image");
                }
                Vec::new()
            }
            Kind::Nested {
                artboard,
                animations,
            } => {
                let Some(nested) = a.nested.get(*artboard as usize) else {
                    return bad("a nested artboard");
                };
                if animations
                    .iter()
                    .any(|x| x.animation as usize >= nested.animations.len())
                {
                    return bad("a nested animation");
                }
                Vec::new()
            }
            _ => Vec::new(),
        };
        if idx.iter().any(|j| *j >= n) {
            return bad("a node index");
        }
    }
    let anims = a.animations.len() as u32;
    if a.play.is_some_and(|p| p >= anims)
        || a.machines.iter().any(|m| m.1.is_some_and(|p| p >= anims))
    {
        return bad("a played animation");
    }
    if a.draws.iter().any(|d| *d >= n) {
        return bad("a drawn node");
    }
    if a.animations
        .iter()
        .flat_map(|x| &x.tracks)
        .any(|t| t.node >= n)
    {
        return bad("an animated node");
    }
    // Keyed words and fonts index the artboard's.
    for t in a.animations.iter().flat_map(|x| &x.tracks) {
        let limit = match t.prop {
            Prop::Text => a.strings.len(),
            Prop::Font => a.fonts.len(),
            _ => continue,
        };
        if t.keys
            .iter()
            .any(|k| k.value < 0.0 || k.value as usize >= limit)
        {
            return bad("a keyed word or font");
        }
    }
    a.nested.iter().try_for_each(check)
}
