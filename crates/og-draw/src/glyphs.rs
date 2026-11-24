//! The glyph atlas's bookkeeping: which rastered glyph sits where on which
//! page, as plain data. Whoever lays text out rasters a glyph the first
//! time it is drawn (og-text) and puts its pixels here; og-render uploads
//! the plots whose pixels changed and draws the paint arena's glyph quads
//! from the pages. Nothing here touches a GPU, so the pages are the same
//! bytes on every host and with no renderer at all.
//!
//! Three tiers, each its own pages ([`GlyphTier`]): coverage masks at the
//! drawn size (one byte a pixel), colour glyphs (premultiplied RGBA) and
//! distance fields (RGBA). A page is cut into [`PLOT`]-pixel plots and a
//! plot is packed in shelves. A plot no glyph of which was used for
//! [`UNUSED_FRAMES`] frames is emptied, in plot order at a fixed cadence
//! of frames; a glyph used this frame is never moved. A tier adds a page
//! only when every plot of its pages was used this frame.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// The atlas as the UI that fills it and the renderer that uploads it
/// share it.
pub type SharedAtlas = Arc<Mutex<Atlas>>;

/// A distance field's pixels an em ([`GlyphTier::Field`]).
pub const FIELD_EM_PX: f32 = 48.0;
/// The distances a field holds, field pixels across: `-FIELD_RANGE / 2` to
/// `FIELD_RANGE / 2` around the edge.
pub const FIELD_RANGE: f32 = 12.0;

/// A plot's side, pixels.
pub const PLOT: u32 = 256;
/// The frames a plot goes unused before it is emptied, and the cadence
/// (every so many frames) at which unused plots are.
pub const UNUSED_FRAMES: u64 = 64;
/// The clear pixels kept around each glyph, so a scaled or filtered
/// sample of one never reads its neighbour.
const GUTTER: u32 = 1;

/// What a page holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum GlyphTier {
    /// Coverage at the drawn size, one byte a pixel: a glyph's fill, its
    /// outline's stroke or its blurred shadow.
    #[default]
    Coverage = 0,
    /// A colour glyph at the drawn size, premultiplied sRGB RGBA.
    Color = 1,
    /// A multi-channel distance field with the true distance in alpha, at
    /// one size for every drawn size.
    Field = 2,
}

impl GlyphTier {
    pub const ALL: [GlyphTier; 3] = [GlyphTier::Coverage, GlyphTier::Color, GlyphTier::Field];

    /// Bytes a pixel.
    pub fn bytes(self) -> u32 {
        match self {
            GlyphTier::Coverage => 1,
            GlyphTier::Color | GlyphTier::Field => 4,
        }
    }
}

/// What a cached raster is of. Two texts that agree on all of it draw the
/// same pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlyphKey {
    pub tier: GlyphTier,
    /// The font file (og-text's `FaceId`) and the face's index in it.
    pub face: u32,
    pub index: u32,
    /// The face's variation location ([`Atlas::location`]); 0 its default.
    pub location: u32,
    pub glyph: u32,
    /// Device pixels per em in 1/64 (0 for a field, which serves every
    /// size).
    pub size: u32,
    /// The pen's quarter of a pixel in x, 0 to 3 (0 at device scale 1,
    /// where the pen is snapped).
    pub bin: u8,
    /// Hinted vertically (axis-aligned text) or not.
    pub hinted: bool,
    /// Bold the face lacks, as an outline grown this much, 1/64 px.
    pub embolden: u16,
    /// A slant the face lacks: the angle's tangent in 1/256 (0 upright).
    pub skew: i16,
    /// The outline's stroke of this width, 1/64 px, instead of its fill
    /// (`-webkit-text-stroke`); 0 the fill.
    pub stroke: u16,
    /// The mask blurred by this CSS blur radius, 1/64 px (a shadow).
    pub blur: u16,
    /// A colour glyph's palette (CSS `font-palette`; 0 the face's first)
    /// and the text colour its palette's foreground entry takes, sRGB.
    pub palette: u16,
    pub ink: [u8; 4],
}

impl GlyphKey {
    /// A face's glyph in `tier`, plain: no variation, bin, synthesis,
    /// stroke, blur or palette.
    pub fn new(tier: GlyphTier, face: u32, index: u32, glyph: u32, size: u32) -> GlyphKey {
        GlyphKey {
            tier,
            face,
            index,
            location: 0,
            glyph,
            size,
            bin: 0,
            hinted: false,
            embolden: 0,
            skew: 0,
            stroke: 0,
            blur: 0,
            palette: 0,
            ink: [0; 4],
        }
    }
}

