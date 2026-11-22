//! The node tree: each node's local transform, its parent and its world transform.

use super::*;

impl Doc<'_> {
    /// The local transform of node `i`, in glTF space.
    pub(crate) fn local(&self, n: &Value) -> Affine {
        if let Some(m) = n.get("matrix").nums().filter(|m| m.len() == 16) {
            return Affine {
                x: DVec3::new(m[0], m[1], m[2]),
                y: DVec3::new(m[4], m[5], m[6]),
                z: DVec3::new(m[8], m[9], m[10]),
                t: DVec3::new(m[12], m[13], m[14]),
            };
        }
        let t = n
            .get("translation")
            .nums()
            .filter(|v| v.len() == 3)
            .map_or(DVec3::ZERO, |v| DVec3::new(v[0], v[1], v[2]));
        let s = n
            .get("scale")
            .nums()
            .filter(|v| v.len() == 3)
            .map_or(DVec3::ONE, |v| DVec3::new(v[0], v[1], v[2]));
        let q = n
            .get("rotation")
            .nums()
            .filter(|v| v.len() == 4)
            .map_or([0.0, 0.0, 0.0, 1.0], |v| [v[0], v[1], v[2], v[3]]);
        trs(t, q, s)
    }

    /// Each node's parent, by index.
    pub(crate) fn parents(&self) -> Vec<Option<usize>> {
        let nodes = self.json.get("nodes").items();
        let mut parent = vec![None; nodes.len()];
        for (i, n) in nodes.iter().enumerate() {
            for c in n.get("children").items() {
                if let Some(c) = c.index().filter(|&c| c < nodes.len()) {
                    parent[c] = Some(i);
                }
            }
        }
        parent
    }

    /// A node's world transform in glTF space.
    pub(crate) fn world(&self, mut i: usize, parents: &[Option<usize>]) -> Affine {
        let mut m = Affine::IDENTITY;
        for _ in 0..=parents.len() {
            m = self.local(self.json.get("nodes").at(i)).then(&m);
            match parents.get(i).copied().flatten() {
                Some(p) => i = p,
                None => break,
            }
        }
        m
    }
}
