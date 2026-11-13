//! Mass elements (`docs/ui.md` §1): damage numbers, overhead bars,
//! nameplate text, loot labels and status icons at thousands per frame.
//!
//! A [`UiMassTemplate`] is declared once: what each element draws (a
//! number formatted on the GPU, one of the template's strings, a bar, an
//! icon), its size, its motion as a timeline over its life
//! ([`UiMassKey`]), how distance scales it and whether the scene hides it.
//! Each element is a [`UiMassInstance`] of a few values (position, value,
//! colour, start tick, seed). A [`UiMassBatch`] of them is one instanced
//! draw of the overlay, sorted by layer with the rest of the UI: the vertex
//! stage projects the position, evaluates the timeline at the element's
//! age, formats the number and places each glyph (`shaders/ui_mass.wgsl`),
//! so nothing is laid out per element on the CPU.
//!
//! [`UiMassTemplate::sample`] evaluates the shared timing value on the CPU.
//! The shader uses exact simple opcodes or a deduplicated 4097-sample LUT;
//! readback tests measure approximation error and preserve step boundaries.

use glam::Vec2;
pub use og_draw::ui::mass::*;
use og_tween::{Ease, StepPosition};
use std::collections::BTreeMap;

/// Private shader opcodes; authors always use the shared timing value.
#[derive(Clone, Copy)]
enum EaseOpcode {
    Linear,
    Out,
    In,
    Back,
    Step,
    Lut,
}

/// Samples per distinct non-opcode curve, plus one metadata texel.
pub const UI_MASS_EASE_SAMPLES: usize = 4097;

#[derive(Default)]
pub(crate) struct EaseTables {
    samples: BTreeMap<[u32; 5], Vec<[f32; 4]>>,
    offsets: BTreeMap<[u32; 5], u32>,
}

impl EaseTables {
    pub(crate) fn begin_frame(&mut self) {
        self.offsets.clear();
    }

    pub(crate) fn end_frame(&mut self) {
        self.samples.retain(|key, _| self.offsets.contains_key(key));
    }

    fn pack(&mut self, ease: Ease, table: &mut Vec<[f32; 4]>) -> (EaseOpcode, u32) {
        let opcode = match ease {
            Ease::Linear => EaseOpcode::Linear,
            Ease::QuadOut => EaseOpcode::Out,
            Ease::QuadIn => EaseOpcode::In,
            Ease::BackOut => EaseOpcode::Back,
            Ease::Steps {
                count: 1,
                position: StepPosition::JumpEnd,
            } => EaseOpcode::Step,
            _ => EaseOpcode::Lut,
        };
        if !matches!(opcode, EaseOpcode::Lut) {
            return (opcode, 0);
        }
        let mut key = [ease.code(), 0, 0, 0, 0];
        match ease {
            Ease::CubicBezier(c) => key[1..].copy_from_slice(&c.map(f32::to_bits)),
            Ease::Steps { count, position } => {
                key[1] = count;
                key[2] = position as u32;
            }
            _ => {}
        }
        let base = *self.offsets.entry(key).or_insert_with(|| {
            let base = table.len() as u32;
            let samples = self.samples.entry(key).or_insert_with(|| {
                let mut samples = Vec::with_capacity(UI_MASS_EASE_SAMPLES + 1);
                let count = match ease {
                    Ease::Steps { count, .. } => count,
                    _ => 0,
                };
                samples.push([count as f32, ease.apply(0.0), ease.apply(1.0), 0.0]);
                for i in 0..UI_MASS_EASE_SAMPLES {
                    let t = i as f64 / (UI_MASS_EASE_SAMPLES - 1) as f64;
                    let (x, at) = if count == 0 {
                        (t as f32, t as f32)
                    } else {
                        // The lookup domain is the *integer step ordinal*, not
                        // time. Interpolating sparse ordinals never softens a jump.
                        let ordinal = (t * f64::from(count)).floor();
                        (
                            ordinal as f32,
                            ((ordinal + 0.5) / f64::from(count)).min(1.0) as f32,
                        )
                    };
                    samples.push([x, ease.apply(at), 0.0, 0.0]);
                }
                samples
            });
            table.extend_from_slice(samples);
            base
        });
        (opcode, base)
    }
}
/// Table entries per row of the table texture (`ui_mass.wgsl`'s `table`).
pub(crate) const TABLE_WIDTH: u32 = 256;

