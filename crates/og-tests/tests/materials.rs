//! The M6 material features (phase 2, owed row 31): the `materials`
//! reference board, and from frames: normals under a non-uniform scale,
//! normal maps in the vertex tangent frame (generated when a mesh has none,
//! mirrored uvs included), and clearcoat.

use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Environment, Exposure, MeshData, RenderScene,
    RenderTarget, Renderer, RgbaImage, StandardMaterial, TextureData, Vertex, shapes,
};
use og_tests::compare::{Tolerance, compare};
use og_tests::{gfx, scenes};
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx")
}

fn frame(scene: &RenderScene) -> RgbaImage {
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    renderer.read_offscreen().expect("capture")
}

fn luma(img: &RgbaImage, x: u32, y: u32) -> f32 {
    let p = &img.pixels[((y * img.width + x) * 4) as usize..][..3];
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

fn diff(a: &RgbaImage, b: &RgbaImage, channel: u8) -> (usize, f64) {
    let t = Tolerance {
        channel,
        ..Tolerance::default()
    };
    let c = compare(&a.pixels, &b.pixels, a.width, a.height, &t).expect("same size");
    (c.outliers, c.rmse)
}

/// One object on black, lit from the upper left, seen from +z.
fn studio() -> RenderScene {
    let mut scene = RenderScene::new();
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(-0.6, 0.7, 0.6),
        intensity: 3.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.environment = Environment {
        intensity: 0.15,
        background: Background::Color(Vec3::ZERO),
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 4.0),
        target: Vec3::ZERO,
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

#[test]
fn materials_matches_its_reference() {
    match gfx::check("materials", &out_dir()) {
        Ok(r) => println!("{r}"),
        Err(e) => panic!("{e}"),
    }
}

/// A unit sphere scaled (1.6, 0.6, 1.0) shades as an ellipsoid mesh with
/// its true normals (x/a², y/b², z/c²) under no transform: normals go
/// through the inverse transpose, not the model matrix.
#[test]
fn a_non_uniform_scale_bends_normals_correctly() {
    let scale = Vec3::new(1.6, 0.6, 1.0);
    let white = StandardMaterial {
        color: Vec3::splat(0.8),
        roughness: 0.4,
        ..Default::default()
    };
    let mut scaled = studio();
    let sphere = scaled.add_mesh(shapes::sphere(64, 48));
    let m = scaled.add_standard_material(white);
    scaled.add_instance(sphere, m, Mat4::from_scale(scale));

    let mut exact = studio();
    let mut mesh = shapes::sphere(64, 48);
    for v in &mut mesh.vertices {
        let p = Vec3::from(v.position) * scale;
        v.position = p.to_array();
        v.normal = (p / (scale * scale)).normalize().to_array();
    }
    let e = exact.add_mesh(mesh);
    let m = exact.add_standard_material(white);
    exact.add_instance(e, m, Mat4::IDENTITY);

    let (a, b) = (frame(&scaled), frame(&exact));
    let (off, rmse) = diff(&a, &b, 6);
    println!("{off} pixels more than 6 apart, RMSE {rmse:.5}");
    assert!(off < 200 && rmse < 0.002, "{off} pixels, RMSE {rmse:.5}");
}

/// A normal map of one constant tilt towards +u.
fn tilt_map() -> TextureData {
    let enc = |c: f32| ((c * 0.5 + 0.5) * 255.0).round() as u8;
    let n = Vec3::new(0.6, 0.0, 0.8);
    TextureData {
        width: 4,
        height: 4,
        rgba: [enc(n.x), enc(n.y), enc(n.z), 255].repeat(16),
        color_space: og_render::ColorSpace::Linear,
        mips: og_render::Mips::None,
    }
}

fn tilted_sphere(segments: u32, rings: u32, tangents: impl Fn(&mut MeshData)) -> RgbaImage {
    let mut scene = studio();
    let mut mesh = shapes::sphere(segments, rings);
    tangents(&mut mesh);
    let mesh = scene.add_mesh(mesh);
    let map = scene.add_texture(tilt_map());
    let m = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.8),
        roughness: 0.5,
        normalMap: map.slot(),
        ..Default::default()
    });
    scene.add_instance(mesh, m, Mat4::from_rotation_y(0.4));
    frame(&scene)
}

