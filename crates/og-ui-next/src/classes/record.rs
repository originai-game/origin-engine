//! Converting a resolved `nsh_tailwind::Style` into a [`Record`] and its
//! [`motion::Spec`].

use super::*;

/// A packed sRGB colour (`0xRRGGBBAA`) as linear RGBA, straight alpha.
pub(super) fn linear(c: Color) -> Rgba {
    let [r, g, b, a] = c.0.to_be_bytes();
    og_css::Srgba {
        rgb: [r, g, b],
        alpha: f32::from(a) / 255.0,
        p3: None,
    }
    .linear()
}

fn fraction(l: &DefiniteLength) -> Option<f32> {
    match l {
        DefiniteLength::Fraction(p) => Some(p.0),
        DefiniteLength::Absolute(_) => None,
    }
}

/// A colour filter function as its matrix (Filter Effects 1 §13, on the
/// encoded colour); `None` for one that needs the pixels around it (blur,
/// drop-shadow).
fn color_matrix(f: &Filter) -> Option<og_draw::paint::ColorMatrix> {
    type M3 = [[f32; 3]; 3];
    let rows = |m: M3, offset: f32, alpha: f32| {
        let [r, g, b] = m.map(|[x, y, z]| [x, y, z, 0.0]);
        [
            r,
            g,
            b,
            [0.0, 0.0, 0.0, alpha],
            [offset, offset, offset, 0.0],
        ]
    };
    let diag = |s: f32| [[s, 0.0, 0.0], [0.0, s, 0.0], [0.0, 0.0, s]];
    // `base` moved `t` of the way to `toward`, entry by entry.
    let mix = |base: M3, toward: M3, t: f32| -> M3 {
        std::array::from_fn(|i| {
            std::array::from_fn(|k| base[i][k] + (toward[i][k] - base[i][k]) * t)
        })
    };
    let id = diag(1.0);
    Some(match *f {
        Filter::Brightness(s) => rows(diag(s), 0.0, 1.0),
        Filter::Contrast(s) => rows(diag(s), 0.5 - 0.5 * s, 1.0),
        Filter::Grayscale(s) => {
            let gray = [[0.2126, 0.7152, 0.0722]; 3];
            rows(mix(id, gray, s.clamp(0.0, 1.0)), 0.0, 1.0)
        }
        Filter::Sepia(s) => {
            let sepia = [
                [0.393, 0.769, 0.189],
                [0.349, 0.686, 0.168],
                [0.272, 0.534, 0.131],
            ];
            rows(mix(id, sepia, s.clamp(0.0, 1.0)), 0.0, 1.0)
        }
        // From the luminance (0) through the colour (1), past it above 1.
        Filter::Saturate(s) => rows(mix([[0.213, 0.715, 0.072]; 3], id, s.max(0.0)), 0.0, 1.0),
        Filter::HueRotate(a) => {
            let (sin, cos) = og_core::math::sin_cos(a.0);
            let m = [
                [
                    0.213 + cos * 0.787 - sin * 0.213,
                    0.715 - cos * 0.715 - sin * 0.715,
                    0.072 - cos * 0.072 + sin * 0.928,
                ],
                [
                    0.213 - cos * 0.213 + sin * 0.143,
                    0.715 + cos * 0.285 + sin * 0.140,
                    0.072 - cos * 0.072 - sin * 0.283,
                ],
                [
                    0.213 - cos * 0.213 - sin * 0.787,
                    0.715 - cos * 0.715 + sin * 0.715,
                    0.072 + cos * 0.928 + sin * 0.072,
                ],
            ];
            rows(m, 0.0, 1.0)
        }
        Filter::Invert(s) => {
            let s = s.clamp(0.0, 1.0);
            rows(diag(1.0 - 2.0 * s), s, 1.0)
        }
        Filter::Opacity(s) => rows(id, 0.0, s.clamp(0.0, 1.0)),
        Filter::Blur(_) | Filter::DropShadow { .. } => return None,
    })
}

