//! Visibility channels carried through the clip rewrites: a mirrored clip
//! and a retargeted one keep the source's visibility channels on the node
//! that plays the part (`ChannelPath::Visibility`; how poses blend the
//! flags is `pose.rs`'s module docs).

use crate::asset::{Channel, ChannelPath, ClipAsset, NONE, SkeletonAsset};
use crate::retarget::HumanoidMap;

/// The visibility channels of `clip` (bound to `skeleton` by `binding`)
/// for its mirror: each moves to its node's counterpart (`pairs`,
/// `mirror_pairs`), as scales are copied; a sided node without a
/// counterpart loses its channel.
pub(crate) fn mirrored(
    skeleton: &SkeletonAsset,
    clip: &ClipAsset,
    binding: &[u32],
    pairs: &[u32],
) -> Vec<Channel> {
    clip.channels
        .iter()
        .zip(binding)
        .filter(|(c, b)| c.path == ChannelPath::Visibility && **b != NONE)
        .filter_map(|(c, &b)| {
            let k = pairs[b as usize];
            (k != NONE).then(|| Channel {
                node: k,
                node_name: skeleton.nodes[k as usize].name.clone(),
                ..c.clone()
            })
        })
        .collect()
}

/// The visibility channels of `clip` (made for `source`) rewritten for
/// `target`: a channel on a mapped humanoid bone goes to the target's
/// node of that bone, any other to the target node of the same name
/// (a prop, a mesh swapped in); a node the target lacks loses its
/// channel, and the first channel onto a target node wins.
pub(crate) fn retargeted(
    source: &SkeletonAsset,
    source_map: &HumanoidMap,
    target: &SkeletonAsset,
    target_map: &HumanoidMap,
    clip: &ClipAsset,
    binding: &[u32],
) -> Vec<Channel> {
    let mut out: Vec<Channel> = Vec::new();
    for (c, &b) in clip.channels.iter().zip(binding) {
        if c.path != ChannelPath::Visibility || b == NONE {
            continue;
        }
        let bone = source_map.nodes.iter().position(|&n| n == b);
        let node = match bone.map(|bone| target_map.nodes[bone]) {
            Some(n) if n != NONE => Some(n),
            _ => target.node(&source.nodes[b as usize].name),
        };
        if let Some(n) = node
            && !out.iter().any(|o| o.node == n)
        {
            out.push(Channel {
                node: n,
                node_name: target.nodes[n as usize].name.clone(),
                ..c.clone()
            });
        }
    }
    out
}
