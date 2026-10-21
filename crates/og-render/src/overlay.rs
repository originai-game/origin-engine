//! The overlay pass ([`crate::Overlay`]): screen-space sprites and MSDF text
//! drawn after every post effect into the output, with per-instance clip
//! rectangles. Sorted like the world's 2D batch (layer, kind, texture,
//! texture array or font; submission order within a run), one instanced
//! draw per run.

use crate::RenderError;
use crate::effects::SpriteBlend;
use crate::scene::{Overlay, TextureArrayHandle, TextureHandle};
use crate::sprites::{FontLayout, RunSource, SpriteRun};
use glam::Vec2;

/// `struct OverlayIn` in overlay.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct OverlayInstance {
    position_rotation: [f32; 4],
    size_anchor: [f32; 4],
    uv_rect: [f32; 4],
    color: [f32; 4],
    clip: [f32; 4],
    /// MSDF text: x the glow's radius in CSS px, y the weight (CSS px of
    /// thickening), z 1 for a glow-only glyph (drawn under the text's fill),
    /// w unused. Zero for sprites and plain text; a sprite drawn from a
    /// texture array has its layer in w (`shaders/overlay_array.wgsl`).
    effect: [f32; 4],
    /// The glow's linear RGBA.
    effect_color: [f32; 4],
}

pub(crate) const OVERLAY_STRIDE: u64 = std::mem::size_of::<OverlayInstance>() as u64;

pub(crate) const OVERLAY_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: OVERLAY_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4],
};

/// The overlay's sorted instances and runs for one frame.
pub(crate) struct OverlayFrame {
    pub instances: Vec<OverlayInstance>,
    pub runs: Vec<SpriteRun>,
    pub sprites: u32,
    pub glyphs: u32,
    pub missing_glyphs: u32,
}

/// No clip: far outside any output.
const UNCLIPPED: [f32; 4] = [-1.0e9, -1.0e9, 1.0e9, 1.0e9];

fn clip_rect(overlay: &Overlay, clip: Option<u32>, what: &str) -> Result<[f32; 4], RenderError> {
    let Some(i) = clip else {
        return Ok(UNCLIPPED);
    };
    let [x, y, w, h] = *overlay.clips.get(i as usize).ok_or_else(|| {
        RenderError::InvalidHandle(format!(
            "overlay {what}: clip {i} of {}",
            overlay.clips.len()
        ))
    })?;
    Ok([x, y, x + w.max(0.0), y + h.max(0.0)])
}

