pub use og_render_data::texture_array::*;

/// A texture array on the GPU (a `D2Array` view of it is what a shader
/// binds).
pub(crate) struct GpuTextureArray {
    /// Held for as long as the scene holds the array (read back natively).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub texture: wgpu::Texture,
    pub info: TextureArrayInfo,
    /// The generation of the scene slot it was uploaded for.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub generation: u8,
}

/// Uploads a checked texture array as one `D2Array` texture.
pub(crate) fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &TextureArrayData,
    info: TextureArrayInfo,
) -> Result<GpuTextureArray, String> {
    use crate::texture::TextureFormatGpu;
    use wgpu::util::DeviceExt;
    let format = info
        .format
        .wgpu(info.color_space)
        .ok_or_else(|| format!("{} is not a GPU format here", info.format.name()))?;
    let texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("texture array"),
            size: wgpu::Extent3d {
                width: info.width,
                height: info.height,
                depth_or_array_layers: info.layers,
            },
            mip_level_count: info.levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
            // Bound as an array however many layers it has (WebGPU compatibility mode infers `2d` for one).
            texture_binding_view_dimension: Some(wgpu::TextureViewDimension::D2Array),
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &data.texels(&info),
    );
    Ok(GpuTextureArray {
        texture,
        info,
        generation: 0,
    })
}
