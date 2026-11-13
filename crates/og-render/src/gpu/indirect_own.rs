//! Indirect draws' args each in a buffer of their own, for WebKit
//! ([`crate::RenderInfo::indirect_per_draw`]).
//!
//! WebKit's Metal backend (Safari 27.2, its 7625 branches) clamps every
//! indirect draw whose pipeline has vertex buffers into one slot owned by
//! the INDIRECT buffer (`Buffer::m_indirectIndexedBuffer`,
//! `RenderPassEncoder::clampIndirectIndexBufferToValidValues`), then
//! fetches the draw's args from that slot, with nothing ordering one
//! draw's fetch before the next draw's clamp writes the same slot. Draws
//! in one pass that share an args buffer then read each other's counts:
//! spheres-20k's ground plane drew with its spheres' 1,152 indices x 518
//! instances, torn triangles and a 16-31 s frame
//! (the former test sample Fixed upstream by WebKit
//! 6dca6330db (bug 321876: per-draw scratch). A cull that writes its args
//! into one buffer copies each draw's to a buffer here after the cull,
//! and its draws read those: a slot per draw.

/// One small INDIRECT buffer per draw slot, grown as needed and reused
/// frame to frame.
#[derive(Default)]
pub(crate) struct OwnArgs {
    buffers: Vec<wgpu::Buffer>,
}

/// The largest args record: `DrawIndexedIndirect`'s five words.
const RECORD: u64 = 20;

impl OwnArgs {
    /// Makes slots `0..n` (those already made are kept).
    pub fn ensure(&mut self, device: &wgpu::Device, n: usize) {
        while self.buffers.len() < n {
            self.buffers
                .push(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("indirect draw args"),
                    size: RECORD,
                    usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
        }
    }

    /// Copies the `bytes` (16 or 20) at `offset` in `src` to slot `slot`
    /// ([`Self::ensure`]d), encoded after what writes them.
    pub fn copy(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        src: &wgpu::Buffer,
        offset: u64,
        slot: usize,
        bytes: u64,
    ) {
        debug_assert!(bytes <= RECORD);
        encoder.copy_buffer_to_buffer(src, offset, &self.buffers[slot], 0, bytes);
    }

    /// The buffer of slot `slot`, its args at offset 0.
    pub fn get(&self, slot: usize) -> &wgpu::Buffer {
        &self.buffers[slot]
    }
}
