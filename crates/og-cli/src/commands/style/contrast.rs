//! `og style check`'s contrast facts. A pack's sheets are drawn without a
//! GPU, in both schemes: the core-control sheet's pages (`ui-sheet` or
//! `sheets/style` of the engine package, every state forced) and every
//! built-in document from its `fixture:default`. Each text the frames
//! draw is read against what is drawn under it, composited in linear
//! light as og-render blends: fills, gradients (sampled where the text
//! is), images (their texels there), glass (its tint) and layers' opacity,
//! at six points of the text's box, the worst point kept. What the UI
//! leaves uncovered is the scene's: it is read over black and over white,
//! and the worse is kept. For each pair the facts are WCAG 2's ratio and
//! APCA's Lc; the bars are the screens guide's (`og api show screens`).
//!
//! The colour-vision facts: how far apart, in OKLab, the pack's team
//! colours and its state colours stay under protanopia, deuteranopia and
//! tritanopia (og-draw's simulation, full strength).

use og_draw::{
    ColorFilter, ColorVision, ColorVisionFilter, TextureHandle, UiGeometry, UiGradient,
    UiGradientKind, UiMode, UiPaint, UiSpread,
};
use og_ui::{DrawCmd, DrawItem, Srgba};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/// The frames a document is built for before it is read: past the longest
/// intro and transition a built-in document declares (300 ms at 60 Hz).
const FRAMES: usize = 30;
const SIZE: (f32, f32) = (1280.0, 720.0);

/// One text and what it is drawn on.
pub struct Pair {
    /// `sheet:controls`, `src/ui/components/menu.svelte`.
    pub sheet: String,
    pub scheme: &'static str,
    pub text: String,
    /// The element that shows it (its probe path), when one does.
    pub path: String,
    pub ink: Srgba,
    pub ground: Srgba,
    /// What the ground is made of: `solid`, `gradient`, `image`, `glass`,
    /// `scene` (the UI covers it only partly, read over black and white).
    pub over: &'static str,
    pub disabled: bool,
    /// The worst WCAG 2 ratio of the ink on its halo (text-shadow and
    /// outline over the ground, a pixel out from the glyphs' edge), when it
    /// has one.
    pub halo: Option<f32>,
    pub wcag: f32,
    pub apca: f32,
}

impl Pair {
    pub fn json(&self) -> Value {
        json!({
            "sheet": self.sheet, "scheme": self.scheme, "text": self.text, "path": self.path,
            "ink": hex(self.ink), "ground": hex(self.ground), "over": self.over,
            "disabled": self.disabled, "halo": self.halo.map(round2),
            "ratio": round2(self.wcag), "apca": round1(self.apca),
        })
    }
}

fn round2(v: f32) -> f64 {
    (f64::from(v) * 100.0).round() / 100.0
}
fn round1(v: f32) -> f64 {
    (f64::from(v) * 10.0).round() / 10.0
}

pub fn hex(c: Srgba) -> String {
    let [r, g, b] = c.rgb;
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// The documents drawn: the core sheet's pages, then every built-in
/// document, each with the props it is drawn from, and the English strings.
struct Sheets {
    docs: Vec<og_ui::UiDocument>,
    draws: Vec<(String, String, Value)>,
    messages: Messages,
}

struct Messages(BTreeMap<String, String>);

impl og_ui::bind::Env for Messages {
    fn translate(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
}

fn sheets() -> Result<Sheets, String> {
    let ui = og_cook::cook::ui_components::ui_dir()
        .ok_or("the engine's built-in documents are not found (OG_ENGINE_PACKAGE)")?;
    let package = ui.parent().and_then(Path::parent).unwrap_or(ui);
    let mut files = Vec::new();
    walk(ui, &mut files)?;
    let sheet = ["ui-sheet", "sheets/style"]
        .iter()
        .map(|d| package.join(d).join("assets/ui"))
        .find(|d| d.join("sheet.svelte").is_file());
    if let Some(dir) = &sheet {
        files.push(dir.join("sheet.svelte"));
        files.push(dir.join("sheet.en.ogt"));
    }
    files.sort();
    let (mut docs, mut draws, mut messages) = (Vec::new(), Vec::new(), BTreeMap::new());
    for path in files {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if name.ends_with(".en.ogt") {
            let table = og_i18n::StringTable::parse(&text)
                .map_err(|e| format!("{}: {e}", path.display()))?;
            messages.extend(table.entries.into_iter().map(|(k, (v, _))| (k, v)));
        } else if name.ends_with(".svelte") {
            let doc = og_ui::UiDocument::parse(name, &text)
                .map_err(|e| format!("{}: {}", path.display(), e.join("; ")))?;
            let core = sheet.as_ref().is_some_and(|d| path.starts_with(d));
            let label = if core {
                "sheet".to_string()
            } else {
                path.strip_prefix(package)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/")
            };
            for (fixture, props) in &doc.fixtures {
                if core || fixture == "default" {
                    let page = props["page"].as_str().unwrap_or(fixture);
                    let sheet = if core {
                        format!("sheet:{page}")
                    } else {
                        label.clone()
                    };
                    draws.push((sheet, doc.name.clone(), props.clone()));
                }
            }
            docs.push(doc);
        }
    }
    Ok(Sheets {
        docs,
        draws,
        messages: Messages(messages),
    })
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        let path = entry.path();
        if ty.is_dir() {
            walk(&path, out)?;
        } else if ty.is_file()
            && path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".svelte") || n.ends_with(".en.ogt"))
        {
            out.push(path);
        }
    }
    Ok(())
}

