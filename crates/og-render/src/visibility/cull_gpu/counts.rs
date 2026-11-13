//! The counts GPU culling reads back: what each pass drew.

use super::*;

impl GpuCulling {
    /// Whether a readback holds a frame's counts not yet read.
    pub fn in_flight(&self) -> bool {
        self.readbacks.iter().any(|r| r.layout.is_some())
    }

    /// Takes every landed readback; the newest frame's counts are kept.
    pub fn poll_counts(&mut self) -> Option<GpuCounts> {
        for r in &mut self.readbacks {
            let ok = r.mapped.lock().unwrap_or_else(|e| e.into_inner()).take();
            let Some(ok) = ok else { continue };
            if let Some(layout) = r.layout.take()
                && ok
            {
                let counts = r
                    .buffer
                    .slice(..)
                    .get_mapped_range()
                    .ok()
                    .map(|data| layout.counts(bytemuck::cast_slice(&data)));
                if let Some(counts) = counts
                    && self.latest.is_none_or(|l| l.frame < counts.frame)
                {
                    self.latest = Some(counts);
                }
            }
            if ok {
                r.buffer.unmap();
            }
        }
        self.latest
    }

    /// Waits for `frame`'s readback (native): its counts and the ones its
    /// stats were given in their place. When the frame found no readback
    /// free and is the last one encoded (`args` still holds its counts),
    /// they are copied now.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn finish(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: u64,
    ) -> Option<(GpuCounts, GpuCounts)> {
        let find = |readbacks: &[Readback]| {
            readbacks
                .iter()
                .position(|r| r.layout.as_ref().is_some_and(|l| l.frame == frame))
        };
        let i = match find(&self.readbacks) {
            Some(i) => i,
            None => {
                let (layout, given) = self.last.clone().filter(|(l, _)| l.frame == frame)?;
                // Every readback in flight lands and is taken.
                device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
                self.poll_counts();
                let words = CountLayout::words(layout.kinds.len(), layout.batches);
                let r = self
                    .readbacks
                    .iter_mut()
                    .find(|r| r.layout.is_none() && r.buffer.size() >= words * 4)?;
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("cull counts"),
                });
                encoder.copy_buffer_to_buffer(&self.args, 0, &r.buffer, 0, words * 4);
                r.layout = Some(layout);
                r.applied = given;
                r.unmapped = true;
                queue.submit(Some(encoder.finish()));
                self.after_submit();
                find(&self.readbacks)?
            }
        };
        let applied = self.readbacks[i].applied;
        loop {
            if self.readbacks[i]
                .mapped
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_some()
            {
                break;
            }
            device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        }
        self.poll_counts();
        self.latest
            .filter(|l| l.frame == frame)
            .map(|exact| (exact, applied))
    }

    /// A frame whose commands were dropped unsubmitted (a covered surface,
    /// fork s70): the readback it would have copied into is free again.
    pub fn dropped(&mut self) {
        for r in &mut self.readbacks {
            if std::mem::take(&mut r.unmapped) {
                r.layout = None;
            }
        }
    }

    /// After the frame's submit: maps the readback it copied into.
    pub fn after_submit(&mut self) {
        for r in &mut self.readbacks {
            if std::mem::take(&mut r.unmapped) {
                let mapped = r.mapped.clone();
                r.buffer
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |result| {
                        *mapped.lock().unwrap_or_else(|e| e.into_inner()) = Some(result.is_ok());
                    });
            }
        }
    }

    /// The word of [`Self::args`] the crowd cull adds its counts from
    /// ([`CROWD_COUNT_WORDS`]), for the frame [`Self::encode`] encoded.
    pub fn crowd_counts_word(&self) -> u32 {
        self.last.as_ref().map_or(0, |(l, _)| {
            CountLayout::crowd_word(l.kinds.len(), l.batches) as u32
        })
    }

    /// The word of [`Self::args`] the meshlet path copies the camera's drawn
    /// cluster triangles to, for the frame [`Self::encode`] encoded.
    pub fn meshlet_triangles_word(&self) -> Option<u32> {
        self.last.as_ref().map(|(l, _)| {
            CountLayout::crowd_word(l.kinds.len(), l.batches) as u32 + MESHLET_TRIANGLES as u32
        })
    }

    /// The batches the meshlet path drew this frame (`batches` indices):
    /// their triangles are its clusters' at the levels it picked, which it
    /// counts itself, not their instances times their whole mesh's.
    pub fn counted_by_meshlets(&mut self, batches: &[u32]) {
        if let Some((l, _)) = self.last.as_mut() {
            for &b in batches {
                if let Some(t) = l.triangles.get_mut(b as usize) {
                    *t = 0;
                }
            }
        }
    }

    /// Copies the frame's counts to a free readback, when `encode` was
    /// asked for them and one is free: encoded after every cull that adds
    /// to them (the crowds').
    pub fn copy_counts(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if !std::mem::take(&mut self.copy) {
            return;
        }
        let Some((layout, given)) = self.last.clone() else {
            return;
        };
        let bytes = CountLayout::words(layout.kinds.len(), layout.batches) * 4;
        let Some(r) = self.readbacks.iter_mut().find(|r| {
            r.layout.is_none()
                && r.buffer.size() >= bytes
                && r.mapped.lock().map(|m| m.is_none()).unwrap_or(false)
        }) else {
            return;
        };
        encoder.copy_buffer_to_buffer(&self.args, 0, &r.buffer, 0, bytes);
        r.layout = Some(layout);
        r.unmapped = true;
        r.applied = given;
    }
}
