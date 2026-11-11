use super::*;

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
#[ignore = "requires GPU"]
fn scattered_copies_are_drawn_from_chunks_and_shrink_over_the_fade() {
    let n = 257u32;
    let size = 1024.0f32;
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(TerrainData::flat(Vec2::splat(size), n, n));
    scene.world.terrains.push(Terrain {
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
    scene.world.scatters.push(og_render::ScatterDraw {
        terrain: 0,
        mesh: cube,
        material: red,
        scatter: s,
        cast_shadows: true,
        impostor: None,
        impostor_distance: 0.0,
    });
    scene.atmosphere.fog = None;
    scene.look.post.tonemap = Tonemap::None;
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.lighting.light.cast_shadows = false;
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
    let placed = scene.world.terrains[0].scatter(
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
            // solid square. Its red pixels within 8 px of its centre
            // (orthographic, from above: a square top, turned by the
            // copy's yaw, so convex) leave no hole inside any row's span.
            // The red region joined to the red pixel nearest the centre
            // (a neighbour's corner may reach into the box).
            let red = |dx: i32, dy: i32| red_at(cx + dx as f32, cy + dy as f32);
            let seed = (-8i32..=8)
                .flat_map(|dy| (-8i32..=8).map(move |dx| (dx, dy)))
                .filter(|&(dx, dy)| red(dx, dy))
                .min_by_key(|&(dx, dy)| dx * dx + dy * dy);
            let Some(seed) = seed else {
                faded_out += 1;
                continue;
            };
            let mut region = vec![seed];
            let mut next = 0;
            while next < region.len() {
                let (x, y) = region[next];
                next += 1;
                for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                    if nx.abs() <= 8 && ny.abs() <= 8 && red(nx, ny) && !region.contains(&(nx, ny))
                    {
                        region.push((nx, ny));
                    }
                }
            }
            let x0 = region.iter().map(|p| p.0).min().unwrap_or(0);
            let x1 = region.iter().map(|p| p.0).max().unwrap_or(0);
            let y0 = region.iter().map(|p| p.1).min().unwrap_or(0);
            let y1 = region.iter().map(|p| p.1).max().unwrap_or(0);
            let mut holes = 0;
            for dy in y0..=y1 {
                let row: Vec<i32> = (x0..=x1).filter(|&dx| region.contains(&(dx, dy))).collect();
                if let (Some(&a), Some(&b)) = (row.first(), row.last()) {
                    holes += (b - a + 1) as usize - row.len();
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
/// A scatter's chunks are kept while what placed them holds and built again
/// when it changes: a second frame builds none, another seed builds every
/// chunk again, back at the first seed its kept chunks are drawn again
/// (none built), and another slope range of the layer builds them again.
#[test]
#[ignore = "requires GPU"]
fn scatter_chunks_are_built_again_only_when_what_placed_them_changes() {
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(TerrainData::flat(Vec2::splat(1024.0), 257, 257));
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer::default()],
        cast_shadows: false,
        ..Terrain::new(h)
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let material = scene.add_material(Material::default());
    scene.world.scatters.push(og_render::ScatterDraw {
        terrain: 0,
        mesh: cube,
        material,
        scatter: TerrainScatter {
            density: 0.03,
            distance: 90.0,
            scale: [3.0, 3.0],
            seed: 5,
            ..TerrainScatter::default()
        },
        cast_shadows: false,
        impostor: None,
        impostor_distance: 0.0,
    });
    scene.lighting.light.cast_shadows = false;
    scene.camera = Camera {
        position: Vec3::new(0.0, 100.0, 0.001),
        target: Vec3::ZERO,
        near: 1.0,
        far: 500.0,
        projection: Projection::Orthographic { height: 200.0 },
        ..Default::default()
    };
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut built = |scene: &RenderScene| {
        r.render(scene, &RenderTarget::Offscreen)
            .expect("render")
            .scatter_built
    };
    let first = built(&scene);
    assert!(first >= 4, "{first}");
    assert_eq!(built(&scene), 0, "a second frame builds nothing");
    scene.world.scatters[0].scatter.seed = 6;
    assert_eq!(
        built(&scene),
        first,
        "another seed builds every chunk again"
    );
    scene.world.scatters[0].scatter.seed = 5;
    assert_eq!(
        built(&scene),
        0,
        "back at the first seed its chunks are kept"
    );
    scene.world.terrains[0].layers[0].slope = [0.0, 45.0];
    assert_eq!(
        built(&scene),
        first,
        "another slope range builds every chunk again"
    );
}

#[test]
#[ignore = "requires GPU"]
fn wind_sways_scattered_copies_downwind() {
    let n = 129u32;
    let data = TerrainData::flat(Vec2::splat(256.0), n, n);
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(data.clone());
    scene.world.terrains.push(Terrain {
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
    scene.world.scatters.push(og_render::ScatterDraw {
        terrain: 0,
        mesh: plate,
        material: red,
        scatter: s,
        cast_shadows: false,
        impostor: None,
        impostor_distance: 0.0,
    });
    scene.atmosphere.fog = None;
    scene.look.post.tonemap = Tonemap::None;
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.lighting.light.cast_shadows = false;
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
    let placed: Vec<Vec2> = scene.world.terrains[0]
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
        scene.atmosphere.wind = Some(steady);
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
    scene.atmosphere.wind = Some(Wind {
        strength: 0.0,
        ..steady
    });
    assert!(
        draw(&mut scene, 9).pixels == still.pixels,
        "strength 0 moved a copy"
    );
    scene.atmosphere.wind = Some(steady);
    scene.world.scatters[0].scatter.sway = 0.0;
    assert!(
        draw(&mut scene, 9).pixels == still.pixels,
        "sway 0 moved a copy"
    );
    scene.world.scatters[0].scatter.sway = 1.0;
    scene.atmosphere.wind = Some(Wind {
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
    // crosswind part: under a quarter of the downwind push, at least 1 m
    // (Metal: 0.26 m across 1.2 m downwind, 12°).
    for o in a.iter().chain(&b) {
        assert!(
            o.y.abs() < 0.25 * o.x.max(1.0) && o.x > -0.1 && o.x < 2.46,
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
    scene.world.scatters[0].material = dim;
    scene.look.post.motion_blur = Some(og_render::MotionBlur {
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
