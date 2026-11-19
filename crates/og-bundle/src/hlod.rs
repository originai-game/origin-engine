//! A scene's HLOD tree (docs/design.md §10.6): the proxies the cook merges
//! its distant models into, and the models each stands in for.
//!
//! A scene with an `hlod { }` block cooks to its scene blob and, beside
//! it, `<scene>#hlod` (`og.Hlod`, [`Hlod`]) and one ordinary model per
//! node, `<scene>#hlod-<n>` ([`node_id`]), with its textures. og-cook's
//! `hlod.rs` builds them; og-runtime's `present/hlod.rs` draws them.

use glam::{Mat4, Vec3, Vec4};

/// The HLOD blob of a scene asset.
pub fn hlod_id(scene: &str) -> String {
    format!("{scene}#hlod")
}

/// The proxy model of node `n` of a scene's tree.
pub fn node_id(scene: &str, n: usize) -> String {
    format!("{scene}#hlod-{n}")
}

/// Whether a cooked output is part of a scene's HLOD (the tree, a proxy,
/// one of a proxy's textures or a streamed texture's `#mips`): they stay
/// in the first group, since a proxy draws while what it stands in for is
/// away.
pub fn is_hlod_output(id: &str) -> bool {
    let id = id.strip_suffix("#mips").unwrap_or(id);
    id.rsplit_once('#')
        .is_some_and(|(_, sub)| sub == "hlod" || sub.starts_with("hlod-"))
}

/// One node of the tree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HlodNode {
    pub parent: Option<usize>,
    /// The index after its last descendant.
    pub end: usize,
    /// Its members: a range of [`Hlod::members`].
    pub members: (usize, usize),
    pub level: u32,
    /// The sphere around its members, in the scene's space.
    pub center: Vec3,
    pub radius: f32,
    /// How far its proxy stands off its members, in metres.
    pub error: f32,
}

/// One member: a model entity of the scene (its index in the scene's
/// entities) and its matrix in the scene's space as the cook read it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HlodMember {
    pub entity: usize,
    pub matrix: Mat4,
}

/// A decoded HLOD tree: its nodes depth first.
#[derive(Clone, Debug, PartialEq)]
pub struct Hlod {
    pub size: f32,
    pub error: f32,
    pub nodes: Vec<HlodNode>,
    pub members: Vec<HlodMember>,
}

impl Hlod {
    /// The children of node `n` (`None`: the roots), in order.
    pub fn children(&self, n: Option<usize>) -> impl Iterator<Item = usize> + '_ {
        let (mut at, end) = match n {
            Some(n) => (n + 1, self.nodes[n].end),
            None => (0, self.nodes.len()),
        };
        std::iter::from_fn(move || {
            (at < end).then(|| {
                let child = at;
                at = self.nodes[child].end.max(child + 1);
                child
            })
        })
    }

    pub(crate) fn from_rows(
        meta: &crate::format::HlodMeta,
        nodes: &[crate::format::HlodNodeRow],
        members: &[crate::format::HlodMemberRow],
    ) -> Result<Hlod, String> {
        let mut out = Hlod {
            size: meta.size,
            error: meta.error,
            nodes: Vec::with_capacity(nodes.len()),
            members: Vec::with_capacity(members.len()),
        };
        for (i, n) in nodes.iter().enumerate() {
            let parent = (n.parent != crate::format::NONE).then_some(n.parent as usize);
            let end = n.end as usize;
            let (first, count) = (n.member_first as usize, n.member_count as usize);
            if end <= i
                || end > nodes.len()
                || parent.is_some_and(|p| p >= i || (nodes[p].end as usize) < end)
                || first + count > members.len()
            {
                return Err(format!("node {i} is out of the tree"));
            }
            out.nodes.push(HlodNode {
                parent,
                end,
                members: (first, first + count),
                level: n.level,
                center: Vec3::new(n.center_x, n.center_y, n.center_z),
                radius: n.radius,
                error: n.error,
            });
        }
        for m in members {
            out.members.push(HlodMember {
                entity: m.entity as usize,
                matrix: Mat4::from_cols(
                    Vec4::new(m.xx, m.yx, m.zx, 0.0),
                    Vec4::new(m.xy, m.yy, m.zy, 0.0),
                    Vec4::new(m.xz, m.yz, m.zz, 0.0),
                    Vec4::new(m.xt, m.yt, m.zt, 1.0),
                ),
            });
        }
        Ok(out)
    }
}
