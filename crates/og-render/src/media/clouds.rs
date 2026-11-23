//! Clouds: a volumetric layer raymarched in the sky pass over a curved shell
//! around the planet, and a thin high layer (cirrus) over it.
//!
//! Their shape is a dimensional profile eroded by noise, after Schneider's
//! "Nubis, Evolved" and "Nubis³" (`third_party/nubis-clouds.md`): a baked
//! map says how far inside a cloud each column of the layer stands and how
//! tall the cloud is there, the [`CloudKind`]'s numbers stand it on a flat
//! base and round its top, and two tiling noise volumes (billows of some
//! hundreds of metres, and of some tens) cut its surface into bulges and
//! creases (`clouds_noise.wgsl`, `clouds_shape.wgsl`). All three fields are
//! made once on the GPU the first frame that has clouds ([`CloudNoise`]).
//!
//! They are lit by the sun (under the physical sky, the sunlight through
//! the air down to each sample's own height, so a sunset lights low clouds
//! orange from below; else the frame's directional light) through the cloud
//! between: two samples near, then a Beer shadow map of the whole field
//! (`clouds/shadow.rs`), scattered by a phase function fitted to cloud
//! droplets and by octaves of multiple scattering; by the sky above them
//! and the ground below, each through the cloud between; and by a lightning
//! strike's flash inside them ([`Clouds::flash`]). sky.wgsl `clouds_march`.
//!
//! The wind moves them by the tick's seconds, so they look the same on
//! every run and after a restore; nothing in the simulation reads them.
//!
//! They are drawn over any background, including authored environment
//! maps: the sky pass draws them, so they do
//! not light the scene by themselves; [`Clouds::light`] is how much a
//! cover dims the sun and the sky's light under it, which the runtime
//! applies.

use glam::Vec3;

mod shadow;
pub(crate) use shadow::{CloudShadow, bake_source};

/// The shape volume's size per side and its levels (clouds_noise.wgsl).
const SHAPE_SIZE: u32 = 128;
const SHAPE_LEVELS: u32 = 4;
/// The detail volume's.
const DETAIL_SIZE: u32 = 64;
const DETAIL_LEVELS: u32 = 3;
/// The map's side: the whole field, four tiles of [`Clouds::scale`].
const MAP_SIZE: u32 = 512;
/// The vec4s of the frame uniform's cloud block (`frame.clouds`).
pub(crate) const CLOUD_VEC4S: usize = 15;

/// What kind of cloud the volumetric layer holds: the shape its clouds
/// take, whatever their cover.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CloudKind {
    /// Heaped clouds on a flat base, their tops domes of billows.
    #[default]
    Cumulus,
    /// A flat, even sheet low in the layer.
    Stratus,
    /// A low sheet of rolls and patches with gaps between.
    Stratocumulus,
    /// Towers through the whole layer, the strongest spreading into an
    /// anvil under its top.
    Cumulonimbus,
}

/// A [`CloudKind`]'s numbers (clouds_shape.wgsl `cloud_column`,
/// `cloud_profile`, `cloud_density_from`).
struct KindShape {
    /// The share of a cloud's height at which it is widest.
    girth: f32,
    /// How far up the layer a cloud stands in its core.
    height: f32,
    /// The power a cloud's top rounds off by: 2 a dome, more a tower
    /// that stands wide to a flat top, less a sheet whose profile falls
    /// from its widest height all the way up.
    power: f32,
    /// How much a cloud narrows from its widest down to its base.
    under: f32,
    /// How far a vigorous tower spreads under the layer's top.
    anvil: f32,
    /// Which of the map's fields places the clouds: 0 the broad one, 1 the
    /// one of closer cells.
    cells: f32,
    /// How much of the field a cloud's side takes from its edge to its core.
    edge: f32,
    /// How deep the noise erodes (of the profile).
    erosion: f32,
    /// How much the tops' heights vary from cloud to cloud.
    vary: f32,
    /// The share of a cloud's height over its base in which it becomes
    /// whole: a cumulus's base is nearly flat, a sheet's hangs ragged.
    base: f32,
}

impl CloudKind {
    fn shape(self) -> KindShape {
        match self {
            // (A heap is widest just over its base and its sides lean from
            // there: by a dome's power, 2, they stood upright over the
            // base for a sixth of the cloud's height, a skirt under every
            // dome.)
            CloudKind::Cumulus => KindShape {
                girth: 0.06,
                height: 1.0,
                power: 1.4,
                under: 0.6,
                anvil: 0.0,
                cells: 0.0,
                edge: 0.45,
                erosion: 0.75,
                vary: 0.7,
                base: 0.16,
            },
            // (A sheet's relief is its tops' erosion: the profile falls
            // from the widest height all the way up (power 1) and the
            // noise erodes nearly all of it, so the billows stand some
            // 80 m out of a sheet 450 m deep. By a dome's profile eroded
            // as a heap's is, they stood 20 m: a sheet as smooth as a
            // table.)
            CloudKind::Stratus => KindShape {
                girth: 0.3,
                height: 0.28,
                power: 1.0,
                under: 0.3,
                anvil: 0.0,
                cells: 0.0,
                edge: 0.6,
                erosion: 0.9,
                vary: 0.8,
                base: 0.2,
            },
            CloudKind::Stratocumulus => KindShape {
                girth: 0.3,
                height: 0.5,
                power: 1.5,
                under: 0.5,
                anvil: 0.0,
                cells: 1.0,
                edge: 0.7,
                erosion: 0.9,
                vary: 0.4,
                base: 0.3,
            },
            CloudKind::Cumulonimbus => KindShape {
                girth: 0.1,
                height: 1.0,
                power: 5.0,
                under: 0.5,
                anvil: 1.2,
                cells: 0.0,
                edge: 0.4,
                erosion: 0.8,
                vary: 0.3,
                base: 0.05,
            },
        }
    }
}
/// The first of the clouds' block's rows that hold the clear sky's light
/// ([`crate::atmosphere::AtmosphereGpu::new`]): overhead, then horizontal
/// towards the sun, away from it and across.
pub(crate) const CLEAR_SKY_ROW: usize = 6;