/// An NSH gradient as a record's: its shape, angle, centre and up to
/// [`GRADIENT_STOPS`] stops.
fn gradient_of(g: &nsh_tailwind::effects::Gradient) -> Gradient {
    let mid = [0.5; 2];
    let (shape, angle, corner, center) = match g.kind {
        GradientKind::Linear { angle } => (Shape::Linear, angle.0, [0, 0], mid),
        GradientKind::LinearCorner { right, bottom } => (
            Shape::Linear,
            0.0,
            [if right { 1 } else { -1 }, if bottom { 1 } else { -1 }],
            mid,
        ),
        GradientKind::Radial { circle, size, x, y } => {
            (Shape::Radial { circle, size }, 0.0, [0, 0], [x.0, y.0])
        }
        GradientKind::Conic { from, x, y } => (Shape::Conic, from.0, [0, 0], [x.0, y.0]),
    };
    let n = g.stops.len().min(GRADIENT_STOPS);
    let mut out = Gradient {
        shape,
        angle,
        corner,
        center,
        count: n as u8,
        stops: [(0.0, [0.0; 4]); GRADIENT_STOPS],
    };
    let all = g.stops.len();
    for (k, slot) in out.stops.iter_mut().take(n).enumerate() {
        // Past the cap, every stop's place is sampled evenly, so
        // the last stop is always kept.
        let i = if n > 1 {
            (k * (all - 1) + (n - 1) / 2) / (n - 1)
        } else {
            0
        };
        let s = &g.stops[i];
        // An unplaced stop spreads evenly between its neighbours.
        let even = if all > 1 {
            i as f32 / (all - 1) as f32
        } else {
            0.0
        };
        let at = s.position.as_ref().and_then(fraction).unwrap_or(even);
        *slot = (at, linear(s.color));
    }
    out
}

