//! Stylisation as data (the IDL's `Stylise` and `MaterialStyle`
//! resources): the primitives a look composes into a non-photographic
//! frame. [`RenderScene::stylise`](crate::RenderScene::stylise) carries
//! them; each is `None` (or zero) by default and then costs nothing.
//!
//! This is the description the renderer reads. The passes that draw it
//! land with the shaders (the stylisation round's phase B);
//! [`STYLISE_DRAWN`] names what this renderer draws today, and the IDL's
//! `RenderCapabilities.stylise` reports it, so a look that names a
//! primitive the renderer lacks still applies and the rest of the frame is
//! drawn.

pub(crate) mod stylise_object;

/// The primitive groups this renderer draws, by the names
/// `RenderCapabilities.stylise` reports: `pixel`, `posterize`, `palette`,
/// `dither`, `halftone`, `hatching`, `paint`, `paintDetail` (the painterly
/// filter's object edges and detail), `watercolor` (the wash over the
/// painted frame, [`Watercolor`]), `ink`, `inkDepth`,
/// `depthBands`, `retro3d`, `surfaceSpace` (the canvas, hatching and
/// halftone laid on surfaces, [`StyliseSpace::Surface`]), `objectSpace`
/// (laid in each instance's own space, [`StyliseSpace::Object`]),
/// `ditherSpace` (the dither's cells on the surfaces, [`Quantize::space`]),
/// `halftoneCmyk` ([`HalftoneMode::Cmyk`], misregistration and
/// [`HalftoneRegion`]), `hatchingColor` ([`Hatching::keep_color`]) and
/// `hatchingPaper` ([`Hatching::paper`])
/// ([`Stylise`]);
/// `toon`, `shadowTint`, `shadowDarkness`, `rim`, `outline`, `outlineDistance`,
/// `flatShading`, `surface`, `toonAmbient` (with the anti-aliased bands and
/// the metals' bands), `outlinePlace` (lines outside the silhouette) and
/// `wireframe` ([`MaterialStyle`]).
pub const STYLISE_DRAWN: &[&str] = &[
    "pixel",
    "posterize",
    "palette",
    "dither",
    "halftone",
    "hatching",
    "paint",
    "paintDetail",
    "watercolor",
    "ink",
    "inkDepth",
    "depthBands",
    "retro3d",
    "surfaceSpace",
    "objectSpace",
    "ditherSpace",
    "halftoneCmyk",
    "hatchingColor",
    "hatchingPaper",
    "toon",
    "shadowTint",
    "shadowDarkness",
    "rim",
    "outline",
    "outlineDistance",
    "flatShading",
    "surface",
    "toonAmbient",
    "outlinePlace",
    "wireframe",
];

pub use crate::scene::PaletteHandle;

/// Up to 256 sRGB colours, in the order the palette file lists them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Palette {
    pub colors: Vec<[u8; 3]>,
}

/// The most colours a [`Palette`] holds.
pub const MAX_PALETTE_COLORS: usize = 256;

/// A colour's bucket in the conformed table: its sRGB bytes by sixteens,
/// so a pixel within stylise.wgsl's `CONFORMED_REACH` (6 levels a
/// channel: block compression and filtering move a flat conformed field
/// by a few) of a colour lies in at most two buckets per channel.
fn conformed_bucket(c: [u8; 3]) -> u32 {
    (u32::from(c[0] >> 4) << 8) | (u32::from(c[1] >> 4) << 4) | u32::from(c[2] >> 4)
}

/// A bucket's first slot (Fibonacci hashing) in a table of `2^bits` slots.
fn conformed_slot(bucket: u32, bits: u32) -> usize {
    ((bucket + 1).wrapping_mul(0x9E37_79B1) >> (32 - bits)) as usize
}

