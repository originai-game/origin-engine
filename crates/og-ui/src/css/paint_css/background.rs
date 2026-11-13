use super::*;

/// A background layer's image.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssImage {
    None,
    Image(ImageSource),
    Set(Vec<(ImageSource, f32)>),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ImageSource {
    Gradient(CssGradient),
    /// `url(…)`: a texture file resolved relative to its stylesheet.
    Url(String),
}

impl CssImage {
    pub(crate) fn select(&self, resolution: f32) -> Option<(&ImageSource, f32)> {
        match self {
            Self::None => None,
            Self::Image(image) => Some((image, 1.0)),
            Self::Set(options) => options
                .iter()
                .filter(|(_, density)| *density > 0.0)
                .min_by(
                    |(_, a), (_, b)| match (*a >= resolution, *b >= resolution) {
                        (true, true) => a.total_cmp(b),
                        (false, false) => b.total_cmp(a),
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                    },
                )
                .map(|(image, density)| (image, *density)),
        }
    }

    pub(crate) fn urls(&self) -> impl Iterator<Item = &str> {
        let single = match self {
            Self::Image(image) => Some(image),
            _ => None,
        };
        let options = match self {
            Self::Set(options) => options.as_slice(),
            _ => &[],
        };
        single
            .into_iter()
            .chain(
                options
                    .iter()
                    .filter(|(_, density)| *density > 0.0)
                    .map(|(image, _)| image),
            )
            .filter_map(|image| match image {
                ImageSource::Url(url) => Some(url.as_str()),
                _ => None,
            })
    }
}

/// Formats decoded by the texture cook. An unsupported type discards only
/// its candidate, before equal-density candidates are deduplicated.
pub(crate) fn image_type_supported(mime: &str) -> bool {
    let (essence, parameters) = match mime.split_once(';') {
        Some((essence, parameters)) => (essence.trim_end_matches([' ', '\t']), Some(parameters)),
        None => (mime, None),
    };
    if parameters.is_some_and(|p| !mime_parameters(p)) {
        return false;
    }
    matches!(
        essence.to_ascii_lowercase().as_str(),
        "image/png"
            | "image/jpeg"
            | "image/webp"
            | "image/gif"
            | "image/svg+xml"
            | "image/ktx2"
            | "image/vnd-ms.dds"
            | "image/vnd.adobe.photoshop"
            | "image/x-tga"
            | "image/vnd.radiance"
            | "image/x-exr"
    )
}

/// RFC 9110 media-type parameters. CSS Images 4 requires a valid MIME
/// string, not MIME Sniffing's forgiving parser that drops malformed parameters.
fn mime_parameters(mut rest: &str) -> bool {
    let token = |c: char| c.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(c);
    let quoted = |c: char| matches!(c, '\t' | ' '..='~' | '\u{80}'..='\u{ff}');
    loop {
        rest = rest.trim_start_matches([' ', '\t']);
        let after_name = rest.trim_start_matches(token);
        if after_name.len() == rest.len() {
            return false;
        }
        let Some(value) = after_name.strip_prefix('=') else {
            return false;
        };
        if let Some(value) = value.strip_prefix('"') {
            let mut chars = value.chars();
            loop {
                match chars.next() {
                    Some('"') => break,
                    Some('\\') => {
                        if !chars.next().is_some_and(quoted) {
                            return false;
                        }
                    }
                    Some(c) if quoted(c) => {}
                    _ => return false,
                }
            }
            rest = chars.as_str();
        } else {
            rest = value.trim_start_matches(token);
            if rest.len() == value.len() {
                return false;
            }
        }
        if rest.is_empty() {
            return true;
        }
        let Some(next) = rest.trim_start_matches([' ', '\t']).strip_prefix(';') else {
            return false;
        };
        rest = next;
    }
}

/// `background-size` of one layer; a percentage is of the box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BgSize {
    Cover,
    Contain,
    /// `None` is `auto`.
    Given([Option<Pos>; 2]),
}

/// `background-position` on one axis: `align × (box − image) + offset`
/// CSS px (`right 10px` is (1, −10), `30%` is (0.3, 0)).
pub(crate) type BgPos = (f32, f32);

