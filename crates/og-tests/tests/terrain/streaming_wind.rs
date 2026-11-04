use super::*;

/// The tiles a frame wants come best first and every one after its parent;
/// the top level is always first; the capacity caps them; a camera far
/// above wants fewer than one on the ground; the detail distance brings
/// the finest tiles around the camera whatever their size on screen.
#[test]
fn wanted_tiles_are_ranked_and_closed_under_their_parents() {
    let data = ts::island();
    let layout = data.layout(128).expect("layout");
    assert_eq!(
        layout.levels, 4,
        "1024 quads in tiles of 128: 8, 4, 2, 1 per side"
    );
    let scene = ts::island_view("near");
    let t = &scene.terrains[0];
    let view = og_render::TerrainView::of(&scene.camera, 720);
    let near = og_render::terrain_tiles_wanted(&layout, t, data.range(), &view, 1000);
    let top = layout.index(layout.levels - 1, 0, 0);
    assert_eq!(near[0], top);
    for (i, tile) in near.iter().enumerate() {
        if let Some(p) = layout.parent(*tile) {
            let at = near.iter().position(|x| *x == p).expect("parent wanted");
            assert!(at < i, "tile {tile} before its parent {p}");
        }
    }
    let unique: std::collections::HashSet<_> = near.iter().collect();
    assert_eq!(unique.len(), near.len(), "a tile twice");
    // The camera's own level-0 tile is wanted.
    let eye = Vec2::new(scene.camera.position.x, scene.camera.position.z) + layout.size * 0.5;
    let own = (eye / (layout.cell() * 128.0)).floor();
    assert!(near.contains(&layout.index(0, own.x as u32, own.y as u32)));
    let capped = og_render::terrain_tiles_wanted(&layout, t, data.range(), &view, 6);
    assert_eq!(capped.len(), 6);
    assert_eq!(&capped[..], &near[..6], "the cap keeps the best");
    let mut high = view;
    high.camera = Vec3::new(0.0, 4000.0, 0.0);
    let from_high = og_render::terrain_tiles_wanted(&layout, t, data.range(), &high, 1000);
    assert!(
        from_high.len() < near.len(),
        "{} from 4 km, {} on the ground",
        from_high.len(),
        near.len()
    );
    // 300 m of detail from 4 km up: the level-0 tiles under the camera come too.
    high.detail = 300.0;
    let detailed = og_render::terrain_tiles_wanted(&layout, t, data.range(), &high, 1000);
    let level0 = |l: &[u32]| {
        l.iter()
            .filter(|i| layout.locate(**i).unwrap().0 == 0)
            .count()
    };
    assert!(level0(&detailed) >= 4 && level0(&detailed) > level0(&from_high));
}

/// The renderer keeps the tiles a frame wants within its terrain's budget,
/// and with fewer than it wants (eight slots) the terrain still covers the
/// view without a crack: what has no tile reads its parent's.
#[test]
fn a_tile_budget_keeps_the_terrain_whole() {
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let full = crack_view(ts::island_scene());
    let s_full = r.render(&full, &RenderTarget::Offscreen).expect("render");
    let img_full = r.read_offscreen().expect("capture");
    assert_eq!(holes(&img_full, 260), 0);
    let mut tight = crack_view(ts::island_scene());
    tight.terrains[0].lod.budget = 8 * ts::island().layout(128).unwrap().slot_bytes();
    let mut r2 = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let s_tight = r2.render(&tight, &RenderTarget::Offscreen).expect("render");
    let img_tight = r2.read_offscreen().expect("capture");
    keep("budget-8", &img_tight);
    assert_eq!(s_tight.terrain_tiles, 8, "{s_tight:?}");
    assert!(s_full.terrain_tiles > 8, "{s_full:?}");
    assert_eq!(
        holes(&img_tight, 260),
        0,
        "background through the terrain on 8 tiles"
    );
    // Moving on, tiles are swapped in place: never more than the budget.
    for step in 1..5 {
        tight.camera.position.x += 60.0;
        tight.camera.target.x += 60.0;
        let s = r2.render(&tight, &RenderTarget::Offscreen).expect("render");
        assert_eq!(s.terrain_tiles, 8);
        assert_eq!(holes(&r2.read_offscreen().unwrap(), 260), 0, "step {step}");
    }
}

