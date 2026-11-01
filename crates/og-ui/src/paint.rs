//! What an element draws with (`docs/ui.md` §5): its shape, layered fills
//! and strokes, effects, opacity and blend, how it clips its children, and
//! whether its subtree draws offscreen through a material. Every value is
//! data: colours are style tokens resolved at draw time ([`ColorRef`]) or
//! literals, so a style change restyles the paints, and an agent reads and
//! writes them as it does any prop.
//!
//! [`ElementDraw`] is carried by an element ([`crate::Ui::set_draw`]) and
//! turned into [`DrawCmd`]s around the element's own commands and its
//! children: shadows, outer glow and backdrop under the fills, inner shadow
//! and glow over them, then the children (cut to the shape when it clips),
//! then the strokes; a layer brackets all of it.

use crate::draw::DrawCmd;
use crate::layout::Rect;
use crate::style::{Srgba, Theme};
use og_core::glam::Vec2;
pub use og_render::{
    TextureHandle, UiAlign as StrokeAlign, UiBlend as Blend, UiBuiltin as Builtin, UiCap as Cap,
    UiCorner as Corner, UiEdge as Edge, UiFit as Fit, UiGradientKind as GradientKind,
    UiJoin as Join, UiMaterialHandle, UiSpread as Spread, UiVectorHandle,
};

/// A colour: a style token (a [`crate::Palette`] name, the IDL's
/// `ThemeColor`) resolved through the theme when the frame is drawn, or a
/// literal sRGB colour as authored.
#[derive(Clone, Debug, PartialEq)]
pub enum ColorRef {
    Token { name: String, alpha: f32 },
    Literal(Srgba),
}

/// What an unknown token draws: a colour no style uses, so a typo shows.
pub const MISSING_TOKEN: [f32; 4] = [1.0, 0.0, 1.0, 1.0];

impl ColorRef {
    pub fn token(name: &str) -> ColorRef {
        ColorRef::Token {
            name: name.to_string(),
            alpha: 1.0,
        }
    }

    /// Linear RGBA through `theme` (an unknown token is [`MISSING_TOKEN`]).
    pub fn resolve(&self, theme: &Theme) -> [f32; 4] {
        match self {
            ColorRef::Token { name, alpha } => theme
                .palette
                .get(name)
                .map_or(MISSING_TOKEN, |c| c.fade(alpha.clamp(0.0, 1.0)).linear()),
            ColorRef::Literal(c) => c.linear(),
        }
    }
}

impl From<Srgba> for ColorRef {
    fn from(c: Srgba) -> ColorRef {
        ColorRef::Literal(c)
    }
}

/// A gradient stop.
#[derive(Clone, Debug, PartialEq)]
pub struct Stop {
    pub offset: f32,
    pub color: ColorRef,
}

/// A gradient over the element's box: `from` and `to` are fractions of it
/// (0,0 top left); radial and diamond take `to` as the radii, conic
/// `to[0]` as the start angle (radians clockwise from +x). Up to 8 stops,
/// mixed in OKLab, dithered.
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub kind: GradientKind,
    pub from: [f32; 2],
    pub to: [f32; 2],
    pub stops: Vec<Stop>,
    pub spread: Spread,
    pub dither: bool,
}

impl Default for Gradient {
    fn default() -> Gradient {
        Gradient {
            kind: GradientKind::Linear,
            from: [0.0, 0.0],
            to: [0.0, 1.0],
            stops: Vec::new(),
            spread: Spread::Pad,
            dither: true,
        }
    }
}

impl Gradient {
    pub fn resolve(&self, theme: &Theme) -> og_render::UiGradient {
        og_render::UiGradient {
            kind: self.kind,
            from: Vec2::from(self.from),
            to: Vec2::from(self.to),
            stops: self
                .stops
                .iter()
                .map(|s| og_render::UiStop {
                    offset: s.offset,
                    color: s.color.resolve(theme),
                })
                .collect(),
            spread: self.spread,
            dither: self.dither,
        }
    }
}

