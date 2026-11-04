//! Terrain stamps read from rendered frames: a mesh stamped into a
//! terrain is shaded by the terrain's own layers by its own normal (rock
//! on a cliff's face, grass on the valley before it), and within `blend`
//! metres of the ground it meets it takes the ground's look, so the seam
//! where the face meets the valley disappears.

use super::*;

/// The luma (0..1) around where a world point lands.
fn luma_at(img: &RgbaImage, camera: &Camera, p: Vec3) -> f32 {
    luma(color_at(img, project(camera, p))) / 255.0
}

/// The cliff's face is the rock layer, the valley the grass layer: the
/// stamp is drawn with its terrain's material, by its own normal.
#[test]
fn a_stamp_is_shaded_by_its_terrains_layers() {
    let check = check("terrain-cliff");
    assert!(check.stats.terrain_draws > 0);
    let scene = ts::cliff_scene(0.0);
    let (img, _) = render(&scene);
    keep("cliff", &img);
    let face = color_at(
        &img,
        project(&scene.camera, Vec3::new(ts::CLIFF_ROCK, 6.0, 0.0)),
    );
    let valley = color_at(
        &img,
        project(&scene.camera, Vec3::new(ts::CLIFF_ROCK + 8.0, 0.0, 0.0)),
    );
    assert!(
        valley[1] > valley[0] * 1.5,
        "the valley is grass: {valley:?}"
    );
    assert!(
        (face[1] - face[0]).abs() < face[0] * 0.2 && face[0] > 20.0,
        "the face is grey rock, drawn: {face:?}"
    );
}

/// Across the seam where the face meets the valley: with blend 0 the
/// rock's lowest half metre and the grass beside it differ in luma by more
/// than 0.1; with blend 1 the face's foot takes the ground's look and
/// normal, and they differ by under 0.05.
#[test]
fn a_blended_stamp_meets_the_ground_without_a_seam() {
    let check = check("terrain-cliff-blend");
    assert!(check.stats.terrain_draws > 0);
    let seam = |blend: f32| {
        let scene = ts::cliff_scene(blend);
        let (img, _) = render(&scene);
        keep(&format!("cliff-seam-{blend}"), &img);
        let rock = luma_at(&img, &scene.camera, Vec3::new(ts::CLIFF_ROCK, 0.3, 0.0));
        let ground = luma_at(
            &img,
            &scene.camera,
            Vec3::new(ts::CLIFF_ROCK + 1.2, 0.0, 0.0),
        );
        (rock - ground).abs()
    };
    let (hard, blended) = (seam(0.0), seam(1.0));
    assert!(hard > 0.1, "blend 0 leaves a seam: luma {hard}");
    assert!(blended < 0.05, "blend 1 hides it: luma {blended}");
}