/// A streamed terrain draws what has arrived: with only its top tile it
/// covers the view at the coarse level; tiles handed over later fade in
/// over a dozen frames and then draw exactly what the field in memory
/// draws; a tile dropped falls back to its parent. Half the tiles at random
/// (each with its parent) leave no crack.
#[test]
fn a_streamed_terrain_draws_what_has_arrived() {
    let data = ts::island();
    let layout = data.layout(128).expect("layout");
    let control = og_render::ControlPyramid::new(&layout, data.control.as_ref().unwrap());
    let heights = data.height_pyramid(&layout);
    let tile = |i: u32| Arc::new(data.tile(&layout, i, &heights, control.as_ref(), None));
    let memory = crack_view(ts::island_scene());
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    r.render(&memory, &RenderTarget::Offscreen).expect("render");
    let reference = r.read_offscreen().expect("capture");

    let mut streamed = crack_view(ts::island_scene());
    let h = streamed.add_terrain_stream(layout, data.range());
    streamed.terrains[0].terrain = h;
    let top = layout.index(layout.levels - 1, 0, 0);
    assert!(streamed.set_terrain_tile(h, top, Some(tile(top))));
    let mut rs = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let s = rs
        .render(&streamed, &RenderTarget::Offscreen)
        .expect("render");
    let coarse = rs.read_offscreen().expect("capture");
    keep("stream-top-only", &coarse);
    assert_eq!(s.terrain_tiles, 1);
    assert_eq!(holes(&coarse, 260), 0, "the top tile alone covers the view");
    assert!(
        rmse(&coarse, &reference) > 0.005,
        "the top tile alone is coarser"
    );
    // Everything it wants arrives: a few frames in it is fading.
    let view = og_render::TerrainView::of(&streamed.camera, 720);
    let wanted =
        og_render::terrain_tiles_wanted(&layout, &streamed.terrains[0], data.range(), &view, 1000);
    for &i in wanted.iter().filter(|i| **i != top) {
        streamed.set_terrain_tile(h, i, Some(tile(i)));
    }
    let s = rs
        .render(&streamed, &RenderTarget::Offscreen)
        .expect("render");
    assert_eq!(
        s.terrain_uploads as usize,
        wanted.len() - 1,
        "one upload per new tile"
    );
    for _ in 0..3 {
        rs.render(&streamed, &RenderTarget::Offscreen)
            .expect("render");
    }
    let fading = rs.read_offscreen().expect("capture");
    assert_eq!(holes(&fading, 260), 0, "no crack while fading");
    for _ in 0..60 {
        let s = rs
            .render(&streamed, &RenderTarget::Offscreen)
            .expect("render");
        assert_eq!(s.terrain_uploads, 0, "nothing uploaded twice");
    }
    let arrived = rs.read_offscreen().expect("capture");
    keep("stream-arrived", &arrived);
    let e = rmse(&arrived, &reference);
    assert!(e < 1e-4, "streamed {e} against the field in memory");
    assert!(rmse(&fading, &reference) > rmse(&arrived, &reference));
    // Half the tiles at random, each with its parent: no crack.
    let mut kept = std::collections::HashSet::new();
    let mut seed = 7u32;
    for &i in &wanted {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let parent_in = layout.parent(i).is_none_or(|p| kept.contains(&p));
        if parent_in && (seed >> 16).is_multiple_of(2) || i == top {
            kept.insert(i);
        } else {
            streamed.set_terrain_tile(h, i, None);
        }
    }
    for _ in 0..2 {
        rs.render(&streamed, &RenderTarget::Offscreen)
            .expect("render");
    }
    let half = rs.read_offscreen().expect("capture");
    keep("stream-half", &half);
    assert_eq!(holes(&half, 260), 0, "background through half the tiles");
}