/// [`UiMassInstance`] as the mass pipeline's instance vertex buffer.
pub(crate) const MASS_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<UiMassInstance>() as u64,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![
        0 => Float32x4, 1 => Float32x4, 2 => Uint32x2, 3 => Float32, 4 => Uint32, 5 => Float32x4,
        6 => Float32x4
    ],
};

/// `struct MassTemplate` in ui_mass.wgsl: one batch's template.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct TemplateGpu {
    head: [u32; 4],
    head2: [u32; 4],
    size: [f32; 4],
    dist: [f32; 4],
    occl: [f32; 4],
    outline: [f32; 4],
    misc: [f32; 4],
    back: [f32; 4],
    ghost: [f32; 4],
    clip: [f32; 4],
    clip_head: [u32; 4],
    keys: [[f32; 4]; UI_MASS_KEYS],
    offsets: [[f32; 4]; UI_MASS_KEYS],
}

/// Texels of the table texture one template takes (`ui_mass.wgsl`'s
/// `mass_template`), at the texture's start; the glyph entries follow them.
pub(crate) const TEMPLATE_TEXELS: usize = 32;

impl TemplateGpu {
    /// Its texels: integers as exact floats.
    pub(crate) fn texels(&self) -> [[f32; 4]; TEMPLATE_TEXELS] {
        let u = |v: [u32; 4]| v.map(|x| x as f32);
        let mut out = [[0.0; 4]; TEMPLATE_TEXELS];
        let rows = [
            u(self.head),
            u(self.head2),
            self.size,
            self.dist,
            self.occl,
            self.outline,
            self.misc,
            self.back,
            self.ghost,
            self.clip,
            u(self.clip_head),
        ];
        out[..rows.len()].copy_from_slice(&rows);
        out[11..11 + UI_MASS_KEYS].copy_from_slice(&self.keys);
        out[11 + UI_MASS_KEYS..11 + 2 * UI_MASS_KEYS].copy_from_slice(&self.offsets);
        out
    }
}

/// `struct MassCamera` in ui_mass.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct MassCameraGpu {
    pub view_proj: [[f32; 4]; 4],
    pub eye: [f32; 4],
    pub forward: [f32; 4],
    pub css: [f32; 4],
    pub tick: [u32; 4],
}

/// The symbols a number draws, in the order of the shader's codes.
const NUMBER_SYMBOLS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '-', '.', 'k', 'M', ' ',
];

const FLAG_WORLD: u32 = 1;
const FLAG_OCCLUDE: u32 = 2;
const FLAG_HIDE: u32 = 4;
const FLAG_OUTLINE: u32 = 8;
const FLAG_PLUS: u32 = 16;
const FLAG_COMPACT: u32 = 32;
const FLAG_DISTANCE: u32 = 64;
const FLAG_DRAG: u32 = 128;

/// A glyph's three table entries: its uv, its plane bounds moved by `pen`
/// (em), its advance.
fn glyph_entries(font: &crate::sprites::FontLayout, ch: char, pen: f32, out: &mut Vec<[f32; 4]>) {
    match font.glyphs.get(&crate::GlyphKey::Char(ch)) {
        Some(g) => {
            let [al, at, ar, ab] = g.atlas_bounds;
            let size = font.atlas_size.max(Vec2::ONE);
            let [l, b, r, t] = g.plane_bounds;
            out.push([al / size.x, at / size.y, ar / size.x, ab / size.y]);
            out.push([l + pen, b, r + pen, t]);
            out.push([g.advance, 0.0, 0.0, 0.0]);
        }
        None => {
            out.push([0.0; 4]);
            out.push([0.0; 4]);
            out.push([0.0; 4]);
        }
    }
}

