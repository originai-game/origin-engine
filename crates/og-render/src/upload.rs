//! Instance uploads that write only what changed since the last frame.
//!
//! The instance buffers (the CPU path's vertex buffer and GPU culling's
//! input) are rewritten every frame from the sorted transforms. A scene
//! whose objects stand still sends the same bytes again; this keeps a copy
//! of what the buffer holds and writes only the runs of [`BLOCK`]-byte
//! blocks that differ, on both tiers (owed in the README's "What phase 2
//! owes": dirty tracking).

/// The comparison and write granularity: 64 instances of 64 bytes. Smaller
/// blocks write fewer bytes for scattered changes but issue more writes.
pub(crate) const BLOCK: usize = 4096;

/// A buffer's contents as last written, for writing only the changes.
#[derive(Default)]
pub(crate) struct DirtyUpload {
    held: Vec<u8>,
    /// Whether `held` describes the buffer (false until the first write and
    /// after the buffer is replaced).
    valid: bool,
}

impl DirtyUpload {
    /// The buffer was replaced: the next write is whole.
    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    /// Writes `bytes` at the start of `buffer`: whole when the buffer is new
    /// to it or the length changed, else only the differing block runs.
    /// Returns the bytes written. `bytes.len()` is a multiple of 4 (wgpu's
    /// copy alignment), and so is every block run.
    pub fn write(&mut self, queue: &wgpu::Queue, buffer: &wgpu::Buffer, bytes: &[u8]) -> u64 {
        if bytes.is_empty() {
            self.held.clear();
            return 0;
        }
        if !self.valid || self.held.len() != bytes.len() {
            queue.write_buffer(buffer, 0, bytes);
            self.held.clear();
            self.held.extend_from_slice(bytes);
            self.valid = true;
            return bytes.len() as u64;
        }
        let runs = changed_runs(&self.held, bytes);
        let mut written = 0;
        for (start, end) in runs {
            queue.write_buffer(buffer, start as u64, &bytes[start..end]);
            self.held[start..end].copy_from_slice(&bytes[start..end]);
            written += (end - start) as u64;
        }
        written
    }
}

/// The byte ranges of `new` whose [`BLOCK`]s differ from `old` (same
/// length), adjacent blocks merged.
pub(crate) fn changed_runs(old: &[u8], new: &[u8]) -> Vec<(usize, usize)> {
    debug_assert_eq!(old.len(), new.len());
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for (i, (a, b)) in old.chunks(BLOCK).zip(new.chunks(BLOCK)).enumerate() {
        if a == b {
            continue;
        }
        let (start, end) = (i * BLOCK, i * BLOCK + b.len());
        match runs.last_mut() {
            Some(r) if r.1 == start => r.1 = end,
            _ => runs.push((start, end)),
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_changed_blocks_are_runs_and_neighbours_merge() {
        let old = vec![0u8; BLOCK * 5 + 64];
        let mut new = old.clone();
        assert!(changed_runs(&old, &new).is_empty());
        new[10] = 1;
        new[BLOCK + 3] = 1;
        new[BLOCK * 3 + 7] = 1;
        new[BLOCK * 5 + 60] = 1;
        assert_eq!(
            changed_runs(&old, &new),
            vec![
                (0, 2 * BLOCK),
                (3 * BLOCK, 4 * BLOCK),
                (5 * BLOCK, 5 * BLOCK + 64)
            ]
        );
    }
}
