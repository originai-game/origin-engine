//! Bytes of the UI pass's draw commands ([`DrawCmd::Shape`], text effects,
//! vectors, clip and layer brackets) for the motion state's frozen drawing
//! of leaving elements ([`crate::motion::codec`], its tag 6): a restored
//! state draws a leaving element's paints, effects, masks and layers as it
//! left.

use crate::draw::DrawCmd;
use crate::motion::codec::{R, W};
use og_core::glam::Vec2;
use og_render::{
    TextureHandle, UiAlign, UiBlend, UiBuiltin, UiCap, UiCorner, UiEdge, UiFit, UiGeometry,
    UiGradient, UiGradientKind, UiImage, UiJoin, UiMask, UiMaterial, UiMaterialHandle, UiMode,
    UiPaint, UiShape, UiSpread, UiStop, UiStroke, UiTextEffects, UiTextOutline, UiTextShadow,
    UiVectorHandle,
};

fn v2(w: &mut W, v: Vec2) {
    w.f32(v.x);
    w.f32(v.y);
}
fn f4(w: &mut W, v: &[f32; 4]) {
    for x in v {
        w.f32(*x);
    }
}
fn rv2(r: &mut R) -> Option<Vec2> {
    Some(Vec2::new(r.f32()?, r.f32()?))
}
fn rf4(r: &mut R) -> Option<[f32; 4]> {
    Some([r.f32()?, r.f32()?, r.f32()?, r.f32()?])
}

fn cap(c: UiCap) -> u8 {
    match c {
        UiCap::Butt => 0,
        UiCap::Round => 1,
        UiCap::Square => 2,
    }
}
fn rcap(r: &mut R) -> Option<UiCap> {
    Some(match r.u8()? {
        0 => UiCap::Butt,
        1 => UiCap::Round,
        2 => UiCap::Square,
        _ => return None,
    })
}

fn align(a: UiAlign) -> u8 {
    match a {
        UiAlign::Inside => 0,
        UiAlign::Center => 1,
        UiAlign::Outside => 2,
    }
}
fn ralign(r: &mut R) -> Option<UiAlign> {
    Some(match r.u8()? {
        0 => UiAlign::Inside,
        1 => UiAlign::Center,
        2 => UiAlign::Outside,
        _ => return None,
    })
}

const BLENDS: [UiBlend; 7] = [
    UiBlend::Normal,
    UiBlend::Multiply,
    UiBlend::Screen,
    UiBlend::Add,
    UiBlend::Overlay,
    UiBlend::SoftLight,
    UiBlend::ColorDodge,
];
const BUILTINS: [UiBuiltin; 10] = [
    UiBuiltin::None,
    UiBuiltin::Dissolve,
    UiBuiltin::Shine,
    UiBuiltin::Hologram,
    UiBuiltin::Glitch,
    UiBuiltin::Ripple,
    UiBuiltin::Pixelate,
    UiBuiltin::ColorAdjust,
    UiBuiltin::Outline,
    UiBuiltin::Blur,
];

fn index_of<T: PartialEq>(all: &[T], v: &T) -> u8 {
    all.iter().position(|x| x == v).unwrap_or(0) as u8
}

fn geometry(w: &mut W, g: &UiGeometry) {
    match g {
        UiGeometry::Rect { radii, corner } => {
            w.u8(0);
            f4(w, radii);
            match corner {
                UiCorner::Round => w.u8(0),
                UiCorner::Squircle(k) => {
                    w.u8(1);
                    w.f32(*k);
                }
                UiCorner::Bevel => w.u8(2),
                UiCorner::Notch => w.u8(3),
            }
        }
        UiGeometry::Ellipse => w.u8(1),
        UiGeometry::Arc {
            start,
            sweep,
            thickness,
            cap: c,
        } => {
            w.u8(2);
            w.f32(*start);
            w.f32(*sweep);
            w.f32(*thickness);
            w.u8(cap(*c));
        }
        UiGeometry::Polygon {
            sides,
            inner,
            rounding,
            rotation,
        } => {
            w.u8(3);
            w.u32(*sides);
            w.f32(*inner);
            w.f32(*rounding);
            w.f32(*rotation);
        }
        UiGeometry::Polyline {
            points,
            closed,
            filled,
            width,
            cap: c,
            join,
        } => {
            w.u8(4);
            w.u32(points.len() as u32);
            for p in points {
                v2(w, *p);
            }
            w.u8(if *filled { 2 } else { u8::from(*closed) });
            w.f32(*width);
            w.u8(cap(*c));
            match join {
                UiJoin::Miter(l) => {
                    w.u8(0);
                    w.f32(*l);
                }
                UiJoin::Round => w.u8(1),
                UiJoin::Bevel => w.u8(2),
            }
        }
    }
}