/// The conformed colours ([`crate::Scene::add_conformed_colors`]) as the
/// spatial hash stylise.wgsl's `conformed` probes: a square R32Uint
/// texture of side `size` (a power of two, 1 for none), its slots in
/// rows, each colour (its sRGB bytes with bit 24 set, so 0 is an empty
/// slot) at its bucket's slot or the next empty one, at most half the
/// slots full. Returns `(size, bytes)`.
pub(crate) fn conformed_table(colors: &[[u8; 3]]) -> (u32, Vec<u8>) {
    let mut size = 1u32;
    while colors.len() as u64 * 2 > u64::from(size) * u64::from(size) {
        size *= 2;
    }
    let slots = (size * size) as usize;
    let mut table = vec![0u32; slots];
    if !colors.is_empty() {
        let bits = slots.trailing_zeros();
        for &c in colors {
            let mut i = conformed_slot(conformed_bucket(c), bits);
            while table[i] != 0 {
                i = (i + 1) & (slots - 1);
            }
            table[i] =
                0x0100_0000 | (u32::from(c[0]) << 16) | (u32::from(c[1]) << 8) | u32::from(c[2]);
        }
    }
    (size, table.iter().flat_map(|k| k.to_le_bytes()).collect())
}

/// A low-resolution render target scaled up by whole numbers with nearest
/// filtering ([`PixelScale::fit`] places it in the output).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelScale {
    /// The logical height in pixels (the width follows the output's aspect).
    pub height: u32,
    /// The whole-number upscale; 0 takes the largest that fits the output.
    pub upscale: u32,
    /// Snap the camera and world sprites to the logical pixel grid.
    pub snap: bool,
}

/// Where a [`PixelScale`] grid lands in an output: its logical size, the
/// whole-number scale and the scaled frame's top-left corner (the border
/// round it is the camera's background).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelFit {
    pub width: u32,
    pub height: u32,
    pub upscale: u32,
    pub offset: (u32, u32),
}

impl PixelScale {
    /// The grid in an output of `out` pixels. The height decides the
    /// scale: `height` rows (at most the output's) at `upscale` when
    /// `height × upscale` fits the output's height, else (and for 0) at
    /// the largest whole number that does. The width is the output's
    /// aspect times the height, rounded, but at most the output's width
    /// over the scale: the scale wins over the last column. Since the
    /// scale fits the height, at most one column is dropped (240 rows at
    /// 1280 × 720 are 426.67 columns, rounded to 427, and 427 × 3 = 1281
    /// does not fit: 426 columns at 3x, 1278 pixels, a 1-pixel border each
    /// side, where the grid used to fall back to 2x inside a 213-pixel
    /// border). Pixels stay square: the scene's aspect is the grid's.
    pub fn fit(&self, out: (u32, u32)) -> PixelFit {
        let out = (out.0.max(1), out.1.max(1));
        let h = self.height.clamp(1, out.1);
        let largest = out.1 / h;
        let up = if self.upscale > 0 && self.upscale <= largest {
            self.upscale
        } else {
            largest
        };
        let w = ((h as f64 * out.0 as f64 / out.1 as f64).round() as u32).clamp(1, out.0 / up);
        PixelFit {
            width: w,
            height: h,
            upscale: up,
            offset: ((out.0 - w * up) / 2, (out.1 - h * up) / 2),
        }
    }
}

/// The threshold pattern quantisation is dithered with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DitherPattern {
    #[default]
    None,
    Bayer2,
    Bayer4,
    Bayer8,
    /// A 64 × 64 void-and-cluster tile (`blue_noise.rs`), repeated.
    BlueNoise,
    /// Interleaved gradient noise (Jimenez), computed per pixel: no
    /// texture, a diagonal structure up close.
    InterleavedGradient,
}

/// Fewer tones and a fixed palette, mixed by a dither.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quantize {
    /// Tones per channel (2 to 64); 0 leaves the channels continuous.
    pub posterize: u32,
    /// How the tones step a colour: each channel apart, or its value and
    /// tint apart.
    pub mode: PosterizeMode,
    /// The palette every pixel is moved to the nearest colour of.
    pub palette: Option<PaletteHandle>,
    pub dither: DitherPattern,
    /// The dither's amplitude in steps.
    pub dither_strength: f32,
    /// One dither cell in logical pixels.
    pub dither_scale: u32,
    /// Where the dither's cells lie: over the frame, or on the surfaces
    /// (world or each instance's space, at least `dither_scale` pixels on
    /// the screen along both axes, each of the surface's axes by whole
    /// octaves, so a receding floor's cells are long along its depth), so a pattern stays on what it shades as the camera
    /// moves instead of crawling over the screen; the sky's cells lie on a
    /// cube round the camera (Lucas Pope's Obra Dinn). On surfaces the
    /// frame runs the prepass.
    pub space: StyliseSpace,
}