/// Clouds over the scene. See the module documentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clouds {
    /// What kind of cloud the volumetric layer holds.
    pub kind: CloudKind,
    /// How much of the sky the volumetric layer covers (0: none, 1: overcast).
    pub coverage: f32,
    /// The extinction per metre of a cloud's body (0.04 to 0.12: cumulus;
    /// lower: thin, translucent clouds).
    pub density: f32,
    /// The layer's base and thickness, in metres above the world's y 0
    /// (thickness 0: no volumetric layer).
    pub base: f32,
    pub thickness: f32,
    /// The size of the largest cloud shapes, in metres: the cloud field
    /// repeats every four of it, and a cloud is a sixth to a third of it
    /// across.
    pub scale: f32,
    /// How far the wind carries them per second (x and z; y turns the
    /// shapes over as they go).
    pub wind: Vec3,
    /// Their albedo (white for water clouds; a grey for storm clouds).
    pub color: Vec3,
    /// How much the noise erodes their surface (0: smooth lumps, 0.35:
    /// billows, 1: ragged and wispy).
    pub detail: f32,
    /// The high thin layer's cover (0: none) and height in metres.
    pub layer: f32,
    pub layer_altitude: f32,
    /// How dark the layer's shadow falls on the world: 1 the shadow it
    /// casts, 0 none (`clouds/shadow.rs`).
    pub shadows: f32,
    /// Samples along a view ray through the layer (16 to 128).
    pub steps: u32,
    /// A lightning flash lighting the cloud from inside: where (frame
    /// coordinates, xyz) and how bright (w; 0: none). Set by
    /// [`crate::add_lightning`].
    pub flash: glam::Vec4,
}

impl Default for Clouds {
    fn default() -> Self {
        Clouds {
            kind: CloudKind::Cumulus,
            coverage: 0.45,
            density: 0.04,
            base: 1500.0,
            thickness: 900.0,
            scale: 9000.0,
            wind: Vec3::new(12.0, 0.4, 4.0),
            color: Vec3::ONE,
            detail: 0.35,
            layer: 0.3,
            layer_altitude: 8000.0,
            shadows: 1.0,
            steps: 48,
            flash: glam::Vec4::ZERO,
        }
    }
}

impl Clouds {
    /// The frame uniform's block at `seconds` of the clock, the planet of
    /// `radius` metres: (coverage, density, base, thickness), (the wind's
    /// offset, scale), (colour, detail), (layer, layer altitude, steps, on),
    /// (radius, the sun's and the sky's dimming by [`Clouds::light`],
    /// which the scene's light is assumed to carry), after the clear
    /// sky's rows where the eye stands in the world (`eye`: its x and z):
    /// the clouds stand over the world, not over the eye; and the kind's
    /// numbers (clouds_shape.wgsl), with the high layer's place. The offset
    /// and the eye's place wrap every four tiles (a period of every scale
    /// the sky reads the volumetric layer's fields at), and the high
    /// layer's place every sixteen (a period of its own, which are
    /// broader), so f32 keeps its precision however long the clock runs and
    /// however far the eye goes.
    pub(crate) fn gpu(
        &self,
        seconds: f64,
        radius: f32,
        eye: glam::DVec2,
    ) -> [[f32; 4]; CLOUD_VEC4S] {
        let scale = self.scale.clamp(500.0, 200_000.0);
        let period = f64::from(scale) * 4.0;
        let offset = |v: f32| (f64::from(v) * seconds).rem_euclid(period) as f32;
        let place = |v: f64| v.rem_euclid(period) as f32;
        // The high layer's place: the eye's less the offset of the wind up
        // there, twice the layer's below.
        let high = |eye: f64, wind: f32| {
            (eye - 2.0 * f64::from(wind) * seconds).rem_euclid(period * 4.0) as f32
        };
        let light = self.light();
        let kind = self.kind.shape();
        let detail = self.detail.clamp(0.0, 1.0);
        [
            [
                self.coverage.clamp(0.0, 1.0),
                self.density.max(0.0),
                self.base,
                self.thickness.max(0.0),
            ],
            [
                offset(self.wind.x),
                offset(self.wind.y),
                offset(self.wind.z),
                scale,
            ],
            self.color.max(Vec3::ZERO).extend(detail).to_array(),
            [
                self.layer.clamp(0.0, 1.0),
                self.layer_altitude.max(self.base + self.thickness),
                self.steps.clamp(8, 256) as f32,
                1.0,
            ],
            // The runtime dims scene sunlight and ambient under this
            // cover. Clouds undo both: they are lit above their cover.
            [radius, light.0, light.1, 0.0],
            self.flash.to_array(),
            // The clear sky's light, filled by AtmosphereGpu::new (rgb), and
            // in this first row's w the sea-level height of world y 0
            // (og.PhysicalSky's altitude): sky.wgsl's cloud shell, its
            // sunlight and the ground below it read it as `clouds[6].w`.
            [0.0; 4],
            [0.0; 4],
            [0.0; 4],
            [0.0; 4],
            [place(eye.x), place(eye.y), 0.0, 0.0],
            [0.0; 4],
            [kind.girth, kind.height, kind.power, kind.anvil],
            // No detail: smooth lumps, eroded half as deep.
            [
                kind.cells,
                kind.edge,
                kind.erosion * (0.5 + 0.5 * (detail / 0.35).min(1.0)),
                kind.vary,
            ],
            [
                kind.base,
                kind.under,
                high(eye.x, self.wind.x),
                high(eye.y, self.wind.z),
            ],
        ]
    }
}