/// Where a glyph's pixels are: its page (of its tier), its rectangle
/// there, and where that rectangle's top left sits from the pen: `left`
/// pixels right of it and `top` above the baseline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlyphSlot {
    pub page: u16,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub left: i16,
    pub top: i16,
    /// The plot it lies in ([`Atlas::touch`]); [`NO_PLOT`] for a glyph
    /// with no ink, which takes no room.
    pub plot: u32,
}

/// [`GlyphSlot::plot`] of a glyph with no ink.
pub const NO_PLOT: u32 = u32::MAX;

/// One row of glyphs in a plot.
#[derive(Clone, Copy, Debug)]
struct Shelf {
    y: u32,
    height: u32,
    /// The next free x.
    x: u32,
}

#[derive(Clone, Debug, Default)]
struct Plot {
    shelves: Vec<Shelf>,
    /// The frame a glyph of it was last used.
    used: u64,
    /// Moves when its pixels change.
    revision: u32,
    keys: Vec<GlyphKey>,
}

/// One page of a tier: `side` × `side` pixels, rows from the top.
#[derive(Clone, Debug)]
pub struct Page {
    pub pixels: Vec<u8>,
    plots: Vec<Plot>,
}

impl Page {
    /// Each plot's revision, in plot order (row by row): a plot whose
    /// revision moved has new pixels to upload.
    pub fn revisions(&self) -> impl Iterator<Item = u32> + '_ {
        self.plots.iter().map(|p| p.revision)
    }
}

#[derive(Clone, Debug)]
struct Tier {
    tier: GlyphTier,
    side: u32,
    max_pages: usize,
    pages: Vec<Page>,
}

/// The pages' sizes and how many a tier may grow to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtlasDesc {
    /// A coverage page's side, pixels (a multiple of [`PLOT`]).
    pub coverage_side: u32,
    /// A colour page's and a field page's side.
    pub color_side: u32,
    pub field_side: u32,
    /// The most pages a tier holds.
    pub max_pages: usize,
}

impl Default for AtlasDesc {
    /// A 4 MB coverage page and 4 MB colour and field pages, each made
    /// when its first glyph arrives; two pages a tier at most.
    fn default() -> Self {
        AtlasDesc {
            coverage_side: 2048,
            color_side: 1024,
            field_side: 1024,
            max_pages: 2,
        }
    }
}

/// What the atlas holds, for the probe.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AtlasStats {
    pub glyphs: u32,
    /// Pages made, by tier ([`GlyphTier`] order).
    pub pages: [u32; 3],
    /// Plots emptied so far.
    pub evicted: u64,
    /// Glyphs refused so far: no plot could take them.
    pub refused: u64,
}

/// The glyph atlas.
#[derive(Debug)]
pub struct Atlas {
    /// Which atlas this is, among every one the process made.
    id: u64,
    tiers: [Tier; 3],
    map: BTreeMap<GlyphKey, GlyphSlot>,
    /// Interned variation locations; entry 0 is the default (empty).
    locations: Vec<Vec<i16>>,
    frame: u64,
    /// Moves whenever any plot's pixels change.
    revision: u64,
    evicted: u64,
    refused: u64,
}

impl Default for Atlas {
    fn default() -> Self {
        Atlas::new(AtlasDesc::default())
    }
}

/// A plot's id: its tier, page and index in the page.
fn plot_id(tier: GlyphTier, page: usize, plot: usize) -> u32 {
    (tier as u32) << 24 | (page as u32) << 16 | plot as u32
}

impl Atlas {
    pub fn new(desc: AtlasDesc) -> Atlas {
        let tier = |tier, side: u32| {
            assert!(
                side >= PLOT && side.is_multiple_of(PLOT),
                "a page is whole plots"
            );
            Tier {
                tier,
                side,
                max_pages: desc.max_pages.max(1),
                pages: Vec::new(),
            }
        };
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Atlas {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            tiers: [
                tier(GlyphTier::Coverage, desc.coverage_side),
                tier(GlyphTier::Color, desc.color_side),
                tier(GlyphTier::Field, desc.field_side),
            ],
            map: BTreeMap::new(),
            locations: vec![Vec::new()],
            frame: 0,
            revision: 0,
            evicted: 0,
            refused: 0,
        }
    }

