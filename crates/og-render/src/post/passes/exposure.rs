//! The post chain's exposure: its rows, its adaptation and its checkpoint.

use super::*;

impl PostPasses {
    /// The post uniform's exposure row (`Post.exposure` in post.wgsl: the
    /// manual EV, the mode, the metered range) and the compensation, which
    /// the flare passes read as well.
    pub fn exposure_rows(exposure: Exposure) -> ([f32; 4], f32) {
        let adapted = exposure.adaptation().is_some();
        match exposure {
            Exposure::Manual { ev } => ([ev, 0.0, 0.0, 0.0], 0.0),
            Exposure::Auto {
                compensation,
                min_ev,
                max_ev,
                ..
            } => (
                [
                    0.0,
                    if adapted { 2.0 } else { 1.0 },
                    min_ev,
                    max_ev.max(min_ev),
                ],
                compensation,
            ),
        }
    }

    /// Advances the adapted exposure's clock to this frame's tick and writes
    /// the adaptation uniform. `exposure` adapts ([`Exposure::adaptation`]);
    /// `slot` is the texture this frame writes. Returns nothing: the frame's
    /// adapt pass does the rest. A frame that does not adapt calls
    /// [`PostPasses::forget_exposure`] instead.
    pub fn advance_exposure(
        &mut self,
        queue: &wgpu::Queue,
        exposure: Exposure,
        tick: u64,
        tick_rate: f32,
        slot: usize,
    ) {
        let Exposure::Auto {
            compensation,
            min_ev,
            max_ev,
            ..
        } = exposure
        else {
            return;
        };
        let (up, down) = exposure.adaptation().unwrap_or((0.0, 0.0));
        // The previous adapted exposure must be in the texture this frame
        // reads; a frame whose tick went back starts again. A frame of tick
        // 0, before the first tick, meters at once and is never adapted
        // from: one host draws it (the web shell's first frame, behind the
        // play gate) and another does not (`og run` draws from tick 1), so
        // the adaptation starts at the first ticked frame's metered
        // exposure on every host (web findings #5: Lantern Vale's first two
        // seconds were brighter on the web, which adapted from its tick 0
        // frame).
        let (dt, snap) = match self.exposure.last {
            _ if tick == 0 => (0.0, true),
            Some((t, written, both)) if tick >= t && (written != slot || both) => {
                ((tick - t) as f32 / tick_rate.max(1e-3), false)
            }
            _ => (0.0, true),
        };
        self.exposure.last = (tick > 0).then_some((tick, slot, false));
        let u = AdaptUniform {
            speed: [dt, up, down, snap as u32 as f32],
            meter: [compensation, min_ev, max_ev.max(min_ev), 0.0],
        };
        queue.write_buffer(&self.adapt_buffer, 0, bytemuck::bytes_of(&u));
    }

    /// A frame without adaptation: the next adapted frame starts afresh.
    pub fn forget_exposure(&mut self) {
        self.exposure.last = None;
    }

    fn texel_copy(
        encoder: &mut crate::Encoder,
        from: &wgpu::Texture,
        from_x: u32,
        to: &wgpu::Texture,
        to_x: u32,
    ) {
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: from,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: from_x,
                    y: 0,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: to,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: to_x,
                    y: 0,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Keeps the adapted exposure as it is now; the answer names it for
    /// [`PostPasses::restore_exposure`]. Recorded into `encoder`, so it
    /// copies what the frames submitted before it wrote.
    pub fn save_exposure(&mut self, encoder: &mut crate::Encoder) -> u64 {
        let id = self.exposure.next_id;
        self.exposure.next_id += 1;
        let at = (id % EXPOSURE_SAVES as u64) as u32;
        if let Some((_, written, _)) = self.exposure.last {
            Self::texel_copy(
                encoder,
                &self.exposure.textures[written],
                0,
                &self.exposure.saves,
                at,
            );
        }
        self.exposure.ring[at as usize] = Some(ExposureSave {
            id,
            last_tick: self.exposure.last.map(|(t, ..)| t),
        });
        id
    }

    /// Brings back the adapted exposure [`PostPasses::save_exposure`] kept
    /// as `id`; one no longer kept (or 0) starts the next adapted frame
    /// afresh. Recorded into `encoder`.
    pub fn restore_exposure(&mut self, encoder: &mut crate::Encoder, id: u64) {
        let at = (id % EXPOSURE_SAVES as u64) as usize;
        let kept = self.exposure.ring[at].filter(|s| s.id == id && id != 0);
        self.exposure.last = match kept.and_then(|s| s.last_tick) {
            Some(tick) => {
                for t in &self.exposure.textures {
                    Self::texel_copy(encoder, &self.exposure.saves, at as u32, t, 0);
                }
                // Both textures hold it: whichever the next frame reads.
                Some((tick, 0, true))
            }
            None => None,
        };
    }
}
