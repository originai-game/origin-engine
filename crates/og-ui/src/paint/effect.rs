use super::{Cap, ColorRef, FilterUse, Paint, StrokeAlign};
use crate::layout::Rect;
use crate::style::Theme;

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
    /// CSS `backdrop-filter`: what is really under the element (the
    /// screen, or the group it is drawn in) through these stages in order,
    /// laid in its shape ([`og_draw::UiLayer::backdrop`]).
    BackdropFilter(std::sync::Arc<Vec<BackdropStage>>),
    /// What is under the element blurred, its saturation and brightness
    /// scaled and tinted: glass, with its material ([`GlassParts`]).
    Backdrop {
        blur: f32,
        saturation: f32,
        brightness: f32,
        tint: ColorRef,
        glass: GlassParts,
    },
}

/// One function of a `backdrop-filter`.
#[derive(Clone, Debug, PartialEq)]
pub enum BackdropStage {
    /// A Gaussian of this standard deviation, CSS px.
    Blur(f32),
    /// A run of colour functions (over sRGB-encoded straight colour).
    Matrix(og_draw::UiColorMatrix),
    /// `drop-shadow()`: its offset, its blur (twice the deviation) and colour.
    DropShadow {
        offset: [f32; 2],
        blur: f32,
        color: ColorRef,
    },
    /// `url(#id)`: an inline SVG `<filter>` in the box's user space.
    Url(std::sync::Arc<FilterUse>),
}

/// A `backdrop-filter`'s stages as one graph over what is under a box
/// of `size` (CSS px), in the box's user space: its region (the box grown
/// by what its stages reach, and the url filters' regions), the graph, and
/// each url stage's first element image.
pub(super) fn backdrop_graph(
    stages: &[BackdropStage],
    size: [f32; 2],
    theme: &Theme,
) -> (Rect, Vec<og_draw::UiFilterPrimitive>, Vec<u32>) {
    use og_draw::{UiFilterInput as I, UiFilterOp as Op, UiFilterPrimitive};
    let mut prims: Vec<UiFilterPrimitive> = Vec::new();
    let mut reach = 0.0f32;
    let mut regions: Vec<[f32; 4]> = Vec::new();
    let mut images = Vec::new();
    let mut elements = 0u32;
    let all = [-1.0e6, -1.0e6, 2.0e6, 2.0e6];
    let last = |prims: &[UiFilterPrimitive]| match prims.len() {
        0 => I::Source,
        n => I::Result(n as u32 - 1),
    };
    let push = |prims: &mut Vec<UiFilterPrimitive>, op, inputs, srgb| {
        prims.push(UiFilterPrimitive {
            op,
            inputs,
            subregion: all,
            srgb,
        });
        I::Result(prims.len() as u32 - 1)
    };
    for stage in stages {
        let from = last(&prims);
        match stage {
            BackdropStage::Blur(sd) => {
                reach += 3.0 * sd;
                push(
                    &mut prims,
                    Op::Blur { std_dev: [*sd; 2] },
                    [from, I::Source],
                    false,
                );
            }
            BackdropStage::Matrix(m) => {
                let r = m.rows;
                let mut v = [0.0; 20];
                for (i, row) in r.iter().enumerate() {
                    v[i * 5..i * 5 + 3].copy_from_slice(&row[..3]);
                    v[i * 5 + 4] = row[3];
                }
                v[18] = m.opacity;
                push(&mut prims, Op::ColorMatrix(v), [from, I::Source], true);
            }
            BackdropStage::DropShadow {
                offset,
                blur,
                color,
            } => {
                let sd = blur / 2.0;
                reach += 3.0 * sd + offset[0].abs().max(offset[1].abs());
                let c = color.resolve(theme);
                let soft = push(
                    &mut prims,
                    Op::Blur { std_dev: [sd; 2] },
                    [from, I::Source],
                    false,
                );
                let moved = push(
                    &mut prims,
                    Op::Offset { d: *offset },
                    [soft, I::Source],
                    false,
                );
                let fill = push(&mut prims, Op::Flood { color: c }, [I::Source; 2], false);
                let shadow = push(
                    &mut prims,
                    Op::Composite {
                        op: og_draw::UiFilterComposite::In,
                        k: [0.0; 4],
                    },
                    [fill, moved],
                    false,
                );
                push(&mut prims, Op::Merge, [from, shadow], false);
            }
            BackdropStage::Url(u) => {
                let (region, graph) = u.filter.resolve([0.0, 0.0, size[0], size[1]]);
                regions.push([region.x, region.y, region.width, region.height]);
                images.push(elements);
                // Its SourceAlpha is the alpha of what came before.
                let alpha = (from != I::Source).then(|| {
                    let mut a = [0.0; 20];
                    a[18] = 1.0;
                    push(&mut prims, Op::ColorMatrix(a), [from, I::Source], false)
                });
                let base = prims.len() as u32;
                for mut p in graph.primitives {
                    for i in &mut p.inputs {
                        *i = match *i {
                            I::Source => from,
                            I::SourceAlpha => alpha.unwrap_or(I::SourceAlpha),
                            I::Result(k) => I::Result(k + base),
                        };
                    }
                    if let Op::Layer { layer } = &mut p.op
                        && let Some(n) = crate::svg::filter::element_of(*layer)
                    {
                        *layer = crate::svg::filter::element_layer(n + elements);
                    }
                    prims.push(p);
                }
                elements += u.filter.elements.len() as u32;
            }
        }
    }
    let mut r = [-reach, -reach, size[0] + 2.0 * reach, size[1] + 2.0 * reach];
    for g in regions {
        let x = r[0].min(g[0]);
        let y = r[1].min(g[1]);
        r = [
            x,
            y,
            (r[0] + r[2]).max(g[0] + g[2]) - x,
            (r[1] + r[3]).max(g[1] + g[3]) - y,
        ];
    }
    (Rect::new(r[0], r[1], r[2], r[3]), prims, images)
}

