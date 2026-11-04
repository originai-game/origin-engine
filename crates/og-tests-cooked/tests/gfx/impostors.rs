//! A cooked model's impostor card against the model itself
//! (og_tests_cooked::impostor_scenes).

use glam::{Vec3, Vec4};
use og_render::{RenderTarget, Renderer};
use og_tests_cooked::impostor_scenes::{CARD_AT, MESH_AT, impostor_self_shadow_scene};

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
        let c = &scene.camera;
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