/// Where a pattern (the canvas, the hatching, the halftone) is laid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StyliseSpace {
    /// Over the frame, in logical pixels: the pattern stays put on the
    /// screen as the camera moves.
    #[default]
    Screen,
    /// On the surfaces, through the depth-normal prepass's position and
    /// normal (triplanar in world space): the pattern sticks to the scene
    /// as the camera moves and wraps round forms. The sky keeps the screen
    /// pattern. The frame runs the prepass.
    Surface,
    /// On the surfaces as [`StyliseSpace::Surface`], but triplanar in each
    /// instance's own space: a moving or turning object carries its
    /// strokes, dots and grain instead of sliding through them. The frame
    /// runs the prepass and then the object pass (`stylise_object.rs`),
    /// which draws the opaque mesh instances again into an `Rgba32Uint`
    /// target holding each pixel's position in its instance's space
    /// (metres: the mesh position times the instance's axis scales, so
    /// the pattern keeps its size on a scaled instance) and its normal
    /// there. Pixels no instance covers (terrain, scattered copies, a
    /// material shader's moved vertices) keep the world pattern, the sky
    /// the screen pattern.
    Object,
}

impl StyliseSpace {
    /// Laid on the surfaces (world or object space).
    pub fn on_surfaces(self) -> bool {
        self != StyliseSpace::Screen
    }

    /// The shader's code for the space: 0 screen, 1 world, 2 object.
    pub(crate) fn code(self) -> f32 {
        match self {
            StyliseSpace::Screen => 0.0,
            StyliseSpace::Surface => 1.0,
            StyliseSpace::Object => 2.0,
        }
    }
}

/// How [`Quantize::posterize`] steps a colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PosterizeMode {
    /// Each display channel rounded to its own step: a colour off the grey
    /// axis can change hue as light moves it across one channel's step
    /// before the others.
    #[default]
    Channel,
    /// The value (the largest display channel) and the tint (the channels
    /// over it) stepped apart: light scales the value alone, so a colour
    /// keeps its hue through exposure, grading and moods.
    Tone,
}

/// How the halftone inks the frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HalftoneMode {
    /// One ink ([`Halftone::color`]) on paper, larger dots where darker.
    #[default]
    Ink,
    /// Four-colour process print: the colour split into cyan, magenta,
    /// yellow and black screens at their classic angles (black at the
    /// grid's angle, cyan 30° before it, magenta 30° after, yellow 45°
    /// before), multiplied on white paper.
    Cmyk,
}

/// Which tones the halftone prints on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HalftoneRegion {
    #[default]
    All,
    /// The shade only (Ben-Day dots in the shadows); light tones keep
    /// their colour.
    Shade,
    /// The highlights only.
    Highlight,
}

/// Halftone dots on a rotated grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Halftone {
    pub strength: f32,
    /// The grid's pitch in logical pixels (on surfaces: about that many
    /// pixels where the dots lie, by whole octaves).
    pub size: f32,
    pub angle_degrees: f32,
    /// The dots' linear colour ([`HalftoneMode::Ink`]).
    pub color: [f32; 3],
    pub space: StyliseSpace,
    pub mode: HalftoneMode,
    /// With [`HalftoneMode::Cmyk`]: how far the cyan, magenta and yellow
    /// plates sit off the black one, in logical pixels (0: in register).
    pub misregistration: f32,
    pub region: HalftoneRegion,
}

/// Cross-hatching from a tonal art map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hatching {
    pub strength: f32,
    /// Line spacing in logical pixels (on surfaces: about that many pixels
    /// where the strokes lie; strokes fade in and out by whole octaves of
    /// world spacing, as a tonal art map's levels).
    pub scale: f32,
    pub levels: u32,
    pub angle_degrees: f32,
    pub color: [f32; 3],
    /// The paper the strokes lie on (linear): the ground lifts towards it,
    /// and under [`Hatching::keep_color`] a weak tint takes its tint.
    /// White keeps the frame's own.
    pub paper: [f32; 3],
    pub space: StyliseSpace,
    /// How strongly the depth-normal prepass's silhouettes and creases
    /// darken the tone the strokes follow, 0 to 1, as [`Ink::depth`] inks
    /// them: strokes gather along contours and folds that the tone alone
    /// leaves light. The edges are [`Ink`]'s (`depth_threshold`,
    /// `normal_threshold`; the outline pass's defaults without ink), found
    /// within one stroke spacing. 0 is the tone alone; above 0 the frame
    /// runs the prepass.
    pub depth: f32,
    /// Colour hatching: the ground keeps its colour and each stroke is
    /// that colour darkened towards [`Hatching::color`], instead of
    /// strokes of one ink over a paper-light ground.
    pub keep_color: bool,
}

