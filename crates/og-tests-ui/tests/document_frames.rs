//! Every engine UI document (`packages/engine`: the components, their groups
//! included, and the screens) built from its `fixture:default` at
//! 1280×720 and compared with `references/vulkan/doc-<path>.png`
//! (og-tests' tolerance; `OG_UI_CAPTURE=1` records). What only drawing
//! shows: how the default style dresses each document's own CSS, such as
//! a button's own box restyling its plate.
//!
//! The documents in [`PLACEHOLDERS`] draw nothing from their fixture alone,
//! so they are skipped by name until a preview gives them their context: a
//! black frame would only record that they draw nothing.

use crate::support;

use og_render::RenderScene;
use og_ui::{Rect, Srgba, Ui, WorldTarget};
use support::documents::populated_preview;
use support::*;

/// Documents (`doc-<path>`) with no reference yet, and why their
/// `fixture:default` alone draws nothing.
const PLACEHOLDERS: &[(&str, &str)] = &[
    ("doc-components-hud-damage-numbers", WORLD),
    ("doc-components-world-interaction-prompt", WORLD),
    ("doc-components-world-overhead-bars", WORLD),
    ("doc-screens-calendar", VIEW_MODEL),
    ("doc-screens-confirm", VIEW_MODEL),
    ("doc-screens-data-list", VIEW_MODEL),
    ("doc-screens-key-values", VIEW_MODEL),
    ("doc-screens-objective-tracker", VIEW_MODEL),
    ("doc-screens-pagination", VIEW_MODEL),
    ("doc-screens-star-rating", VIEW_MODEL),
    ("doc-screens-tree", VIEW_MODEL),
    ("doc-screens-hud-editor", SCREEN),
    ("doc-screens-notifications", SCREEN),
    ("doc-screens-splash", SCREEN),
    ("doc-screens-subtitles", SCREEN),
];
const WORLD: &str = "world-anchored: it draws at entities the preview's world does not have";
const VIEW_MODEL: &str = "a component that reads its data as `$name` from a view model, which a component's props do not set";
const SCREEN: &str = "a screen that draws nothing from its fixture, pushed or built as a component";