/// A streamed tile whose arrays are not the lengths its layout uploads is
/// refused (`set_terrain_tile` answers false and the terrain does not hold
/// it), so the frame after draws what the terrain holds instead of failing
/// in the upload.
#[test]
fn a_streamed_tile_of_the_wrong_size_is_refused() {
    let data = ts::island();
    let layout = data.layout(128).expect("layout");
    let control = og_render::ControlPyramid::new(&layout, data.control.as_ref().unwrap());
    let top = layout.index(layout.levels - 1, 0, 0);
    let good = data.tile(
        &layout,
        top,
        &data.height_pyramid(&layout),
        control.as_ref(),
        None,
    );
    let mut scene = crack_view(ts::island_scene());
    let h = scene.add_terrain_stream(layout, data.range());
    scene.terrains[0].terrain = h;
    let edit = |f: &dyn Fn(&mut og_render::TerrainTile)| {
        let mut t = good.clone();
        f(&mut t);
        Arc::new(t)
    };
    for (what, tile) in [
        ("heights", edit(&|t| t.heights = Arc::from(&t.heights[1..]))),
        ("normals", edit(&|t| t.normals = Arc::from(&t.normals[2..]))),
        (
            "control",
            edit(&|t| {
                let c = t.control.clone().expect("the island has a control map");
                t.control = Some(Arc::from(&c[4..]));
            }),
        ),
    ] {
        assert!(
            !scene.set_terrain_tile(h, top, Some(tile)),
            "a tile with too few {what} is refused"
        );
        assert!(
            scene.terrain_tiles(h).expect("streamed").is_empty(),
            "{what}"
        );
    }
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let s = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    assert_eq!(s.terrain_tiles, 0, "nothing held, nothing drawn");
    assert!(scene.set_terrain_tile(h, top, Some(Arc::new(good))));
    let s = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    assert_eq!(s.terrain_tiles, 1);
}