/// Batch `b`'s template on the GPU, its glyphs, strings or frames appended
/// to `table`. `clip`: its clip rectangle (x0, y0, x1, y1) and clip head.
pub(crate) fn template_gpu(
    b: &UiMassBatch,
    clip: ([f32; 4], u32),
    font: Option<&crate::sprites::FontLayout>,
    table: &mut Vec<[f32; 4]>,
    eases: &mut EaseTables,
) -> TemplateGpu {
    let t = &*b.template;
    let base = table.len() as u32;
    let mut flags = 0;
    if b.space == UiMassSpace::World {
        flags |= FLAG_WORLD;
    }
    match t.occlusion {
        UiOcclusion::None => {}
        UiOcclusion::Fade { .. } => flags |= FLAG_OCCLUDE,
        UiOcclusion::Hide => flags |= FLAG_HIDE,
    }
    if t.outline.is_some_and(|(w, c)| w > 0.0 && c[3] > 0.0) {
        flags |= FLAG_OUTLINE;
    }
    if t.distance.x > 0.0 {
        flags |= FLAG_DISTANCE;
    }
    if t.drag {
        flags |= FLAG_DRAG;
    }
    let (kind, entries, decimals) = match &t.kind {
        UiMassKind::Number {
            decimals,
            plus,
            compact,
        } => {
            if *plus {
                flags |= FLAG_PLUS;
            }
            if *compact {
                flags |= FLAG_COMPACT;
            }
            for ch in NUMBER_SYMBOLS {
                match font {
                    Some(f) => glyph_entries(f, ch, 0.0, table),
                    None => table.extend([[0.0; 4]; 3]),
                }
            }
            (0, 16, *decimals)
        }
        UiMassKind::Text { strings } => {
            // The runs first (start, count, width), then every string's
            // glyphs laid out with the font's advances and kerning.
            let runs = table.len();
            table.extend(std::iter::repeat_n([0.0; 4], strings.len()));
            let mut start = 0usize;
            for (i, s) in strings.iter().enumerate() {
                let mut pen = 0.0;
                let mut prev: Option<char> = None;
                let mut count = 0usize;
                for ch in s.chars().take(UI_MASS_QUADS as usize) {
                    if let (Some(f), Some(p)) = (font, prev) {
                        pen += f.kerning.get(&(p, ch)).copied().unwrap_or(0.0);
                    }
                    match font {
                        Some(f) => {
                            glyph_entries(f, ch, pen, table);
                            pen += f
                                .glyphs
                                .get(&crate::GlyphKey::Char(ch))
                                .map_or(0.0, |g| g.advance);
                        }
                        None => table.extend([[0.0; 4]; 3]),
                    }
                    prev = Some(ch);
                    count += 1;
                }
                table[runs + i] = [start as f32, count as f32, pen, 0.0];
                start += count;
            }
            (1, strings.len() as u32, 0)
        }
        UiMassKind::Bar { .. } => (2, 0, 0),
        UiMassKind::Icon { frames } => {
            table.extend(frames.iter().copied());
            (3, frames.len() as u32, 0)
        }
        UiMassKind::Particle { .. } => (4, 0, 0),
    };
    let keys = t.keys_drawn();
    let mut g = TemplateGpu {
        head: [kind, t.quads(), base, flags],
        head2: [t.lifetime, keys.len() as u32, decimals, entries],
        size: [t.size.x, t.size.y, t.pivot.x, t.pivot.y],
        dist: [t.distance.x, t.distance.y, t.distance.z, t.drift],
        occl: [
            match t.occlusion {
                UiOcclusion::Fade { alpha } => alpha.clamp(0.0, 1.0),
                _ => 0.0,
            },
            0.25,
            0.5,
            t.gravity,
        ],
        clip: clip.0,
        clip_head: [0, 0, 0, clip.1],
        ..Default::default()
    };
    if let Some((w, c)) = t.outline {
        g.outline = c;
        g.misc[0] = w;
    }
    if let UiMassKind::Bar {
        back,
        ghost,
        radius,
    } = &t.kind
    {
        g.back = *back;
        g.ghost = *ghost;
        g.misc[1] = *radius;
    }
    if let UiMassKind::Particle { shape } = &t.kind {
        g.misc[2] = match shape {
            UiParticleShape::Square => 0.0,
            UiParticleShape::Circle => 1.0,
            UiParticleShape::Diamond => 2.0,
        };
    }
    for (i, k) in keys.iter().enumerate() {
        let (opcode, base) = eases.pack(k.ease, table);
        g.keys[i] = [k.t, k.scale, k.alpha, opcode as u32 as f32];
        g.offsets[i] = [k.offset.x, k.offset.y, base as f32, 0.0];
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn numbers_format_as_the_shader_writes_them() {
        assert_eq!(format_number(1234.0, 0, false, false), "1234");
        assert_eq!(format_number(-17.0, 0, false, false), "-17");
        assert_eq!(format_number(17.0, 0, true, false), "+17");
        // Zero takes no sign, rounded or not.
        assert_eq!(format_number(0.2, 0, true, false), "0");
        assert_eq!(format_number(7.456, 2, false, false), "7.46");
        assert_eq!(format_number(0.05, 1, false, false), "0.1");
        assert_eq!(format_number(12_345.0, 0, false, true), "12.3k");
        // Under the compact threshold numbers stay whole.
        assert_eq!(format_number(9_999.0, 0, false, true), "9999");
        assert_eq!(format_number(2_500_000.0, 0, true, true), "+2.5M");
    }

    #[test]
    fn a_timeline_eases_between_its_keys_and_holds_its_ends() {
        let tpl = UiMassTemplate {
            keys: vec![
                UiMassKey {
                    t: 1.0,
                    offset: Vec2::new(0.0, -40.0),
                    alpha: 0.0,
                    ease: Ease::QuadOut,
                    ..Default::default()
                },
                UiMassKey {
                    t: 0.0,
                    scale: 2.0,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        // Keys are sorted: t = 0 is the first.
        let (o, s, a) = tpl.sample(0.0);
        assert_eq!((o, s, a), (Vec2::ZERO, 2.0, 1.0));
        // Half way out of a quadratic ease-out is three quarters of the way.
        let (o, s, a) = tpl.sample(0.5);
        assert!((o.y + 30.0).abs() < 1e-4, "{o}");
        assert!((s - 1.25).abs() < 1e-4 && (a - 0.25).abs() < 1e-4);
        assert_eq!(tpl.sample(3.0).2, 0.0);
    }

    #[test]
    #[ignore = "requires GPU"]
    fn shared_mass_curves_match_gpu_readback() {
        let mut curves = og_tween::EASES.to_vec();
        curves.push(Ease::CubicBezier([0.17, 0.85, 0.63, 0.08]));
        for count in [1, 3, 4, 7, 1027, 8195] {
            for position in [
                StepPosition::JumpStart,
                StepPosition::JumpEnd,
                StepPosition::JumpBoth,
                StepPosition::JumpNone,
            ] {
                let ease = Ease::Steps { count, position };
                if ease.is_valid() {
                    curves.push(ease);
                }
            }
        }
        let mut cache = EaseTables::default();
        let mut table = Vec::new();
        let mut texels = Vec::new();
        let mut inputs = Vec::<[f32; 4]>::new();
        let mut expected = Vec::new();
        for (index, &ease) in curves.iter().enumerate() {
            let template = Arc::new(UiMassTemplate {
                kind: UiMassKind::Particle {
                    shape: UiParticleShape::Square,
                },
                keys: vec![
                    UiMassKey::default(),
                    UiMassKey {
                        t: 1.0,
                        offset: Vec2::new(1.0, -3.0),
                        scale: 2.0,
                        alpha: 0.0,
                        ease,
                    },
                ],
                ..Default::default()
            });
            let batch = UiMassBatch {
                template: template.clone(),
                space: UiMassSpace::Screen,
                instances: vec![],
                layer: 0,
                clip: None,
            };
            texels
                .extend(template_gpu(&batch, ([0.0; 4], 0), None, &mut table, &mut cache).texels());
            let bytes = table.len();
            template_gpu(&batch, ([0.0; 4], 0), None, &mut table, &mut cache);
            assert_eq!(table.len(), bytes, "same curve does not duplicate its LUT");
            // Prime spacing exercises interpolation, not just LUT knots.
            let mut times: Vec<_> = (0..=4093).map(|i| i as f32 / 4093.0).collect();
            if let Ease::Steps { count, .. } = ease {
                for j in 1..count.min(10) {
                    let t = j as f32 / count as f32;
                    times.extend([
                        f32::from_bits(t.to_bits() - 1),
                        t,
                        f32::from_bits(t.to_bits() + 1),
                    ]);
                }
            }
            for t in times {
                inputs.push([index as f32, t, 0.0, 0.0]);
                expected.push(template.sample(t).0.x);
            }
        }
        let base = texels.len();
        texels.extend(table);
        let source = include_str!("../../../shaders/ui_mass.wgsl");
        let structure = &source[source.find("struct MassTemplate").unwrap()
            ..source.find("// The table texture").unwrap()];
        let template = &source
            [source.find("fn mass_template").unwrap()..source.find("// One element:").unwrap()];
        let sampler =
            &source[source.find("fn ease_lut").unwrap()..source.find("fn hash01").unwrap()];
        let shader = format!(
            r#"
@group(0) @binding(0) var<storage, read> texels: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> inputs: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> outputs: array<vec4<f32>>;
fn texel(i: u32) -> vec4<f32> {{ return texels[i]; }}
fn table(i: u32) -> vec4<f32> {{ return texels[{base}u + i]; }}
{structure}
{template}
{sampler}
@compute @workgroup_size(64) fn test(@builtin(global_invocation_id) id: vec3<u32>) {{
  if (id.x < arrayLength(&inputs)) {{
    outputs[id.x] = sample_keys(mass_template(u32(inputs[id.x].x)), inputs[id.x].y);
  }}
}}
"#
        );
        let instance = crate::RenderDesc::default().backend.instance();
        let adapter =
            pollster::block_on(instance.request_adapter(&Default::default())).expect("GPU adapter");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mass timing readback"),
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &module,
            entry_point: Some("test"),
            compilation_options: Default::default(),
            cache: None,
        });
        let data = crate::buffer_init(
            &device,
            &queue,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&texels),
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let queries = crate::buffer_init(
            &device,
            &queue,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&inputs),
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let size = (inputs.len() * 16) as u64;
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let entries: Vec<_> = [&data, &queries, &output]
            .iter()
            .enumerate()
            .map(|(binding, b)| wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: b.as_entire_binding(),
            })
            .collect();
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups((inputs.len() as u32).div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&output, 0, &staging, 0, size);
        queue.submit([encoder.finish()]);
        staging.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let mapped = staging.slice(..).get_mapped_range().unwrap();
        let results: &[[f32; 4]] = bytemuck::cast_slice(&mapped);
        let mut maxima = vec![0.0f32; curves.len()];
        for ((input, &expected), actual) in inputs.iter().zip(&expected).zip(results) {
            let index = input[0] as usize;
            let error = (actual[0] - expected).abs();
            maxima[index] = maxima[index].max(error);
            let tolerance = match curves[index] {
                Ease::Linear | Ease::QuadIn | Ease::QuadOut | Ease::BackOut => 1e-6,
                Ease::Steps {
                    count: 1..=4096, ..
                } => 1e-6,
                _ => 0.00025,
            };
            assert!(
                error <= tolerance,
                "{:?} at {}: GPU {} CPU {expected}, error {error}",
                curves[index],
                input[1],
                actual[0]
            );
        }
        for (curve, error) in curves.iter().zip(maxima) {
            println!("{curve:?}: max error {error:.9}");
        }
    }
}
