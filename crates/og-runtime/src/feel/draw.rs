//! Sprite, floating text, tile layer and camera extraction.

use super::*;

// ── extraction ─────────────────────────────────────────────────────────────

/// A layer's chunks as drawn: the map, the chunk versions they were built at.
type TileCache = (u64, Vec<u64>, Arc<Vec<og_render::TileChunk>>);

/// What extraction keeps between frames for the feel modules.
#[derive(Default)]
pub(crate) struct FeelRender {
    pub feel: Option<(Feel, Records)>,
    /// The soft round texture of blob shadows, made on first use.
    blob: Option<og_render::TextureHandle>,
    /// Per (entity, layer): the map handle, the chunk versions and the chunks.
    tiles: BTreeMap<(u64, usize), TileCache>,
}

fn blob_texture(scene: &mut og_render::RenderScene) -> og_render::TextureHandle {
    const N: u32 = 64;
    let mut rgba = Vec::with_capacity((N * N * 4) as usize);
    for y in 0..N {
        for x in 0..N {
            let u = (x as f32 + 0.5) / N as f32 * 2.0 - 1.0;
            let v = (y as f32 + 0.5) / N as f32 * 2.0 - 1.0;
            // A solid core with a soft rim (the outer quarter fades).
            let d = ((1.0 - (u * u + v * v).sqrt()) * 4.0).clamp(0.0, 1.0);
            rgba.extend_from_slice(&[255, 255, 255, (d * 255.0).round() as u8]);
        }
    }
    scene.add_texture(og_render::TextureData {
        width: N,
        height: N,
        rgba,
        color_space: og_render::ColorSpace::Linear,
        mips: og_render::Mips::None,
    })
}

struct SpriteOut {
    sprite: og_render::Sprite,
    key: Option<f32>,
    flash: [f32; 4],
    glow: f32,
    additive: bool,
    cast_shadow: bool,
    /// Drawn from this layer of a texture array instead of its texture.
    array: Option<(og_render::TextureArrayHandle, u32)>,
    /// `Sprite.dropShadow`: a flattened dark copy on the ground (0 none),
    /// and `Sprite.dropShadowHeight`, the level ground under it (a higher
    /// terrain wins, `terrain::ground_drop_shadows`).
    drop_shadow: f32,
    drop_shadow_height: f32,
    /// `Sprite.toneMapped`: false keeps it out of exposure, bloom and the
    /// tone curve ([`og_render::SpriteFx::tone_mapped`]).
    tone_mapped: bool,
    /// `Sprite.facing`: a billboard ([`og_render::RenderScene::sprite_facing`]).
    facing: og_render::SpriteFacing,
    /// `Sprite.outline`: its width and logical pixel (texels) and colour
    /// ([`og_render::SpriteOutline`]).
    outline: Option<(f32, f32, [f32; 3])>,
    /// `Sprite.material`: drawn with its shader's hooks
    /// ([`og_render::SpriteShading`]).
    material: Option<og_render::MaterialHandle>,
}

/// A floating text's facing as the og.sprite state block stores it (its
/// decode reads it back): 0 none, 1 camera, 2 cameraY.
pub(super) fn facing_byte(f: og_render::SpriteFacing) -> u8 {
    match f {
        og_render::SpriteFacing::None => 0,
        og_render::SpriteFacing::Camera => 1,
        og_render::SpriteFacing::CameraY => 2,
    }
}

/// `SpriteFacing` by its IDL name (og.Sprite.facing, SpriteDesc.facing):
/// `none` for any other.
pub(crate) fn facing_named(name: Option<&str>) -> og_render::SpriteFacing {
    match name {
        Some("camera") => og_render::SpriteFacing::Camera,
        Some("cameraY") => og_render::SpriteFacing::CameraY,
        _ => og_render::SpriteFacing::None,
    }
}

/// Sets world sprite or text `index`'s facing in `list`
/// (`RenderScene::sprite_facing` or `text_facing`; the list stays short
/// while none faces the camera).
pub(crate) fn set_facing(
    list: &mut Vec<og_render::SpriteFacing>,
    index: usize,
    facing: og_render::SpriteFacing,
) {
    if facing == og_render::SpriteFacing::None && list.len() <= index {
        return;
    }
    if list.len() <= index {
        list.resize(index + 1, og_render::SpriteFacing::None);
    }
    list[index] = facing;
}

