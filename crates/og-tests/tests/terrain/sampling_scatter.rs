use super::*;

/// `Terrain::scatter` places instances only where its layer shows, about
/// as many as the density and the layer's weight ask for, never beyond its
/// distance, at full size even where they fade; each keeps its cell's place
/// and turn as the camera moves.
#[test]
fn scatter_follows_its_layer_distance_and_stays_put() {
    let scene = ts::grass_scene();
    let terrain = scene.terrains[0].clone();
    let data = ts::island();
    let eye = scene.camera.position;
    let s = ts::grass_scatter();
    let placed = terrain.scatter(&data, &s, eye);
    let flat = |p: Vec3| Vec2::new(p.x - eye.x, p.z - eye.z).length();
    let fade_from = s.fade_from();
    let mut expected = 0.0;
    // The layer's weight summed over a 1 m grid inside the distance.
    let r = s.distance as i32;
    for dz in -r..=r {
        for dx in -r..=r {
            let p = Vec2::new(eye.x + dx as f32, eye.z + dz as f32);
            if p.distance(Vec2::new(eye.x, eye.z)) <= s.distance {
                expected += terrain.layer_weights(&data, p)[0] * s.density;
            }
        }
    }
    let n = placed.len() as f32;
    assert!(
        (n - expected).abs() < expected * 0.08,
        "{n} instances for an expected {expected:.0}"
    );
    for (_, m) in &placed {
        let (scale, _, at) = m.to_scale_rotation_translation();
        let local = Vec2::new(at.x, at.z);
        let w = terrain.layer_weights(&data, local)[0];
        assert!(
            w > 0.0,
            "an instance at {at} where the grass layer's weight is 0"
        );
        assert!(flat(at) <= s.distance + 1e-3, "{at} beyond the distance");
        assert!(
            (at.y - data.height(local)).abs() < 1e-3,
            "{at} off the ground"
        );
        assert!(
            scale.x <= s.scale[1] + 1e-4 && scale.x >= s.scale[0] - 1e-4,
            "scale {} at {:.1} m",
            scale.x,
            flat(at)
        );
    }
    // Near the edge, inside the fade, at full size (the renderer fades them).
    let edge: Vec<f32> = placed
        .iter()
        .map(|(_, m)| m.to_scale_rotation_translation())
        .filter(|(_, _, at)| flat(*at) > s.distance * 0.97)
        .map(|(sc, _, _)| sc.x)
        .collect();
    assert!(
        !edge.is_empty() && edge.iter().all(|&x| x >= s.scale[0] - 1e-4),
        "{edge:?}"
    );
    // The camera moves 7 m: instances well inside both circles are the same.
    let moved = eye + Vec3::new(5.0, 0.0, -5.0);
    let again: std::collections::HashMap<u32, Mat4> =
        terrain.scatter(&data, &s, moved).into_iter().collect();
    let mut same = 0;
    for (key, m) in &placed {
        let at = m.w_axis.truncate();
        let near = |e: Vec3| Vec2::new(at.x - e.x, at.z - e.z).length() < fade_from - 8.0;
        if near(eye) && near(moved) {
            assert_eq!(again.get(key), Some(m), "cell {key} moved");
            same += 1;
        }
    }
    assert!(same > 1_000, "{same} instances compared");
    // A seed places them elsewhere; the rock layer's stones stand on slopes.
    let other = terrain.scatter(&data, &TerrainScatter { seed: 4, ..s }, eye);
    assert_ne!(other.first().map(|x| x.1), placed.first().map(|x| x.1));
    // Under the mountains (300 m north of the meadow), where rock shows.
    let stones = terrain.scatter(&data, &ts::stone_scatter(), Vec3::new(-82.0, 0.0, -150.0));
    assert!(!stones.is_empty());
    for (_, m) in &stones {
        let at = m.w_axis.truncate();
        let n = data.normal(Vec2::new(at.x, at.z));
        assert!(
            n.y.acos().to_degrees() > 30.0 - 6.0,
            "a stone on a {:.0} degree slope",
            n.y.acos().to_degrees()
        );
        // Leaning with the ground: the stone's up is the normal.
        let up = m.transform_vector3(Vec3::Y).normalize();
        assert!(up.dot(n) > 0.999, "up {up} normal {n}");
    }
    // The sand track the control map paints south of the lake grows none.
    let track = Vec2::new(
        ts::LAKE.x - 40.0 + (400.0 - ts::LAKE.y) * 0.25 + 4.0f32.sin() * 30.0,
        400.0,
    );
    assert!(terrain.layer_weights(&data, track)[0] < 0.05);
    let there = terrain.scatter(&data, &s, Vec3::new(track.x, 0.0, track.y));
    assert!(there.len() > 1_000, "grass beside the track");
    assert!(
        there
            .iter()
            .all(|(_, m)| (m.w_axis.x - track.x).abs() > 2.5 || (m.w_axis.z - track.y).abs() > 2.5),
        "grass on the track"
    );
    // A layer the terrain does not have, or no density, places nothing.
    assert!(
        terrain
            .scatter(&data, &TerrainScatter { layer: 7, ..s }, eye)
            .is_empty()
    );
    assert!(
        terrain
            .scatter(&data, &TerrainScatter { density: 0.0, ..s }, eye)
            .is_empty()
    );
}