/// Sorts an overlay's sprites and texts into runs (the world batch's order).
/// `array_layers(a)`: the layers of an uploaded texture array (`None`: not
/// uploaded or removed). A sprite drawn from an array
/// ([`Overlay::sprite_arrays`]) joins its array's run.
pub(crate) fn build<'a>(
    overlay: &Overlay,
    font: impl Fn(usize) -> Option<&'a FontLayout>,
    texture_count: usize,
    array_layers: impl Fn(TextureArrayHandle) -> Option<u32>,
) -> Result<OverlayFrame, RenderError> {
    let arrays = crate::sprites::array_entries(
        &overlay.sprite_arrays,
        overlay.sprites.len(),
        "overlay sprite",
        array_layers,
    )?;
    let mut keyed: Vec<(i32, RunSource, OverlayInstance)> =
        Vec::with_capacity(overlay.sprites.len() + overlay.texts.len() * 8);
    for (i, s) in overlay.sprites.iter().enumerate() {
        let clip = clip_rect(overlay, s.clip, &format!("sprite {i}"))?;
        let mut inst = OverlayInstance {
            position_rotation: [s.position.x, s.position.y, 0.0, s.rotation],
            size_anchor: [s.size.x, s.size.y, s.anchor.x, s.anchor.y],
            uv_rect: s.uv_rect,
            color: s.color,
            clip,
            effect: [0.0; 4],
            effect_color: [0.0; 4],
        };
        if let Some(&(array, layer)) = arrays.get(&(i as u32)) {
            inst.effect[3] = layer as f32;
            keyed.push((s.layer, RunSource::Array(SpriteBlend::Alpha, array), inst));
            continue;
        }
        if let Some(t) = s.texture
            && t.index() >= texture_count
        {
            return Err(RenderError::InvalidHandle(format!(
                "overlay sprite {i}: texture {}",
                t.index()
            )));
        }
        keyed.push((
            s.layer,
            RunSource::Sprite(SpriteBlend::Alpha, s.texture),
            inst,
        ));
    }
    for (i, g) in overlay.glass.iter().enumerate() {
        let clip = clip_rect(overlay, g.clip, &format!("glass {i}"))?;
        keyed.push((
            g.layer,
            RunSource::Glass,
            OverlayInstance {
                position_rotation: [g.position.x, g.position.y, 0.0, 0.0],
                size_anchor: [g.size.x, g.size.y, 0.0, 0.0],
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: g.tint,
                clip,
                effect: [g.radius.max(0.0), g.blur.max(0.0), g.size.x, g.size.y],
                effect_color: [0.0, 0.0, 0.0, g.alpha.clamp(0.0, 1.0)],
            },
        ));
    }
    let sprites = keyed.len() as u32;
    let mut missing_glyphs = 0;
    for (i, text) in overlay.texts.iter().enumerate() {
        let layout = font(text.font.index()).ok_or_else(|| {
            RenderError::InvalidHandle(format!("overlay text {i}: font {}", text.font.index()))
        })?;
        let clip = clip_rect(overlay, text.clip, &format!("text {i}"))?;
        let (sin, cos) = text.rotation.sin_cos();
        // y down: a positive rotation turns clockwise on screen.
        let rotate = |v: Vec2| Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos);
        let origin = text.position.truncate();
        let glow = text.glow.filter(|g| g.radius > 0.0 && g.color[3] > 0.0);
        let weight = text.weight.max(0.0);
        // A glowing text draws every glyph's glow first, then every fill, so
        // no glyph's glow lies over its neighbour's ink.
        for pass in [true, false] {
            if pass && glow.is_none() {
                continue;
            }
            // The pen in em, y down from the first baseline.
            layout.walk(text, layout.line_height, |ch, g, pen, offset| {
                let Some(g) = g else {
                    if !ch.is_whitespace() && !pass {
                        missing_glyphs += 1;
                    }
                    return;
                };
                let [l, b, r, t] = g.plane_bounds;
                if r > l && t > b {
                    // plane bounds are em y up from the baseline: the glyph's top
                    // left is (l, -t) in y-down em; a placement's y is up too.
                    let corner = rotate(
                        (pen + Vec2::new(offset.x, -offset.y) + Vec2::new(l, -t)) * text.size,
                    );
                    let [al, at, ar, ab] = g.atlas_bounds;
                    let (effect, effect_color) = match glow {
                        Some(gl) => (
                            [gl.radius, weight, f32::from(u8::from(pass)), 0.0],
                            gl.color,
                        ),
                        None => ([0.0, weight, 0.0, 0.0], [0.0; 4]),
                    };
                    keyed.push((
                        text.layer,
                        RunSource::Font(text.font),
                        OverlayInstance {
                            position_rotation: [
                                origin.x + corner.x,
                                origin.y + corner.y,
                                0.0,
                                text.rotation,
                            ],
                            size_anchor: [(r - l) * text.size, (t - b) * text.size, 0.0, 0.0],
                            uv_rect: [
                                al / layout.atlas_size.x,
                                at / layout.atlas_size.y,
                                ar / layout.atlas_size.x,
                                ab / layout.atlas_size.y,
                            ],
                            color: text.color,
                            clip,
                            effect,
                            effect_color,
                        },
                    ));
                }
            });
        }
    }
    let glyphs = keyed.len() as u32 - sprites;
    keyed.sort_by_key(|(layer, source, _)| (*layer, *source));
    let mut runs: Vec<SpriteRun> = Vec::new();
    let mut instances = Vec::with_capacity(keyed.len());
    for (index, (_, source, inst)) in keyed.into_iter().enumerate() {
        match runs.last_mut() {
            Some(run) if run.source == source => run.count += 1,
            _ => runs.push(SpriteRun {
                source,
                first: index as u32,
                count: 1,
                // The overlay is drawn after the tonemap.
                tone_mapped: true,
            }),
        }
        instances.push(inst);
    }
    Ok(OverlayFrame {
        instances,
        runs,
        sprites,
        glyphs,
        missing_glyphs,
    })
}

