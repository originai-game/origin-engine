//! The stylisation scenes (`og-render/src/stylise.rs`, the IDL's `Stylise`
//! and `MaterialStyle`): each primitive group over a scene the references
//! already know, so a frame shows the primitive and nothing else new.
//!
//! - `stylise-pixel`: the toon scene on a 180-row grid scaled up by whole
//!   numbers, six tones per channel through a Bayer 4 dither.
//! - `stylise-palette`: the same at 240 rows, every pixel the nearest of a
//!   16-colour palette, blue-noise dithered.
//! - `stylise-print`: cross-hatching and ink lines over the physical
//!   materials scene (a pencil drawing).
//! - `stylise-halftone`: halftone dots over the toon scene.
//! - `stylise-paint`: painterly smoothing, edge darkening and bleed over the
//!   physical materials scene.
//! - `stylise-bands`: four flat depth bands over the toon scene (layered
//!   silhouettes).
//! - `stylise-retro`: vertex snapping, affine textures and 12-bit depth
//!   over the physical materials scene.
//! - `stylise-material`: the material style alone: every lit material toon
//!   with a warm shadow tint, a rim, outlines that thin with distance, and
//!   flat facets, over the physical materials scene (no per-material
//!   stylised fields).

use og_render::{
    DepthBands, DitherPattern, Halftone, Hatching, Ink, MaterialStyle, Paint, Palette, PixelScale,
    Quantize, RenderScene, Retro3d,
};

/// A 16-colour palette: a dusk ramp from near-black through purples and
/// oranges to cream (sRGB).
fn dusk_palette() -> Palette {
    Palette {
        colors: vec![
            [16, 12, 24],
            [40, 24, 56],
            [72, 36, 88],
            [112, 48, 104],
            [160, 64, 104],
            [200, 88, 96],
            [232, 128, 88],
            [248, 176, 96],
            [252, 216, 136],
            [255, 244, 200],
            [48, 72, 96],
            [64, 112, 128],
            [96, 160, 152],
            [152, 208, 176],
            [208, 240, 208],
            [255, 255, 255],
        ],
    }
}

pub fn pixel_scene() -> Result<RenderScene, String> {
    let mut scene = crate::fx_scenes::stylised_scene()?;
    scene.stylise.pixel = Some(PixelScale {
        height: 180,
        upscale: 0,
        snap: true,
    });
    scene.stylise.quantize = Some(Quantize {
        posterize: 6,
        palette: None,
        dither: DitherPattern::Bayer4,
        dither_strength: 1.0,
        dither_scale: 1,
    });
    Ok(scene)
}

pub fn palette_scene() -> Result<RenderScene, String> {
    let mut scene = crate::fx_scenes::stylised_scene()?;
    let palette = scene.add_palette(dusk_palette());
    scene.stylise.pixel = Some(PixelScale {
        height: 240,
        upscale: 0,
        snap: true,
    });
    scene.stylise.quantize = Some(Quantize {
        posterize: 0,
        palette: Some(palette),
        dither: DitherPattern::BlueNoise,
        dither_strength: 1.0,
        dither_scale: 1,
    });
    Ok(scene)
}

pub fn print_scene() -> RenderScene {
    let mut scene = crate::physical_scenes::physical_scene();
    scene.stylise.hatching = Some(Hatching {
        strength: 0.9,
        scale: 6.0,
        levels: 4,
        angle_degrees: 45.0,
        color: [0.04, 0.035, 0.03],
    });
    scene.stylise.ink = Some(Ink {
        threshold: 0.12,
        width: 1.5,
        color: [0.02, 0.02, 0.03],
    });
    scene
}

pub fn halftone_scene() -> Result<RenderScene, String> {
    let mut scene = crate::fx_scenes::stylised_scene()?;
    scene.stylise.halftone = Some(Halftone {
        strength: 1.0,
        size: 7.0,
        angle_degrees: 30.0,
        color: [0.05, 0.02, 0.08],
    });
    Ok(scene)
}

pub fn paint_scene() -> RenderScene {
    let mut scene = crate::physical_scenes::physical_scene();
    scene.stylise.paint = Some(Paint {
        radius: 4.0,
        anisotropy: 0.6,
        edge_darkening: 0.6,
        bleed: 0.4,
        canvas: None,
        canvas_strength: 0.0,
        canvas_scale: 1.0,
    });
    scene
}

pub fn bands_scene() -> Result<RenderScene, String> {
    let mut scene = crate::fx_scenes::stylised_scene()?;
    scene.stylise.depth_bands = Some(DepthBands {
        count: 4,
        near: 4.0,
        far: 30.0,
        near_color: [0.03, 0.02, 0.06],
        far_color: [0.6, 0.65, 0.8],
        flatten: 1.0,
    });
    Ok(scene)
}

pub fn retro_scene() -> RenderScene {
    let mut scene = crate::physical_scenes::physical_scene();
    scene.stylise.pixel = Some(PixelScale {
        height: 240,
        upscale: 0,
        snap: false,
    });
    scene.stylise.retro = Some(Retro3d {
        vertex_snap: 2.0,
        affine: 1.0,
        depth_bits: 12,
    });
    scene
}

pub fn material_scene() -> RenderScene {
    let mut scene = crate::physical_scenes::physical_scene();
    scene.stylise.material = MaterialStyle {
        toon: true,
        gradient_map: None,
        shadow_tint: [0.5, 0.2, 0.1],
        shadow_tint_amount: 0.8,
        rim_color: [1.0, 0.95, 0.85],
        rim_intensity: 0.5,
        rim_power: 3.0,
        outline_color: [0.05, 0.03, 0.02],
        outline_width: 3.0,
        outline_distance: 8.0,
        flat_shading: true,
    };
    scene
}
