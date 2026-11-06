//! A cooked font's file at run time: shaping (HarfRust, the `shaping`
//! feature) and the metrics of characters outside its atlas.
//!
//! A font cooked with a shaped script (Arabic, Devanagari and og-i18n's
//! other `needs_shaping` scripts) carries its font file and every glyph its
//! substitutions reach from the charset (contextual forms, ligatures,
//! conjuncts) under U+F0000 + glyph id. A run of such a script is shaped
//! with HarfRust in its direction and drawn as those private-use
//! characters, placed where HarfRust puts them (marks over their bases,
//! a right-to-left run already in visual order).
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

/// Where shaped glyphs are drawn from: U+F0000 + glyph id (og-cook's font cook).
pub const GLYPH_PUA: u32 = 0xF_0000;

/// The OpenType features a type step turns on (docs/ui.md §15.5.4): small
/// caps, capitals to small caps, tabular and lining figures, case-sensitive
/// forms. A run with any of them is shaped with the font file, so its glyphs
/// and advances are the feature's; the font's import names the same
/// features (`features smcp tnum`) so its atlas holds their glyphs. A
/// feature the font lacks changes nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Features(u8);

impl Features {
    /// No feature: the font's defaults.
    pub const NONE: Features = Features(0);
    /// The features, in bit order, by their OpenType tags.
    pub const TAGS: [&'static str; 5] = ["smcp", "c2sc", "tnum", "lnum", "case"];

    /// The features of `tags`; an unknown tag is an error naming it.
    pub fn parse<'a>(tags: impl IntoIterator<Item = &'a str>) -> Result<Features, String> {
        let mut bits = 0u8;
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

    /// Both sets of features.
    pub const fn with(self, other: Features) -> Features {
        Features(self.0 | other.0)
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

    /// The tags turned on, in [`Features::TAGS`] order.
    pub fn tags(self) -> impl Iterator<Item = &'static str> {
        Self::TAGS
            .into_iter()
            .enumerate()
            .filter(move |(i, _)| self.0 & (1 << i) != 0)
            .map(|(_, t)| t)
    }

    /// The bits, for a cache key.
    pub fn bits(self) -> u8 {
        self.0
    }

    /// The features of `bits` (as [`Features::bits`] gives them), for the
    /// generated default style.
    pub const fn from_bits(bits: u8) -> Features {
        Features(bits & ((1 << Self::TAGS.len()) - 1))
    }
}

/// One shaped glyph, in em.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shaped {
    /// U+F0000 + glyph id.
    pub ch: char,
    pub advance: f32,
    /// The glyph's offset from the pen (y up).
    pub x: f32,
    pub y: f32,
    /// The byte offset in the run of the first character it draws.
    pub cluster: u32,
}

/// A shaped run in visual order, or `None` when the font cannot shape it
/// with the glyphs its atlas has.
pub type ShapedRun = Option<Arc<Vec<Shaped>>>;

/// A cooked font's file: shaping and run-time glyph metrics.
pub struct FontFile {
    ttf: Arc<Vec<u8>>,
    runtime: bool,
    units_per_em: f32,
    #[cfg(feature = "shaping")]
    shaper: harfrust::ShaperData,
    #[cfg(feature = "shaping")]
    shaped: Mutex<HashMap<(String, bool, u8), ShapedRun>>,
    /// Characters outside the atlas: their advance in em (`None`: not in the font).
    extra: Mutex<BTreeMap<char, Option<f32>>>,
    /// Characters outside the atlas laid out since the host last took them.
    wanted: Mutex<BTreeSet<char>>,
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
        let units_per_em = f32::from(
            face.metrics(Size::unscaled(), LocationRef::default())
                .units_per_em,
        );
        #[cfg(feature = "shaping")]
        let shaper = harfrust::ShaperData::new(&face);
        Some(Arc::new(FontFile {
            ttf,
            runtime,
            units_per_em,
            #[cfg(feature = "shaping")]
            shaper,
            #[cfg(feature = "shaping")]
            shaped: Mutex::new(HashMap::new()),
            extra: Mutex::new(BTreeMap::new()),
            wanted: Mutex::new(BTreeSet::new()),
        }))
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
    pub fn shape(&self, run: &str, rtl: bool, has: &dyn Fn(char) -> bool) -> ShapedRun {
        self.shape_with(run, rtl, Features::NONE, has)
    }

