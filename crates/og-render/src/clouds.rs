//! Clouds (the R4 round): a volumetric layer raymarched in the sky pass
//! over a curved shell around the planet, lit by the sun (under the
//! physical sky, the sunlight through the air down to each sample's own
//! height, so a sunset lights low clouds orange from below; else the
//! frame's directional light: Beer's law towards it, with three octaves
//! of the multiple scattering approximation of Wrenninge et al.), the
//! environment's irradiance and a lightning strike's flash inside them
//! ([`Clouds::flash`]), faded into the sky with distance; and a thin high layer
//! (cirrus) over it. Their shapes come from a tiling 64³ noise volume
//! (Perlin–Worley base, two Worley detail octaves, a coverage channel),
//! made once on the GPU by a compute pass the first frame that has clouds.
//! The wind moves them by the tick's seconds, so they look the same on
//! every run and after a restore; nothing in the simulation reads them.
//!
//! They are drawn over the procedural sky only
//! ([`crate::Background::Procedural`]): the sky pass draws them, so they do
//! not light the scene by themselves; [`Clouds::light`] is how much a
//! cover dims the sun and the sky's light under it, which the runtime
//! applies.

use glam::Vec3;

/// The noise volume's size per side.
pub(crate) const NOISE_SIZE: u32 = 64;
/// The vec4s of the frame uniform's cloud block (`frame.clouds`).
pub(crate) const CLOUD_VEC4S: usize = 6;

/// Clouds over the scene. See the module documentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clouds {
    /// How much of the sky the volumetric layer covers (0: none, 1: overcast).
    pub coverage: f32,
    /// Their extinction per metre at full density (0.03 to 0.08: cumulus;
    /// lower: thin, translucent clouds).
    pub density: f32,
    /// The layer's base and thickness, in metres above the world's y 0
    /// (thickness 0: no volumetric layer).
    pub base: f32,
    pub thickness: f32,
    /// The size of the largest cloud shapes, in metres (one tile of the noise).
    pub scale: f32,
    /// How far the wind carries them per second (x and z; y turns the
    /// shapes over as they go).
    pub wind: Vec3,
    /// Their albedo (white for water clouds; a grey for storm clouds).
    pub color: Vec3,
    /// How much the small noise erodes their edges (0: smooth blobs).
    pub detail: f32,
    /// The high thin layer's cover (0: none) and height in metres.
    pub layer: f32,
    pub layer_altitude: f32,
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
            coverage: 0.45,
            density: 0.04,
            base: 1500.0,
            thickness: 1800.0,
            scale: 9000.0,
            wind: Vec3::new(12.0, 0.4, 4.0),
            color: Vec3::ONE,
            detail: 0.35,
            layer: 0.3,
            layer_altitude: 8000.0,
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
    /// which the scene's light is assumed to carry). The offset wraps every
    /// four tiles (the coverage channel's period) so f32 keeps its
    /// precision however long the clock runs.
    pub(crate) fn gpu(&self, seconds: f64, radius: f32) -> [[f32; 4]; CLOUD_VEC4S] {
        let scale = self.scale.clamp(500.0, 200_000.0);
        let period = f64::from(scale) * 4.0;
        let offset = |v: f32| (f64::from(v) * seconds).rem_euclid(period) as f32;
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
            self.color
                .max(Vec3::ZERO)
                .extend(self.detail.clamp(0.0, 1.0))
                .to_array(),
            [
                self.layer.clamp(0.0, 1.0),
                self.layer_altitude.max(self.base + self.thickness),
                self.steps.clamp(8, 256) as f32,
                1.0,
            ],
            {
                // How much the scene's sun and sky light were dimmed by
                // this cover (the runtime applies Clouds::light to them);
                // the clouds' own lighting undoes it.
                let (sun, sky) = self.light();
                [radius, sun, sky, 0.0]
            },
            self.flash.to_array(),
        ]
    }
}

/// The sun's direct beam through a closed deck of full thickness (an
/// optical depth of 7): [`Clouds::light`]'s least, never 0, so the clouds'
/// own lighting can undo the dimming (frame.wgsl, sky.wgsl).
pub const DECK_BEAM: f32 = 9.1e-4;