/// The overlay's sprite and MSDF pipelines for one target format.
pub(crate) struct OverlayPipelines {
    pub sprite: wgpu::RenderPipeline,
    pub msdf: wgpu::RenderPipeline,
    pub glass: wgpu::RenderPipeline,
}

/// The overlay's pipelines and buffers: in the output's format and, for a
/// non-sRGB output the backend can view as sRGB, in that sRGB format.
pub(crate) struct OverlayRenderer {
    pub output: OverlayPipelines,
    pub linear: Option<OverlayPipelines>,
    /// Group 0 (the uniform), and the formats of `output` and `linear`:
    /// what the array sprite variant (src/sprite_array.rs) is made with.
    pub layout: wgpu::BindGroupLayout,
    pub output_format: wgpu::TextureFormat,
    pub linear_format: Option<wgpu::TextureFormat>,
    pub uniform: wgpu::Buffer,
    pub group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
}

impl OverlayRenderer {
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        linear_format: Option<wgpu::TextureFormat>,
        sprite_layout: &wgpu::BindGroupLayout,
    ) -> OverlayRenderer {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("overlay"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("overlay"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("overlay"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("overlay"),
            bind_group_layouts: &[Some(&layout), Some(sprite_layout)],
            immediate_size: 0,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("overlay"),
            source: wgpu::ShaderSource::Wgsl(OVERLAY_WGSL.into()),
        });
        let buffers = [Some(OVERLAY_LAYOUT)];
        let pipeline = |label: &str, fs: &str, format: wgpu::TextureFormat| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &buffers,
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        };
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("overlay instances"),
            size: OVERLAY_STRIDE * 256,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let pipelines = |format| OverlayPipelines {
            sprite: pipeline("overlay sprite", "fs", format),
            msdf: pipeline("overlay msdf", "fs_msdf", format),
            glass: pipeline("overlay glass", "fs_glass", format),
        };
        OverlayRenderer {
            output: pipelines(output_format),
            linear: linear_format.map(pipelines),
            layout,
            output_format,
            linear_format,
            uniform,
            group,
            buffer,
        }
    }

    /// The uniform for `overlay` on an output of `width`×`height` device
    /// pixels: its scale, or (0) the scale that fits its height to the output.
    pub fn uniform(width: u32, height: u32, overlay: &Overlay, encode: bool) -> [f32; 4] {
        let scale = if overlay.scale > 0.0 {
            overlay.scale
        } else if overlay.height > 0.0 {
            height as f32 / overlay.height
        } else {
            1.0
        };
        [width as f32, height as f32, scale, encode as u32 as f32]
    }
}

/// The texture arrays of the array runs (for bind groups made before the pass).
pub(crate) fn run_arrays(frame: &OverlayFrame) -> impl Iterator<Item = TextureArrayHandle> + '_ {
    frame.runs.iter().filter_map(|r| match r.source {
        RunSource::Array(_, a) => Some(a),
        _ => None,
    })
}

/// The texture of a sprite run (for bind groups made before the pass).
pub(crate) fn run_textures(
    frame: &OverlayFrame,
) -> impl Iterator<Item = Option<TextureHandle>> + '_ {
    frame.runs.iter().filter_map(|r| match r.source {
        RunSource::Sprite(_, t) => Some(t),
        RunSource::Font(_)
        | RunSource::Strip(..)
        | RunSource::Metaballs(_)
        | RunSource::Glass
        | RunSource::Array(..) => None,
    })
}

/// The overlay's module (`crate::shaders` validates it).
pub(crate) const OVERLAY_WGSL: &str = wgsl!("overlay.wgsl");
