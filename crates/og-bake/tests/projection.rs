//! Projection texturing through og-bake's public API: what lands in which
//! texel, checked against independent ray casts, never against the code
//! that projects.

use glam::{Vec2, Vec3};
use og_bake::projection::composite::{ViewMasks, finish};
use og_bake::projection::{
    GBuffer, Level, Paint, Texels, ViewKind, ViewSpec, fill_unpainted, project,
};

/// A 10 m floor (uv: its x, z over [-5, 5] onto the left half of the
/// texture) and a 2 m square wall standing on it at z = 0 facing +z (uv:
/// the right half), wound so both face up / towards +z.
fn scene() -> Level {
    labelled_scene("floor", "wall")
}

fn labelled_scene(floor_name: &str, wall_name: &str) -> Level {
    let floor = [(-5.0, -5.0), (5.0, -5.0), (5.0, 5.0), (-5.0, 5.0)];
    let mut pos: Vec<Vec3> = floor.iter().map(|&(x, z)| Vec3::new(x, 0.0, z)).collect();
    let mut uvs: Vec<Vec2> = floor
        .iter()
        .map(|&(x, z)| Vec2::new((x + 5.0) / 20.0, (z + 5.0) / 10.0))
        .collect();
    let wall = [(-1.0, 0.0), (1.0, 0.0), (1.0, 2.0), (-1.0, 2.0)];
    pos.extend(wall.iter().map(|&(x, y)| Vec3::new(x, y, 0.0)));
    uvs.extend(
        wall.iter()
            .map(|&(x, y)| Vec2::new(0.55 + (x + 1.0) * 0.2, 0.1 + (2.0 - y) * 0.4)),
    );
    let indices = vec![0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7];
    Level::new(
        pos,
        Vec::new(),
        uvs,
        indices,
        vec![0, 0, 1, 1],
        vec![floor_name.into(), wall_name.into()],
    )
}

/// Every pixel's colour names the pixel: (x, y) in two channels each, 12 bits.
fn index_image(size: u32) -> Vec<[u8; 4]> {
    (0..size * size)
        .map(|i| {
            let (x, y) = (i % size, i / size);
            [
                (x & 255) as u8,
                (y & 255) as u8,
                ((x >> 8) | ((y >> 8) << 4)) as u8,
                255,
            ]
        })
        .collect()
}

fn decode(c: [u8; 3]) -> (u32, u32) {
    (
        c[0] as u32 | ((c[2] as u32 & 15) << 8),
        c[1] as u32 | ((c[2] as u32 >> 4) << 8),
    )
}

fn camera() -> ViewSpec {
    ViewSpec::looking(
        Vec3::new(0.0, 3.0, 7.0),
        Vec3::new(0.0, 0.0, -2.0),
        70.0,
        ViewKind::Ground,
    )
}

