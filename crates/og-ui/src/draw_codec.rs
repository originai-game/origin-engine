//! Bytes of the UI pass's draw commands ([`DrawCmd::Shape`], text effects,
//! vectors, clip and layer brackets) for the motion state's frozen drawing
//! of leaving elements ([`crate::motion::codec`], its tag 6): a restored
//! state draws a leaving element's paints, effects, masks and layers as it
//! left.

use crate::draw::DrawCmd;
use crate::motion::codec::{R, W};
use og_core::glam::Vec2;
use og_draw::{
    TextureHandle, UiAlign, UiBackgroundSize, UiBlend, UiBuiltin, UiCap, UiCorner, UiEdge,
    UiFillRule, UiFilter, UiFilterComposite, UiFilterInput, UiFilterOp, UiFilterPrimitive, UiFit,
    UiGeometry, UiGradient, UiGradientKind, UiImage, UiJoin, UiLight, UiMask, UiMaterial,
    UiMaterialHandle, UiMode, UiPaint, UiPath, UiRepeat, UiShape, UiSpread, UiStop, UiStroke,
    UiTextEffects, UiTextOutline, UiTextShadow, UiTransfer, UiVectorHandle, UiVerb,
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

fn input(w: &mut W, i: UiFilterInput) {
    w.u32(match i {
        UiFilterInput::Source => 0,
        UiFilterInput::SourceAlpha => 1,
        UiFilterInput::Result(k) => k + 2,
    });
}
fn rinput(r: &mut R) -> Option<UiFilterInput> {
    Some(match r.u32()? {
        0 => UiFilterInput::Source,
        1 => UiFilterInput::SourceAlpha,
        k => UiFilterInput::Result(k - 2),
    })
}
fn floats(w: &mut W, v: &[f32]) {
    w.u32(v.len() as u32);
    for x in v {
        w.f32(*x);
    }
}
fn rfloats(r: &mut R) -> Option<Vec<f32>> {
    let n = r.u32()? as usize;
    if n > 4096 {
        return None;
    }
    (0..n).map(|_| r.f32()).collect()
}
fn v3(w: &mut W, v: og_core::glam::Vec3) {
    w.f32(v.x);
    w.f32(v.y);
    w.f32(v.z);
}
fn rv3(r: &mut R) -> Option<og_core::glam::Vec3> {
    Some(og_core::glam::Vec3::new(r.f32()?, r.f32()?, r.f32()?))
}

/// A `kernelUnitLength`, or none.
fn unit(w: &mut W, u: &Option<[f32; 2]>) {
    floats(w, u.as_ref().map_or(&[][..], |u| &u[..]));
}
fn runit(r: &mut R) -> Option<Option<[f32; 2]>> {
    let v = rfloats(r)?;
    Some(v.try_into().ok())
}

/// A layer's filter region and graph.
fn filter(w: &mut W, (region, f): &(crate::layout::Rect, UiFilter)) {
    w.rect(region);
    floats(w, &f.space);
    w.u32(f.primitives.len() as u32);
    for p in &f.primitives {
        input(w, p.inputs[0]);
        input(w, p.inputs[1]);
        f4(w, &p.subregion);
        w.u8(u8::from(p.srgb));
        match &p.op {
            UiFilterOp::Blur { std_dev } => {
                w.u8(0);
                floats(w, std_dev);
            }
            UiFilterOp::Offset { d } => {
                w.u8(1);
                floats(w, d);
            }
            UiFilterOp::Flood { color } => {
                w.u8(2);
                f4(w, color);
            }
            UiFilterOp::Composite { op, k } => {
                w.u8(3);
                w.u8(*op as u8);
                f4(w, k);
            }
            UiFilterOp::Blend(b) => {
                w.u8(4);
                w.u8(index_of(&BLENDS, b));
            }
            UiFilterOp::ColorMatrix(m) => {
                w.u8(5);
                floats(w, m);
            }
            UiFilterOp::ComponentTransfer(funcs) => {
                w.u8(6);
                for t in funcs.iter() {
                    match t {
                        UiTransfer::Identity => w.u8(0),
                        UiTransfer::Table(v) => {
                            w.u8(1);
                            floats(w, v);
                        }
                        UiTransfer::Discrete(v) => {
                            w.u8(2);
                            floats(w, v);
                        }
                        UiTransfer::Linear { slope, intercept } => {
                            w.u8(3);
                            floats(w, &[*slope, *intercept]);
                        }
                        UiTransfer::Gamma {
                            amplitude,
                            exponent,
                            offset,
                        } => {
                            w.u8(4);
                            floats(w, &[*amplitude, *exponent, *offset]);
                        }
                    }
                }
            }
            UiFilterOp::Morphology { radius, dilate } => {
                w.u8(7);
                floats(w, radius);
                w.u8(u8::from(*dilate));
            }
            UiFilterOp::Displacement { scale, channels } => {
                w.u8(8);
                w.f32(*scale);
                w.u8(channels[0]);
                w.u8(channels[1]);
            }
            UiFilterOp::Turbulence {
                base_frequency,
                octaves,
                seed,
                stitch,
                fractal,
                user,
                tile,
            } => {
                w.u8(9);
                floats(w, base_frequency);
                w.u32(*octaves);
                w.f32(*seed);
                w.u8(u8::from(*stitch) | u8::from(*fractal) << 1);
                floats(w, user);
                f4(w, tile);
            }
            UiFilterOp::Image { texture, uv, rect } => {
                w.u8(10);
                w.u32(texture.slot());
                f4(w, uv);
                f4(w, rect);
            }
            UiFilterOp::Tile { tile } => {
                w.u8(11);
                f4(w, tile);
            }
            UiFilterOp::Layer { layer } => {
                w.u8(15);
                w.u32(*layer);
            }
            UiFilterOp::Lighting {
                specular,
                surface_scale,
                constant,
                exponent,
                color,
                light,
                kernel_unit,
            } => {
                w.u8(12);
                unit(w, kernel_unit);
                w.u8(u8::from(*specular));
                floats(w, &[*surface_scale, *constant, *exponent]);
                floats(w, color);
                match light {
                    UiLight::Distant { azimuth, elevation } => {
                        w.u8(0);
                        floats(w, &[*azimuth, *elevation]);
                    }
                    UiLight::Point { at } => {
                        w.u8(1);
                        v3(w, *at);
                    }
                    UiLight::Spot {
                        at,
                        points_at,
                        exponent,
                        cone,
                    } => {
                        w.u8(2);
                        v3(w, *at);
                        v3(w, *points_at);
                        w.f32(*exponent);
                        w.f32(cone.unwrap_or(f32::NAN));
                    }
                }
            }
            UiFilterOp::Merge => w.u8(13),
            UiFilterOp::Convolve {
                order,
                kernel,
                divisor,
                bias,
                target,
                edge,
                preserve_alpha,
                kernel_unit,
            } => {
                w.u8(14);
                unit(w, kernel_unit);
                w.u32(order[0]);
                w.u32(order[1]);
                floats(w, kernel);
                floats(w, &[*divisor, *bias]);
                w.i32(target[0]);
                w.i32(target[1]);
                w.u8(*edge);
                w.u8(u8::from(*preserve_alpha));
            }
        }
    }
}

fn rfilter(r: &mut R) -> Option<(crate::layout::Rect, UiFilter)> {
    let region = r.rect()?;
    let space: [f32; 6] = rfloats(r)?.try_into().ok()?;
    let n = r.u32()? as usize;
    if n > 4096 {
        return None;
    }
    let two = |r: &mut R| -> Option<[f32; 2]> { rfloats(r)?.try_into().ok() };
    let mut primitives = Vec::with_capacity(n);
    for _ in 0..n {
        let inputs = [rinput(r)?, rinput(r)?];
        let subregion = rf4(r)?;
        let srgb = r.u8()? != 0;
        let op = match r.u8()? {
            0 => UiFilterOp::Blur { std_dev: two(r)? },
            1 => UiFilterOp::Offset { d: two(r)? },
            2 => UiFilterOp::Flood { color: rf4(r)? },
            3 => UiFilterOp::Composite {
                op: match r.u8()? {
                    0 => UiFilterComposite::Over,
                    1 => UiFilterComposite::In,
                    2 => UiFilterComposite::Out,
                    3 => UiFilterComposite::Atop,
                    4 => UiFilterComposite::Xor,
                    5 => UiFilterComposite::Lighter,
                    6 => UiFilterComposite::Arithmetic,
                    _ => return None,
                },
                k: rf4(r)?,
            },
            4 => UiFilterOp::Blend(*BLENDS.get(r.u8()? as usize)?),
            5 => UiFilterOp::ColorMatrix(rfloats(r)?.try_into().ok()?),
            6 => {
                let mut funcs: [UiTransfer; 4] = Default::default();
                for t in &mut funcs {
                    *t = match r.u8()? {
                        0 => UiTransfer::Identity,
                        1 => UiTransfer::Table(rfloats(r)?),
                        2 => UiTransfer::Discrete(rfloats(r)?),
                        3 => {
                            let [slope, intercept] = rfloats(r)?.try_into().ok()?;
                            UiTransfer::Linear { slope, intercept }
                        }
                        4 => {
                            let [amplitude, exponent, offset] = rfloats(r)?.try_into().ok()?;
                            UiTransfer::Gamma {
                                amplitude,
                                exponent,
                                offset,
                            }
                        }
                        _ => return None,
                    };
                }
                UiFilterOp::ComponentTransfer(Box::new(funcs))
            }
            7 => UiFilterOp::Morphology {
                radius: two(r)?,
                dilate: r.u8()? != 0,
            },
            8 => UiFilterOp::Displacement {
                scale: r.f32()?,
                channels: [r.u8()?, r.u8()?],
            },
            9 => {
                let base_frequency = two(r)?;
                let octaves = r.u32()?;
                let seed = r.f32()?;
                let flags = r.u8()?;
                UiFilterOp::Turbulence {
                    base_frequency,
                    octaves,
                    seed,
                    stitch: flags & 1 != 0,
                    fractal: flags & 2 != 0,
                    user: rfloats(r)?.try_into().ok()?,
                    tile: rf4(r)?,
                }
            }
            10 => UiFilterOp::Image {
                texture: TextureHandle::from_slot(r.u32()?)?,
                uv: rf4(r)?,
                rect: rf4(r)?,
            },
            11 => UiFilterOp::Tile { tile: rf4(r)? },
            12 => {
                let kernel_unit = runit(r)?;
                let specular = r.u8()? != 0;
                let [surface_scale, constant, exponent] = rfloats(r)?.try_into().ok()?;
                let color = rfloats(r)?.try_into().ok()?;
                let light = match r.u8()? {
                    0 => {
                        let [azimuth, elevation] = two(r)?;
                        UiLight::Distant { azimuth, elevation }
                    }
                    1 => UiLight::Point { at: rv3(r)? },
                    2 => UiLight::Spot {
                        at: rv3(r)?,
                        points_at: rv3(r)?,
                        exponent: r.f32()?,
                        cone: Some(r.f32()?).filter(|c| !c.is_nan()),
                    },
                    _ => return None,
                };
                UiFilterOp::Lighting {
                    specular,
                    surface_scale,
                    constant,
                    exponent,
                    color,
                    light,
                    kernel_unit,
                }
            }
            13 => UiFilterOp::Merge,
            14 => {
                let kernel_unit = runit(r)?;
                let order = [r.u32()?, r.u32()?];
                let kernel = rfloats(r)?;
                let [divisor, bias] = two(r)?;
                UiFilterOp::Convolve {
                    order,
                    kernel,
                    divisor,
                    bias,
                    target: [r.i32()?, r.i32()?],
                    edge: r.u8()?,
                    preserve_alpha: r.u8()? != 0,
                    kernel_unit,
                }
            }
            15 => UiFilterOp::Layer { layer: r.u32()? },
            _ => return None,
        };
        primitives.push(UiFilterPrimitive {
            op,
            inputs,
            subregion,
            srgb,
        });
    }
    Some((region, UiFilter { primitives, space }))
}

const BLENDS: [UiBlend; 21] = [
    UiBlend::Normal,
    UiBlend::Multiply,
    UiBlend::Screen,
    UiBlend::Add,
    UiBlend::Overlay,
    UiBlend::SoftLight,
    UiBlend::ColorDodge,
    UiBlend::Darken,
    UiBlend::Lighten,
    UiBlend::ColorBurn,
    UiBlend::HardLight,
    UiBlend::Difference,
    UiBlend::Exclusion,
    UiBlend::Hue,
    UiBlend::Saturation,
    UiBlend::Color,
    UiBlend::Luminosity,
    UiBlend::SourceIn,
    UiBlend::SourceOut,
    UiBlend::DestinationIn,
    UiBlend::Xor,
];
const REPEATS: [UiRepeat; 4] = [
    UiRepeat::Repeat,
    UiRepeat::NoRepeat,
    UiRepeat::Space,
    UiRepeat::Round,
];
const BUILTINS: [UiBuiltin; 12] = [
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
    UiBuiltin::ColorMatrix,
    UiBuiltin::DropShadow,
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
                UiCorner::Css(c) => {
                    w.u8(4);
                    f4(w, &c.vertical);
                    f4(w, &c.shape);
                }
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
        UiGeometry::Path(path) => {
            w.u8(5);
            w.u8(u8::from(path.fill_rule == UiFillRule::EvenOdd));
            w.u32(path.verbs.len() as u32);
            for v in &path.verbs {
                w.u8(index_of(&VERBS, v));
            }
            w.u32(path.points.len() as u32);
            for p in &path.points {
                v2(w, *p);
            }
        }
    }
}

