use super::*;
use crate::tools::export::{Material, Part};
use crate::tools::image::Texture;

/// An `n`-sided prism of radius 10 and height 10 whose sides are one
/// surface (as a loft's are), its caps two more.
fn prism(n: u32, at: DVec3) -> Solid {
    let mut s = Solid::default();
    for z in [0.0, 10.0] {
        for k in 0..n {
            let (sn, cs) = trig::sin_cos(trig::TAU * f64::from(k) / f64::from(n));
            s.push_vertex(at + DVec3::new(10.0 * cs, 10.0 * sn, z));
        }
    }
    for k in 0..n {
        let (a, b) = (k, (k + 1) % n);
        s.push_triangle([a, b, b + n], 0, &[]);
        s.push_triangle([a, b + n, a + n], 0, &[]);
    }
    for k in 1..n - 1 {
        s.push_triangle([0, k + 1, k], 1, &[]);
        s.push_triangle([n, n + k, n + k + 1], 2, &[]);
    }
    s
}

fn sides_flat(n: u32) -> bool {
    let s = prism(n, DVec3::ZERO);
    let crease = trig::cos(trig::to_radians(CREASE_DEGREES));
    let normals = corner_normals(&s, crease);
    s.triangles.iter().zip(&normals).all(|(t, c)| {
        let [a, b, d] = t.map(|v| s.positions[v as usize]);
        let f = face_normal(a, b, d);
        c.iter().all(|n| (*n - f).length() < 1e-12)
    })
}

#[test]
fn facets_past_the_crease_are_flat_and_finer_ones_smooth() {
    // A hexagon's facets meet at 60°: each shaded flat. A 24-gon's
    // meet at 15°: shaded smooth across them.
    assert!(sides_flat(6));
    assert!(sides_flat(8));
    assert!(!sides_flat(12));
    assert!(!sides_flat(24));
}

/// A square 20 across, flat at height `z`.
fn quad(z: f64) -> Solid {
    let mut s = Solid::default();
    for (x, y) in [(-10.0, -10.0), (10.0, -10.0), (10.0, 10.0), (-10.0, 10.0)] {
        s.push_vertex(DVec3::new(x, y, z));
    }
    s.push_triangle([0, 1, 2], 0, &[]);
    s.push_triangle([0, 2, 3], 0, &[]);
    s
}

fn part<'a>(name: &str, s: &'a Solid, rgb: [f64; 3], opacity: f64) -> Part<'a> {
    Part {
        material: Material {
            color: Some(rgb),
            opacity: Some(opacity),
            ..Default::default()
        },
        ..Part::new(name, s)
    }
}

fn centre(parts: Vec<Part>) -> [u8; 3] {
    let scene = Scene {
        name: "t".into(),
        parts,
    };
    let d = Preview {
        width: 32,
        views: 1,
        elevation: Some(trig::PI / 2.0 - 1e-3),
        no_edges: true,
        no_ground: true,
        ..Default::default()
    };
    let im = &preview(&scene, &d).unwrap()[0];
    let i = ((16 * 32 + 16) * 4) as usize;
    [im.rgba[i], im.rgba[i + 1], im.rgba[i + 2]]
}

#[test]
fn transparent_parts_blend_back_to_front_whatever_their_order() {
    // Seen from above: a red quad at z 0, blue at 5 (opacity 0.5),
    // green at 10 (opacity 0.5).
    let (red, blue, green) = (quad(0.0), quad(5.0), quad(10.0));
    let one = centre(vec![
        part("red", &red, [1.0, 0.0, 0.0], 1.0),
        part("blue", &blue, [0.0, 0.0, 1.0], 0.5),
        part("green", &green, [0.0, 1.0, 0.0], 0.5),
    ]);
    let other = centre(vec![
        part("green", &green, [0.0, 1.0, 0.0], 0.5),
        part("blue", &blue, [0.0, 0.0, 1.0], 0.5),
        part("red", &red, [1.0, 0.0, 0.0], 1.0),
    ]);
    assert_eq!(one, other);
    // Every colour shows through: green over blue over red.
    assert!(one[0] > 0 && one[1] > 0 && one[2] > 0, "{one:?}");
    // Blue drawn over green (the wrong order) would leave more blue.
    assert!(one[1] > one[2], "{one:?}");
    // Opaque, the nearest hides the rest.
    let solid = centre(vec![
        part("red", &red, [1.0, 0.0, 0.0], 1.0),
        part("green", &green, [0.0, 1.0, 0.0], 1.0),
    ]);
    assert_eq!((solid[0], solid[2]), (0, 0));
}

