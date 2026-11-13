//! The post chain's uniforms and its baked colour table.

use super::*;

impl PostPasses {
    /// Writes the stylisation passes' parameters.
    pub fn write_stylise(&self, queue: &wgpu::Queue, uniform: &StyliseUniform) {
        queue.write_buffer(&self.stylise_buffer, 0, bytemuck::bytes_of(uniform));
    }

    /// Bakes the table the tonemap samples for `adjust` after `grade` (the
    /// grading table with its scene, slot and revision, and its intensity)
    /// into [`PostPasses::adjusted_lut`], unless the last bake was of the
    /// same: the texture is made once and written in place, so its view
    /// (and the bindings made with it) stays.
    pub fn bake_adjusted_lut(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        adjust: &crate::ColorAdjust,
        grade: Option<(&crate::ColorLut, (u64, usize, u64), f32)>,
    ) {
        let key: AdjustedKey = (*adjust, grade.map(|(_, (s, i, r), k)| (s, i, r, k)));
        if self.adjusted_lut.as_ref().is_some_and(|(k, ..)| *k == key) {
            return;
        }
        use crate::effects::AdjustedLut;
        let lut = crate::ColorLut::adjusted(adjust, grade.map(|(lut, _, k)| (lut, k)));
        match &mut self.adjusted_lut {
            Some((k, texture, _)) => {
                write_lut(queue, texture, &lut);
                *k = key;
            }
            None => {
                let texture = new_lut_texture(device, "adjusted grading lut", lut.size);
                write_lut(queue, &texture, &lut);
                let view = texture.create_view(&Default::default());
                self.adjusted_lut = Some((key, texture, view));
            }
        }
    }

    /// The table [`PostPasses::bake_adjusted_lut`] last baked.
    pub fn adjusted_lut(&self) -> Option<&wgpu::TextureView> {
        self.adjusted_lut.as_ref().map(|(_, _, view)| view)
    }

    /// Writes the post uniform. `glow`: the glow buffer feeds the bloom
    /// chain; `god_rays`: their texture is added; `untonemapped`: the lit
    /// image's alpha is the fraction of each pixel to tone map (sprites
    /// drawn untonemapped lowered it); `tick` seeds the grain; `flare`:
    /// the lens flare's passes run, with the starburst's turn with the
    /// camera.
    #[allow(clippy::too_many_arguments)]
    pub fn write_uniform(
        &self,
        queue: &wgpu::Queue,
        exposure: Exposure,
        post: &PostEffects,
        glow: bool,
        god_rays: bool,
        untonemapped: bool,
        tick: u64,
        width: u32,
        flare: Option<(&crate::LensFlare, f32)>,
    ) {
        let (exposure, compensation) = Self::exposure_rows(exposure);
        let lit_bloom = post.bloom.filter(|b| b.intensity > 0.0);
        let bloom = match (lit_bloom, post.glow.filter(|_| glow)) {
            // UnrealBloomPass: the mips are stored weighted, so the top
            // level is their mix; the additive copy multiplies it by the
            // strength again (its alpha, strength times the weights' sum).
            (Some(b), _) => match post.bloom_unreal {
                Some(r) => {
                    let sum: f32 = (0..5)
                        .map(|i| {
                            let f = 1.0 - 0.2 * i as f32;
                            f + (1.2 - 2.0 * f) * r
                        })
                        .sum();
                    [
                        b.threshold,
                        b.knee,
                        b.intensity * b.intensity * sum,
                        compensation,
                    ]
                }
                None => [b.threshold, b.knee, b.intensity, compensation],
            },
            (None, Some(g)) => [1.0, 0.5, g.intensity.max(0.0), compensation],
            // The lens flare alone reads the chain's first level: the lit
            // image unthresholded (the flare's own thresholds decide), and
            // the tone curve adds none of it as bloom.
            (None, None) if flare.is_some() => [0.0, 0.0, 0.0, compensation],
            (None, None) => [1.0, 0.5, 0.0, compensation],
        };
        // The lit image feeds the chain with bloom, or for the flare alone
        // (with the 2D glow and no bloom, the flare reads the glow).
        let lit_feeds =
            lit_bloom.is_some() || (flare.is_some() && post.glow.filter(|_| glow).is_none());
        let tonemap = match post.tonemap {
            Tonemap::Aces => 0.0,
            Tonemap::None => 1.0,
            Tonemap::Agx => 2.0,
            Tonemap::Neutral => 3.0,
        };
        let v = post.vignette.unwrap_or(crate::Vignette {
            intensity: 0.0,
            ..Default::default()
        });
        let ca = post
            .chromatic_aberration
            .map_or(0.0, |c| (c.intensity.max(0.0) * 2.0 / 720.0).min(0.1));
        let grain = post.film_grain.map_or([0.0; 4], |g| {
            let seed = crate::particles::hash_unit(tick as u32 ^ (tick >> 32) as u32);
            [g.intensity.max(0.0), g.size.max(1.0), seed, 0.0]
        });
        let flash = post.flash.map_or([0.0; 4], |f| {
            f.color.extend(f.amount.clamp(0.0, 1.0)).to_array()
        });
        let uniform = PostUniform {
            hdr: self.hdr,
            exposure,
            bloom,
            mode: [tonemap, 1.0, lit_feeds as u32 as f32, glow as u32 as f32],
            grade: [
                // The adjusted table carries the grading at its intensity.
                if post.adjust().is_some() {
                    1.0
                } else {
                    post.color_grading
                        .map_or(0.0, |g| g.intensity.clamp(0.0, 1.0))
                },
                v.intensity.clamp(0.0, 1.0),
                v.radius,
                v.smoothness,
            ],
            vignette: v.color.extend(ca).to_array(),
            grain,
            flash,
            extra: [
                god_rays as u32 as f32,
                untonemapped as u32 as f32,
                post.sharpen.is_some() as u32 as f32,
                post.sharpen.unwrap_or(0.0).clamp(0.0, 1.0),
            ],
            unreal: match (lit_bloom, post.bloom_unreal) {
                (Some(_), Some(r)) => [
                    1.0,
                    r,
                    (width / 2).max(1) as f32,
                    if post.bloom_contrast > 0.0 {
                        post.bloom_contrast
                    } else {
                        1.0
                    },
                ],
                _ => [0.0; 4],
            },
            flare: flare.map_or([0.0; 4], |(f, turn)| {
                [f.intensity.max(0.0), f.starburst as u32 as f32, turn, 0.0]
            }),
        };
        queue.write_buffer(&self.post_buffer, 0, bytemuck::bytes_of(&uniform));
    }
}