fn push(scene: &mut og_render::RenderScene, s: SpriteOut) {
    let index = scene.sprite.sprites.len() as u32;
    set_facing(&mut scene.sprite.sprite_facing, index as usize, s.facing);
    scene.sprite.sprites.push(s.sprite);
    if let Some((array, layer)) = s.array {
        scene.sprite.sprite_arrays.push(og_render::SpriteArray {
            sprite: index,
            array,
            layer,
        });
    }
    if let Some(key) = s.key {
        scene
            .sprite
            .sprite_order
            .push(og_render::SpriteOrder { sprite: index, key });
    }
    if let Some((width, pixel, [r, g, b])) = s.outline {
        scene.sprite.sprite_outlines.push(og_render::SpriteOutline {
            sprite: index,
            width,
            pixel,
            color: [r, g, b, 1.0],
        });
    }
    if let Some(material) = s.material {
        scene.sprite.sprite_shading.push(og_render::SpriteShading {
            sprite: index,
            material,
        });
    }
    if s.cast_shadow {
        scene
            .sprite
            .occluders_2d
            .push(og_render::Occluder2d::from_sprite(&s.sprite, 0.0));
    }
    if s.drop_shadow > 0.0 {
        scene.sprite.sprite_shadows.push(og_render::SpriteShadow {
            sprite: index,
            opacity: s.drop_shadow.min(1.0),
            ground: og_render::ShadowGround::level(s.drop_shadow_height),
        });
    }
    if s.flash[3] > 0.0 || s.glow > 0.0 || s.additive || !s.tone_mapped {
        scene.sprite.sprite_fx.push(og_render::SpriteFx {
            sprite: index,
            flash: s.flash,
            glow: s.glow,
            blend: if s.additive {
                og_render::SpriteBlend::Additive
            } else {
                og_render::SpriteBlend::Alpha
            },
            tone_mapped: s.tone_mapped,
        });
    }
}

/// What a sprite is drawn from: a texture (`None`: solid), or a layer of a
/// texture array.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Image {
    Texture(Option<og_render::TextureHandle>),
    Array(og_render::TextureArrayHandle, u32),
}

impl Image {
    fn texture(self) -> Option<og_render::TextureHandle> {
        match self {
            Image::Texture(t) => t,
            Image::Array(..) => None,
        }
    }

    fn array(self) -> Option<(og_render::TextureArrayHandle, u32)> {
        match self {
            Image::Texture(_) => None,
            Image::Array(a, layer) => Some((a, layer)),
        }
    }
}

/// The image, uv region and anchor a flipbook frame gives a sprite: a
/// region of the sheet's texture, or the whole layer of an `array true`
/// sheet's array (`None` while that array is not in the renderer).
fn sheet_frame(
    feel: &Feel,
    rs: (&[og_render::TextureHandle], &[super::assets::LoadedArray]),
    sheet: u64,
    clip: u32,
    frame: u32,
) -> Option<(Image, [f32; 4], Vec2)> {
    let s = feel.flipbooks.sheet(sheet)?;
    let c = s.clips.get(clip as usize)?;
    let index = *c.frames.get(frame as usize)?;
    let f = s.frames.get(index as usize)?;
    let pivot = Vec2::new(f.pivot[0], f.pivot[1]);
    match *lock(&feel.sheet_textures).get(&sheet)? {
        SheetImage::Texture(t) => Some((Image::Texture(Some(*rs.0.get(t)?)), s.uv(index), pivot)),
        SheetImage::Array(a) => {
            let loaded = rs.1.get(a)?;
            let layer = index.min(loaded.layers.saturating_sub(1));
            Some((
                Image::Array(loaded.handle?, layer),
                [0.0, 0.0, 1.0, 1.0],
                pivot,
            ))
        }
    }
}