/// An image paint: a region of a texture, its pixel size, a tint, how it
/// fits the box and where it is anchored ([`Fit`]: fill, cover, contain,
/// none, nine-slice, pattern).
#[derive(Clone, Debug, PartialEq)]
pub struct ImagePaint {
    pub texture: TextureHandle,
    pub uv: [f32; 4],
    pub size: [f32; 2],
    pub tint: ColorRef,
    pub fit: Fit,
    pub focal: [f32; 2],
}

/// One layer of fill or a stroke's paint.
#[derive(Clone, Debug, PartialEq)]
pub enum Paint {
    Solid(ColorRef),
    Gradient(Gradient),
    Image(ImagePaint),
    /// A cooked vector (`type vector`) contained and centred in the
    /// element's rectangle, its paints multiplied by `tint`. As a stroke's
    /// paint it is its tint.
    Vector {
        vector: UiVectorHandle,
        tint: ColorRef,
    },
}

impl Paint {
    pub fn resolve(&self, theme: &Theme) -> og_render::UiPaint {
        match self {
            Paint::Solid(c) => og_render::UiPaint::Solid(c.resolve(theme)),
            Paint::Gradient(g) => og_render::UiPaint::Gradient(g.resolve(theme)),
            Paint::Image(im) => og_render::UiPaint::Image(og_render::UiImage {
                texture: im.texture,
                uv: im.uv,
                size: Vec2::from(im.size),
                tint: im.tint.resolve(theme),
                fit: im.fit,
                focal: Vec2::from(im.focal),
            }),
            Paint::Vector { tint, .. } => og_render::UiPaint::Solid(tint.resolve(theme)),
        }
    }
}

/// A stroke along the element's outline with a paint of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    pub width: f32,
    pub align: StrokeAlign,
    /// Dash and gap, CSS px (0: solid).
    pub dash: [f32; 2],
    pub dash_offset: f32,
    pub dash_cap: Cap,
    pub paint: Paint,
}

/// An effect of the element (§5.3).
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    DropShadow {
        offset: [f32; 2],
        blur: f32,
        spread: f32,
        color: ColorRef,
    },
    InnerShadow {
        offset: [f32; 2],
        blur: f32,
        spread: f32,
        color: ColorRef,
    },
    OuterGlow {
        blur: f32,
        spread: f32,
        color: ColorRef,
    },
    InnerGlow {
        blur: f32,
        spread: f32,
        color: ColorRef,
    },
    /// Blurs the element and its subtree (a layer with the blur material).
    LayerBlur { radius: f32 },
    /// What is under the element blurred, its saturation and brightness
    /// scaled and tinted: glass.
    Backdrop {
        blur: f32,
        saturation: f32,
        brightness: f32,
        tint: ColorRef,
    },
}

/// The element's outline.
#[derive(Clone, Debug, PartialEq)]
pub enum ShapeKind {
    /// Its rectangle; `radii` top left, top right, bottom right, bottom
    /// left (`None`: the element's radius at every corner).
    Rect {
        radii: Option<[f32; 4]>,
        corner: Corner,
    },
    Ellipse,
    Polygon {
        sides: u32,
        inner: f32,
        rounding: f32,
        rotation: f32,
    },
    Arc {
        start: f32,
        sweep: f32,
        thickness: f32,
        cap: Cap,
    },
    /// A line `width` CSS px through `points` (CSS px from the element's
    /// top left); with `curves`, each three points after the first are a
    /// cubic Bézier's two controls and its end (connectors, graphs).
    Path {
        points: Vec<[f32; 2]>,
        closed: bool,
        curves: bool,
        width: f32,
        cap: Cap,
        join: Join,
    },
}

impl Default for ShapeKind {
    fn default() -> ShapeKind {
        ShapeKind::Rect {
            radii: None,
            corner: Corner::Round,
        }
    }
}