impl Clouds {
    /// Towards the light that keys the clouds in `scene`, which their own
    /// shadow is baked along (`clouds/shadow.rs`): under a procedural
    /// physical sky the sky's sun or its moon, whichever is the brighter
    /// at the cloud base, as sky.wgsl `cloud_key` chooses (a shadow baked
    /// along the set sun's ray while the rising moon keyed the clouds lay
    /// across them, and they stood darker than the night sky); under any
    /// other sky the scene's directional light.
    pub(crate) fn key_direction(&self, scene: &crate::RenderScene) -> Vec3 {
        let environment = &scene.lighting.environment;
        let crate::Background::Procedural {
            sun_direction,
            sun_color,
        } = environment.background
        else {
            return scene.lighting.light.direction_to_light;
        };
        let Some((look, air)) = environment
            .sky
            .look
            .and_then(|look| Some((look, look.physical?)))
        else {
            return scene.lighting.light.direction_to_light;
        };
        let sun = sun_direction.normalize_or(Vec3::Y);
        let moon_light = look.moon_light();
        if moon_light == Vec3::ZERO {
            return sun;
        }
        let moon = look.moon_toward(sun);
        let base = self.base + air.altitude.max(0.0);
        let luma = Vec3::new(0.2126, 0.7152, 0.0722);
        let s = (sun_color * air.sun_transmittance(base, sun.y)).dot(luma);
        let m = (moon_light * air.sun_transmittance(base, moon.y)).dot(luma);
        if m > s { moon } else { sun }
    }
}

/// The sun's direct beam through a closed deck of full thickness (an
/// optical depth of 7): [`Clouds::light`]'s least, never 0, so the clouds'
/// own lighting can undo the dimming (sky_look.wgsl, sky.wgsl).
pub const DECK_BEAM: f32 = 9.1e-4;

impl Clouds {
    /// How much of the sun's light (and, its second value, of the sky's)
    /// reaches the ground under this cover: a scattered sky keeps the sun
    /// (the clouds passing over it hide a share, `0.8 k`, of a sun whose
    /// clouds cast no shadow: [`Clouds::shadows`]), and a deck that
    /// closes the sky, as the sky draws it closing (sky_look.wgsl `cloud_deck`,
    /// over 0.6 to 0.95 cover), hides it: its direct beam through a deck
    /// of full thickness is the e⁻⁷ of an optical depth of some 7, and the
    /// light under it is the dimmer, flatter sky light. A thin deck (low
    /// density or thickness) hides it only in part. Every lit surface takes
    /// it, so under an overcast nothing is sunlit (a fifth of the sun left
    /// on lit foam and crests drew them 1.7 times the brightest sky).
    pub fn light(&self) -> (f32, f32) {
        let thick =
            (self.density / 0.04).clamp(0.0, 1.5) * (self.thickness / 1800.0).clamp(0.0, 1.5);
        let cover = self.coverage.clamp(0.0, 1.0);
        let k = (cover.powi(3) * thick.min(1.0)).clamp(0.0, 1.0);
        // (What the clouds passing over the sun hide is their shadow on
        // the world where they cast one, not a dimmer sun everywhere.)
        let passing = 0.8 * k * (1.0 - self.shadows.clamp(0.0, 1.0));
        let hidden = passing.max(self.closed()).min(1.0 - DECK_BEAM);
        (1.0 - hidden, 1.0 - 0.45 * k)
    }

