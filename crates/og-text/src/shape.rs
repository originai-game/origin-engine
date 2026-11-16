//! A cooked font's file at run time: shaping (HarfRust, the `shaping`
//! feature) and the metrics of characters outside its atlas.
//!
//! A font cooked with a shaped script (Arabic, Devanagari and og-i18n's
//! other `needs_shaping` scripts) carries its font file and every glyph its
//! substitutions reach from the charset (contextual forms, ligatures,
//! conjuncts) by glyph id ([`GlyphKey::Id`]). A run of such a script is
//! shaped with HarfRust in its direction and drawn as those glyphs, placed
//! where HarfRust puts them (marks over their bases, a right-to-left run
//! already in visual order).
//!
//! A font cooked with `runtime true` carries its file too: a character
//! outside the charset is measured from the file's metrics and recorded,
//! and the host renders it into a page of its own (`crate::glyphs`) before
//! the frame is drawn.

use skrifa::{
    MetadataProvider,
    instance::{LocationRef, Size},
};
#[cfg(feature = "shaping")]
use std::collections::HashMap;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

pub use og_draw::GlyphKey;

/// The OpenType features a type step turns on (docs/ui.md §6.3.4): small
/// caps, capitals to small caps, tabular and lining figures, case-sensitive
/// forms by name, and any other feature settings CSS gives (`font-variant-*`,
/// `font-feature-settings`, `font-kerning`: a tag and its value, `0` turning
/// a default feature such as `kern` or `liga` off). A run with any of them
/// is shaped with the font file, so its glyphs and advances are the
/// features'; the font's import names the features its atlas holds glyphs
/// for (`features smcp tnum`), and a run-time font renders the rest. A
/// feature the font lacks changes nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Features(u32);

/// One OpenType feature setting: its tag and value.
pub type Setting = ([u8; 4], u16);

/// Every distinct list of settings a [`Features`] has carried, by its id
/// less one: a few per game (one per CSS feature combination).
static SETTINGS: Mutex<Vec<Arc<[Setting]>>> = Mutex::new(Vec::new());

impl Features {
    /// No feature: the font's defaults.
    pub const NONE: Features = Features(0);
    /// The features, in bit order, by their OpenType tags.
    pub const TAGS: [&'static str; 5] = ["smcp", "c2sc", "tnum", "lnum", "case"];
    /// The bits of the named features; the rest is the settings' id.
    const NAMED: u32 = (1 << Self::TAGS.len()) - 1;

    /// The features of `tags`; an unknown tag is an error naming it.
    pub fn parse<'a>(tags: impl IntoIterator<Item = &'a str>) -> Result<Features, String> {
        let mut bits = 0;
        for t in tags {
            let i = Self::TAGS.iter().position(|k| *k == t).ok_or_else(|| {
                format!(
                    "unknown OpenType feature '{t}' (known: {})",
                    Self::TAGS.join(", ")
                )
            })?;
            bits |= 1 << i;
        }
        Ok(Features(bits))
    }

    /// Tabular lining figures (`tnum lnum`): what text in the numeric font
    /// role always sets (scores, timers, the HUD's numerals).
    pub const FIGURES: Features = Features(1 << 2 | 1 << 3);

    /// Both sets of named features, with `self`'s settings (else `other`'s).
    pub const fn with(self, other: Features) -> Features {
        let settings = if self.0 & !Self::NAMED != 0 {
            self.0 & !Self::NAMED
        } else {
            other.0 & !Self::NAMED
        };
        Features(settings | ((self.0 | other.0) & Self::NAMED))
    }

    /// These features with `settings` applied in order, each a tag and its
    /// value: a named feature at 1 is turned on and at 0 off; every other
    /// setting (a named one at another value too) is kept, a later one
    /// replacing an earlier one of its tag.
    pub fn with_settings(self, settings: &[Setting]) -> Features {
        let mut bits = self.0 & Self::NAMED;
        let mut list: Vec<Setting> = self.settings().to_vec();
        for &(tag, value) in settings {
            list.retain(|(t, _)| *t != tag);
            match Self::TAGS.iter().position(|k| k.as_bytes() == tag) {
                Some(i) if value <= 1 => {
                    bits &= !(1 << i);
                    bits |= u32::from(value) << i;
                }
                Some(i) => {
                    bits &= !(1 << i);
                    list.push((tag, value));
                }
                None => list.push((tag, value)),
            }
        }
        list.sort();
        if list.is_empty() {
            return Features(bits);
        }
        let mut all = SETTINGS.lock().unwrap_or_else(|e| e.into_inner());
        let id = match all.iter().position(|s| **s == *list) {
            Some(i) => i,
            None => {
                all.push(list.into());
                all.len() - 1
            }
        };
        Features(bits | ((id as u32 + 1) << Self::TAGS.len()))
    }