/// The tangents generated at upload are the sphere's own (along +u, the
/// bitangent along +v): the frame is the one drawn with analytic tangents.
#[test]
fn generated_tangents_match_the_surface() {
    let generated = tilted_sphere(48, 32, |_| {});
    let analytic = tilted_sphere(48, 32, |m| {
        for v in &mut m.vertices {
            // shapes::sphere: u runs with the angle about +y.
            let p = Vec3::from(v.position);
            let t = Vec3::new(-p.z, 0.0, p.x).normalize_or(Vec3::X);
            // The bitangent up the image: towards -v, here +y.
            let u_dir = if Vec3::from(v.normal).cross(t).dot(Vec3::Y) >= 0.0 {
                1.0
            } else {
                -1.0
            };
            v.tangent = t.extend(u_dir).to_array();
        }
    });
    let (off, rmse) = diff(&generated, &analytic, 4);
    println!("{off} pixels more than 4 apart, RMSE {rmse:.5}");
    assert!(off < 300 && rmse < 0.003, "{off} pixels, RMSE {rmse:.5}");
}

/// A coarse sphere (16 × 12) with a strongly tilted map shades smoothly in
/// the interpolated vertex frame, as a fine one does; a frame from screen
/// derivatives is constant per triangle and shows the facets.
#[test]
fn normal_maps_follow_the_vertex_tangent_frame() {
    let coarse = tilted_sphere(16, 12, |_| {});
    // Facets: big steps between neighbouring pixels inside the silhouette.
    let steps = |img: &RgbaImage| {
        let mut n = 0;
        for y in 200..520 {
            for x in 480..800 {
                let here = luma(img, x, y);
                if here < 20.0 || luma(img, x + 1, y) < 20.0 || luma(img, x, y + 1) < 20.0 {
                    continue;
                }
                let d = (here - luma(img, x + 1, y))
                    .abs()
                    .max((here - luma(img, x, y + 1)).abs());
                if d > 6.0 {
                    n += 1;
                }
            }
        }
        n
    };
    let fine = tilted_sphere(64, 48, |_| {});
    let (c, f) = (steps(&coarse), steps(&fine));
    println!("steps: coarse {c}, fine {f}");
    assert!(
        c < f + 150,
        "the coarse sphere shows facets ({c} steps against {f})"
    );
}

/// Mirrored uvs (u running the other way) flip the bitangent's sign, so
/// the map still tilts towards +u on the surface: the mirrored quad is the
/// plain one mirrored.
#[test]
fn mirrored_uvs_keep_the_map_the_right_way_round() {
    let quad = |mirror: bool| {
        let mut scene = studio();
        scene.light.direction_to_light = Vec3::new(1.0, 0.2, 0.6);
        let mut mesh = shapes::plane(2.0, 1.0);
        for v in &mut mesh.vertices {
            if mirror {
                v.uv[0] = 1.0 - v.uv[0];
            }
        }
        let mesh = scene.add_mesh(mesh);
        let map = scene.add_texture(tilt_map());
        let m = scene.add_standard_material(StandardMaterial {
            color: Vec3::splat(0.8),
            roughness: 0.9,
            normalMap: map.slot(),
            ..Default::default()
        });
        scene.add_instance(mesh, m, Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2));
        let img = frame(&scene);
        luma(&img, 640, 360)
    };
    let (plain, mirrored) = (quad(false), quad(true));
    println!("lit towards +x: plain {plain:.0}, mirrored {mirrored:.0}");
    // The light comes from +x: the map tilts the plain quad towards it
    // (+u is +x) and the mirrored one away (+u is -x).
    assert!(
        plain > mirrored + 25.0,
        "plain {plain:.0}, mirrored {mirrored:.0}"
    );
}

