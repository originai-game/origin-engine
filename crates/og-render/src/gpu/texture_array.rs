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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::TextureData;
    use crate::scene::{ColorSpace, Mips, TextureSource};

    fn layer(v: u8, w: u32, h: u32) -> TextureSource {
        TextureSource::Rgba(TextureData {
            width: w,
            height: h,
            rgba: vec![v; (w * h * 4) as usize],
            color_space: ColorSpace::Srgb,
            mips: Mips::Generate,
        })
    }

    #[test]
    fn layers_must_agree() {
        let ok = TextureArrayData {
            layers: (0..6).map(|i| layer(i * 40, 8, 4)).collect(),
        };
        let info = ok.info().unwrap();
        assert_eq!((info.layers, info.levels), (6, 4));
        let texels = ok.texels(&info);
        let per = (8 * 4 + 4 * 2 + 2 + 1) * 4;
        assert_eq!(texels.len(), per * 6);
        assert_eq!(texels[per * 5], 200, "the last layer last");
        assert_eq!(info.bytes, (per * 6) as u64);
        let one = TextureArrayData {
            layers: vec![layer(1, 4, 4)],
        };
        assert_eq!(one.info().unwrap().layers, 1);
        let mixed = TextureArrayData {
            layers: vec![layer(1, 4, 4), layer(1, 8, 4)],
        };
        assert!(mixed.info().unwrap_err().contains("layer 1 is 8x4"));
        assert!(TextureArrayData { layers: vec![] }.info().is_err());
    }
}