/// A 64 × 16 m strip with a 6 m bump at its centre, one red layer, drawn
/// against a magenta background without fog or shadows; looked at from
/// 120 m straight above, or from the side at the bump.
fn strip_scene(transform: Mat4, from_side: bool) -> RenderScene {
    let (cols, rows) = (65u32, 17u32);
    let size = Vec2::new(64.0, 16.0);
    let mut heights = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let x = -size.x * 0.5 + c as f32;
            let z = -size.y * 0.5 + r as f32;
            heights.push(6.0 * (-(x * x + z * z) / 8.0).exp());
        }
    }
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(TerrainData {
        size,
        columns: cols,
        rows,
        heights: Arc::from(heights),
        control: None,
        holes: None,
    });
    scene.terrains.push(Terrain {
        transform,
        layers: vec![TerrainLayer {
            color: Vec3::new(0.8, 0.05, 0.05),
            roughness: 1.0,
            ..Default::default()
        }],
        cast_shadows: false,
        ..Terrain::new(h)
    });
    scene.fog = None;
    scene.environment.background = Background::Color(Vec3::new(1.0, 0.0, 1.0));
    scene.post.tonemap = Tonemap::None;
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(0.3, 1.0, 0.2),
        cast_shadows: false,
        ..Default::default()
    };
    scene.camera = Camera {
        position: if from_side {
            Vec3::new(0.0, 6.0, 80.0)
        } else {
            Vec3::new(0.0, 120.0, 0.01)
        },
        target: if from_side {
            Vec3::new(0.0, 6.0, 0.0)
        } else {
            Vec3::ZERO
        },
        near: 0.5,
        far: 1000.0,
        projection: Projection::Perspective {
            fov_y_degrees: 55.0,
        },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

fn is_background(img: &RgbaImage, at: (u32, u32)) -> bool {
    let p = color_at(img, at);
    p[0] > 220.0 && p[1] < 40.0 && p[2] > 220.0
}

/// A terrain is drawn where its transform puts it: turned a quarter about
/// y its strip runs along world z, scaled twice along x it reaches twice as
/// far, scaled three times along y its bump stands three times as tall,
/// and a translation moves all of it; its flat part shades the same turned
/// or not (normals follow the transform). Scattered copies stand on the
/// transformed surface.
#[test]
#[ignore = "requires GPU"]
fn a_terrain_honours_its_transform() {
    let quarter = Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let cases: [(&str, Mat4, Vec3, bool); 7] = [
        (
            "identity along x",
            Mat4::IDENTITY,
            Vec3::new(24.0, 0.0, 0.0),
            true,
        ),
        (
            "identity not along z",
            Mat4::IDENTITY,
            Vec3::new(0.0, 0.0, 24.0),
            false,
        ),
        ("turned along z", quarter, Vec3::new(0.0, 0.0, 24.0), true),
        (
            "turned not along x",
            quarter,
            Vec3::new(24.0, 0.0, 0.0),
            false,
        ),
        (
            "scaled along x",
            Mat4::from_scale(Vec3::new(2.0, 1.0, 1.0)),
            Vec3::new(48.0, 0.0, 0.0),
            true,
        ),
        (
            "unscaled not that far",
            Mat4::IDENTITY,
            Vec3::new(48.0, 0.0, 0.0),
            false,
        ),
        (
            "moved",
            Mat4::from_translation(Vec3::new(0.0, 0.0, 30.0)),
            Vec3::new(20.0, 0.0, 30.0),
            true,
        ),
    ];
    let mut flat = Vec::new();
    for (label, transform, at, covered) in cases {
        let scene = strip_scene(transform, false);
        let (img, _) = render(&scene);
        keep(&format!("transform-{}", label.replace(' ', "-")), &img);
        let pixel = project(&scene.camera, at);
        assert_eq!(
            !is_background(&img, pixel),
            covered,
            "{label}: {at} at {pixel:?} is {:?}",
            color_at(&img, pixel)
        );
        if covered && label != "moved" {
            flat.push((label, color_at(&img, pixel)));
        }
    }
    for (label, c) in &flat {
        let d = (0..3)
            .map(|i| (c[i] - flat[0].1[i]).abs())
            .fold(0.0, f32::max);
        assert!(
            d < 6.0,
            "{label}: the flat ground shades {c:?}, not {:?}",
            flat[0].1
        );
    }
    // From the side: the bump's top (6 m) is under 12 m, three times taller is over it.
    let probe = Vec3::new(0.0, 12.0, 0.0);
    let plain = strip_scene(Mat4::IDENTITY, true);
    let (img, _) = render(&plain);
    assert!(
        is_background(&img, project(&plain.camera, probe)),
        "6 m bump under 12 m"
    );
    let tall = strip_scene(Mat4::from_scale(Vec3::new(1.0, 3.0, 1.0)), true);
    let (img, _) = render(&tall);
    keep("transform-tall", &img);
    assert!(
        !is_background(&img, project(&tall.camera, probe)),
        "18 m bump over 12 m"
    );
    // Scattered copies on the turned, raised and doubled strip lie on it.
    let placed_transform = Mat4::from_scale_rotation_translation(
        Vec3::new(2.0, 1.5, 2.0),
        glam::Quat::from_rotation_y(0.7),
        Vec3::new(5.0, 3.0, -4.0),
    );
    let scene = strip_scene(placed_transform, false);
    let terrain = scene.terrains[0].clone();
    let data = scene.terrain(terrain.terrain).cloned().expect("data");
    let s = TerrainScatter {
        density: 1.0,
        distance: 300.0,
        ..TerrainScatter::default()
    };
    let placed = terrain.scatter(&data, &s, Vec3::new(5.0, 50.0, -4.0));
    assert!(placed.len() > 1_000, "{} copies", placed.len());
    let back = placed_transform.inverse();
    for (_, m) in &placed {
        let local = back.transform_point3(m.w_axis.truncate());
        assert!(
            local.x.abs() <= 32.0 + 1e-3 && local.z.abs() <= 8.0 + 1e-3,
            "{local}"
        );
        let ground = data.height(Vec2::new(local.x, local.z));
        assert!(
            (local.y - ground).abs() < 1e-3,
            "{local} off the ground {ground}"
        );
    }
}