/// An og.Sprite's own image: its texture array's layer when it names one
/// (`None` while the array's streaming group is not resident: nothing is
/// drawn), else its texture.
fn sprite_image(
    v: &impl Rec,
    s: &SpriteSlots,
    textures: &[og_render::TextureHandle],
    arrays: &[super::assets::LoadedArray],
) -> Option<Image> {
    let handle = |key: &Slot| v.num(key).and_then(super::assets::index_of);
    match handle(&s.array) {
        Some(i) => {
            let loaded = arrays.get(i)?;
            let layer = v.uint(&s.array_layer).unwrap_or(0) as u32;
            Some(Image::Array(
                loaded.handle?,
                layer.min(loaded.layers.saturating_sub(1)),
            ))
        }
        None => Some(Image::Texture(
            handle(&s.texture).and_then(|i| textures.get(i).copied()),
        )),
    }
}

fn flipped(mut uv: [f32; 4], mut anchor: Vec2, flip: u8) -> ([f32; 4], Vec2) {
    if flip & 1 != 0 {
        uv.swap(0, 2);
        anchor.x = 1.0 - anchor.x;
    }
    if flip & 2 != 0 {
        uv.swap(1, 3);
        anchor.y = 1.0 - anchor.y;
    }
    (uv, anchor)
}