/// A flat quad 40 × 100 in z = 0, x across and y away from a camera at
/// −Y, UVs u = x, v = y over the whole quad.
fn floor_quad() -> Solid {
    let mut s = Solid::default();
    for (x, y) in [(-20.0, 0.0), (20.0, 0.0), (20.0, 100.0), (-20.0, 100.0)] {
        s.push_vertex(DVec3::new(x, y, 0.0));
    }
    let uv = |x: f64, y: f64| DVec2::new(x, y);
    s.push_triangle_uv(
        [0, 1, 2],
        0,
        &[],
        [uv(0.0, 0.0), uv(1.0, 0.0), uv(1.0, 1.0)],
    );
    s.push_triangle_uv(
        [0, 2, 3],
        0,
        &[],
        [uv(0.0, 0.0), uv(1.0, 1.0), uv(0.0, 1.0)],
    );
    s
}

/// A PNG `w` × `h` from a function of the texel (x, y), rows top down.
fn map(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Texture {
    let rgba: Vec<u8> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .flat_map(|(x, y)| f(x, y))
        .collect();
    Texture::from_bytes(
        "t.png",
        crate::tools::png(&Image {
            width: w,
            height: h,
            rgba,
        }),
    )
}

fn one_view(s: &Solid, material: Material, d: Preview) -> Views {
    let scene = Scene {
        name: "t".into(),
        parts: vec![Part {
            material,
            ..Part::new("p", s)
        }],
    };
    views(
        &scene,
        &Preview {
            views: 1,
            no_edges: true,
            no_ground: true,
            ..d
        },
    )
    .unwrap()
}

fn rgb_at(im: &Image, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * im.width + x) * 4) as usize;
    [im.rgba[i], im.rgba[i + 1], im.rgba[i + 2]]
}

/// A picture all round lights the scene: a matte floor under a
/// blue sky is bluer than under the plain light, and a mirror
/// floor shows the sky itself. A Radiance file reads as the
/// picture it holds.
#[test]
fn a_picture_all_round_lights_the_scene() {
    let s = floor_quad();
    // The upper half blue, the lower half red.
    let texels: Vec<[f32; 3]> = (0..64 * 32)
        .map(|i| {
            if i / 64 < 16 {
                [0.1, 0.2, 1.0]
            } else {
                [1.0, 0.1, 0.1]
            }
        })
        .collect();
    let env = Environment::new(64, 32, &texels).unwrap();
    let lit = |material: Material, env: Option<&Environment>| {
        let scene = Scene {
            name: "t".into(),
            parts: vec![Part {
                material,
                ..Part::new("p", &s)
            }],
        };
        let d = Preview {
            views: 1,
            no_edges: true,
            no_ground: true,
            ..Default::default()
        };
        rgb_at(&views_in(&scene, &d, env).unwrap().images[0], 128, 128)
    };
    let grey = Material {
        color: Some([0.5; 3]),
        ..Default::default()
    };
    let plain = lit(grey.clone(), None);
    assert_eq!(plain[0], plain[2]);
    let [r, _, b] = lit(grey.clone(), Some(&env));
    assert!(b > r + 15, "{r} {b}");
    let matte = (r, b);
    let mirror = Material {
        color: Some([1.0; 3]),
        metalness: 1.0,
        roughness: Some(0.0),
        ..Default::default()
    };
    let [r, _, b] = lit(mirror, Some(&env));
    assert!(b > 150 && r < 110, "{r} {b}");
    // The same picture as a Radiance file, flat: each texel a
    // mantissa a channel and one exponent.
    let mut hdr = b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y 32 +X 64\n".to_vec();
    for p in &texels {
        // Exponent 129: a mantissa of m is m / 128.
        hdr.extend_from_slice(&[
            (p[0] * 128.0) as u8,
            (p[1] * 128.0) as u8,
            (p[2] * 128.0) as u8,
            129,
        ]);
    }
    let read = Environment::from_hdr(&hdr).unwrap();
    let [r2, _, b2] = lit(grey, Some(&read));
    assert!(
        r2.abs_diff(matte.0) <= 3 && b2.abs_diff(matte.1) <= 3,
        "{r2} {b2} for {matte:?}"
    );
    assert!(Environment::from_hdr(b"#?RADIANCE\n\n+Y 2 +X 2\n").is_err());
    assert!(Environment::new(2, 2, &[[0.0; 3]; 4]).is_none());
}