    /// How far the cover has closed into a deck (0 broken, 1 closed), as
    /// the sky draws it closing over 0.6 to 0.95 cover; a thin deck never
    /// closes whole.
    pub(crate) fn closed(&self) -> f32 {
        let thick =
            (self.density / 0.04).clamp(0.0, 1.5) * (self.thickness / 1800.0).clamp(0.0, 1.5);
        let t = ((self.coverage.clamp(0.0, 1.0) - 0.6) / 0.35).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t) * thick.min(1.0)
    }

    /// Dims `scene`'s light under this cover, as every scene with clouds
    /// is lit: the sun by [`Clouds::light`]'s first value (its shadow
    /// softened and filled in by what the cover scattered,
    /// [`crate::DirectionalLight::cover`]); the image-based
    /// light by its second only where that light is not the live sky's
    /// capture ([`RenderScene::sky_captured`]), which holds the clouds
    /// already (dimmed twice, an overcast noon drew as night).
    /// Tint existing fog under the same cover. Call once after constructing
    /// the scene's light and fog, before adding weather's own atmosphere.
    pub fn light_scene(&self, scene: &mut crate::RenderScene) {
        let (sun, sky) = self.light();
        scene.lighting.light.cover(sun);
        if !scene.sky_captured() {
            scene.lighting.environment.intensity *= sky;
        }
        if let Some(fog) = &mut scene.atmosphere.fog {
            (fog.color, fog.sun_color) = self.fog_under(fog.color, fog.sun_color);
        }
        if let Some(fog) = &mut scene.atmosphere.volumetric_fog {
            fog.color = self.fog_under(fog.color, fog.color).0;
            fog.ambient = self.fog_under(fog.ambient, fog.ambient).0;
        }
    }

    /// A fog's colour and its colour looking into the sun under this cover:
    /// the air under an overcast is lit by the deck, grey and as dim as the
    /// sky light [`Clouds::light`] leaves, with no glow towards the hidden
    /// sun (a clear sky's bright horizon haze under a storm read as a clear
    /// band at the horizon).
    pub fn fog_under(&self, color: Vec3, sun_color: Vec3) -> (Vec3, Vec3) {
        let (sun, sky) = self.light();
        let k = (1.0 - sky) / 0.45;
        let grey = |c: Vec3| {
            c.lerp(
                Vec3::splat(c.dot(Vec3::new(0.2126, 0.7152, 0.0722))),
                (1.1 * k).min(1.0),
            )
        };
        // The air is lit by the deck's base, a third of the clear sky's
        // light at full cover (as the sky draws it, sky.wgsl `cloud_deck`);
        // a broken sky's air is the clear sky's.
        let t = ((k - 0.3) / 0.6).clamp(0.0, 1.0);
        let deck = 1.0 - 0.65 * t * t * (3.0 - 2.0 * t);
        (grey(color) * sky * deck, grey(sun_color) * sky * sun * deck)
    }
}

/// The clouds' baked fields and their sampler, bound by the sky pass (its
/// group's bindings 1, 2, 7 and 8) and the shadow's bake: the shape and
/// detail volumes and the map (clouds_noise.wgsl). Placeholders of one
/// texel until a frame has clouds.
pub(crate) struct CloudNoise {
    /// The shape volume.
    pub view: wgpu::TextureView,
    /// The detail volume.
    pub detail: wgpu::TextureView,
    /// The map.
    pub map: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    /// Whether the full fields are made: drawn, or their draw pending.
    pub made: bool,
    /// The fields' draw [`CloudNoise::make`] planned and
    /// [`CloudNoise::submit`] submits.
    pending: Option<NoiseDraw>,
    _textures: [wgpu::Texture; 3],
}

/// The fields' pipelines (clouds_noise.wgsl's entry points), compiling on a
/// worker from the frame that plans the draw.
struct NoisePipelines {
    shape: wgpu::ComputePipeline,
    detail: wgpu::ComputePipeline,
    map: wgpu::ComputePipeline,
    mips: wgpu::ComputePipeline,
}

#[derive(Clone, Copy)]
enum NoiseStep {
    Shape,
    Detail,
    Map,
    Mips,
}

struct NoiseDraw {
    pipelines: crate::deferred::Deferred<NoisePipelines>,
    /// Each step's group and workgroups, in order: a level after the one
    /// it averages.
    steps: Vec<(NoiseStep, wgpu::BindGroup, [u32; 3])>,
}

/// The main camera's clouds marched at half the frame's size (render/pass_clouds.rs, sky.wgsl `fs_clouds_low`): a
/// march is the sky's whole cost and a cloud has no edge a pixel wide. A frame that has the frame before's clouds
/// marches one texel of each two by two and reprojects the rest ([`CloudsTemporal`]).
pub(crate) struct CloudsLow {
    /// The cloud target's group in the sky's draw (group 2).
    pub layout: wgpu::BindGroupLayout,
    /// The march into the cloud target, or into the quarter target in a frame that reprojects.
    pub march: crate::deferred::Deferred<wgpu::RenderPipeline>,
    /// The march in a frame with a prepass: it skips the texels only covered pixels read. A second pipeline, since
    /// the depth's read costs a frame without one about a tenth of the march (0.3 ms of 2.8 at 720p on an Apple M4
    /// Pro).
    pub march_depth: crate::deferred::Deferred<wgpu::RenderPipeline>,
    /// The history reprojected around this frame's quarter march, into the cloud target.
    pub resolve: crate::deferred::Deferred<wgpu::RenderPipeline>,
    /// The procedural sky drawn at the frame's size under the cloud target's clouds, in place of
    /// `SkyState::sky_look_pipeline`.
    pub sky: crate::deferred::Sampled<wgpu::RenderPipeline>,
    pub temporal: CloudsTemporal,
}