/// A box a layer is placed in or cut to (CSS Backgrounds 3 §3.7, 3.8;
/// CSS Masking 1 §6.7, 6.8).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BoxEdge {
    #[default]
    Border,
    Padding,
    Content,
    /// `background-clip: text`.
    Text,
    /// `mask-clip: no-clip`.
    NoClip,
}

/// `background-attachment`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Attach {
    #[default]
    Scroll,
    Fixed,
    Local,
}

/// One `background` or `mask` layer (CSS Backgrounds 3 §3, CSS Masking 1
/// §6): its image and how it is sized, placed, repeated, cut and combined.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssLayer {
    pub image: CssImage,
    pub size: BgSize,
    pub position: [BgPos; 2],
    pub repeat: [og_draw::UiRepeat; 2],
    pub origin: BoxEdge,
    pub clip: BoxEdge,
    pub attach: Attach,
    pub blend: Blend,
    /// `mask-mode: luminance`.
    pub luminance: bool,
}

impl CssLayer {
    pub(crate) fn new(image: CssImage, mask: bool) -> CssLayer {
        CssLayer {
            image,
            size: BgSize::Given([None; 2]),
            position: [(0.0, 0.0); 2],
            repeat: [og_draw::UiRepeat::Repeat; 2],
            origin: if mask {
                BoxEdge::Border
            } else {
                BoxEdge::Padding
            },
            clip: BoxEdge::Border,
            attach: Attach::Scroll,
            blend: Blend::Normal,
            luminance: false,
        }
    }
}

/// A box keyword of a layer (`mask`: the SVG boxes as the border box).
fn box_edge(w: &str, mask: bool) -> Option<BoxEdge> {
    Some(match w {
        "border-box" => BoxEdge::Border,
        "padding-box" => BoxEdge::Padding,
        "content-box" => BoxEdge::Content,
        "fill-box" | "stroke-box" | "view-box" if mask => BoxEdge::Border,
        _ => return None,
    })
}

fn attach(w: &str) -> Option<Attach> {
    Some(match w {
        "scroll" => Attach::Scroll,
        "fixed" => Attach::Fixed,
        "local" => Attach::Local,
        _ => return None,
    })
}

/// `mask-composite`: its Porter-Duff operator over the layers under it.
fn composite(w: &str) -> Option<Blend> {
    Some(match w {
        "add" => Blend::Normal,
        "subtract" => Blend::SourceOut,
        "intersect" => Blend::SourceIn,
        "exclude" => Blend::Xor,
        _ => return None,
    })
}

/// `mask-mode`: luminance or not.
fn mask_mode(w: &str) -> Option<bool> {
    match w {
        "luminance" => Some(true),
        "alpha" | "match-source" => Some(false),
        _ => None,
    }
}

/// A `background`'s layers, the first on top, and its trailing colour.
pub(crate) type Background = (Vec<CssLayer>, Option<Tok>);

pub(super) fn is_image(t: &Tok) -> bool {
    matches!(t, Tok::Func(f, _) if f == "url" || f == "image-set") || gradient(t).is_some()
}

/// A quoted or unquoted file URL.
pub(crate) fn url(args: &[Vec<Tok>]) -> Result<String, String> {
    let mut out = String::new();
    for t in args.iter().flatten() {
        match t {
            Tok::Str(s) | Tok::Ident(s) => out.push_str(s),
            Tok::Num(n, u) => {
                out.push_str(n);
                out.push_str(u);
            }
            Tok::Slash => out.push('/'),
            t => return Err(format!("url(): '{}' is not part of a name", describe(t))),
        }
    }
    if out.is_empty() {
        return Err("url() names an image file".into());
    }
    Ok(out)
}