fn rgeometry(r: &mut R) -> Option<UiGeometry> {
    Some(match r.u8()? {
        0 => UiGeometry::Rect {
            radii: rf4(r)?,
            corner: match r.u8()? {
                0 => UiCorner::Round,
                1 => UiCorner::Squircle(r.f32()?),
                2 => UiCorner::Bevel,
                3 => UiCorner::Notch,
                _ => return None,
            },
        },
        1 => UiGeometry::Ellipse,
        2 => UiGeometry::Arc {
            start: r.f32()?,
            sweep: r.f32()?,
            thickness: r.f32()?,
            cap: rcap(r)?,
        },
        3 => UiGeometry::Polygon {
            sides: r.u32()?,
            inner: r.f32()?,
            rounding: r.f32()?,
            rotation: r.f32()?,
        },
        4 => {
            let n = r.u32()? as usize;
            let mut points = Vec::with_capacity(n.min(4096));
            for _ in 0..n {
                points.push(rv2(r)?);
            }
            let closed = r.u8()?;
            UiGeometry::Polyline {
                points,
                closed: closed != 0,
                filled: closed == 2,
                width: r.f32()?,
                cap: rcap(r)?,
                join: match r.u8()? {
                    0 => UiJoin::Miter(r.f32()?),
                    1 => UiJoin::Round,
                    2 => UiJoin::Bevel,
                    _ => return None,
                },
            }
        }
        _ => return None,
    })
}

fn gradient(w: &mut W, g: &UiGradient) {
    w.u8(match g.kind {
        UiGradientKind::Linear => 0,
        UiGradientKind::Radial => 1,
        UiGradientKind::Conic => 2,
        UiGradientKind::Diamond => 3,
    });
    v2(w, g.from);
    v2(w, g.to);
    w.u32(g.stops.len() as u32);
    for s in &g.stops {
        w.f32(s.offset);
        f4(w, &s.color);
    }
    w.u8(match g.spread {
        UiSpread::Pad => 0,
        UiSpread::Repeat => 1,
        UiSpread::Reflect => 2,
    });
    w.u8(u8::from(g.dither));
}

fn rgradient(r: &mut R) -> Option<UiGradient> {
    let kind = match r.u8()? {
        0 => UiGradientKind::Linear,
        1 => UiGradientKind::Radial,
        2 => UiGradientKind::Conic,
        3 => UiGradientKind::Diamond,
        _ => return None,
    };
    let from = rv2(r)?;
    let to = rv2(r)?;
    let n = r.u32()? as usize;
    let mut stops = Vec::with_capacity(n.min(og_render::UI_GRADIENT_STOPS));
    for _ in 0..n {
        stops.push(UiStop {
            offset: r.f32()?,
            color: rf4(r)?,
        });
    }
    let spread = match r.u8()? {
        0 => UiSpread::Pad,
        1 => UiSpread::Repeat,
        2 => UiSpread::Reflect,
        _ => return None,
    };
    Some(UiGradient {
        kind,
        from,
        to,
        stops,
        spread,
        dither: r.u8()? != 0,
    })
}

fn edge(e: UiEdge) -> u8 {
    match e {
        UiEdge::Stretch => 0,
        UiEdge::Repeat => 1,
        UiEdge::Round => 2,
    }
}

