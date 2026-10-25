//! Instance uploads that write only what a draw reads and what changed
//! since the last frame.
//!
//! The instance buffers (the CPU path's vertex buffer and GPU culling's
//! input) are rewritten every frame from the sorted transforms. A scene
//! whose objects stand still sends the same bytes again; this keeps a copy
//! of what the buffer holds and writes only the runs of [`BLOCK`]-byte
//! blocks that differ, on both tiers (owed in the README's "What phase 2
//! owes": dirty tracking). A frame also names the ranges its draws read:
//! the CPU path draws a batch that lost more than a tenth of its instances
//! to culling from copies appended after the sorted instances, so the
//! sorted records no list draws whole are never read and are not written
//! (a field of 20,000 spheres, a fifth of them off screen, wrote its
//! 20,000 sorted records and the 16,000 copies drawn: 116 bytes an
//! instance against the 64 of one record).

use std::ops::Range;

/// The comparison and write granularity: 4 KiB, 64 instances of 64
/// bytes (`instances::GpuTransform`). Smaller blocks write fewer bytes for
/// scattered changes but issue more writes.
pub(crate) const BLOCK: usize = 4096;

/// A buffer's contents as last written, for writing only the changes.
#[derive(Default)]
pub(crate) struct DirtyUpload {
    held: Vec<u8>,
    /// Per [`BLOCK`] of `held`, whether it is what the buffer holds (false
    /// until the block is written, and for every block after the buffer is
    /// replaced or the length changed).
    known: Vec<bool>,
}

impl DirtyUpload {
    /// The buffer was replaced: every block is written again.
    pub fn invalidate(&mut self) {
        self.known.fill(false);
    }

    /// Writes the blocks of `bytes` (the buffer's contents from its start)
    /// that hold a byte of a `drawn` range and differ from what the buffer
    /// holds; the bytes of other blocks are left as they were (no draw
    /// reads them). Returns the bytes written. `bytes.len()` is a multiple
    /// of 4 (wgpu's copy alignment), and so is every block run. `drawn` is
    /// sorted and its ranges do not overlap.
    pub fn write(
        &mut self,
        queue: &wgpu::Queue,
        buffer: &wgpu::Buffer,
        bytes: &[u8],
        drawn: &[Range<usize>],
    ) -> u64 {
        let runs = self.runs(bytes, drawn);
        let mut written = 0;
        for (start, end) in runs {
            queue.write_buffer(buffer, start as u64, &bytes[start..end]);
            written += (end - start) as u64;
        }
        written
    }

    /// The block runs [`DirtyUpload::write`] writes, taken as written:
    /// `held` and `known` describe the buffer after them.
    fn runs(&mut self, bytes: &[u8], drawn: &[Range<usize>]) -> Vec<(usize, usize)> {
        if self.held.len() != bytes.len() {
            self.held.clear();
            self.held.resize(bytes.len(), 0);
            self.known.clear();
            self.known.resize(bytes.len().div_ceil(BLOCK), false);
        }
        let mut runs: Vec<(usize, usize)> = Vec::new();
        // Two ranges may end and start in one block: it is seen once.
        let mut next = 0;
        for r in drawn.iter().filter(|r| r.start < r.end) {
            let end = r.end.min(bytes.len());
            for i in (r.start / BLOCK).max(next)..end.div_ceil(BLOCK) {
                let (start, end) = (i * BLOCK, ((i + 1) * BLOCK).min(bytes.len()));
                next = i + 1;
                if self.known[i] && self.held[start..end] == bytes[start..end] {
                    continue;
                }
                self.held[start..end].copy_from_slice(&bytes[start..end]);
                self.known[i] = true;
                match runs.last_mut() {
                    Some(r) if r.1 == start => r.1 = end,
                    _ => runs.push((start, end)),
                }
            }
        }
        runs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_changed_blocks_are_runs_and_neighbours_merge() {
        let old = vec![0u8; BLOCK * 5 + 64];
        let whole = 0..old.len();
        let all = std::slice::from_ref(&whole);
        let mut up = DirtyUpload::default();
        assert_eq!(up.runs(&old, all), vec![(0, old.len())]);
        let mut new = old.clone();
        assert!(up.runs(&new, all).is_empty());
        new[10] = 1;
        new[BLOCK + 3] = 1;
        new[BLOCK * 3 + 7] = 1;
        new[BLOCK * 5 + 60] = 1;
        assert_eq!(
            up.runs(&new, all),
            vec![
                (0, 2 * BLOCK),
                (3 * BLOCK, 4 * BLOCK),
                (5 * BLOCK, 5 * BLOCK + 64)
            ]
        );
        assert!(up.runs(&new, all).is_empty());
        // Replaced: whole again.
        up.invalidate();
        assert_eq!(up.runs(&new, all), vec![(0, new.len())]);
    }

    /// Only the blocks a draw reads are written; a block no draw read
    /// before is written once a draw reads it, whatever `held` says of it
    /// (the buffer never received those bytes).
    #[test]
    fn blocks_no_draw_reads_are_not_written() {
        let mut bytes = vec![0u8; BLOCK * 6];
        let mut up = DirtyUpload::default();
        // Blocks 1, 2 (two ranges meeting in it) and 4 to 5.
        let drawn = [
            BLOCK + 8..2 * BLOCK + 100,
            2 * BLOCK + 200..3 * BLOCK,
            4 * BLOCK..6 * BLOCK,
        ];
        assert_eq!(
            up.runs(&bytes, &drawn),
            vec![(BLOCK, 3 * BLOCK), (4 * BLOCK, 6 * BLOCK)]
        );
        // Every byte changes: the same blocks.
        bytes.fill(7);
        assert_eq!(
            up.runs(&bytes, &drawn),
            vec![(BLOCK, 3 * BLOCK), (4 * BLOCK, 6 * BLOCK)]
        );
        // Everything drawn now: blocks 0 and 3 were never written, the
        // rest is as written.
        assert_eq!(
            up.runs(&bytes, std::slice::from_ref(&(0..BLOCK * 6))),
            vec![(0, BLOCK), (3 * BLOCK, 4 * BLOCK)]
        );
        // Another length: every drawn block again.
        bytes.push(7);
        assert_eq!(
            up.runs(&bytes, std::slice::from_ref(&(5 * BLOCK..bytes.len()))),
            vec![(5 * BLOCK, bytes.len())]
        );
    }
}