pub(super) fn image(t: &Tok) -> Result<CssImage, String> {
    match t {
        Tok::Ident(w) if w == "none" => Ok(CssImage::None),
        Tok::Func(f, a) if f == "url" => url(a).map(|u| CssImage::Image(ImageSource::Url(u))),
        Tok::Func(f, options) if f == "image-set" => {
            let mut out = Vec::new();
            for option in options {
                let (first, descriptors) = option
                    .split_first()
                    .ok_or("image-set(): an image is required")?;
                let source = match first {
                    Tok::Str(url) => ImageSource::Url(url.clone()),
                    t => match image(t)? {
                        CssImage::Image(source) => source,
                        _ => {
                            return Err(
                                "image-set(): candidates cannot be none or another image-set()"
                                    .into(),
                            );
                        }
                    },
                };
                let mut density = None;
                let mut mime = None;
                for descriptor in descriptors {
                    match descriptor {
                        Tok::Func(f, args) if f == "type" && mime.is_none() => {
                            let [arg] = args.as_slice() else {
                                return Err("image-set(): type() takes one string".into());
                            };
                            let [Tok::Str(value)] = arg.as_slice() else {
                                return Err("image-set(): type() takes one string".into());
                            };
                            mime = Some(value);
                        }
                        t if density.is_none() => {
                            let (value, unit) = number(t)?;
                            let value = match unit {
                                "x" | "dppx" => value,
                                "dpi" => value / 96.0,
                                "dpcm" => value * 2.54 / 96.0,
                                _ => {
                                    return Err(
                                        "image-set(): resolution uses x, dppx, dpi or dpcm".into(),
                                    );
                                }
                            };
                            if !value.is_finite() || value < 0.0 {
                                return Err(
                                    "image-set(): resolution must be finite and nonnegative".into(),
                                );
                            }
                            density = Some(value);
                        }
                        _ => {
                            return Err(
                                "image-set(): one resolution and one type() per candidate".into()
                            );
                        }
                    }
                }
                let density = density.unwrap_or(1.0);
                if mime.is_none_or(|m| image_type_supported(m))
                    && !out.iter().any(|(_, d)| *d == density)
                {
                    out.push((source, density));
                }
            }
            Ok(CssImage::Set(out))
        }
        t => match gradient(t) {
            Some(g) => g.map(|g| CssImage::Image(ImageSource::Gradient(g))),
            None => Err(format!(
                "'{}' is not an image: url(…), a gradient or none",
                describe(t)
            )),
        },
    }
}

/// A `background-position` (one to four values, CSS Backgrounds 3
/// §3.6): keywords, lengths and percentages, an edge keyword followed by
/// its offset.
pub(crate) fn bg_position(t: &[Tok]) -> Result<[BgPos; 2], String> {
    let err = || {
        "a position is one to four of: left, center, right, top, bottom, <n>px, <n>%".to_string()
    };
    // An offset from the start: (align, px).
    let len = |t: &Tok| -> Result<BgPos, String> {
        Ok(match pos(t)? {
            Pos::Frac(f) => (f, 0.0),
            Pos::Px(p) => (0.0, p),
        })
    };
    // Which axis a keyword names (0 x, 1 y, 2 either) and its fraction.
    let word = |t: &Tok| -> Option<(u8, f32)> {
        match t {
            Tok::Ident(w) => match w.as_str() {
                "left" => Some((0, 0.0)),
                "right" => Some((0, 1.0)),
                "top" => Some((1, 0.0)),
                "bottom" => Some((1, 1.0)),
                "center" => Some((2, 0.5)),
                _ => None,
            },
            _ => None,
        }
    };
    let center = (0.5, 0.0);
    match t {
        [] => Ok([(0.0, 0.0); 2]),
        [a] => match word(a) {
            Some((1, f)) => Ok([center, (f, 0.0)]),
            Some((_, f)) => Ok([(f, 0.0), center]),
            None => Ok([len(a)?, center]),
        },
        [a, b] => {
            let (wa, wb) = (word(a), word(b));
            match (wa, wb) {
                // `top left`: y first.
                (Some((1, fa)), Some((k, fb))) if k != 1 => Ok([(fb, 0.0), (fa, 0.0)]),
                (Some((2, fa)), Some((0, fb))) => Ok([(fb, 0.0), (fa, 0.0)]),
                (Some((ka, fa)), Some((kb, fb))) if ka != 1 && kb != 0 => {
                    Ok([(fa, 0.0), (fb, 0.0)])
                }
                (Some((ka, fa)), None) if ka != 1 => Ok([(fa, 0.0), len(b)?]),
                (None, Some((kb, fb))) if kb != 0 => Ok([len(a)?, (fb, 0.0)]),
                (None, None) => Ok([len(a)?, len(b)?]),
                _ => Err(err()),
            }
        }
        _ if t.len() <= 4 => {
            // Edge keywords, each with an optional offset from that edge;
            // a `center` takes the axis the others leave.
            let mut edges = Vec::new();
            let mut i = 0;
            while i < t.len() {
                let (k, f) = word(&t[i]).ok_or_else(err)?;
                let offset = match t.get(i + 1) {
                    Some(o) if word(o).is_none() => {
                        i += 1;
                        Some(len(o)?)
                    }
                    _ => None,
                };
                i += 1;
                if k == 2 && offset.is_some() {
                    return Err(err());
                }
                edges.push((k, f, offset));
            }
            let mut out: [Option<BgPos>; 2] = [None; 2];
            for (k, f, offset) in edges
                .iter()
                .filter(|e| e.0 != 2)
                .chain(edges.iter().filter(|e| e.0 == 2))
            {
                let axis = match k {
                    2 => out.iter().position(Option::is_none).ok_or_else(err)?,
                    k => *k as usize,
                };
                if out[axis].is_some() {
                    return Err(err());
                }
                // An offset from the far edge counts back from it.
                out[axis] = Some(match *offset {
                    None => (*f, 0.0),
                    Some((a, p)) if *f == 1.0 => (1.0 - a, -p),
                    Some(o) => o,
                });
            }
            Ok([out[0].unwrap_or(center), out[1].unwrap_or(center)])
        }
        _ => Err(err()),
    }
}