/// og.Sprite rows (with their flipbook frames, nine-slices, shadows and
/// afterimages), then the floating texts and the tile layers.
// Rendering only: a sprite's drawn rotation.
#[allow(clippy::disallowed_methods)]
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    let Some((feel, r)) = rs.feel.feel.clone() else {
        return;
    };
    let s = &r.slots.sprite;
    let scene_tick = world.time().tick;
    let scaled_now = world.time_scale().elapsed;
    // Read in place from their columns (decoding each row to JSON was most
    // of a 2D frame's extraction: Hollowbell's 200 sprites; a packed copy
    // of each, most of what was left at 20,000).
    let flips = column_rows(world, &r.flipbook.1);
    let flips: IdMap<u64, ColRow<'_>> = IdMap::from_iter(flips.iter());
    let sprites = column_rows(world, &r.sprite.1);
    let afterimages = column_rows(world, &r.afterimage.1);
    let afterimages: IdMap<u64, ColRow<'_>> = IdMap::from_iter(afterimages.iter());
    let ghosts = lock(&feel.shared).ghosts.clone();
    if rs.feel.blob.is_none() && sprites.iter().any(|(_, v)| f(&v, &s.shadow, 0.0) > 0.0) {
        rs.feel.blob = Some(blob_texture(&mut rs.scene));
    }
    let blob = rs.feel.blob;
    // og.LightMask2D by entity: the 2D lights each receiver takes.
    let light_masks = crate::fx::light_masks(world, &rs.draw.fx);
    let textures = rs.textures.clone();
    let materials = rs.materials.clone();
    let arrays = &rs.texture_arrays;
    let scene = &mut rs.scene;
    let (art, notes) = (&mut rs.art, &mut rs.notes);
    // og.Sprite rows' sprites by index, for the probe's id pass.
    let mut drawn: Vec<(u32, u64)> = Vec::new();
    for (e, v) in sprites.iter() {
        // Its layers: the renderer draws it on the cameras they admit
        // (the main one and every og.CameraView or og.Mirror).
        let layers = v.uint(&s.layers).map_or(1, |x| x as u32);
        if !on(&v, &s.visible, true) {
            continue;
        }
        let first = scene.sprite.sprites.len();
        let m = h.world_of(e);
        let pos = m.w_axis.truncate();
        let xa = m.x_axis.truncate();
        let rotation = xa.y.atan2(xa.x);
        let scale = Vec2::new(xa.length(), m.y_axis.truncate().length());
        let size = Vec2::new(f(&v, &s.width, 1.0), f(&v, &s.height, 1.0));
        let flip = on(&v, &s.flip_x, false) as u8 | (on(&v, &s.flip_y, false) as u8) << 1;
        // A texture array whose streaming group is not resident draws nothing.
        let Some(mut image) = sprite_image(&v, s, &textures, arrays) else {
            continue;
        };
        let lo = v2_of(&v, &s.uv_min, Vec2::ZERO);
        let hi = v2_of(&v, &s.uv_max, Vec2::ONE);
        let mut uv = [lo.x, lo.y, hi.x, hi.y];
        let mut anchor = v2_of(&v, &s.anchor, Vec2::splat(0.5));
        let flipbook = flips.get(&e).map(|fb| {
            (
                fb.num(&r.slots.flipbook.sheet).map_or(0, |x| x as u64),
                fb.uint(&r.slots.flipbook.clip).unwrap_or(0) as u32,
                fb.uint(&r.slots.flipbook.frame).unwrap_or(0) as u32,
            )
        });
        if let Some((img, fuv, pivot)) =
            flipbook.and_then(|(s, c, fr)| sheet_frame(&feel, (&textures, arrays), s, c, fr))
        {
            image = img;
            uv = fuv;
            anchor = pivot;
        }
        let (uv, anchor) = flipped(uv, anchor, flip);
        let c = rgb_of(&v, &s.color, Vec3::ONE);
        let opacity = f(&v, &s.opacity, 1.0);
        let layer = v.int(&s.layer).unwrap_or(0) as i32;
        let key = on(&v, &s.sort_y, false).then_some(-pos.y);
        let fc = rgb_of(&v, &s.flash_color, Vec3::ONE);
        let flash = [fc.x, fc.y, fc.z, f(&v, &s.flash, 0.0).clamp(0.0, 1.0)];
        let glow = f(&v, &s.glow, 0.0).max(0.0);
        let additive = v.text(&s.blend).as_deref() == Some("additive");
        let tone_mapped = on(&v, &s.tone_mapped, true);
        let facing = facing_named(v.text(&s.facing).as_deref());
        // A palette swap: the texture's index texture through a row of
        // `palette` (its afterimages too).
        let palette = v
            .num(&s.palette)
            .and_then(super::assets::index_of)
            .and_then(|i| textures.get(i).copied());
        let row = v.uint(&s.palette_row).unwrap_or(0) as u32;
        // Its material shader (og.Sprite.material): the material's handle,
        // live in the render state.
        let material = v
            .num(&s.material)
            .and_then(super::assets::index_of)
            .and_then(|i| materials.get(i).copied());
        let mut swap = |scene: &mut og_render::RenderScene, t| match palette {
            Some(p) => super::art::paletted(scene, (&mut *art, &mut *notes), t, p, row),
            None => t,
        };
        let base = og_render::Sprite {
            texture: image.texture().map(|t| swap(scene, t)),
            position: pos,
            size: size * scale,
            anchor,
            rotation,
            color: [c.x, c.y, c.z, opacity],
            uv_rect: uv,
            layer,
            clip: None,
        };
        // The blob shadow: under every sprite of the layer.
        let shadow = f(&v, &s.shadow, 0.0);
        if shadow > 0.0 {
            let mut ss = v2_of(&v, &s.shadow_size, Vec2::ZERO);
            if ss == Vec2::ZERO {
                ss = Vec2::new(0.8, 0.3) * base.size.x;
            }
            let off = v2_of(&v, &s.shadow_offset, Vec2::ZERO);
            push(
                scene,
                SpriteOut {
                    sprite: og_render::Sprite {
                        texture: blob,
                        position: pos + off.extend(0.0),
                        size: ss,
                        anchor: Vec2::splat(0.5),
                        rotation: 0.0,
                        color: [0.0, 0.0, 0.0, shadow.min(1.0)],
                        uv_rect: [0.0, 0.0, 1.0, 1.0],
                        layer,
                        clip: None,
                    },
                    key: Some(f32::MIN),
                    flash: [1.0, 1.0, 1.0, 0.0],
                    glow: 0.0,
                    additive: false,
                    cast_shadow: false,
                    array: None,
                    drop_shadow: 0.0,
                    drop_shadow_height: 0.0,
                    tone_mapped: true,
                    facing: og_render::SpriteFacing::None,
                    outline: None,
                    material: None,
                },
            );
        }
        // Afterimages: oldest first, under the sprite.
        if let (Some(a), Some(list)) = (afterimages.get(&e), ghosts.get(&e)) {
            let count = a.uint(&r.slots.afterimage.count).unwrap_or(4).max(1) as f32;
            let interval = a.uint(&r.slots.afterimage.interval).unwrap_or(3).max(1) as f32;
            let tint = rgb_of(a, &r.slots.afterimage.color, Vec3::ONE);
            let op = f(a, &r.slots.afterimage.opacity, 0.5);
            let add = on(a, &r.slots.afterimage.additive, false);
            for g in list.iter().rev() {
                let age = scene_tick.saturating_sub(g.tick) as f32 / (count * interval);
                let alpha = op * (1.0 - age).clamp(0.0, 1.0);
                if alpha <= 0.0 {
                    continue;
                }
                let (gimage, guv, ganchor) =
                    sheet_frame(&feel, (&textures, arrays), g.sheet, g.clip, g.frame).unwrap_or((
                        image,
                        [lo.x, lo.y, hi.x, hi.y],
                        v2_of(&v, &s.anchor, Vec2::splat(0.5)),
                    ));
                let (guv, ganchor) = flipped(guv, ganchor, g.flip);
                let gtexture = gimage.texture().map(|t| swap(scene, t));
                push(
                    scene,
                    SpriteOut {
                        sprite: og_render::Sprite {
                            texture: gtexture,
                            position: g.position,
                            size: size * g.scale,
                            anchor: ganchor,
                            rotation: g.rotation,
                            color: [c.x * tint.x, c.y * tint.y, c.z * tint.z, opacity * alpha],
                            uv_rect: guv,
                            layer,
                            clip: None,
                        },
                        key: key.map(|k| k - 1e-4),
                        flash: [1.0, 1.0, 1.0, 0.0],
                        glow: 0.0,
                        additive: add,
                        cast_shadow: false,
                        array: gimage.array(),
                        drop_shadow: 0.0,
                        drop_shadow_height: 0.0,
                        tone_mapped,
                        facing,
                        outline: None,
                        material: None,
                    },
                );
            }
        }
        let sx = v2_of(&v, &s.slice_x, Vec2::ZERO);
        let sy = v2_of(&v, &s.slice_y, Vec2::ZERO);
        let size_of = match image {
            Image::Texture(t) => t.and_then(|t| scene.texture_size(t)),
            Image::Array(h, _) => arrays.iter().find(|a| a.handle == Some(h)).map(|a| a.size),
        };
        // Its nine-slice pieces, or itself (no list is made for a sprite
        // drawn whole, which nearly every one is).
        let sliced = match size_of {
            Some(tsize) if sx != Vec2::ZERO || sy != Vec2::ZERO => Some(og_render::nine_slice(
                &base,
                [sx.x, sy.x, sx.y, sy.y],
                tsize,
                f(&v, &s.slice_scale, 0.0625),
            )),
            _ => None,
        };
        let pieces = sliced.as_deref().unwrap_or(std::slice::from_ref(&base));
        // Its pixel outline (its own, or the art contract's), on a sprite
        // drawn whole.
        let outline = if pieces.len() == 1 {
            super::art::outline(
                &art.frame,
                scene,
                base.texture,
                (
                    f(&v, &s.outline, 0.0),
                    rgb_of(&v, &s.outline_color, Vec3::from(super::art::OUTLINE_COLOR)).into(),
                ),
                base.uv_rect,
                base.size,
            )
        } else {
            None
        };
        let cast = on(&v, &s.cast_shadow, false);
        let drop_shadow = f(&v, &s.drop_shadow, 0.0).clamp(0.0, 1.0);
        let drop_shadow_height = f(&v, &s.drop_shadow_height, 0.0);
        // A faced nine-slice turns as one panel about the sprite's anchor.
        let one_panel = facing != og_render::SpriteFacing::None && pieces.len() > 1;
        for (i, &p) in pieces.iter().enumerate() {
            drawn.push((scene.sprite.sprites.len() as u32, e));
            push(
                scene,
                SpriteOut {
                    sprite: if one_panel {
                        p.anchored_at(base.position)
                    } else {
                        p
                    },
                    key,
                    flash,
                    glow,
                    additive,
                    cast_shadow: cast && i == 0,
                    array: image.array(),
                    drop_shadow,
                    drop_shadow_height,
                    tone_mapped,
                    facing,
                    outline,
                    material,
                },
            );
        }
        // Its blob shadow, afterimages and pieces carry its mask.
        if layers != 1 {
            let masks = &mut scene.sprite.sprite_layers;
            masks.resize(first, 1);
            masks.resize(scene.sprite.sprites.len(), layers);
        }
        // So do its 2D light layers.
        if let Some(&m) = light_masks.get(&e)
            && m != 1
        {
            let masks = &mut scene.sprite.sprite_light_masks;
            masks.resize(first, 1);
            masks.resize(scene.sprite.sprites.len(), m);
        }
    }
    rs.ids.sprites = drawn;
    floating(rs, &feel, scaled_now);
    tile_layers(world, rs, &feel, &r, h, scaled_now, &light_masks);
}