/// The material extensions each change what is drawn: a clear coat
/// reflects over a matte surface, a sheen adds its colour towards
/// the rim, what transmits shows what is behind it in its own
/// colour, and a thin film colours a metal's reflection.
#[test]
fn the_material_extensions_show() {
    let s = floor_quad();
    let at = |m: Material| rgb_at(&one_view(&s, m, Preview::default()).images[0], 128, 128);
    let grey = Material {
        color: Some([0.5; 3]),
        ..Default::default()
    };
    let plain = at(grey.clone());
    let coated = at(Material {
        clearcoat: 1.0,
        ..grey.clone()
    });
    assert!(coated[0] > plain[0] + 3, "{coated:?} {plain:?}");
    let [r, g, _] = at(Material {
        sheen: [1.0, 0.0, 0.0],
        sheen_roughness: 1.0,
        ..grey.clone()
    });
    assert!(r > plain[0] + 10 && g == plain[1], "{r} {g} {plain:?}");
    // Red glass over the pale background: red stays, green goes.
    let [r, g, _] = at(Material {
        color: Some([1.0, 0.2, 0.2]),
        transmission: 1.0,
        ..Default::default()
    });
    assert!(r > 200 && g < 150, "{r} {g}");
    let steel = Material {
        color: Some([0.8; 3]),
        metalness: 1.0,
        roughness: Some(0.2),
        ..Default::default()
    };
    let bare = at(steel.clone());
    let filmed = at(Material {
        iridescence: 1.0,
        ..steel
    });
    let spread = |c: [u8; 3]| c.iter().max().unwrap() - c.iter().min().unwrap();
    assert!(
        spread(bare) < 4 && spread(filmed) > 20,
        "{bare:?} {filmed:?}"
    );
}

#[test]
fn a_map_is_sampled_the_right_way_up_and_perspective_correct() {
    // Near half (v < 0.5: the image's bottom rows) red, far half green.
    let halves = map(4, 256, |_, y| {
        if y >= 128 {
            [255, 0, 0, 255]
        } else {
            [0, 255, 0, 255]
        }
    });
    let s = floor_quad();
    let m = Material {
        map: Some(halves),
        ..Default::default()
    };
    let d = Preview {
        width: 64,
        height: 128,
        elevation: Some(1.2),
        fov: Some(1.0),
        unlit: true,
        ..Default::default()
    };
    let v = one_view(&s, m, d);
    let (im, c) = (&v.images[0], &v.cameras[0]);
    // Where the quad's 3D midpoint (y = 50) lands on screen.
    let fwd = (c.target - c.position).normalize();
    let focal = 64.0 / trig::tan(0.5);
    let row = |p: DVec3| {
        let d = p - c.position;
        64.0 - d.dot(c.up) * focal / d.dot(fwd)
    };
    let (near, mid, far) = (
        row(DVec3::ZERO),
        row(DVec3::new(0.0, 50.0, 0.0)),
        row(DVec3::new(0.0, 100.0, 0.0)),
    );
    // Perspective puts the midpoint well above the screen midpoint.
    assert!(mid < (near + far) / 2.0 - 1.5, "{near} {mid} {far}");
    // The first row up from the near edge that is more green than red.
    let cross = (0..128)
        .rev()
        .find(|&y| {
            let [r, g, _] = rgb_at(im, 32, y);
            g > r
        })
        .unwrap() as f64
        + 1.0;
    // Rows below the crossing are red; an affine (screen-space)
    // interpolation would put it at the screen midpoint, 2 rows lower.
    assert!(
        (cross - mid).abs() < 0.75,
        "crossing at {cross}, midpoint {mid}"
    );
    assert_eq!(rgb_at(im, 32, near as u32 - 2), [255, 0, 0]);
    assert_eq!(rgb_at(im, 32, far as u32 + 2), [0, 255, 0]);
}