/// The texture assets a pack's `url()`s name, by the path the cascade
/// resolves them to, decoded the first time a ground reads them.
pub struct Textures {
    files: Vec<(String, PathBuf)>,
    decoded: HashMap<u32, Option<(u32, u32, Vec<u8>)>>,
}

impl Textures {
    /// The images under `directory` as `<namespace>/<file>` and those of
    /// the shared styles directory as `@styles/<file>`.
    pub fn of(directory: Option<&Path>, namespace: &str) -> Textures {
        let mut files = Vec::new();
        let mut add = |root: &Path, prefix: &str| {
            let mut found = Vec::new();
            images(root, &mut found);
            for p in found {
                let rel = p
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push((format!("{prefix}/{rel}"), p));
            }
        };
        if let Some(d) = directory {
            add(d, namespace);
        }
        if let Some(d) = og_cook::cook::look::styles_dir() {
            add(d, "@styles");
        }
        Textures {
            files,
            decoded: HashMap::new(),
        }
    }

    /// The same files with nothing decoded yet, for another worker.
    fn fresh(&self) -> Textures {
        Textures {
            files: self.files.clone(),
            decoded: HashMap::new(),
        }
    }

    fn define(&self, ui: &mut og_ui::Ui) {
        for (k, (id, _)) in self.files.iter().enumerate() {
            ui.define_texture(id, TextureHandle::from_index(k + 1));
        }
    }

    /// The straight sRGB texel at `uv` of texture `t`, if it is one of these.
    fn texel(&mut self, t: TextureHandle, uv: [f32; 2]) -> Option<[f32; 4]> {
        let files = &self.files;
        let image = self.decoded.entry(t.0).or_insert_with(|| {
            let (_, path) = files.get(t.index().checked_sub(1)?)?;
            let bytes = std::fs::read(path).ok()?;
            let i = og_cook::cook::decode_image(&bytes).ok()?;
            Some((i.width, i.height, i.rgba))
        });
        let (w, h, rgba) = image.as_ref()?;
        let x = ((uv[0].clamp(0.0, 1.0) * *w as f32) as u32).min(w - 1);
        let y = ((uv[1].clamp(0.0, 1.0) * *h as f32) as u32).min(h - 1);
        let at = ((y * w + x) * 4) as usize;
        let p = &rgba[at..at + 4];
        let lin = |v: u8| og_core::color::srgb8_to_linear(v);
        Some([lin(p[0]), lin(p[1]), lin(p[2]), f32::from(p[3]) / 255.0])
    }
}

fn images(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(ty) = entry.file_type() else { continue };
        let path = entry.path();
        if ty.is_dir()
            && !matches!(
                entry.file_name().to_str(),
                Some(".og" | "cooked" | "og-out")
            )
        {
            images(&path, out);
        } else if ty.is_file()
            && path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                matches!(
                    e.to_ascii_lowercase().as_str(),
                    "png" | "jpg" | "jpeg" | "webp" | "svg" | "ktx2" | "tga"
                )
            })
        {
            out.push(path);
        }
    }
}