/// Painterly smoothing, pigment pooling and a canvas (stylise.wgsl
/// `paint`): an anisotropic Kuwahara filter with eight sectors over a
/// smoothed structure tensor (Kyprianidis 2009 and 2010, Acerola's port),
/// which keeps coherent structure (edges, thin lines) and paints over
/// incoherent texture, aware of objects and of detail where the frame has
/// the depth-normal prepass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Paint {
    /// The filter's radius in logical pixels (0: no smoothing).
    pub radius: f32,
    /// 0 round brush cells, 1 thin strokes along coherent structure (the
    /// ellipse's α from 1 at 0.5 to its floor at 1).
    pub anisotropy: f32,
    pub edge_darkening: f32,
    pub bleed: f32,
    /// The paper or canvas texture and how strongly it modulates the frame.
    pub canvas: Option<crate::TextureHandle>,
    pub canvas_strength: f32,
    /// Canvas texels per logical pixel ([`StyliseSpace::Screen`]).
    pub canvas_scale: f32,
    /// Where the canvas is laid: over the frame, or on the surfaces so the
    /// grain does not slide over the scene as the camera moves.
    pub canvas_space: StyliseSpace,
    /// Canvas texels per metre on surfaces ([`StyliseSpace::Surface`]);
    /// 0 is [`DEFAULT_CANVAS_WORLD_SCALE`].
    pub canvas_world_scale: f32,
    /// How far detail keeps its shape, 0 to 1: a pixel's radius shrinks by
    /// its detail (its material's `paintDetail`, or `character_detail` on
    /// a skinned instance) times this, and no tap crosses an object's edge.
    /// Above 0 the frame runs the prepass when it draws meshes; 0 paints
    /// every pixel alike, over the objects' edges.
    pub detail: f32,
    /// The detail of a skinned instance's pixels (a character's face and
    /// hands), 0 to 1, where its material sets no more.
    pub character_detail: f32,
    /// The watercolour wash drawn over the painted frame.
    pub watercolor: Watercolor,
}

/// The watercolour wash (stylise.wgsl `watercolor`, a pass after `paint`
/// that reads the painted frame): Bousseau et al. 2006's pigment density
/// about 1, which thins a colour towards the paper or thickens it, from
/// the paper's granulation and a slow turbulence; MNPR's (Montesdeoca et
/// al. 2018, `third_party/mnpr`) edge darkening as a density in a rim
/// where the painted regions meet, dry brush over the paper's peaks and
/// substrate distortion. Every field 0 draws no pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Watercolor {
    /// Pigment pooled in a thin rim at the painted regions' edges, 0 to 1.
    pub edge_darkening: f32,
    /// The density's amplitude from the paper's grain ([`Paint::canvas`]).
    pub granulation: f32,
    /// The density's amplitude from a smooth noise a few dozen pixels
    /// across, where the canvas lies.
    pub turbulence: f32,
    /// The paper's peaks left bare where the wash is thin, 0 to 1.
    pub dry_brush: f32,
    /// How far the paper's relief displaces the frame, logical pixels.
    pub wobble: f32,
    /// How far the wash is thinned with water, 0 to 1: the pigment's
    /// density scaled by one minus this, so the darks lift towards the
    /// paper.
    pub dilution: f32,
    /// The paper's colour (linear), what the wash thins towards.
    pub paper: [f32; 3],
}

impl Watercolor {
    pub const DEFAULT: Watercolor = Watercolor {
        edge_darkening: 0.0,
        granulation: 0.0,
        turbulence: 0.0,
        dry_brush: 0.0,
        wobble: 0.0,
        dilution: 0.0,
        paper: [1.0; 3],
    };