// Rendering only: floating text's pop.
#[allow(clippy::disallowed_methods)]
fn floating(rs: &mut RenderState, feel: &Feel, now: f64) {
    let list = lock(&feel.shared).floating.clone();
    let ui = rs.ui.clone();
    for t in list {
        // No font of its own: the UI's for this text (its body face, or the
        // locale's or a fallback when the body face lacks a character), cut
        // into one piece per font when it mixes them: a style's Chinese
        // companion draws the Chinese of a line whose Latin is the body's.
        let body = ui.as_ref().map(|u| {
            let u = u.lock();
            let font = u.fonts().for_text(og_ui::FontRole::Body, &t.text);
            og_ui::pieces(font, &t.text, u.direction(), 0.0)
        });
        let age = (now - t.start) as f32;
        if age < 0.0 || age > t.lifetime {
            continue;
        }
        let k = age / t.lifetime;
        let rise = t.rise * (1.0 - (1.0 - k) * (1.0 - k));
        let pop = 1.0 + t.pop * (1.0 - (k / 0.2).min(1.0)).powi(2);
        let size = t.size * pop;
        let alpha = if k > 1.0 - t.fade && t.fade > 0.0 {
            ((1.0 - k) / t.fade).clamp(0.0, 1.0)
        } else {
            1.0
        };
        // Each piece: its font's pages, where it starts and how wide it is
        // (em), and what it draws.
        let mut parts: Vec<(Vec<og_render::FontHandle>, f32, f32, og_ui::Piece)> = Vec::new();
        if t.font == u32::MAX {
            let Some(pieces) = body else { continue };
            for piece in pieces {
                let Some(h) = piece.font else { continue };
                let pages = rs.ui_pages.get(&h).cloned().unwrap_or_else(|| vec![h]);
                parts.push((pages, piece.dx, piece.width, piece));
            }
        } else {
            let Some(p) = rs.fonts.get(t.font as usize) else {
                continue;
            };
            let piece = og_ui::Piece {
                font: None,
                dx: 0.0,
                width: 0.0,
                text: t.text.clone(),
                placements: Vec::new(),
            };
            parts.push((p.pages.clone(), 0.0, 0.0, piece));
        }
        // One piece is measured by its font (a font that is not mixed gives
        // no width).
        if let [(pages, _, width, piece)] = parts.as_mut_slice() {
            match pages.first().and_then(|p| rs.scene.font(*p)) {
                Some(first) => *width = first.measure(&piece.text),
                None => continue,
            }
        }
        let line: f32 = parts.iter().map(|p| p.2).sum();
        // Centred on its point: moved back by half its width, or, faced,
        // turned about the point with its line centred on it (each piece
        // aligned so the line's middle is the point).
        let faced = t.facing != og_render::SpriteFacing::None;
        for (pages, dx, width, piece) in parts {
            let position = Vec3::new(
                t.position.x + t.drift * k + if faced { 0.0 } else { (dx - line / 2.0) * size },
                t.position.y + rise,
                t.position.z,
            );
            let align = if width > 0.0 {
                (line / 2.0 - dx) / width
            } else {
                0.5
            };
            for page in pages {
                let index = rs.scene.sprite.texts.len() as u32;
                if faced {
                    rs.scene.sprite.text_facing.push(og_render::TextFacing {
                        text: index,
                        facing: t.facing,
                        align,
                    });
                }
                rs.scene.sprite.texts.push(og_render::Text {
                    font: page,
                    text: piece.text.clone(),
                    position,
                    size,
                    rotation: 0.0,
                    color: [t.color.x, t.color.y, t.color.z, alpha],
                    layer: t.layer,
                    clip: None,
                    tracking: 0.0,
                    weight: 0.0,
                    glow: None,
                    placements: piece.placements.clone(),
                });
                if t.outline > 0.0 {
                    rs.scene.sprite.text_outlines.push(og_render::TextOutline {
                        text: index,
                        width: t.outline,
                        color: [
                            t.outline_color.x,
                            t.outline_color.y,
                            t.outline_color.z,
                            alpha,
                        ],
                    });
                }
            }
        }
    }
}

