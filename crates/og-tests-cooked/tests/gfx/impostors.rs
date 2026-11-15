//! A cooked model's impostor card against the model itself
//! (og_tests_cooked::impostor_scenes).

use glam::{Vec3, Vec4};
use og_render::{RenderTarget, Renderer};
use og_tests_cooked::impostor_scenes::{
    CARD_AT, MESH_AT, impostor_scene, impostor_self_shadow_scene,
};

/// G5 (R6's "impostors take no shadow of their own"): under a sun in front
/// of it, a tree's trunk is in its crown's shadow, and its card draws the
/// trunk as dark as the meshes do: the card shades itself from its depth
/// atlas, the cascades reaching it only past its sphere.
///
/// Ways this could fail: the self-shadow never read (the card's trunk lit
/// like a bare post, several times the mesh's), read from the wrong frame
/// (the trunk dark or lit at random over its height), or shading what the
/// sun sees (the crown's top as dark as the trunk).
#[test]
#[ignore = "requires GPU"]
fn a_cards_trunk_is_in_its_own_crowns_shadow() {
    let (scene, (min, max)) = impostor_self_shadow_scene().expect("the cooked tree");
    let mut r = Renderer::new_blocking(&og_tests::scenes::base_desc()).expect("renderer");
    r.render(&scene, &RenderTarget::Offscreen).expect("render");
    let img = r.read_offscreen().expect("capture");
    let dir = super::out_dir();
    std::fs::create_dir_all(&dir).expect("out dir");
    image::save_buffer(
        dir.join("impostor-self-shadow.png"),
        &img.pixels,
        img.width,
        img.height,
        image::ColorType::Rgba8,
    )
    .expect("saved");
    let vp = {
        let c = &scene.frame.camera;
        let og_render::Projection::Perspective { fov_y_degrees } = c.projection else {
            unreachable!("a perspective camera")
        };
        glam::camera::rh::proj::directx::perspective(
            fov_y_degrees.to_radians(),
            img.width as f32 / img.height as f32,
            c.near,
            c.far,
        ) * glam::camera::rh::view::look_at_mat4(c.position, c.target, c.up)
    };
    let px = |p: Vec3| {
        let c = vp * Vec4::new(p.x, p.y, p.z, 1.0);
        let n = c.truncate() / c.w;
        (
            ((n.x * 0.5 + 0.5) * img.width as f32) as i32,
            ((0.5 - n.y * 0.5) * img.height as f32) as i32,
        )
    };
    let luma = |x: i32, y: i32| {
        let i = ((y as u32 * img.width + x as u32) * 4) as usize;
        let p = &img.pixels[i..i + 3];
        0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2])
    };
    // Mean luma of a band of heights (shares of the tree's), 5 px wide
    // about the trunk's axis (the crown's at its top).
    let band = |at: Vec3, lo: f32, hi: f32| {
        let (mut sum, mut n) = (0.0, 0.0);
        for k in 0..16 {
            let y = min.y + (max.y - min.y) * (lo + (hi - lo) * k as f32 / 15.0);
            let (x, py) = px(at + Vec3::new(0.0, y, 0.0));
            for dx in -2..=2 {
                sum += luma(x + dx, py);
                n += 1.0;
            }
        }
        sum / n
    };
    let (mesh, card) = (band(MESH_AT, 0.1, 0.5), band(CARD_AT, 0.1, 0.5));
    let (mesh_top, card_top) = (band(MESH_AT, 0.8, 0.9), band(CARD_AT, 0.8, 0.9));
    println!(
        "trunk under the crown: mesh {mesh:.1}, card {card:.1}; crown top: mesh {mesh_top:.1}, card {card_top:.1}"
    );
    assert!(
        (card - mesh).abs() < 0.25 * mesh,
        "the card's trunk {card:.1} against the mesh's {mesh:.1}"
    );
    assert!(
        card < 0.5 * card_top && (card_top - mesh_top).abs() < 0.2 * mesh_top,
        "the card's crown top {card_top:.1} (mesh {mesh_top:.1}) against its trunk {card:.1}"
    );
}

