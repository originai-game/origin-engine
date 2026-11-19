//! The mini-game tier (`docs/design.md` §2.6 Tiers) on a full device.
//!
//! The tier is named by the host (`RenderDesc::tier`, which the mini-game
//! adapter sets), never read off the device: WeChat ignores
//! `requiredLimits` and its adapter may report limits that allow several
//! colour targets while only `colorAttachments[0]` draws. Without MRT the
//! depth prepass draws its packed form (post/prepass_form.rs), and it
//! must keep every pixel the three-target prepass keeps, the HLOD
//! cross-fade's dither (pbr_prepass.wgsl `reveal_cut`) included.

use glam::{Mat4, Vec3};
use og_render::{
    Camera, Caps, InstanceFx, MeshData, PostShader, Projection, RenderDesc, RenderScene,
    RenderTarget, Renderer, StandardMaterial, Tier, Vertex,
};

const SIZE: u32 = 64;

/// Red where the prepass's linear depth holds a surface, green where its
/// normal faces the camera: what a pass that reads the prepass sees.
const COVERAGE: &str = "fn post(f: PostInput, p: Params) -> vec4<f32> {
  let near = depth(f.uv) < 100.0;
  let facing = normal(f.uv).z > 0.5;
  return vec4<f32>(select(0.0, 50.0, near), select(0.0, 50.0, facing), 0.0, 1.0);
}
";

/// A quad wider than the view, facing the camera, each fade `(reveal,
/// fade_out)` an instance of it at the same place.
fn scene(fades: &[(f32, bool)]) -> RenderScene {
    let mut scene = RenderScene::new();
    let mut quad = MeshData::default();
    for position in [
        [-4.0, -4.0, 0.0],
        [4.0, -4.0, 0.0],
        [4.0, 4.0, 0.0],
        [-4.0, 4.0, 0.0],
    ] {
        quad.vertices.push(Vertex {
            position,
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        });
    }
    quad.indices = vec![0, 1, 2, 0, 2, 3];
    let mesh = scene.add_mesh(quad);
    let material = scene.add_standard_material(StandardMaterial::default());
    for &(reveal, fade_out) in fades {
        scene.add_instance(mesh, material, Mat4::IDENTITY);
        scene.objects.instances.last_mut().expect("added").fx = InstanceFx {
            reveal,
            fade_out,
            ..InstanceFx::NONE
        };
    }
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 0.0, 3.0),
        target: Vec3::ZERO,
        projection: Projection::Orthographic { height: 2.0 },
        ..Default::default()
    };
    scene.lighting.light.cast_shadows = false;
    let post = scene.add_post(PostShader {
        name: "coverage".into(),
        code: COVERAGE.into(),
        params: Vec::new(),
    });
    scene.look.custom_post.push(post);
    scene
}

fn renderer(tier: Tier) -> Renderer {
    pollster::block_on(Renderer::new(&RenderDesc {
        width: SIZE,
        height: SIZE,
        tier,
        ..Default::default()
    }))
    .expect("renderer")
}

/// Each pixel: (the prepass holds a surface, its normal faces the camera).
fn coverage(renderer: &mut Renderer, scene: &RenderScene) -> Vec<(bool, bool)> {
    renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("capture");
    image
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| (c[0] > 128, c[1] > 128))
        .collect()
}

/// A device whose limits allow MRT, made for the mini-game tier, draws
/// the tier's schedule: no MRT, asynchronous pipelines or timestamps in
/// its capabilities, the prepass and every pass that reads it present,
/// and a frame that reads the prepass drawn.
#[test]
#[ignore = "requires GPU"]
fn the_named_tier_rules_over_the_device_limits() {
    let mut full = renderer(Tier::Full);
    if !full.info().caps.contains(Caps::MRT) {
        return eprintln!("skipped: this device has no MRT to rule over");
    }
    let mut mini = renderer(Tier::MiniGame);
    assert_eq!(mini.info().tier, Tier::MiniGame);
    for lacking in [Caps::MRT, Caps::ASYNC_PIPELINES, Caps::TIMESTAMPS] {
        assert!(!mini.info().caps.contains(lacking), "{lacking:?}");
    }
    let absent: Vec<_> = mini.absent_passes().map(|(n, _)| n).collect();
    for kept in ["prepass", "view", "contact"] {
        assert!(!absent.contains(&kept), "{kept} absent: {absent:?}");
    }
    let whole = scene(&[(1.0, false)]);
    let drawn = coverage(&mut mini, &whole);
    assert!(
        drawn.iter().all(|&c| c == (true, true)),
        "the prepass lost the quad"
    );
    assert_eq!(drawn, coverage(&mut full, &whole));
}

/// The packed prepass cuts exactly the pixels the three-target one cuts
/// for an instance fading in and one fading out, and the two sides of a
/// cross-fade keep each pixel once.
#[test]
#[ignore = "requires GPU"]
fn the_packed_prepass_keeps_the_cross_fades_dither() {
    let mut full = renderer(Tier::Full);
    let mut mini = renderer(Tier::MiniGame);
    let covered = |c: &[(bool, bool)]| c.iter().filter(|c| c.0).count();
    for side in [(0.35, false), (0.35, true), (0.65, true)] {
        let one = scene(&[side]);
        let packed = coverage(&mut mini, &one);
        assert_eq!(packed, coverage(&mut full, &one), "{side:?}");
        let n = covered(&packed);
        assert!(
            n > 0 && n < packed.len(),
            "{side:?}: {n} of {} kept",
            packed.len()
        );
    }
    let incoming = coverage(&mut mini, &scene(&[(0.35, false)]));
    let outgoing = coverage(&mut mini, &scene(&[(0.65, true)]));
    for (i, (a, b)) in incoming.iter().zip(&outgoing).enumerate() {
        assert!(a.0 != b.0, "pixel {i}: drawn by both or neither");
    }
}