    /// Whether the wash draws at all.
    pub fn active(&self) -> bool {
        self.edge_darkening > 0.0
            || self.dilution > 0.0
            || self.granulation > 0.0
            || self.turbulence > 0.0
            || self.dry_brush > 0.0
            || self.wobble > 0.0
    }

    /// Whether the wash reads the paper's grain.
    pub fn reads_paper(&self) -> bool {
        self.granulation > 0.0 || self.dry_brush > 0.0 || self.wobble > 0.0
    }

    /// Whether the wash reads its outer boundaries (the prepass's object
    /// edges, silhouettes and creases): where its rim pools and its dry
    /// brush skips.
    pub fn edges(&self) -> bool {
        self.edge_darkening > 0.0 || self.dry_brush > 0.0
    }
}

/// The most [`Watercolor::dilution`] thins a wash: past it the frame is
/// the paper.
pub const MAX_WASH_DILUTION: f32 = 0.9;

/// [`Paint::detail`]'s default: detail keeps its shape fully.
pub const DEFAULT_PAINT_DETAIL: f32 = 1.0;

/// [`Paint::character_detail`]'s default: a character keeps most of its
/// face and hands (a fifth of the radius left).
pub const DEFAULT_CHARACTER_DETAIL: f32 = 0.8;

/// [`Paint::canvas_world_scale`] when it is 0: a 256² canvas repeats every
/// two metres.
pub const DEFAULT_CANVAS_WORLD_SCALE: f32 = 128.0;

impl Paint {
    /// The IDL's defaults: every field off but the detail awareness.
    pub const DEFAULT: Paint = Paint {
        radius: 0.0,
        anisotropy: 0.0,
        edge_darkening: 0.0,
        bleed: 0.0,
        canvas: None,
        canvas_strength: 0.0,
        canvas_scale: 1.0,
        canvas_space: StyliseSpace::Screen,
        canvas_world_scale: 0.0,
        detail: DEFAULT_PAINT_DETAIL,
        character_detail: DEFAULT_CHARACTER_DETAIL,
        watercolor: Watercolor::DEFAULT,
    };

    /// Whether the filter reads the prepass's object ids and detail.
    pub fn detail_aware(&self) -> bool {
        self.radius > 0.0 && self.detail > 0.0
    }

    /// Whether the paint pass draws (the Kuwahara, the bleed, the pooling
    /// or the canvas's grain): the watercolour wash alone draws only its
    /// own pass.
    pub fn paints(&self) -> bool {
        self.radius > 0.0
            || self.edge_darkening > 0.0
            || self.bleed > 0.0
            || (self.canvas.is_some() && self.canvas_strength > 0.0)
    }

    /// Whether the frame binds the canvas texture: laid over the frame by
    /// `canvas_strength`, or read as the wash's paper.
    pub fn uses_canvas(&self) -> bool {
        self.canvas.is_some() && (self.canvas_strength > 0.0 || self.watercolor.reads_paper())
    }

    /// Whether the canvas is drawn on surfaces (the frame runs the prepass).
    pub fn canvas_on_surfaces(&self) -> bool {
        self.canvas_space.on_surfaces() && self.uses_canvas()
    }
}

/// Ink lines on luminance edges, and with `depth` on the prepass's depth
/// and normal edges too.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ink {
    pub threshold: f32,
    /// Line width in logical pixels.
    pub width: f32,
    pub color: [f32; 3],
    /// How strongly lines are drawn where the depth-normal prepass breaks
    /// (silhouettes against what is behind, creases), 0 to 1: lines on
    /// flat colour that luminance alone misses. 0 is luminance only; above
    /// 0 the frame runs the prepass.
    pub depth: f32,
    /// With `depth`: the relative depth step to a farther neighbour that is
    /// a silhouette (0.04: 4 %), as `Outline::depth_threshold`.
    pub depth_threshold: f32,
    /// With `depth`: the normal turn between neighbours of about the same
    /// depth that is a crease (1 − cos of the angle; 0.4 ≈ 53°), as
    /// `Outline::normal_threshold`.
    pub normal_threshold: f32,
}

impl Default for Ink {
    /// The IDL's defaults: luminance edges only, the outline pass's
    /// thresholds for depth edges.
    fn default() -> Self {
        Ink {
            threshold: 0.1,
            width: 1.0,
            color: [0.02, 0.02, 0.03],
            depth: 0.0,
            depth_threshold: 0.04,
            normal_threshold: 0.4,
        }
    }
}