    /// Its settings beyond the named features, by tag.
    pub fn settings(self) -> Arc<[Setting]> {
        match (self.0 >> Self::TAGS.len()).checked_sub(1) {
            Some(i) => SETTINGS.lock().unwrap_or_else(|e| e.into_inner())[i as usize].clone(),
            None => Arc::new([]),
        }
    }

    /// Every feature it sets, a tag and its value: the named ones on, then
    /// its settings.
    pub fn values(self) -> Vec<Setting> {
        let mut out: Vec<Setting> = self
            .tags()
            .map(|t| {
                let b = t.as_bytes();
                ([b[0], b[1], b[2], b[3]], 1)
            })
            .collect();
        out.extend(self.settings().iter().copied());
        out
    }

    /// [`Features::values`] in CSS `font-feature-settings` syntax, as
    /// Parley reads it.
    pub fn css(self) -> String {
        self.values()
            .iter()
            .map(|(t, v)| format!("'{}' {v}", String::from_utf8_lossy(t)))
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Tabular figures (`tnum`): every digit on one advance, so a timer or
    /// a score keeps its width as it ticks. A face without the feature is
    /// given it ([`crate::FontMetrics::run_with`]).
    pub fn tabular(self) -> bool {
        self.0 & (1 << 2) != 0
    }

    /// The named tags turned on, in [`Features::TAGS`] order.
    pub fn tags(self) -> impl Iterator<Item = &'static str> {
        Self::TAGS
            .into_iter()
            .enumerate()
            .filter(move |(i, _)| self.0 & (1 << i) != 0)
            .map(|(_, t)| t)
    }

    /// The named features' bits ([`Features::from_bits`] reads them back).
    pub fn bits(self) -> u8 {
        (self.0 & Self::NAMED) as u8
    }

    /// The named features of `bits` (as [`Features::bits`] gives them), for
    /// the generated default style.
    pub const fn from_bits(bits: u8) -> Features {
        Features(bits as u32 & Self::NAMED)
    }
}

/// One shaped glyph, in em.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shaped {
    /// The character it draws: the glyph's own (its cmap's), or for a
    /// shaped form (`glyph`) the first of its cluster.
    pub ch: char,
    /// A shaped form, drawn by the font's glyph id instead of `ch`'s glyph.
    pub glyph: Option<u16>,
    pub advance: f32,
    /// The glyph's offset from the pen (y up).
    pub x: f32,
    pub y: f32,
    /// The byte offset in the run of the first character it draws.
    pub cluster: u32,
}

impl Shaped {
    /// The glyph it draws: its form by id, else its character's.
    pub fn key(&self) -> GlyphKey {
        self.glyph.map_or(GlyphKey::Char(self.ch), GlyphKey::Id)
    }
}

/// A shaped run in visual order, or `None` when the font cannot shape it
/// with the glyphs its atlas has.
pub type ShapedRun = Option<Arc<Vec<Shaped>>>;

/// A cooked font's file: shaping and run-time glyph metrics.
pub struct FontFile {
    ttf: Arc<Vec<u8>>,
    runtime: bool,
    /// An emoji face: one whose characters have COLR colour glyphs.
    colour: bool,
    units_per_em: f32,
    /// The height of its lowercase letters, em (OS/2 `sxHeight`; half an
    /// em where the font gives none, as CSS Values 4 §6.1.1 assumes).
    x_height: f32,
    /// The height of its capitals, em (OS/2 `sCapHeight`; 0 where the
    /// font gives none).
    cap_height: f32,
    /// How far its descent reaches below the baseline, em (the line
    /// metrics' descent, positive; 0 where the font gives none).
    descent: f32,
    /// The underline it suggests (`post`): its top's distance below the
    /// baseline and its thickness, em.
    underline: Option<(f32, f32)>,
    #[cfg(feature = "shaping")]
    shaper: harfrust::ShaperData,
    #[cfg(feature = "shaping")]
    shaped: Mutex<HashMap<(String, bool, Features), ShapedRun>>,
    /// Glyphs outside the atlas: their advance in em (`None`: not in the font).
    extra: Mutex<BTreeMap<GlyphKey, Option<f32>>>,
    /// Glyphs outside the atlas laid out since the host last took them.
    wanted: Mutex<BTreeSet<GlyphKey>>,
}