/// The cloud target's size for a frame of `width` x `height`: half each way.
pub(crate) fn low_size(width: u32, height: u32) -> (u32, u32) {
    ((width / 2).max(1), (height / 2).max(1))
}

/// The quarter march's target for a cloud target of `size`: one texel of each two by two.
pub(crate) fn quarter_size((w, h): (u32, u32)) -> (u32, u32) {
    (w.div_ceil(2), h.div_ceil(2))
}

/// sky.wgsl `CloudsTemporal`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TemporalUniform {
    prev_view_proj: [[f32; 4]; 4],
    size: [f32; 4],
    mode: [f32; 4],
}

/// The texel of each two by two that the frame of `phase` marches: the four in turn, each across from the last.
fn block_offset(phase: u64) -> [f32; 2] {
    [[0.0, 0.0], [1.0, 1.0], [1.0, 0.0], [0.0, 1.0]][(phase % 4) as usize]
}

/// How far the frame of `phase` moves the march's dither, in texels along
/// the diagonal: interleaved gradient noise's value moves through its
/// range 3.865 times in a texel of it, so this is a share of the range.
/// The four texels of a two by two start their marches a quarter of the
/// range apart, across from each other first (a 2 x 2 ordered dither over
/// the noise), and each time a texel is marched again its start moves on
/// by the golden ratio: the marches the resolve blends into a texel are
/// spread evenly over a step, whichever of the four it is. (Moved by the
/// frame's count alone, a row's two texels kept starts close together and
/// the next row's another pair: fine level lines on a cloud's shaded side
/// and across a base.) None without a history to average them in.
fn dither_turn(phase: u64) -> f32 {
    const RANGE_PER_TEXEL: f64 = 52.982_918_9 * (0.067_110_56 + 0.005_837_15);
    let ordered = [0.0, 0.5, 0.25, 0.75][(phase % 4) as usize];
    let again = ((phase / 4) % 64) as f64 * 0.618_033_988_75;
    ((ordered + again).fract() / RANGE_PER_TEXEL) as f32
}

/// The cloud target's history (Schneider 2015, the temporal upsampling of the Horizon Zero Dawn clouds): the last
/// frame's resolved clouds, the view-projection that frame marched with, and the uniform that tells this frame's
/// march and resolve which texel of each two by two it marches. A frame whose history is the frame before's (by the
/// phase, so a capture, a restore and a cut start afresh) marches one texel in four; any other marches every texel.
/// It is part of the frame history a save keeps (`renderer/history.rs`).
pub(crate) struct CloudsTemporal {
    /// Group 2 of the march without a prepass, of the march with one (its view depth at 1), and of the resolve.
    pub march: wgpu::BindGroupLayout,
    pub depth: wgpu::BindGroupLayout,
    pub resolve: wgpu::BindGroupLayout,
    pub uniform: wgpu::Buffer,
    history: Option<(wgpu::Texture, wgpu::TextureView)>,
    /// The phase of the frame that wrote the history and its jittered view-projection.
    drawn: Option<(u64, glam::Mat4)>,
    /// This frame marches one texel in four and resolves ([`CloudsTemporal::prepare`]).
    pub reproject: bool,
}

/// [`CloudsTemporal::save`]: the history's copy and the frame that wrote it.
pub(crate) struct CloudsSave {
    copy: crate::gpu::copy::GpuCopy,
    drawn: Option<(u64, glam::Mat4)>,
}

impl CloudsSave {
    pub(crate) fn bytes(&self) -> u64 {
        self.copy.bytes()
    }
}