#[test]
fn a_fine_map_far_off_is_its_average_not_noise() {
    // A 256² one-texel checker on a quad 40 pixels across: every pixel
    // is the linear mean, sRGB 188, not black or white.
    let fine = map(256, 256, |x, y| {
        if (x + y) % 2 == 0 {
            [0, 0, 0, 255]
        } else {
            [255; 4]
        }
    });
    let s = floor_quad();
    let m = Material {
        map: Some(fine),
        ..Default::default()
    };
    let d = Preview {
        width: 48,
        elevation: Some(trig::PI / 2.0),
        unlit: true,
        ..Default::default()
    };
    let im = &one_view(&s, m, d).images[0];
    for y in 10..38 {
        let [r, ..] = rgb_at(im, 24, y);
        assert!((180..=196).contains(&r), "row {y}: {r}");
    }
}

#[test]
fn vertex_colours_tint_and_white_paint_is_the_plain_colour() {
    let mut s = floor_quad();
    let plain = one_view(&s, Material::default(), Preview::default()).images;
    // A white map and white vertex colours: the same bytes as neither.
    s.colors = Some(vec![[[1.0; 4]; 3]; 2]);
    let white = Material {
        map: Some(map(2, 2, |_, _| [255; 4])),
        ..Default::default()
    };
    assert_eq!(one_view(&s, white, Preview::default()).images, plain);
    // Red corners: red pixels.
    s.colors = Some(vec![[[1.0, 0.0, 0.0, 1.0]; 3]; 2]);
    let im = &one_view(&s, Material::default(), Preview::default()).images[0];
    let [r, g, b] = rgb_at(im, 128, 128);
    assert!(r > 100 && g == 0 && b == 0, "{r} {g} {b}");
    // A map that does not decode is reported and the part drawn plain.
    let bad = Material {
        map: Some(Texture::from_bytes("x.webp", b"RIFF0000WEBP".to_vec())),
        ..Default::default()
    };
    let v = one_view(&floor_quad(), bad, Preview::default());
    assert_eq!(v.images, plain);
    assert_eq!(v.undrawn.len(), 1);
}

/// Metal, gloss, occlusion and the emissive map each change the
/// picture the way they should, and a matte part that is no metal
/// is drawn as it always was.
#[test]
fn metal_gloss_occlusion_and_glow_maps_show() {
    let s = floor_quad();
    let at = |m: Material, d: Preview| rgb_at(&one_view(&s, m, d).images[0], 128, 128);
    let grey = Material {
        color: Some([0.5; 3]),
        ..Default::default()
    };
    let plain = at(grey.clone(), Preview::default());
    // Roughness 1 and no metal, said out loud: the same bytes.
    let said = Material {
        roughness: Some(1.0),
        ..grey.clone()
    };
    assert_eq!(at(said, Preview::default()), plain);
    // A metal is not black: it shows the sky, tinted by its colour.
    let gold = Material {
        color: Some([1.0, 0.8, 0.2]),
        metalness: 1.0,
        ..Default::default()
    };
    let [r, g, b] = at(gold, Preview::default());
    assert!(r > 150 && r > g && g > b && b > 20, "{r} {g} {b}");
    // The same metal polished shows more of the sky above it.
    let rough = Material {
        metalness: 1.0,
        ..grey.clone()
    };
    let polished = Material {
        roughness: Some(0.0),
        ..rough.clone()
    };
    assert!(at(polished, Preview::default())[0] > at(rough, Preview::default())[0]);
    // Occlusion darkens; with intensity 0 it does nothing.
    let dark = map(2, 2, |_, _| [0, 255, 255, 255]);
    let pit = Material {
        ao_map: Some(dark.clone()),
        ..grey.clone()
    };
    assert!(at(pit, Preview::default())[0] < plain[0]);
    let off = Material {
        ao_map: Some(dark),
        ao_map_intensity: Some(0.0),
        ..grey.clone()
    };
    assert_eq!(at(off, Preview::default()), plain);
    // The glow is its colour times its map's: a white glow under a
    // blue map is blue, and black where the map is black.
    let glowing = |texel: [u8; 4]| Material {
        color: Some([0.0; 3]),
        emissive: [1.0; 3],
        emissive_map: Some(map(2, 2, move |_, _| texel)),
        ..Default::default()
    };
    let unlit = Preview {
        unlit: true,
        ..Default::default()
    };
    assert_eq!(at(glowing([0, 0, 255, 255]), unlit), [0, 0, 255]);
    assert_eq!(at(glowing([0, 0, 0, 255]), unlit), [0, 0, 0]);
    // The metalness map's blue channel turns the metal off.
    let no_metal = Material {
        metalness: 1.0,
        metalness_map: Some(map(2, 2, |_, _| [0, 255, 0, 255])),
        ..grey
    };
    assert_eq!(at(no_metal, Preview::default()), plain);
}