fn tile_layers(
    world: &World,
    rs: &mut RenderState,
    feel: &Feel,
    r: &Records,
    h: &mut Hierarchy,
    now: f64,
    light_masks: &IdMap<u64, u32>,
) {
    let rows = column_rows(world, &r.tilemap.1);
    if rows.is_empty() {
        return;
    }
    let tiles = lock(&feel.tiles);
    let cam = rs.scene.frame.camera.position;
    let RenderState {
        scene,
        feel: fr,
        textures,
        art,
        notes,
        ..
    } = rs;
    let mut keep = BTreeSet::new();
    for (e, v) in rows.iter() {
        if !on(&v, &r.slots.tilemap.visible, true) {
            continue;
        }
        let (Some(live), map) = (
            tiles.live.get(&e),
            v.num(&r.slots.tilemap.map).map_or(0, |x| x as u64),
        ) else {
            continue;
        };
        if live.map != map {
            continue;
        }
        let Some(asset) = tiles.maps.get(&map) else {
            continue;
        };
        let origin = h.world_of(e).w_axis.truncate();
        let tint = rgb_of(&v, &r.slots.tilemap.color, Vec3::ONE);
        let opacity = f(&v, &r.slots.tilemap.opacity, 1.0);
        let base = v.int(&r.slots.tilemap.layer).unwrap_or(-10) as i32;
        // A palette swap of every layer (Tilemap.palette).
        let palette = v
            .num(&r.slots.tilemap.palette)
            .and_then(super::assets::index_of)
            .and_then(|i| textures.get(i).copied());
        let row = v.uint(&r.slots.tilemap.palette_row).unwrap_or(0) as u32;
        let (cw, ch) = live.state.chunks();
        let world_size = asset.asset.world_size();
        for (li, layer) in asset.asset.layers.iter().enumerate() {
            if !layer.visible {
                continue;
            }
            let Some(Some(ti)) = asset.textures.get(layer.tileset as usize) else {
                continue;
            };
            let Some(&texture) = textures.get(*ti) else {
                continue;
            };
            let Some((tw, th)) = scene.texture_size(texture) else {
                continue;
            };
            keep.insert((e, li));
            let versions: Vec<u64> = (0..ch)
                .flat_map(|cy| (0..cw).map(move |cx| (cx, cy)))
                .map(|(cx, cy)| live.state.chunk_version(li, cx, cy))
                .collect();
            let fresh = fr
                .tiles
                .get(&(e, li))
                .is_some_and(|(m, vs, _)| *m == map && *vs == versions);
            if !fresh {
                let mut chunks = Vec::new();
                for cy in 0..ch {
                    for cx in 0..cw {
                        let q = og_tilemap::chunk_quads(&live.state, li, cx, cy, tw, th);
                        if q.quads.is_empty() {
                            continue;
                        }
                        let mut min = Vec2::splat(f32::MAX);
                        let mut max = Vec2::splat(f32::MIN);
                        for t in &q.quads {
                            min = min.min(t.min);
                            max = max.max(t.min + t.size);
                        }
                        chunks.push(og_render::TileChunk {
                            min,
                            max,
                            quads: q
                                .quads
                                .iter()
                                .map(|t| og_render::TileQuad {
                                    min: t.min,
                                    size: t.size,
                                    uv: t.uv,
                                })
                                .collect(),
                            animated: q
                                .animated
                                .iter()
                                .map(|a| og_render::TileAnimation {
                                    quad: a.quad,
                                    uvs: a.frames.clone(),
                                    durations: a.durations.clone(),
                                })
                                .collect(),
                        });
                    }
                }
                fr.tiles.insert((e, li), (map, versions, Arc::new(chunks)));
            }
            let chunks = fr.tiles[&(e, li)].2.clone();
            let p = layer.parallax;
            let at = Vec3::new(
                origin.x + layer.offset.x + cam.x * (1.0 - p.x),
                origin.y + layer.offset.y + cam.y * (1.0 - p.y),
                origin.z,
            );
            let texture = match palette {
                Some(p) => super::art::paletted(scene, (&mut *art, &mut *notes), texture, p, row),
                None => texture,
            };
            scene.sprite.tile_layers.push(og_render::TileLayer {
                texture,
                chunks,
                origin: at,
                layer: base + layer.z,
                color: [tint.x, tint.y, tint.z, opacity * layer.opacity],
                repeat: [
                    layer.repeat_x.then_some(world_size.x),
                    layer.repeat_y.then_some(world_size.y),
                ],
                time: now as f32,
                lit: layer.lit,
                light_mask: light_masks.get(&e).copied().unwrap_or(1),
            });
        }
    }
    fr.tiles.retain(|k, _| keep.contains(k));
}

