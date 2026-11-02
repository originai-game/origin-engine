//! State blocks: world state a module keeps outside archetypes and
//! resources, because it is neither fixed-layout nor per entity (a physics
//! engine's contact manifolds, solver warm starts and broad-phase tree, say).
//! A block is part of the world: [`World::snapshot`](crate::World::snapshot)
//! saves it, [`World::world_hash`](crate::World::world_hash) covers it, and a
//! restore brings it back. Like systems, blocks are code: restore into a
//! world that registered the same blocks.

use og_core::Result;
use std::any::Any;

/// One module's state, saved and restored with the world.
///
/// `save` must be a function of the state: equal state, equal bytes (the
/// hash compares them). A restore first calls `decode` on every block and
/// applies nothing unless the whole snapshot is valid, then `apply` on each;
/// `apply(None)` means the snapshot had no bytes for this block (an older
/// format, or a world that did not have it), and the block starts empty.
pub trait StateBlock: Send {
    /// Appends the block's state.
    fn save(&self, out: &mut Vec<u8>);
    /// Decodes saved bytes without changing the block.
    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>>;
    /// Takes the decoded state (`Some`, from [`Self::decode`]) or resets.
    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>);
    /// Moves every place in the world the block keeps by `shift.d`: the
    /// world's zero moved ([`crate::World::shift`], R12's og.WorldOrigin).
    /// A value in an entity's Transform frame moves only on a root
    /// ([`crate::Shift::of`]). A block with no world positions does
    /// nothing, and says so: there is no default, so a new block decides.
    fn shift(&mut self, shift: &crate::Shift);
}

pub(crate) struct Block {
    pub name: String,
    pub block: Box<dyn StateBlock>,
}

/// The optional trailer a block appends after its first form's bytes, so a
/// block with no version of its own (`og.physics`, `og.nav`, `og.ai`) can
/// grow without a format change: `OGXT` (a magic `u32`), a format version
/// byte ([`TRAILER_VERSION`]), then sections of a `u16` tag, a `u32` length
/// and that many bytes, all little-endian, tags strictly ascending. A block
/// writes it only when a section is non-empty, so a state using no
/// extension saves (and hashes) byte for byte as before; a decoder skips a
/// tag it does not know, so a later extension adds a tag, not a format.
pub mod trailer {
    use og_core::{Error, ErrorKind, Result};

    /// The trailer's first four bytes.
    pub const TRAILER_MAGIC: [u8; 4] = *b"OGXT";
    /// The layout of the section headers that follow the magic.
    pub const TRAILER_VERSION: u8 = 1;

    /// Appends the trailer holding `sections` (`(tag, bytes)`, tags strictly
    /// ascending) to `out`; nothing when `sections` is empty.
    pub fn write(out: &mut Vec<u8>, sections: &[(u16, Vec<u8>)]) {
        if sections.is_empty() {
            return;
        }
        debug_assert!(sections.windows(2).all(|w| w[0].0 < w[1].0));
        out.extend_from_slice(&TRAILER_MAGIC);
        out.push(TRAILER_VERSION);
        for (tag, bytes) in sections {
            out.extend_from_slice(&tag.to_le_bytes());
            out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            out.extend_from_slice(bytes);
        }
    }

    /// The sections of the trailer `rest` (the bytes after a block's first
    /// form; empty is no trailer), in tag order. `block` names the block in
    /// errors.
    pub fn read<'a>(block: &str, mut rest: &'a [u8]) -> Result<Vec<(u16, &'a [u8])>> {
        let bad = |why: &str| {
            Error::new(
                ErrorKind::Parse,
                format!("{block}'s extension trailer {why}"),
            )
        };
        let mut out: Vec<(u16, &[u8])> = Vec::new();
        if rest.is_empty() {
            return Ok(out);
        }
        rest = rest
            .strip_prefix(&TRAILER_MAGIC)
            .ok_or_else(|| bad("does not start with OGXT"))?;
        let (&version, body) = rest
            .split_first()
            .ok_or_else(|| bad("ends before its version"))?;
        if version != TRAILER_VERSION {
            return Err(bad(&format!(
                "has version {version}, not {TRAILER_VERSION}"
            )));
        }
        if body.is_empty() {
            return Err(bad("has no sections"));
        }
        rest = body;
        while !rest.is_empty() {
            if rest.len() < 6 {
                return Err(bad("ends inside a section header"));
            }
            let tag = u16::from_le_bytes([rest[0], rest[1]]);
            let len = u32::from_le_bytes([rest[2], rest[3], rest[4], rest[5]]) as usize;
            rest = &rest[6..];
            if rest.len() < len {
                return Err(bad("ends inside a section"));
            }
            if out.last().is_some_and(|&(last, _)| last >= tag) {
                return Err(bad("repeats a tag or has them out of order"));
            }
            out.push((tag, &rest[..len]));
            rest = &rest[len..];
        }
        Ok(out)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn round_trip_skips_nothing_and_writes_nothing_empty() {
            let mut out = vec![9];
            write(&mut out, &[]);
            assert_eq!(out, [9]);
            write(&mut out, &[(1, vec![1, 2]), (7, vec![])]);
            assert_eq!(
                &out[1..],
                [
                    b'O', b'G', b'X', b'T', 1, 1, 0, 2, 0, 0, 0, 1, 2, 7, 0, 0, 0, 0, 0
                ]
            );
            let read = read("og.test", &out[1..]).unwrap();
            assert_eq!(read, [(1, &[1u8, 2][..]), (7, &[][..])]);
            assert!(super::read("og.test", &[]).unwrap().is_empty());
        }

        #[test]
        fn rejects_truncation_bad_magic_version_and_order() {
            let mut good = Vec::new();
            write(&mut good, &[(2, vec![5, 6, 7])]);
            for cut in 1..good.len() {
                assert!(read("og.test", &good[..cut]).is_err(), "cut at {cut}");
            }
            let mut v2 = good.clone();
            v2[4] = 2;
            assert!(read("og.test", &v2).is_err());
            assert!(read("og.test", b"OGX\x01").is_err());
            let mut twice = good.clone();
            twice.extend_from_slice(&good[5..]);
            assert!(read("og.test", &twice).is_err());
        }
    }
}