#[test]
fn emissive_adds_its_colour_in_shade_and_light() {
    // A black part glowing red is red whatever the light; a dim red
    // glow on a grey part reddens it; no glow is the plain bytes.
    let s = floor_quad();
    let black = Material {
        color: Some([0.0; 3]),
        emissive: [1.0, 0.0, 0.0],
        ..Default::default()
    };
    for unlit in [false, true] {
        let d = Preview {
            unlit,
            ..Default::default()
        };
        let im = &one_view(&s, black.clone(), d).images[0];
        assert_eq!(rgb_at(im, 128, 128), [255, 0, 0]);
    }
    let grey = Material {
        color: Some([0.5; 3]),
        ..Default::default()
    };
    let plain = rgb_at(
        &one_view(&s, grey.clone(), Preview::default()).images[0],
        128,
        128,
    );
    let warm = Material {
        emissive: [0.1, 0.0, 0.0],
        ..grey
    };
    let lit = rgb_at(&one_view(&s, warm, Preview::default()).images[0], 128, 128);
    assert!(lit[0] > plain[0] && lit[1] == plain[1] && lit[2] == plain[2]);
}

#[test]
fn a_normal_map_bends_the_light_per_pixel() {
    // A flat floor whose map leans its left half toward −x and its
    // right half toward +x: the light comes from −x, so the left half
    // is brighter than a flat floor and the right half darker.
    let s = floor_quad();
    let grey = Material {
        color: Some([0.5; 3]),
        ..Default::default()
    };
    let lean = |x: u8| [x, 128, 200, 255];
    let bumped = Material {
        normal_map: Some(map(8, 8, |x, _| if x < 4 { lean(100) } else { lean(155) })),
        ..grey.clone()
    };
    let plain = &one_view(&s, grey.clone(), Preview::default()).images[0];
    let bent = &one_view(&s, bumped.clone(), Preview::default()).images[0];
    let centre = plain.width / 2;
    let row = plain.height / 2;
    let flat = rgb_at(plain, centre, row)[0];
    let (left, right) = (
        rgb_at(bent, centre - 12, row)[0],
        rgb_at(bent, centre + 12, row)[0],
    );
    assert!(left > flat && flat > right, "{left} {flat} {right}");
    // A flat map (128, 128, 255) changes nothing but rounding.
    let level = Material {
        normal_map: Some(map(8, 8, |_, _| [128, 128, 255, 255])),
        ..grey.clone()
    };
    let same = rgb_at(
        &one_view(&s, level, Preview::default()).images[0],
        centre,
        row,
    )[0];
    assert!(same.abs_diff(flat) <= 2, "{same} {flat}");
    // Unlit, the map is not read: colour only.
    let unlit = Preview {
        unlit: true,
        ..Default::default()
    };
    assert_eq!(
        rgb_at(&one_view(&s, bumped, unlit).images[0], centre, row),
        rgb_at(&one_view(&s, grey, unlit).images[0], centre, row)
    );
}