fn paint(w: &mut W, p: &UiPaint) {
    match p {
        UiPaint::Solid(c) => {
            w.u8(0);
            f4(w, c);
        }
        UiPaint::Gradient(g) => {
            w.u8(1);
            gradient(w, g);
        }
        UiPaint::Image(im) => {
            w.u8(2);
            w.u32(im.texture.slot());
            f4(w, &im.uv);
            v2(w, im.size);
            f4(w, &im.tint);
            match im.fit {
                UiFit::Fill => w.u8(0),
                UiFit::Cover => w.u8(1),
                UiFit::Contain => w.u8(2),
                UiFit::None => w.u8(3),
                UiFit::NineSlice {
                    source,
                    dest,
                    edges,
                    center,
                } => {
                    w.u8(4);
                    f4(w, &source);
                    f4(w, &dest);
                    w.u8(edge(edges));
                    w.u8(u8::from(center));
                }
                UiFit::Pattern {
                    tile,
                    rotation,
                    offset,
                    scroll,
                } => {
                    w.u8(5);
                    v2(w, tile);
                    w.f32(rotation);
                    v2(w, offset);
                    v2(w, scroll);
                }
            }
            v2(w, im.focal);
        }
        UiPaint::Layer(k) => {
            w.u8(3);
            w.u32(*k);
        }
    }
}

fn rpaint(r: &mut R) -> Option<UiPaint> {
    Some(match r.u8()? {
        0 => UiPaint::Solid(rf4(r)?),
        1 => UiPaint::Gradient(rgradient(r)?),
        2 => {
            let texture = TextureHandle::from_slot(r.u32()?)?;
            let uv = rf4(r)?;
            let size = rv2(r)?;
            let tint = rf4(r)?;
            let fit = match r.u8()? {
                0 => UiFit::Fill,
                1 => UiFit::Cover,
                2 => UiFit::Contain,
                3 => UiFit::None,
                4 => UiFit::NineSlice {
                    source: rf4(r)?,
                    dest: rf4(r)?,
                    edges: match r.u8()? {
                        0 => UiEdge::Stretch,
                        1 => UiEdge::Repeat,
                        2 => UiEdge::Round,
                        _ => return None,
                    },
                    center: r.u8()? != 0,
                },
                5 => UiFit::Pattern {
                    tile: rv2(r)?,
                    rotation: r.f32()?,
                    offset: rv2(r)?,
                    scroll: rv2(r)?,
                },
                _ => return None,
            };
            UiPaint::Image(UiImage {
                texture,
                uv,
                size,
                tint,
                fit,
                focal: rv2(r)?,
            })
        }
        3 => UiPaint::Layer(r.u32()?),
        _ => return None,
    })
}

fn mode(w: &mut W, m: &UiMode) {
    match m {
        UiMode::Fill => w.u8(0),
        UiMode::Stroke(s) => {
            w.u8(1);
            w.f32(s.width);
            w.u8(align(s.align));
            w.f32(s.dash[0]);
            w.f32(s.dash[1]);
            w.f32(s.dash_offset);
            w.u8(cap(s.dash_cap));
        }
        UiMode::Shadow {
            offset,
            blur,
            spread,
        } => {
            w.u8(2);
            v2(w, *offset);
            w.f32(*blur);
            w.f32(*spread);
        }
        UiMode::InnerShadow {
            offset,
            blur,
            spread,
        } => {
            w.u8(3);
            v2(w, *offset);
            w.f32(*blur);
            w.f32(*spread);
        }
        UiMode::Backdrop {
            blur,
            saturation,
            brightness,
            glass,
        } => {
            // Tag 4 is the plain backdrop as it always was; 5 adds the
            // material.
            let plain = *glass == og_render::UiGlass::default();
            w.u8(if plain { 4 } else { 5 });
            w.f32(*blur);
            w.f32(*saturation);
            w.f32(*brightness);
            if !plain {
                w.f32(glass.edge);
                f4(w, &glass.edge_color);
                for sh in &glass.shadows {
                    v2(w, sh.offset);
                    w.f32(sh.blur);
                    w.f32(sh.spread);
                    f4(w, &sh.color);
                }
                glass_flags(w, glass);
            }
        }
    }
}

