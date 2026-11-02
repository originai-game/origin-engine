//! Meshes for the parts that bend: a grid over the part, the cells that
//! hold any of its pixels kept, each vertex weighted along the part's
//! bones. Across the joint with its parent a vertex goes from the
//! parent's bone (a band's width into the overlap) through the joint
//! bone (at the joint; it turns half-way between the two) to the part's
//! own (a band's width out), and across its tip the same way into the
//! next part; along a chain (a cape, a tail) it is shared between the
//! chain's bones. So an elbow, a knee or a cape bends, and the pixels at
//! a joint turn rigidly at half its angle instead of being pinched.

use super::parts::Part;
use og_anim::{Affine2, AttachmentKind, Influence, MeshVertex2d};

type P = [f32; 2];

/// How a part bends: its bones (a chain's in order) laid along `dir` from
/// `origin`, each `step` long, and the bones it shares weight with.
pub struct Bend {
    pub bones: Vec<usize>,
    pub origin: P,
    /// A unit vector along the bones, rig space.
    pub dir: P,
    pub step: f32,
    /// The joint bone at the start and the parent part's bone.
    pub start: Option<(usize, usize)>,
    /// The joint bone at the tip and the next part's bone.
    pub end: Option<(usize, usize)>,
}

/// The weights of a point `d` pixels along the bones from the origin.
fn weights(bend: &Bend, d: f32, band: f32) -> Vec<(usize, f32)> {
    let total = bend.step * bend.bones.len() as f32;
    let mut out = Vec::with_capacity(4);
    // Across a joint, from -1 (all the far side) through 0 (all the joint
    // bone) to 1 (all the near side); `own` is what stays with the part.
    let mut own = 1.0;
    if let Some((joint, parent)) = bend.start {
        let t = (d / band).clamp(-1.0, 1.0);
        if t < 0.0 {
            out.push((parent, -t));
            out.push((joint, 1.0 + t));
            own = 0.0;
        } else {
            out.push((joint, 1.0 - t));
            own = t;
        }
    }
    if let Some((joint, child)) = bend.end {
        let t = ((d - total) / band).clamp(-1.0, 1.0);
        if t > 0.0 {
            out.push((joint, own * (1.0 - t)));
            out.push((child, own * t));
            own = 0.0;
        } else {
            out.push((joint, own * (1.0 + t)));
            own *= -t;
        }
    }
    // Along a chain: each bone's share peaks at its middle and falls to
    // nothing at its neighbours' middles; past the ends, the end bones.
    let k = bend.bones.len();
    let at = (d / bend.step - 0.5).clamp(0.0, (k - 1) as f32);
    let i = (at.floor() as usize).min(k - 1);
    let f = at - i as f32;
    out.push((bend.bones[i], own * (1.0 - f)));
    if i + 1 < k {
        out.push((bend.bones[i + 1], own * f));
    }
    out.retain(|w| w.1 > 1e-3);
    let sum: f32 = out.iter().map(|w| w.1).sum();
    out.iter_mut().for_each(|w| w.1 /= sum);
    out
}

/// The mesh of `part`, drawn from its atlas rectangle `rect`; `to_rig`
/// maps canvas pixels into the rig and `world` holds the bones' setup
/// transforms.
pub fn mesh(
    part: &Part,
    rect: [u32; 4],
    bend: &Bend,
    to_rig: &dyn Fn(P) -> P,
    world: &[Affine2],
) -> AttachmentKind {
    let (w, h) = (part.width as usize, part.height as usize);
    let thin = w.min(h) as f32;
    let cell = if bend.bones.len() > 1 {
        (thin / 4.0).clamp(3.0, 32.0)
    } else {
        (thin / 3.0).clamp(2.0, 24.0)
    };
    let (nx, ny) = (
        (w as f32 / cell).ceil().max(1.0) as usize,
        (h as f32 / cell).ceil().max(1.0) as usize,
    );
    let gx = |i: usize| ((i as f32 * cell).round() as usize).min(w);
    let gy = |j: usize| ((j as f32 * cell).round() as usize).min(h);
    let drawn = |i: usize, j: usize| {
        (gy(j)..gy(j + 1)).any(|y| (gx(i)..gx(i + 1)).any(|x| part.rgba[(y * w + x) * 4 + 3] > 0))
    };
    // The part's thickness across its bones: the joint band is half of it.
    let across = part
        .solid()
        .map(|(x, y)| {
            let q = to_rig([x as f32 + 0.5, y as f32 + 0.5]);
            let v = [q[0] - bend.origin[0], q[1] - bend.origin[1]];
            (v[0] * -bend.dir[1] + v[1] * bend.dir[0]).abs()
        })
        .fold(0.0f32, f32::max);
    let total = bend.step * bend.bones.len() as f32;
    let band = across.clamp(3.0, (0.45 * total).max(3.0));
    let inverse: Vec<Affine2> = world.iter().map(Affine2::inverse).collect();
    let mut index = vec![u32::MAX; (nx + 1) * (ny + 1)];
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for j in 0..ny {
        for i in 0..nx {
            if !drawn(i, j) {
                continue;
            }
            let mut corner = |ci: usize, cj: usize| {
                let k = cj * (nx + 1) + ci;
                if index[k] == u32::MAX {
                    let (px, py) = (gx(ci) as f32, gy(cj) as f32);
                    let q = to_rig([part.left as f32 + px, part.top as f32 + py]);
                    let d = (q[0] - bend.origin[0]) * bend.dir[0]
                        + (q[1] - bend.origin[1]) * bend.dir[1];
                    let influences = weights(bend, d, band)
                        .into_iter()
                        .map(|(bone, weight)| {
                            let [x, y] = inverse[bone].apply_to(q);
                            Influence { bone, x, y, weight }
                        })
                        .collect();
                    index[k] = vertices.len() as u32;
                    vertices.push(MeshVertex2d {
                        uv: [rect[0] as f32 + px, rect[1] as f32 + py],
                        influences,
                    });
                }
                index[k]
            };
            let (a, b, c, d) = (
                corner(i, j),
                corner(i + 1, j),
                corner(i + 1, j + 1),
                corner(i, j + 1),
            );
            triangles.extend([a, b, c, c, d, a]);
        }
    }
    AttachmentKind::Mesh {
        vertices,
        triangles,
    }
}