impl CloudsTemporal {
    pub(crate) fn new(
        device: &wgpu::Device,
        march: wgpu::BindGroupLayout,
        depth: wgpu::BindGroupLayout,
        resolve: wgpu::BindGroupLayout,
    ) -> CloudsTemporal {
        CloudsTemporal {
            march,
            depth,
            resolve,
            uniform: crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some("clouds temporal"),
                    size: std::mem::size_of::<TemporalUniform>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            ),
            history: None,
            drawn: None,
            reproject: false,
        }
    }

    /// Writes the step of the frame of `phase` at `frame_size`, whose march has `view_proj` (its jitter too); `cut`:
    /// the camera cut, so the history shows another view. Makes the history at the cloud target's size.
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame_size: (u32, u32),
        phase: u64,
        view_proj: glam::Mat4,
        cut: bool,
    ) {
        let (w, h) = low_size(frame_size.0, frame_size.1);
        if self
            .history
            .as_ref()
            .is_none_or(|(t, _)| (t.width(), t.height()) != (w, h))
        {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("clouds history"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: crate::HDR_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
                texture_binding_view_dimension: None,
            });
            let view = texture.create_view(&Default::default());
            self.history = Some((texture, view));
            self.drawn = None;
        }
        let previous = self.drawn.filter(|(f, _)| f + 1 == phase && !cut);
        self.reproject = previous.is_some();
        let [x, y] = block_offset(phase);
        let u = TemporalUniform {
            prev_view_proj: previous.map_or(view_proj, |(_, m)| m).to_cols_array_2d(),
            size: [w as f32, h as f32, x, y],
            mode: [
                f32::from(u8::from(self.reproject)),
                if self.reproject {
                    dither_turn(phase)
                } else {
                    0.0
                },
                0.0,
                0.0,
            ],
        };
        crate::traffic::write_buffer(queue, &self.uniform, 0, bytemuck::bytes_of(&u));
        self.drawn = Some((phase, view_proj));
    }

    /// The history: what the resolve reads, and what this frame's cloud target is copied into after it.
    pub(crate) fn history(&self) -> &(wgpu::Texture, wgpu::TextureView) {
        self.history.as_ref().expect("CloudsTemporal::prepare")
    }

    /// Forgets the history (a resize, a new scene, a readback).
    pub(crate) fn reset(&mut self) {
        self.drawn = None;
    }

    /// The history copied for a presentation checkpoint; `None` before a frame had clouds.
    pub(crate) fn save(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<CloudsSave> {
        let (texture, _) = self.history.as_ref()?;
        Some(CloudsSave {
            copy: crate::gpu::copy::GpuCopy::of(device, encoder, &[texture], &[]),
            drawn: self.drawn,
        })
    }

    /// Brings back [`CloudsTemporal::save`]'s history; true when it came back or neither had one (it starts afresh
    /// otherwise).
    pub(crate) fn restore(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        saved: Option<&CloudsSave>,
    ) -> bool {
        self.drawn = None;
        match (saved, &self.history) {
            (None, _) => true,
            (Some(s), Some((texture, _))) => {
                let back = s.copy.back(encoder, &[texture], &[]);
                if back {
                    self.drawn = s.drawn;
                }
                back
            }
            (Some(_), None) => false,
        }
    }
}

const FIELD_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// The map's: half floats, for a cloud's top follows it (eight bits
/// would give that top 256 heights).
const MAP_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// A field's texture: a volume of `size` a side with `levels` levels, or
/// (`flat`) a map of `size` a side.
fn field(device: &wgpu::Device, label: &str, size: u32, levels: u32, flat: bool) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: if flat { 1 } else { size },
        },
        mip_level_count: levels,
        sample_count: 1,
        dimension: if flat {
            wgpu::TextureDimension::D2
        } else {
            wgpu::TextureDimension::D3
        },
        format: if flat { MAP_FORMAT } else { FIELD_FORMAT },
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
        texture_binding_view_dimension: None,
    })
}

/// One level of `texture`.
fn level(texture: &wgpu::Texture, level: u32) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor {
        base_mip_level: level,
        mip_level_count: Some(1),
        ..Default::default()
    })
}