impl ShapeKind {
    /// The geometry of an element of `radius` (its default corner radius)
    /// and `size` (CSS px).
    pub fn geometry(&self, radius: f32, size: [f32; 2]) -> og_render::UiGeometry {
        match self {
            ShapeKind::Rect { radii, corner } => og_render::UiGeometry::Rect {
                radii: radii.unwrap_or([radius; 4]),
                corner: *corner,
            },
            ShapeKind::Ellipse => og_render::UiGeometry::Ellipse,
            ShapeKind::Polygon {
                sides,
                inner,
                rounding,
                rotation,
            } => og_render::UiGeometry::Polygon {
                sides: *sides,
                inner: *inner,
                rounding: *rounding,
                rotation: *rotation,
            },
            ShapeKind::Arc {
                start,
                sweep,
                thickness,
                cap,
            } => og_render::UiGeometry::Arc {
                start: *start,
                sweep: *sweep,
                thickness: *thickness,
                cap: *cap,
            },
            ShapeKind::Path {
                points,
                closed,
                curves,
                width,
                cap,
                join,
            } => {
                let half = Vec2::from(size) * 0.5;
                let p: Vec<Vec2> = points.iter().map(|p| Vec2::from(*p) - half).collect();
                let mut line = p.first().map(|p| vec![*p]).unwrap_or_default();
                if *curves {
                    for c in p[1.min(p.len())..].as_chunks::<3>().0 {
                        let from = *line.last().expect("the first point");
                        // A quarter CSS px from the curve.
                        line.extend(
                            og_render::flatten_cubic(from, c[0], c[1], c[2], 0.25)
                                .into_iter()
                                .skip(1),
                        );
                    }
                } else {
                    line.extend(p.iter().skip(1));
                }
                og_render::UiGeometry::Polyline {
                    points: line,
                    closed: *closed,
                    width: *width,
                    cap: *cap,
                    join: *join,
                }
            }
        }
    }
}

/// An alpha mask over the element's subtree.
#[derive(Clone, Debug, PartialEq)]
pub enum Mask {
    Gradient(Gradient),
    Image {
        texture: TextureHandle,
        uv: [f32; 4],
    },
}

/// How the element cuts its children.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Clip {
    #[default]
    None,
    /// To its shape, antialiased.
    Shape,
    /// By a mask's alpha (the subtree draws offscreen).
    Mask(Mask),
}

/// What a layer composites through.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Material {
    #[default]
    Plain,
    Builtin(Builtin),
    Custom(UiMaterialHandle),
}

/// The element's subtree drawn offscreen, then composited through a material.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layer {
    pub material: Material,
    pub params: [[f32; 4]; og_render::UI_MATERIAL_PARAMS],
}

/// Effects of the element's text (§5.6).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextEffects {
    pub shadows: Vec<TextShadow>,
    pub outline: Option<TextOutline>,
    /// A gradient over the text's line box.
    pub fill: Option<Gradient>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextShadow {
    pub offset: [f32; 2],
    pub blur: f32,
    pub color: ColorRef,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextOutline {
    pub width: f32,
    pub color: ColorRef,
    pub align: StrokeAlign,
}

/// What an element draws with. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct ElementDraw {
    pub shape: ShapeKind,
    pub fills: Vec<Paint>,
    pub strokes: Vec<Stroke>,
    pub effects: Vec<Effect>,
    /// Multiplies its own paints (a layer's composite, when it has one).
    pub opacity: f32,
    pub blend: Blend,
    pub clip: Clip,
    pub layer: Option<Layer>,
    pub text: Option<TextEffects>,
}

impl Default for ElementDraw {
    fn default() -> ElementDraw {
        ElementDraw {
            shape: ShapeKind::default(),
            fills: Vec::new(),
            strokes: Vec::new(),
            effects: Vec::new(),
            opacity: 1.0,
            blend: Blend::Normal,
            clip: Clip::None,
            layer: None,
            text: None,
        }
    }
}

/// Where an element is drawn: its rectangle, default corner radius, clip
/// and the alpha it fades by.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Placed {
    pub rect: Rect,
    pub radius: f32,
    pub clip: Rect,
    pub alpha: f32,
}

impl ElementDraw {
    /// Whether its subtree draws offscreen.
    fn layered(&self) -> bool {
        self.layer.is_some()
            || matches!(self.clip, Clip::Mask(_))
            || self
                .effects
                .iter()
                .any(|e| matches!(e, Effect::LayerBlur { .. }))
    }