/// The renderer draws scattered copies from cached chunks: the copies a
/// frame draws are the CPU's placement (each cube where `Terrain::scatter`
/// puts it, at full size inside the fade), none beyond the distance, and
/// over the fade the field shrinks and thins out: each copy draws whole
/// (no pixel inside its footprint dropped) at a smaller size, some gone,
/// and a frozen tick draws the same pixels frame after frame. A second
/// frame builds no chunk; moving on builds only the chunks that came into
/// range.
///
/// Broken before G1 by the static dither: a fading copy kept its full
/// footprint with holes in it, grain that TAA cannot resolve.
#[test]
fn scattered_copies_are_drawn_from_chunks_and_shrink_over_the_fade() {
    let n = 257u32;
    let size = 1024.0f32;
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(TerrainData::flat(Vec2::splat(size), n, n));
    scene.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::splat(0.05),
            ..Default::default()
        }],
        cast_shadows: false,
        ..Terrain::new(h)
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let red = scene.add_material(Material {
        base_color: [0.0, 0.0, 0.0, 1.0],
        emissive: [8.0, 0.0, 0.0],
        ..Default::default()
    });
    let s = TerrainScatter {
        density: 0.03,
        distance: 90.0,
        fade: 0.5,
        scale: [3.0, 3.0],
        seed: 5,
        ..TerrainScatter::default()
    };
    scene.scatters.push(og_render::ScatterDraw {
        terrain: 0,
        mesh: cube,
        material: red,
        scatter: s,
        cast_shadows: true,
        impostor: None,
        impostor_distance: 0.0,
    });
    scene.fog = None;
    scene.post.tonemap = Tonemap::None;
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.light.cast_shadows = false;
    scene.camera = Camera {
        position: Vec3::new(0.0, 100.0, 0.001),
        target: Vec3::ZERO,
        near: 1.0,
        far: 500.0,
        projection: Projection::Orthographic { height: 200.0 },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let first = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    let img = r.read_offscreen().expect("capture");
    keep("scatter-fade", &img);
    assert!(first.scatter_built >= 4, "{first:?}");
    let again = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    assert_eq!(again.scatter_built, 0, "a second frame builds nothing");
    assert_eq!(again.scatter_copies, first.scatter_copies);
    // The distance fade has no temporal noise: the same tick draws the
    // same pixels on the next frame.
    let next = r.read_offscreen().expect("capture");
    assert!(img.pixels == next.pixels, "a frozen tick's fade is stable");
    // Where each copy is: 3.6 pixels a metre (720 px over 200 m).
    let px_of = |p: Vec3| (640.0 + p.x * 720.0 / 200.0, 360.0 + p.z * 720.0 / 200.0);
    let red_at = |x: f32, y: f32| {
        let (x, y) = (x.round() as i64, y.round() as i64);
        if x < 0 || y < 0 || x >= 1280 || y >= 720 {
            return false;
        }
        let p = px(&img, x as u32, y as u32);
        p[0] > 150.0 && p[1] < 60.0
    };
    let placed = scene.terrains[0].scatter(
        &TerrainData::flat(Vec2::splat(size), n, n),
        &s,
        scene.camera.position,
    );
    let (mut solid, mut thinned, mut gone) = (0, 0, 0);
    let (mut shrunk, mut faded_out) = (0, 0);
    let at_of = |m: &Mat4| m.w_axis.truncate();
    for (_, m) in &placed {
        let at = at_of(m);
        // Copies with a neighbour within 4 m would mix their pixels: skip them.
        if placed
            .iter()
            .any(|(_, o)| o != m && at_of(o).distance(at) < 4.0)
        {
            continue;
        }
        let d = Vec2::new(at.x, at.z).length();
        let (cx, cy) = px_of(at);
        // A disc of 4 px inside the 3 m (10.8 px) cube's top.
        let (mut inside, mut hit) = (0, 0);
        for dy in -4i32..=4 {
            for dx in -4i32..=4 {
                let r2 = (dx * dx + dy * dy) as f32;
                if r2 > 16.0 {
                    continue;
                }
                inside += 1;
                if red_at(cx + dx as f32, cy + dy as f32) {
                    hit += 1;
                }
            }
        }
        let fill = hit as f32 / inside as f32;
        if d < s.fade_from() - 3.0 {
            assert!(fill > 0.97, "a copy {d:.1} m out is filled {fill:.2}");
            solid += 1;
        } else if d > 60.0 && d < 74.0 {
            // Fading (fade 0.3 to 0.75): the copy is gone or a smaller,
            // solid square. Its red pixels' box within 8 px of its centre
            // (orthographic, from above: a square top) holds no hole
            // inside its 1 px rim.
            let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
            for dy in -8i32..=8 {
                for dx in -8i32..=8 {
                    if red_at(cx + dx as f32, cy + dy as f32) {
                        (x0, y0, x1, y1) = (x0.min(dx), y0.min(dy), x1.max(dx), y1.max(dy));
                    }
                }
            }
            if x0 > x1 {
                faded_out += 1;
                continue;
            }
            let mut holes = 0;
            for dy in (y0 + 1)..y1 {
                for dx in (x0 + 1)..x1 {
                    if !red_at(cx + dx as f32, cy + dy as f32) {
                        holes += 1;
                    }
                }
            }
            assert_eq!(
                holes, 0,
                "a copy {d:.1} m out has holes in its footprint: dithered, not shrunk"
            );
            // 3 m at 3.6 px a metre: 10.8 px whole.
            if (x1 - x0).max(y1 - y0) < 9 {
                shrunk += 1;
            }
            thinned += 1;
        }
    }
    assert!(
        shrunk > 0 && faded_out > 0,
        "the fade shrinks copies ({shrunk}) and thins them out ({faded_out} gone)"
    );
    assert!(solid > 5 && thinned > 3, "{solid} solid, {thinned} thinned");
    // Nothing past the distance: no red in the corners' ring beyond 92 m.
    for (x, y) in [(0u32, 0u32), (1279, 0), (0, 719), (1279, 719)] {
        let p = px(&img, x, y);
        assert!(p[0] < 100.0, "red at the corner {x},{y}");
    }
    for (_, m) in &placed {
        let at = m.w_axis.truncate();
        if Vec2::new(at.x, at.z).length() > 92.0 {
            gone += 1;
        }
    }
    assert_eq!(gone, 0, "the CPU placement keeps to the distance");
    // Moving 150 m east builds only the chunks that came into range.
    scene.camera.position.x += 150.0;
    scene.camera.target.x += 150.0;
    let moved = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    assert!(
        moved.scatter_built > 0 && moved.scatter_built < first.scatter_built,
        "{} chunks built moving, {} at first",
        moved.scatter_built,
        first.scatter_built
    );
}