/// A model's card seen from under its horizon (the roadmap's "view from
/// below"): whole, and with `impostorBelow` the model's own underside.
///
/// Ways this could fail: the card faces an eye its atlases hold no frames
/// around and reprojects the horizon's along their depth (the trunk's
/// foot and the crown's edge drawn as loose pieces, well outside the
/// mesh's outline: its bounds were 1.4 and 1.6 times as tall), the lower
/// hemisphere's frames never read or read from the upper half (the card
/// from 70 degrees below less than half the mesh's size), or read mirrored
/// or from the wrong cell (a card far off the mesh's outline), or each
/// frame reprojected to a different side of a step in depth (the thin
/// trunk before the crown: crown pieces loose beside the trunk, 192 px
/// from 45 degrees, and the trunk blotched), or a frame drawing what lies
/// behind the one surface it holds (specks of crown beside the trunk, 49
/// px), or the card's own shadow read straight along each of the four
/// frames about the sun (each put the crown's shadow edge elsewhere on the
/// thin trunk, combing it into streaks up close: 162 and 86 px in small
/// pieces from 30 and 45 degrees, the mesh 9 and 12).
#[test]
#[ignore = "requires GPU"]
fn a_card_seen_from_below_is_whole() {
    for below in [false, true] {
        let (mut scene, (min, max)) = impostor_scene(below).expect("the cooked tree");
        let mid = (min + max) * 0.5;
        let mut r = Renderer::new_blocking(&og_tests::scenes::base_desc()).expect("renderer");
        for (name, eye) in [
            ("45", Vec3::new(0.0, mid.y - 7.8, 7.8)),
            ("70", Vec3::new(0.0, mid.y - 10.3, 3.8)),
        ] {
            scene.frame.camera.position = eye;
            r.render(&scene, &RenderTarget::Offscreen).expect("render");
            let img = r.read_offscreen().expect("capture");
            let dir = super::out_dir();
            std::fs::create_dir_all(&dir).expect("out dir");
            image::save_buffer(
                dir.join(format!("impostor-below-{name}-{below}.png")),
                &img.pixels,
                img.width,
                img.height,
                image::ColorType::Rgba8,
            )
            .expect("saved");
            let mesh = tree_pixels(&img, 0, img.width / 2);
            let card = tree_pixels(&img, img.width / 2, img.width);
            println!(
                "from {name} degrees below, impostorBelow {below}: mesh {mesh:?}, card {card:?}"
            );
            assert!(
                card.1 < 1.12 * mesh.1 && card.2 < 1.1 * mesh.2,
                "from {name} degrees below (impostorBelow {below}) the card's pieces reach {card:?}, the mesh {mesh:?}"
            );
            let (mesh_loose, card_loose) = (
                loose_pixels(&img, 0, img.width / 2),
                loose_pixels(&img, img.width / 2, img.width),
            );
            println!("  loose pixels: mesh {mesh_loose}, card {card_loose}");
            assert!(
                card_loose <= mesh_loose + 10,
                "from {name} degrees below (impostorBelow {below}) the card has {card_loose} pixels apart from its tree, the mesh {mesh_loose}"
            );
            if below {
                assert!(
                    (0.9..1.2).contains(&(card.0 / mesh.0))
                        && card.1 > 0.9 * mesh.1
                        && card.2 > 0.9 * mesh.2,
                    "from {name} degrees below the card {card:?} against the mesh {mesh:?}"
                );
            }
        }
        if below {
            // Up close, the card's own shadow on its trunk: one edge, as the
            // mesh's, not combed into small lit and shaded pieces.
            for (name, distance, angle) in [("near30", 3.6f32, 30f32), ("near45", 4.0, 45.0)] {
                let a = angle.to_radians();
                let target = CARD_AT + Vec3::new(0.0, mid.y * 0.8, 0.0);
                scene.frame.camera.position = target + Vec3::new(0.0, -a.sin(), a.cos()) * distance;
                scene.frame.camera.target = target;
                r.render(&scene, &RenderTarget::Offscreen).expect("render");
                let img = r.read_offscreen().expect("capture");
                let streaks = streak_pixels(&img);
                println!("{name}: trunk streak pixels {streaks}");
                assert!(
                    streaks <= 60,
                    "up close from {name} degrees below, the card's trunk has {streaks} pixels in small lit or shaded pieces"
                );
            }
        }
    }
}

