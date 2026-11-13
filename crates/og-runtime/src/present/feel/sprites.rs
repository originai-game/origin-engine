//! The og.sprite system and its state block.

use super::*;

/// The rotation about z of a quaternion (deterministic arithmetic).
pub(super) fn yaw_z(q: Quat) -> f32 {
    og_core::math::atan2(
        2.0 * (q.w * q.z + q.x * q.y),
        1.0 - 2.0 * (q.y * q.y + q.z * q.z),
    )
}

pub(super) fn sprite_system(
    ctx: &mut SystemContext<'_>,
    shared: &Mutex<FeelShared>,
    r: &Records,
) -> og_core::Result<()> {
    let tick = ctx.time().tick;
    let now = ctx.time_scale().elapsed;
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let mut sh = lock(shared);
    sh.floating
        .retain(|t| now - t.start <= f64::from(t.lifetime));
    let mut seen = BTreeSet::new();
    for e in rows {
        let key = e.to_bits();
        seen.insert(key);
        let row: record::Afterimage = r
            .afterimage
            .1
            .read(&ctx.get_bytes(e, r.afterimage.0)?)
            .map_err(invalid)?;
        let s: record::Sprite = r
            .sprite
            .1
            .read(&ctx.get_bytes(e, r.sprite.0)?)
            .map_err(invalid)?;
        let t: record::Transform = r
            .transform
            .1
            .read(&ctx.get_bytes(e, r.transform.0)?)
            .map_err(invalid)?;
        let count = row.count.min(16) as usize;
        let interval = u64::from(row.interval).max(1);
        let list = sh.ghosts.entry(key).or_default();
        // Ghosts older than the whole trail have faded out.
        let span = interval * count as u64;
        list.retain(|g| tick.saturating_sub(g.tick) < span.max(1));
        if row.active && tick.is_multiple_of(interval) && count > 0 {
            let (sheet, clip, frame) = if ctx.components_of(e)?.contains(&r.flipbook.0) {
                let fb: record::Flipbook = r
                    .flipbook
                    .1
                    .read(&ctx.get_bytes(e, r.flipbook.0)?)
                    .map_err(|e| Error::new(ErrorKind::Invalid, e))?;
                (fb.sheet, fb.clip, fb.frame)
            } else {
                (0, 0, 0)
            };
            list.insert(
                0,
                Ghost {
                    position: t.position,
                    rotation: yaw_z(t.quaternion),
                    scale: Vec2::new(t.scale.x, t.scale.y),
                    sheet,
                    clip,
                    frame,
                    flip: s.flip_x as u8 | (s.flip_y as u8) << 1,
                    tick,
                },
            );
            list.truncate(count);
        }
    }
    sh.ghosts.retain(|k, v| seen.contains(k) && !v.is_empty());
    Ok(())
}

pub(super) struct SpriteBlock(pub(super) Arc<Mutex<FeelShared>>);