/// og.Wind sways scattered copies downwind, seen from above: in a steady
/// wind (gust 0, turbulence 0) the plate each copy holds 2 m up moves by
/// strength × 2 m along the wind's horizontal direction, whatever the
/// tick; in gusts each moves downwind by about 0 to twice that (the wind
/// field's spring and pressure add a little), differently from tick to
/// tick and from copy to copy. Strength 0 and sway 0 draw what no wind
/// draws.
#[test]
fn wind_sways_scattered_copies_downwind() {
    let n = 129u32;
    let data = TerrainData::flat(Vec2::splat(256.0), n, n);
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(data.clone());
    scene.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::splat(0.05),
            ..Default::default()
        }],
        cast_shadows: false,
        ..Terrain::new(h)
    });
    // A 0.6 m plate held 2 m above the copy's root.
    let mut plate = shapes::plane(0.6, 1.0);
    for v in &mut plate.vertices {
        v.position[1] = 2.0;
    }
    let plate = scene.add_mesh(plate);
    let red = scene.add_material(Material {
        base_color: [0.0, 0.0, 0.0, 1.0],
        emissive: [8.0, 0.0, 0.0],
        ..Default::default()
    });
    let s = TerrainScatter {
        density: 0.01,
        distance: 200.0,
        fade: 0.0,
        scale: [1.0, 1.0],
        seed: 7,
        ..TerrainScatter::default()
    };
    scene.scatters.push(og_render::ScatterDraw {
        terrain: 0,
        mesh: plate,
        material: red,
        scatter: s,
        cast_shadows: false,
        impostor: None,
        impostor_distance: 0.0,
    });
    scene.fog = None;
    scene.post.tonemap = Tonemap::None;
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.light.cast_shadows = false;
    scene.camera = Camera {
        position: Vec3::new(0.0, 100.0, 0.001),
        target: Vec3::ZERO,
        near: 1.0,
        far: 500.0,
        projection: Projection::Orthographic { height: 60.0 },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    // 12 pixels a metre (720 px over 60 m), +z down the image.
    let px_of = |p: Vec2| Vec2::new(640.0 + p.x * 12.0, 360.0 + p.y * 12.0);
    // The roots in view with no other within 5 m, so no other plate comes
    // within 3 m of 1 m downwind of one.
    let placed: Vec<Vec2> = scene.terrains[0]
        .scatter(&data, &s, scene.camera.position)
        .iter()
        .map(|(_, m)| Vec2::new(m.w_axis.x, m.w_axis.z))
        .collect();
    let roots: Vec<Vec2> = placed
        .iter()
        .copied()
        .filter(|r| r.x.abs() < 49.0 && r.y.abs() < 26.0)
        .filter(|r| placed.iter().all(|o| o == r || o.distance(*r) > 5.0))
        .collect();
    assert!(roots.len() >= 6, "{} roots apart", roots.len());
    let draw = |scene: &mut RenderScene, tick: u64| {
        scene.tick = tick;
        let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
        r.render(scene, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };
    // Each plate's offset from its root in metres: the red centroid within
    // 3 m (36 px) of 1 m downwind of the root along `dir`.
    let offsets = |img: &RgbaImage, dir: Vec2| -> Vec<Vec2> {
        roots
            .iter()
            .map(|&r| {
                let near = px_of(r + dir);
                let (mut sum, mut w) = (Vec2::ZERO, 0.0);
                for y in (near.y as i32 - 36).max(0)..(near.y as i32 + 36).min(720) {
                    for x in (near.x as i32 - 36).max(0)..(near.x as i32 + 36).min(1280) {
                        let p = px(img, x as u32, y as u32);
                        let at = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                        if p[0] > 100.0 && p[1] < 60.0 && at.distance(near) < 36.0 {
                            sum += at * p[0];
                            w += p[0];
                        }
                    }
                }
                assert!(w > 0.0, "no plate near the root at {r}");
                (sum / w - px_of(r)) / 12.0
            })
            .collect()
    };
    let still = draw(&mut scene, 0);
    for o in offsets(&still, Vec2::ZERO) {
        assert!(o.length() < 0.06, "a plate off its root without wind: {o}");
    }
    // Steady: no gusts and no turbulence, so the wind field (G2) holds
    // og.Wind's lean exactly.
    let steady = Wind {
        strength: 0.5,
        direction: Vec3::new(3.0, 5.0, 4.0),
        speed: 3.0,
        gust: 0.0,
        turbulence: 0.0,
        ..Wind::default()
    };
    let dir = Vec2::new(0.6, 0.8);
    for tick in [0, 37] {
        scene.wind = Some(steady);
        let moved = offsets(&draw(&mut scene, tick), dir);
        let worst = moved.iter().map(|o| o.distance(dir)).fold(0.0, f32::max);
        println!(
            "steady wind, tick {tick}: {} plates moved {dir} within {worst:.3} m",
            moved.len()
        );
        assert!(
            worst < 0.06,
            "tick {tick}: moved {moved:?}, not {dir} (0.5 × 2 m)"
        );
    }
    scene.wind = Some(Wind {
        strength: 0.0,
        ..steady
    });
    assert!(
        draw(&mut scene, 9).pixels == still.pixels,
        "strength 0 moved a copy"
    );
    scene.wind = Some(steady);
    scene.scatters[0].scatter.sway = 0.0;
    assert!(
        draw(&mut scene, 9).pixels == still.pixels,
        "sway 0 moved a copy"
    );
    scene.scatters[0].scatter.sway = 1.0;
    scene.wind = Some(Wind {
        gust: 1.0,
        direction: Vec3::X,
        ..steady
    });
    let (a, b) = (
        offsets(&draw(&mut scene, 0), Vec2::X),
        offsets(&draw(&mut scene, 20), Vec2::X),
    );
    // The wind field (G2) carries the gusts as air: its pressure solve
    // turns a little of a gust front sideways and its spring overshoots by
    // under a fifth, so a plate moves downwind by 0 to 2.4 m with a small
    // crosswind part.
    for o in a.iter().chain(&b) {
        assert!(
            o.y.abs() < 0.25 && o.x > -0.1 && o.x < 2.46,
            "a gust moved a plate {o}: not downwind by 0 to 2.4 m"
        );
    }
    let changed = a
        .iter()
        .zip(&b)
        .filter(|(p, q)| (p.x - q.x).abs() > 0.1)
        .count();
    let spread = a.iter().map(|o| o.x).fold(f32::MIN, f32::max)
        - a.iter().map(|o| o.x).fold(f32::MAX, f32::min);
    println!(
        "gusts: {changed} of {} plates moved on by tick 20; spread at tick 0 {spread:.2} m",
        a.len()
    );
    assert!(
        changed * 2 >= a.len(),
        "the gusts hardly moved: {a:?} then {b:?}"
    );
    assert!(spread > 0.3, "every copy leans alike in the gusts: {a:?}");
    // The motion vectors follow the sway: with the camera still, motion
    // blur smears the plates (dimmed, so a smeared one is not clamped
    // back to full red) in a frame drawn 30 ticks after the renderer's
    // last, whose phases are the previous ones, and leaves a renderer's
    // first frame of the same tick as it is.
    let dim = scene.add_material(Material {
        base_color: [0.0, 0.0, 0.0, 1.0],
        emissive: [0.8, 0.0, 0.0],
        ..Default::default()
    });
    scene.scatters[0].material = dim;
    scene.post.motion_blur = Some(og_render::MotionBlur {
        shutter: 1.0,
        samples: 16,
        max_pixels: 16.0,
    });
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    for tick in [0, 30] {
        scene.tick = tick;
        r.render(&scene, &RenderTarget::Offscreen).expect("render");
    }
    let smeared = r.read_offscreen().expect("capture");
    let crisp = draw(&mut scene, 30);
    let blurred = smeared
        .pixels
        .chunks(4)
        .zip(crisp.pixels.chunks(4))
        .filter(|(p, q)| p[0].abs_diff(q[0]) > 32)
        .count();
    println!("motion blur of the sway: {blurred} pixels differ");
    assert!(
        blurred > 400,
        "the sway left the motion vectors still ({blurred} pixels)"
    );
}

/// G2's plate field: 0.6 m red plates held 2 m above their copies' roots
/// over a flat 256 m terrain, seen from straight above at 12 px a metre
/// (+x right, +z down), and the roots with no other within 5 m.
fn plate_field() -> (RenderScene, Vec<Vec2>) {
    let n = 129u32;
    let data = TerrainData::flat(Vec2::splat(256.0), n, n);
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(data.clone());
    scene.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::splat(0.05),
            ..Default::default()
        }],
        cast_shadows: false,
        ..Terrain::new(h)
    });
    let mut plate = shapes::plane(0.6, 1.0);
    for v in &mut plate.vertices {
        v.position[1] = 2.0;
    }
    let plate = scene.add_mesh(plate);
    let red = scene.add_material(Material {
        base_color: [0.0, 0.0, 0.0, 1.0],
        emissive: [8.0, 0.0, 0.0],
        ..Default::default()
    });
    let s = TerrainScatter {
        density: 0.04,
        distance: 200.0,
        fade: 0.0,
        scale: [1.0, 1.0],
        seed: 7,
        ..TerrainScatter::default()
    };
    scene.scatters.push(og_render::ScatterDraw {
        terrain: 0,
        mesh: plate,
        material: red,
        scatter: s,
        cast_shadows: false,
        impostor: None,
        impostor_distance: 0.0,
    });
    scene.fog = None;
    scene.post.tonemap = Tonemap::None;
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.light.cast_shadows = false;
    scene.camera = Camera {
        position: Vec3::new(0.0, 100.0, 0.001),
        target: Vec3::ZERO,
        near: 1.0,
        far: 500.0,
        projection: Projection::Orthographic { height: 60.0 },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    let placed: Vec<Vec2> = scene.terrains[0]
        .scatter(&data, &s, scene.camera.position)
        .iter()
        .map(|(_, m)| Vec2::new(m.w_axis.x, m.w_axis.z))
        .collect();
    let roots = placed
        .iter()
        .copied()
        .filter(|r| r.x.abs() < 46.0 && r.y.abs() < 23.0)
        .filter(|r| placed.iter().all(|o| o == r || o.distance(*r) > 5.0))
        .collect();
    (scene, roots)
}

