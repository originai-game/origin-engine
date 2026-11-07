//! A read `.riv` ([`super::read`]) as og-ui artboards
//! ([`og_ui::rive::Artboard`]), phase 1 of `docs/ui.md` §11.2: artboards,
//! groups, shapes and their paths (points, rectangle, ellipse, triangle,
//! polygon, star), fills and strokes of solid colours and gradients,
//! linear animations keying transform, opacity, colour and the paths'
//! numbers, and nested artboards. Anything else is an error naming the
//! feature and the object it is on; nothing is dropped silently.
//!
//! Ids follow rive-runtime's `File::readObjects`: objects after an
//! artboard take the next index of its list (the artboard is 0) when they
//! are components or interpolators, or when the runtime would read them as
//! empty objects there; animations and their keys take none.

use super::read::{Object, RivFile, is_a, type_name};
use og_core::glam::Vec2;
use og_render::{UiBlend, UiCap, UiFillRule};
use og_ui::rive::{
    Animation, Artboard, Ease, Fit, HAlign, Join, Key, Kind, Loop, NO_FONT, NestedAnimation, Node,
    Outline, Overflow, Prop, Sizing, TextLayout, Track, VAlign, Values, Vertex,
};
use std::collections::HashMap;

// Type keys (rive-runtime's generated headers, `super::defs`).
const ARTBOARD: u16 = 1;
const NODE: u16 = 2;
const SHAPE: u16 = 3;
const ELLIPSE: u16 = 4;
const STRAIGHT_VERTEX: u16 = 5;
const DETACHED_VERTEX: u16 = 6;
const RECTANGLE: u16 = 7;
const TRIANGLE: u16 = 8;
const COMPONENT: u16 = 10;
const POINTS_PATH: u16 = 16;
const RADIAL_GRADIENT: u16 = 17;
const SOLID_COLOR: u16 = 18;
const GRADIENT_STOP: u16 = 19;
const FILL: u16 = 20;
const LINEAR_GRADIENT: u16 = 22;
const STROKE: u16 = 24;
const KEYED_OBJECT: u16 = 25;
const KEYED_PROPERTY: u16 = 26;
const CUBIC_EASE: u16 = 28;
const KEYFRAME_DOUBLE: u16 = 30;
const LINEAR_ANIMATION: u16 = 31;
const ASYMMETRIC_VERTEX: u16 = 34;
const MIRRORED_VERTEX: u16 = 35;
const KEYFRAME_COLOR: u16 = 37;
const POLYGON: u16 = 51;
const STAR: u16 = 52;
const STATE_MACHINE: u16 = 53;
const SM_LAYER: u16 = 57;
const ANIMATION_STATE: u16 = 61;
const ANY_STATE: u16 = 62;
const ENTRY_STATE: u16 = 63;
const EXIT_STATE: u16 = 64;
const STATE_TRANSITION: u16 = 65;
const NESTED_ARTBOARD: u16 = 92;
const NESTED_SIMPLE_ANIMATION: u16 = 96;
const CUBIC_VALUE: u16 = 138;
const KEYFRAME_INTERPOLATOR: u16 = 175;
const USER_INPUT: u16 = 663;
const IMAGE: u16 = 100;
const IMAGE_ASSET: u16 = 105;
const FILE_ASSET_CONTENTS: u16 = 106;
const FILE_ASSET: u16 = 103;
const KEYFRAME_ID: u16 = 50;
const TEXT: u16 = 134;
const TEXT_VALUE_RUN: u16 = 135;
const TEXT_STYLE_PAINT: u16 = 137;
const FONT_ASSET: u16 = 141;
const KEYFRAME_STRING: u16 = 142;
const TEXT_STYLE_AXIS: u16 = 144;

/// The objects that are file assets or file-level data the artboards'
/// converted objects never reference (the ones that are referenced, by an
/// image or a text, are refused on that object).
const FILE_LEVEL: &[&str] = &[
    "Backboard",
    "ImageAsset",
    "FontAsset",
    "AudioAsset",
    "FileAssetContents",
];

/// Every artboard of `file`, in file order, converted.
#[cfg(test)]
pub(crate) fn artboards(file: &RivFile) -> Result<Vec<Artboard>, String> {
    let groups = group(file)?;
    let assets = assets(file);
    (0..groups.len())
        .map(|g| convert(&groups, &assets, g, 0))
        .collect()
}

/// Artboard `index` of `file` (in file order) converted, with the
/// artboards it nests; the others are not looked at.
pub(crate) fn artboard(
    file: &RivFile,
    index: usize,
    fonts: &HashMap<usize, Vec<u8>>,
) -> Result<Artboard, String> {
    let groups = group(file)?;
    let mut assets = assets(file);
    for (k, a) in assets.iter_mut().enumerate() {
        if a.bytes.is_none() {
            a.bytes = fonts.get(&k).map(Vec::as_slice);
        }
    }
    if index >= groups.len() {
        return Err(format!(
            "the file has {} artboards, not {}",
            groups.len(),
            index + 1
        ));
    }
    convert(&groups, &assets, index, 0)
}

/// The names of `file`'s artboards, in order.
pub(crate) fn names(file: &RivFile) -> Vec<String> {
    file.objects
        .iter()
        .filter(|o| o.ty == ARTBOARD)
        .map(|o| o.string(4).unwrap_or("").to_string())
        .collect()
}

/// A file asset (images, fonts, audio, …, in file order: the index an
/// image or a text names it by) and its embedded bytes (none: hosted on
/// Rive's CDN or supplied by the game at run time).
struct Asset<'a> {
    obj: &'a Object,
    bytes: Option<&'a [u8]>,
}