    fn shape(
        &self,
        at: &Placed,
        paint: og_render::UiPaint,
        mode: og_render::UiMode,
        opacity: f32,
        blend: Blend,
    ) -> DrawCmd {
        let r = at.rect;
        DrawCmd::Shape {
            shape: Box::new(og_render::UiShape {
                geometry: self
                    .shape
                    .geometry(at.radius, [at.rect.width, at.rect.height]),
                center: Vec2::new(r.x + r.width / 2.0, r.y + r.height / 2.0),
                half: Vec2::new(r.width / 2.0, r.height / 2.0),
                transform: [1.0, 0.0, 0.0, 1.0],
                paint,
                mode,
                opacity,
                blend,
                layer: 0,
                clip: None,
            }),
            clip: at.clip,
        }
    }

    /// What draws before the element's own commands: a layer's start, then
    /// its shadows, outer glows and backdrops, then its fills, inner
    /// shadows and inner glows.
    pub(crate) fn enter(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        if self.layered() {
            let (material, params) = match &self.layer {
                Some(l) => (
                    match l.material {
                        Material::Plain => og_render::UiMaterial::Plain,
                        Material::Builtin(b) => og_render::UiMaterial::Builtin(b),
                        Material::Custom(h) => og_render::UiMaterial::Custom(h),
                    },
                    l.params,
                ),
                None => match self.effects.iter().find_map(|e| match e {
                    Effect::LayerBlur { radius } => Some(*radius),
                    _ => None,
                }) {
                    Some(radius) => {
                        let mut p = [[0.0; 4]; og_render::UI_MATERIAL_PARAMS];
                        p[0][0] = radius;
                        (og_render::UiMaterial::Builtin(Builtin::Blur), p)
                    }
                    None => (
                        og_render::UiMaterial::Plain,
                        [[0.0; 4]; og_render::UI_MATERIAL_PARAMS],
                    ),
                },
            };
            let mask = match &self.clip {
                Clip::Mask(Mask::Gradient(g)) => {
                    Some(og_render::UiMask::Gradient(g.resolve(theme)))
                }
                Clip::Mask(Mask::Image { texture, uv }) => Some(og_render::UiMask::Image {
                    texture: *texture,
                    uv: *uv,
                }),
                _ => None,
            };
            let blur = match material {
                og_render::UiMaterial::Builtin(Builtin::Blur) => params[0][0],
                og_render::UiMaterial::Builtin(Builtin::Outline) => params[0][0],
                og_render::UiMaterial::Builtin(Builtin::Ripple) => params[0][0],
                _ => 0.0,
            };
            out.push(DrawCmd::BeginLayer {
                material,
                params,
                mask,
                expand: blur.max(0.0) * 1.5,
                rect: Rect::default(),
            });
        }
        // Inside a layer the composite carries the opacity and blend.
        let (opacity, blend) = if self.layered() {
            (at.alpha, Blend::Normal)
        } else {
            (self.opacity * at.alpha, self.blend)
        };
        let o = og_render::UiMode::Fill;
        for e in &self.effects {
            let (mode, color) = match e {
                Effect::DropShadow {
                    offset,
                    blur,
                    spread,
                    color,
                } => (
                    og_render::UiMode::Shadow {
                        offset: Vec2::from(*offset),
                        blur: *blur,
                        spread: *spread,
                    },
                    color,
                ),
                Effect::OuterGlow {
                    blur,
                    spread,
                    color,
                } => (
                    og_render::UiMode::Shadow {
                        offset: Vec2::ZERO,
                        blur: *blur,
                        spread: *spread,
                    },
                    color,
                ),
                _ => continue,
            };
            out.push(self.shape(
                at,
                og_render::UiPaint::Solid(color.resolve(theme)),
                mode,
                opacity,
                Blend::Normal,
            ));
        }
        for e in &self.effects {
            if let Effect::Backdrop {
                blur,
                saturation,
                brightness,
                tint,
            } = e
            {
                out.push(self.shape(
                    at,
                    og_render::UiPaint::Solid(tint.resolve(theme)),
                    og_render::UiMode::Backdrop {
                        blur: *blur,
                        saturation: *saturation,
                        brightness: *brightness,
                    },
                    opacity,
                    Blend::Normal,
                ));
            }
        }
        for f in &self.fills {
            if let Paint::Vector { vector, tint } = f {
                let mut tint = tint.resolve(theme);
                tint[3] *= opacity;
                out.push(DrawCmd::Vector {
                    vector: *vector,
                    rect: at.rect,
                    transform: [1.0, 0.0, 0.0, 1.0],
                    tint,
                    clip: at.clip,
                });
                continue;
            }
            out.push(self.shape(at, f.resolve(theme), o, opacity, blend));
        }
        for e in &self.effects {
            let (mode, color) = match e {
                Effect::InnerShadow {
                    offset,
                    blur,
                    spread,
                    color,
                } => (
                    og_render::UiMode::InnerShadow {
                        offset: Vec2::from(*offset),
                        blur: *blur,
                        spread: *spread,
                    },
                    color,
                ),
                Effect::InnerGlow {
                    blur,
                    spread,
                    color,
                } => (
                    og_render::UiMode::InnerShadow {
                        offset: Vec2::ZERO,
                        blur: *blur,
                        spread: *spread,
                    },
                    color,
                ),
                _ => continue,
            };
            out.push(self.shape(
                at,
                og_render::UiPaint::Solid(color.resolve(theme)),
                mode,
                opacity,
                Blend::Normal,
            ));
        }
        out
    }