/// A backdrop's material beyond its blur and tint (og-render's
/// [`og_draw::UiGlass`]); the default is none of it, the plain frosted
/// backdrop.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GlassParts {
    /// One of the design language's weights: its blur, 200 % saturation,
    /// tint (unless the effect gives one), 0.5 px hairline and CSS mixing,
    /// in the theme's scheme; under reduced transparency or increased
    /// contrast the solid fallback. `None`: the effect's own values.
    pub weight: Option<og_draw::UiGlassWeight>,
    /// The soft shadow outside it (`None`: none).
    pub elevation: Option<og_draw::UiElevation>,
    /// A hairline inside the edge, CSS px (0: none, or the weight's).
    pub edge: f32,
    /// The hairline's colour (`None`: the weight's, or none).
    pub edge_color: Option<ColorRef>,
    /// The tint drawn opaque; nothing under it is read.
    pub solid: bool,
    /// Mixed in sRGB-encoded values, as CSS does.
    pub css_mix: bool,
    /// The effect named its tint: a weight keeps it instead of its own.
    pub own_tint: bool,
    /// CSS supplies the filter; the weight supplies only the material.
    pub own_filter: bool,
    /// A colour matrix over the blurred backdrop (CSS's `backdrop-filter`
    /// colour functions, [`og_draw::UiGlass::matrix`]).
    pub matrix: Option<og_draw::UiColorMatrix>,
}

impl GlassParts {
    /// The backdrop's mode and tint for `theme` (`tint`: the effect's own,
    /// resolved; `explicit_tint`: the effect named one).
    pub fn resolve(
        &self,
        blur: f32,
        saturation: f32,
        brightness: f32,
        tint: [f32; 4],
        explicit_tint: bool,
        theme: &Theme,
    ) -> (og_draw::UiMode, [f32; 4]) {
        let filter = (blur, saturation, brightness);
        let dark = theme.scheme == crate::style::Scheme::Dark;
        let fallback = if theme.reduced_transparency {
            og_draw::UiGlassFallback::ReducedTransparency
        } else if theme.contrast == crate::style::Contrast::High {
            og_draw::UiGlassFallback::IncreasedContrast
        } else {
            og_draw::UiGlassFallback::None
        };
        let (mut glass, blur, saturation, brightness, tint) = match self.weight {
            Some(weight) => {
                let style = og_draw::UiGlassStyle {
                    weight,
                    elevation: self.elevation.unwrap_or_default(),
                    dark,
                    fallback,
                };
                let (mode, preset) = style.backdrop();
                let og_draw::UiMode::Backdrop {
                    blur,
                    saturation,
                    brightness,
                    glass,
                } = mode
                else {
                    unreachable!("a glass style is a backdrop")
                };
                let solid = fallback != og_draw::UiGlassFallback::None;
                let tint = if explicit_tint && !solid {
                    tint
                } else {
                    preset
                };
                (glass, blur, saturation, brightness, tint)
            }
            None => {
                let mut glass = og_draw::UiGlass::default();
                if let Some(e) = self.elevation {
                    let style = og_draw::UiGlassStyle {
                        elevation: e,
                        dark,
                        ..Default::default()
                    };
                    if let og_draw::UiMode::Backdrop { glass: g, .. } = style.backdrop().0 {
                        glass.shadows = g.shadows;
                    }
                }
                (glass, blur, saturation, brightness, tint)
            }
        };
        if self.edge > 0.0 {
            glass.edge = self.edge;
        }
        if let Some(c) = &self.edge_color {
            glass.edge_color = c.resolve(theme);
        }
        glass.solid |= self.solid;
        glass.srgb_mix |= self.css_mix;
        if self.matrix.is_some() {
            glass.matrix = self.matrix;
        }
        let (blur, saturation, brightness) = if self.own_filter && !glass.solid {
            filter
        } else {
            (blur, saturation, brightness)
        };
        let mode = og_draw::UiMode::Backdrop {
            blur,
            saturation,
            brightness,
            glass,
        };
        (mode, tint)
    }
}