impl CloudNoise {
    pub(crate) fn placeholder(device: &wgpu::Device) -> CloudNoise {
        // (Zero where never written: nothing reads them without clouds.)
        let textures = [
            field(device, "cloud shape", 1, 1, false),
            field(device, "cloud detail", 1, 1, false),
            field(device, "cloud map", 1, 1, true),
        ];
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cloud noise"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        CloudNoise {
            view: textures[0].create_view(&Default::default()),
            detail: textures[1].create_view(&Default::default()),
            map: textures[2].create_view(&Default::default()),
            sampler,
            made: false,
            pending: None,
            _textures: textures,
        }
    }

    /// Makes the full fields (once), to draw by compute passes of
    /// `clouds_noise.wgsl` before the frame's passes read them
    /// ([`CloudNoise::submit`]). Their pipelines start compiling on a
    /// worker now, so the frame's other work and compiles overlap them:
    /// made here, they were the first frame's longest wait before any of
    /// the frame's pipelines started (a second of llvmpipe's).
    pub(crate) fn make(&mut self, device: &wgpu::Device) {
        let textures = [
            field(device, "cloud shape", SHAPE_SIZE, SHAPE_LEVELS, false),
            field(device, "cloud detail", DETAIL_SIZE, DETAIL_LEVELS, false),
            field(device, "cloud map", MAP_SIZE, 1, true),
        ];
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty,
            count: None,
        };
        let out = |view_dimension| wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: match view_dimension {
                wgpu::TextureViewDimension::D2 => MAP_FORMAT,
                _ => FIELD_FORMAT,
            },
            view_dimension,
        };
        let layout = |label, entries: &[wgpu::BindGroupLayoutEntry]| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries,
            })
        };
        let volume = layout(
            "cloud noise volume",
            &[entry(0, out(wgpu::TextureViewDimension::D3))],
        );
        let map = layout(
            "cloud noise map",
            &[entry(1, out(wgpu::TextureViewDimension::D2))],
        );
        let mips = layout(
            "cloud noise mips",
            &[
                entry(0, out(wgpu::TextureViewDimension::D3)),
                entry(
                    2,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                ),
            ],
        );
        let group = |layout: &wgpu::BindGroupLayout, entries: &[(u32, &wgpu::TextureView)]| {
            let entries: Vec<_> = entries
                .iter()
                .map(|(binding, view)| wgpu::BindGroupEntry {
                    binding: *binding,
                    resource: wgpu::BindingResource::TextureView(view),
                })
                .collect();
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("cloud noise"),
                layout,
                entries: &entries,
            })
        };
        let mut steps = Vec::new();
        for (texture, step, size, levels) in [
            (&textures[0], NoiseStep::Shape, SHAPE_SIZE, SHAPE_LEVELS),
            (&textures[1], NoiseStep::Detail, DETAIL_SIZE, DETAIL_LEVELS),
        ] {
            let views: Vec<_> = (0..levels).map(|l| level(texture, l)).collect();
            steps.push((step, group(&volume, &[(0, &views[0])]), [size / 4; 3]));
            for l in 1..levels as usize {
                let n = (size >> l) / 4;
                steps.push((
                    NoiseStep::Mips,
                    group(&mips, &[(0, &views[l]), (2, &views[l - 1])]),
                    [n; 3],
                ));
            }
        }
        let map_view = textures[2].create_view(&Default::default());
        steps.push((
            NoiseStep::Map,
            group(&map, &[(1, &map_view)]),
            [MAP_SIZE / 8, MAP_SIZE / 8, 1],
        ));
        let pipelines = {
            let device = device.clone();
            crate::deferred::Deferred::anywhere(move || {
                crate::deferred::note(|| "cloud noise".into());
                let module = crate::gpu::shader_module(
                    &device,
                    wgpu::ShaderModuleDescriptor {
                        label: Some("cloud noise"),
                        source: wgpu::ShaderSource::Wgsl(NOISE_WGSL.into()),
                    },
                );
                let pipeline = |layout: &wgpu::BindGroupLayout, entry_point| {
                    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("cloud noise"),
                        bind_group_layouts: &[Some(layout)],
                        immediate_size: 0,
                    });
                    crate::traffic::create_compute_pipeline(
                        &device,
                        &wgpu::ComputePipelineDescriptor {
                            label: Some("cloud noise"),
                            layout: Some(&layout),
                            module: &module,
                            entry_point: Some(entry_point),
                            compilation_options: Default::default(),
                            cache: crate::pipeline_cache::of(&device).as_ref(),
                        },
                    )
                };
                NoisePipelines {
                    shape: pipeline(&volume, "shape"),
                    detail: pipeline(&volume, "detail"),
                    map: pipeline(&map, "map"),
                    mips: pipeline(&mips, "mips"),
                }
            })
        };
        crate::deferred::Start::start(&pipelines);
        self.pending = Some(NoiseDraw { pipelines, steps });
        self.view = textures[0].create_view(&Default::default());
        self.detail = textures[1].create_view(&Default::default());
        self.map = map_view;
        self._textures = textures;
        self.made = true;
    }

    /// Whether the full fields are drawn: what a bake that keeps what it
    /// reads waits for.
    pub(crate) fn drawn(&self) -> bool {
        self.made && self.pending.is_none()
    }

    /// Whether the full fields are drawn or [`CloudNoise::submit`] draws
    /// them this frame.
    pub(crate) fn drawing(&self) -> bool {
        self.made
            && self
                .pending
                .as_ref()
                .is_none_or(|draw| draw.pipelines.ready().is_some())
    }

    /// Submits the fields' draw [`CloudNoise::make`] planned, waiting for
    /// its pipelines if they are still compiling natively: before the
    /// frame's own commands, the first that read them. While the pipelines
    /// are not ready the draw waits for a later frame (gpu/deferred.rs).
    pub(crate) fn submit(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self
            .pending
            .as_ref()
            .is_some_and(|draw| draw.pipelines.ready().is_none())
        {
            return;
        }
        let Some(draw) = self.pending.take() else {
            return;
        };
        let pipelines = draw.pipelines.wait();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("cloud noise"),
        });
        // A pass a step: a level is read by the step after the one that
        // wrote it.
        for (step, group, [x, y, z]) in &draw.steps {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("cloud noise"),
                timestamp_writes: None,
            });
            pass.set_pipeline(match step {
                NoiseStep::Shape => &pipelines.shape,
                NoiseStep::Detail => &pipelines.detail,
                NoiseStep::Map => &pipelines.map,
                NoiseStep::Mips => &pipelines.mips,
            });
            pass.set_bind_group(0, group, &[]);
            pass.dispatch_workgroups(*x, *y, *z);
        }
        queue.submit(Some(encoder.finish()));
    }
}