    /// [`FontFile::shape`] with OpenType `features` turned on.
    pub fn shape_with(
        &self,
        run: &str,
        rtl: bool,
        features: Features,
        has: &dyn Fn(char) -> bool,
    ) -> ShapedRun {
        #[cfg(feature = "shaping")]
        {
            let key = (run.to_string(), rtl, features.bits());
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
        has: &dyn Fn(char) -> bool,
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
            .tags()
            .map(|t| {
                let b = t.as_bytes();
                let tag = harfrust::Tag::new(&[b[0], b[1], b[2], b[3]]);
                harfrust::Feature::new(tag, 1, ..)
            })
            .collect();
        let shaper = self.shaper.shaper(&face).build();
        let out = shaper.shape(buf, harfrust::ShapeOptions::new().features(&on));
        let em = self.units_per_em;
        let mut glyphs = Vec::with_capacity(out.len());
        for (g, p) in out.glyph_infos().iter().zip(out.glyph_positions()) {
            let mut ch = char::from_u32(GLYPH_PUA + g.glyph_id)?;
            if !has(ch) {
                // The character's own glyph, kept under its code point.
                let c = run.get(g.cluster as usize..)?.chars().next()?;
                ch = [c, crate::text::mirror(c)].into_iter().find(|&c| {
                    face.charmap().map(c).map(|i| i.to_u32()) == Some(g.glyph_id) && has(c)
                })?;
            }
            glyphs.push(Shaped {
                ch,
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
        if !self.runtime {
            return None;
        }
        let a = self.lookup(c);
        if a.is_some()
            && !c.is_whitespace()
            && let Ok(mut w) = self.wanted.lock()
        {
            w.insert(c);
        }
        a
    }

    /// Whether the font file has `c` (without recording it).
    pub fn covers(&self, c: char) -> bool {
        self.runtime && self.lookup(c).is_some()
    }

    /// Keep an atlas character when it names this glyph; otherwise request
    /// the glyph by the cook's private-use id, including new ligatures.
    #[cfg(feature = "shaping")]
    pub(crate) fn character_for_glyph(
        &self,
        id: u32,
        source: char,
        has: &dyn Fn(char) -> bool,
    ) -> char {
        let face = skrifa::FontRef::new(&self.ttf).expect("validated font");
        if let Some(c) = [source, crate::text::mirror(source)]
            .into_iter()
            .find(|&c| {
                face.charmap().map(c).is_some_and(|g| g.to_u32() == id) && (has(c) || !self.runtime)
            })
        {
            return c;
        }
        if id == 0 && !self.runtime {
            return source;
        }
        let c = char::from_u32(GLYPH_PUA + id).expect("OpenType glyph id");
        if !has(c) {
            self.advance(c);
        }
        c
    }

    fn lookup(&self, c: char) -> Option<f32> {
        let mut extra = self.extra.lock().ok()?;
        *extra.entry(c).or_insert_with(|| {
            let face = skrifa::FontRef::new(&self.ttf).ok()?;
            let g = match (c as u32).checked_sub(GLYPH_PUA) {
                Some(id) if id <= u16::MAX as u32 => skrifa::GlyphId::new(id),
                _ => face.charmap().map(c)?,
            };
            Some(
                face.glyph_metrics(Size::unscaled(), LocationRef::default())
                    .advance_width(g)
                    .unwrap_or(0.0)
                    / self.units_per_em,
            )
        })
    }

    /// The characters outside the atlas laid out since the last call, in
    /// code point order: what the host renders before drawing the frame.
    pub fn take_wanted(&self) -> Vec<char> {
        self.wanted
            .lock()
            .map(|mut w| std::mem::take(&mut *w).into_iter().collect())
            .unwrap_or_default()
    }
}