/// Every text pair the sheets draw under `pack` (none: the core default).
///
/// Each (scheme, document) is built and read on its own, so they run on
/// every core (each worker decodes the textures it reads itself); the
/// pairs come out in the order a single pass would give them.
pub fn pairs(
    pack: Option<&og_ui::style_pack::StylePack>,
    textures: &Textures,
) -> Result<Vec<Pair>, String> {
    let sheets = sheets()?;
    let jobs: Vec<(&'static str, &(String, String, Value))> = ["dark", "light"]
        .into_iter()
        .flat_map(|scheme| sheets.draws.iter().map(move |d| (scheme, d)))
        .collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let workers = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(jobs.len());
    let mut done: Vec<(usize, Result<Vec<Pair>, String>)> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                s.spawn(|| {
                    let mut textures = textures.fresh();
                    let mut mine = Vec::new();
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(&(scheme, (sheet, document, props))) = jobs.get(i) else {
                            break;
                        };
                        let theme = if scheme == "dark" {
                            og_ui::Theme::dark()
                        } else {
                            og_ui::Theme::light()
                        };
                        let pairs = drawn(
                            theme,
                            pack,
                            &sheets,
                            &textures,
                            document,
                            props,
                            sheet.starts_with("sheet:"),
                        )
                        .map(|ui| {
                            let mut out = Vec::new();
                            read(&ui, sheet, scheme, &mut textures, &mut out);
                            out
                        })
                        .map_err(|e| format!("{sheet}: {e}"));
                        mine.push((i, pairs));
                    }
                    mine
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("a contrast worker panicked"))
            .collect()
    });
    done.sort_by_key(|(i, _)| *i);
    let mut out = Vec::new();
    for (_, pairs) in done {
        out.extend(pairs?);
    }
    Ok(out)
}

/// `document` mounted from `props` in a UI of `theme` dressed by `pack`,
/// built [`FRAMES`] times (the core sheet's states forced after the first).
fn drawn(
    theme: og_ui::Theme,
    pack: Option<&og_ui::style_pack::StylePack>,
    sheets: &Sheets,
    textures: &Textures,
    document: &str,
    props: &Value,
    force: bool,
) -> Result<og_ui::Ui, String> {
    // Text needs a font with a handle to be drawn; its metrics are fixed.
    let font = og_ui::UiFont {
        handle: Some(og_draw::FontHandle::from_index(0)),
        ..og_ui::UiFont::unrendered(0.5)
    };
    let mut ui = og_ui::Ui::new(og_ui::UiDesc {
        theme,
        fonts: og_ui::Fonts {
            body: font,
            ..Default::default()
        },
        ..Default::default()
    });
    textures.define(&mut ui);
    if let Some(pack) = pack {
        ui.set_style_pack(pack)?;
    }
    for d in &sheets.docs {
        ui.define_document(d.clone());
    }
    // A camera 6 m back and 3 m up looking at the origin: what a fixture
    // anchors in the world near it is on screen.
    ui.set_world_view(Some(og_ui::WorldView::look_at(
        og_core::glam::Vec3::new(0.0, 3.0, 6.0),
        og_core::glam::Vec3::ZERO,
        og_core::glam::Vec3::Y,
        50.0,
        SIZE.0 / SIZE.1,
        0.1,
        100.0,
    )));
    ui.mount_with_values(
        og_ui::reactive::SourceId::new("check", "sheet"),
        document,
        props,
    )?;
    let viewport = og_ui::Viewport::new(SIZE.0, SIZE.1);
    for frame in 0..FRAMES {
        ui.begin(og_ui::InputFrame::default(), viewport);
        ui.build_retained_documents(&sheets.messages)?;
        ui.end();
        if frame == 0 && force {
            force_states(&mut ui);
        }
    }
    Ok(ui)
}

/// The core sheet draws each state of a control in a cell named after it
/// (`primary-hover`, `field-focus`): those cells and what is under them
/// are held in that state, as `og ui sheet` holds them.
fn force_states(ui: &mut og_ui::Ui) {
    let cells: Vec<(String, &str)> = ui
        .elements()
        .iter()
        .filter_map(|e| {
            let id = e.path.rsplit('/').next()?;
            let state = [
                ("-hover", "hover"),
                ("-pressed", "pressed"),
                ("-focus", "focused"),
            ]
            .into_iter()
            .find_map(|(end, s)| id.ends_with(end).then_some(s))?;
            Some((e.path.clone(), state))
        })
        .collect();
    let all: Vec<String> = ui.elements().iter().map(|e| e.path.clone()).collect();
    for (cell, state) in cells {
        let under = format!("{cell}/");
        for path in all.iter().filter(|p| **p == cell || p.starts_with(&under)) {
            ui.force_state(path, "interaction", Some(state));
        }
    }
}

/// Linear RGB, opaque.
type Rgb = [f32; 3];

