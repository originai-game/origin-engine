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
    /// What the buffer holds from its start, as far as any frame wrote it:
    /// a shorter frame leaves the bytes past its end as they were, and a
    /// later longer one compares against them.
    held: Vec<u8>,
    /// Per [`BLOCK`] of `held`, whether it is what the buffer holds (false
    /// until the block is written, and for every block after the buffer is
    /// replaced). Neither shrinks with a shorter frame: a shadow layer the
    /// static cache keeps (shadow_cache.rs) appends no copies, so a still
    /// scene's records shorten without changing, and when the layer is
    /// drawn again its copies come back at the same offsets, which the
    /// buffer still holds.
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
        let old = self.held.len();
        if bytes.len() > old {
            self.held.resize(bytes.len(), 0);
            self.known.resize(bytes.len().div_ceil(BLOCK), false);
            // A block the old length ended in holds bytes past it that
            // `held` never saw.
            if !old.is_multiple_of(BLOCK) {
                self.known[old / BLOCK] = false;
            }
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
        // Longer: the buffer still holds block 5; only the new one.
        bytes.push(7);
        assert_eq!(
            up.runs(&bytes, std::slice::from_ref(&(5 * BLOCK..bytes.len()))),
            vec![(6 * BLOCK, bytes.len())]
        );
        // Longer again, from a partial block: the block it ended in is
        // written whole (the buffer never held its new bytes).
        bytes.extend_from_slice(&[7, 7, 7, 7]);
        assert_eq!(
            up.runs(&bytes, std::slice::from_ref(&(5 * BLOCK..bytes.len()))),
            vec![(6 * BLOCK, bytes.len())]
        );
    }

    /// A shorter frame of the same records (a still scene whose shadow
    /// layers the static cache keeps, so their lists append no copies)
    /// writes nothing; a changed record in it is still written.
    #[test]
    fn a_shorter_frame_of_the_same_records_writes_nothing() {
        let mut bytes: Vec<u8> = (0..BLOCK * 5 + 64).map(|i| (i % 251) as u8).collect();
        let mut up = DirtyUpload::default();
        assert_eq!(
            up.runs(&bytes, std::slice::from_ref(&(0..bytes.len()))),
            vec![(0, bytes.len())]
        );
        bytes.truncate(BLOCK * 3 + 100);
        assert!(
            up.runs(&bytes, std::slice::from_ref(&(0..bytes.len())))
                .is_empty()
        );
        bytes[BLOCK * 3 + 10] ^= 1;
        assert_eq!(
            up.runs(&bytes, std::slice::from_ref(&(0..bytes.len()))),
            vec![(BLOCK * 3, bytes.len())]
        );
        // The layer drawn again: its records return at the offsets the
        // buffer still holds them (block 3 as last written, its tail as
        // the first frame left it); only the changed block 4 is written.
        let mut again: Vec<u8> = (0..BLOCK * 5 + 64).map(|i| (i % 251) as u8).collect();
        again[BLOCK * 3 + 10] ^= 1;
        again[BLOCK * 4 + 5] ^= 1;
        assert_eq!(
            up.runs(&again, std::slice::from_ref(&(0..again.len()))),
            vec![(BLOCK * 4, BLOCK * 5)]
        );
    }
}