/// A `background-size` of one layer.
fn bg_size(t: &[Tok]) -> Result<BgSize, String> {
    let one = |t: &Tok| -> Result<Option<Pos>, String> {
        match t {
            Tok::Ident(w) if w == "auto" => Ok(None),
            t => pos(t).map(Some),
        }
    };
    match t {
        [Tok::Ident(w)] if w == "cover" => Ok(BgSize::Cover),
        [Tok::Ident(w)] if w == "contain" => Ok(BgSize::Contain),
        [a] => Ok(BgSize::Given([one(a)?, None])),
        [a, b] => Ok(BgSize::Given([one(a)?, one(b)?])),
        _ => Err("a background-size is cover, contain, or one or two of auto, <n>px, <n>%".into()),
    }
}

fn repeat_word(w: &str) -> Option<og_draw::UiRepeat> {
    use og_draw::UiRepeat as R;
    Some(match w {
        "repeat" => R::Repeat,
        "no-repeat" => R::NoRepeat,
        "space" => R::Space,
        "round" => R::Round,
        _ => return None,
    })
}

/// A `background-repeat` of one layer.
fn bg_repeat(t: &[Tok]) -> Result<[og_draw::UiRepeat; 2], String> {
    use og_draw::UiRepeat as R;
    let word = |t: &Tok| match t {
        Tok::Ident(w) => repeat_word(w),
        _ => None,
    };
    match t {
        [Tok::Ident(w)] if w == "repeat-x" => Ok([R::Repeat, R::NoRepeat]),
        [Tok::Ident(w)] if w == "repeat-y" => Ok([R::NoRepeat, R::Repeat]),
        [a] if word(a).is_some() => Ok([word(a).unwrap(); 2]),
        [a, b] if word(a).is_some() && word(b).is_some() => Ok([word(a).unwrap(), word(b).unwrap()]),
        _ => Err("a background-repeat is repeat-x, repeat-y, or one or two of repeat, no-repeat, space, round".into()),
    }
}