/// Colour bands by depth (fxpost.wgsl `fs_depth_bands`): `count` equal
/// layers from `near` to `far`, the first `near_color`, the last
/// `far_color` (and everything beyond `far`), those between stepping
/// evenly in OkLCh (`band_color`: lightness and chroma straight, hue the
/// short way round), never through grey.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepthBands {
    /// 2 to 8 bands.
    pub count: u32,
    pub near: f32,
    pub far: f32,
    pub near_color: [f32; 3],
    pub far_color: [f32; 3],
    /// 1 flat silhouettes in the band's colour; 0 a tint: the lit colour
    /// times the band's at unit luminance, the shading kept; between, a mix.
    pub flatten: f32,
}

/// The vertex and depth artefacts of early 3D hardware.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Retro3d {
    /// The screen grid vertices snap to, in logical pixels (0: none).
    pub vertex_snap: f32,
    /// 0 perspective-correct, 1 affine texture coordinates.
    pub affine: f32,
    /// Depth quantised to this many bits (0: full precision).
    pub depth_bits: u32,
}

impl Retro3d {
    /// The frame's `stylise` uniform (frame.wgsl) for `retro` on a
    /// `width` by `height` target: the snap grid in cells, the affine
    /// share and the depth bits, all 0 without retro 3D. pbr.wgsl
    /// `retro_clip` and the visibility buffer's rasteriser
    /// (meshlet_raster.wgsl) read the same four numbers.
    pub fn uniform(retro: Option<Retro3d>, width: u32, height: u32) -> [f32; 4] {
        match retro {
            Some(r) if r.vertex_snap > 0.0 || r.affine > 0.0 || r.depth_bits > 0 => {
                let cells = |size: u32| {
                    if r.vertex_snap > 0.0 {
                        size as f32 / r.vertex_snap
                    } else {
                        0.0
                    }
                };
                [
                    cells(width),
                    cells(height),
                    r.affine.clamp(0.0, 1.0),
                    r.depth_bits.min(24) as f32,
                ]
            }
            _ => [0.0; 4],
        }
    }
}

/// Stylised defaults for every lit material that leaves the field at its
/// own default (the IDL's `MaterialStyle`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialStyle {
    pub toon: bool,
    pub gradient_map: Option<crate::TextureHandle>,
    /// The linear colour shadowed light moves towards, and how far.
    pub shadow_tint: [f32; 3],
    pub shadow_tint_amount: f32,
    /// How far shadowed surfaces fall towards black: their share of the
    /// sun (the toon ramp's dark side, the side turned away, the cast
    /// shadow) loses this much of the sun's ramp and the ambient light
    /// (environment and hemisphere) too; 1 is a solid black mass, 0 off.
    pub shadow_darkness: f32,
    pub rim_color: [f32; 3],
    pub rim_intensity: f32,
    pub rim_power: f32,
    pub outline_color: [f32; 3],
    pub outline_width: f32,
    /// The distance at which outlines are `outline_width` wide (0: constant).
    pub outline_distance: f32,
    pub flat_shading: bool,
    /// A made surface on every lit material (the IDL's `MaterialSurface`).
    pub surface: MaterialSurface,
    /// How strongly it shows (0: not at all).
    pub surface_strength: f32,
    /// Its pattern's size in metres of the mesh's own space (0: the
    /// surface's own, [`MaterialSurface::scale`]).
    pub surface_scale: f32,
    /// How smooth a toon material's ambient light (the environment and
    /// the hemisphere light) stays: 0 one flat colour, the light from
    /// above and below averaged (MToon's equalised ambient), with a flat
    /// reflection of the environment, so the bands sit on flat colour;
    /// 1 the smooth PBR ambient and reflections.
    pub toon_ambient: f32,
    /// The toon bands' and highlight's edges, anti-aliased over this many
    /// pixels (lilToon's screen-space smoothing); 0 hard steps.
    pub toon_softness: f32,
    /// Where outlines lie against the shape (the outline pass, and
    /// `ArtLine.place`).
    pub outline_place: OutlinePlace,
    /// Every lit material's feature edges (`wireframe_angle`) drawn over
    /// its faces, in the colour its surface shows along them (its colour
    /// and maps plus its emissive light) or `wireframe_color`, the faces hiding
    /// the edges behind them: vector and neon looks.
    pub wireframe: bool,
    /// The crease angle of `wireframe` in degrees (0: every edge).
    pub wireframe_angle: f32,
    /// The look's line colour for `wireframe` (linear); black: each edge
    /// in the colour its surface shows there.
    pub wireframe_color: [f32; 3],
}

