use super::*;

/// Hollowbell FINDINGS #7: the grid's fit at 720p, 1080p and a phone in
/// both orientations: (columns, rows, scale, offset).
#[test]
fn the_pixel_grid_keeps_its_scale_and_drops_at_most_a_column() {
    let fit = |height, upscale, out| {
        let f = PixelScale {
            height,
            upscale,
            snap: true,
        }
        .fit(out);
        (f.width, f.height, f.upscale, f.offset)
    };
    let p720 = (1280, 720);
    // 240 rows: 426.67 columns round to 427, 427 x 3 = 1281: 426 at 3x,
    // a 1-pixel border each side (it was 2x in a 213-pixel border).
    assert_eq!(fit(240, 0, p720), (426, 240, 3, (1, 0)));
    assert_eq!(fit(240, 3, p720), (426, 240, 3, (1, 0)));
    // A smaller scale asked for is honoured with the border it implies.
    assert_eq!(fit(240, 2, p720), (427, 240, 2, (213, 120)));
    // One the height cannot take falls back to the largest that fits.
    assert_eq!(fit(240, 4, p720), (426, 240, 3, (1, 0)));
    // The 16bit-pixel preset's 224 rows fit 3x with columns to spare.
    assert_eq!(fit(224, 0, p720), (398, 224, 3, (43, 24)));
    assert_eq!(fit(180, 0, p720), (320, 180, 4, (0, 0)));
    assert_eq!(fit(2160, 0, p720), (1280, 720, 1, (0, 0)));
    let p1080 = (1920, 1080);
    assert_eq!(fit(240, 0, p1080), (427, 240, 4, (106, 60)));
    assert_eq!(fit(270, 0, p1080), (480, 270, 4, (0, 0)));
    assert_eq!(fit(360, 0, p1080), (640, 360, 3, (0, 0)));
    assert_eq!(fit(240, 3, p1080), (427, 240, 3, (319, 180)));
    // A phone (2532 x 1170), landscape and portrait.
    assert_eq!(fit(240, 0, (2532, 1170)), (519, 240, 4, (228, 105)));
    assert_eq!(fit(240, 0, (1170, 2532)), (111, 240, 10, (30, 66)));
    // Every fit lies inside its output.
    for out in [
        p720,
        p1080,
        (2532, 1170),
        (1170, 2532),
        (1366, 768),
        (390, 844),
    ] {
        for height in [120, 180, 224, 240, 270, 360, 480] {
            for upscale in 0..6 {
                let (w, h, up, (x, y)) = fit(height, upscale, out);
                assert!(up >= 1 && w * up + 2 * x <= out.0 && h * up + 2 * y <= out.1);
                let ideal = (h as f64 * out.0 as f64 / out.1 as f64).round() as u32;
                assert!(
                    w + 1 >= ideal.min(out.0),
                    "{out:?} {height} {upscale}: {w} of {ideal}"
                );
            }
        }
    }
}

#[test]
fn the_chain_bits_follow_primitives_not_their_settings() {
    let off = Stylise::default();
    assert_eq!(off.chain_bits(), 0);
    let q = Quantize {
        posterize: 6,
        mode: PosterizeMode::Channel,
        palette: None,
        dither: DitherPattern::Bayer4,
        dither_strength: 1.0,
        dither_scale: 1,
        space: StyliseSpace::Screen,
    };
    let six = Stylise {
        quantize: Some(q),
        ..off
    };
    let eight = Stylise {
        quantize: Some(Quantize { posterize: 8, ..q }),
        ..off
    };
    assert_ne!(six.chain_bits(), off.chain_bits(), "a pass added");
    assert_eq!(six.chain_bits(), eight.chain_bits(), "a setting only");
    let grid = Stylise {
        pixel: Some(PixelScale {
            height: 180,
            upscale: 0,
            snap: false,
        }),
        ..six
    };
    assert_ne!(grid.chain_bits(), six.chain_bits(), "the pixel grid on");
    let mut toon = six;
    toon.material.toon = true;
    assert_ne!(toon.chain_bits(), six.chain_bits(), "the material style");
}