impl std::fmt::Debug for FontFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FontFile({} bytes)", self.ttf.len())
    }
}

impl PartialEq for FontFile {
    fn eq(&self, other: &Self) -> bool {
        self.runtime == other.runtime && Arc::ptr_eq(&self.ttf, &other.ttf)
    }
}

/// The most shaped runs a font remembers before it starts over.
#[cfg(feature = "shaping")]
const CACHE_RUNS: usize = 4096;

impl FontFile {
    /// The file of a cooked font (`None`: not a font Skrifa reads).
    pub fn new(ttf: Arc<Vec<u8>>) -> Option<Arc<FontFile>> {
        Self::read(ttf, true)
    }

    /// Layout-only tables. Keeping shaping data does not opt a font into
    /// runtime rasterization or extend the game's declared charset.
    pub fn for_shaping(ttf: Arc<Vec<u8>>) -> Option<Arc<FontFile>> {
        Self::read(ttf, false)
    }

    fn read(ttf: Arc<Vec<u8>>, runtime: bool) -> Option<Arc<FontFile>> {
        let face = skrifa::FontRef::new(&ttf).ok()?;
        let metrics = face.metrics(Size::unscaled(), LocationRef::default());
        let units_per_em = f32::from(metrics.units_per_em);
        let x_height = metrics
            .x_height
            .filter(|x| *x > 0.0)
            .map_or(0.5, |x| x / units_per_em.max(1.0));
        let cap_height = metrics
            .cap_height
            .filter(|x| *x > 0.0)
            .map_or(0.0, |x| x / units_per_em.max(1.0));
        let descent = (-metrics.descent / units_per_em.max(1.0)).max(0.0);
        let underline = metrics.underline.filter(|u| u.thickness > 0.0).map(|u| {
            (
                -u.offset / units_per_em.max(1.0),
                u.thickness / units_per_em.max(1.0),
            )
        });
        #[cfg(feature = "shaping")]
        let shaper = harfrust::ShaperData::new(&face);
        let colour = {
            let glyphs = face.color_glyphs();
            face.charmap()
                .mappings()
                .any(|(_, gid)| glyphs.get(gid).is_some())
        };
        Some(Arc::new(FontFile {
            ttf,
            runtime,
            colour,
            units_per_em,
            x_height,
            cap_height,
            descent,
            underline,
            #[cfg(feature = "shaping")]
            shaper,
            #[cfg(feature = "shaping")]
            shaped: Mutex::new(HashMap::new()),
            extra: Mutex::new(BTreeMap::new()),
            wanted: Mutex::new(BTreeSet::new()),
        }))
    }

    /// Whether it is an emoji face: some character maps to a COLR colour
    /// glyph. Its spaces and punctuation are not a text face's (Noto Color
    /// Emoji's space is 1.25 em), so they never stick to it in a mixed text.
    pub fn is_colour(&self) -> bool {
        self.colour
    }

    /// The height of its lowercase letters, em.
    pub fn x_height(&self) -> f32 {
        self.x_height
    }

    /// The height of its capitals, em (0: the font gives none).
    pub fn cap_height(&self) -> f32 {
        self.cap_height
    }

    /// How far its descent reaches below the baseline, em (0: none given).
    pub fn descent(&self) -> f32 {
        self.descent
    }

    /// The underline it suggests: its top's distance below the baseline
    /// and its thickness, em.
    pub fn underline(&self) -> Option<(f32, f32)> {
        self.underline
    }

    /// The font file's bytes.
    pub fn bytes(&self) -> &Arc<Vec<u8>> {
        &self.ttf
    }