pub(super) fn record_of(style: &Style, props: &ClassProps, theme: &Theme) -> Record {
    let mut r = Record {
        layout: CompactStyle::from(&style.layout),
        ..Record::default()
    };
    // `pt-[calc(env(hud-inset-top)+16px)]`: padding that adds the HUD's
    // safe area, on the side whose own inset it reads.
    for (k, var) in style.env_padding().into_iter().enumerate() {
        use nsh_tailwind::style::EnvVar::*;
        let side = match var {
            Some(HudInsetTop) => 0,
            Some(HudInsetRight) => 1,
            Some(HudInsetBottom) => 2,
            Some(HudInsetLeft) => 3,
            _ => continue,
        };
        if side == k {
            r.safe[1] |= 1 << k;
        }
    }
    // A `<Line>`'s stroke: `stroke-accent`, `stroke-2`.
    match &style.paint.stroke {
        Some(SvgPaint::Color(c)) => r.stroke = Some(linear(*c)),
        Some(SvgPaint::None) => r.stroke = Some([0.0; 4]),
        Some(SvgPaint::CurrentColor) | None => {}
    }
    r.fill = style.paint.fill.as_ref().map(|f| match f {
        SvgPaint::Color(c) => Some(linear(*c)),
        SvgPaint::None => Some([0.0; 4]),
        SvgPaint::CurrentColor => None,
    });
    if let Some(w) = style.paint.stroke_width {
        r.stroke_width = w.0;
    }
    r.cursor = style.view.cursor;
    r.z_index = style.z_index;
    r.hidden = style.view.hidden;
    r.object = style
        .object_fit()
        .map(|fit| (fit, style.view.object_position));
    r.touch = props
        .pointer_events
        .map(|p| p == nsh_tailwind::props::PointerEvents::Auto);
    let (view, text, skin) = (&style.view, &style.text, &mut r.skin);
    skin.fill = linear(view.background_color);
    skin.opacity = view.opacity;
    skin.blend = match view.blend_mode {
        nsh_tailwind::effects::BlendMode::PlusLighter => 1,
        nsh_tailwind::effects::BlendMode::Screen => 2,
        nsh_tailwind::effects::BlendMode::Multiply => 3,
        _ => 0,
    };
    let c = &view.corner_radius;
    let all = view.border_radius.0;
    skin.radius =
        [c.top_left, c.top_right, c.bottom_right, c.bottom_left].map(|v| v.map_or(all, |p| p.0));
    let w = &view.borders.widths;
    skin.border = [w.top.0, w.right.0, w.bottom.0, w.left.0];
    // The commonest side colour is the border's; the sides of the first
    // other one take it (a third colour draws as the second); a tie goes to
    // top's.
    let cs = view.borders.colors;
    let base = *cs
        .iter()
        .rev()
        .max_by_key(|c| cs.iter().filter(|d| d == c).count())
        .expect("four sides");
    skin.border_color = linear(base);
    skin.border_sides = 0;
    if let Some(&side) = cs.iter().find(|&&c| c != base) {
        skin.border_side = linear(side);
        skin.border_sides = (0..4).filter(|&k| cs[k] != base).fold(0, |m, k| m | 1 << k);
    }
    r.layout.set_border(skin.border);
    skin.outline.width = view.outline.width.0;
    skin.outline.offset = view.outline.offset.0;
    skin.outline.color = linear(view.outline.color);
    // Translate (a fraction of the box's own size resolved at paint),
    // scale, rotate and 2D skew; 3D transforms are left out.
    for op in view.transform.ops() {
        let (t, v) = (&mut skin.transform, op.value);
        match op.kind {
            TransformKind::TranslateX => t.translate[0] += v,
            TransformKind::TranslateY => t.translate[1] += v,
            TransformKind::TranslateXFraction => t.percent[0] += v,
            TransformKind::TranslateYFraction => t.percent[1] += v,
            TransformKind::Scale => t.scale = [t.scale[0] * v, t.scale[1] * v],
            TransformKind::ScaleX => t.scale[0] *= v,
            TransformKind::ScaleY => t.scale[1] *= v,
            TransformKind::Rotate => t.rotate += v.to_radians(),
            TransformKind::SkewX => t.skew[0] += v.to_radians(),
            TransformKind::SkewY => t.skew[1] += v.to_radians(),
            _ => {}
        }
    }
    skin.transform.origin = [
        fraction(&view.transform_origin.x).unwrap_or(0.5),
        fraction(&view.transform_origin.y).unwrap_or(0.5),
    ];
    if let Some(fx) = &view.effects {
        for (k, s) in fx.box_shadows.iter().take(SHADOWS).enumerate() {
            skin.shadows[k] = Shadow {
                offset: [s.offset_x.0, s.offset_y.0],
                blur: s.blur.0,
                spread: s.spread.0,
                color: linear(s.color),
                inset: s.inset,
            };
            skin.shadow_count = k as u8 + 1;
        }
        skin.gradient = fx.background_images.first().map(gradient_of);
        r.mask = fx.mask_images.first().map(gradient_of);
        r.filter = fx
            .filters
            .iter()
            .filter_map(color_matrix)
            .reduce(|m, next| og_draw::paint::color_then(&m, &next));
        let effects = fx
            .filters
            .iter()
            .filter_map(|f| match *f {
                Filter::Blur(p) if p.0 > 0.0 => Some(PaintEffect::Blur(p.0)),
                Filter::DropShadow {
                    offset_x,
                    offset_y,
                    blur,
                    color,
                    // Its blur is a radius, as a box shadow's is: a Gaussian
                    // of half its deviation, as browsers draw it.
                } if color.0 & 0xff != 0 => Some(PaintEffect::DropShadow {
                    offset: [offset_x.0, offset_y.0],
                    sigma: 0.5 * blur.0.max(0.0),
                    color: linear(color),
                }),
                _ => None,
            })
            .take(crate::record::EFFECTS);
        r.effect_count = 0;
        for (k, e) in effects.enumerate() {
            r.effects[k] = e;
            r.effect_count = k as u8 + 1;
        }
        // Blurs in a list add up (a Gaussian of their deviations'
        // root sum of squares).
        skin.backdrop_blur = fx
            .backdrop_filters
            .iter()
            .filter_map(|f| match f {
                Filter::Blur(p) => Some(p.0 * p.0),
                _ => None,
            })
            .sum::<f32>()
            .sqrt();
        r.backdrop_filter = fx
            .backdrop_filters
            .iter()
            .filter_map(color_matrix)
            .reduce(|m, next| og_draw::paint::color_then(&m, &next));
    }
    if let Some(c) = text.color {
        skin.ink = linear(c);
        r.inked = true;
    }
    // Every layer; `text-shadow-none` sets none, so a child of a shadowed
    // parent draws none instead of inheriting it.
    if let Some(list) = text.shadows {
        let t = &mut skin.text_shadows;
        *t = TextShadows::default();
        for (k, l) in list.layers().iter().enumerate() {
            t.layers[k] = Shadow {
                offset: [l.offset_x.0, l.offset_y.0],
                blur: l.radius.0.max(0.0),
                spread: 0.0,
                color: linear(l.color),
                inset: false,
            };
            t.ink[k] = l.ink;
        }
        t.count = list.layers().len() as u8;
        r.shadowed = true;
    }
    if let Some(d) = text.decoration {
        skin.decoration.line = u8::from(d.underline) | (u8::from(d.line_through) << 2);
        skin.decoration.color = text.decoration_color.map_or([0.0; 4], linear);
        skin.decoration.thickness = text.decoration_thickness.map_or(0.0, |p| p.0);
    }
    skin.decoration.style = text.decoration_style.unwrap_or_default();
    skin.decoration.offset = text.underline_offset.map(|p| p.0);
    skin.caret = text.caret_color.or(props.caret_color).map(linear);
    skin.placeholder = text
        .placeholder_color
        .or(props.placeholder_color)
        .map(linear);
    skin.selection = text.selection_color.or(props.selection_color).map(linear);
    // The outline (`text-stroke-*`), its colour the text's by default.
    if let Some(w) = text.stroke_width {
        skin.text_stroke.width = w.0;
        skin.text_stroke.color = text.stroke_color.map_or(skin.ink, linear);
        r.stroked = true;
    }
    let t = &mut r.text;
    if let Some(s) = text.font_size {
        r.font_size = s.0;
        t.sized = true;
    }
    r.font_size_em = text.font_size_em;
    t.sized |= r.font_size_em.is_some();
    if let Some(w) = text.font_weight {
        t.weight = w;
        t.set |= Text::WEIGHT;
    }
    if let Some(m) = text.line_height_multiple.or(text.line_height_em) {
        (t.line, t.line_px) = (m, false);
        t.set |= Text::LINE;
    } else if let Some(p) = text.line_height {
        (t.line, t.line_px) = (p.0, true);
        t.set |= Text::LINE;
    }
    if let Some(em) = text.letter_spacing_em {
        (t.tracking_em, t.tracking_px) = (em, 0.0);
        t.set |= Text::TRACKING;
    } else if let Some(p) = text.letter_spacing {
        (t.tracking_em, t.tracking_px) = (0.0, p.0);
        t.set |= Text::TRACKING;
    }
    if let Some(a) = text.text_align {
        t.align = match a {
            TextAlign::Auto => 0,
            TextAlign::Center => 1,
            TextAlign::Right => 2,
            TextAlign::Left => 3,
            TextAlign::Justify => 4,
        };
        t.set |= Text::ALIGN;
    }
    if let Some(x) = text.transform {
        t.transform = match x {
            TextTransform::Uppercase => 1,
            TextTransform::Lowercase => 2,
            TextTransform::Capitalize => 3,
            _ => 0,
        };
        t.set |= Text::TRANSFORM;
    }
    if let Some(h) = text.hyphens {
        t.hyphens = h;
        t.set |= Text::HYPHENS;
    }
    // NSH's family list, slant and variant forms go to the text's layout
    // as they are (text.rs): no copy of their meaning here.
    if let Some(f) = text.font_family {
        t.family = Some(f);
        t.set |= Text::FAMILY;
    }
    if let Some(i) = text.italic {
        t.italic = i;
        t.set |= Text::ITALIC;
    }
    if let Some(v) = text.font_variant {
        t.variant = v.0;
        t.set |= Text::VARIANT;
    }
    if let Some(w) = props.overflow_wrap {
        t.wrap = w;
        t.set |= Text::WRAP;
    }
    if let Some(v) = props.vertical {
        t.vertical = v;
        t.set |= Text::VERTICAL;
    }
    if let Some(on) = props.selectable {
        t.select = on;
        t.set |= Text::SELECT;
    }
    r.motion = motion_of(style, theme);
    if let Some(nowrap) = props.no_wrap {
        t.nowrap = nowrap;
        t.set |= Text::NOWRAP;
    }
    if let Some(n) = props.number_of_lines {
        t.clamp = n.min(u32::from(u16::MAX)) as u16;
        t.set |= Text::CLAMP;
    }
    if let Some(mode) = props.ellipsize {
        // `text-clip` cuts with no ellipsis; Parley places one at a
        // line's end only, so every other mode is the tail's.
        t.overflow = u8::from(mode != nsh_tailwind::style::EllipsizeMode::Clip);
        t.set |= Text::OVERFLOW;
    }
    r
}