fn assets(file: &RivFile) -> Vec<Asset<'_>> {
    let mut out: Vec<Asset> = Vec::new();
    for o in &file.objects {
        if o.ty == FILE_ASSET_CONTENTS {
            if let Some(a) = out.last_mut() {
                a.bytes = o.bytes(212);
            }
        } else if is_a(o.ty, FILE_ASSET) {
            out.push(Asset {
                obj: o,
                bytes: None,
            });
        }
    }
    out
}

/// The file's font assets, by asset index: each one's name in Rive and its
/// embedded bytes (none: hosted or referenced).
pub(crate) fn font_assets(file: &RivFile) -> Vec<(usize, String, Option<&[u8]>)> {
    assets(file)
        .iter()
        .enumerate()
        .filter(|(_, a)| a.obj.ty == FONT_ASSET)
        .map(|(k, a)| (k, a.obj.string(203).unwrap_or("").to_string(), a.bytes))
        .collect()
}

/// The characters each font asset (by index) sets, over every artboard of
/// the file: every run's words at rest and as keyed, in every font its
/// style takes at rest or as keyed, and the ellipsis of a text that
/// ellipsizes. What the fonts' cooks take as their charset.
pub(crate) fn font_chars(file: &RivFile) -> Result<HashMap<usize, String>, String> {
    let mut out: HashMap<usize, String> = HashMap::new();
    for g in group(file)? {
        // Per slot: a style's fonts, a run's words and style.
        let mut fonts: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut words: HashMap<usize, Vec<String>> = HashMap::new();
        let mut styles: HashMap<usize, usize> = HashMap::new();
        let mut ellipsis: Vec<usize> = Vec::new();
        for (slot, o) in g.slots.iter().enumerate() {
            let Some(o) = o else { continue };
            match o.ty {
                TEXT_STYLE_PAINT => {
                    fonts.entry(slot).or_default().push(o.u(279) as usize);
                }
                TEXT_VALUE_RUN => {
                    words
                        .entry(slot)
                        .or_default()
                        .push(o.string(268).unwrap_or("").to_string());
                    styles.insert(slot, o.u(272) as usize);
                }
                TEXT if o.u(287) == 3 => ellipsis.push(slot),
                _ => {}
            }
        }
        let (mut object, mut key) = (0usize, 0u16);
        for o in &g.rest {
            match o.ty {
                KEYED_OBJECT => object = o.u(51) as usize,
                KEYED_PROPERTY => key = o.u(53) as u16,
                KEYFRAME_STRING if key == 268 => words
                    .entry(object)
                    .or_default()
                    .push(o.string(280).unwrap_or("").to_string()),
                KEYFRAME_ID if key == 279 => {
                    fonts.entry(object).or_default().push(o.u(122) as usize)
                }
                _ => {}
            }
        }
        for (run, style) in &styles {
            let parent = g.slots[*run].map_or(0, |o| o.u(5) as usize);
            let dots = ellipsis.contains(&parent);
            for f in fonts.get(style).into_iter().flatten() {
                let set = out.entry(*f).or_default();
                for w in words.get(run).into_iter().flatten() {
                    set.push_str(w);
                }
                if dots {
                    set.push('.');
                }
            }
        }
    }
    for set in out.values_mut() {
        let mut c: Vec<char> = set
            .chars()
            .filter(|c| !matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
            .collect();
        c.sort_unstable();
        c.dedup();
        *set = c.into_iter().collect();
    }
    Ok(out)
}

/// The file's embedded images, by asset index, with their bytes.
pub(crate) fn embedded_images(file: &RivFile) -> Vec<(usize, &[u8])> {
    assets(file)
        .iter()
        .enumerate()
        .filter(|(_, a)| a.obj.ty == IMAGE_ASSET)
        .filter_map(|(k, a)| Some((k, a.bytes?)))
        .collect()
}

/// One artboard's objects: its slot list (`None`: an empty object that
/// holds its place) and what follows its components (animations, state
/// machines), in file order.
struct Group<'a> {
    slots: Vec<Option<&'a Object>>,
    rest: Vec<&'a Object>,
}

fn group(file: &RivFile) -> Result<Vec<Group<'_>>, String> {
    let mut groups: Vec<Group> = Vec::new();
    let concrete = |ty: u16| super::defs::TYPES.iter().find(|t| t.0 == ty).map(|t| t.3);
    for o in &file.objects {
        let name = type_name(o.ty);
        if name.starts_with("Script") {
            return Err(format!(
                "{name}{}: Luau scripting is not supported (convert the script's behaviour into animations, or drive the component from TypeScript)",
                named(o)
            ));
        }
        if o.ty == ARTBOARD {
            groups.push(Group {
                slots: vec![Some(o)],
                rest: Vec::new(),
            });
            continue;
        }
        let Some(g) = groups.last_mut() else {
            if FILE_LEVEL.contains(&name.as_str()) || concrete(o.ty) == Some(false) {
                continue;
            }
            return Err(format!(
                "{name}{}: not converted (view models and other file-level data are phase 2 of the Rive import)",
                named(o)
            ));
        };
        match concrete(o.ty) {
            None => {
                return Err(format!(
                    "object type {} in artboard '{}': newer than the pinned Rive format tables (rive-runtime {}); the import does not know it",
                    o.ty,
                    g.slots[0].and_then(|a| a.string(4)).unwrap_or(""),
                    "third_party/rive"
                ));
            }
            // Read by the runtime as an empty object, which holds an
            // artboard slot before the animations.
            Some(false) => {
                if g.rest.is_empty() {
                    g.slots.push(None);
                }
            }
            Some(true) => {
                if is_a(o.ty, COMPONENT)
                    || is_a(o.ty, KEYFRAME_INTERPOLATOR)
                    || is_a(o.ty, USER_INPUT)
                {
                    if !g.rest.is_empty() && is_a(o.ty, COMPONENT) {
                        return Err(format!(
                            "{name}{} comes after the artboard's animations; the file is not as the Rive exporter writes it",
                            named(o)
                        ));
                    }
                    g.slots.push(Some(o));
                } else if FILE_LEVEL.contains(&name.as_str()) {
                    continue;
                } else {
                    g.rest.push(o);
                }
            }
        }
    }
    if groups.is_empty() {
        return Err("the file has no artboard".into());
    }
    Ok(groups)
}