/// glTF's convention: a map's green tilts the surface up the image (uvs
/// run down it). A quad whose v runs down the screen, lit from above,
/// is brighter with green up than with green down.
#[test]
fn a_normal_maps_green_points_up_the_image() {
    let lit = |green: f32| {
        let mut scene = studio();
        scene.light.direction_to_light = Vec3::new(0.0, 1.0, 0.5);
        let mesh = scene.add_mesh(shapes::plane(2.0, 1.0));
        let enc = |c: f32| ((c * 0.5 + 0.5) * 255.0).round() as u8;
        let n = Vec3::new(0.0, green, 0.8).normalize();
        let map = scene.add_texture(TextureData {
            width: 4,
            height: 4,
            rgba: [enc(n.x), enc(n.y), enc(n.z), 255].repeat(16),
            color_space: og_render::ColorSpace::Linear,
            mips: og_render::Mips::None,
        });
        let m = scene.add_standard_material(StandardMaterial {
            color: Vec3::splat(0.8),
            roughness: 0.9,
            normalMap: map.slot(),
            ..Default::default()
        });
        // shapes::plane's v runs along +z; turned to face the camera it runs down the screen.
        scene.add_instance(mesh, m, Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2));
        luma(&frame(&scene), 640, 360)
    };
    let (up, down) = (lit(0.6), lit(-0.6));
    println!("lit from above: green up {up:.0}, green down {down:.0}");
    assert!(up > down + 25.0, "green up {up:.0}, green down {down:.0}");
}

/// A glossy coat over a rough red base adds a sharp highlight the base
/// alone does not have, sharper the smoother the coat; no coat is the
/// plain material.
#[test]
fn clearcoat_adds_a_glossy_layer() {
    let ball = |coat: f32, coat_roughness: f32| {
        let mut scene = studio();
        let sphere = scene.add_mesh(shapes::sphere(64, 48));
        let m = scene.add_standard_material(StandardMaterial {
            color: Vec3::new(0.7, 0.06, 0.05),
            roughness: 0.8,
            clearcoat: coat,
            clearcoatRoughness: coat_roughness,
            ..Default::default()
        });
        scene.add_instance(sphere, m, Mat4::IDENTITY);
        frame(&scene)
    };
    let peak = |img: &RgbaImage| {
        let mut best = 0.0f32;
        for y in 160..560 {
            for x in 440..840 {
                best = best.max(luma(img, x, y));
            }
        }
        best
    };
    let (bare, glossy, rough) = (ball(0.0, 0.0), ball(1.0, 0.03), ball(1.0, 0.5));
    let (pb, pg, pr) = (peak(&bare), peak(&glossy), peak(&rough));
    println!("highlight: bare {pb:.0}, glossy coat {pg:.0}, rough coat {pr:.0}");
    assert!(pg > pb + 60.0, "the glossy coat adds a highlight");
    assert!(pg > pr + 20.0, "a smoother coat is sharper");
    let mut plain = studio();
    let sphere = plain.add_mesh(shapes::sphere(64, 48));
    let m = plain.add_standard_material(StandardMaterial {
        color: Vec3::new(0.7, 0.06, 0.05),
        roughness: 0.8,
        ..Default::default()
    });
    plain.add_instance(sphere, m, Mat4::IDENTITY);
    assert_eq!(frame(&plain).pixels, bare.pixels);
}