/// Each plate's offset from its root (m): the red centroid within 2 m of it.
fn plate_offsets(img: &RgbaImage, roots: &[Vec2]) -> Vec<Vec2> {
    let px_of = |p: Vec2| Vec2::new(640.0 + p.x * 12.0, 360.0 + p.y * 12.0);
    roots
        .iter()
        .map(|&r| {
            let c = px_of(r);
            let (mut sum, mut w) = (Vec2::ZERO, 0.0);
            for y in (c.y as i32 - 24).max(0)..(c.y as i32 + 24).min(720) {
                for x in (c.x as i32 - 24).max(0)..(c.x as i32 + 24).min(1280) {
                    let p = px(img, x as u32, y as u32);
                    let at = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                    if p[0] > 100.0 && p[1] < 60.0 && at.distance(c) < 24.0 {
                        sum += at * p[0];
                        w += p[0];
                    }
                }
            }
            if w > 0.0 {
                (sum / w - c) / 12.0
            } else {
                Vec2::ZERO
            }
        })
        .collect()
}

/// G2: an omni og.WindMotor in still air pushes the copies around it
/// outward: within its radius the plates' offsets point away from it
/// (their mean direction's correlation with the outward vector above
/// 0.8), and copies past its reach stay put. Before G2 nothing read a
/// motor: every plate stood over its root.
#[test]
fn a_wind_motor_pushes_copies_away_from_it() {
    let (mut scene, roots) = plate_field();
    scene.wind = None;
    scene.wind_motors = vec![og_render::WindMotor {
        position: Vec3::ZERO,
        radius: 18.0,
        strength: 12.0,
        ..Default::default()
    }];
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    // A few ticks so the push spreads through the field.
    for tick in 0..20 {
        scene.tick = tick;
        r.render(&scene, &RenderTarget::Offscreen).expect("render");
    }
    let img = r.read_offscreen().expect("capture");
    keep("grass-motor-plates", &img);
    let offsets = plate_offsets(&img, &roots);
    let (mut dot, mut n, mut far) = (0.0, 0, 0.0f32);
    for (root, o) in roots.iter().zip(&offsets) {
        let d = root.length();
        if d > 3.0 && d < 14.0 && o.length() > 0.02 {
            dot += o.normalize().dot(root.normalize());
            n += 1;
        } else if d > 22.0 {
            far = far.max(o.length());
        }
    }
    assert!(n >= 4, "{n} plates inside the motor's reach moved");
    let corr = dot / n as f32;
    println!("motor: {n} plates, outward correlation {corr:.2}, farthest moved {far:.3} m outside");
    assert!(corr > 0.8, "the push is not outward: {corr:.2}");
    assert!(far < 0.1, "a plate past the motor's reach moved {far:.3} m");
}