    /// The id of variation location `coords` (normalized, 2.14) for
    /// [`GlyphKey::location`]; the empty location is 0.
    pub fn location(&mut self, coords: &[i16]) -> u32 {
        // A face's default location, whatever its axis count.
        if coords.iter().all(|c| *c == 0) {
            return 0;
        }
        match self.locations.iter().position(|l| l == coords) {
            Some(i) => i as u32,
            None => {
                self.locations.push(coords.to_vec());
                self.locations.len() as u32 - 1
            }
        }
    }

    /// Where `key`'s pixels are, marking its plot used this frame; `None`
    /// when it is not in the atlas (raster it and [`Atlas::insert`]).
    pub fn get(&mut self, key: &GlyphKey) -> Option<GlyphSlot> {
        let slot = *self.map.get(key)?;
        self.touch(slot.plot);
        Some(slot)
    }

    /// Marks plot `plot` ([`GlyphSlot::plot`]) used this frame: a paint
    /// kept from an earlier frame touches the plots its quads read, so
    /// they are not emptied under it.
    pub fn touch(&mut self, plot: u32) {
        if plot == NO_PLOT {
            return;
        }
        let (tier, page, index) = (plot >> 24, (plot >> 16) & 0xff, plot & 0xffff);
        if let Some(p) = self
            .tiers
            .get_mut(tier as usize)
            .and_then(|t| t.pages.get_mut(page as usize))
            .and_then(|p| p.plots.get_mut(index as usize))
        {
            p.used = self.frame;
        }
    }

    /// Puts `key`'s raster in: `width` × `height` pixels of its tier's
    /// format, rows from the top, whose top left sits `left` right of the
    /// pen and `top` above the baseline. `None` when it is larger than a
    /// plot, or every plot of every page its tier may have was used this
    /// frame (draw it from the field tier, or next frame).
    pub fn insert(
        &mut self,
        key: GlyphKey,
        (left, top): (i32, i32),
        (width, height): (u32, u32),
        pixels: &[u8],
    ) -> Option<GlyphSlot> {
        let bytes = key.tier.bytes();
        assert_eq!(
            pixels.len(),
            (width * height * bytes) as usize,
            "a glyph's pixels are its size"
        );
        let (left, top) = (i16::try_from(left).ok()?, i16::try_from(top).ok()?);
        if width == 0 || height == 0 {
            let slot = GlyphSlot {
                left,
                top,
                plot: NO_PLOT,
                ..Default::default()
            };
            self.map.insert(key, slot);
            return Some(slot);
        }
        let (w, h) = (width + GUTTER, height + GUTTER);
        if w > PLOT || h > PLOT {
            self.refused += 1;
            return None;
        }
        let Some((page, plot, x, y)) = self.place(key.tier, w, h) else {
            self.refused += 1;
            return None;
        };
        let frame = self.frame;
        let t = &mut self.tiers[key.tier as usize];
        let side = t.side;
        let p = &mut t.pages[page];
        let per_row = side / PLOT;
        let (px, py) = (
            (plot as u32 % per_row) * PLOT + x,
            (plot as u32 / per_row) * PLOT + y,
        );
        let row = (width * bytes) as usize;
        for r in 0..height as usize {
            let at = (((py as usize + r) * side as usize) + px as usize) * bytes as usize;
            p.pixels[at..at + row].copy_from_slice(&pixels[r * row..(r + 1) * row]);
        }
        let held = &mut p.plots[plot];
        held.used = frame;
        held.revision = held.revision.wrapping_add(1);
        held.keys.push(key);
        self.revision += 1;
        let slot = GlyphSlot {
            page: page as u16,
            x: px as u16,
            y: py as u16,
            width: width as u16,
            height: height as u16,
            left,
            top,
            plot: plot_id(key.tier, page, plot),
        };
        self.map.insert(key, slot);
        Some(slot)
    }