fn over(top: [f32; 4], below: Rgb) -> Rgb {
    [0, 1, 2].map(|i| top[i] * top[3] + below[i] * (1.0 - top[3]))
}

fn srgb(c: Rgb) -> Srgba {
    Srgba::from_linear([c[0], c[1], c[2], 1.0])
}

/// What one draw command lays at a screen point: its linear colour
/// (straight alpha) and what it is.
fn paint_at(
    item: &DrawItem,
    p: (f32, f32),
    textures: &mut Textures,
) -> Option<([f32; 4], &'static str)> {
    let local = match item.transform.invert() {
        Some(m) if !item.transform.is_identity() => m.apply(p.0, p.1),
        _ => p,
    };
    let inside = |r: &og_ui::Rect| {
        local.0 >= r.x && local.0 <= r.x + r.width && local.1 >= r.y && local.1 <= r.y + r.height
    };
    let frac = |r: &og_ui::Rect| {
        [
            (local.0 - r.x) / r.width.max(1e-3),
            (local.1 - r.y) / r.height.max(1e-3),
        ]
    };
    match &item.cmd {
        DrawCmd::Quad { rect, color, .. } if inside(rect) => Some((*color, "solid")),
        DrawCmd::Image {
            rect,
            texture,
            uv,
            color,
            ..
        } if inside(rect) => match texture {
            None => Some((*color, "solid")),
            Some(t) => {
                let f = frac(rect);
                let at = [
                    uv[0] + f[0] * (uv[2] - uv[0]),
                    uv[1] + f[1] * (uv[3] - uv[1]),
                ];
                let texel = textures.texel(*t, at)?;
                Some((mul(texel, *color), "image"))
            }
        },
        DrawCmd::Glass {
            rect, tint, alpha, ..
        } if inside(rect) => {
            let mut t = *tint;
            t[3] *= alpha;
            Some((t, "glass"))
        }
        DrawCmd::Shape { shape, .. } if shape.mode == UiMode::Fill => {
            if matches!(shape.geometry, UiGeometry::Polyline { .. }) {
                return None;
            }
            // In the shape's frame: its 2×2 about its centre undone.
            let [a, b, c, d] = shape.transform;
            let (dx, dy) = (local.0 - shape.center.x, local.1 - shape.center.y);
            let det = a * d - b * c;
            if det.abs() < 1e-9 {
                return None;
            }
            let (x, y) = ((d * dx - c * dy) / det, (-b * dx + a * dy) / det);
            if x.abs() > shape.half.x || y.abs() > shape.half.y {
                return None;
            }
            let (w, h) = (2.0 * shape.half.x, 2.0 * shape.half.y);
            let f = [
                (x + shape.half.x) / w.max(1e-3),
                (y + shape.half.y) / h.max(1e-3),
            ];
            let (mut color, what) = match &shape.paint {
                UiPaint::Solid(c) => (*c, "solid"),
                UiPaint::Gradient(g) => (gradient(g, f, (w, h)), "gradient"),
                UiPaint::GradientImage(g) => (gradient(&g.gradient, f, (w, h)), "gradient"),
                UiPaint::Image(i) => {
                    let at = [
                        i.uv[0] + f[0] * (i.uv[2] - i.uv[0]),
                        i.uv[1] + f[1] * (i.uv[3] - i.uv[1]),
                    ];
                    (mul(textures.texel(i.texture, at)?, i.tint), "image")
                }
                UiPaint::Layer(_) => return None,
            };
            color[3] *= shape.opacity;
            Some((color, what))
        }
        _ => None,
    }
}

fn mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
}