fn named(o: &Object) -> String {
    match o.string(4) {
        Some(n) if !n.is_empty() => format!(" '{n}'"),
        _ => String::new(),
    }
}

/// Artboard `g`, its nested artboards converted into it.
fn convert(groups: &[Group], assets: &[Asset], g: usize, depth: u32) -> Result<Artboard, String> {
    let grp = &groups[g];
    let ab = grp.slots[0].expect("an artboard is its own slot 0");
    let ab_name = ab.string(4).unwrap_or("").to_string();
    let at = |o: &Object| format!("{}{} in artboard '{ab_name}'", type_name(o.ty), named(o));
    if depth > 8 {
        return Err(format!(
            "artboard '{ab_name}': nested artboards nest more than 8 deep (a cycle?)"
        ));
    }
    let mut a = Artboard {
        name: ab_name.clone(),
        size: Vec2::new(ab.f(7), ab.f(8)),
        origin: Vec2::new(ab.f(11), ab.f(12)),
        clip: ab.get(196).is_none_or(|_| ab.flag(196)),
        fit: Fit::Contain,
        align: Vec2::splat(0.5),
        role: "image".into(),
        label: ab_name.clone(),
        ..Artboard::default()
    };
    // Pass 1: each slot's node, its parent a slot (a child may come
    // before its parent in the file).
    let mut interpolators: HashMap<usize, &Object> = HashMap::new();
    let mut nested_ids: Vec<u32> = Vec::new();
    let mut raw: Vec<(usize, usize, &Object, Node)> = vec![(
        0,
        0,
        ab,
        Node {
            name: ab_name.clone(),
            parent: 0,
            kind: Kind::Artboard { paints: Vec::new() },
            values: Values::default(),
        },
    )];
    let mut nested_anims: Vec<(usize, &Object)> = Vec::new();
    for (slot, o) in grp.slots.iter().enumerate().skip(1) {
        let Some(o) = o else { continue };
        if is_a(o.ty, KEYFRAME_INTERPOLATOR) {
            if !matches!(o.ty, CUBIC_EASE | CUBIC_VALUE) {
                return Err(format!(
                    "{}: only cubic and value interpolators are converted",
                    at(o)
                ));
            }
            interpolators.insert(slot, o);
            continue;
        }
        let parent_slot = o.u(5) as usize;
        let mut v = Values::default();
        let kind = match o.ty {
            NODE => Kind::Group,
            SHAPE => {
                let blend = match o.u(23) {
                    3 => UiBlend::Normal,
                    14 => UiBlend::Screen,
                    15 => UiBlend::Overlay,
                    18 => UiBlend::ColorDodge,
                    21 => UiBlend::SoftLight,
                    24 => UiBlend::Multiply,
                    m => {
                        return Err(format!(
                            "{}: blend mode {m} is not converted (normal, screen, overlay, colour dodge, soft light and multiply are)",
                            at(o)
                        ));
                    }
                };
                Kind::Shape {
                    paths: Vec::new(),
                    paints: Vec::new(),
                    hidden: o.u(129) & 1 != 0,
                    blend,
                }
            }
            POINTS_PATH => Kind::Path {
                outline: Outline::Points {
                    vertices: Vec::new(),
                },
                closed: o.flag(32),
            },
            RECTANGLE | ELLIPSE | TRIANGLE | POLYGON | STAR => {
                v.width = o.f(20);
                v.height = o.f(21);
                v.origin = Vec2::new(o.f(123), o.f(124));
                let outline = match o.ty {
                    RECTANGLE => {
                        let tl = o.f(31);
                        v.corners = if o.flag(164) {
                            [tl; 4]
                        } else {
                            [tl, o.f(161), o.f(163), o.f(162)]
                        };
                        Outline::Rect
                    }
                    ELLIPSE => Outline::Ellipse,
                    TRIANGLE => Outline::Triangle,
                    POLYGON => {
                        v.corners[0] = o.f(126);
                        Outline::Polygon { points: o.u(125) }
                    }
                    _ => {
                        v.corners[0] = o.f(126);
                        v.inner = o.f(127);
                        Outline::Star { points: o.u(125) }
                    }
                };
                Kind::Path {
                    outline,
                    closed: true,
                }
            }
            STRAIGHT_VERTEX => {
                v.radius = o.f(26);
                Kind::Vertex(Vertex::Straight)
            }
            DETACHED_VERTEX => {
                v.in_rotation = o.f(84);
                v.in_distance = o.f(85);
                v.rotation = o.f(86);
                v.out_distance = o.f(87);
                Kind::Vertex(Vertex::Detached)
            }
            MIRRORED_VERTEX => {
                v.rotation = o.f(82);
                v.in_distance = o.f(83);
                Kind::Vertex(Vertex::Mirrored)
            }
            ASYMMETRIC_VERTEX => {
                v.rotation = o.f(79);
                v.in_distance = o.f(80);
                v.out_distance = o.f(81);
                Kind::Vertex(Vertex::Asymmetric)
            }
            FILL => Kind::Fill {
                rule: if o.u(40) == 1 {
                    UiFillRule::EvenOdd
                } else {
                    UiFillRule::NonZero
                },
                shader: 0,
                visible: o.flag(41),
            },
            STROKE => {
                v.thickness = o.f(47);
                if o.u(470) != 1 {
                    return Err(format!(
                        "{}: a stroke inside or outside its path is not converted (centred strokes are)",
                        at(o)
                    ));
                }
                Kind::Stroke {
                    cap: match o.u(48) {
                        1 => UiCap::Round,
                        2 => UiCap::Square,
                        _ => UiCap::Butt,
                    },
                    join: match o.u(49) {
                        1 => Join::Round,
                        2 => Join::Bevel,
                        _ => Join::Miter,
                    },
                    scaled: o.flag(50),
                    shader: 0,
                    visible: o.flag(41),
                }
            }
            SOLID_COLOR => {
                v.color = o.u(37);
                Kind::Solid
            }
            LINEAR_GRADIENT | RADIAL_GRADIENT => {
                for k in 1103..=1107u16 {
                    if o.get(k).is_some() && o.num(k) != super::read::default_of(k) {
                        return Err(format!("{}: a gradient transform is not converted", at(o)));
                    }
                }
                v.start = Vec2::new(o.f(42), o.f(33));
                v.end = Vec2::new(o.f(34), o.f(35));
                v.opacity = o.f(46);
                Kind::Gradient {
                    radial: o.ty == RADIAL_GRADIENT,
                    stops: Vec::new(),
                }
            }
            GRADIENT_STOP => {
                v.color = o.u(38);
                v.position = o.f(39);
                Kind::Stop
            }
            IMAGE => {
                let k = o.u(206) as usize;
                if o.u(23) != 3 {
                    return Err(format!(
                        "{}: an image's blend mode is not converted (normal is)",
                        at(o)
                    ));
                }
                let asset = assets
                    .get(k)
                    .filter(|x| x.obj.ty == IMAGE_ASSET)
                    .ok_or_else(|| {
                        format!(
                            "{}: names image asset {k}, which the file does not have",
                            at(o)
                        )
                    })?;
                let Some(bytes) = asset.bytes else {
                    return Err(format!(
                        "{}: its image '{}' is not embedded in the file (hosted or referenced); export it embedded",
                        at(o),
                        asset.obj.string(203).unwrap_or("")
                    ));
                };
                let image = match a.images.iter().position(|i| i.id == format!("#image{k}")) {
                    Some(i) => i,
                    None => {
                        let img = super::super::texture::decode(bytes)
                            .map_err(|e| format!("{}: its image does not decode: {e}", at(o)))?;
                        a.images.push(og_ui::rive::Image {
                            id: format!("#image{k}"),
                            size: Vec2::new(img.width as f32, img.height as f32),
                            texture: None,
                        });
                        a.images.len() - 1
                    }
                };
                v.origin = Vec2::new(o.f(380), o.f(381));
                Kind::Image {
                    image: image as u32,
                }
            }
            NESTED_ARTBOARD => {
                let id = o.u(197);
                if id as usize >= groups.len() {
                    return Err(format!(
                        "{}: names artboard {id}, which the file does not have",
                        at(o)
                    ));
                }
                nested_ids.push(id);
                Kind::Nested {
                    artboard: (nested_ids.len() - 1) as u32,
                    animations: Vec::new(),
                }
            }
            NESTED_SIMPLE_ANIMATION => {
                nested_anims.push((parent_slot, o));
                continue;
            }
            TEXT => {
                if o.u(23) != 3 {
                    return Err(format!(
                        "{}: a text's blend mode is not converted (normal is)",
                        at(o)
                    ));
                }
                if o.u(1026) != 0 {
                    return Err(format!(
                        "{}: vertical trim is not converted (Rive text's trim to cap or x-height)",
                        at(o)
                    ));
                }
                if o.u(932) != 0 {
                    return Err(format!(
                        "{}: runs from a view-model list are not converted (data binding is phase 2 of the Rive import)",
                        at(o)
                    ));
                }
                v.width = o.f(285);
                v.height = o.f(286);
                v.origin = Vec2::new(o.f(366), o.f(367));
                v.spacing = o.f(371);
                let enumerated = |key: u16, max: u32, what: &str| -> Result<u32, String> {
                    let x = o.u(key);
                    if x > max {
                        return Err(format!("{}: {what} {x} is not one Rive has", at(o)));
                    }
                    Ok(x)
                };
                let layout = TextLayout {
                    sizing: [Sizing::AutoWidth, Sizing::AutoHeight, Sizing::Fixed]
                        [enumerated(284, 2, "sizing")? as usize],
                    overflow: [
                        Overflow::Visible,
                        Overflow::Hidden,
                        Overflow::Clipped,
                        Overflow::Ellipsis,
                        Overflow::Fit,
                        Overflow::FitFontSize,
                    ][enumerated(287, 5, "overflow")? as usize],
                    align: [HAlign::Left, HAlign::Right, HAlign::Center]
                        [enumerated(281, 2, "alignment")? as usize],
                    valign: [VAlign::Top, VAlign::Bottom, VAlign::Middle]
                        [enumerated(685, 2, "vertical alignment")? as usize],
                    wrap: enumerated(683, 1, "wrap")? == 0,
                    break_words: enumerated(446, 2, "word break")? != 1,
                    baseline: o.u(377) == 1,
                    fit_from_baseline: o.flag(703),
                };
                if o.u(446) == 2 {
                    return Err(format!(
                        "{}: word break 'break all' is not converted (normal and break word are)",
                        at(o)
                    ));
                }
                Kind::Text {
                    runs: Vec::new(),
                    layout,
                }
            }
            TEXT_VALUE_RUN => {
                v.text = string_of(&mut a, o.string(268).unwrap_or(""));
                // Its style's slot, made a node index below.
                Kind::TextRun { style: o.u(272) }
            }
            TEXT_STYLE_PAINT => {
                v.font_size = o.f(274);
                v.line_height = o.f(370);
                v.letter_spacing = o.f(390);
                v.font = font_of(&mut a, assets, o.u(279) as usize, &at(o))?;
                Kind::TextStyle { paints: Vec::new() }
            }
            TEXT_STYLE_AXIS => {
                // A variable font's axis: converted at the face's default
                // (the font cooks at its defaults).
                let style = grp.slots.get(parent_slot).copied().flatten();
                let k = style.map_or(0, |st| st.u(279) as usize);
                let tag = o.u(289);
                let want = o.f(288);
                if let Some(bytes) = assets.get(k).and_then(|x| x.bytes)
                    && let Ok(face) = ttf_parser::Face::parse(bytes, 0)
                    && let Some(axis) = face
                        .tables()
                        .fvar
                        .and_then(|f| f.axes.into_iter().find(|ax| ax.tag.0 == tag))
                    && (axis.def_value - want).abs() > 1e-3
                {
                    return Err(format!(
                        "{}: sets the font's '{}' axis to {want} (its default is {}); a variable font's other instances are not converted",
                        at(o),
                        String::from_utf8_lossy(&tag.to_be_bytes()),
                        axis.def_value
                    ));
                }
                continue;
            }
            _ => {
                return Err(format!(
                    "{}: not converted by phase 1 of the Rive import (shapes, paths, paints, gradients, nested artboards and linear animations are)",
                    at(o)
                ));
            }
        };
        // Every node is a transform component except paints, colours and
        // vertices, which have their own numbers.
        if is_a(o.ty, NODE) || o.ty == NESTED_ARTBOARD {
            v.x = o.f(13);
            v.y = o.f(14);
            v.rotation = o.f(15);
            v.scale_x = o.f(16);
            v.scale_y = o.f(17);
            v.opacity = o.f(18);
        } else if matches!(
            o.ty,
            STRAIGHT_VERTEX | DETACHED_VERTEX | MIRRORED_VERTEX | ASYMMETRIC_VERTEX
        ) {
            v.x = o.f(24);
            v.y = o.f(25);
        }
        raw.push((
            slot,
            parent_slot,
            o,
            Node {
                name: o.string(4).unwrap_or("").to_string(),
                parent: 0,
                kind,
                values: v,
            },
        ));
    }
    // Pass 2: parents before children (by depth, then file order).
    let index: HashMap<usize, usize> = raw.iter().enumerate().map(|(i, r)| (r.0, i)).collect();
    let mut level = vec![0u32; raw.len()];
    for i in 1..raw.len() {
        let (mut j, mut d) = (i, 0u32);
        while j != 0 {
            let Some(&p) = index.get(&raw[j].1) else {
                return Err(format!(
                    "{}: its parent (object {}) is not a converted object",
                    at(raw[j].2),
                    raw[j].1
                ));
            };
            d += 1;
            if d as usize > raw.len() {
                return Err(format!("{}: its parents form a cycle", at(raw[i].2)));
            }
            j = p;
        }
        level[i] = d;
    }
    let mut order: Vec<usize> = (0..raw.len()).collect();
    order.sort_by_key(|&i| (level[i], raw[i].0));
    let mut node_of: HashMap<usize, u32> = HashMap::new();
    for (new, &i) in order.iter().enumerate() {
        node_of.insert(raw[i].0, new as u32);
    }
    let objects: Vec<&Object> = order.iter().map(|&i| raw[i].2).collect();
    let slots: Vec<usize> = order.iter().map(|&i| raw[i].0).collect();
    let parents: Vec<usize> = order.iter().map(|&i| raw[i].1).collect();
    let mut taken: Vec<Option<Node>> = raw.into_iter().map(|r| Some(r.3)).collect();
    for (new, &i) in order.iter().enumerate() {
        let mut n = taken[i].take().expect("each once");
        n.parent = if new == 0 { 0 } else { node_of[&parents[new]] };
        a.nodes.push(n);
    }
    // Attach each to what owns it, in file order (paints, paths and
    // vertices keep the file's order).
    let mut by_slot: Vec<u32> = (1..a.nodes.len() as u32).collect();
    by_slot.sort_by_key(|&i| slots[i as usize]);
    for i in by_slot {
        let o = objects[i as usize];
        let parent = a.nodes[i as usize].parent;
        let kind = a.nodes[i as usize].kind.clone();
        let owner = &mut a.nodes[parent as usize].kind;
        let bad = match (&kind, owner) {
            (Kind::Path { .. }, _) => {
                // A path belongs to the nearest shape above it.
                let mut s = parent;
                loop {
                    if let Kind::Shape { paths, .. } = &mut a.nodes[s as usize].kind {
                        paths.push(i);
                        break false;
                    }
                    if s == 0 {
                        break true;
                    }
                    s = a.nodes[s as usize].parent;
                }
            }
            (
                Kind::Vertex(_),
                Kind::Path {
                    outline: Outline::Points { vertices },
                    ..
                },
            ) => {
                vertices.push(i);
                false
            }
            (Kind::Fill { .. } | Kind::Stroke { .. }, Kind::Shape { paints, .. })
            | (Kind::Fill { .. } | Kind::Stroke { .. }, Kind::TextStyle { paints })
            | (Kind::Fill { .. } | Kind::Stroke { .. }, Kind::Artboard { paints }) => {
                paints.push(i);
                false
            }
            (
                Kind::Solid | Kind::Gradient { .. },
                Kind::Fill { shader, .. } | Kind::Stroke { shader, .. },
            ) => {
                *shader = i;
                false
            }
            (Kind::Stop, Kind::Gradient { stops, .. }) => {
                stops.push(i);
                false
            }
            (Kind::TextRun { .. }, Kind::Text { runs, .. }) => {
                runs.push(i);
                false
            }
            (Kind::TextStyle { .. }, Kind::Text { .. }) => false,
            (
                Kind::Group
                | Kind::Shape { .. }
                | Kind::Nested { .. }
                | Kind::Image { .. }
                | Kind::Text { .. },
                k,
            ) => !matches!(k, Kind::Artboard { .. } | Kind::Group | Kind::Shape { .. }),
            _ => true,
        };
        if bad {
            return Err(format!("{}: not under an object it can belong to", at(o)));
        }
    }
    // A run's style: its slot made a node, a style of the same text.
    for i in 0..a.nodes.len() {
        let Kind::TextRun { style } = a.nodes[i].kind else {
            continue;
        };
        let node = node_of.get(&(style as usize)).copied();
        let ok = node.is_some_and(|n| {
            matches!(a.nodes[n as usize].kind, Kind::TextStyle { .. })
                && a.nodes[n as usize].parent == a.nodes[i].parent
        });
        let Some(n) = node.filter(|_| ok) else {
            return Err(format!(
                "{}: its style (object {style}) is not a style of its text",
                at(objects[i])
            ));
        };
        a.nodes[i].kind = Kind::TextRun { style: n };
    }
    for (parent_slot, o) in nested_anims {
        let n = node_of.get(&parent_slot).copied();
        let Some(Kind::Nested { animations, .. }) = n.map(|n| &mut a.nodes[n as usize].kind) else {
            return Err(format!("{}: not on a nested artboard", at(o)));
        };
        animations.push(NestedAnimation {
            animation: o.u(198),
            speed: o.f(199),
            playing: o.flag(201),
        });
    }
    // A paint without a colour draws Rive's default grey; a shape's paints
    // are checked to have one.
    for n in &a.nodes {
        if let Kind::Fill { shader: 0, .. } | Kind::Stroke { shader: 0, .. } = n.kind {
            return Err(format!(
                "a paint '{}' in artboard '{ab_name}' has no colour or gradient",
                n.name
            ));
        }
    }
    // Draw order: rive-runtime draws its drawables last added first.
    let mut draws: Vec<u32> = (0..a.nodes.len() as u32)
        .filter(|&i| {
            match a.nodes[i as usize].kind {
                Kind::Shape { .. } | Kind::Nested { .. } => true,
                // A hidden image or text (drawableFlags' Hidden) is not drawn.
                Kind::Image { .. } | Kind::Text { .. } => objects[i as usize].u(129) & 1 == 0,
                _ => false,
            }
        })
        .collect();
    draws.sort_by_key(|&i| std::cmp::Reverse(slots[i as usize]));
    a.draws = draws;
    animations(&mut a, grp, assets, &node_of, &interpolators)?;
    for id in nested_ids {
        a.nested
            .push(convert(groups, assets, id as usize, depth + 1)?);
    }
    let nested = a.nested.clone();
    for n in &a.nodes {
        if let Kind::Nested {
            artboard,
            animations,
        } = &n.kind
        {
            let count = nested[*artboard as usize].animations.len();
            if let Some(x) = animations.iter().find(|x| x.animation as usize >= count) {
                return Err(format!(
                    "nested artboard '{}' in artboard '{ab_name}': plays animation {}, which its artboard does not have",
                    n.name, x.animation
                ));
            }
        }
    }
    og_ui::rive::check(&a)?;
    Ok(a)
}