    /// Whether this build shapes (the `shaping` feature).
    pub fn shapes() -> bool {
        cfg!(feature = "shaping")
    }

    /// A run shaped in its direction, in visual order; `None` when this
    /// build does not shape or the atlas lacks a glyph the result needs
    /// (`has`), so the run is drawn as written. A glyph that is its
    /// character's own (the font's character map gives it, as for a full
    /// stop or a digit inside an Arabic run, or the mirror of a bracket) is
    /// drawn as that character when `has` it: the cook keeps such glyphs
    /// under their code points, not as shaped forms.
    pub fn shape(&self, run: &str, rtl: bool, has: &dyn Fn(GlyphKey) -> bool) -> ShapedRun {
        self.shape_with(run, rtl, Features::NONE, has)
    }

    /// [`FontFile::shape`] with OpenType `features` turned on.
    pub fn shape_with(
        &self,
        run: &str,
        rtl: bool,
        features: Features,
        has: &dyn Fn(GlyphKey) -> bool,
    ) -> ShapedRun {
        #[cfg(feature = "shaping")]
        {
            let key = (run.to_string(), rtl, features);
            if let Some(hit) = self.shaped.lock().ok()?.get(&key) {
                return hit.clone();
            }
            let shaped = self.shape_now(run, rtl, features, has).map(Arc::new);
            if let Ok(mut c) = self.shaped.lock() {
                if c.len() >= CACHE_RUNS {
                    c.clear();
                }
                c.insert(key, shaped.clone());
            }
            shaped
        }
        #[cfg(not(feature = "shaping"))]
        {
            let _ = (run, rtl, features, has);
            None
        }
    }

    #[cfg(feature = "shaping")]
    fn shape_now(
        &self,
        run: &str,
        rtl: bool,
        features: Features,
        has: &dyn Fn(GlyphKey) -> bool,
    ) -> Option<Vec<Shaped>> {
        let face = skrifa::FontRef::new(&self.ttf).ok()?;
        let mut buf = harfrust::UnicodeBuffer::new();
        buf.push_str(run);
        buf.guess_segment_properties();
        buf.set_direction(if rtl {
            harfrust::Direction::RightToLeft
        } else {
            harfrust::Direction::LeftToRight
        });
        let on: Vec<harfrust::Feature> = features
            .values()
            .into_iter()
            .map(|(t, v)| harfrust::Feature::new(harfrust::Tag::new(&t), u32::from(v), ..))
            .collect();
        let shaper = self.shaper.shaper(&face).build();
        let out = shaper.shape(buf, harfrust::ShapeOptions::new().features(&on));
        let em = self.units_per_em;
        let mut glyphs = Vec::with_capacity(out.len());
        for (g, p) in out.glyph_infos().iter().zip(out.glyph_positions()) {
            let c = run.get(g.cluster as usize..)?.chars().next()?;
            let id = u16::try_from(g.glyph_id).ok()?;
            let (ch, glyph) = if has(GlyphKey::Id(id)) {
                (c, Some(id))
            } else {
                // The character's own glyph, kept under its code point.
                let own = [c, crate::text::mirror(c)].into_iter().find(|&c| {
                    face.charmap().map(c).map(|i| i.to_u32()) == Some(g.glyph_id)
                        && has(GlyphKey::Char(c))
                })?;
                (own, None)
            };
            glyphs.push(Shaped {
                ch,
                glyph,
                advance: p.x_advance as f32 / em,
                x: p.x_offset as f32 / em,
                y: p.y_offset as f32 / em,
                cluster: g.cluster,
            });
        }
        Some(glyphs)
    }

    /// A character outside the atlas: its advance in em from the font's
    /// metrics, recorded for the host to render; `None` when the font does
    /// not have it either.
    pub fn advance(&self, c: char) -> Option<f32> {
        self.advance_of(GlyphKey::Char(c))
    }

    /// [`FontFile::advance`] of a glyph by its key (a shaped form by id).
    pub fn advance_of(&self, key: GlyphKey) -> Option<f32> {
        if !self.runtime {
            return None;
        }
        let a = self.lookup(key);
        if a.is_some()
            && !key.char().is_some_and(char::is_whitespace)
            && let Ok(mut w) = self.wanted.lock()
        {
            w.insert(key);
        }
        a
    }