#[test]
fn every_written_texel_took_the_pixel_that_sees_it_and_hidden_texels_stay_unpainted() {
    let level = scene();
    let size = 512;
    let texels = Texels::rasterise(&level, size, 2);
    let mut paint = Paint::new(&texels);
    let vs = 256;
    let g = GBuffer::cast(&level, camera(), vs, 2);
    let image = index_image(vs);
    // No silhouette band: the decoded pixel must be exact enough to check.
    let edge = vec![0u8; (vs * vs) as usize];
    let p = project(&texels, &mut paint, &g, &edge, &image, 0.4, 2);
    assert!(p.written > 10_000, "only {} texels written", p.written);

    let (mut checked, mut hidden_checked) = (0, 0);
    for (i, t) in paint.texels.iter().enumerate() {
        let pos = texels.position[i];
        // Independent visibility: does the ray from the camera reach this texel's point first?
        let to = pos - camera().position;
        // Hidden with a margin: the point and its neighbours 5 cm away all behind the wall
        // (a texel straddling the silhouette may take either side).
        let blocked = [
            Vec3::ZERO,
            Vec3::X * 0.05,
            -Vec3::X * 0.05,
            Vec3::Z * 0.05,
            -Vec3::Z * 0.05,
        ]
        .iter()
        .all(|&o| {
            let d = pos + o - camera().position;
            level
                .hit(camera().position, d.normalize(), d.length() - 0.05)
                .is_some()
        });
        let in_frame = camera().project(pos, vs).is_some_and(|q| {
            q.x >= 1.0 && q.y >= 1.0 && q.x < vs as f32 - 1.0 && q.y < vs as f32 - 1.0
        });
        if blocked && in_frame && texels.normal[i].dot(-to) > 0.1 {
            hidden_checked += 1;
            assert_eq!(
                t.quality, 0.0,
                "texel {} behind the wall was painted",
                texels.index[i]
            );
        }
        if t.quality == 0.0 {
            continue;
        }
        // The pixel it took its colour from must see this texel: the renderer's own uv there.
        let (x, y) = decode(t.rgb);
        let hit = g.hits[(y * vs + x) as usize].expect("painted from a sky pixel");
        let texel = Vec2::new(
            (texels.index[i] % size) as f32 + 0.5,
            (texels.index[i] / size) as f32 + 0.5,
        );
        // How many texels one pixel spans there, from the renderer's uvs at its neighbours
        // (bilinear sampling may take a neighbour's colour): two footprints, at least 2 texels.
        let span = [(1i32, 0i32), (0, 1), (-1, 0), (0, -1)]
            .iter()
            .filter_map(|&(dx, dy)| {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if nx < 0 || ny < 0 || nx >= vs as i32 || ny >= vs as i32 {
                    return None;
                }
                g.hits[(ny as u32 * vs + nx as u32) as usize]
                    .map(|n| ((n.uv - hit.uv) * size as f32).length())
            })
            .fold(0.0f32, f32::max);
        let footprint = (2.0 * span).max(2.0);
        assert!(
            (hit.uv * size as f32 - texel).length() <= footprint,
            "texel {texel} painted from pixel ({x}, {y}) showing uv {}",
            hit.uv * size as f32
        );
        checked += 1;
    }
    assert!(
        hidden_checked > 500,
        "the scene should hide floor texels behind the wall ({hidden_checked})"
    );
    assert_eq!(checked, p.written);
}

#[test]
fn finish_keeps_known_pixels_and_leaves_out_what_the_model_left_grey() {
    let (w, h) = (64usize, 64usize);
    // Left half known (green texture), right half unknown clay (grey 140).
    let state: Vec<[u8; 4]> = (0..w * h)
        .map(|i| {
            if i % w < 32 {
                [40, 160, 40, 255]
            } else {
                [140, 140, 140, 255]
            }
        })
        .collect();
    let masks = ViewMasks {
        geometry: vec![1; w * h],
        known: (0..w * h).map(|i| (i % w < 32) as u8).collect(),
    };
    // The model repaints everything brown, except the bottom rows of the unknown half it hands back unchanged.
    let painted: Vec<[u8; 4]> = (0..w * h)
        .map(|i| {
            if i / w >= 48 && i % w >= 32 {
                state[i]
            } else {
                [120, 80, 30, 255]
            }
        })
        .collect();
    let f = finish(&state, &painted, &masks, None, w, h);
    // Known pixels well away from the seam stay exactly as they were.
    for y in 0..h {
        for x in 0..8 {
            assert_eq!(
                f.image[y * w + x],
                state[y * w + x],
                "known pixel ({x}, {y}) changed"
            );
        }
    }
    // The grey block the model did not paint is transparent: it must not project.
    for y in 52..h {
        for x in 40..w {
            assert_eq!(
                f.image[y * w + x][3],
                0,
                "unpainted clay at ({x}, {y}) would project"
            );
        }
    }
    // Painted unknown pixels are opaque and no longer grey.
    let c = f.image[10 * w + 50];
    assert_eq!(c[3], 255);
    assert_ne!([c[0], c[1], c[2]], [140, 140, 140]);
    // 16 of the unknown half's 32 rows, minus the blur's eroded border.
    assert!(f.left > 0.15 && f.left <= 0.25, "left {}", f.left);
}

