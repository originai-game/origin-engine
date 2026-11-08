//! A material shader's previous parameters (`previous_shader_params()`,
//! which the motion vectors of a vertex its `vertex_offset` moved read:
//! pbr.wgsl's prepass, scatter.wgsl) are the ones the frame before drew.
//! They differ from the current ones in the frame a parameter changes
//! only, and agree again from the next frame on, also while the clock
//! stands still (a paused game, where a stamp of the scene's time would
//! keep matching), with nothing written then (engine s100: the uniform's
//! stamp names the drawn frame of the change). A fragment hook paints the
//! comparison, so it reads back exactly, which TAA's blended image would
//! not allow.

use glam::{Mat4, Vec3};
use og_render::{
    Camera, MaterialShader, MeshData, ParamKind, RenderDesc, RenderScene, RenderTarget, Renderer,
    ShaderMode, ShaderParam, StandardMaterial, Vertex,
};

/// Red where the previous parameters differ from the current ones, green
/// where they agree.
const HOOK: &str = "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {
  let moved = previous_shader_params().v != p.v;
  return vec4<f32>(select(0.0, 1.0, moved), select(1.0, 0.0, moved), 0.0, 1.0);
}
";

const SIZE: u32 = 32;

#[test]
#[ignore = "requires GPU"]
fn a_parameter_change_moves_in_its_own_frame_only() {
    let mut scene = RenderScene::new();
    let mut quad = MeshData::default();
    for position in [
        [-1.0, -1.0, 0.0],
        [1.0, -1.0, 0.0],
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
    ] {
        quad.vertices.push(Vertex {
            position,
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        });
    }
    quad.indices = vec![0, 1, 2, 0, 2, 3];
    let mesh = scene.add_mesh(quad);
    let unlit = |grey: f32| StandardMaterial {
        unlit: 1,
        color: Vec3::splat(grey),
        ..StandardMaterial::default()
    };
    let material = scene.add_standard_material(unlit(1.0));
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
    assert!(scene.set_material_shader(material, Some(shader)));
    scene.add_instance(mesh, material, Mat4::IDENTITY);
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 3.0),
        target: Vec3::ZERO,
        ..Default::default()
    };
    scene.light.cast_shadows = false;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: SIZE,
        height: SIZE,
        ..Default::default()
    })
    .expect("renderer");
    // Draws a frame: whether the quad's centre reads its parameters moved.
    let mut moved = |scene: &RenderScene, at: &str| -> bool {
        renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
        let image = renderer.read_offscreen().expect("capture");
        let i = ((SIZE / 2 * SIZE + SIZE / 2) * 4) as usize;
        let c = &image.pixels[i..i + 3];
        assert!(
            (c[0] > 150) != (c[1] > 150),
            "{at}: red or green, not {c:?}"
        );
        c[0] > 150
    };
    // The scene's tick, so its clock, stands still until said otherwise.
    assert!(!moved(&scene, "as made"));
    assert!(!moved(&scene, "drawn again"));
    assert!(scene.set_shader_param(material, "v", &[1.0]));
    assert!(moved(&scene, "the frame of the change"));
    for k in 1..=3 {
        assert!(!moved(
            &scene,
            &format!("paused, {k} frames after the change")
        ));
    }
    // A colour edit rebuilds the material's group, not its parameters'
    // history: still nothing moved...
    assert!(scene.set_standard_material(material, unlit(0.5)));
    assert!(!moved(&scene, "a colour edit"));
    // ...and with a parameter change in the same frame, that frame moves.
    scene.tick += 1;
    assert!(scene.set_standard_material(material, unlit(0.25)));
    assert!(scene.set_shader_param(material, "v", &[2.0]));
    assert!(moved(&scene, "a colour edit with a parameter change"));
    scene.tick += 1;
    assert!(!moved(&scene, "the frame after, the clock running"));
}