/// The camera feel on the frame only: trauma shake, punch, kick, and the
/// zone the followed entity is in (visual: the world's camera is untouched).
// Rendering only: the drawn camera's field of view.
#[allow(clippy::disallowed_methods)]
pub(crate) fn camera_frame(world: &World, rs: &mut RenderState, aspect: f32) {
    let Some((_, r)) = rs.feel.feel.clone() else {
        return;
    };
    let Ok(bytes) = world.resource_bytes(r.camera.0) else {
        return;
    };
    let Ok(cam) = r.camera.1.read::<record::Camera>(&bytes) else {
        return;
    };
    let scene = &mut rs.scene;
    let trauma = cam.trauma;
    let held = scene.frame.shake != og_render::CameraShake::default();
    if trauma > 0.0 && !held {
        scene.frame.shake = og_render::CameraShake::trauma(
            trauma,
            scene.frame.tick,
            scene.frame.tick_rate,
            0,
            cam.shake_offset,
            cam.shake_roll,
            cam.shake_frequency,
        );
    }
    let punch = cam.punch;
    if punch != 0.0 {
        let k = (1.0 + punch).max(0.1);
        scene.frame.camera.projection = match scene.frame.camera.projection {
            og_render::Projection::Orthographic { height } => {
                og_render::Projection::Orthographic { height: height / k }
            }
            og_render::Projection::Perspective { fov_y_degrees } => {
                let half = (fov_y_degrees.to_radians() / 2.0).tan() / k;
                og_render::Projection::Perspective {
                    fov_y_degrees: (2.0 * half.atan()).to_degrees(),
                }
            }
        };
    }
    let mut shift = cam.kick;
    // The zone og.camera picked (and the one a room transition leaves).
    if let og_render::Projection::Orthographic { height } = scene.frame.camera.projection
        && cam.zone_active
    {
        let half = Vec2::new(height * 0.5 * aspect, height * 0.5);
        let c = scene.frame.camera.position.truncate() + shift.truncate();
        let clamp = |v: f32, a: f32, b: f32, hf: f32| {
            if b - a <= 2.0 * hf {
                (a + b) * 0.5
            } else {
                v.clamp(a + hf, b - hf)
            }
        };
        let inside = |lo: Vec2, hi: Vec2| {
            Vec2::new(
                clamp(c.x, lo.x, hi.x, half.x),
                clamp(c.y, lo.y, hi.y, half.y),
            )
        };
        // A non-finite bound reads 0, as the hand read had it: a NaN one
        // would make `clamp` panic.
        let finite = |v: Vec2| Vec2::select(v.is_finite_mask(), v, Vec2::ZERO);
        let mut n = inside(finite(cam.zone_min), finite(cam.zone_max));
        let k = cam.zone_scroll.clamp(0.0, 1.0);
        if k < 1.0 && cam.zone_from_active {
            let from = inside(finite(cam.zone_from_min), finite(cam.zone_from_max));
            // Eased in and out over the scroll.
            let e = k * k * (3.0 - 2.0 * k);
            n = from + (n - from) * e;
        }
        shift += (n - c).extend(0.0);
    }
    if shift != Vec3::ZERO {
        scene.frame.camera.position += shift;
        scene.frame.camera.target += shift;
    }
}