/// An orthographic camera 20 m away, looking 30° down at the scene, 4 m
/// to its image's edge.
fn ortho_camera() -> ViewSpec {
    ViewSpec {
        position: Vec3::new(0.0, 10.0, 17.32),
        yaw: 0.0,
        pitch: (-30f32).to_radians(),
        fov: 0.0,
        ortho: 4.0,
        kind: ViewKind::Ground,
    }
}

/// Under an orthographic camera a texel lands at its offset across the view
/// over `ortho`, at any depth (a perspective camera would shrink the far
/// floor), and the floor just behind the wall stays unpainted though the
/// camera is far away.
#[test]
fn an_orthographic_view_paints_each_texel_from_its_parallel_ray() {
    let level = scene();
    let size = 256;
    let texels = Texels::rasterise(&level, size, 2);
    let mut paint = Paint::new(&texels);
    let vs = 256;
    let cam = ortho_camera();
    let g = GBuffer::cast(&level, cam, vs, 2);
    let edge = vec![0u8; (vs * vs) as usize];
    let p = project(&texels, &mut paint, &g, &edge, &index_image(vs), 0.4, 2);
    assert!(p.written > 2_000, "only {} texels written", p.written);
    let (f, r, _) = cam.axes();
    let (mut far, mut hidden) = (0, 0);
    for (i, t) in paint.texels.iter().enumerate() {
        let pos = texels.position[i];
        // Hidden along the view's own direction, with a 5 cm margin.
        let blocked = [
            Vec3::ZERO,
            Vec3::X * 0.05,
            -Vec3::X * 0.05,
            Vec3::Z * 0.05,
            -Vec3::Z * 0.05,
        ]
        .iter()
        .all(|&o| level.hit(pos + o - f * 0.05, -f, 100.0).is_some());
        if blocked && pos.y < 0.01 {
            hidden += 1;
            assert_eq!(
                t.quality, 0.0,
                "floor texel {} behind the wall was painted",
                texels.index[i]
            );
        }
        if t.quality == 0.0 {
            continue;
        }
        let (x, _) = decode(t.rgb);
        let expect = ((pos - cam.position).dot(r) / cam.ortho + 1.0) * 0.5 * vs as f32;
        assert!(
            (x as f32 + 0.5 - expect).abs() <= 1.5,
            "texel at {pos} took pixel column {x}, not {expect}"
        );
        far += usize::from(pos.z < -3.0);
    }
    assert!(far > 100, "the far floor should be painted ({far})");
    assert!(hidden > 200, "the wall should hide floor texels ({hidden})");
}

/// A texel no view painted takes the colour of the painted texel nearest
/// it in the world, found here by brute force.
#[test]
fn fill_takes_the_nearest_painted_texel_in_the_world() {
    let level = scene();
    let size = 128;
    let texels = Texels::rasterise(&level, size, 2);
    let mut paint = Paint::new(&texels);
    // Paint the wall alone, each texel a colour of its own place.
    for (i, t) in paint.texels.iter_mut().enumerate() {
        let p = texels.position[i];
        if p.y > 0.01 {
            t.rgb = [(p.x * 100.0 + 128.0) as u8, (p.y * 100.0) as u8, 7];
            t.quality = 1.0;
        }
    }
    let before = paint.clone();
    let painted: Vec<usize> = (0..texels.len())
        .filter(|&i| before.texels[i].quality > 0.0)
        .collect();
    let filled = fill_unpainted(&texels, &mut paint, 2);
    assert_eq!(filled, texels.len() - painted.len());
    for i in (0..texels.len())
        .step_by(37)
        .filter(|&i| before.texels[i].quality == 0.0)
    {
        let p = texels.position[i];
        let d = |j: usize| (texels.position[j] - p).length_squared();
        let best = painted
            .iter()
            .copied()
            .min_by(|&a, &b| d(a).total_cmp(&d(b)).then(a.cmp(&b)))
            .unwrap();
        assert_eq!(
            paint.texels[i].rgb, before.texels[best].rgb,
            "texel {i} at {p}"
        );
        assert!(paint.texels[i].quality > 0.0);
    }
    // A view replaces a filled texel.
    assert!(paint.texels.iter().all(|t| t.quality <= 1.0));
}
