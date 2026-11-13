//! Drawing pixel terrains and pieces.

use super::*;

/// Extraction's textures of the terrains and pieces, by entity: the
/// texture and the (epoch, revision) its pixels were painted at.
#[derive(Default)]
pub(crate) struct PixelRender {
    pub(super) textures: BTreeMap<u64, (og_render::TextureHandle, u64, u64)>,
}

pub(super) fn srgb(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * og_core::math::pow(c, 1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// A grid's pixels, top row first: ground in its colour with a fixed grain
/// per cell (by its place in the terrain), bedrock, empty clear.
pub(super) fn paint(
    width: u32,
    height: u32,
    cells: &[Cell],
    grain: [i32; 2],
    look: &Look,
) -> Vec<u8> {
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    let rock = look.bedrock.map(srgb);
    for y in 0..height {
        let row = (height - 1 - y) * width;
        for x in 0..width {
            let c = cells[(y * width + x) as usize];
            let px = &mut rgba[((row + x) * 4) as usize..((row + x) * 4 + 4) as usize];
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
    /// Whether it turns with its og.Transform (a piece does).
    pub(super) turns: bool,
}

/// Draws every terrain and piece as a sprite of its cells.
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
            turns,
        } = d;
        if !world.is_alive(Entity::from_bits(k)) {
            continue;
        }
        live.insert(k);
        let painted = rs.pixel.textures.get(&k).copied();
        let texture = match painted {
            Some((t, e, r)) if e == epoch && r == revision => t,
            _ => {
                let data = og_render::TextureData {
                    width: w,
                    height: ht,
                    rgba: paint(w, ht, cells, grain, look),
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
