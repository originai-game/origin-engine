//! The render graph object: passes ordered by the targets they read and
//! write, refusals that name what is wrong, transient aliasing, and the
//! graph a real frame was recorded from.

use og_render::{RenderTarget, Renderer};
use og_tests::scenes;

/// The renderer records a frame from its graph: the showcase's passes in
/// order (shadows, main, TAA, metering, bloom down and up, tonemap) and the
/// transient textures it allocated.
#[test]
fn the_showcase_frame_is_recorded_from_its_graph() {
    let mut built = scenes::build("showcase").expect("scene");
    let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
    let stats = renderer
        .render(&built.scene, &RenderTarget::Offscreen)
        .expect("render");
    let plan = renderer.frame_graph().expect("a graph");
    let names = plan.ordered_names();
    println!("showcase graph: {}", names.join(" -> "));
    let position = |n: &str| {
        names
            .iter()
            .position(|m| *m == n)
            .unwrap_or_else(|| panic!("no pass {n} in {names:?}"))
    };
    for c in 0..3 {
        assert!(position(&format!("cascade.{c}")) < position("main"));
    }
    assert!(position("main") < position("tonemap"));
    assert!(position("bloom-prefilter") < position("bloom-down.0"));
    assert!(position("bloom-down.0") < position("bloom-up.0"));
    assert!(position("bloom-up.0") < position("tonemap"));
    assert!(position("meter") < position("bloom-prefilter"));
    assert_eq!(names.last(), Some(&"tonemap"));
    assert_eq!(stats.graph_passes as usize, names.len());
    assert_eq!(stats.transient_textures as usize, plan.physical().len());
    // HDR, depth, three metering levels and the bloom chain.
    assert!(
        stats.transient_textures >= 5 + 3,
        "{} transients",
        stats.transient_textures
    );
    // Bloom off: its passes and targets leave the graph.
    built.scene.post.bloom = None;
    let without = renderer
        .render(&built.scene, &RenderTarget::Offscreen)
        .expect("render");
    assert!(without.graph_passes < stats.graph_passes);
    assert!(without.transient_textures < stats.transient_textures);
    assert!(
        !renderer
            .frame_graph()
            .expect("a graph")
            .ordered_names()
            .iter()
            .any(|n| n.starts_with("bloom"))
    );
}

/// Half-resolution ambient occlusion (`AoResolution::Half`) searches and
/// blurs a quarter of the pixels, then an `ao-upsample` pass writes the
/// full-resolution `ao` the main pass reads; the default keeps today's two
/// full-resolution passes.
#[test]
fn half_resolution_ao_adds_its_upsample_and_halves_its_targets() {
    let mut built = scenes::build("ao-gtao").expect("scene");
    let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
    let graph = |renderer: &mut Renderer, scene: &og_render::RenderScene| {
        renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
        let plan = renderer.frame_graph().expect("a graph");
        let names: Vec<String> = plan.ordered_names().iter().map(|n| n.to_string()).collect();
        let size = |t: &str| {
            plan.slot(t)
                .map(|s| (plan.physical()[s].width, plan.physical()[s].height))
        };
        (names, size("ao.raw"), size("ao.blur"), size("ao"))
    };
    let (w, h) = (built.desc.width, built.desc.height);

    let (names, raw, blur, ao) = graph(&mut renderer, &built.scene);
    assert!(names.iter().any(|n| n == "ao-blur"), "{names:?}");
    assert!(!names.iter().any(|n| n == "ao-upsample"), "{names:?}");
    assert_eq!((raw, blur, ao), (Some((w, h)), None, Some((w, h))));

    built.scene.post.ao.as_mut().expect("ao").resolution = og_render::AoResolution::Half;
    let (names, raw, blur, ao) = graph(&mut renderer, &built.scene);
    let at = |n: &str| {
        names
            .iter()
            .position(|m| m == n)
            .unwrap_or_else(|| panic!("no {n} in {names:?}"))
    };
    assert!(at("ao") < at("ao-blur") && at("ao-blur") < at("ao-upsample"));
    assert!(at("ao-upsample") < at("main"));
    let half = Some((w.div_ceil(2), h.div_ceil(2)));
    assert_eq!((raw, blur, ao), (half, half, Some((w, h))));
}