fn rmode(r: &mut R) -> Option<UiMode> {
    Some(match r.u8()? {
        0 => UiMode::Fill,
        1 => UiMode::Stroke(UiStroke {
            width: r.f32()?,
            align: ralign(r)?,
            dash: [r.f32()?, r.f32()?],
            dash_offset: r.f32()?,
            dash_cap: rcap(r)?,
        }),
        2 => UiMode::Shadow {
            offset: rv2(r)?,
            blur: r.f32()?,
            spread: r.f32()?,
        },
        3 => UiMode::InnerShadow {
            offset: rv2(r)?,
            blur: r.f32()?,
            spread: r.f32()?,
        },
        4 => UiMode::Backdrop {
            blur: r.f32()?,
            saturation: r.f32()?,
            brightness: r.f32()?,
            glass: og_render::UiGlass::default(),
        },
        5 => {
            let (blur, saturation, brightness) = (r.f32()?, r.f32()?, r.f32()?);
            let edge = r.f32()?;
            let edge_color = rf4(r)?;
            let mut shadows = [og_render::UiGlassShadow::default(); 2];
            for sh in &mut shadows {
                *sh = og_render::UiGlassShadow {
                    offset: rv2(r)?,
                    blur: r.f32()?,
                    spread: r.f32()?,
                    color: rf4(r)?,
                };
            }
            let mut glass = og_render::UiGlass {
                edge,
                edge_color,
                shadows,
                ..Default::default()
            };
            rglass_flags(r, &mut glass)?;
            UiMode::Backdrop {
                blur,
                saturation,
                brightness,
                glass,
            }
        }
        _ => return None,
    })
}

fn shape(w: &mut W, s: &UiShape) {
    geometry(w, &s.geometry);
    v2(w, s.center);
    v2(w, s.half);
    f4(w, &s.transform);
    paint(w, &s.paint);
    mode(w, &s.mode);
    w.f32(s.opacity);
    w.u8(index_of(&BLENDS, &s.blend));
}

fn rshape(r: &mut R) -> Option<UiShape> {
    Some(UiShape {
        geometry: rgeometry(r)?,
        center: rv2(r)?,
        half: rv2(r)?,
        transform: rf4(r)?,
        paint: rpaint(r)?,
        mode: rmode(r)?,
        opacity: r.f32()?,
        blend: *BLENDS.get(r.u8()? as usize)?,
        layer: 0,
        clip: None,
    })
}

fn effects(w: &mut W, fx: &UiTextEffects) {
    w.u32(fx.shadows.len() as u32);
    for s in &fx.shadows {
        v2(w, s.offset);
        w.f32(s.blur);
        f4(w, &s.color);
    }
    match &fx.outline {
        None => w.u8(0),
        Some(o) => {
            w.u8(1);
            w.f32(o.width);
            f4(w, &o.color);
            w.u8(align(o.align));
        }
    }
    match &fx.fill {
        None => w.u8(0),
        Some(g) => {
            w.u8(1);
            gradient(w, g);
        }
    }
    f4(w, &fx.fill_box);
    f4(w, &fx.transform);
}

fn reffects(r: &mut R) -> Option<UiTextEffects> {
    let n = r.u32()? as usize;
    let mut shadows = Vec::with_capacity(n.min(64));
    for _ in 0..n {
        shadows.push(UiTextShadow {
            offset: rv2(r)?,
            blur: r.f32()?,
            color: rf4(r)?,
        });
    }
    let outline = match r.u8()? {
        0 => None,
        _ => Some(UiTextOutline {
            width: r.f32()?,
            color: rf4(r)?,
            align: ralign(r)?,
        }),
    };
    let fill = match r.u8()? {
        0 => None,
        _ => Some(rgradient(r)?),
    };
    Some(UiTextEffects {
        text: 0,
        shadows,
        outline,
        fill,
        fill_box: rf4(r)?,
        transform: rf4(r)?,
    })
}

/// A glass material's flags (solid 1, sRGB mix 2) and, when it limits its
/// backdrop or lights its top edge, flag 4 and those two values: a save
/// from before them reads unchanged.
pub(crate) fn glass_flags(w: &mut W, g: &og_render::UiGlass) {
    let lit = g.backdrop_max != 0.0 || g.edge_top != 0.0;
    w.u8(u8::from(g.solid) | u8::from(g.srgb_mix) << 1 | u8::from(lit) << 2);
    if lit {
        w.f32(g.backdrop_max);
        w.f32(g.edge_top);
    }
}

pub(crate) fn rglass_flags(r: &mut R, g: &mut og_render::UiGlass) -> Option<()> {
    let flags = r.u8()?;
    g.solid = flags & 1 != 0;
    g.srgb_mix = flags & 2 != 0;
    if flags & 4 != 0 {
        g.backdrop_max = r.f32()?;
        g.edge_top = r.f32()?;
    }
    Some(())
}