/// The index of `text` in the artboard's words, added once.
fn string_of(a: &mut Artboard, text: &str) -> u32 {
    match a.strings.iter().position(|s| s == text) {
        Some(i) => i as u32,
        None => {
            a.strings.push(text.to_string());
            (a.strings.len() - 1) as u32
        }
    }
}

/// The artboard's font of asset `k` (added once, its id `#font<k>` until
/// the cook places it), or [`NO_FONT`] when asset `k` is not a font: Rive
/// draws a style without one as nothing. A font without its bytes is an
/// error naming it.
fn font_of(a: &mut Artboard, assets: &[Asset], k: usize, at: &str) -> Result<u32, String> {
    let Some(asset) = assets.get(k).filter(|x| x.obj.ty == FONT_ASSET) else {
        return Ok(NO_FONT);
    };
    let id = format!("#font{k}");
    if let Some(i) = a.fonts.iter().position(|f| f.id == id) {
        return Ok(i as u32);
    }
    let name = asset.obj.string(203).unwrap_or("");
    let Some(bytes) = asset.bytes else {
        return Err(format!(
            "{at}: its font '{name}' is not embedded in the file (hosted or referenced); export it embedded, or name its file in the import settings (font \"{name}\" {{ source \"fonts/<file>.ttf\" }})"
        ));
    };
    let face = ttf_parser::Face::parse(bytes, 0)
        .map_err(|e| format!("{at}: its font '{name}' does not parse: {e}"))?;
    let em = f32::from(face.units_per_em()).max(1.0);
    a.fonts.push(og_ui::rive::Font {
        id,
        ascent: f32::from(face.ascender()) / em,
        descent: -f32::from(face.descender()) / em,
        font: None,
    });
    Ok((a.fonts.len() - 1) as u32)
}