/// The preview's panel ground belongs below the scene, not a screen
/// element: anchored UI must paint above it and below screen UI.
fn render_document(
    ui: &og_ui::Ui,
    doc: &EngineDocument,
    scene: &mut RenderScene,
    width: u32,
    height: u32,
    picture: Option<og_draw::TextureHandle>,
) -> Rendered {
    let world = doc.path.contains("/world/");
    let mut overlay = og_render::Overlay::new(width as f32, height as f32, 1.0);
    let mut ground = og_draw::UiShape::rect(
        [0.0, 0.0, width as f32, height as f32],
        0.0,
        if world {
            Srgba::rgb(5, 7, 12).linear()
        } else {
            ui.theme().palette.panel.linear()
        },
    );
    ground.layer = -og_ui::LAYER_BASE;
    if let Some(texture) = picture {
        ground.paint = og_draw::UiPaint::Image(og_draw::UiImage {
            texture,
            uv: [0.0, 0.0, 1.0, 1.0],
            size: og_core::glam::Vec2::new(960.0, 540.0),
            density: 1.0,
            tint: [1.0; 4],
            fit: og_draw::UiFit::Cover,
            focal: og_core::glam::Vec2::splat(0.5),
            offset: og_core::glam::Vec2::ZERO,
            luminance: false,
            nearest: false,
            orient: true,
        });
    }
    overlay.ui.shapes.push(ground);
    ui.frame().emit(&og_ui::EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    on_backdrop(ui, scene);
    let mut rendered = render_frame(scene, &sized(width, height));
    if world {
        // Measure actual paint before adding guides: transparent parts of a
        // ground's square do not add acres of empty stage to its preview.
        let mut bounds: Option<Rect> = None;
        let union = |a: Rect, b: Rect| {
            let (x, y) = (a.x.min(b.x), a.y.min(b.y));
            Rect::new(
                x,
                y,
                a.right().max(b.right()) - x,
                a.bottom().max(b.bottom()) - y,
            )
        };
        for (i, p) in rendered.image.pixels.as_chunks::<4>().0.iter().enumerate() {
            if p[..3]
                .iter()
                .zip([5_u8, 7, 12])
                .any(|(a, b)| a.abs_diff(b) > 1)
            {
                let pixel = Rect::new(
                    (i as u32 % width) as f32,
                    (i as u32 / width) as f32,
                    1.0,
                    1.0,
                );
                bounds = Some(bounds.map_or(pixel, |b| union(b, pixel)));
            }
        }
        let overlay = scene.ui.overlay.as_mut().unwrap();
        if let Some(marker) = world_guides(ui, doc, overlay) {
            bounds = Some(bounds.map_or(marker, |b| union(b, marker)));
        }
        rendered = render_frame(scene, &sized(width, height));
        if !ui.world_anchor_points().iter().any(|a| a.clamped)
            && let Some(bounds) = bounds
        {
            let pad = (24.0 * ui.theme().text_scale).ceil();
            let x = (bounds.x.floor() - pad).max(0.0) as u32;
            let y = (bounds.y.floor() - pad).max(0.0) as u32;
            let right = (bounds.x + bounds.width + pad).ceil().min(width as f32) as u32;
            let bottom = (bounds.y + bounds.height + pad).ceil().min(height as f32) as u32;
            let image = &rendered.image;
            let mut pixels = Vec::with_capacity(((right - x) * (bottom - y) * 4) as usize);
            for row in y..bottom {
                let start = ((row * width + x) * 4) as usize;
                pixels.extend_from_slice(&image.pixels[start..start + ((right - x) * 4) as usize]);
            }
            rendered.image = og_render::RgbaImage {
                width: right - x,
                height: bottom - y,
                pixels,
            };
        }
    }
    rendered
}

/// Guides belong to the preview renderer, never the document or its findings.
fn world_guides(ui: &Ui, doc: &EngineDocument, overlay: &mut og_render::Overlay) -> Option<Rect> {
    use og_core::glam::{Vec2, Vec3};
    let view = ui.world_view()?;
    let (width, height) = (overlay.width, overlay.height);
    let base = ui
        .world_anchor_points()
        .iter()
        .find_map(|a| match a.target {
            WorldTarget::Point(p) => Some(p),
            WorldTarget::Entity(_) => None,
        })
        .or_else(|| {
            let p: Vec<f32> = doc
                .props
                .get("at")?
                .as_str()?
                .split_whitespace()
                .map(str::parse)
                .collect::<Result<_, _>>()
                .ok()?;
            (p.len() == 3).then(|| Vec3::new(p[0], p[1], p[2]))
        })?;
    let mut line = |a: Vec3, b: Vec3, alpha: f32| -> Option<Rect> {
        let (x0, y0, _) = view.project(a, width, height)?;
        let (x1, y1, _) = view.project(b, width, height)?;
        let a = Vec2::new(x0, y0);
        let b = Vec2::new(x1, y1);
        let delta = b - a;
        let length = delta.length();
        if length < 0.01 {
            return None;
        }
        let direction = delta / length;
        // These guides live on a fixed dark stage. Composite their sRGB ink
        // there so a six-percent grid stays faint under linear-light drawing.
        let ink = Srgba::rgb(255, 255, 255)
            .fade(alpha)
            .over(Srgba::rgb(5, 7, 12))
            .linear();
        let mut shape = og_draw::UiShape::rect([0.0, 0.0, length, 1.0], 0.0, ink);
        shape.center = (a + b) * 0.5;
        shape.transform = [direction.x, direction.y, -direction.y, direction.x];
        shape.layer = -og_ui::LAYER_BASE + 1;
        overlay.ui.shapes.push(shape);
        Some(Rect::new(
            x0.min(x1) - 0.5,
            y0.min(y1) - 0.5,
            (x1 - x0).abs() + 1.0,
            (y1 - y0).abs() + 1.0,
        ))
    };
    for axis in 0..2 {
        for across in -30..=30 {
            for along in -30..30 {
                let midpoint = Vec2::new(across as f32, along as f32 + 0.5);
                let fade = (1.0 - midpoint.length() / 30.0).max(0.0);
                if fade == 0.0 {
                    continue;
                }
                let point = |along| {
                    if axis == 0 {
                        Vec3::new(across as f32, 0.0, along)
                    } else {
                        Vec3::new(along, 0.0, across as f32)
                    }
                };
                let origin = Vec3::new(base.x, 0.0, base.z);
                line(
                    origin + point(along as f32),
                    origin + point(along as f32 + 1.0),
                    fade * if across % 5 == 0 { 0.1 } else { 0.06 },
                );
            }
        }
    }
    line(base, base + Vec3::Y * 1.8, 0.12)
}

#[test]
#[ignore = "requires GPU"]
fn engine_documents_match_their_reference_frames() {
    let (docs, messages) = engine_documents(&["src/ui/components", "src/ui/screens"]);
    let (width, height) = (1280, 720);
    let mut failures = Vec::new();
    let name_of = |path: &str| {
        // `src/ui/components/rpg/party.svelte` is `doc-components-rpg-party`.
        format!(
            "doc-{}",
            path.trim_start_matches("src/ui/")
                .trim_end_matches(".svelte")
                .replace('/', "-")
        )
    };
    for (name, _) in PLACEHOLDERS {
        assert!(
            docs.iter().any(|d| name_of(&d.path) == *name),
            "{name} is a placeholder for a document that no longer exists"
        );
    }
    for doc in &docs {
        let name = name_of(&doc.path);
        if let Some((_, why)) = PLACEHOLDERS.iter().find(|(n, _)| *n == name) {
            eprintln!("{name}: no reference yet: {why}");
            continue;
        }
        let mut scene = RenderScene::new();
        let mut ui = ui_with_font(add_test_font(&mut scene));
        let picture = populated_preview(
            &mut ui,
            &mut scene,
            &docs,
            doc,
            &messages,
            width as f32,
            height as f32,
        );
        let rendered = render_document(&ui, doc, &mut scene, width, height, picture);
        if let Err(e) = check_reference(&name, &rendered.image, &rendered.info, Against::Recorded) {
            failures.push(e);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
