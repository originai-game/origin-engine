//! A frame's 2D batch: the scene's sprites, text and tiles as sorted runs of instances.

use super::*;

/// The world rectangle an orthographic camera sees (for culling tiles).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ViewRect {
    pub min: Vec2,
    pub max: Vec2,
}

/// Appends the instances of a tile layer's chunks inside `view` (all of
/// them without one), per repeat, and returns (quads, chunks). `lit`: the
/// tiles take the 2D light map as a lit sprite does (`build`'s `lit`).
pub(super) fn tile_instances(
    layer: &TileLayer,
    view: Option<ViewRect>,
    lit: f32,
    out: &mut Vec<SpriteInstance>,
    mut gpu: Option<(usize, &mut Vec<crate::tiles_gpu::TileDraw>)>,
) -> (u32, u32) {
    let origin = layer.origin.truncate();
    let mut offsets: Vec<Vec2> = vec![Vec2::ZERO];
    if let Some(v) = view {
        // Repeats: every copy of the layer that can touch the view.
        let mut bounds = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        for c in layer.chunks.iter() {
            bounds.0 = bounds.0.min(c.min);
            bounds.1 = bounds.1.max(c.max);
        }
        for axis in 0..2 {
            let Some(period) = layer.repeat[axis].filter(|p| *p > 0.0) else {
                continue;
            };
            let lo = (v.min[axis] - origin[axis] - bounds.1[axis]) / period;
            let hi = (v.max[axis] - origin[axis] - bounds.0[axis]) / period;
            let (lo, hi) = (lo.floor() as i64, hi.ceil() as i64);
            let mut next = Vec::new();
            for o in &offsets {
                for k in lo.max(-4096)..=hi.min(lo.max(-4096) + 4096) {
                    let mut n = *o;
                    n[axis] = k as f32 * period;
                    next.push(n);
                }
            }
            offsets = next;
        }
    }
    let (mut quads, mut chunks) = (0u32, 0u32);
    for off in offsets {
        let at = origin + off;
        for (chunk, c) in layer.chunks.iter().enumerate() {
            if let Some(v) = view
                && (at.x + c.max.x < v.min.x
                    || at.x + c.min.x > v.max.x
                    || at.y + c.max.y < v.min.y
                    || at.y + c.min.y > v.max.y)
            {
                continue;
            }
            chunks += 1;
            if let Some((layer, draws)) = &mut gpu {
                draws.push(crate::tiles_gpu::TileDraw {
                    layer: *layer,
                    chunk,
                    offset: off,
                    lit,
                });
                quads += c.quads.len() as u32;
                continue;
            }
            let first = out.len();
            for q in &c.quads {
                out.push(SpriteInstance {
                    position_rotation: [at.x + q.min.x, at.y + q.min.y, layer.origin.z, 0.0],
                    size_anchor: [q.size.x, q.size.y, 0.0, 0.0],
                    uv_rect: q.uv,
                    color: layer.color,
                    flash: NO_FLASH,
                    fx: [0.0, lit, 0.0],
                    ground: NO_GROUND,
                    layer: 0,
                    facing: FACING_NONE,
                    outline: NO_OUTLINE,
                    outline_color: [0; 4],
                });
            }
            for a in &c.animated {
                let total: f32 = a.durations.iter().sum();
                if total <= 0.0 || a.uvs.is_empty() {
                    continue;
                }
                let mut t = layer.time.rem_euclid(total);
                let mut frame = 0;
                for (i, d) in a.durations.iter().enumerate() {
                    if t < *d {
                        frame = i;
                        break;
                    }
                    t -= d;
                    frame = i;
                }
                if let Some(inst) = out.get_mut(first + a.quad as usize) {
                    inst.uv_rect = a.uvs[frame.min(a.uvs.len() - 1)];
                }
            }
            quads += c.quads.len() as u32;
        }
    }
    (quads, chunks)
}

