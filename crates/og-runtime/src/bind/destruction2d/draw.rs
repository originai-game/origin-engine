//! Drawing pixel terrains and pieces.

use super::*;

/// Extraction's textures of the terrains and pieces, by entity: the
/// texture and the (epoch, revision) its pixels were painted at.
#[derive(Default)]
pub(crate) struct PixelRender {
    pub(super) textures: BTreeMap<u64, (og_render::TextureHandle, u64, u64)>,
}

pub(super) use og_core::color::linear_to_srgb8 as srgb;

/// The last edit's dirty rectangles and the current seamless outlines.
#[inline(never)]
pub(crate) fn debug_lines(id: u32) -> Vec<(Vec3, Vec3, [f32; 4])> {
    let Ok(e) = engine(id) else {
        return Vec::new();
    };
    let g = lock(&e.modules.destruction2d.0);
    let mut lines = Vec::new();
    for t in g.state.terrains.values() {
        let at = |x: f32, y: f32| Vec3::from(t.origin) + Vec3::new(x, y, 0.0) * t.look.cell;
        if let Some(&(_, [x, y, w, h])) = t.updates.last() {
            let p = [[x, y], [x + w, y], [x + w, y + h], [x, y + h]];
            for i in 0..4 {
                lines.push((
                    at(p[i][0] as f32, p[i][1] as f32),
                    at(p[(i + 1) % 4][0] as f32, p[(i + 1) % 4][1] as f32),
                    [1.0, 0.5, 0.0, 1.0],
                ));
            }
        }
        for &k in t.colliders.keys() {
            for chain in t.grid.chunk_chains(k, 0.0) {
                for p in chain.windows(2) {
                    lines.push((
                        at(p[0][0], p[0][1]),
                        at(p[1][0], p[1][1]),
                        [0.2, 1.0, 0.3, 1.0],
                    ));
                }
            }
        }
    }
    lines
}

/// A grid's pixels, top row first: ground in its colour with a fixed grain
/// per cell (by its place in the terrain), bedrock, empty clear.
pub(super) fn paint(
    width: u32,
    cells: &[Cell],
    grain: [i32; 2],
    look: &Look,
    rect: [u32; 4],
) -> Vec<u8> {
    let [rx, ry, rw, rh] = rect;
    let mut rgba = vec![0u8; (rw * rh * 4) as usize];
    let rock = look.bedrock.map(srgb);
    for y in ry..ry + rh {
        let row = (ry + rh - 1 - y) * rw;
        for x in rx..rx + rw {
            let c = cells[(y * width + x) as usize];
            let px = &mut rgba[((row + x - rx) * 4) as usize..((row + x - rx) * 4 + 4) as usize];
            match c {
                0 => {}
                BEDROCK => px.copy_from_slice(&[rock[0], rock[1], rock[2], 255]),
                _ => {
                    let (gx, gy) = (grain[0] + x as i32, grain[1] + y as i32);
                    let h = splitmix(
                        u64::from(look.seed) << 40 ^ (gx as u32 as u64) << 20 ^ gy as u32 as u64,
                    );
                    let n = (h >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0;
                    let k = 1.0 + look.grain * n;
                    let [r, g, b] = look.color.map(|v| srgb(v * k));
                    px.copy_from_slice(&[r, g, b, 255]);
                }
            }
        }
    }
    rgba
}

/// A terrain or a piece as drawn: a sprite of its cells.
pub(super) struct Drawable<'a> {
    pub(super) entity: u64,
    pub(super) size: [u32; 2],
    pub(super) cells: &'a [Cell],
    /// Its cell (0, 0)'s place in the terrain (its grain).
    pub(super) grain: [i32; 2],
    pub(super) look: &'a Look,
    /// Its cells' revision (a piece's never change).
    pub(super) revision: u64,
    pub(super) updates: &'a [(u64, [u32; 4])],
    /// Whether it turns with its og.Transform (a piece does).
    pub(super) turns: bool,
}