    /// `c`'s advance down a vertical line, em: the font's `vmtx` (`None`
    /// without one, or without `c`).
    pub fn vertical_advance(&self, c: char) -> Option<f32> {
        let face = ttf_parser::Face::parse(&self.ttf, 0).ok()?;
        let g = face.glyph_index(c)?;
        let a = face.glyph_ver_advance(g)?;
        Some(a as f32 / face.units_per_em() as f32)
    }

    /// `run` shaped down a vertical line (HarfBuzz's top-to-bottom shaping,
    /// with the font's `vert` and `vrt2` forms): each glyph's form, `None`
    /// where it is the character's own, and `None` for the whole when this
    /// build does not shape or the atlas lacks a form (`has`).
    pub fn vertical_forms(
        &self,
        run: &str,
        has: &dyn Fn(GlyphKey) -> bool,
    ) -> Option<Vec<(u32, Option<u16>)>> {
        #[cfg(feature = "shaping")]
        {
            let face = skrifa::FontRef::new(&self.ttf).ok()?;
            let mut buf = harfrust::UnicodeBuffer::new();
            buf.push_str(run);
            buf.guess_segment_properties();
            buf.set_direction(harfrust::Direction::TopToBottom);
            let shaper = self.shaper.shaper(&face).build();
            let out = shaper.shape(buf, harfrust::ShapeOptions::new());
            let mut forms = Vec::with_capacity(out.len());
            for g in out.glyph_infos() {
                let c = run.get(g.cluster as usize..)?.chars().next()?;
                let own = face.charmap().map(c).map(|i| i.to_u32()) == Some(g.glyph_id);
                let id = u16::try_from(g.glyph_id).ok()?;
                if own {
                    forms.push((g.cluster, None));
                } else if has(GlyphKey::Id(id)) {
                    forms.push((g.cluster, Some(id)));
                } else {
                    return None;
                }
            }
            Some(forms)
        }
        #[cfg(not(feature = "shaping"))]
        {
            let _ = (run, has);
            None
        }
    }

    /// Whether the font file has `c` (without recording it).
    pub fn covers(&self, c: char) -> bool {
        self.runtime && self.lookup(GlyphKey::Char(c)).is_some()
    }

    /// Keep an atlas character when it names this glyph; otherwise request
    /// the glyph by its id, including new ligatures: the character drawn
    /// and the glyph id it is drawn as (`None`: its own glyph).
    #[cfg(feature = "shaping")]
    pub fn character_for_glyph(
        &self,
        id: u32,
        source: char,
        has: &dyn Fn(GlyphKey) -> bool,
    ) -> (char, Option<u16>) {
        let face = skrifa::FontRef::new(&self.ttf).expect("validated font");
        if let Some(c) = [source, crate::text::mirror(source)]
            .into_iter()
            .find(|&c| {
                face.charmap().map(c).is_some_and(|g| g.to_u32() == id)
                    && (has(GlyphKey::Char(c)) || !self.runtime)
            })
        {
            return (c, None);
        }
        let Ok(id) = u16::try_from(id) else {
            return (source, None);
        };
        if id == 0 && !self.runtime {
            return (source, None);
        }
        if !has(GlyphKey::Id(id)) {
            self.advance_of(GlyphKey::Id(id));
        }
        (source, Some(id))
    }

    fn lookup(&self, key: GlyphKey) -> Option<f32> {
        let mut extra = self.extra.lock().ok()?;
        *extra.entry(key).or_insert_with(|| {
            let face = skrifa::FontRef::new(&self.ttf).ok()?;
            let g = match key {
                GlyphKey::Id(id) => skrifa::GlyphId::new(u32::from(id)),
                GlyphKey::Char(c) => face.charmap().map(c)?,
            };
            Some(
                face.glyph_metrics(Size::unscaled(), LocationRef::default())
                    .advance_width(g)
                    .unwrap_or(0.0)
                    / self.units_per_em,
            )
        })
    }

    /// The glyphs outside the atlas laid out since the last call, in key
    /// order: what the host renders before drawing the frame.
    pub fn take_wanted(&self) -> Vec<GlyphKey> {
        self.wanted
            .lock()
            .map(|mut w| std::mem::take(&mut *w).into_iter().collect())
            .unwrap_or_default()
    }
}
