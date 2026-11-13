//! The prefractured asset ([`FractureAsset`]): its chunks, a hierarchy of
//! two levels and the bonds of its support graph (Blast's model,
//! `docs/design.md` 2.7). The cook writes it (og-cook `cook/fracture.rs`,
//! the pieces from `og_geom::fracture`); og-physics' `destruction` breaks
//! it.

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// No chunk (a root's parent, a leaf's children).
pub const NONE: u32 = u32::MAX;

/// The format version of [`FractureAsset::to_bytes`].
pub const ASSET_VERSION: u32 = 2;
/// [`FractureAsset::mortar`] when there is none.
pub const NO_MATERIAL: u32 = u32::MAX;

/// One chunk of a prefractured asset, in the model's space.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    /// Its centre of mass.
    pub centroid: Vec3,
    pub volume: f32,
    /// Its vertices relative to its centroid (its collider is their hull).
    pub hull: Vec<Vec3>,
    /// The glTF node that draws it, at its centroid.
    pub node: u32,
    /// The chunk it is part of ([`NONE`] for a support chunk).
    pub parent: u32,
    /// Its children (`first`, `count`; count 0 for a leaf).
    pub children: (u32, u32),
    /// The area by which it rests on what holds the asset up (its anchor
    /// faces, `anchor` in the source's `fracture` block); 0 when none.
    pub anchor_area: f32,
    pub anchor_centroid: Vec3,
}

/// The surface two support chunks share (Blast's bond).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Bond {
    pub a: u32,
    pub b: u32,
    pub centroid: Vec3,
    /// From a towards b.
    pub normal: Vec3,
    pub area: f32,
}

/// What the cook makes of a source with a `fracture` block: chunks (the
/// support chunks first, `0..support`, then their children, each parent's
/// contiguous), and the bonds between support chunks, sorted by (a, b).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FractureAsset {
    pub chunks: Vec<Chunk>,
    pub bonds: Vec<Bond>,
    pub support: u32,
    /// The intact model's bounds.
    pub min: Vec3,
    pub max: Vec3,
    /// The glTF material of the cut faces.
    pub interior: u32,
    /// The glTF material of the mortar between bricks (`fracture.mortar`),
    /// which no face draws: what crumbles from a broken joint is drawn
    /// with it. [`NO_MATERIAL`] for none.
    pub mortar: u32,
}

impl FractureAsset {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = ASSET_VERSION.to_le_bytes().to_vec();
        out.extend(bincode::serialize(self).expect("a fracture asset serialises"));
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<FractureAsset, String> {
        let (v, rest) = bytes
            .split_first_chunk::<4>()
            .ok_or("a fracture asset is cut short")?;
        let v = u32::from_le_bytes(*v);
        if v != ASSET_VERSION {
            return Err(format!(
                "a fracture asset of version {v}, not {ASSET_VERSION}: cook it again"
            ));
        }
        let a: FractureAsset = bincode::deserialize(rest).map_err(|e| e.to_string())?;
        a.check()?;
        Ok(a)
    }

    /// Every index in range and the support chunks first.
    pub fn check(&self) -> Result<(), String> {
        let n = self.chunks.len() as u32;
        if self.support == 0 || self.support > n {
            return Err(format!("{} support chunks of {n}", self.support));
        }
        for (i, c) in self.chunks.iter().enumerate() {
            let support = (i as u32) < self.support;
            if support != (c.parent == NONE) {
                return Err(format!("chunk {i}: support chunks come first"));
            }
            if c.children.1 > 0 && c.children.0.saturating_add(c.children.1) > n {
                return Err(format!("chunk {i}: its children are out of range"));
            }
        }
        for b in &self.bonds {
            if b.a >= b.b || b.b >= self.support {
                return Err(format!("bond {}-{}: not between support chunks", b.a, b.b));
            }
        }
        Ok(())
    }

    /// The bonds of each support chunk (indices into `bonds`).
    pub fn bonds_of(&self) -> Vec<Vec<u32>> {
        let mut out = vec![Vec::new(); self.support as usize];
        for (k, b) in self.bonds.iter().enumerate() {
            out[b.a as usize].push(k as u32);
            out[b.b as usize].push(k as u32);
        }
        out
    }
}
