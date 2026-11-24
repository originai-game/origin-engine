pub use og_render_data::environment::*;

/// How a frame uses an environment map ([`crate::RenderScene::environment_map`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvironmentMap {
    pub map: crate::EnvironmentMapHandle,
    /// Turns the map about +y, in radians (counter-clockwise seen from
    /// above: what was at +x moves towards -z), as three.js
    /// `scene.environmentRotation.y`.
    pub rotation: f32,
    /// Multiplies the map where it is drawn as the background
    /// ([`crate::Background::Sky`]), as three.js `backgroundIntensity`; the
    /// light's strength is [`crate::Environment::intensity`].
    pub background_intensity: f32,
    /// A second map blended over this one, light and background alike
    /// (indoor to outdoor): `None` is this map alone.
    pub blend: Option<EnvironmentBlend>,
}

/// A blend from [`EnvironmentMap::map`] towards another map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvironmentBlend {
    /// The map blended towards (turned and scaled as the first).
    pub to: crate::EnvironmentMapHandle,
    /// 0 is the first map, 1 the second. The caller drives it from the
    /// tick, never from a clock, so a replay blends alike.
    pub weight: f32,
}

/// The steps a blend's weight is drawn at (each step regenerates the
/// image-based light once; finer steps are invisible).
pub const ENVIRONMENT_BLEND_STEPS: f32 = 128.0;

impl EnvironmentBlend {
    /// The weight drawn: within 0..1 (not finite: 0), in
    /// [`ENVIRONMENT_BLEND_STEPS`] steps.
    pub fn drawn_weight(&self) -> f32 {
        let w = if self.weight.is_finite() {
            self.weight.clamp(0.0, 1.0)
        } else {
            0.0
        };
        (w * ENVIRONMENT_BLEND_STEPS).round() / ENVIRONMENT_BLEND_STEPS
    }
}

/// `a`'s harmonics moved `w` of the way to `b`'s: the irradiance of the
/// blend of two radiances is the blend of their irradiances.
pub(crate) fn mix_irradiance(a: &[[f32; 3]; 9], b: &[[f32; 3]; 9], w: f32) -> [[f32; 3]; 9] {
    std::array::from_fn(|k| std::array::from_fn(|c| a[k][c] + (b[k][c] - a[k][c]) * w))
}

impl EnvironmentMap {
    /// `map` unturned, its background as cooked.
    pub fn new(map: crate::EnvironmentMapHandle) -> EnvironmentMap {
        EnvironmentMap {
            map,
            rotation: 0.0,
            background_intensity: 1.0,
            blend: None,
        }
    }
}

/// The IBL passes' uniform block for a map: (cos, sin, scale, the second
/// map's weight in a blend) then the 27 irradiance floats.
pub(crate) fn env_params(
    rotation: f32,
    scale: f32,
    blend: f32,
    irradiance: &[[f32; 3]; 9],
) -> [[f32; 4]; 8] {
    let mut out = [[0.0; 4]; 8];
    out[0] = [rotation.cos(), rotation.sin(), scale, blend];
    for (k, v) in irradiance.iter().flatten().enumerate() {
        out[1 + k / 4][k % 4] = *v;
    }
    out
}

/// Uploads lighting's GGX cube or the authored background's own box mips.
pub(crate) fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    map: &EnvironmentMapData,
    background: bool,
) -> (wgpu::Texture, wgpu::TextureView) {
    let (size, levels, texels) = if background {
        (
            map.background_size,
            map.background_size.ilog2() + 1,
            &map.background,
        )
    } else {
        (map.size, ENVIRONMENT_MAP_LEVELS, &map.texels)
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("environment map"),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 6,
        },
        mip_level_count: levels,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgb9e5Ufloat,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
        // Bound as a cube (WebGPU compatibility mode infers `2d-array` for six layers).
        texture_binding_view_dimension: Some(wgpu::TextureViewDimension::Cube),
    });
    let mut at = 0;
    for level in 0..levels {
        let n = size >> level;
        let count = 6 * (n * n) as usize;
        crate::traffic::write_texture(
            queue,
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&texels[at..at + count]),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * n),
                rows_per_image: Some(n),
            },
            wgpu::Extent3d {
                width: n,
                height: n,
                depth_or_array_layers: 6,
            },
        );
        at += count;
    }
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("environment map"),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    });
    (texture, view)
}