/// The fields' compute shader, after the hashes it draws from.
pub(crate) const NOISE_WGSL: &str = wgsl_library!("clouds_noise.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    /// Under an overcast the fog is the deck's dim grey with no sun glow;
    /// under a few clouds it keeps a clear sky's blue.
    #[test]
    fn fog_under_an_overcast_is_dim_grey() {
        let blue = Vec3::new(0.5, 0.65, 0.9);
        let sun = Vec3::new(1.0, 0.8, 0.5);
        let fair = Clouds {
            coverage: 0.3,
            ..Clouds::default()
        };
        let (c, s) = fair.fog_under(blue, sun);
        assert!((c - blue).length() < 0.05 && (s - sun).length() < 0.06);
        let storm = Clouds {
            coverage: 0.97,
            density: 0.09,
            thickness: 1800.0,
            ..Clouds::default()
        };
        let mut scene = crate::RenderScene::new();
        scene.atmosphere.fog = Some(crate::Fog {
            color: blue,
            sun_color: sun,
            ..Default::default()
        });
        scene.atmosphere.volumetric_fog = Some(crate::VolumetricFog {
            color: blue,
            ambient: blue,
            ..Default::default()
        });
        storm.light_scene(&mut scene);
        let fog = scene.atmosphere.fog.unwrap();
        let (c, s) = (fog.color, fog.sun_color);
        let lit = scene.atmosphere.volumetric_fog.unwrap();
        assert_eq!((lit.color, lit.ambient), (c, c));
        assert!(
            c.max_element() - c.min_element() < 0.1 * c.max_element(),
            "grey: {c}"
        );
        assert!(
            c.max_element() < 0.6 * blue.max_element() && s.max_element() < 0.2,
            "dim: {c} {s}"
        );
    }

    /// The block: the wind's offset follows the clock and wraps at four
    /// tiles (same offset a period later), the steps are clamped, and the
    /// high layer never sits inside the volumetric one.
    #[test]
    fn the_cloud_block_moves_with_the_clock_and_wraps() {
        let c = Clouds::default();
        let a = c.gpu(10.0, 6.36e6, glam::DVec2::ZERO);
        assert_eq!(a[3][3], 1.0);
        assert!((a[1][0] - 120.0).abs() < 1e-3 && (a[1][2] - 40.0).abs() < 1e-3);
        let period = f64::from(c.scale) * 4.0 / f64::from(c.wind.x);
        let b = c.gpu(10.0 + period * 3.0, 6.36e6, glam::DVec2::ZERO);
        assert!(
            (a[1][0] - b[1][0]).abs() < 0.05,
            "{} vs {}",
            a[1][0],
            b[1][0]
        );
        // A day of clock keeps sub-metre steps.
        let late = c.gpu(86_400.0, 6.36e6, glam::DVec2::ZERO)[1][0];
        let later = c.gpu(86_400.0 + 1.0 / 60.0, 6.36e6, glam::DVec2::ZERO)[1][0];
        assert!((later - late - 0.2).abs() < 0.01, "{late} {later}");
        let low = Clouds {
            layer_altitude: 100.0,
            steps: 1000,
            ..c
        };
        let g = low.gpu(0.0, 6.36e6, glam::DVec2::ZERO);
        assert_eq!(g[3][2], 256.0);
        assert_eq!(g[3][1], c.base + c.thickness);
        // The eye's place wraps as the offset does, far from the origin too.
        let far = glam::DVec2::new(1.0e7 + 250.0, -50.0);
        let at = c.gpu(0.0, 6.36e6, far)[10];
        let tile = f64::from(c.scale) * 4.0;
        assert!(
            (f64::from(at[0]) - far.x.rem_euclid(tile)).abs() < 0.5,
            "{at:?}"
        );
        assert!((f64::from(at[1]) - (tile - 50.0)).abs() < 0.01, "{at:?}");
        // The high layer's place goes against twice the wind and with the
        // eye, and wraps at sixteen tiles.
        let high = c.gpu(10.0, 6.36e6, glam::DVec2::new(500.0, 0.0))[14];
        assert!((high[2] - 260.0).abs() < 1e-3, "{high:?}");
        assert!(
            (f64::from(high[3]) - (tile * 4.0 - 80.0)).abs() < 0.01,
            "{high:?}"
        );
        let round = c.gpu(10.0, 6.36e6, glam::DVec2::new(500.0 + tile * 4.0, 0.0))[14];
        assert!((high[2] - round[2]).abs() < 0.01, "{high:?} {round:?}");
    }

    /// A broken sky barely dims the sun; overcast takes most of it, and
    /// thin overcast less than thick.
    #[test]
    fn overcast_dims_the_light() {
        let at = |coverage, density| {
            Clouds {
                coverage,
                density,
                thickness: 1800.0,
                ..Clouds::default()
            }
            .light()
        };
        assert!(at(0.0, 0.04) == (1.0, 1.0));
        assert!(at(0.3, 0.04).0 > 0.95);
        // A closed deck hides the sun; one thin enough lets most through.
        let (sun, sky) = at(1.0, 0.04);
        assert!((sun - DECK_BEAM).abs() < 1e-6 && sky > 0.5, "{sun} {sky}");
        assert!(at(1.0, 0.01).0 > 0.7);
        // Broken cover under the deck's closing: its clouds' shadows fall
        // on the world, and the sun between them is whole; without them
        // the sun is dimmed by the share they hide.
        assert_eq!(at(0.5, 0.04).0, 1.0);
        let unshadowed = Clouds {
            coverage: 0.5,
            shadows: 0.0,
            thickness: 1800.0,
            ..Clouds::default()
        };
        assert!((unshadowed.light().0 - (1.0 - 0.8 * 0.125)).abs() < 1e-5);
        // Closing: monotonic, from the broken sky's dimming to the deck's.
        let mut last = 1.0;
        for i in 0..=40 {
            let s = at(i as f32 / 40.0, 0.04).0;
            assert!(s <= last + 1e-6, "{i}: {s} after {last}");
            last = s;
        }
        assert!(at(0.9, 0.04).0 < 0.1);
    }
}