/// A swaying model's card (`og.Model.sway`, the roadmap's "sway") leans
/// with the wind every vegetation tier reads, by its height squared.
///
/// Ways this could fail: a model's card never swayed (no row moves), the
/// card bent as one quad (its foot slides downwind, 6 px here, and its
/// lower trunk stands twice too far), swayed by the sphere's height
/// rather than the model's, or its shadow left behind.
#[test]
#[ignore = "requires GPU"]
fn a_swaying_models_card_bends_from_its_foot() {
    let (mut scene, (_, max)) = impostor_scene(false).expect("the cooked tree");
    scene.objects.impostors[0].sway = 1.0;
    scene.objects.impostors[0].top = max.y;
    let strength = 0.12;
    let mut r = Renderer::new_blocking(&og_tests::scenes::base_desc()).expect("renderer");
    let mut draw = |strength: f32| {
        scene.atmosphere.wind = Some(og_render::Wind {
            strength,
            direction: Vec3::X,
            speed: 3.0,
            gust: 0.0,
            turbulence: 0.0,
            ..Default::default()
        });
        // The simulated field settles on a steady wind's lean.
        for tick in 0..3 {
            scene.frame.tick = tick;
            r.render(&scene, &RenderTarget::Offscreen).expect("render");
        }
        r.read_offscreen().expect("capture")
    };
    let (still, blown) = (draw(0.0), draw(strength));
    let vp = {
        let c = &scene.frame.camera;
        let og_render::Projection::Perspective { fov_y_degrees } = c.projection else {
            unreachable!("a perspective camera")
        };
        glam::camera::rh::proj::directx::perspective(
            fov_y_degrees.to_radians(),
            still.width as f32 / still.height as f32,
            c.near,
            c.far,
        ) * glam::camera::rh::view::look_at_mat4(c.position, c.target, c.up)
    };
    let px = |p: Vec3| {
        let c = vp * Vec4::new(p.x, p.y, p.z, 1.0);
        let n = c.truncate() / c.w;
        glam::Vec2::new(
            (n.x * 0.5 + 0.5) * still.width as f32,
            (0.5 - n.y * 0.5) * still.height as f32,
        )
    };
    // The trunk's middle (its brown pixels' mean x) on the row of height
    // `h`, within 60 px of the card's axis.
    let trunk = |img: &og_render::RgbaImage, h: f32| {
        let at = px(CARD_AT + Vec3::new(0.0, h, 0.0));
        let (mut sum, mut n) = (0.0, 0.0);
        for x in at.x as u32 - 60..at.x as u32 + 60 {
            let i = ((at.y as u32 * img.width + x) * 4) as usize;
            let p = &img.pixels[i..i + 3];
            if p[0] > p[1] + 8 && p[0] > p[2] + 8 {
                sum += x as f32 + 0.5;
                n += 1.0;
            }
        }
        assert!(n > 4.0, "no trunk on the row of height {h}");
        sum / n
    };
    // What scatter_sway moves a point at height `h` by, in pixels.
    let expected = |h: f32| {
        let d = strength * h * (h / max.y).min(1.0);
        px(CARD_AT + Vec3::new(d, h, 0.0)).x - px(CARD_AT + Vec3::new(0.0, h, 0.0)).x
    };
    for share in [0.04, 0.2, 0.4] {
        let h = share * max.y;
        let moved = trunk(&blown, h) - trunk(&still, h);
        println!(
            "at {h:.2} m the trunk moved {moved:.1} px, the model's {:.1}",
            expected(h)
        );
        assert!(
            (moved - expected(h)).abs() < 1.5,
            "at {h:.2} m the card's trunk moved {moved:.1} px, the model's {:.1}",
            expected(h)
        );
    }
    assert!(expected(0.4 * max.y) > 5.0, "a wind too weak to tell");
}