impl Clouds {
    /// How much of the sun's light (and, its second value, of the sky's)
    /// reaches the ground under this cover: a scattered sky keeps the sun
    /// (the clouds passing over it hide a share, `0.8 k`), and a deck that
    /// closes the sky, as the sky draws it closing (frame.wgsl `cloud_deck`,
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
        let t = ((cover - 0.6) / 0.35).clamp(0.0, 1.0);
        let closed = t * t * (3.0 - 2.0 * t) * thick.min(1.0);
        let hidden = (0.8 * k).max(closed).min(1.0 - DECK_BEAM);
        (1.0 - hidden, 1.0 - 0.45 * k)
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

/// The noise volume and its sampler, bound by the sky pass (its group's
/// bindings 1 and 2): a 1³ placeholder until a frame has clouds.
pub(crate) struct CloudNoise {
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    /// Whether the full volume has been drawn.
    pub made: bool,
    _texture: wgpu::Texture,
}

fn volume(device: &wgpu::Device, size: u32, storage: bool) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("cloud noise"),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: size,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: if storage {
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING
        } else {
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
        },
        view_formats: &[],
    })
}

impl CloudNoise {
    pub(crate) fn placeholder(device: &wgpu::Device, queue: &wgpu::Queue) -> CloudNoise {
        let texture = volume(device, 1, false);
        queue.write_texture(
            texture.as_image_copy(),
            &[0, 0, 0, 0],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cloud noise"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        CloudNoise {
            view: texture.create_view(&Default::default()),
            sampler,
            made: false,
            _texture: texture,
        }
    }

    /// Draws the full volume (once): a compute pass of `clouds_noise.wgsl`.
    pub(crate) fn make(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let texture = volume(device, NOISE_SIZE, true);
        let view = texture.create_view(&Default::default());
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cloud noise"),
            source: wgpu::ShaderSource::Wgsl(NOISE_WGSL.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cloud noise"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D3,
                },
                count: None,
            }],
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("cloud noise"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("cloud noise"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                }),
            ),
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cloud noise"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("cloud noise"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("cloud noise"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &group, &[]);
            let n = NOISE_SIZE / 4;
            pass.dispatch_workgroups(n, n, n);
        }
        queue.submit(Some(encoder.finish()));
        self.view = view;
        self._texture = texture;
        self.made = true;
    }
}

/// The noise volume's compute shader.
pub(crate) const NOISE_WGSL: &str = wgsl!("clouds_noise.wgsl");

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
            ..Clouds::default()
        };
        let (c, s) = storm.fog_under(blue, sun);
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
        let a = c.gpu(10.0, 6.36e6);
        assert_eq!(a[3][3], 1.0);
        assert!((a[1][0] - 120.0).abs() < 1e-3 && (a[1][2] - 40.0).abs() < 1e-3);
        let period = f64::from(c.scale) * 4.0 / f64::from(c.wind.x);
        let b = c.gpu(10.0 + period * 3.0, 6.36e6);
        assert!(
            (a[1][0] - b[1][0]).abs() < 0.05,
            "{} vs {}",
            a[1][0],
            b[1][0]
        );
        // A day of clock keeps sub-metre steps.
        let late = c.gpu(86_400.0, 6.36e6)[1][0];
        let later = c.gpu(86_400.0 + 1.0 / 60.0, 6.36e6)[1][0];
        assert!((later - late - 0.2).abs() < 0.01, "{late} {later}");
        let low = Clouds {
            layer_altitude: 100.0,
            steps: 1000,
            ..c
        };
        let g = low.gpu(0.0, 6.36e6);
        assert_eq!(g[3][2], 256.0);
        assert_eq!(g[3][1], c.base + c.thickness);
    }

    /// A broken sky barely dims the sun; overcast takes most of it, and
    /// thin overcast less than thick.
    #[test]
    fn overcast_dims_the_light() {
        let at = |coverage, density| {
            Clouds {
                coverage,
                density,
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
        // Broken cover under the deck's closing keeps the old dimming.
        assert!((at(0.5, 0.04).0 - (1.0 - 0.8 * 0.125)).abs() < 1e-5);
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
