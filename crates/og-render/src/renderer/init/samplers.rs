//! The renderer's samplers (materials', sprites' anisotropic and nearest,
//! trilinear) and its shadow maps (the cascades' array and the local
//! lights').

use super::*;

pub(super) struct SamplersAndShadowMapsInputs<'a> {
    pub device: &'a wgpu::Device,
    pub info: &'a RenderInfo,
    pub shadow_size: u32,
    pub cascades: u32,
}

pub(super) struct SamplersAndShadowMapsOutputs {
    pub material_sampler: wgpu::Sampler,
    pub sprite_sampler: wgpu::Sampler,
    pub trilinear_sampler: wgpu::Sampler,
    pub sprite_sampler_nearest: wgpu::Sampler,
    pub cascade_map: ShadowMap,
    pub local_map: ShadowMap,
}

impl Renderer {
    /// Makes the samplers and the shadow maps.
    #[allow(clippy::too_many_lines)]
    pub(super) fn samplers_and_shadow_maps(
        SamplersAndShadowMapsInputs {
            device,
            info,
            shadow_size,
            cascades,
        }: SamplersAndShadowMapsInputs<'_>,
    ) -> SamplersAndShadowMapsOutputs {
        // Samplers and fixed-size targets.
        let material_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("material"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            // Filter grazing-angle footprints along both axes, where the
            // device filters anisotropically: the sprites' rule
            // (RenderInfo::sprite_anisotropy), so a software rasterizer,
            // whose anisotropic filter sampled every texture black, stays at 1.
            anisotropy_clamp: info.sprite_anisotropy,
            ..Default::default()
        });
        // Sprites and the UI's images filter anisotropically where the
        // device can (RenderInfo::sprite_anisotropy): a nine-slice's middle
        // squeezed along one axis reads its mip level by the other one.
        let sprite_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: info.sprite_anisotropy,
            ..Default::default()
        });
        // Trilinear and clamped: the font atlases (an MSDF distance field
        // averaged along a squeezed axis loses its edge: the ink spreads and
        // thins), the decal atlas and the 2D lightmap (sampled as the
        // surfaces under them are, by the material sampler's filter).
        let trilinear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("trilinear"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let sprite_sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite (nearest)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        // The default quality's cascades; a frame that asks another count
        // makes the array again (`fit_shelter_layer`).
        let cascade_map = ShadowMap::new(
            device,
            "cascades",
            shadow_size,
            cascades.min(crate::RenderQuality::default().shadow_cascades),
            wgpu::TextureFormat::R32Float,
        );
        // Twelve layers to begin with; a frame that needs more grows it
        // (`grow_local_shadows`).
        let local_map = ShadowMap::new(
            device,
            "local shadows",
            LOCAL_SHADOW_SIZE,
            shadows::INITIAL_LOCAL_LAYERS
                .min(shadows::capacity(
                    info.shadowed_lights as usize,
                    info.point_shadows,
                ))
                .max(1) as u32,
            LOCAL_SHADOW_FORMAT,
        );
        SamplersAndShadowMapsOutputs {
            material_sampler,
            sprite_sampler,
            trilinear_sampler,
            sprite_sampler_nearest,
            cascade_map,
            local_map,
        }
    }
}
