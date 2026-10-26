//! Material shader hooks beyond the lit pass (phase 2, owed row 23): on
//! skinned meshes, and in the shadow passes and the depth prepass. A mesh
//! moved by its shader's `vertex_offset` must draw exactly as the same mesh
//! moved by its transform: its pixels, its shadow on the ground (the shadow
//! passes) and its ambient occlusion and outline (the prepass).

use glam::{Mat4, Vec3};
use og_render::{
    AmbientOcclusion, AoMethod, MaterialHandle, RenderScene, RenderTarget, Renderer, RgbaImage,
    SkinnedInstance, StandardMaterial, shapes,
};
use og_tests::compare::{Tolerance, compare};
use og_tests::{fx_scenes, gfx, p2_scenes, scenes};
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

/// Pixels with a channel more than 8 apart, and the RMSE.
fn diff(a: &RgbaImage, b: &RgbaImage) -> (usize, f64) {
    let t = Tolerance {
        channel: 8,
        ..Tolerance::default()
    };
    let c = compare(&a.pixels, &b.pixels, a.width, a.height, &t).expect("same size");
    (c.outliers, c.rmse)
}

/// A plain ground under the default sun, ambient occlusion on.
fn stage() -> RenderScene {
    let mut scene = RenderScene::new();
    let ground = scene.add_mesh(shapes::plane(30.0, 8.0));
    let grey = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.6),
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(ground, grey, Mat4::IDENTITY);
    scene.post.ao = Some(AmbientOcclusion {
        radius: 0.6,
        intensity: 1.2,
        method: AoMethod::Gtao,
        direct: 0.0,
        radius_scale: 0.0,
    });
    scene.camera.position = Vec3::new(0.0, 3.0, 7.0);
    scene.camera.target = Vec3::new(0.0, 0.8, 0.0);
    scene
}

/// A material with the ripple shader (no ripple), outlined, moved by `offset`.
/// Its stripes are the material's own colour: the cylinders' caps lie on a
/// band edge, where the band flips with the last bit of the position and
/// lavapipe's rounding differs between the two placements.
fn rippled(scene: &mut RenderScene, offset: Vec3) -> MaterialHandle {
    let color = Vec3::new(0.8, 0.35, 0.2);
    let shader = scene.add_shader(p2_scenes::ripple_shader());
    let m = scene.add_standard_material(StandardMaterial {
        color,
        roughness: 0.5,
        outlineWidth: 2.0,
        ..Default::default()
    });
    scene.set_material_shader(m, Some(shader));
    scene.set_shader_param(m, "offset", &offset.to_array());
    scene.set_shader_param(m, "stripe", &color.to_array());
    m
}

const SHIFT: Vec3 = Vec3::new(1.5, 0.6, -0.5);

/// A box moved by its material shader draws, shadows and occludes as the
/// box moved by its transform.
#[test]
fn an_offset_mesh_draws_shadows_and_occludes_where_it_moved() {
    let render = |by_shader: bool| {
        let mut scene = stage();
        let cube = scene.add_mesh(shapes::cuboid(Vec3::new(0.6, 1.6, 0.6)));
        let (offset, at) = if by_shader {
            (SHIFT, Vec3::new(-1.0, 0.8, 0.0))
        } else {
            (Vec3::ZERO, Vec3::new(-1.0, 0.8, 0.0) + SHIFT)
        };
        let m = rippled(&mut scene, offset);
        scene.add_instance(cube, m, Mat4::from_translation(at));
        frame(&scene)
    };
    let (shader, transform) = (render(true), render(false));
    let (off, rmse) = diff(&shader, &transform);
    println!("{off} pixels more than 8 apart, RMSE {rmse:.5}");
    assert!(off < 40 && rmse < 0.002, "{off} pixels, RMSE {rmse:.5}");
}

/// The same for skinned meshes: the bending cylinders moved by the shader
/// draw, shadow and occlude as the cylinders moved by their transforms.
#[test]
fn an_offset_skinned_mesh_draws_shadows_and_occludes_where_it_moved() {
    let render = |by_shader: bool| {
        let mut scene = stage();
        let (mesh, skin) = fx_scenes::skinned_cylinder(0.22, 24, 24);
        let cylinder = scene.add_skinned_mesh(mesh, skin);
        let m = rippled(&mut scene, if by_shader { SHIFT } else { Vec3::ZERO });
        for i in 0..3 {
            scene
                .joint_palettes
                .push(fx_scenes::bend_palette(-0.4 + i as f32 * 0.4, 0.3));
            let at = Vec3::new(-1.5 + i as f32 * 1.2, 0.0, 0.0);
            let at = if by_shader { at } else { at + SHIFT };
            scene.skinned_instances.push(SkinnedInstance::new(
                cylinder,
                m,
                Mat4::from_translation(at),
                i,
            ));
        }
        frame(&scene)
    };
    let (shader, transform) = (render(true), render(false));
    let (off, rmse) = diff(&shader, &transform);
    println!("{off} pixels more than 8 apart, RMSE {rmse:.5}");
    assert!(off < 40 && rmse < 0.002, "{off} pixels, RMSE {rmse:.5}");
}

/// A skinned mesh draws with its material shader's colour: the banded
/// cylinders of `skinned-hooks` are not the plain ones of `skinning`.
#[test]
fn skinned_hooks_match_their_reference() {
    match gfx::check("skinned-hooks", &out_dir()) {
        Ok(r) => println!("{r}"),
        Err(e) => panic!("{e}"),
    }
    let hooked = frame(&p2_scenes::skinned_hooks_scene());
    let plain = frame(&fx_scenes::skinning_scene());
    // Gold bands: pixels far more yellow than the plain pink flesh.
    let gold = |img: &RgbaImage| {
        img.pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[0] > 150 && p[1] > 140 && p[2] < 110)
            .count()
    };
    let (g, p) = (gold(&hooked), gold(&plain));
    println!("gold pixels: hooked {g}, plain {p}");
    assert!(
        g > p + 500,
        "the skinned cylinders were not banded ({g} against {p})"
    );
}
