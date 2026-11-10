//! Copies of one material aliased together (`RenderScene::alias_material`)
//! draw in one batch while they draw the same. Equal parameters now do not
//! make equal previous ones (`previous_shader_params()`, which the prepass's
//! motion vectors read), so in the frame a shader parameter of either
//! moves, each draws with its own uniform, and they batch again from the
//! next frame. A fragment hook paints the comparison, so it reads back
//! exactly.

use glam::{Mat4, Vec3};
use og_render::{
    Camera, MaterialHandle, MaterialShader, MeshData, ParamKind, Projection, RenderDesc,
    RenderScene, RenderTarget, Renderer, ShaderMode, ShaderParam, StandardMaterial, Vertex,
};

/// Red where the previous parameters differ from the current ones, green
/// where they agree.
const HOOK: &str = "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {
  let moved = previous_shader_params().v != p.v;
  return vec4<f32>(select(0.0, 1.0, moved), select(1.0, 0.0, moved), 0.0, 1.0);
}
";

const SIZE: u32 = 32;

/// Two quads of one mesh, `a` on the left and `b` on the right, each with
/// its own copy of one material with the hook, `b` aliased to `a`.
fn stage() -> (RenderScene, MaterialHandle, MaterialHandle) {
    let mut scene = RenderScene::new();
    let mut quad = MeshData::default();
    for position in [
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ] {
        quad.vertices.push(Vertex {
            position,
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        });
    }
    quad.indices = vec![0, 1, 2, 0, 2, 3];
    let mesh = scene.add_mesh(quad);
    let shader = scene.add_shader(MaterialShader {
        name: "moved".into(),
        code: HOOK.into(),
        params: vec![ShaderParam {
            name: "v".into(),
            kind: ParamKind::Float,
            default: [0.0; 4],
        }],
        mode: ShaderMode::Hooks,
    });
    let [a, b] = [(); 2].map(|_| {
        let m = scene.add_standard_material(StandardMaterial {
            unlit: 1,
            ..StandardMaterial::default()
        });
        assert!(scene.set_material_shader(m, Some(shader)));
        m
    });
    assert!(scene.alias_material(b, a));
    scene.add_instance(mesh, a, Mat4::from_translation(Vec3::new(-1.0, 0.0, 0.0)));
    scene.add_instance(mesh, b, Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)));
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 3.0),
        target: Vec3::ZERO,
        projection: Projection::Orthographic { height: 4.0 },
        ..Default::default()
    };
    scene.light.cast_shadows = false;
    (scene, a, b)
}

/// Draws a frame and checks what it drew: whether `a`'s and `b`'s
/// parameters read as moved, and in how many (mesh, material) batches.
fn expect(renderer: &mut Renderer, scene: &RenderScene, at: &str, want: (bool, bool, u32)) {
    let stats = renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("capture");
    // Each quad's centre: a quarter of the width in from its side.
    let moved = |x: u32| {
        let i = ((SIZE / 2 * SIZE + x) * 4) as usize;
        let c = &image.pixels[i..i + 3];
        assert!(
            (c[0] > 150) != (c[1] > 150),
            "{at}: red or green, not {c:?}"
        );
        c[0] > 150
    };
    let got = (moved(SIZE / 4), moved(SIZE * 3 / 4), stats.batches);
    assert_eq!(got, want, "{at}: (a moved, b moved, batches)");
}

fn renderer() -> Renderer {
    Renderer::new_blocking(&RenderDesc {
        width: SIZE,
        height: SIZE,
        ..Default::default()
    })
    .expect("renderer")
}

/// The maintainer's case on #16: `a` moves and parts from `b`; then `b`
/// moves to `a`'s value. Joining `a`'s batch in that frame would draw `b`
/// with `a`'s previous parameters, which already match: no motion where
/// `b` moved.
#[test]
#[ignore = "requires GPU"]
fn a_copy_that_moves_into_an_alias_draws_its_own_motion_in_that_frame() {
    let (mut scene, a, b) = stage();
    let mut renderer = renderer();
    expect(&mut renderer, &scene, "aliased", (false, false, 1));
    assert!(scene.set_shader_param(a, "v", &[1.0]));
    expect(&mut renderer, &scene, "a moves, parting", (true, false, 2));
    expect(&mut renderer, &scene, "parted, still", (false, false, 2));
    assert!(scene.set_shader_param(b, "v", &[1.0]));
    expect(&mut renderer, &scene, "b moves to a", (false, true, 2));
    expect(&mut renderer, &scene, "the frame after", (false, false, 1));
}

/// The other way round: `b` moves and parts from `a`; then `a`, the one
/// `b` is aliased to, moves to `b`'s value. Drawing `b` in `a`'s batch in
/// that frame would give `b` the motion only `a` has.
#[test]
#[ignore = "requires GPU"]
fn a_copy_that_stays_still_draws_no_motion_when_its_alias_moves_to_it() {
    let (mut scene, a, b) = stage();
    let mut renderer = renderer();
    expect(&mut renderer, &scene, "aliased", (false, false, 1));
    assert!(scene.set_shader_param(b, "v", &[1.0]));
    expect(&mut renderer, &scene, "b moves, parting", (false, true, 2));
    expect(&mut renderer, &scene, "parted, still", (false, false, 2));
    assert!(scene.set_shader_param(a, "v", &[1.0]));
    expect(&mut renderer, &scene, "a moves to b", (true, false, 2));
    expect(&mut renderer, &scene, "the frame after", (false, false, 1));
}