/// A gradient's colour at `f` (fractions of its box, `size` px): its
/// parameter by its kind and spread, between its stops in OKLab with
/// premultiplied alpha, as og-render interpolates them.
fn gradient(g: &UiGradient, f: [f32; 2], size: (f32, f32)) -> [f32; 4] {
    let Some(first) = g.stops.first() else {
        return [0.0; 4];
    };
    let (px, py) = (f[0] * size.0, f[1] * size.1);
    let (fx, fy) = (g.from.x * size.0, g.from.y * size.1);
    let (tx, ty) = (g.to.x * size.0, g.to.y * size.1);
    let mut t = match g.kind {
        UiGradientKind::Linear => {
            let (dx, dy) = (tx - fx, ty - fy);
            ((px - fx) * dx + (py - fy) * dy) / (dx * dx + dy * dy).max(1e-6)
        }
        UiGradientKind::Radial => {
            let (rx, ry) = (tx.abs().max(1e-3), ty.abs().max(1e-3));
            (((px - fx) / rx).powi(2) + ((py - fy) / ry).powi(2)).sqrt()
        }
        UiGradientKind::Diamond => {
            let (rx, ry) = (tx.abs().max(1e-3), ty.abs().max(1e-3));
            ((px - fx) / rx).abs() + ((py - fy) / ry).abs()
        }
        UiGradientKind::Conic => {
            let angle = (py - fy).atan2(px - fx) - g.to.x;
            (angle / std::f32::consts::TAU).rem_euclid(1.0)
        }
    };
    let (lo, hi) = (first.offset, g.stops.last().map_or(1.0, |s| s.offset));
    t = match g.spread {
        UiSpread::Pad => t,
        UiSpread::Repeat => t.rem_euclid(1.0),
        UiSpread::Reflect => 1.0 - ((t.rem_euclid(2.0)) - 1.0).abs(),
        UiSpread::RepeatStops if hi > lo => lo + (t - lo).rem_euclid(hi - lo),
        UiSpread::RepeatStops => t,
    };
    let k = g.stops.iter().position(|s| s.offset > t);
    let (a, b, w) = match k {
        None => return g.stops.last().unwrap().color,
        Some(0) => return first.color,
        Some(k) => {
            let (a, b) = (&g.stops[k - 1], &g.stops[k]);
            (
                a.color,
                b.color,
                (t - a.offset) / (b.offset - a.offset).max(1e-6),
            )
        }
    };
    use color::ColorSpaceTag as S;
    let lab = |c: [f32; 4]| {
        let l = S::LinearSrgb.convert(S::Oklab, [c[0], c[1], c[2]]);
        [l[0] * c[3], l[1] * c[3], l[2] * c[3], c[3]]
    };
    let (a, b) = (lab(a), lab(b));
    let m: [f32; 4] = [0, 1, 2, 3].map(|i| a[i] + (b[i] - a[i]) * w);
    if m[3] <= 1e-6 {
        return [0.0; 4];
    }
    let rgb = S::Oklab.convert(S::LinearSrgb, [m[0] / m[3], m[1] / m[3], m[2] / m[3]]);
    [rgb[0].max(0.0), rgb[1].max(0.0), rgb[2].max(0.0), m[3]]
}

/// Each draw's alpha from the layers it is in (their composite opacity)
/// and the clip shapes it is cut to.
fn layers(items: &[DrawItem]) -> (Vec<f32>, Vec<Vec<og_ui::Rect>>) {
    let mut opacity = vec![1.0f32; items.len()];
    let mut clips = vec![Vec::new(); items.len()];
    let (mut open, mut stack): (Vec<usize>, Vec<og_ui::Rect>) = (Vec::new(), Vec::new());
    for (k, it) in items.iter().enumerate() {
        clips[k] = stack.clone();
        match &it.cmd {
            DrawCmd::BeginLayer { .. } => open.push(k),
            DrawCmd::EndLayer { opacity: o, .. } => {
                if let Some(start) = open.pop() {
                    opacity[start..k].iter_mut().for_each(|v| *v *= o);
                }
            }
            DrawCmd::PushClip { center, half, .. } => {
                stack.push(it.transform.map_rect(&og_ui::Rect::new(
                    center.0 - half.0,
                    center.1 - half.1,
                    2.0 * half.0,
                    2.0 * half.1,
                )))
            }
            DrawCmd::PopClip => {
                stack.pop();
            }
            _ => {}
        }
    }
    (opacity, clips)
}

