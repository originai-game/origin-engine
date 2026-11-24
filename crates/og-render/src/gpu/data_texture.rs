//! Data textures: the arrays a vertex stage reads, as 2D textures of
//! 16-byte texels addressed by index (`common.wgsl` `data_at`), DATA_WIDTH
//! texels a row. WebGPU's compatibility mode allows no storage in the
//! vertex stage; a texture read with `textureLoad` works on every tier.
//! A writer fills one from the CPU ([`DataTexture::write`]) or with
//! `textureStore` in compute (made with `STORAGE_BINDING`).

/// Texels a row (`common.wgsl` `DATA_WIDTH`).
pub(crate) const DATA_WIDTH: u32 = 1024;
const ROW_BYTES: u64 = DATA_WIDTH as u64 * 16;

pub(crate) struct DataTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    /// The texels it holds: whole rows.
    pub texels: u32,
}

impl DataTexture {
    /// At least `texels` (one row at least) of `format`, a four-component
    /// 32-bit format (`Rgba32Uint`, `Rgba32Float`), readable as a texture
    /// and with `usage`.
    pub fn new(
        device: &wgpu::Device,
        label: &str,
        format: wgpu::TextureFormat,
        texels: u32,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let max_rows = device.limits().max_texture_dimension_2d.max(1);
        let wanted_rows = texels.div_ceil(DATA_WIDTH).max(1);
        if wanted_rows > max_rows {
            log::error!(
                "DataTexture `{label}` requested {wanted_rows} rows ({texels} texels), clamping to device max_texture_dimension_2d={max_rows}"
            );
        }
        let rows = wanted_rows.min(max_rows);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: DATA_WIDTH,
                height: rows,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: usage | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
            texture_binding_view_dimension: None,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        DataTexture {
            texture,
            view,
            texels: rows * DATA_WIDTH,
        }
    }

    /// Writes `bytes` (whole texels) from the start, made again first
    /// (twice the texels, of its format and usage) when they do not fit;
    /// returns whether it was made again, so its bind groups must be too.
    pub fn grow_and_write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
        bytes: &[u8],
    ) -> bool {
        let max_texels = device
            .limits()
            .max_texture_dimension_2d
            .max(1)
            .saturating_mul(DATA_WIDTH);
        let texels = ((bytes.len() / 16) as u32).min(max_texels);
        let grew = texels > self.texels;
        if grew {
            let target = texels
                .checked_next_power_of_two()
                .unwrap_or(max_texels)
                .min(max_texels);
            *self = DataTexture::new(
                device,
                label,
                self.texture.format(),
                target,
                self.texture.usage(),
            );
        }
        if !bytes.is_empty() {
            self.write(queue, 0, bytes);
        }
        grew
    }

    /// Writes `data` (whole 16-byte texels) from texel `first`: the rest
    /// of its row, whole rows, then the start of the last; made with
    /// `COPY_DST`.
    pub fn write(&self, queue: &wgpu::Queue, first: u32, data: &[u8]) {
        debug_assert!(data.len().is_multiple_of(16));
        if first >= self.texels {
            return;
        }
        let max_write = (self.texels - first) as usize * 16;
        let data = &data[..data.len().min(max_write)];
        let put = |at: u32, width: u32, height: u32, bytes: &[u8]| {
            if width == 0 || height == 0 {
                return;
            }
            crate::traffic::write_texture(
                queue,
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: at % DATA_WIDTH,
                        y: at / DATA_WIDTH,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 16),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        };
        let texels = (data.len() / 16) as u32;
        let head = texels.min((DATA_WIDTH - first % DATA_WIDTH) % DATA_WIDTH);
        put(first, head, 1, &data[..head as usize * 16]);
        let rest = &data[head as usize * 16..];
        let (at, whole) = (first + head, (texels - head) / DATA_WIDTH);
        let split = whole as usize * ROW_BYTES as usize;
        put(at, DATA_WIDTH, whole, &rest[..split]);
        put(
            at + whole * DATA_WIDTH,
            (texels - head) % DATA_WIDTH,
            1,
            &rest[split..],
        );
    }
}

/// A data texture's layout entry at `binding`, read in `visibility`:
/// `uint` for `texture_2d<u32>`, else `texture_2d<f32>` (unfilterable).
pub(crate) fn data_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    uint: bool,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Texture {
            sample_type: if uint {
                wgpu::TextureSampleType::Uint
            } else {
                wgpu::TextureSampleType::Float { filterable: false }
            },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}