/// Where an outline lies against its object's silhouette.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum OutlinePlace {
    /// On the object's own pixels, inside its silhouette.
    #[default]
    Inside,
    /// On the pixels round it, outside its silhouette: the shape keeps
    /// its own pixels whole.
    Outside,
    /// Half its width each side.
    Both,
}

impl OutlinePlace {
    /// The shader's code: 0 inside, 1 outside, 2 both.
    pub(crate) fn code(self) -> f32 {
        match self {
            OutlinePlace::Inside => 0.0,
            OutlinePlace::Outside => 1.0,
            OutlinePlace::Both => 2.0,
        }
    }
}

/// [`MaterialStyle::wireframe_angle`]'s default (the IDL's): creases past
/// 30 degrees, so a smooth surface keeps only its silhouette's borders.
pub const DEFAULT_WIREFRAME_ANGLE: f32 = 30.0;

/// [`MaterialStyle::toon_softness`]'s default (the IDL's): one pixel.
pub const DEFAULT_TOON_SOFTNESS: f32 = 1.0;

impl Default for MaterialStyle {
    /// Off, as the IDL's defaults: no field changes a material; a toon
    /// material's bands are anti-aliased over a pixel and its ambient flat.
    fn default() -> Self {
        MaterialStyle {
            toon: false,
            gradient_map: None,
            shadow_tint: [0.0; 3],
            shadow_tint_amount: 0.0,
            shadow_darkness: 0.0,
            rim_color: [0.0; 3],
            rim_intensity: 0.0,
            rim_power: 0.0,
            outline_color: [0.0; 3],
            outline_width: 0.0,
            outline_distance: 0.0,
            flat_shading: false,
            surface: MaterialSurface::None,
            surface_strength: 0.0,
            surface_scale: 0.0,
            toon_ambient: 0.0,
            toon_softness: DEFAULT_TOON_SOFTNESS,
            outline_place: OutlinePlace::Inside,
            wireframe: false,
            wireframe_angle: DEFAULT_WIREFRAME_ANGLE,
            wireframe_color: [0.0; 3],
        }
    }
}

/// A made surface the material style lays on every lit material: its
/// normal detail, colour grain and light response, drawn by the lit
/// shader (`pbr_fs.wgsl`, `material_surface`) in each mesh's own space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MaterialSurface {
    #[default]
    None,
    /// Lumps, thumb prints, wrapped warm light, matte.
    Clay,
    /// Fibre grain in colour and normal, a little wrap, fully rough.
    Paper,
    /// A satin sheen, a soft wrap, a faint moulding ripple.
    Plastic,
}

impl MaterialSurface {
    /// The variant by its IDL index (`none`, `clay`, `paper`, `plastic`).
    pub fn from_index(i: usize) -> MaterialSurface {
        match i {
            1 => MaterialSurface::Clay,
            2 => MaterialSurface::Paper,
            3 => MaterialSurface::Plastic,
            _ => MaterialSurface::None,
        }
    }

    /// Its pattern's own size in metres.
    pub fn scale(self) -> f32 {
        match self {
            MaterialSurface::None => 0.0,
            MaterialSurface::Clay => 0.12,
            MaterialSurface::Paper => 0.15,
            MaterialSurface::Plastic => 0.3,
        }
    }
}

impl MaterialStyle {
    /// Whether it changes any material.
    pub fn is_active(&self) -> bool {
        self.toon
            || self.shadow_tint_amount > 0.0
            || self.shadow_darkness > 0.0
            || self.rim_intensity > 0.0
            || self.outline_width > 0.0
            || self.flat_shading
            || self.wireframe
            || self.surface_on()
    }

    /// Whether a surface is laid on the materials.
    pub fn surface_on(&self) -> bool {
        self.surface != MaterialSurface::None && self.surface_strength > 0.0
    }

