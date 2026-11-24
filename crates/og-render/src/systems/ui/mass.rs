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

use og_core::ease::{Ease, StepPosition};
use og_draw::glyphs::GlyphTier;
pub use og_draw::ui::mass::*;
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
    bar_rim: [f32; 4],
    bar_shield: [f32; 4],
    bar_resource: [f32; 4],
    bar_low: [f32; 4],
    // Minimum health size, low threshold and maximum world distance.
    bar: [f32; 4],
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
        out[27..32].copy_from_slice(&[
            self.bar_rim,
            self.bar_shield,
            self.bar_resource,
            self.bar_low,
            self.bar,
        ]);
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

const FLAG_WORLD: u32 = 1;
const FLAG_OCCLUDE: u32 = 2;
const FLAG_HIDE: u32 = 4;
const FLAG_OUTLINE: u32 = 8;
const FLAG_PLUS: u32 = 16;
const FLAG_COMPACT: u32 = 32;
const FLAG_DISTANCE: u32 = 64;
const FLAG_DRAG: u32 = 128;

/// A template glyph's three table entries: its uv (texels over the page's
/// side), its quad (em) and its advance, with a colour glyph's flag in the
/// advance texel.
fn glyph_entries(g: &UiMassGlyph, side: f32, out: &mut Vec<[f32; 4]>) {
    let [ul, ut, ur, ub] = g.glyph.uv.map(f32::from);
    out.push([ul / side, ut / side, ur / side, ub / side]);
    out.push(g.glyph.rect);
    out.push([
        g.advance,
        u32::from(g.glyph.tier == GlyphTier::Color) as f32,
        0.0,
        0.0,
    ]);
}

/// The blank entries of a glyph that draws from another page of the glyph
/// atlas (or found none): no uv, an empty quad, but its own advance, which
/// the number's pen still moves by.
fn blank_entries(advance: f32, out: &mut Vec<[f32; 4]>) {
    out.push([0.0; 4]);
    out.push([0.0; 4]);
    out.push([advance, 0.0, 0.0, 0.0]);
}

/// Batch `b`'s template on the GPU, its glyphs, strings or frames appended
/// to `table`. `clip`: its clip rectangle (x0, y0, x1, y1) and clip head.
/// `sides`: the glyph atlas's page side by tier, which its glyphs' texels
/// read. `page`: which page of the glyph atlas this table packs its text
/// from (`None`: a kind with no text; a text kind's glyphs from the other
/// pages take blank entries that keep their advances).
pub(crate) fn template_gpu(
    b: &UiMassBatch,
    clip: ([f32; 4], u32),
    sides: [f32; 3],
    page: Option<(GlyphTier, u16)>,
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
            glyphs,
        } => {
            if *plus {
                flags |= FLAG_PLUS;
            }
            if *compact {
                flags |= FLAG_COMPACT;
            }
            // The symbols in the shader's code order, whether this page has
            // them or not: the shader places each by the advances before it.
            for code in 0..NUMBER_SYMBOLS.len() {
                match glyphs.get(code) {
                    Some(g) if g.glyph.rect[2] > g.glyph.rect[0] => {
                        if Some((g.glyph.tier, g.glyph.page)) == page {
                            glyph_entries(g, sides[g.glyph.tier as usize].max(1.0), table);
                        } else {
                            blank_entries(g.advance, table);
                        }
                    }
                    Some(g) => blank_entries(g.advance, table),
                    None => blank_entries(0.0, table),
                }
            }
            (0, NUMBER_SYMBOLS.len() as u32, *decimals)
        }
        UiMassKind::Text { strings } => {
            // The runs first (start, count, width), then the page's glyphs
            // in string order, each's pen in its quad; a string's glyphs
            // from the other pages take no entry, its run on this page
            // counts the ones it has.
            let runs = table.len();
            table.extend(std::iter::repeat_n([0.0; 4], strings.len()));
            let mut start = 0usize;
            let mut on_page = Vec::new();
            for (i, s) in strings.iter().enumerate() {
                let mut count = 0usize;
                for g in &s.glyphs {
                    if let Some((tier, page)) = page
                        && g.glyph.rect[2] > g.glyph.rect[0]
                        && g.glyph.tier == tier
                        && g.glyph.page == page
                    {
                        on_page.push(g);
                        count += 1;
                    }
                }
                table[runs + i] = [start as f32, count as f32, s.width, 0.0];
                start += count;
            }
            for g in &on_page {
                glyph_entries(g, sides[g.glyph.tier as usize].max(1.0), table);
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
    g.bar[3] = t.max_distance;
    if let Some(bar) = &t.bar {
        g.bar_rim = bar.rim;
        g.bar_shield = bar.shield;
        g.bar_resource = bar.resource;
        g.bar_low = bar.low;
        g.bar[..3].copy_from_slice(&[bar.min_size.x, bar.min_size.y, bar.low_at]);
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