/// The prop a keyed property key sets on a node of `kind`.
fn prop_of(key: u16, kind: &Kind, linked: bool) -> Option<Vec<Prop>> {
    let one = |p| Some(vec![p]);
    match (key, kind) {
        (13, _) => one(Prop::X),
        (14, _) => one(Prop::Y),
        (15, _) => one(Prop::Rotation),
        (16, _) => one(Prop::ScaleX),
        (17, _) => one(Prop::ScaleY),
        (18, _) => one(Prop::Opacity),
        (380, Kind::Image { .. }) => one(Prop::OriginX),
        (381, Kind::Image { .. }) => one(Prop::OriginY),
        (20, Kind::Path { .. }) => one(Prop::Width),
        (21, Kind::Path { .. }) => one(Prop::Height),
        (123, Kind::Path { .. }) => one(Prop::OriginX),
        (124, Kind::Path { .. }) => one(Prop::OriginY),
        (31, Kind::Path { .. }) if linked => Some((0..4).map(Prop::Corner).collect()),
        (31 | 126, Kind::Path { .. }) => one(Prop::Corner(0)),
        (161, Kind::Path { .. }) => one(Prop::Corner(1)),
        (163, Kind::Path { .. }) => one(Prop::Corner(2)),
        (162, Kind::Path { .. }) => one(Prop::Corner(3)),
        (127, Kind::Path { .. }) => one(Prop::Inner),
        (24, Kind::Vertex(_)) => one(Prop::X),
        (25, Kind::Vertex(_)) => one(Prop::Y),
        (26, Kind::Vertex(Vertex::Straight)) => one(Prop::Radius),
        (84, Kind::Vertex(Vertex::Detached)) => one(Prop::InRotation),
        (85, Kind::Vertex(Vertex::Detached)) => one(Prop::InDistance),
        (86, Kind::Vertex(Vertex::Detached)) => one(Prop::Rotation),
        (87, Kind::Vertex(Vertex::Detached)) => one(Prop::OutDistance),
        (82, Kind::Vertex(Vertex::Mirrored)) => one(Prop::Rotation),
        (83, Kind::Vertex(Vertex::Mirrored)) => one(Prop::InDistance),
        (79, Kind::Vertex(Vertex::Asymmetric)) => one(Prop::Rotation),
        (80, Kind::Vertex(Vertex::Asymmetric)) => one(Prop::InDistance),
        (81, Kind::Vertex(Vertex::Asymmetric)) => one(Prop::OutDistance),
        (47, Kind::Stroke { .. }) => one(Prop::Thickness),
        (37, Kind::Solid) | (38, Kind::Stop) => one(Prop::Color),
        (39, Kind::Stop) => one(Prop::Position),
        (42, Kind::Gradient { .. }) => one(Prop::StartX),
        (33, Kind::Gradient { .. }) => one(Prop::StartY),
        (34, Kind::Gradient { .. }) => one(Prop::EndX),
        (35, Kind::Gradient { .. }) => one(Prop::EndY),
        (46, Kind::Gradient { .. }) => one(Prop::Opacity),
        (268, Kind::TextRun { .. }) => one(Prop::Text),
        (274, Kind::TextStyle { .. }) => one(Prop::FontSize),
        (279, Kind::TextStyle { .. }) => one(Prop::Font),
        (370, Kind::TextStyle { .. }) => one(Prop::LineHeight),
        (390, Kind::TextStyle { .. }) => one(Prop::LetterSpacing),
        (285, Kind::Text { .. }) => one(Prop::Width),
        (286, Kind::Text { .. }) => one(Prop::Height),
        (366, Kind::Text { .. }) => one(Prop::OriginX),
        (367, Kind::Text { .. }) => one(Prop::OriginY),
        (371, Kind::Text { .. }) => one(Prop::Spacing),
        _ => None,
    }
}