/// G2: vegetation springs into the wind and settles after it: when the
/// wind stops, the next tick still draws the plates leaning (the spring
/// has not returned) and they come back over ticks rather than in one.
/// Before G2 the lean followed og.Wind each frame: the first tick without
/// wind drew every plate over its root.
#[test]
fn copies_settle_over_ticks_when_the_wind_stops() {
    let (mut scene, roots) = plate_field();
    let blowing = Wind {
        strength: 0.5,
        direction: Vec3::X,
        gust: 0.0,
        turbulence: 0.0,
        ..Wind::default()
    };
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut lean_at = |scene: &mut RenderScene, tick: u64, wind: Option<Wind>| {
        scene.tick = tick;
        scene.wind = wind;
        r.render(scene, &RenderTarget::Offscreen).expect("render");
        let o = plate_offsets(&r.read_offscreen().expect("capture"), &roots);
        o.iter().map(|o| o.x).sum::<f32>() / o.len().max(1) as f32
    };
    for tick in 0..30 {
        lean_at(&mut scene, tick, Some(blowing));
    }
    let blown = lean_at(&mut scene, 30, Some(blowing));
    let first = lean_at(&mut scene, 31, None);
    let later = lean_at(&mut scene, 45, None);
    println!("lean: blown {blown:.3} m, 1 tick after {first:.3} m, 15 ticks after {later:.3} m");
    assert!(
        blown > 0.8,
        "the wind leaned the plates {blown:.3} m, not about 1 m"
    );
    assert!(
        first > 0.5 * blown,
        "the plates snapped back in one tick: {first:.3}"
    );
    assert!(
        later < first,
        "the plates did not start to settle: {first:.3} then {later:.3}"
    );
}
