//! A device made again after a loss that takes other texture encodings
//! than the lost one (G1-4; a browser may come back without BC, or on
//! another adapter): a cooked Basis texture transcoded for the old device
//! is refused by the new one, and transcoded again from the bundle for it
//! draws the frame a renderer made with those encodings from the start
//! draws. og-runtime's `assets::retranscode` does the transcoding again
//! for a game (tests/display.rs checks it without a GPU). GPU test: the
//! coordinator runs it on the runners (metal on macrunner, DX12 on
//! studio-win).
//!
//! Ways this could fail: the new device handed the old encoding (a
//! validation error, or a panic natively); a texture transcoded again
//! for the old formats instead of the new ones; a recreated renderer that
//! draws a texture replaced on its first frame differently from a fresh
//! one.

use og_cook::runtime::{Bundle, CookedTexture};
use og_cook::{CookOptions, cook_project};
use og_render::{
    RenderDesc, RenderError, RenderScene, RenderTarget, Renderer, RgbaImage, TextureCompression,
    TextureFormat, TextureSource,
};
use og_tests::scenes;
use std::path::PathBuf;

/// A 64×64 colour texture cooked (ETC1S, a Basis encoding) into a scratch
/// project.
fn cooked() -> CookedTexture {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-device-formats");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/tex")).expect("assets dir");
    let image = image::RgbaImage::from_fn(64, 64, |x, y| {
        image::Rgba([(x * 4) as u8, (y * 4) as u8, ((x ^ y) * 4) as u8, 255])
    });
    image
        .save(root.join("assets/tex/grad.png"))
        .expect("write the png");
    let report = cook_project(&root, &CookOptions::default()).expect("cook");
    assert!(report.ok(), "{:?}", report.errors());
    Bundle::open(&root.join("cooked"))
        .expect("bundle")
        .load_texture("tex/grad.png")
        .expect("the texture loads")
}

/// The capture of `scene`: its bytes are a function of the scene alone.
/// A plain frame's are not: natively a lit pipeline draws with a variant
/// for the frame's features until its full variant has compiled in the
/// background (pipeline_features.rs), and the two differ in their last
/// bit, so two renderers' frames match only when their compiles finished
/// at the same frame.
fn draw(renderer: &mut Renderer, scene: &RenderScene) -> RgbaImage {
    renderer
        .render_capture(scene, &RenderTarget::Offscreen)
        .expect("frame");
    renderer.read_offscreen().expect("capture")
}

#[test]
#[ignore = "requires GPU"]
fn a_device_with_other_encodings_draws_textures_transcoded_again_for_it() {
    let texture = cooked();
    let g = scenes::build("materials").expect("scene");
    let mut scene = g.scene;
    let mut renderer = Renderer::new_blocking(&g.desc).expect("renderer");
    let old = renderer.info().texture_formats.clone();
    let source = texture.to_texture_source(Some(&old)).expect("transcode");
    assert!(
        matches!(source, TextureSource::Encoded(_)),
        "this device takes no compressed encoding ({old:?}): nothing to check"
    );
    let t = scene.add_texture_source(source);
    let m = scene.material_handles().next().expect("a material");
    let mut material = *scene.standard_material(m).expect("standard");
    material.map = t.slot();
    scene.set_standard_material(m, material);
    draw(&mut renderer, &scene);

    renderer.lose_device();
    let mut again = renderer
        .successor()
        .with_texture_compression(TextureCompression::Off)
        .make_blocking()
        .expect("a new device");
    drop(renderer);
    let new = again.info().texture_formats.clone();
    assert_eq!(new.supported, vec![TextureFormat::Rgba8], "{new:?}");
    // The old device's encoding is refused, not handed to the new device.
    match again.render(&scene, &RenderTarget::Offscreen) {
        Err(RenderError::Unsupported(_)) => {}
        other => panic!(
            "the lost device's encoding on the new one: {:?}",
            other.map(|_| ())
        ),
    }

    let source = texture.to_texture_source(Some(&new)).expect("transcode");
    let TextureSource::Rgba(data) = source else {
        panic!("transcoded again to an encoding the new device does not take");
    };
    scene.set_texture(t, data);
    let after = draw(&mut again, &scene);

    let desc = RenderDesc {
        texture_compression: TextureCompression::Off,
        ..g.desc
    };
    let mut fresh = Renderer::new_blocking(&desc).expect("renderer");
    let expected = draw(&mut fresh, &scene);
    assert!(
        after.pixels == expected.pixels,
        "the recreated renderer's frame differs from a fresh renderer's with the same encodings"
    );
}