/// The artboard's linear animations, and what plays by default: the
/// default state machine reduced to the animation it plays, or the first
/// animation when it has none.
fn animations(
    a: &mut Artboard,
    grp: &Group,
    assets: &[Asset],
    node_of: &HashMap<usize, u32>,
    interpolators: &HashMap<usize, &Object>,
) -> Result<(), String> {
    let ab_name = a.name.clone();
    let mut fps = 60.0f32;
    let mut track: Option<(u32, Vec<Prop>)> = None;
    let mut machines: Vec<(String, Option<u32>)> = Vec::new();
    // The current machine's layer: its states (animation ids; None for
    // any, entry and exit) and the entry's transitions.
    let mut in_machine = false;
    let mut states: Vec<(u16, Option<u32>)> = Vec::new();
    let mut transitions: Vec<(usize, u32)> = Vec::new();
    let mut layers = 0;
    let finish = |machines: &mut Vec<(String, Option<u32>)>,
                  states: &[(u16, Option<u32>)],
                  transitions: &[(usize, u32)],
                  layers: u32|
     -> Result<(), String> {
        let Some((name, play)) = machines.last_mut() else {
            return Ok(());
        };
        let bad = || {
            Err(format!(
                "state machine '{name}' in artboard '{ab_name}': not converted (state machines beyond one layer whose entry plays one animation are phase 2 of the Rive import)"
            ))
        };
        if layers > 1 {
            return bad();
        }
        // No layer, or a layer that never leaves its entry: it plays nothing.
        if layers == 0 || transitions.is_empty() {
            return Ok(());
        }
        let entry = states.iter().position(|s| s.0 == ENTRY_STATE);
        let from_entry: Vec<_> = transitions.iter().filter(|t| Some(t.0) == entry).collect();
        if transitions.len() != from_entry.len() || from_entry.len() != 1 {
            return bad();
        }
        match states.get(from_entry[0].1 as usize) {
            Some((ANIMATION_STATE, Some(anim))) => *play = Some(*anim),
            _ => return bad(),
        }
        Ok(())
    };
    for o in &grp.rest {
        let at = || format!("{}{} in artboard '{ab_name}'", type_name(o.ty), named(o));
        match o.ty {
            LINEAR_ANIMATION => {
                if in_machine {
                    return Err(format!("{}: after a state machine", at()));
                }
                fps = o.u(56).max(1) as f32;
                let work = o.flag(62).then(|| (o.u(60), o.u(61)));
                a.animations.push(Animation {
                    name: o.string(55).unwrap_or("").to_string(),
                    fps: o.u(56),
                    duration: o.u(57),
                    speed: o.f(58),
                    looping: match o.u(59) {
                        1 => Loop::Loop,
                        2 => Loop::PingPong,
                        _ => Loop::OneShot,
                    },
                    work,
                    tracks: Vec::new(),
                });
                track = None;
            }
            KEYED_OBJECT => {
                let slot = o.u(51) as usize;
                let Some(&n) = node_of.get(&slot) else {
                    return Err(format!(
                        "{}: keys object {slot}, which is not a converted object",
                        at()
                    ));
                };
                track = Some((n, Vec::new()));
            }
            KEYED_PROPERTY => {
                let Some((n, props)) = &mut track else {
                    return Err(format!("{}: outside a keyed object", at()));
                };
                let node = &a.nodes[*n as usize];
                // A rectangle whose corners are linked keys them together.
                let linked = matches!(
                    node.kind,
                    Kind::Path {
                        outline: Outline::Rect,
                        ..
                    }
                ) && node
                    .values
                    .corners
                    .iter()
                    .all(|c| *c == node.values.corners[0]);
                let key = o.u(53) as u16;
                let Some(p) = prop_of(key, &node.kind, linked) else {
                    return Err(format!(
                        "an animation of artboard '{ab_name}' keys {}.{} of '{}': not converted by phase 1 (transform, opacity, colours and the paths' numbers are)",
                        super::defs::PROPERTIES
                            .iter()
                            .find(|x| x.0 == key)
                            .map_or("?", |x| x.1),
                        super::defs::PROPERTIES
                            .iter()
                            .find(|x| x.0 == key)
                            .map_or("?", |x| x.2),
                        node.name
                    ));
                };
                let anim = a
                    .animations
                    .last_mut()
                    .ok_or_else(|| format!("{}: outside an animation", at()))?;
                for q in &p {
                    anim.tracks.push(Track {
                        node: *n,
                        prop: *q,
                        keys: Vec::new(),
                    });
                }
                *props = p;
            }
            KEYFRAME_DOUBLE | KEYFRAME_COLOR | KEYFRAME_STRING | KEYFRAME_ID => {
                let (Some((_, props)), false) = (&track, a.animations.is_empty()) else {
                    return Err(format!("{}: outside a keyed property", at()));
                };
                let ease = match interpolators.get(&(o.u(69) as usize)) {
                    Some(i) if o.get(69).is_some() && o.u(68) != 0 => {
                        let (x1, y1, x2, y2) = (i.f(63), i.f(64), i.f(65), i.f(66));
                        if i.ty == CUBIC_VALUE {
                            if o.ty == KEYFRAME_COLOR {
                                return Err(format!("{}: a value interpolator on a colour", at()));
                            }
                            Ease::CubicValue { x1, y1, x2, y2 }
                        } else {
                            Ease::Cubic { x1, y1, x2, y2 }
                        }
                    }
                    _ if o.u(68) == 0 => Ease::Hold,
                    _ => Ease::Linear,
                };
                let props = props.clone();
                let n = props.len();
                for q in &props {
                    let want = match q {
                        Prop::Color => KEYFRAME_COLOR,
                        Prop::Text => KEYFRAME_STRING,
                        Prop::Font => KEYFRAME_ID,
                        _ => KEYFRAME_DOUBLE,
                    };
                    if want != o.ty {
                        return Err(format!("{}: of the wrong type for its property", at()));
                    }
                }
                // Words and fonts hold until the next key.
                let (value, ease) = match o.ty {
                    KEYFRAME_DOUBLE => (o.f(70), ease),
                    KEYFRAME_STRING => {
                        (string_of(a, o.string(280).unwrap_or("")) as f32, Ease::Hold)
                    }
                    KEYFRAME_ID => {
                        let f = font_of(a, assets, o.u(122) as usize, &at())?;
                        if f == NO_FONT {
                            return Err(format!(
                                "{}: keys a style's font to asset {}, which is not a font",
                                at(),
                                o.u(122)
                            ));
                        }
                        (f as f32, Ease::Hold)
                    }
                    _ => (0.0, ease),
                };
                let key = Key {
                    seconds: o.u(67) as f32 / fps,
                    value,
                    color: if o.ty == KEYFRAME_COLOR { o.u(88) } else { 0 },
                    ease,
                };
                let anim = a.animations.last_mut().expect("checked above");
                let len = anim.tracks.len();
                for t in &mut anim.tracks[len - n..] {
                    t.keys.push(key);
                }
            }
            STATE_MACHINE => {
                finish(&mut machines, &states, &transitions, layers)?;
                in_machine = true;
                machines.push((o.string(55).unwrap_or("").to_string(), None));
                states.clear();
                transitions.clear();
                layers = 0;
            }
            SM_LAYER if in_machine => {
                layers += 1;
                states.clear();
                transitions.clear();
            }
            ANY_STATE | ENTRY_STATE | EXIT_STATE if in_machine => states.push((o.ty, None)),
            ANIMATION_STATE if in_machine => {
                let id = o.u(149);
                if id as usize >= a.animations.len() {
                    return Err(format!(
                        "{}: plays animation {id}, which the artboard does not have",
                        at()
                    ));
                }
                states.push((o.ty, Some(id)));
            }
            STATE_TRANSITION if in_machine => {
                let Some(from) = states.len().checked_sub(1) else {
                    return Err(format!("{}: before any state", at()));
                };
                transitions.push((from, o.u(151)));
            }
            _ if in_machine => {
                return Err(format!(
                    "{}: in state machine '{}', not converted (inputs, conditions, listeners and events are phase 2 of the Rive import)",
                    at(),
                    machines.last().map_or("", |m| m.0.as_str())
                ));
            }
            _ => {
                return Err(format!(
                    "{}: not converted by phase 1 of the Rive import",
                    at()
                ));
            }
        }
    }
    finish(&mut machines, &states, &transitions, layers)?;
    for anim in &mut a.animations {
        for t in &mut anim.tracks {
            t.keys.sort_by(|x, y| x.seconds.total_cmp(&y.seconds));
        }
    }
    a.machines = machines;
    let default = grp.slots[0].map_or(0, |ab| ab.u(236)) as usize;
    a.play = match a.machines.get(default).or(a.machines.first()) {
        Some((_, play)) => *play,
        None => (!a.animations.is_empty()).then_some(0),
    };
    Ok(())
}