fn material(w: &mut W, m: &UiMaterial) {
    match m {
        UiMaterial::Plain => w.u8(0),
        UiMaterial::Builtin(b) => {
            w.u8(1);
            w.u8(index_of(&BUILTINS, b));
        }
        UiMaterial::Custom(h) => {
            w.u8(2);
            w.u32(h.0);
        }
    }
}

fn rmaterial(r: &mut R) -> Option<UiMaterial> {
    Some(match r.u8()? {
        0 => UiMaterial::Plain,
        1 => UiMaterial::Builtin(*BUILTINS.get(r.u8()? as usize)?),
        2 => UiMaterial::Custom(UiMaterialHandle(r.u32()?)),
        _ => return None,
    })
}

impl W<'_> {
    /// A UI pass command (one of [`DrawCmd`]'s since the old six); false
    /// for one of those.
    pub(crate) fn ui_cmd(&mut self, c: &DrawCmd) -> bool {
        let w = self;
        match c {
            DrawCmd::StyledText { text, effects: fx } => {
                w.u8(0);
                w.cmd(text);
                effects(w, fx);
            }
            DrawCmd::Shape { shape: s, clip } => {
                w.u8(1);
                shape(w, s);
                w.rect(clip);
            }
            DrawCmd::Vector {
                vector,
                rect,
                transform,
                tint,
                clip,
            } => {
                w.u8(2);
                w.u32(vector.0);
                w.rect(rect);
                f4(w, transform);
                f4(w, tint);
                w.rect(clip);
            }
            DrawCmd::PushClip {
                geometry: g,
                center,
                half,
            } => {
                w.u8(3);
                geometry(w, g);
                v2(w, Vec2::new(center.0, center.1));
                v2(w, Vec2::new(half.0, half.1));
            }
            DrawCmd::PopClip => w.u8(4),
            DrawCmd::BeginLayer {
                material: m,
                params,
                mask,
                expand,
                rect,
            } => {
                w.u8(5);
                material(w, m);
                for p in params {
                    f4(w, p);
                }
                match mask {
                    None => w.u8(0),
                    Some(UiMask::Gradient(g)) => {
                        w.u8(1);
                        gradient(w, g);
                    }
                    Some(UiMask::Image { texture, uv }) => {
                        w.u8(2);
                        w.u32(texture.slot());
                        f4(w, uv);
                    }
                }
                w.f32(*expand);
                w.rect(rect);
            }
            DrawCmd::EndLayer {
                geometry: g,
                rect,
                opacity,
                blend,
                clip,
            } => {
                w.u8(6);
                match g {
                    None => w.u8(0),
                    Some(g) => {
                        w.u8(1);
                        geometry(w, g);
                    }
                }
                w.rect(rect);
                w.f32(*opacity);
                w.u8(index_of(&BLENDS, blend));
                w.rect(clip);
            }
            _ => return false,
        }
        true
    }
}