    /// The frame's `material_surface` (frame.wgsl): x the surface (0 none,
    /// 1 clay, 2 paper, 3 plastic), y its strength, z its scale in metres.
    pub(crate) fn surface_uniform(&self) -> [f32; 4] {
        if !self.surface_on() {
            return [0.0; 4];
        }
        let scale = if self.surface_scale > 0.0 {
            self.surface_scale
        } else {
            self.surface.scale()
        };
        [
            self.surface as u32 as f32,
            self.surface_strength.clamp(0.0, 1.0),
            scale.max(1e-3),
            0.0,
        ]
    }
}

/// The stylisation of one frame. Every primitive is off by default.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stylise {
    pub pixel: Option<PixelScale>,
    pub quantize: Option<Quantize>,
    pub halftone: Option<Halftone>,
    pub hatching: Option<Hatching>,
    pub paint: Option<Paint>,
    pub ink: Option<Ink>,
    pub depth_bands: Option<DepthBands>,
    pub retro: Option<Retro3d>,
    pub material: MaterialStyle,
}

impl Stylise {
    /// Whether a pattern is laid on surfaces: the frame runs the
    /// depth-normal prepass and the paint or lines pass reads it.
    pub fn on_surfaces(&self) -> bool {
        self.surface_spaces()
            .iter()
            .any(|s| s.is_some_and(StyliseSpace::on_surfaces))
    }

    /// Whether the halftone prints under a region (Ben-Day dots), whose
    /// dots fade on surfaces seen edge-on: the frame runs the prepass for
    /// their normals where it draws meshes.
    pub fn halftone_faces(&self) -> bool {
        self.halftone
            .is_some_and(|h| h.strength > 0.0 && h.region != HalftoneRegion::All)
    }

    /// Whether a pattern is laid in object space: the frame runs the
    /// object pass after the prepass.
    pub fn in_object_space(&self) -> bool {
        self.surface_spaces().contains(&Some(StyliseSpace::Object))
    }

    /// The space of the dither, the canvas, the hatching and the halftone where each
    /// is drawn (`None` where it is not).
    fn surface_spaces(&self) -> [Option<StyliseSpace>; 4] {
        [
            self.quantize
                .filter(|q| q.dither != DitherPattern::None)
                .map(|q| q.space),
            self.paint
                .filter(Paint::uses_canvas)
                .map(|p| p.canvas_space),
            self.hatching.filter(|h| h.strength > 0.0).map(|h| h.space),
            self.halftone.filter(|h| h.strength > 0.0).map(|h| h.space),
        ]
    }

    /// Whether a pass reads the depth-normal prepass's edges: ink or
    /// hatching with `depth` above 0.
    pub fn depth_edges(&self) -> bool {
        self.ink.is_some_and(|i| i.depth > 0.0)
            || self
                .hatching
                .is_some_and(|h| h.depth > 0.0 && h.strength > 0.0)
    }

    /// Whether any primitive is on.
    pub fn is_active(&self) -> bool {
        self.pixel.is_some()
            || self.quantize.is_some()
            || self.halftone.is_some()
            || self.hatching.is_some()
            || self.paint.is_some()
            || self.ink.is_some()
            || self.depth_bands.is_some()
            || self.retro.is_some()
            || self.material.is_active()
    }

    /// Which primitives are on, one bit each (not their settings): what
    /// passes the frame runs (depth bands, paint, lines, quantise), whether
    /// it is drawn on a pixel grid, and whether retro 3D or the material
    /// style change the lit pass. The renderer resets the TAA history when
    /// this changes between two frames, as on a camera cut, so the first
    /// frame of a new look holds no ghost of the old one; a change of a
    /// setting alone (a posterize level, a palette) keeps it.
    pub fn chain_bits(&self) -> u16 {
        [
            self.pixel.is_some(),
            self.quantize.is_some(),
            self.halftone.is_some(),
            self.hatching.is_some(),
            self.paint.is_some_and(|p| p.paints()),
            self.ink.is_some(),
            self.paint.is_some_and(|p| p.watercolor.active()),
            self.depth_bands.is_some(),
            self.retro.is_some(),
            self.material.is_active(),
        ]
        .iter()
        .enumerate()
        .fold(0, |bits, (i, on)| bits | (u16::from(*on) << i))
    }
}

#[cfg(test)]
mod tests;