/// The tree's pixels (anything not the sky's blue) in the columns from
/// `x0` to `x1`: how many, and their bounds' width and height.
/// A tree pixel of `img`: not the sky's blue.
fn is_tree(img: &og_render::RgbaImage, x: u32, y: u32) -> bool {
    let i = ((y * img.width + x) * 4) as usize;
    let p = &img.pixels[i..i + 3];
    !(p[2] > p[0] && p[2] > p[1])
}

/// The tree pixels in columns `x0..x1` apart from the largest 8-connected
/// piece of them (the tree itself).
fn loose_pixels(img: &og_render::RgbaImage, x0: u32, x1: u32) -> usize {
    let w = (x1 - x0) as usize;
    let mut seen = vec![false; w * img.height as usize];
    let (mut total, mut largest) = (0, 0);
    for start in 0..seen.len() {
        let (sx, sy) = ((start % w) as u32 + x0, (start / w) as u32);
        if seen[start] || !is_tree(img, sx, sy) {
            continue;
        }
        seen[start] = true;
        let (mut stack, mut size) = (vec![(sx, sy)], 0);
        while let Some((x, y)) = stack.pop() {
            size += 1;
            for (dx, dy) in [
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ] {
                let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                if nx < i64::from(x0)
                    || nx >= i64::from(x1)
                    || ny < 0
                    || ny >= i64::from(img.height)
                {
                    continue;
                }
                let (nx, ny) = (nx as u32, ny as u32);
                let k = (ny as usize) * w + (nx - x0) as usize;
                if !seen[k] && is_tree(img, nx, ny) {
                    seen[k] = true;
                    stack.push((nx, ny));
                }
            }
        }
        total += size;
        largest = largest.max(size);
    }
    total - largest
}

/// The trunk's pixels (brown) in pieces under 40 px of either side of its
/// shading (lit or shaded: luma above or below the middle of its 10th and
/// 90th percentiles), 8-connected: a shadow edge combed into streaks.
fn streak_pixels(img: &og_render::RgbaImage) -> usize {
    let (w, h) = (img.width as usize, img.height as usize);
    let px = |k: usize| &img.pixels[k * 4..k * 4 + 3];
    let trunk: Vec<bool> = (0..w * h)
        .map(|k| {
            let p = px(k);
            p[0] > p[1] + 8 && p[0] > p[2] + 8
        })
        .collect();
    let luma = |k: usize| {
        let p = px(k);
        0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2])
    };
    let mut l: Vec<f32> = (0..w * h).filter(|&k| trunk[k]).map(luma).collect();
    if l.is_empty() {
        return 0;
    }
    l.sort_by(f32::total_cmp);
    let mid = 0.5 * (l[l.len() / 10] + l[l.len() * 9 / 10]);
    let side: Vec<bool> = (0..w * h).map(|k| luma(k) > mid).collect();
    let mut seen = vec![false; w * h];
    let mut streaks = 0;
    for start in 0..w * h {
        if seen[start] || !trunk[start] {
            continue;
        }
        seen[start] = true;
        let (mut stack, mut piece) = (vec![start], 0);
        while let Some(k) = stack.pop() {
            piece += 1;
            let (x, y) = ((k % w) as i64, (k / w) as i64);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                        continue;
                    }
                    let n = ny as usize * w + nx as usize;
                    if !seen[n] && trunk[n] && side[n] == side[start] {
                        seen[n] = true;
                        stack.push(n);
                    }
                }
            }
        }
        if piece < 40 {
            streaks += piece;
        }
    }
    streaks
}

fn tree_pixels(img: &og_render::RgbaImage, x0: u32, x1: u32) -> (f32, f32, f32) {
    let (mut n, mut lo, mut hi) = (0.0, [u32::MAX; 2], [0u32; 2]);
    for y in 0..img.height {
        for x in x0..x1 {
            if !is_tree(img, x, y) {
                continue;
            }
            n += 1.0;
            lo = [lo[0].min(x), lo[1].min(y)];
            hi = [hi[0].max(x), hi[1].max(y)];
        }
    }
    (n, (hi[0] - lo[0]) as f32, (hi[1] - lo[1]) as f32)
}