    /// What starts the children: the cut to the shape, when it clips.
    pub(crate) fn children(&self, at: &Placed) -> Vec<DrawCmd> {
        if self.clip != Clip::Shape {
            return Vec::new();
        }
        let r = at.rect;
        vec![DrawCmd::PushClip {
            geometry: self
                .shape
                .geometry(at.radius, [at.rect.width, at.rect.height]),
            center: (r.x + r.width / 2.0, r.y + r.height / 2.0),
            half: (r.width / 2.0, r.height / 2.0),
        }]
    }

    /// What draws after the subtree: the end of the cut, the strokes, then
    /// the layer's composite.
    pub(crate) fn leave(&self, theme: &Theme, at: &Placed) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        if self.clip == Clip::Shape {
            out.push(DrawCmd::PopClip);
        }
        let (opacity, blend) = if self.layered() {
            (at.alpha, Blend::Normal)
        } else {
            (self.opacity * at.alpha, self.blend)
        };
        for s in &self.strokes {
            out.push(self.shape(
                at,
                s.paint.resolve(theme),
                og_render::UiMode::Stroke(og_render::UiStroke {
                    width: s.width,
                    align: s.align,
                    dash: s.dash,
                    dash_offset: s.dash_offset,
                    dash_cap: s.dash_cap,
                }),
                opacity,
                blend,
            ));
        }
        if self.layered() {
            // The composite covers the layer; a mask or a material clips
            // to the element's shape only when it clips.
            let shaped = matches!(self.clip, Clip::Shape);
            out.push(DrawCmd::EndLayer {
                geometry: shaped.then(|| {
                    self.shape
                        .geometry(at.radius, [at.rect.width, at.rect.height])
                }),
                rect: at.rect,
                opacity: self.opacity,
                blend: self.blend,
                clip: at.clip,
            });
        }
        out
    }

    /// Its text effects over the line box `bounds`.
    pub(crate) fn text_effects(
        &self,
        theme: &Theme,
        bounds: Rect,
    ) -> Option<og_render::UiTextEffects> {
        let t = self.text.as_ref()?;
        Some(og_render::UiTextEffects {
            text: 0,
            shadows: t
                .shadows
                .iter()
                .map(|s| og_render::UiTextShadow {
                    offset: Vec2::from(s.offset),
                    blur: s.blur,
                    color: s.color.resolve(theme),
                })
                .collect(),
            outline: t.outline.as_ref().map(|o| og_render::UiTextOutline {
                width: o.width,
                color: o.color.resolve(theme),
                align: o.align,
            }),
            fill: t.fill.as_ref().map(|g| g.resolve(theme)),
            fill_box: [bounds.x, bounds.y, bounds.width, bounds.height],
            transform: [1.0, 0.0, 0.0, 1.0],
        })
    }

    /// Wraps the text commands an element pushed in its effects.
    pub(crate) fn style_text(&self, theme: &Theme, cmds: &mut [DrawCmd]) {
        if self.text.is_none() {
            return;
        }
        for c in cmds.iter_mut() {
            if let DrawCmd::Text { bounds, .. } = c {
                let Some(fx) = self.text_effects(theme, *bounds) else {
                    return;
                };
                let text = std::mem::replace(c, DrawCmd::PopClip);
                *c = DrawCmd::StyledText {
                    text: Box::new(text),
                    effects: Box::new(fx),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at() -> Placed {
        Placed {
            rect: Rect::new(10.0, 20.0, 200.0, 100.0),
            radius: 12.0,
            clip: Rect::new(0.0, 0.0, 1000.0, 1000.0),
            alpha: 1.0,
        }
    }

    #[test]
    fn tokens_resolve_through_the_theme_at_draw_time() {
        let theme = Theme::default();
        let c = ColorRef::token("primary");
        assert_eq!(c.resolve(&theme), theme.palette.primary.linear());
        let mut other = Theme::default();
        other.palette.primary = Srgba::rgb(10, 200, 30);
        assert_eq!(c.resolve(&other), other.palette.primary.linear());
        assert_eq!(ColorRef::token("primry").resolve(&theme), MISSING_TOKEN);
    }

    #[test]
    fn effects_go_under_the_fills_and_strokes_after_the_children() {
        let theme = Theme::default();
        let d = ElementDraw {
            fills: vec![Paint::Solid(ColorRef::token("panel"))],
            strokes: vec![Stroke {
                width: 2.0,
                align: StrokeAlign::Inside,
                dash: [0.0; 2],
                dash_offset: 0.0,
                dash_cap: Cap::Butt,
                paint: Paint::Solid(ColorRef::token("line")),
            }],
            effects: vec![
                Effect::InnerGlow {
                    blur: 4.0,
                    spread: 0.0,
                    color: ColorRef::token("highlight"),
                },
                Effect::DropShadow {
                    offset: [0.0, 4.0],
                    blur: 12.0,
                    spread: 0.0,
                    color: ColorRef::token("shadow"),
                },
            ],
            clip: Clip::Shape,
            ..Default::default()
        };
        let modes = |cmds: &[DrawCmd]| -> Vec<&'static str> {
            cmds.iter()
                .map(|c| match c {
                    DrawCmd::Shape { shape, .. } => match shape.mode {
                        og_render::UiMode::Fill => "fill",
                        og_render::UiMode::Stroke(_) => "stroke",
                        og_render::UiMode::Shadow { .. } => "shadow",
                        og_render::UiMode::InnerShadow { .. } => "inner",
                        og_render::UiMode::Backdrop { .. } => "backdrop",
                    },
                    DrawCmd::PushClip { .. } => "push",
                    DrawCmd::PopClip => "pop",
                    _ => "other",
                })
                .collect()
        };
        assert_eq!(modes(&d.enter(&theme, &at())), ["shadow", "fill", "inner"]);
        assert_eq!(modes(&d.children(&at())), ["push"]);
        assert_eq!(modes(&d.leave(&theme, &at())), ["pop", "stroke"]);
    }

    #[test]
    fn a_mask_or_a_blur_draws_the_subtree_offscreen() {
        let theme = Theme::default();
        let d = ElementDraw {
            effects: vec![Effect::LayerBlur { radius: 6.0 }],
            opacity: 0.5,
            ..Default::default()
        };
        let enter = d.enter(&theme, &at());
        let Some(DrawCmd::BeginLayer {
            material, params, ..
        }) = enter.first()
        else {
            panic!("{enter:?}");
        };
        assert_eq!(*material, og_render::UiMaterial::Builtin(Builtin::Blur));
        assert_eq!(params[0][0], 6.0);
        let leave = d.leave(&theme, &at());
        assert!(
            matches!(leave.last(), Some(DrawCmd::EndLayer { opacity, .. }) if *opacity == 0.5),
            "{leave:?}"
        );
    }
}