/// What `transition-*`, `duration-*`, `delay-*`, `ease-*` and `animate-*`
/// ask of motion; NSH's milliseconds become seconds.
pub(super) fn motion_of(style: &Style, theme: &Theme) -> motion::Spec {
    let mut m = motion::Spec::default();
    if let Some(t) = &style.view.transition {
        // NSH's groups are motion's bits one to one, `LAYOUT` (ours) too.
        m.props = t.properties.0;
        m.duration = t.duration / 1000.0;
        m.delay = t.delay / 1000.0;
        m.ease = match t.spring {
            Some(s) => motion::Ease::Spring {
                stiffness: s.stiffness,
                damping: s.damping,
                mass: s.mass,
            },
            None => motion::Ease::Bezier(t.easing),
        };
    }
    m.animate = style.view.animation.map(|a| {
        let (keyframes, peak) = match a.keyframes {
            Keyframes::Spin => (motion::Keyframes::Spin, 0.0),
            Keyframes::Ping => (motion::Keyframes::Ping, 0.0),
            Keyframes::Pulse => (motion::Keyframes::Pulse, 0.0),
            Keyframes::Bounce => (motion::Keyframes::Bounce, 0.0),
            Keyframes::Pop => (motion::Keyframes::Pop, 0.0),
            Keyframes::Stamp => (motion::Keyframes::Stamp, 0.0),
            Keyframes::Beat => (
                motion::Keyframes::Beat,
                theme.scale("combo-pop").unwrap_or(0.0),
            ),
            Keyframes::Float => (
                motion::Keyframes::Float,
                theme.scale("damage-pop").unwrap_or(0.0),
            ),
            Keyframes::Shimmer => (motion::Keyframes::Shimmer, 0.0),
            Keyframes::Rise => (motion::Keyframes::Rise, 0.0),
            Keyframes::Charge => (motion::Keyframes::Charge, 0.0),
            Keyframes::Nudge => (motion::Keyframes::Nudge, 0.0),
            Keyframes::Breathe => (
                motion::Keyframes::Breathe,
                theme.scale("ready-pulse").unwrap_or(0.0),
            ),
            Keyframes::Shake => (motion::Keyframes::Shake, 0.0),
            Keyframes::Flash => (
                motion::Keyframes::Flash,
                theme.opacity("flash").unwrap_or(0.0),
            ),
            Keyframes::Recoil => (motion::Keyframes::Recoil, 0.0),
        };
        motion::Loop {
            keyframes,
            duration: a.duration / 1000.0,
            delay: a.delay / 1000.0,
            ease: a.easing,
            iterations: a.iterations,
            peak,
        }
    });
    m
}