/// Sorts the scene's sprites, the frame's 2D particles (`extra`) and texts
/// into runs, with the tile layers and the layered strips placed before
/// the sprites of their layer. `font(i)` is the layout of the scene's font
/// `i`; `array_layers(a)` the layers of an uploaded texture array (`None`:
/// not uploaded or removed); `lit(layer, light mask)` is the light map a
/// receiver takes, 0 unlit, else 1 + its strip (`light2d::LightSlots`); `view` culls tile chunks (orthographic cameras). A sprite or
/// particle drawn from an array ([`RenderScene::sprite_arrays`]) joins its
/// array's run, and the glow pass by array when it glows. `mask` is the
/// camera's three.js layer mask: a world sprite or 2D particle whose own
/// ([`RenderScene::sprite_layers`], [`ExtraSprite::layers`]) shares no bit
/// with it is left out, with its shadow and glow (`views.rs`). A faced
/// sprite ([`RenderScene::sprite_facing`]) is keyed back to front from
/// `eye` unless it has a key of its own, and left out (its glow too, not
/// its drop shadow) when its bounds miss the eye's frustum.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build<'a>(
    scene: &RenderScene,
    font: impl Fn(usize) -> Option<&'a FontLayout>,
    texture_count: usize,
    array_layers: impl Fn(TextureArrayHandle) -> Option<u32>,
    extra: &[ExtraSprite],
    gpu: (&[crate::particles_gpu::SpriteSystem], &mut GpuSprites),
    lit: impl Fn(i32, u32) -> f32,
    view: Option<ViewRect>,
    strips: &[LayeredStrip],
    mask: u32,
    eye: &SpriteEye,
) -> Result<SpriteFrame, RenderError> {
    // Per-sprite effects by index (the last entry for an index wins).
    let mut fx: HashMap<u32, &crate::SpriteFx> = HashMap::new();
    for f in &scene.sprite.sprite_fx {
        if f.sprite as usize >= scene.sprite.sprites.len() {
            return Err(RenderError::InvalidHandle(format!(
                "sprite_fx: sprite {} of {}",
                f.sprite,
                scene.sprite.sprites.len()
            )));
        }
        fx.insert(f.sprite, f);
    }
    let mut order: HashMap<u32, f32> = HashMap::new();
    for o in &scene.sprite.sprite_order {
        if o.sprite as usize >= scene.sprite.sprites.len() {
            return Err(RenderError::InvalidHandle(format!(
                "sprite_order: sprite {} of {}",
                o.sprite,
                scene.sprite.sprites.len()
            )));
        }
        order.insert(o.sprite, if o.key.is_finite() { o.key } else { 0.0 });
    }
    // Sprites drawn from texture arrays, by index (the last entry wins).
    let arrays = array_entries(
        &scene.sprite.sprite_arrays,
        scene.sprite.sprites.len(),
        "sprite",
        &array_layers,
    )?;
    // Drop shadows by sprite index (the last entry wins), under a sun.
    let mut shadows: HashMap<u32, (f32, crate::ShadowGround)> = HashMap::new();
    for s in &scene.sprite.sprite_shadows {
        if s.sprite as usize >= scene.sprite.sprites.len() {
            return Err(RenderError::InvalidHandle(format!(
                "sprite_shadows: sprite {} of {}",
                s.sprite,
                scene.sprite.sprites.len()
            )));
        }
        shadows.insert(s.sprite, (s.opacity.clamp(0.0, 1.0), s.ground));
    }
    let ground =
        shadow_ground(scene.lighting.light.direction_to_light).filter(|_| !shadows.is_empty());
    // Pixel outlines by sprite index (the last entry wins).
    let mut lines: HashMap<u32, &crate::SpriteOutline> = HashMap::new();
    for o in &scene.sprite.sprite_outlines {
        if o.sprite as usize >= scene.sprite.sprites.len() {
            return Err(RenderError::InvalidHandle(format!(
                "sprite_outlines: sprite {} of {}",
                o.sprite,
                scene.sprite.sprites.len()
            )));
        }
        lines.insert(o.sprite, o);
    }
    // Material shaders by sprite index (the last entry wins); a material
    // without a shader sprites run draws plain.
    let mut shading: HashMap<u32, crate::MaterialHandle> = HashMap::new();
    for m in &scene.sprite.sprite_shading {
        if m.sprite as usize >= scene.sprite.sprites.len() {
            return Err(RenderError::InvalidHandle(format!(
                "sprite_shading: sprite {} of {}",
                m.sprite,
                scene.sprite.sprites.len()
            )));
        }
        match scene.sprite_shader(m.material) {
            Some(_) => shading.insert(m.sprite, m.material),
            None => shading.remove(&m.sprite),
        };
    }
    let mut shadow_count = 0u32;
    let mut culled = 0u32;
    let outlines: HashMap<u32, &crate::TextOutline> = scene
        .sprite
        .text_outlines
        .iter()
        .map(|o| (o.text, o))
        .collect();
    let text_facing: HashMap<u32, &crate::TextFacing> = scene
        .sprite
        .text_facing
        .iter()
        .map(|t| (t.text, t))
        .collect();
    // (layer, key, source, untonemapped, instance), in submission order.
    let mut keyed: Vec<(i32, f32, RunSource, bool, SpriteInstance)> =
        Vec::with_capacity(scene.sprite.sprites.len() + extra.len());
    let mut glowing: Vec<(RunSource, SpriteInstance)> = Vec::new();
    // Billboards (depth, layer, source, instance) and their drop shadows.
    let mut boards: Vec<(f32, i32, RunSource, SpriteInstance)> = Vec::new();
    let mut board_shadows: Vec<(RunSource, SpriteInstance)> = Vec::new();
    for (i, s) in scene.sprite.sprites.iter().enumerate() {
        if let Some(t) = s.texture
            && t.index() >= texture_count
        {
            return Err(RenderError::InvalidHandle(format!(
                "sprite {i}: texture {}",
                t.index()
            )));
        }
        if crate::views::layer_of(&scene.sprite.sprite_layers, i) & mask == 0 {
            continue;
        }
        let found = if fx.is_empty() {
            None
        } else {
            fx.get(&(i as u32))
        };
        let (flash, glow, blend, tone_mapped) = match found {
            Some(f) => (
                f.flash,
                f.glow.max(0.0),
                f.blend,
                // Additive light is always the scene's.
                f.tone_mapped || f.blend == SpriteBlend::Additive,
            ),
            None => ([1.0, 1.0, 1.0, 0.0], 0.0, SpriteBlend::Alpha, true),
        };
        let mut inst = sprite_instance(
            s,
            flash,
            glow,
            lit(
                s.layer,
                crate::views::layer_of(&scene.sprite.sprite_light_masks, i),
            ),
            false,
        );
        if let Some(o) = lines.get(&(i as u32)) {
            outline(&mut inst, o, s.texture.and_then(|t| scene.texture_size(t)));
        }
        let facing = facing_of(&scene.sprite.sprite_facing, i);
        inst.facing = facing_code(facing);
        let faced = facing != SpriteFacing::None;
        // A billboard of its own depth sorts with the transparent meshes
        // (`billboards`); with a key, or untonemapped (its cover pass is
        // the sprite order's), it keeps the layer order.
        let shaded = shading.get(&(i as u32)).copied();
        let board = faced && tone_mapped && shaded.is_none() && !order.contains_key(&(i as u32));
        // Its texture, with the material's hooks when it has one.
        let plain = |blend| match shaded {
            Some(m) => RunSource::Shaded(blend, s.texture, m),
            None => RunSource::Sprite(blend, s.texture),
        };
        // A billboard sorts back to front from the eye within its layer.
        let key = match order.get(&(i as u32)) {
            Some(&k) => k,
            None if faced => -eye.depth(s.position),
            None => 0.0,
        };
        // Its drop shadow: the same quad and texel alpha in black, projected
        // onto the ground, before every sprite of the layer.
        if let (Some(g), Some(&(opacity, on))) = (ground, shadows.get(&(i as u32)))
            && opacity > 0.0
        {
            let mut shadow = sprite_instance(s, [1.0, 1.0, 1.0, 0.0], 0.0, 0.0, false);
            shadow.color = [0.0, 0.0, 0.0, opacity * s.color[3]];
            let height = if on.height.is_finite() {
                on.height
            } else {
                0.0
            };
            shadow.ground = [g[0], g[1], g[2], g[3] + height];
            let [rise_x, rise_z] = on.slope();
            shadow.fx = [0.0, rise_x, rise_z];
            if faced {
                shadow.facing = FACING_SUN;
            }
            let source = match arrays.get(&(i as u32)) {
                Some(&(array, layer)) => {
                    shadow.layer = layer;
                    RunSource::Array(SpriteBlend::Alpha, array)
                }
                None => plain(SpriteBlend::Alpha),
            };
            let cells = match on.bend {
                // Bent over the ground: one instance per cell of the quad.
                Some(bend) => bent_shadow(shadow, &bend, g[3]).collect(),
                None => vec![shadow],
            };
            for cell in cells {
                if board {
                    board_shadows.push((source, cell));
                } else {
                    keyed.push((s.layer, f32::MIN, source, false, cell));
                    shadow_count += 1;
                }
            }
        }
        if faced
            && let Some(frustum) = eye.frustum
            && !frustum.intersects_sphere(s.position, sprite_radius(s))
        {
            culled += 1;
            continue;
        }
        if let Some(&(array, layer)) = arrays.get(&(i as u32)) {
            inst.layer = layer;
            if glow > 0.0 {
                glowing.push((RunSource::Array(SpriteBlend::Additive, array), inst));
            }
            if board {
                boards.push((
                    eye.depth(s.position),
                    s.layer,
                    RunSource::Array(blend, array),
                    inst,
                ));
                continue;
            }
            keyed.push((
                s.layer,
                key,
                RunSource::Array(blend, array),
                !tone_mapped,
                inst,
            ));
            continue;
        }
        if glow > 0.0 {
            glowing.push((RunSource::Sprite(SpriteBlend::Additive, s.texture), inst));
        }
        if board {
            boards.push((
                eye.depth(s.position),
                s.layer,
                RunSource::Sprite(blend, s.texture),
                inst,
            ));
            continue;
        }
        keyed.push((s.layer, key, plain(blend), !tone_mapped, inst));
    }
    let sprite_boards = boards.len() as u32;
    for e in extra.iter().filter(|e| e.layers & mask != 0) {
        let s = &e.sprite;
        let mut inst = sprite_instance(s, [1.0, 1.0, 1.0, 0.0], e.glow, lit(s.layer, 1), e.soft);
        let array = match e.array {
            Some((array, layer)) => {
                let layers = array_layers(array).ok_or_else(|| {
                    RenderError::InvalidHandle(format!(
                        "a 2D particle's texture array {}",
                        array.index()
                    ))
                })?;
                if layer >= layers {
                    return Err(RenderError::InvalidData(format!(
                        "a 2D particle: layer {layer} of a texture array of {layers}"
                    )));
                }
                inst.layer = layer;
                Some(array)
            }
            None => None,
        };
        let run = |b: SpriteBlend| match array {
            Some(a) => RunSource::Array(b, a),
            None => RunSource::Sprite(b, s.texture),
        };
        if e.glow > 0.0 {
            glowing.push((run(SpriteBlend::Additive), inst));
        }
        keyed.push((s.layer, 0.0, run(e.blend), false, inst));
    }
    // The 2D particles simulated in compute: a run of their range each
    // (`GpuSprites`), sorted as the CPU's particles are, after the layer's
    // sprites.
    let (gpu_systems, gpu_sprites) = gpu;
    let mut gpu_glow: Vec<u32> = Vec::new();
    let placeholder = sprite_instance(&Sprite::default(), [1.0, 1.0, 1.0, 0.0], 0.0, 0.0, false);
    let mut gpu_count = 0u32;
    for (i, g) in gpu_systems.iter().enumerate() {
        if g.layers & mask == 0 {
            continue;
        }
        let k = gpu_sprites.take(i, g, lit(g.layer, 1));
        if let RunSource::Array(_, a) = gpu_sprites.ranges[k as usize].source {
            array_layers(a).ok_or_else(|| {
                RenderError::InvalidHandle(format!("a 2D particle's texture array {}", a.index()))
            })?;
        }
        keyed.push((g.layer, 0.0, RunSource::Particles(k), false, placeholder));
        gpu_count += 1;
        if g.glow > 0.0 {
            gpu_glow.push(k);
        }
    }
    // The sprite meshes, sorted as sprites are; each is a run of its own
    // corners (`mesh_ranges`), not an instance.
    let mesh_ranges = crate::sprite_mesh::ranges(scene, texture_count)?;
    let mesh_count = scene.sprite.sprite_meshes.len() as u32;
    for (i, m) in scene.sprite.sprite_meshes.iter().enumerate() {
        let key = if m.key.is_finite() { m.key } else { 0.0 };
        let placeholder =
            sprite_instance(&Sprite::default(), [1.0, 1.0, 1.0, 0.0], 0.0, 0.0, false);
        keyed.push((m.layer, key, RunSource::Mesh(i as u32), false, placeholder));
    }
    let sprites = keyed.len() as u32 - shadow_count - mesh_count - gpu_count + sprite_boards;
    let after_sprites = keyed.len() as u32;
    let mut missing_glyphs = 0;
    for (i, text) in scene.sprite.texts.iter().enumerate() {
        let font = font(text.font.index()).ok_or_else(|| {
            RenderError::InvalidHandle(format!("text {i}: font {}", text.font.index()))
        })?;
        // An outline rides on the glyph's flash colour and glow strength
        // (which plain glyphs leave unused).
        let (outline_color, outline) = match outlines.get(&(i as u32)) {
            Some(o) if o.width > 0.0 => (
                o.color.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8),
                o.width.min(1.0),
            ),
            _ => ([255, 255, 255, 0], 0.0),
        };
        // A colour glyph silhouette page draws only an outline: the shape
        // grown by its width, all in the outline's colour, under the
        // colour images (world text has no shadow or glow).
        let silhouette_color = match outlines.get(&(i as u32)) {
            _ if !font.silhouette => None,
            Some(o) if o.width > 0.0 => Some([
                o.color[0],
                o.color[1],
                o.color[2],
                o.color[3] * text.color[3],
            ]),
            _ => continue,
        };
        let (sin, cos) = text.rotation.sin_cos();
        let rotate = |v: Vec2| Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos);
        let origin = text.position.truncate();
        // Faced text (`text_facing`) turns as one panel about its pivot:
        // every glyph anchored there, its corner the anchor's offset (each
        // line moved back by `align` of its width); a billboard sorting
        // with the transparent meshes.
        let (facing, align) = text_facing
            .get(&(i as u32))
            .map_or((SpriteFacing::None, 0.0), |t| (t.facing, t.align));
        let widths = if facing != SpriteFacing::None && align != 0.0 {
            font.walk(text, -font.line_height, |_, _, _, _| {})
        } else {
            Vec::new()
        };
        let depth = eye.depth(text.position);
        font.walk(text, -font.line_height, |ch, g, pen, offset| {
            let Some(g) = g else {
                if !ch.is_whitespace() {
                    missing_glyphs += 1;
                }
                return;
            };
            let [l, b, r, t] = g.plane_bounds;
            if r > l && t > b {
                let corner = (pen + offset + Vec2::new(l, b)) * text.size;
                let [al, at, ar, ab] = g.atlas_bounds;
                let size = [(r - l) * text.size, (t - b) * text.size];
                let mut inst = SpriteInstance {
                    position_rotation: [
                        origin.x + rotate(corner).x,
                        origin.y + rotate(corner).y,
                        text.position.z,
                        text.rotation,
                    ],
                    size_anchor: [size[0], size[1], 0.0, 0.0],
                    uv_rect: [
                        al / font.atlas_size.x,
                        at / font.atlas_size.y,
                        ar / font.atlas_size.x,
                        ab / font.atlas_size.y,
                    ],
                    // A colour page's images keep their colours (those
                    // in the text's colour mix it in, foreground_mix).
                    color: if let Some(c) = silhouette_color {
                        c
                    } else if font.color && !font.foreground {
                        [1.0, 1.0, 1.0, text.color[3]]
                    } else {
                        text.color
                    },
                    // An outline rides on the flash colour, which on a
                    // colour page would flash its images to it.
                    flash: if font.color {
                        [255, 255, 255, 0]
                    } else {
                        outline_color
                    },
                    fx: [if font.color { 0.0 } else { outline }, 0.0, 0.0],
                    ground: NO_GROUND,
                    layer: 0,
                    facing: FACING_NONE,
                    outline: NO_OUTLINE,
                    outline_color: [0; 4],
                };
                if facing == SpriteFacing::None {
                    keyed.push((text.layer, 0.0, RunSource::Font(text.font), false, inst));
                } else {
                    let line = (pen.y / -font.line_height).round() as usize;
                    let back = widths.get(line).map_or(0.0, |w| w * align * text.size);
                    let corner = corner - Vec2::new(back, 0.0);
                    inst.position_rotation[0] = origin.x;
                    inst.position_rotation[1] = origin.y;
                    inst.size_anchor[2] = -corner.x / size[0];
                    inst.size_anchor[3] = -corner.y / size[1];
                    inst.facing = facing_code(facing);
                    boards.push((depth, text.layer, RunSource::Font(text.font), inst));
                }
            }
        });
    }
    let glyphs = keyed.len() as u32 - after_sprites + boards.len() as u32 - sprite_boards;
    // Stable: within a (layer, key, source, tone) run, submission order is kept.
    og_core::sort::sort_by(&mut keyed, |a, b| {
        a.0.cmp(&b.0)
            .then(a.1.total_cmp(&b.1))
            .then(a.2.cmp(&b.2))
            .then(a.3.cmp(&b.3))
    });
    // Tile layers and layered strips, each before the sprites of its layer
    // (tiles first), in submission order within a layer.
    let mut under: Vec<(i32, u8, usize)> = Vec::new();
    for (i, t) in scene.sprite.tile_layers.iter().enumerate() {
        if t.texture.index() >= texture_count {
            return Err(RenderError::InvalidHandle(format!(
                "tile layer {i}: texture {}",
                t.texture.index()
            )));
        }
        under.push((t.layer, 0, i));
    }
    for (i, s) in strips.iter().enumerate() {
        under.push((s.layer, 1, i));
    }
    og_core::sort::sort_by_key(&mut under, |u| (u.0, u.1));
    let mut runs: Vec<SpriteRun> = Vec::new();
    let mut instances: Vec<SpriteInstance> = Vec::with_capacity(keyed.len());
    let push = |runs: &mut Vec<SpriteRun>,
                source: RunSource,
                tone_mapped: bool,
                first: u32,
                count: u32| {
        if count == 0 {
            return;
        }
        match runs.last_mut() {
            Some(run)
                if run.source == source
                    && run.tone_mapped == tone_mapped
                    && !matches!(source, RunSource::Strip(..) | RunSource::Metaballs(_))
                    && run.first + run.count == first =>
            {
                run.count += count
            }
            _ => runs.push(SpriteRun {
                source,
                first,
                count,
                tone_mapped,
            }),
        }
    };
    let (mut tiles, mut tile_chunks) = (0u32, 0u32);
    let mut next_under = 0;
    let mut place_under = |below: i32,
                           instances: &mut Vec<SpriteInstance>,
                           runs: &mut Vec<SpriteRun>,
                           next_under: &mut usize| {
        while *next_under < under.len() && under[*next_under].0 <= below {
            let (_, kind, i) = under[*next_under];
            if kind == 0 {
                let layer = &scene.sprite.tile_layers[i];
                let first = instances.len() as u32;
                let gpu_first = gpu_sprites.tiles.as_ref().map_or(0, |t| t.len());
                // Lit as the sprites of its layer are, unless it opts out.
                let (q, c) = tile_instances(
                    layer,
                    view,
                    if layer.lit {
                        lit(layer.layer, layer.light_mask)
                    } else {
                        0.0
                    },
                    instances,
                    gpu_sprites.tiles.as_mut().map(|t| (i, t)),
                );
                tiles += q;
                tile_chunks += c;
                if let Some(draws) = &gpu_sprites.tiles {
                    for (k, draw) in draws.iter().enumerate().skip(gpu_first) {
                        push(
                            runs,
                            RunSource::Tiles(k as u32),
                            true,
                            0,
                            layer.chunks[draw.chunk].quads.len() as u32,
                        );
                    }
                } else {
                    push(
                        runs,
                        RunSource::Sprite(SpriteBlend::Alpha, Some(layer.texture)),
                        true,
                        first,
                        q,
                    );
                }
            } else {
                let s = &strips[i];
                push(runs, s.source, true, s.first, s.count);
            }
            *next_under += 1;
        }
    };
    for (layer, _, source, untonemapped, inst) in keyed {
        place_under(layer, &mut instances, &mut runs, &mut next_under);
        if let RunSource::Mesh(i) = source {
            let (first, count) = mesh_ranges[i as usize];
            push(&mut runs, source, true, first, count);
            continue;
        }
        if let RunSource::Particles(k) = source {
            let r = &gpu_sprites.ranges[k as usize];
            push(&mut runs, source, true, r.first, r.count);
            continue;
        }
        let index = instances.len() as u32;
        instances.push(inst);
        push(&mut runs, source, !untonemapped, index, 1);
    }
    place_under(i32::MAX, &mut instances, &mut runs, &mut next_under);
    // The billboards' shadows, then the billboards far to near (ties by
    // layer, then as submitted), after the layer runs in the buffer.
    let mut billboard_shadows: Vec<SpriteRun> = Vec::new();
    for (source, inst) in board_shadows {
        let index = instances.len() as u32;
        instances.push(inst);
        push(&mut billboard_shadows, source, true, index, 1);
    }
    og_core::sort::sort_by(&mut boards, |a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut billboards: Vec<(f32, SpriteRun)> = Vec::with_capacity(boards.len());
    for (depth, _, source, inst) in boards {
        billboards.push((
            depth,
            SpriteRun {
                source,
                first: instances.len() as u32,
                count: 1,
                tone_mapped: true,
            },
        ));
        instances.push(inst);
    }
    // The glow pass: by texture or array (additive, so order does not
    // matter; a stable sort keeps the plain textures' order as before).
    og_core::sort::sort_by_key(&mut glowing, |(source, _)| *source);
    let mut glow_runs: Vec<SpriteRun> = Vec::new();
    for (source, inst) in glowing {
        let index = instances.len() as u32;
        match glow_runs.last_mut() {
            Some(run) if run.source == source => run.count += 1,
            _ => glow_runs.push(SpriteRun {
                source,
                first: index,
                count: 1,
                tone_mapped: true,
            }),
        }
        instances.push(inst);
    }
    // The glowing 2D particles simulated in compute: their ranges again.
    for k in gpu_glow {
        let r = &gpu_sprites.ranges[k as usize];
        glow_runs.push(SpriteRun {
            source: RunSource::Particles(k),
            first: r.first,
            count: r.count,
            tone_mapped: true,
        });
    }
    let untonemapped = runs.iter().any(|r| !r.tone_mapped);
    Ok(SpriteFrame {
        instances,
        runs,
        untonemapped,
        glow_runs,
        sprites,
        glyphs,
        missing_glyphs,
        tiles,
        tile_chunks,
        culled,
        billboards,
        billboard_shadows,
    })
}