/// Draws every terrain and piece as a sprite of its cells.
#[inline(never)]
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    let Ok(sh) = engine(rs.engine).map(|e| e.modules.destruction2d.0.clone()) else {
        return;
    };
    let g = lock(&sh);
    let epoch = g.epoch;
    let mut drawn: Vec<Drawable> = Vec::new();
    for (&k, t) in &g.state.terrains {
        drawn.push(Drawable {
            entity: k,
            size: [t.grid.width(), t.grid.height()],
            cells: t.grid.cells(),
            grain: [0, 0],
            look: &t.look,
            revision: t.revision,
            updates: &t.updates,
            turns: false,
        });
    }
    for (&k, p) in &g.state.pieces {
        drawn.push(Drawable {
            entity: k,
            size: [p.width, p.height],
            cells: &p.cells,
            grain: p.grain,
            look: &p.look,
            revision: 0,
            updates: &[],
            turns: true,
        });
    }
    let mut live = BTreeSet::new();
    for d in drawn {
        let Drawable {
            entity: k,
            size: [w, ht],
            cells,
            grain,
            look,
            revision,
            updates,
            turns,
        } = d;
        if !world.is_alive(Entity::from_bits(k)) {
            continue;
        }
        live.insert(k);
        let painted = rs.pixel.textures.get(&k).copied();
        let texture = match painted {
            Some((t, e, r)) if e == epoch && r == revision => t,
            Some((t, e, r))
                if e == epoch && updates.first().is_some_and(|(first, _)| r >= first - 1) =>
            {
                let rect = updates
                    .iter()
                    .filter(|(v, _)| *v > r)
                    .map(|(_, r)| *r)
                    .reduce(|a, b| {
                        let (x, y) = (a[0].min(b[0]), a[1].min(b[1]));
                        [
                            x,
                            y,
                            (a[0] + a[2]).max(b[0] + b[2]) - x,
                            (a[1] + a[3]).max(b[1] + b[3]) - y,
                        ]
                    });
                if let Some(rect) = rect {
                    let rgba = paint(w, cells, grain, look, rect);
                    rs.scene.set_texture_region(
                        t,
                        [rect[0], ht - rect[1] - rect[3], rect[2], rect[3]],
                        &rgba,
                    );
                }
                rs.pixel.textures.insert(k, (t, epoch, revision));
                t
            }
            _ => {
                let data = og_render::TextureData {
                    width: w,
                    height: ht,
                    rgba: paint(w, cells, grain, look, [0, 0, w, ht]),
                    color_space: og_render::ColorSpace::Srgb,
                    mips: og_render::Mips::None,
                };
                let t = match painted {
                    Some((t, ..)) => {
                        rs.scene.set_texture(t, data);
                        t
                    }
                    None => {
                        let t = rs.scene.add_texture(data);
                        rs.scene
                            .set_texture_filter(t, og_render::TextureFilter::Nearest);
                        t
                    }
                };
                rs.pixel.textures.insert(k, (t, epoch, revision));
                t
            }
        };
        let m = h.world_of(k);
        let xa = m.x_axis.truncate();
        rs.scene.sprite.sprites.push(og_render::Sprite {
            texture: Some(texture),
            position: m.w_axis.truncate(),
            size: Vec2::new(w as f32, ht as f32) * look.cell,
            anchor: Vec2::ZERO,
            rotation: if turns {
                og_core::math::atan2(xa.y, xa.x)
            } else {
                0.0
            },
            layer: look.layer,
            ..Default::default()
        });
    }
    drop(g);
    let stale: Vec<u64> = rs
        .pixel
        .textures
        .keys()
        .filter(|k| !live.contains(k))
        .copied()
        .collect();
    for k in stale {
        if let Some((t, ..)) = rs.pixel.textures.remove(&k) {
            rs.scene.remove_texture(t);
        }
    }
}