impl StateBlock for SpriteBlock {
    fn save(&self, out: &mut Vec<u8>) {
        let sh = lock(&self.0);
        let u32_ = |out: &mut Vec<u8>, v: u32| out.extend_from_slice(&v.to_le_bytes());
        let f32_ = |out: &mut Vec<u8>, v: f32| out.extend_from_slice(&v.to_le_bytes());
        out.extend_from_slice(&sh.spawned.to_le_bytes());
        u32_(out, sh.ghosts.len() as u32);
        for (k, list) in &sh.ghosts {
            out.extend_from_slice(&k.to_le_bytes());
            u32_(out, list.len() as u32);
            for g in list {
                for x in g.position.to_array() {
                    f32_(out, x);
                }
                f32_(out, g.rotation);
                f32_(out, g.scale.x);
                f32_(out, g.scale.y);
                out.extend_from_slice(&g.sheet.to_le_bytes());
                u32_(out, g.clip);
                u32_(out, g.frame);
                u32_(out, u32::from(g.flip));
                out.extend_from_slice(&g.tick.to_le_bytes());
            }
        }
        u32_(out, sh.floating.len() as u32);
        for t in &sh.floating {
            put_bytes(out, t.text.as_bytes());
            u32_(out, t.font);
            for x in t.position.to_array() {
                f32_(out, x);
            }
            f32_(out, t.drift);
            out.extend_from_slice(&t.start.to_le_bytes());
            f32_(out, t.size);
            for x in t.color.to_array() {
                f32_(out, x);
            }
            f32_(out, t.outline);
            for x in t.outline_color.to_array() {
                f32_(out, x);
            }
            for x in [t.rise, t.lifetime, t.fade, t.pop] {
                f32_(out, x);
            }
            out.extend_from_slice(&t.layer.to_le_bytes());
        }
        for map in [&sh.squash, &sh.flash] {
            u32_(out, map.len() as u32);
            for (k, v) in map {
                out.extend_from_slice(&k.to_le_bytes());
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
        // The floating texts' facings, a byte each, only when one faces the
        // camera: a block without is the one engines before
        // FloatingTextDesc.facing wrote and read, and its hash unchanged.
        if sh
            .floating
            .iter()
            .any(|t| t.facing != og_render::SpriteFacing::None)
        {
            out.extend(sh.floating.iter().map(|t| facing_byte(t.facing)));
        }
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        let mut r = Reader::new(bytes, "og.sprite");
        let mut sh = FeelShared {
            spawned: r.u64()?,
            ..FeelShared::default()
        };
        let v3r = |r: &mut Reader<'_>| -> og_core::Result<Vec3> {
            Ok(Vec3::new(r.f32()?, r.f32()?, r.f32()?))
        };
        for _ in 0..r.u32()? {
            let k = r.u64()?;
            let mut list = Vec::new();
            for _ in 0..r.u32()? {
                list.push(Ghost {
                    position: v3r(&mut r)?,
                    rotation: r.f32()?,
                    scale: Vec2::new(r.f32()?, r.f32()?),
                    sheet: r.u64()?,
                    clip: r.u32()?,
                    frame: r.u32()?,
                    flip: r.u32()? as u8,
                    tick: r.u64()?,
                });
            }
            sh.ghosts.insert(k, list);
        }
        for _ in 0..r.u32()? {
            let text = String::from_utf8(r.bytes()?.to_vec()).map_err(|_| {
                Error::new(ErrorKind::Parse, "og.sprite state block: text is not UTF-8")
            })?;
            sh.floating.push(Floating {
                text,
                font: r.u32()?,
                position: v3r(&mut r)?,
                drift: r.f32()?,
                start: r.f64()?,
                size: r.f32()?,
                color: v3r(&mut r)?,
                outline: r.f32()?,
                outline_color: v3r(&mut r)?,
                rise: r.f32()?,
                lifetime: r.f32()?,
                fade: r.f32()?,
                pop: r.f32()?,
                layer: r.i32()?,
                facing: og_render::SpriteFacing::None,
            });
        }
        for i in 0..2 {
            let mut map = IdMap::new();
            for _ in 0..r.u32()? {
                map.insert(r.u64()?, r.u64()?);
            }
            if i == 0 {
                sh.squash = map;
            } else {
                sh.flash = map;
            }
        }
        if !r.at_end() {
            for t in &mut sh.floating {
                t.facing = match r.u8()? {
                    1 => og_render::SpriteFacing::Camera,
                    2 => og_render::SpriteFacing::CameraY,
                    _ => og_render::SpriteFacing::None,
                };
            }
        }
        r.end()?;
        Ok(Box::new(sh))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut sh = lock(&self.0);
        let ops = std::mem::take(&mut sh.camera_ops);
        *sh = decoded
            .map(|d| *d.downcast::<FeelShared>().expect("decoded by SpriteBlock"))
            .unwrap_or_default();
        // The camera ops are the og.camera block's.
        sh.camera_ops = ops;
    }

    /// Afterimages and floating texts are at places in the world.
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut sh = lock(&self.0);
        for list in sh.ghosts.values_mut() {
            for g in list {
                g.position = s.moved(g.position);
            }
        }
        for t in &mut sh.floating {
            t.position = s.moved(t.position);
        }
    }
}