#[test]
fn surface_space_asks_for_the_prepass_only_where_it_draws() {
    let mut s = Stylise::default();
    assert!(!s.on_surfaces());
    let paint = Paint {
        canvas_strength: 0.5,
        canvas_space: StyliseSpace::Surface,
        ..Paint::DEFAULT
    };
    s.paint = Some(paint);
    assert!(!s.on_surfaces(), "a surface canvas without a texture");
    s.hatching = Some(Hatching {
        strength: 0.8,
        scale: 6.0,
        levels: 4,
        angle_degrees: 45.0,
        color: [0.0; 3],
        paper: [1.0; 3],
        space: StyliseSpace::Screen,
        depth: 0.0,
        keep_color: false,
    });
    assert!(!s.on_surfaces(), "screen hatching");
    assert!(!s.depth_edges(), "hatching on the tone alone");
    s.hatching.as_mut().unwrap().depth = 0.5;
    assert!(
        s.depth_edges(),
        "hatching from depth edges reads the prepass"
    );
    s.hatching.as_mut().unwrap().depth = 0.0;
    s.hatching.as_mut().unwrap().space = StyliseSpace::Surface;
    assert!(s.on_surfaces());
    assert!(!s.in_object_space(), "world space runs no object pass");
    s.hatching.as_mut().unwrap().space = StyliseSpace::Object;
    assert!(s.on_surfaces() && s.in_object_space());
    s.hatching.as_mut().unwrap().strength = 0.0;
    assert!(!s.in_object_space(), "only where the pattern draws");
    s.hatching.as_mut().unwrap().strength = 0.8;
    s.hatching.as_mut().unwrap().space = StyliseSpace::Surface;
    assert_eq!(
        s.chain_bits(),
        Stylise {
            hatching: Some(Hatching {
                space: StyliseSpace::Screen,
                ..s.hatching.unwrap()
            }),
            ..s
        }
        .chain_bits(),
        "the space is a setting, not a pass"
    );
}

/// Obra Dinn's anchored dither: dither cells on the surfaces ask for the
/// prepass (and the object pass in object space), but only while a
/// dither draws; a quantise without one, or a screen dither, does not.
#[test]
fn a_surface_dither_asks_for_the_prepass() {
    let q = Quantize {
        posterize: 2,
        mode: PosterizeMode::Channel,
        palette: None,
        dither: DitherPattern::Bayer8,
        dither_strength: 1.0,
        dither_scale: 1,
        space: StyliseSpace::Surface,
    };
    let mut s = Stylise {
        quantize: Some(q),
        ..Stylise::default()
    };
    assert!(s.on_surfaces() && !s.in_object_space());
    s.quantize = Some(Quantize {
        space: StyliseSpace::Object,
        ..q
    });
    assert!(s.on_surfaces() && s.in_object_space());
    s.quantize = Some(Quantize {
        dither: DitherPattern::None,
        ..q
    });
    assert!(!s.on_surfaces(), "no dither, no cells to lay");
    s.quantize = Some(Quantize {
        space: StyliseSpace::Screen,
        ..q
    });
    assert!(!s.on_surfaces(), "a screen dither reads no prepass");
}

#[test]
fn everything_is_off_by_default() {
    assert!(!Stylise::default().is_active());
    let mut s = Stylise::default();
    s.material.outline_width = 1.0;
    assert!(s.is_active());
}

/// Shadow darkness alone is a material style: a look that sets only it
/// (heavy-ink-shadows' black masses) reaches the lit shaders.
#[test]
fn shadow_darkness_alone_is_active() {
    let mut s = Stylise::default();
    s.material.shadow_darkness = 1.0;
    assert!(s.material.is_active());
    assert!(s.is_active());
}

/// Retro 3D snaps a vertex to the nearest multiple of the snap size in
/// pixels and floors its depth to the bits' steps.
#[test]
fn retro_vertices_snap_to_the_pixel_grid_and_depth_steps() {
    let stylise = crate::Retro3d::uniform(
        Some(crate::Retro3d {
            vertex_snap: 4.0,
            affine: 0.0,
            depth_bits: 8,
        }),
        640,
        480,
    );
    let w = 3.0;
    // Pixel x (0.1234 / 2 + 0.5) 640 = 359.49 snaps to 360, ndc 0.125;
    // pixel y from the top (0.5 + 0.2 / 2) 480 = 288 stays on the grid.
    let c = crate::meshlet::retro_clip(
        glam::Vec4::new(0.1234 * w, -0.2 * w, 0.5004 * w, w),
        stylise,
    );
    assert!((c.x / w - 0.125).abs() < 1e-6, "{c:?}");
    assert!((c.y / w + 0.2).abs() < 1e-6, "{c:?}");
    // 0.5004 x 256 = 128.1: step 128, depth 0.5.
    assert!((c.z / w - 0.5).abs() < 1e-6, "{c:?}");
    assert_eq!(c.w, w);
    // Without retro 3D, the identity.
    let p = glam::Vec4::new(0.3, 0.2, 0.1, 2.0);
    assert_eq!(
        crate::meshlet::retro_clip(p, crate::Retro3d::uniform(None, 640, 480)),
        p
    );
}
