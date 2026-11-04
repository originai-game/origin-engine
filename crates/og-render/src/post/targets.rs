//! Persistent targets and texture creation.

use super::*;

/// The size-dependent targets that outlive a frame (imported into the
/// graph): the offscreen output and the TAA history.
pub(crate) struct Persistent {
    pub offscreen: wgpu::Texture,
    pub offscreen_view: wgpu::TextureView,
    /// The offscreen target seen as sRGB when its format is not
    /// ([`crate::linear_view_format`]): where the overlay blends.
    pub offscreen_linear: Option<wgpu::TextureView>,
    /// TAA history ping-pong: frame `n` writes `history[n % 2]`.
    pub history: Option<[wgpu::TextureView; 2]>,
    /// T2x stores the two raw, spatially filtered display frames, never
    /// the temporal resolve. sRGB in SDR; float for resolved HDR output.
    pub smaa_history: Option<[wgpu::TextureView; 2]>,
    /// Screen-space GI's copy of the lit image without TAA (`ssgi.rs`),
    /// with its size: frame `n` copies into `gi[n % 2]` and reads the
    /// other. Made by [`Persistent::ensure_gi`] on the first such frame.
    pub gi: Option<((u32, u32), [wgpu::TextureView; 2])>,
}

impl Persistent {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        output_format: wgpu::TextureFormat,
        linear_format: Option<wgpu::TextureFormat>,
        antialias: Antialias,
        chain_format: wgpu::TextureFormat,
    ) -> Persistent {
        let attach_bind =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let offscreen = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: output_format,
            usage: attach_bind | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
            view_formats: linear_format.as_slice(),
        });
        let offscreen_view = offscreen.create_view(&Default::default());
        let offscreen_linear = linear_format.map(|format| {
            offscreen.create_view(&wgpu::TextureViewDescriptor {
                format: Some(format),
                ..Default::default()
            })
        });
        let history = (antialias == Antialias::Taa).then(|| {
            [0, 1].map(|_| {
                target_texture(
                    device,
                    "taa history",
                    width,
                    height,
                    HDR_FORMAT,
                    // FSR 3.1 writes it from compute (fsr.rs).
                    attach_bind | wgpu::TextureUsages::STORAGE_BINDING,
                )
                .create_view(&Default::default())
            })
        });
        Persistent {
            offscreen,
            offscreen_view,
            offscreen_linear,
            history,
            smaa_history: smaa_history(device, width, height, chain_format, antialias),
            gi: None,
        }
    }

    /// The GI's copies at the scene's size; true when they were made anew
    /// (what they hold is not the previous frame's).
    pub fn ensure_gi(&mut self, device: &wgpu::Device, width: u32, height: u32) -> bool {
        if self
            .gi
            .as_ref()
            .is_some_and(|(size, _)| *size == (width, height))
        {
            return false;
        }
        let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        self.gi = Some((
            (width, height),
            [0, 1].map(|_| {
                target_texture(device, "gi history", width, height, HDR_FORMAT, usage)
                    .create_view(&Default::default())
            }),
        ));
        true
    }
}

impl Persistent {
    /// The TAA history at a new scene size (the offscreen target keeps the
    /// output's).
    pub fn resize_history(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        antialias: Antialias,
        chain_format: wgpu::TextureFormat,
    ) {
        let attach_bind =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        self.history = (antialias == Antialias::Taa).then(|| {
            [0, 1].map(|_| {
                target_texture(
                    device,
                    "taa history",
                    width,
                    height,
                    HDR_FORMAT,
                    // FSR 3.1 writes it from compute (fsr.rs).
                    attach_bind | wgpu::TextureUsages::STORAGE_BINDING,
                )
                .create_view(&Default::default())
            })
        });
        self.smaa_history = smaa_history(device, width, height, chain_format, antialias);
    }
}

fn smaa_history(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    antialias: Antialias,
) -> Option<[wgpu::TextureView; 2]> {
    (antialias == Antialias::SmaaT2x).then(|| {
        [0, 1].map(|_| {
            target_texture(
                device,
                "smaa history",
                width,
                height,
                format,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .create_view(&Default::default())
        })
    })
}

/// A `width` × `height` texture of `format` from its RGBA8 texels.
pub(crate) fn texture_from_rgba8(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    rgba: &[u8],
) -> wgpu::TextureView {
    use wgpu::util::DeviceExt;
    device
        .create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            rgba,
        )
        .create_view(&Default::default())
}

/// The bloom chain's length for a target size (the first level at half
/// resolution, each next one half the previous).
pub(crate) fn bloom_levels(width: u32, height: u32) -> u32 {
    let (bw, bh) = ((width / 2).max(1), (height / 2).max(1));
    (32 - bw.min(bh).max(1).leading_zeros())
        .saturating_sub(3)
        .clamp(1, MAX_BLOOM_LEVELS)
}

/// Levels of the auto-exposure metering chain (64, 8, 1).
pub(crate) fn meter_levels() -> u32 {
    let mut n = 1;
    let mut size = METER_SIZE;
    while size > 1 {
        size /= 8;
        n += 1;
    }
    n
}