impl R<'_> {
    /// [`W::ui_cmd`]'s command.
    pub(crate) fn ui_cmd(&mut self) -> Option<DrawCmd> {
        let r = self;
        Some(match r.u8()? {
            0 => {
                let text = r.cmd()?;
                DrawCmd::StyledText {
                    text: Box::new(text),
                    effects: Box::new(reffects(r)?),
                }
            }
            1 => DrawCmd::Shape {
                shape: Box::new(rshape(r)?),
                clip: r.rect()?,
            },
            2 => DrawCmd::Vector {
                vector: UiVectorHandle(r.u32()?),
                rect: r.rect()?,
                transform: rf4(r)?,
                tint: rf4(r)?,
                clip: r.rect()?,
            },
            3 => {
                let geometry = rgeometry(r)?;
                let c = rv2(r)?;
                let h = rv2(r)?;
                DrawCmd::PushClip {
                    geometry,
                    center: (c.x, c.y),
                    half: (h.x, h.y),
                }
            }
            4 => DrawCmd::PopClip,
            5 => {
                let material = rmaterial(r)?;
                let mut params = [[0.0; 4]; og_render::UI_MATERIAL_PARAMS];
                for p in &mut params {
                    *p = rf4(r)?;
                }
                let mask = match r.u8()? {
                    0 => None,
                    1 => Some(UiMask::Gradient(rgradient(r)?)),
                    2 => Some(UiMask::Image {
                        texture: TextureHandle::from_slot(r.u32()?)?,
                        uv: rf4(r)?,
                    }),
                    _ => return None,
                };
                DrawCmd::BeginLayer {
                    material,
                    params,
                    mask,
                    expand: r.f32()?,
                    rect: r.rect()?,
                }
            }
            6 => {
                let geometry = match r.u8()? {
                    0 => None,
                    _ => Some(rgeometry(r)?),
                };
                DrawCmd::EndLayer {
                    geometry,
                    rect: r.rect()?,
                    opacity: r.f32()?,
                    blend: *BLENDS.get(r.u8()? as usize)?,
                    clip: r.rect()?,
                }
            }
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::draw::DrawCmd;
    use crate::layout::Rect;
    use crate::motion::codec::{R, W};
    use og_core::glam::Vec2;
    use og_render::{UiGlass, UiGlassShadow, UiMode, UiShape};

    /// A leaving element's glass keeps its material through a saved state
    /// (the primitive and a UI shape's backdrop), and a plain one is the
    /// bytes it always was.
    #[test]
    fn glass_material_survives_a_saved_state() {
        let material = UiGlass {
            edge: 0.5,
            edge_color: [1.0, 1.0, 1.0, 0.08],
            shadows: [
                UiGlassShadow {
                    offset: Vec2::new(0.0, 8.0),
                    blur: 24.0,
                    spread: -8.0,
                    color: [0.0, 0.0, 0.0, 0.6],
                },
                UiGlassShadow::default(),
            ],
            solid: false,
            srgb_mix: true,
            backdrop_max: 0.42,
            edge_top: 0.22,
        };
        let rect = Rect::new(10.0, 20.0, 200.0, 100.0);
        let glass = |saturation, material| DrawCmd::Glass {
            rect,
            radius: 12.0,
            blur: 44.0,
            saturation,
            tint: [0.1, 0.1, 0.12, 0.78],
            material,
            alpha: 1.0,
            clip: rect,
        };
        let mut shape = UiShape::rect([0.0, 0.0, 50.0, 40.0], 8.0, [0.0, 0.0, 0.0, 0.3]);
        shape.mode = UiMode::Backdrop {
            blur: 20.0,
            saturation: 2.0,
            brightness: 1.0,
            glass: UiGlass {
                solid: true,
                ..material
            },
        };
        let cmds = [
            glass(2.0, material),
            glass(1.0, UiGlass::default()),
            DrawCmd::Shape {
                shape: Box::new(shape),
                clip: rect,
            },
        ];
        let mut bytes = Vec::new();
        for c in &cmds {
            W(&mut bytes).cmd(c);
        }
        let mut r = R {
            bytes: &bytes,
            at: 0,
            version: crate::ui::STATE_VERSION,
        };
        for c in &cmds {
            assert_eq!(r.cmd().as_ref(), Some(c));
        }
        assert_eq!(r.at, bytes.len());
        // The plain glass is tag 0 and its six fields, as before.
        let mut plain = Vec::new();
        W(&mut plain).cmd(&cmds[1]);
        assert_eq!(
            (plain[0], plain.len()),
            (0, 1 + 4 * (4 + 1 + 1 + 4 + 1 + 4))
        );
    }

    #[test]
    fn a_filled_path_keeps_its_area_through_the_codec() {
        for (closed, filled) in [(false, false), (true, false), (true, true)] {
            let g = UiGeometry::Polyline {
                points: vec![
                    Vec2::new(0.0, 0.0),
                    Vec2::new(8.0, 0.0),
                    Vec2::new(4.0, 6.0),
                ],
                closed,
                filled,
                width: 2.0,
                cap: UiCap::Round,
                join: UiJoin::Round,
            };
            let mut bytes = Vec::new();
            geometry(&mut W(&mut bytes), &g);
            let back = rgeometry(&mut R {
                bytes: &bytes,
                at: 0,
                version: crate::ui::STATE_VERSION,
            });
            assert_eq!(back, Some(g));
        }
    }
}