/// One layer of the `background` shorthand; `last` may end in a colour.
fn layer(item: &str, last: bool, mask: bool) -> Result<(CssLayer, Option<Tok>), String> {
    let t = tokens(item)?;
    let mut img = None;
    let mut colour = None;
    let mut place = Vec::new();
    let mut size = None;
    let mut repeat = Vec::new();
    let mut boxes = Vec::new();
    let mut l = CssLayer::new(CssImage::None, mask);
    let mut k = 0;
    while k < t.len() {
        let tok = &t[k];
        let ident = match tok {
            Tok::Ident(w) => Some(w.as_str()),
            _ => None,
        };
        if is_image(tok) || ident == Some("none") {
            if img.is_some() {
                return Err("a background layer takes one image".into());
            }
            img = Some(image(tok)?);
        } else if matches!(tok, Tok::Slash) {
            // `<position> / <size>`: one or two sizes.
            let n = t[k + 1..]
                .iter()
                .take(2)
                .take_while(|t| match t {
                    Tok::Num(..) => true,
                    Tok::Ident(w) => matches!(w.as_str(), "auto" | "cover" | "contain"),
                    _ => false,
                })
                .count();
            if place.is_empty() || n == 0 {
                return Err("a background's size follows its position: <position> / <size>".into());
            }
            size = Some(bg_size(&t[k + 1..k + 1 + n])?);
            k += 1 + n;
            continue;
        } else if matches!(tok, Tok::Num(..))
            || matches!(ident, Some("left" | "right" | "top" | "bottom" | "center"))
        {
            place.push(tok.clone());
        } else if ident
            .is_some_and(|w| repeat_word(w).is_some() || w == "repeat-x" || w == "repeat-y")
        {
            repeat.push(tok.clone());
        } else if let Some(e) = ident.and_then(|w| box_edge(w, mask)) {
            boxes.push(e);
        } else if ident == Some(if mask { "no-clip" } else { "text" }) {
            l.clip = if mask { BoxEdge::NoClip } else { BoxEdge::Text };
            boxes.push(l.clip);
        } else if let Some(a) = ident.filter(|_| !mask).and_then(attach) {
            l.attach = a;
        } else if let Some(c) = ident.filter(|_| mask).and_then(composite) {
            l.blend = c;
        } else if let Some(m) = ident.filter(|_| mask).and_then(mask_mode) {
            l.luminance = m;
        } else if last && !mask && colour.is_none() && is_color(tok) {
            colour = Some(tok.clone());
        } else {
            return Err(format!(
                "'{}' is not part of a background layer: an image, <position> [/ <size>], a repeat{}",
                describe(tok),
                if last {
                    ", a colour"
                } else {
                    " (a colour only in the last layer)"
                }
            ));
        }
        k += 1;
    }
    l.image = img.unwrap_or(CssImage::None);
    // One box is the origin and the clip; two the origin, then the clip
    // (`text` and `no-clip` the clip only).
    match boxes.as_slice() {
        [] => {}
        [BoxEdge::Text | BoxEdge::NoClip] => {}
        [e] => (l.origin, l.clip) = (*e, *e),
        [o, c] if !matches!(o, BoxEdge::Text | BoxEdge::NoClip) => (l.origin, l.clip) = (*o, *c),
        _ => return Err("a layer takes one or two boxes: <origin> [<clip>]".into()),
    }
    l.position = bg_position(&place)?;
    if let Some(s) = size {
        l.size = s;
    }
    if !repeat.is_empty() {
        l.repeat = bg_repeat(&repeat)?;
    }
    Ok((l, colour))
}

/// A `background` (or `background-image`) value with an image layer: its
/// layers, the first on top, and the colour under them. `None` when it
/// has no image (a plain colour, lowered as before).
pub(crate) fn background(text: &str) -> Option<Result<Background, String>> {
    let Ok(t) = tokens(text) else { return None };
    if !t
        .iter()
        .any(|t| is_image(t) || matches!(t, Tok::Ident(w) if w == "none"))
    {
        return None;
    }
    let items = super::boxes::items(text);
    let n = items.len();
    Some((|| {
        let mut layers = Vec::new();
        let mut colour = None;
        for (k, item) in items.into_iter().enumerate() {
            let (l, c) = layer(item, k + 1 == n, false)?;
            layers.push(l);
            colour = colour.or(c);
        }
        Ok((layers, colour))
    })())
}