/// The public generator: a quad's tangent runs along +u, its sign follows
/// the uvs' handedness, and a tangent already set is kept.
#[test]
fn generate_tangents_follows_the_uvs() {
    let quad = |u_flip: bool| {
        let uv = |u: f32, v: f32| [if u_flip { 1.0 - u } else { u }, v];
        let vertex = |p: [f32; 3], t: [f32; 2]| Vertex {
            position: p,
            normal: [0.0, 0.0, 1.0],
            uv: t,
            ..Default::default()
        };
        let mut m = MeshData {
            vertices: vec![
                vertex([0.0, 0.0, 0.0], uv(0.0, 1.0)),
                vertex([1.0, 0.0, 0.0], uv(1.0, 1.0)),
                vertex([1.0, 1.0, 0.0], uv(1.0, 0.0)),
                vertex([0.0, 1.0, 0.0], uv(0.0, 0.0)),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        };
        m.generate_tangents();
        m
    };
    // +u is +x and +v is -y, so the bitangent (up the image, towards -v)
    // is +y = cross(+z, +x): w 1.
    let plain = quad(false);
    for v in &plain.vertices {
        assert_eq!(v.tangent, [1.0, 0.0, 0.0, 1.0]);
    }
    let mirrored = quad(true);
    for v in &mirrored.vertices {
        assert_eq!(v.tangent, [-1.0, 0.0, 0.0, -1.0]);
    }
    let mut kept = quad(false);
    kept.vertices[0].tangent = [0.0, 1.0, 0.0, 1.0];
    kept.vertices[1].tangent = [0.0; 4];
    kept.generate_tangents();
    assert_eq!(kept.vertices[0].tangent, [0.0, 1.0, 0.0, 1.0]);
    assert_eq!(kept.vertices[1].tangent, [1.0, 0.0, 0.0, 1.0]);
}

/// ground-grid (meridian-loop #15): og.render.planeGeometry's `uvRepeat`
/// tiles a texture across a 1,600 m ground, a grid line every 10 m, the
/// posts on them every 10 m too. Without the repeat (1) the one tile is
/// stretched over the whole plane and no line is in view.
#[test]
fn ground_grid_repeats_every_10_m() {
    let report = match gfx::check("ground-grid", &out_dir()) {
        Ok(r) => r,
        Err(e) => panic!("{e}"),
    };
    println!("{report}");
    let tiled = frame(&scenes::ground_grid(160.0));
    let stretched = frame(&scenes::ground_grid(1.0));
    let (outliers, rmse) = diff(&tiled, &stretched, 24);
    assert!(
        outliers > (tiled.width * tiled.height / 20) as usize,
        "the repeat draws lines the stretched tile has not: {outliers} outliers, RMSE {rmse}"
    );
}

/// The outline's cyan: bright green and blue, well above the red.
fn is_cyan(p: &[u8]) -> bool {
    p[1] > 180 && p[2] > 180 && p[1] as i32 - p[0] as i32 > 40
}

/// Pixels of the outline's cyan in the ground-only part of
/// `stylise-outline-ground`'s frame: its left 40 %,
/// from 6 rows under the horizon (the first row from the top whose left
/// column is no longer the background) to the bottom.
fn cyan_on_the_ground(img: &RgbaImage) -> (usize, usize) {
    let px = |x: u32, y: u32| &img.pixels[((y * img.width + x) * 4) as usize..][..3];
    let horizon = (0..img.height)
        .find(|&y| px(4, y) != px(4, 0))
        .expect("the ground is in view");
    let (mut cyan, mut all) = (0, 0);
    for y in (horizon + 6)..img.height {
        for x in 0..img.width * 2 / 5 {
            let p = px(x, y);
            all += 1;
            if is_cyan(p) {
                cyan += 1;
            }
        }
    }
    (cyan, all)
}

/// stylise-outline-ground (meridian-loop #7): neon-vector's outline over a
/// 1,600 m ground at a grazing angle is drawn only where something ends:
/// with `outlineDistance` 120 the ground under a chase camera was one cyan
/// wash, at a constant 1.5 px a pale band some 30 px tall along the
/// horizon. Now neither: no cyan on the plane-only part of the ground. The
/// barriers keep their lines.
#[test]
fn a_large_ground_keeps_its_outline_only_where_it_ends() {
    let report = match gfx::check("stylise-outline-ground", &out_dir()) {
        Ok(r) => r,
        Err(e) => panic!("{e}"),
    };
    println!("{report}");
    for (width, distance) in [(1.4, 120.0), (1.5, 0.0)] {
        let img = frame(&og_tests::stylise_scenes::outline_ground_scene(
            width, distance,
        ));
        let (cyan, all) = cyan_on_the_ground(&img);
        assert!(
            cyan * 1000 < all,
            "outlineDistance {distance}: {cyan} of {all} ground pixels are outline"
        );
        let barriers = img
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| is_cyan(&p[..]))
            .count();
        assert!(
            barriers > 2000,
            "the barriers are lined: {barriers} cyan pixels"
        );
    }
}
