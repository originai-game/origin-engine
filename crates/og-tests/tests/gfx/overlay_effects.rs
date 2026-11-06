use super::*;

fn overlay_px(img: &RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * img.width + x) * 4) as usize;
    [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
}

fn srgb8(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        linear * 12.92 * 255.0
    } else {
        (1.055 * linear.powf(1.0 / 2.4) - 0.055) * 255.0
    }
}

fn near(p: [u8; 3], linear: [f32; 3], within: f32) -> bool {
    p.iter()
        .zip(linear)
        .all(|(&a, e)| (a as f32 - srgb8(e)).abs() <= within)
}

/// The overlay is drawn over the finished frame: after TAA, bloom, metered
/// exposure and ACES, an opaque overlay colour comes out exactly as authored
/// (sRGB-encoded), its edges are sharp on every TAA frame, it covers the
/// scene without a depth test, and its clip cuts rows at the clip's edge.
#[test]
#[ignore = "requires GPU"]
fn the_overlay_is_drawn_after_post_untonemapped_and_clipped() {
    let g = built("overlay");
    let mut renderer = Renderer::new_blocking(&g.desc).expect("renderer");
    let mut scene = g.scene;
    let mut frames = Vec::new();
    for f in 0..g.frames {
        if let Some(update) = g.update {
            update(&mut scene, f);
        }
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        if f + 2 >= g.frames {
            frames.push(renderer.read_offscreen().expect("capture"));
        }
    }
    let img = frames.last().expect("a frame");
    assert_eq!((img.width, img.height), (1280, 720));
    // The health bar's fill and its track, one pixel either side of the
    // fill's end: exact colours, no TAA smear, no tone mapping.
    let fill = [0.9, 0.18, 0.05];
    let track = [0.1, 0.1, 0.1];
    for img in &frames {
        assert!(
            near(overlay_px(img, 100, 102), fill, 1.5),
            "fill {:?}",
            overlay_px(img, 100, 102)
        );
        assert!(
            near(overlay_px(img, 239, 102), fill, 1.5),
            "fill edge {:?}",
            overlay_px(img, 239, 102)
        );
        assert!(
            near(overlay_px(img, 240, 102), track, 1.5),
            "track edge {:?}",
            overlay_px(img, 240, 102)
        );
    }
    // The crosshair over the scene: white, whatever the scene's depth.
    assert!(near(overlay_px(img, 639, 352), [1.0, 1.0, 1.0], 12.0));
    // The list: the first row is cut at the clip's top (y 300), not dropped;
    // above the clip nothing of it shows.
    let row = [0.2, 0.22, 0.26];
    assert!(
        near(overlay_px(img, 1100, 302), row, 1.5),
        "the cut row at the clip's top {:?}",
        overlay_px(img, 1100, 302)
    );
    let without = {
        let g = built("overlay");
        let mut bare = g.scene;
        let o = bare.overlay.as_mut().expect("overlay");
        o.sprites.retain(|s| s.clip.is_none());
        o.texts.retain(|t| t.clip.is_none());
        let mut r = Renderer::new_blocking(&g.desc).expect("renderer");
        for f in 0..g.frames {
            if let Some(update) = g.update {
                update(&mut bare, f);
            }
            r.render(&bare, &RenderTarget::Offscreen).expect("render");
        }
        r.read_offscreen().expect("capture")
    };
    for x in (990..1230).step_by(7) {
        for y in [282u32, 290, 298, 452, 460, 470] {
            assert_eq!(
                overlay_px(img, x, y),
                overlay_px(&without, x, y),
                "a clipped row shows at ({x}, {y}), outside its clip"
            );
        }
    }
}

#[test]
#[ignore = "requires GPU"]
fn skeleton_2d_gpu_weighting_matches_cpu_triangles() {
    let desc = built("skeleton-2d").desc;
    for normal in [false, true] {
        for blend in [
            og_render::SpriteBlend::Alpha,
            og_render::SpriteBlend::Additive,
        ] {
            let make = |gpu| {
                let mut scene = og_tests::skeleton_scenes::skeleton_scene(gpu).unwrap();
                if normal {
                    let texture = scene.sprite_meshes[0].texture.unwrap();
                    let n = scene.add_texture(og_render::TextureData {
                        width: 1,
                        height: 1,
                        rgba: vec![190, 90, 230, 255],
                        color_space: og_render::ColorSpace::Linear,
                        mips: og_render::Mips::None,
                    });
                    assert!(scene.set_texture_normal(texture, Some(n)));
                }
                for m in &mut scene.sprite_meshes {
                    m.blend = blend;
                }
                scene
            };
            let (cpu, cpu_stats) = frame(&desc, &make(false), 1);
            let (gpu, gpu_stats) = frame(&desc, &make(true), 1);
            assert_eq!(cpu_stats.sprite_draws, gpu_stats.sprite_draws);
            // LBS changes arithmetic order but not the image: one channel
            // quantum accommodates fragment rounding, not changed coverage.
            let max = cpu
                .pixels
                .iter()
                .zip(&gpu.pixels)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            assert!(
                max <= 1,
                "normal={normal} blend={blend:?}: max channel difference {max}"
            );
        }
    }
}
