//! CSS paint in documents (docs/ui.md, CSS paint): gradient backgrounds
//! (linear, radial, conic, repeating), `mask-image`, `clip-path` shapes,
//! `text-shadow`, `filter` chains (a drop shadow that follows a clipped
//! star's alpha, colour functions in order, opacity and blur), and
//! `backdrop-filter` colour functions over stripes, and image layers
//! (`url()` of texture assets) sized, placed and repeated: 16 px tiles cut
//! by a radius, a cover photo under a badge 6 px from its corner, a
//! contained icon over a gradient, and tiles spaced edge to edge.
//! The frames match `references/vulkan/css-paint-{1280,390}.png`
//! (`OG_UI_CAPTURE=1` records them).
//!
//! Ways it could fail: a gradient's angle read in maths' convention or not
//! lengthened to the corners of a wide box, a clip's percentages read
//! against the wrong box (or the clip drawn uncut), a mask that fades the
//! wrong way, or a retained element whose resolved paint rebuilds every
//! frame (idle must build nothing).

use crate::support;
use og_ui::prelude::*;

use og_ui::*;
use support::*;

#[test]
fn image_urls_resolve_files_not_atlas_names_or_handles() {
    use og_draw::UiPaint;
    let mut handles = Handles::default();
    let mut ui = ui_with_font(test_font_metrics());
    let mut textures = Vec::new();
    for name in ["coin.svg", "7", "atlas"] {
        let texture = handles.texture();
        ui.define_texture(&format!("ui/{name}"), texture);
        textures.push(texture);
    }
    ui.set_icons(IconAtlas {
        texture: Some(textures[2]),
        columns: 1,
        rows: 1,
        names: vec!["coin.svg".into()],
        ..Default::default()
    });
    ui.define_document(
        UiDocument::parse(
            "ui/urls.svelte",
            r#"
<meta name="component" content="urls" />
<section><div id="file"></div><div id="missing"></div><div id="number"></div></section>
<style>
div { width: 8px; height: 8px; }
#file { background-image: url(coin.svg); }
#missing { background-image: url(missing/coin.svg); }
#number { background-image: url(7); }
</style>"#,
        )
        .unwrap(),
    );
    ui.mount_document(
        reactive::SourceId::new("test", "urls"),
        "urls",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    ui.begin(InputFrame::default(), Viewport::new(100.0, 100.0));
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    let images: Vec<_> = ui
        .frame()
        .items
        .iter()
        .filter_map(|item| match &item.cmd {
            DrawCmd::Shape { shape, .. } => match &shape.paint {
                UiPaint::Image(image) => Some(image.texture),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(images, textures[..2]);
    assert!(
        ui.frame()
            .warnings
            .iter()
            .any(|w| w.contains("missing/coin.svg"))
    );
    assert!(!ui.frame().warnings.iter().any(|w| w.contains("url(7)")));
}

#[test]
fn image_sets_reselect_on_density_changes() {
    use og_draw::UiPaint;
    let mut handles = Handles::default();
    let mut ui = ui_with_font(test_font_metrics());
    let mut textures = Vec::new();
    for name in ["low.png", "high.png"] {
        let texture = handles.texture();
        ui.define_texture(&format!("ui/{name}"), texture);
        textures.push(texture);
    }
    ui.define_document(
        UiDocument::parse(
            "ui/set.svelte",
            r#"
<meta name="component" content="set" />
<div id="image"></div>
<style>
#image { width: 32px; height: 32px;
  background-image: image-set("low.png" calc(2dppx - 96dpi),
    url(high.png) clamp(1x, calc(480dpi / 2), 2dppx));
  background-repeat: no-repeat; }
</style>"#,
        )
        .unwrap(),
    );
    ui.mount_document(
        reactive::SourceId::new("test", "set"),
        "set",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    for (dpr, chosen) in [(1.0, 0), (1.5, 1), (3.0, 1), (0.75, 0)] {
        let mut viewport = Viewport::new(100.0, 100.0);
        viewport.dpr = dpr;
        ui.begin(InputFrame::default(), viewport);
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let images: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter_map(|item| match &item.cmd {
                DrawCmd::Shape { shape, .. } => match &shape.paint {
                    UiPaint::Image(image) => Some((image.texture, image.density)),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        assert_eq!(
            images,
            [(textures[chosen], (chosen + 1) as f32)],
            "DPR {dpr}"
        );
    }
}