const VERBS: [UiVerb; 5] = [
    UiVerb::Move,
    UiVerb::Line,
    UiVerb::Quad,
    UiVerb::Cubic,
    UiVerb::Close,
];

fn rgeometry(r: &mut R) -> Option<UiGeometry> {
    Some(match r.u8()? {
        0 => UiGeometry::Rect {
            radii: rf4(r)?,
            corner: match r.u8()? {
                0 => UiCorner::Round,
                1 => UiCorner::Squircle(r.f32()?),
                2 => UiCorner::Bevel,
                3 => UiCorner::Notch,
                4 => UiCorner::Css(og_draw::UiCssCorners {
                    vertical: rf4(r)?,
                    shape: rf4(r)?,
                }),
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
        5 => {
            let mut path = UiPath {
                fill_rule: if r.u8()? == 1 {
                    UiFillRule::EvenOdd
                } else {
                    UiFillRule::NonZero
                },
                ..UiPath::default()
            };
            let n = r.u32()? as usize;
            path.verbs.reserve(n.min(4096));
            for _ in 0..n {
                path.verbs.push(*VERBS.get(r.u8()? as usize)?);
            }
            let n = r.u32()? as usize;
            path.points.reserve(n.min(4096));
            for _ in 0..n {
                path.points.push(rv2(r)?);
            }
            path.check().ok()?;
            UiGeometry::Path(std::sync::Arc::new(path))
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
        UiSpread::RepeatStops => 3,
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
    let mut stops = Vec::with_capacity(n.min(og_draw::UI_GRADIENT_STOPS));
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
        3 => UiSpread::RepeatStops,
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
        UiEdge::Space => 3,
    }
}

fn redge(r: &mut R) -> Option<UiEdge> {
    Some(match r.u8()? {
        0 => UiEdge::Stretch,
        1 => UiEdge::Repeat,
        2 => UiEdge::Round,
        3 => UiEdge::Space,
        _ => return None,
    })
}

fn fit(w: &mut W, f: &UiFit) {
    match *f {
        UiFit::Fill => w.u8(0),
        UiFit::Cover => w.u8(1),
        UiFit::Contain => w.u8(2),
        UiFit::None => w.u8(3),
        UiFit::ScaleDown => w.u8(9),
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
        UiFit::Background(b) => {
            // The positioning area, when it is not the box, after a tag
            // of its own (6 stays what it was).
            w.u8(if b.area == [0.0; 4] { 6 } else { 8 });
            match b.size {
                UiBackgroundSize::Cover => w.u8(1),
                UiBackgroundSize::Contain => w.u8(2),
                UiBackgroundSize::Given(g) => {
                    w.u8(0);
                    for v in g {
                        w.f32(v.unwrap_or(-1.0));
                    }
                }
            }
            v2(w, b.align);
            v2(w, b.offset);
            for r in b.repeat {
                w.u8(index_of(&REPEATS, &r));
            }
            if b.area != [0.0; 4] {
                f4(w, &b.area);
            }
        }
        UiFit::BorderImage(b) => {
            w.u8(7);
            f4(w, &b.slice);
            f4(w, &b.width);
            w.u8(edge(b.repeat[0]));
            w.u8(edge(b.repeat[1]));
            w.u8(u8::from(b.fill));
        }
    }
}

fn rfit(r: &mut R) -> Option<UiFit> {
    let tag = r.u8()?;
    Some(match tag {
        0 => UiFit::Fill,
        1 => UiFit::Cover,
        2 => UiFit::Contain,
        3 => UiFit::None,
        9 => UiFit::ScaleDown,
        4 => UiFit::NineSlice {
            source: rf4(r)?,
            dest: rf4(r)?,
            edges: redge(r)?,
            center: r.u8()? != 0,
        },
        5 => UiFit::Pattern {
            tile: rv2(r)?,
            rotation: r.f32()?,
            offset: rv2(r)?,
            scroll: rv2(r)?,
        },
        6 | 8 => UiFit::Background(og_draw::UiBackground {
            size: match r.u8()? {
                0 => UiBackgroundSize::Given([r.f32()?, r.f32()?].map(|v| (v >= 0.0).then_some(v))),
                1 => UiBackgroundSize::Cover,
                2 => UiBackgroundSize::Contain,
                _ => return None,
            },
            align: rv2(r)?,
            offset: rv2(r)?,
            repeat: [
                *REPEATS.get(r.u8()? as usize)?,
                *REPEATS.get(r.u8()? as usize)?,
            ],
            area: if tag == 8 { rf4(r)? } else { [0.0; 4] },
        }),
        7 => UiFit::BorderImage(og_draw::UiBorderImage {
            slice: rf4(r)?,
            width: rf4(r)?,
            repeat: [redge(r)?, redge(r)?],
            fill: r.u8()? != 0,
        }),
        _ => return None,
    })
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
            // A luminance image is its own tag (2 stays what it was); one
            // with an offset or sampled nearest is 6, with both after it.
            let density = im.density != 1.0;
            let extra = im.offset != Vec2::ZERO || im.nearest || density;
            w.u8(match (density, extra, im.luminance) {
                (true, _, _) => 7,
                (false, true, _) => 6,
                (false, false, true) => 4,
                (false, false, false) => 2,
            });
            w.u32(im.texture.slot());
            f4(w, &im.uv);
            v2(w, im.size);
            f4(w, &im.tint);
            fit(w, &im.fit);
            v2(w, im.focal);
            if extra {
                v2(w, im.offset);
                w.u8(u8::from(im.luminance) | u8::from(im.nearest) << 1);
            }
            if density {
                w.f32(im.density);
            }
        }
        UiPaint::Layer(k) => {
            w.u8(3);
            w.u32(*k);
        }
        UiPaint::GradientImage(g) => {
            w.u8(5);
            gradient(w, &g.gradient);
            v2(w, g.size);
            fit(w, &g.fit);
            w.u8(u8::from(g.luminance));
        }
    }
}

fn rpaint(r: &mut R) -> Option<UiPaint> {
    let tag = r.u8()?;
    Some(match tag {
        0 => UiPaint::Solid(rf4(r)?),
        1 => UiPaint::Gradient(rgradient(r)?),
        2 | 4 | 6 | 7 => {
            let texture = TextureHandle::from_slot(r.u32()?)?;
            let uv = rf4(r)?;
            let size = rv2(r)?;
            let tint = rf4(r)?;
            let fit = rfit(r)?;
            let focal = rv2(r)?;
            let (offset, flags) = if tag >= 6 {
                (rv2(r)?, r.u8()?)
            } else {
                (Vec2::ZERO, u8::from(tag == 4))
            };
            UiPaint::Image(UiImage {
                texture,
                uv,
                size,
                density: if tag == 7 { r.f32()? } else { 1.0 },
                tint,
                fit,
                focal,
                offset,
                luminance: flags & 1 != 0,
                nearest: flags & 2 != 0,
            })
        }
        3 => UiPaint::Layer(r.u32()?),
        5 => UiPaint::GradientImage(Box::new(og_draw::UiGradientImage {
            gradient: rgradient(r)?,
            size: rv2(r)?,
            fit: rfit(r)?,
            luminance: r.u8()? != 0,
        })),
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
            let plain = *glass == og_draw::UiGlass::default();
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
            glass: og_draw::UiGlass::default(),
        },
        5 => {
            let (blur, saturation, brightness) = (r.f32()?, r.f32()?, r.f32()?);
            let edge = r.f32()?;
            let edge_color = rf4(r)?;
            let mut shadows = [og_draw::UiGlassShadow::default(); 2];
            for sh in &mut shadows {
                *sh = og_draw::UiGlassShadow {
                    offset: rv2(r)?,
                    blur: r.f32()?,
                    spread: r.f32()?,
                    color: rf4(r)?,
                };
            }
            let mut glass = og_draw::UiGlass {
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
/// backdrop or lights its top edge, flag 4 and those two values, then with
/// a colour matrix flag 8 and its rows and opacity: a save from before
/// them reads unchanged.
pub(crate) fn glass_flags(w: &mut W, g: &og_draw::UiGlass) {
    let lit = g.backdrop_max != 0.0 || g.edge_top != 0.0;
    w.u8(u8::from(g.solid)
        | u8::from(g.srgb_mix) << 1
        | u8::from(lit) << 2
        | u8::from(g.matrix.is_some()) << 3);
    if lit {
        w.f32(g.backdrop_max);
        w.f32(g.edge_top);
    }
    if let Some(m) = &g.matrix {
        for row in &m.rows {
            f4(w, row);
        }
        w.f32(m.opacity);
    }
}

pub(crate) fn rglass_flags(r: &mut R, g: &mut og_draw::UiGlass) -> Option<()> {
    let flags = r.u8()?;
    g.solid = flags & 1 != 0;
    g.srgb_mix = flags & 2 != 0;
    if flags & 4 != 0 {
        g.backdrop_max = r.f32()?;
        g.edge_top = r.f32()?;
    }
    if flags & 8 != 0 {
        g.matrix = Some(og_draw::UiColorMatrix {
            rows: [rf4(r)?, rf4(r)?, rf4(r)?],
            opacity: r.f32()?,
        });
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
                filter: f,
                role,
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
                match f {
                    None => w.u8(0),
                    Some(f) => {
                        w.u8(1);
                        filter(w, f);
                    }
                }
                match role {
                    crate::LayerRole::Draw => w.u8(0),
                    crate::LayerRole::Backdrop => w.u8(1),
                    crate::LayerRole::Image(n) => {
                        w.u8(2);
                        w.u32(*n);
                    }
                }
            }
            // A ground's quad is the frame's, not the element's: a frozen
            // drawing keeps none (it leaves where it was laid out, unseen).
            DrawCmd::EndLayer {
                geometry: g,
                rect,
                opacity,
                blend,
                clip,
                ground: _,
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
                let mut params = [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS];
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
                    filter: match r.u8()? {
                        0 => None,
                        _ => Some(Box::new(rfilter(r)?)),
                    },
                    role: match r.u8()? {
                        0 => crate::LayerRole::Draw,
                        1 => crate::LayerRole::Backdrop,
                        2 => crate::LayerRole::Image(r.u32()?),
                        _ => return None,
                    },
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
                    ground: None,
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
    use og_draw::{UiGlass, UiGlassShadow, UiMode, UiShape};

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
            matrix: None,
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
                matrix: Some(og_draw::UiColorMatrix {
                    rows: [
                        [0.5, 0.0, 0.0, 0.25],
                        [0.0, 1.5, 0.0, 0.0],
                        [0.1, 0.2, 0.3, 0.0],
                    ],
                    opacity: 0.75,
                }),
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
            legacy_glyphs: false,
            affine_only: false,
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
    fn image_density_survives_a_saved_drawing() {
        for density in [1.0, 2.0] {
            let image = UiPaint::Image(UiImage {
                texture: TextureHandle::from_slot(1).unwrap(),
                uv: [0.25, 0.0, 0.75, 1.0],
                size: Vec2::ZERO,
                density,
                tint: [0.5, 1.0, 0.25, 0.75],
                fit: UiFit::None,
                focal: Vec2::new(0.0, 1.0),
                offset: Vec2::new(3.0, -2.0),
                luminance: true,
                nearest: true,
            });
            let mut bytes = Vec::new();
            paint(&mut W(&mut bytes), &image);
            let mut reader = R {
                bytes: &bytes,
                at: 0,
                legacy_glyphs: false,
                affine_only: false,
            };
            assert_eq!(rpaint(&mut reader), Some(image));
            assert_eq!(reader.at, bytes.len());
        }
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
                legacy_glyphs: false,
                affine_only: false,
            });
            assert_eq!(back, Some(g));
        }
    }

    /// A run-time path keeps its verbs, points and even-odd rule (a
    /// leaving chart or imported shape draws as it left); a cut-short one
    /// reads as nothing, not as a different path.
    #[test]
    fn a_vector_path_keeps_its_curves_through_the_codec() {
        let mut path = UiPath::new();
        path.move_to(Vec2::new(1.0, 2.0))
            .quad_to(Vec2::new(5.0, -3.0), Vec2::new(9.0, 2.0))
            .cubic_to(
                Vec2::new(9.0, 8.0),
                Vec2::new(1.0, 8.0),
                Vec2::new(1.0, 4.0),
            )
            .close();
        path.ellipse(Vec2::new(4.0, 4.0), Vec2::new(2.0, 1.0));
        path.fill_rule = UiFillRule::EvenOdd;
        let g = UiGeometry::Path(std::sync::Arc::new(path));
        let mut bytes = Vec::new();
        geometry(&mut W(&mut bytes), &g);
        let read = |b: &[u8]| {
            rgeometry(&mut R {
                bytes: b,
                at: 0,
                legacy_glyphs: false,
                affine_only: false,
            })
        };
        assert_eq!(read(&bytes), Some(g));
        assert_eq!(read(&bytes[..bytes.len() - 4]), None);
    }
}