/// The longhands over a `background`'s or `mask`'s layers, by their
/// names after `background-` or `mask-` (`size`, `position`, `repeat`,
/// `origin`, `clip`, `attachment`, `blend-mode`, `mode`, `composite`),
/// each a comma list repeated to the layers' count (CSS Backgrounds 3
/// §3); an empty value is the shorthand's own.
pub(crate) fn longhands(
    layers: &mut [CssLayer],
    values: &[(&str, Option<&str>)],
    mask: bool,
) -> Result<(), String> {
    fn each<T: Copy>(
        text: Option<&str>,
        layers: &mut [CssLayer],
        parse: impl Fn(&[Tok]) -> Result<T, String>,
        set: impl Fn(&mut CssLayer, T),
    ) -> Result<(), String> {
        let Some(text) = text.filter(|t| !t.trim().is_empty()) else {
            return Ok(());
        };
        let values: Vec<T> = super::boxes::items(text)
            .iter()
            .map(|i| tokens(i).and_then(|t| parse(&t)))
            .collect::<Result<_, _>>()?;
        for (k, l) in layers.iter_mut().enumerate() {
            set(l, values[k % values.len()]);
        }
        Ok(())
    }
    fn word<T>(
        what: &'static str,
        f: impl Fn(&str) -> Option<T>,
    ) -> impl Fn(&[Tok]) -> Result<T, String> {
        move |t: &[Tok]| {
            match t {
                [Tok::Ident(w)] => f(w),
                _ => None,
            }
            .ok_or_else(|| format!("not a {what}"))
        }
    }
    for (name, text) in values {
        match *name {
            "size" => each(*text, layers, bg_size, |l, v| l.size = v)?,
            "position" => each(*text, layers, bg_position, |l, v| l.position = v)?,
            "repeat" => each(*text, layers, bg_repeat, |l, v| l.repeat = v)?,
            "origin" => each(*text, layers, word("box", |w| box_edge(w, mask)), |l, v| {
                l.origin = v
            })?,
            "clip" => each(
                *text,
                layers,
                word("clip box", |w| match w {
                    "text" if !mask => Some(BoxEdge::Text),
                    "no-clip" if mask => Some(BoxEdge::NoClip),
                    w => box_edge(w, mask),
                }),
                |l, v| l.clip = v,
            )?,
            "attachment" => each(
                *text,
                layers,
                word("attachment: scroll, fixed, local", attach),
                |l, v| l.attach = v,
            )?,
            "blend-mode" => each(
                *text,
                layers,
                word("blend mode", |w| blend(w).ok()),
                |l, v| l.blend = v,
            )?,
            "mode" => each(
                *text,
                layers,
                word("mask mode: alpha, luminance, match-source", mask_mode),
                |l, v| l.luminance = v,
            )?,
            "composite" => each(
                *text,
                layers,
                word("composite: add, subtract, intersect, exclude", composite),
                |l, v| l.blend = v,
            )?,
            _ => {}
        }
    }
    Ok(())
}

/// The props of the layer, border image and mask border longhands, in
/// [`super::css::STYLE`]'s order.
pub(crate) const C4_PROPS: [&str; 22] = [
    "backgroundOrigin",
    "backgroundClip",
    "backgroundAttachment",
    "backgroundBlendMode",
    "maskSize",
    "maskPosition",
    "maskRepeat",
    "maskOrigin",
    "maskClip",
    "maskMode",
    "maskComposite",
    "borderImageSource",
    "borderImageSlice",
    "borderImageWidth",
    "borderImageOutset",
    "borderImageRepeat",
    "maskBorderSource",
    "maskBorderSlice",
    "maskBorderWidth",
    "maskBorderOutset",
    "maskBorderRepeat",
    "maskBorderMode",
];

/// The layer longhands (after `background-` or `mask-`), each with its
/// element prop.
pub(crate) const BACKGROUND_LONGHANDS: [(&str, &str); 7] = [
    ("size", "backgroundSize"),
    ("position", "backgroundPosition"),
    ("repeat", "backgroundRepeat"),
    ("origin", "backgroundOrigin"),
    ("clip", "backgroundClip"),
    ("attachment", "backgroundAttachment"),
    ("blend-mode", "backgroundBlendMode"),
];
pub(crate) const MASK_LONGHANDS: [(&str, &str); 7] = [
    ("size", "maskSize"),
    ("position", "maskPosition"),
    ("repeat", "maskRepeat"),
    ("origin", "maskOrigin"),
    ("clip", "maskClip"),
    ("mode", "maskMode"),
    ("composite", "maskComposite"),
];

/// `mask-image` or the `mask` shorthand: its layers, the first on top
/// (a `none` layer draws nothing and is left out).
pub(crate) fn masks(text: &str) -> Result<Vec<CssLayer>, String> {
    if text.trim() == "none" {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for item in super::boxes::items(text) {
        let (l, _) = layer(item, false, true)?;
        if l.image != CssImage::None {
            out.push(l);
        }
    }
    Ok(out)
}