    /// Room for `w` × `h` pixels in `tier`: the page, the plot and the
    /// place in the plot. The first plot with room, in page and plot
    /// order; else the plot unused longest (the lowest of equals) is
    /// emptied, unless every plot was used this frame, when a page is
    /// added if the tier may have one more.
    fn place(&mut self, tier: GlyphTier, w: u32, h: u32) -> Option<(usize, usize, u32, u32)> {
        let frame = self.frame;
        let t = &mut self.tiers[tier as usize];
        for (pi, page) in t.pages.iter_mut().enumerate() {
            for (ki, plot) in page.plots.iter_mut().enumerate() {
                if let Some((x, y)) = shelve(plot, w, h) {
                    return Some((pi, ki, x, y));
                }
            }
        }
        let oldest = t
            .pages
            .iter()
            .enumerate()
            .flat_map(|(pi, page)| {
                page.plots
                    .iter()
                    .enumerate()
                    .map(move |(ki, plot)| (plot.used, pi, ki))
            })
            .filter(|&(used, _, _)| used < frame)
            .min();
        let (pi, ki) = match oldest {
            Some((_, pi, ki)) => {
                self.empty(tier, pi, ki);
                (pi, ki)
            }
            None => {
                let t = &mut self.tiers[tier as usize];
                if t.pages.len() >= t.max_pages {
                    return None;
                }
                let plots = (t.side / PLOT) * (t.side / PLOT);
                t.pages.push(Page {
                    pixels: vec![0; (t.side * t.side * tier.bytes()) as usize],
                    plots: vec![Plot::default(); plots as usize],
                });
                (t.pages.len() - 1, 0)
            }
        };
        let plot = &mut self.tiers[tier as usize].pages[pi].plots[ki];
        shelve(plot, w, h).map(|(x, y)| (pi, ki, x, y))
    }

    /// Empties a plot: its glyphs leave the atlas and its pixels clear.
    fn empty(&mut self, tier: GlyphTier, page: usize, plot: usize) {
        let t = &mut self.tiers[tier as usize];
        let (side, bytes) = (t.side as usize, tier.bytes() as usize);
        let p = &mut t.pages[page];
        let per_row = side / PLOT as usize;
        let (px, py) = (
            (plot % per_row) * PLOT as usize,
            (plot / per_row) * PLOT as usize,
        );
        for r in 0..PLOT as usize {
            let at = ((py + r) * side + px) * bytes;
            p.pixels[at..at + PLOT as usize * bytes].fill(0);
        }
        let held = &mut p.plots[plot];
        for key in held.keys.drain(..) {
            self.map.remove(&key);
        }
        held.shelves.clear();
        held.revision = held.revision.wrapping_add(1);
        self.revision += 1;
        self.evicted += 1;
    }

    /// Ends a frame. Every [`UNUSED_FRAMES`] frames, the plots unused for
    /// that many are emptied, in tier, page and plot order.
    pub fn advance(&mut self) {
        self.frame += 1;
        if !self.frame.is_multiple_of(UNUSED_FRAMES) {
            return;
        }
        for tier in GlyphTier::ALL {
            for page in 0..self.tiers[tier as usize].pages.len() {
                for plot in 0..self.tiers[tier as usize].pages[page].plots.len() {
                    let p = &self.tiers[tier as usize].pages[page].plots[plot];
                    if !p.keys.is_empty() && p.used + UNUSED_FRAMES <= self.frame {
                        self.empty(tier, page, plot);
                    }
                }
            }
        }
    }

    /// A tier's pages and their side in pixels.
    pub fn pages(&self, tier: GlyphTier) -> (&[Page], u32) {
        let t = &self.tiers[tier as usize];
        debug_assert_eq!(t.tier, tier);
        (&t.pages, t.side)
    }

    /// Moves whenever any plot's pixels change: equal to the value last
    /// seen of the atlas with this [`Atlas::id`], nothing is new to upload.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Which atlas this is: no two a process makes share one, so pages
    /// uploaded from one are never taken for another's.
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn stats(&self) -> AtlasStats {
        AtlasStats {
            glyphs: self.map.len() as u32,
            pages: [
                self.tiers[0].pages.len() as u32,
                self.tiers[1].pages.len() as u32,
                self.tiers[2].pages.len() as u32,
            ],
            evicted: self.evicted,
            refused: self.refused,
        }
    }
}

/// A place for `w` × `h` pixels in `plot`: on the first shelf tall enough
/// and no taller than half again what it needs, else on a new shelf.
fn shelve(plot: &mut Plot, w: u32, h: u32) -> Option<(u32, u32)> {
    for s in &mut plot.shelves {
        if s.height >= h && s.height <= h + h / 2 && s.x + w <= PLOT {
            let at = (s.x, s.y);
            s.x += w;
            return Some(at);
        }
    }
    let y = plot.shelves.last().map_or(0, |s| s.y + s.height);
    if y + h > PLOT {
        return None;
    }
    plot.shelves.push(Shelf { y, height: h, x: w });
    Some((0, y))
}