/// The pairs of one built frame.
fn read(
    ui: &og_ui::Ui,
    sheet: &str,
    scheme: &'static str,
    textures: &mut Textures,
    out: &mut Vec<Pair>,
) {
    let items = &ui.frame().items;
    let (opacity, clips) = layers(items);
    let elements = ui.elements();
    let contains = |r: &og_ui::Rect, p: (f32, f32)| {
        p.0 >= r.x && p.0 <= r.x + r.width && p.1 >= r.y && p.1 <= r.y + r.height
    };
    for (k, it) in items.iter().enumerate() {
        let (text_cmd, effects) = match &it.cmd {
            DrawCmd::StyledText { text, effects } => (&**text, Some(&**effects)),
            c => (c, None),
        };
        let DrawCmd::Text {
            text,
            color,
            bounds,
            clip,
            ..
        } = text_cmd
        else {
            continue;
        };
        if text.trim().is_empty() || color[3] * opacity[k] < 0.05 {
            continue;
        }
        let shown = it.transform.map_rect(bounds).intersect(clip);
        if shown.width < 1.0 || shown.height < 1.0 {
            continue;
        }
        let mut ink_color = *color;
        ink_color[3] *= opacity[k];
        let effects = effects.filter(|e| e.outline.is_some() || !e.shadows.is_empty());
        let mut halo: Option<f32> = None;
        let mut worst: Option<(f32, Rgb, Rgb, &'static str)> = None;
        for (u, v) in [
            (1.0, 1.0),
            (3.0, 1.0),
            (5.0, 1.0),
            (1.0, 2.0),
            (3.0, 2.0),
            (5.0, 2.0),
        ] {
            let p = (
                shown.x + shown.width * u / 6.0,
                shown.y + shown.height * v / 3.0,
            );
            // Over black and over white: what the UI leaves of the scene.
            let mut grounds: [Rgb; 2] = [[0.0; 3], [1.0; 3]];
            let mut what = "solid";
            for (j, under) in items[..k].iter().enumerate() {
                if !clip_of(&under.cmd).is_none_or(|c| contains(c, p))
                    || !clips[j].iter().all(|c| contains(c, p))
                {
                    continue;
                }
                if let Some((mut c, kind)) = paint_at(under, p, textures) {
                    c[3] = (c[3] * opacity[j]).clamp(0.0, 1.0);
                    if c[3] > 0.02 {
                        if kind != "solid" {
                            what = kind;
                        }
                        grounds = grounds.map(|g| over(c, g));
                    }
                }
            }
            let scene = (lum(grounds[0]) - lum(grounds[1])).abs() > 0.01;
            for g in grounds {
                let ink = over(ink_color, g);
                let r = srgb(ink).contrast(srgb(g));
                // A halo over the ground: what the glyphs' edges sit on.
                let haloed = effects.map(|e| halo_ratio(e, ink, g));
                if let Some(h) = haloed {
                    halo = Some(halo.map_or(h, |m: f32| m.min(h)));
                }
                if worst.is_none_or(|w| r < w.0) {
                    worst = Some((r, ink, g, if scene { "scene" } else { what }));
                }
            }
        }
        let Some((wcag, ink, ground, over)) = worst else {
            continue;
        };
        let centre = (shown.x + shown.width / 2.0, shown.y + shown.height / 2.0);
        // The innermost element there that shows the text, else the
        // innermost there.
        let at: Vec<_> = elements
            .iter()
            .filter(|e| contains(&og_ui::Rect::new(e.x, e.y, e.width, e.height), centre))
            .collect();
        let element = at
            .iter()
            .rfind(|e| e.text.contains(text.trim()))
            .or(at.last())
            .copied();
        let (ink, ground) = (srgb(ink), srgb(ground));
        out.push(Pair {
            sheet: sheet.to_string(),
            scheme,
            text: text.clone(),
            path: element.map(|e| e.path.clone()).unwrap_or_default(),
            ink,
            ground,
            over,
            // Inactive where it or a control it is part of is disabled.
            disabled: element.is_some_and(|e| {
                at.iter().any(|a| {
                    a.disabled && (e.path == a.path || e.path.starts_with(&format!("{}/", a.path)))
                })
            }),
            halo,
            wcag,
            apca: ink.apca(ground),
        });
    }
}

/// The ink's worst WCAG 2 ratio on its halo over `g`: the ground
/// [`HALO_READ`] CSS px out from the glyphs' edge, in the direction its
/// shadows and outline cover least. Each covers there as the overlay
/// shader draws it (overlay.wgsl `fs_msdf`): a shadow, the glyphs moved by
/// its offset with their edge softened over its blur (at least a pixel),
/// `clamp((offset·n − d) / blur + ½)`, so a blurred halo covers at most
/// half at the edge and its offset half lies on one side only; an
/// outline, solid to its width. Taken at one device pixel per CSS px; the
/// font's distance field bounds a blur and an outline further on small
/// text.
fn halo_ratio(e: &og_draw::UiTextEffects, ink: Rgb, g: Rgb) -> f32 {
    let d = HALO_READ;
    (0..16)
        .map(|k| {
            let a = k as f32 * std::f32::consts::TAU / 16.0;
            let n = og_core::glam::Vec2::new(a.cos(), a.sin());
            let h = e.shadows.iter().fold(g, |g, s| {
                let cover = ((s.offset.dot(n) - d) / s.blur.max(1.0) + 0.5).clamp(0.0, 1.0);
                over([s.color[0], s.color[1], s.color[2], s.color[3] * cover], g)
            });
            let h = e.outline.as_ref().map_or(h, |o| {
                let cover = if matches!(o.align, og_draw::UiAlign::Inside) {
                    0.0
                } else {
                    let width = if matches!(o.align, og_draw::UiAlign::Center) {
                        o.width / 2.0
                    } else {
                        o.width
                    };
                    (width - d + 0.5).clamp(0.0, 1.0)
                };
                over([o.color[0], o.color[1], o.color[2], o.color[3] * cover], h)
            });
            srgb(ink).contrast(srgb(h))
        })
        .fold(f32::INFINITY, f32::min)
}

/// How far out from the glyphs' edge a halo is read, CSS px.
const HALO_READ: f32 = 1.0;

/// The clip rectangle a command carries (screen space), if it has one.
fn clip_of(cmd: &DrawCmd) -> Option<&og_ui::Rect> {
    match cmd {
        DrawCmd::Glass { clip, .. }
        | DrawCmd::Quad { clip, .. }
        | DrawCmd::Image { clip, .. }
        | DrawCmd::Bar { clip, .. }
        | DrawCmd::Text { clip, .. }
        | DrawCmd::Shape { clip, .. }
        | DrawCmd::Vector { clip, .. } => Some(clip),
        _ => None,
    }
}

fn lum(c: Rgb) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// The colour sets a player tells apart by colour, by the roles they are:
/// the sides of a fight and the states.
const SETS: [(&str, &[&str]); 2] = [
    ("team", &["team1", "team2", "team3", "team4"]),
    ("state", &["success", "warn", "danger", "info"]),
];

/// How far apart each pair of each set stays in OKLab (ΔEOK) for each
/// colour vision, in both schemes.
pub fn vision(pack: Option<&og_ui::style_pack::StylePack>) -> Result<Vec<Value>, String> {
    let mut out = Vec::new();
    for (scheme, theme) in [
        ("dark", og_ui::Theme::dark()),
        ("light", og_ui::Theme::light()),
    ] {
        let mut ui = og_ui::Ui::new(og_ui::UiDesc {
            theme,
            ..Default::default()
        });
        if let Some(pack) = pack {
            ui.set_style_pack(pack)?;
        }
        for (set, roles) in SETS {
            let colours: Vec<(&str, Srgba)> = roles
                .iter()
                .filter_map(|r| Some((*r, ui.theme().color(r)?)))
                .collect();
            for v in ColorVision::ALL {
                let filter = ColorFilter::new(v, ColorVisionFilter::Simulate, 1.0);
                let seen = |c: Srgba| match &filter {
                    Some(f) => Srgba {
                        rgb: f.apply_srgb8(c.rgb),
                        alpha: 1.0,
                    },
                    None => Srgba { alpha: 1.0, ..c },
                };
                let mut pairs = Vec::new();
                for (i, (a, ca)) in colours.iter().enumerate() {
                    for (b, cb) in &colours[i + 1..] {
                        let (sa, sb) = (seen(*ca), seen(*cb));
                        pairs.push(json!({
                            "a": a, "b": b, "seenA": hex(sa), "seenB": hex(sb),
                            "deltaE": (f64::from(oklab_distance(sa, sb)) * 1000.0).round() / 1000.0,
                            "ratio": round2(sa.contrast(sb)),
                        }));
                    }
                }
                out.push(json!({"scheme": scheme, "set": set, "vision": v.name(), "pairs": pairs}));
            }
        }
    }
    Ok(out)
}

fn oklab_distance(a: Srgba, b: Srgba) -> f32 {
    use color::ColorSpaceTag as S;
    let lab = |c: Srgba| {
        let l = c.linear();
        S::LinearSrgb.convert(S::Oklab, [l[0], l[1], l[2]])
    };
    let (a, b) = (lab(a), lab(b));
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pairs `body` draws in the dark default style.
    fn read_body(body: &str) -> Vec<Pair> {
        let doc = og_ui::UiDocument::parse(
            "t.svelte",
            &format!("<meta name=\"component\" content=\"t\">\n{body}"),
        )
        .unwrap();
        let sheets = Sheets {
            docs: vec![doc],
            draws: Vec::new(),
            messages: Messages(BTreeMap::new()),
        };
        let mut textures = Textures {
            files: Vec::new(),
            decoded: HashMap::new(),
        };
        let theme = og_ui::Theme::dark();
        let ui = drawn(theme, None, &sheets, &textures, "t", &Value::Null, false).unwrap();
        let mut out = Vec::new();
        read(&ui, "t", "dark", &mut textures, &mut out);
        out
    }

    fn of<'a>(pairs: &'a [Pair], text: &str) -> &'a Pair {
        pairs.iter().find(|p| p.text == text).unwrap_or_else(|| {
            panic!(
                "no {text} in {:?}",
                pairs.iter().map(|p| &p.text).collect::<Vec<_>>()
            )
        })
    }

    /// The expected values are worked out from the colours alone, in linear
    /// light: a half-black plate over a white scene is luminance 0.5, so
    /// white text on it reads 1.05 / 0.55 = 1.91:1 (and 21:1 over a black
    /// scene: the worse is kept, and the ground is the scene's); an opaque
    /// fill reads the two colours' own ratio; a hard-stop gradient is read
    /// where the text is, so a caption on its black half reads 21:1 where
    /// the average of its stops (mid grey) would give 2.9:1.
    #[test]
    fn texts_are_read_on_what_is_drawn_under_them() {
        let pairs = read_body(
            r#"<div id="Page">
  <div id="Plate"><p id="OnPlate">Plate</p></div>
  <div id="Solid"><p id="OnSolid">Solid</p></div>
  <div id="Split"><p id="OnSplit">Split</p></div>
</div>
<style>
#Page { display: flex; flex-direction: column; gap: 8px; }
p { color: #ffffff; margin: 0; }
#Plate { background: rgb(0 0 0 / 50%); width: 200px; }
#Solid { background: #2060c0; width: 200px; }
#OnSolid { color: #f0f0f0; }
#Split { width: 200px; height: 200px; background: linear-gradient(#000 0%, #000 50%, #fff 50%, #fff 100%); }
</style>"#,
        );
        let plate = of(&pairs, "Plate");
        assert_eq!(plate.over, "scene");
        assert!((plate.wcag - 1.05 / 0.55).abs() < 0.03, "{}", plate.wcag);
        let solid = of(&pairs, "Solid");
        let want = Srgba::rgb(0xf0, 0xf0, 0xf0).contrast(Srgba::rgb(0x20, 0x60, 0xc0));
        assert_eq!(solid.over, "solid");
        assert!((solid.wcag - want).abs() < 0.05, "{} vs {want}", solid.wcag);
        assert!(
            (solid.apca - Srgba::rgb(0xf0, 0xf0, 0xf0).apca(Srgba::rgb(0x20, 0x60, 0xc0))).abs()
                < 0.5
        );
        let split = of(&pairs, "Split");
        assert_eq!(split.over, "gradient");
        assert!(split.wcag > 20.0, "{}", split.wcag);
    }

    /// A halo is read a pixel out from the glyphs' edge, on its weakest
    /// side, as the overlay shader covers it: a blurred shadow covers at
    /// most half at the edge and nothing past its blur, and a drop lies on
    /// one side, so white text with either over a white scene still reads
    /// 1:1; eight hard copies round the glyphs cover the ring nearly
    /// whole (at least 0.97 black: 1.05 / 0.08 = 13:1 at worst). Counting
    /// each shadow at its full colour read the first two 21:1.
    #[test]
    fn halos_are_read_where_the_glyphs_edge_sits() {
        let ring = "1.5px 0 0 #000, -1.5px 0 0 #000, 0 1.5px 0 #000, 0 -1.5px 0 #000, \
                    1px 1px 0 #000, -1px 1px 0 #000, 1px -1px 0 #000, -1px -1px 0 #000";
        let pairs = read_body(&format!(
            r#"<div id="Page">
  <p id="Soft">Soft</p>
  <p id="Drop">Drop</p>
  <p id="Ring">Ring</p>
</div>
<style>
#Page {{ display: flex; flex-direction: column; gap: 8px; }}
p {{ color: #ffffff; margin: 0; }}
#Soft {{ text-shadow: 0 0 2px #000, 0 1px 2px #000; }}
#Drop {{ text-shadow: 0 2px 0 #000; }}
#Ring {{ text-shadow: {ring}; }}
</style>"#
        ));
        for id in ["Soft", "Drop"] {
            let halo = of(&pairs, id).halo.unwrap();
            assert!(halo < 1.05, "{id} reads {halo:.2}:1 on its halo");
        }
        let halo = of(&pairs, "Ring").halo.unwrap();
        assert!(halo > 12.0, "the ring reads {halo:.2}:1");
    }
}
