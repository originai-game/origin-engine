//! The render graph object: passes ordered by the targets they read and
//! write, refusals that name what is wrong, transient aliasing, and the
//! graph a real frame was recorded from.

use og_render::wgpu::{TextureFormat, TextureUsages};
use og_render::{GraphError, RenderGraph, RenderTarget, Renderer, TargetDesc};
use og_tests::scenes;

fn desc(width: u32) -> TargetDesc {
    TargetDesc {
        width,
        height: width,
        format: TextureFormat::Rgba16Float,
        samples: 1,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
    }
}

/// Declared out of order, the passes run in data-flow order; declared in a
/// valid order, they keep it.
#[test]
fn passes_run_in_data_flow_order() {
    let mut g = RenderGraph::new();
    g.transient("lit", desc(64))
        .transient("blur", desc(32))
        .import("output")
        .pass("tonemap", &["lit", "blur"], &["output"])
        .pass("blur", &["lit"], &["blur"])
        .pass("main", &[], &["lit"]);
    let plan = g.compile().expect("compiles");
    assert_eq!(plan.ordered_names(), ["main", "blur", "tonemap"]);

    let mut g = RenderGraph::new();
    g.transient("a", desc(64))
        .import("output")
        .pass("first", &[], &["a"])
        .pass("second", &["a"], &["output"])
        .pass("third", &[], &["output"]);
    assert_eq!(
        g.compile().expect("compiles").ordered_names(),
        ["first", "second", "third"]
    );
}

#[test]
fn a_cycle_is_refused_with_its_passes() {
    let mut g = RenderGraph::new();
    g.transient("x", desc(64))
        .transient("y", desc(64))
        .pass("a", &["y"], &["x"])
        .pass("b", &["x"], &["y"]);
    let err = g.compile().expect_err("a cycle");
    match &err {
        GraphError::Cycle(passes) => {
            assert_eq!(passes.first(), passes.last(), "{passes:?} closes");
            let mut names: Vec<&str> = passes[1..].iter().map(String::as_str).collect();
            names.sort();
            assert_eq!(names, ["a", "b"]);
        }
        other => panic!("expected a cycle, got {other:?}"),
    }
    let text = err.to_string();
    assert!(
        text.contains("cycle") && text.contains("a") && text.contains("b"),
        "{text}"
    );
}

#[test]
fn a_missing_target_is_refused_with_the_pass_and_the_name() {
    let mut g = RenderGraph::new();
    g.import("output").pass("tonemap", &["hdr"], &["output"]);
    assert_eq!(
        g.compile().expect_err("missing"),
        GraphError::MissingTarget {
            pass: "tonemap".into(),
            target: "hdr".into()
        }
    );
    let mut g = RenderGraph::new();
    g.transient("hdr", desc(64))
        .import("output")
        .pass("tonemap", &["hdr"], &["output"]);
    assert_eq!(
        g.compile().expect_err("never written"),
        GraphError::NeverWritten {
            target: "hdr".into(),
            pass: "tonemap".into()
        }
    );
    let mut g = RenderGraph::new();
    g.import("output").import("output");
    assert_eq!(
        g.compile().expect_err("twice"),
        GraphError::DuplicateTarget("output".into())
    );
}

/// Transient targets of one description whose lifetimes do not overlap
/// share a texture; overlapping ones do not.
#[test]
fn transients_alias_when_their_lifetimes_do_not_overlap() {
    let mut g = RenderGraph::new();
    g.transient("a", desc(64))
        .transient("b", desc(64))
        .transient("c", desc(64))
        .transient("small", desc(32))
        .import("output")
        .pass("make-a", &[], &["a"])
        .pass("a-to-b", &["a"], &["b"])
        .pass("b-to-small", &["b"], &["small"])
        .pass("small-to-c", &["small"], &["c"])
        .pass("c-out", &["c"], &["output"]);
    let plan = g.compile().expect("compiles");
    assert_ne!(plan.slot("a"), plan.slot("b"), "a and b are alive together");
    assert_eq!(plan.slot("a"), plan.slot("c"), "a is dead when c is made");
    assert_eq!(plan.physical().len(), 3, "a/c, b, small");
    assert_eq!(
        plan.slot("output"),
        None,
        "imported targets are not allocated"
    );
}

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
    assert!(!renderer
        .frame_graph()
        .expect("a graph")
        .ordered_names()
        .iter()
        .any(|n| n.starts_with("bloom")));
}
