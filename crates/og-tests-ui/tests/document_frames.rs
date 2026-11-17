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
        preview(&mut ui, &docs, doc, &messages, width as f32, height as f32);
        let rendered = render_ui(&ui, &mut scene, width, height);
        if let Err(e) = check_reference(&name, &rendered.image, &rendered.info, Against::Recorded) {
            failures.push(e);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// This matrix deliberately does not use PLACEHOLDERS or references. An empty
// default fixture is an unchecked cell, not a successful geometry check.
use og_ui::{DrawCmd, InputFrame, Rect, Ui, Viewport, style_pack::StylePack};

fn geometry_packs() -> Vec<(&'static str, Option<StylePack>)> {
    let root = engine_dir("").join("../../templates");
    let mut files = Vec::new();
    let mut pending = vec![root.join("styles")];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let ty = entry.file_type().unwrap();
            let path = entry.path();
            if ty.is_dir() {
                pending.push(path);
            } else if ty.is_file() && path.extension().is_some_and(|e| e == "css") {
                files.push((
                    format!(
                        "@{}",
                        path.strip_prefix(&root)
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .replace('\\', "/")
                    ),
                    std::fs::read_to_string(path).unwrap(),
                ));
            }
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut packs = vec![("default", None)];
    for name in ["fantasy", "scifi", "cartoon"] {
        packs.push((
            name,
            Some(StylePack::parse(&format!("@styles/{name}"), files.clone()).unwrap()),
        ));
    }
    packs
}

/// The positions one point of content can occupy after scrolling the ancestors
/// visited so far. Intersect *before* going out to the next ancestor: an outer
/// scroll cannot recover something a non-scrolling inner clip already lost.
/// Test both edges independently, not whole-box containment: a long document
/// need not fit on screen at once.
fn edge_overflow(mut point: [f32; 2], axis: usize, steps: &[(Rect, [f32; 2], [f32; 2])]) -> f32 {
    for (view, offset, range) in steps {
        point[0] -= (range[axis] - offset[axis]).max(0.0);
        point[1] += offset[axis];
        let (lo, hi) = if axis == 0 {
            (view.x, view.right())
        } else {
            (view.y, view.bottom())
        };
        point[0] = point[0].max(lo);
        point[1] = point[1].min(hi);
        // One CSS pixel is the native text-cut contract's rounding allowance.
        if point[0] > point[1] + 1.0 {
            return point[0] - point[1];
        }
    }
    0.0
}

fn lost_edges(bounds: Rect, steps: &[(Rect, [f32; 2], [f32; 2])]) -> Vec<String> {
    [
        (0, bounds.x, "left"),
        (0, bounds.right(), "right"),
        (1, bounds.y, "top"),
        (1, bounds.bottom(), "bottom"),
    ]
    .into_iter()
    .filter_map(|(axis, p, edge)| {
        let overflow = edge_overflow([p, p], axis, steps);
        (overflow > 0.0).then(|| format!("{edge}: {overflow:.2}px"))
    })
    .collect()
}

fn geometry_steps(ui: &Ui, node: usize) -> Vec<(Rect, [f32; 2], [f32; 2])> {
    let nodes = ui.tree.nodes();
    let mut steps = Vec::new();
    let mut at = nodes[node].parent;
    while let Some(n) = nodes.get(at) {
        if n.scrolls() {
            let path = ui.tree.path_of(at);
            let state = ui.scroll_state(&path).expect("a laid out scroll node");
            // Layout scroll distances are local; bounds are on screen.
            let sx = if n.layout.w > 0.0 {
                n.placed.screen.width / n.layout.w
            } else {
                1.0
            };
            let sy = if n.layout.h > 0.0 {
                n.placed.screen.height / n.layout.h
            } else {
                1.0
            };
            steps.push((
                n.placed.screen,
                [state.offset_x * sx, state.offset_y * sy],
                [n.max_scroll_x() * sx, n.max_scroll() * sy],
            ));
        } else if n.decor.as_ref().is_some_and(|d| d.clip) {
            steps.push((n.placed.screen, [0.0; 2], [0.0; 2]));
        }
        if at == 0 {
            break;
        }
        at = n.parent;
    }
    let v = ui.viewport();
    steps.push((
        Rect::new(
            v.safe_area.left,
            v.safe_area.top,
            v.width - v.safe_area.horizontal(),
            v.height - v.safe_area.vertical(),
        ),
        [0.0; 2],
        [0.0; 2],
    ));
    steps
}

fn geometry_nodes(ui: &Ui) -> Vec<usize> {
    let nodes = ui.tree.nodes();
    (1..nodes.len())
        .filter(|&i| {
            if !ui.tree.index().in_tree.get(i).copied().unwrap_or(false)
                || ui.tree.kinds()[i].0 == "vacant"
            {
                return false;
            }
            let mut at = i;
            while let Some(n) = nodes.get(at) {
                if n.layout.varied.hidden.unwrap_or(n.hidden) || n.placed.alpha <= 0.0 {
                    return false;
                }
                if at == 0 {
                    break;
                }
                at = n.parent;
            }
            true
        })
        .collect()
}

fn document_geometry(ui: &Ui) -> (usize, Vec<String>) {
    let nodes = ui.tree.nodes();
    let active = geometry_nodes(ui);
    let mut failures = Vec::new();
    let mut content = 0;
    for &i in &active {
        let n = &nodes[i];
        let kind = ui.tree.kinds()[i].0;
        // Layout boxes and artwork are not content bounds. Art may deliberately
        // bleed behind a bezel; text and controls may not disappear with it.
        let text = !matches!(kind, "icon" | "image" | "art")
            && n.probe_text.chars().any(char::is_alphanumeric);
        // Semantic pictures (the crosshair, an item icon, a portrait) are
        // content too. Only their element box is checked, never the artwork's
        // crop, mask, shadows or decorative part shapes.
        let picture =
            kind != "art" && (matches!(kind, "icon" | "image") || n.a11y.role == Some("image"));
        if !text && !picture && !n.focusable && !n.el.interactive {
            continue;
        }
        let r = n.placed.screen;
        if r.width <= 0.0 || r.height <= 0.0 {
            continue;
        }
        content += 1;
        let lost = lost_edges(r, &geometry_steps(ui, i));
        if !lost.is_empty() {
            failures.push(format!(
                "{} ({kind}) {lost:?} unreachable: {r:?}",
                ui.tree.path_of(i)
            ));
        }
    }
    // Use real laid-out line boxes, not string-length estimates. Union the
    // clips of identical lines: a meter paints the same label in two inks.
    let mut lines: Vec<(String, Rect, Rect)> = Vec::new();
    for cmd in og_ui::drawn_lines(ui.frame().items.iter().map(|i| &i.cmd)) {
        let DrawCmd::Text {
            text,
            bounds,
            clip,
            color,
            ..
        } = cmd
        else {
            continue;
        };
        if color[3] <= 0.0 || !text.chars().any(char::is_alphanumeric) {
            continue;
        }
        // Rotation/skew lives on DrawItem; the line's clip is already in
        // screen space. Compare screen bounds to screen bounds, as the probe
        // does, rather than accusing a rotated card's text at its old place.
        let item = ui
            .frame()
            .items
            .iter()
            .find(|item| {
                let inner = match &item.cmd {
                    DrawCmd::StyledText { text, .. } => &**text,
                    c => c,
                };
                std::ptr::eq(inner, cmd)
            })
            .expect("drawn line belongs to a frame item");
        let bounds = item.transform.map_rect(bounds);
        if let Some((_, _, had)) = lines.iter_mut().find(|(t, b, _)| t == text && *b == bounds) {
            let x = had.x.min(clip.x);
            let y = had.y.min(clip.y);
            *had = Rect::new(
                x,
                y,
                had.right().max(clip.right()) - x,
                had.bottom().max(clip.bottom()) - y,
            );
        } else {
            lines.push((text.clone(), bounds, *clip));
        }
    }
    for (text, bounds, clip) in lines {
        // The probe's owner rule, using the *unclipped* boxes so a completely
        // scrolled-out line still has its own ancestry (ids repeat in lists).
        let owner = active
            .iter()
            .copied()
            .filter(|&i| {
                let b = nodes[i].placed.screen;
                b.x <= bounds.right() + 0.5
                    && b.right() >= bounds.x - 0.5
                    && b.y <= bounds.bottom() + 0.5
                    && b.bottom() >= bounds.y - 0.5
            })
            .min_by_key(|&i| {
                let n = &nodes[i];
                let same = !n.probe_text.is_empty()
                    && n.probe_text.to_uppercase().contains(&text.to_uppercase());
                (
                    (!same) as u8,
                    (n.placed.screen.width * n.placed.screen.height) as u64,
                )
            });
        let Some(i) = owner else {
            failures.push(format!(
                "unowned text {text:?} at {bounds:?}: cannot verify its scroll path"
            ));
            continue;
        };
        content += 1;
        let inherited = nodes[i].placed.clip;
        // Only add the line's *own* clipping here. Inherited screen/scroll
        // edges are checked with their actual ranges below, not at offset 0.
        let mut local = bounds;
        if clip.x > inherited.x + 0.5 {
            local.width = local.right() - clip.x;
            local.x = clip.x;
        }
        if clip.y > inherited.y + 0.5 {
            local.height = local.bottom() - clip.y;
            local.y = clip.y;
        }
        if clip.right() < inherited.right() - 0.5 {
            local.width = clip.right() - local.x;
        }
        if clip.bottom() < inherited.bottom() - 0.5 {
            local.height = clip.bottom() - local.y;
        }
        let mut steps = vec![(local, [0.0; 2], [0.0; 2])];
        steps.extend(geometry_steps(ui, i));
        let lost = lost_edges(bounds, &steps);
        if !lost.is_empty() {
            failures.push(format!(
                "{} text {text:?}: {lost:?} unreachable, bounds {bounds:?}, clip {clip:?}",
                ui.tree.path_of(i)
            ));
        }
    }
    (content, failures)
}

#[test]
#[ignore = "requires GPU"]
fn engine_documents_have_reachable_phone_geometry() {
    phone_geometry_matrix(true, None, None);
}

/// The identical geometry cells without submitting GPU work. Useful for
/// diagnosing document CSS before the L4 runs the rendering matrix.
#[test]
#[ignore = "full document layout matrix; rendering is checked separately on L4"]
fn engine_documents_phone_geometry_layout() {
    // Diagnostic selection only; the GPU gate always checks the whole matrix.
    let document = std::env::var("OG_UI_GEOMETRY_DOCUMENT").ok();
    let pack = std::env::var("OG_UI_GEOMETRY_PACK").ok();
    phone_geometry_matrix(false, document.as_deref(), pack.as_deref());
}

fn geometry_positions(ui: &Ui) -> Vec<(String, Rect, Rect)> {
    geometry_nodes(ui)
        .into_iter()
        .map(|i| {
            let n = &ui.tree.nodes()[i];
            (ui.tree.path_of(i), n.placed.screen, n.placed.clip)
        })
        .collect()
}

/// Static fixtures need no half-second animation playback. Reduced motion
/// removes their intros; require identical layout/visibility on ticks 2 and 3
/// instead of assuming a settle count. A cell still moving is a named failure.
fn settle_geometry(
    ui: &mut Ui,
    messages: &Messages,
    mut viewport: Viewport,
) -> Result<bool, String> {
    viewport.reduced_motion = true;
    let mut previous = Vec::new();
    for tick in 0..3 {
        ui.begin(InputFrame::default(), viewport);
        // Inside InteractionPrompt's default 6 m visibility distance.
        ui.set_world_view(Some(og_ui::WorldView::look_at(
            og_core::glam::Vec3::new(0.0, 2.0, 4.0),
            og_core::glam::Vec3::ZERO,
            og_core::glam::Vec3::Y,
            50.0,
            viewport.width / viewport.height,
            0.1,
            100.0,
        )));
        ui.build_retained_documents(messages)?;
        ui.end();
        if tick == 1 {
            previous = geometry_positions(ui);
        }
    }
    Ok(previous == geometry_positions(ui))
}

fn phone_geometry_matrix(render: bool, document: Option<&str>, selected_pack: Option<&str>) {
    let (docs, messages) = engine_documents(&["src/ui/components", "src/ui/screens"]);
    let mut failures = Vec::new();
    let mut cells = 0;
    for (style, pack) in geometry_packs() {
        if selected_pack.is_some_and(|selected| selected != style) {
            continue;
        }
        let started = std::time::Instant::now();
        eprintln!(
            "document geometry: pack {style}, {} documents; reduced-motion ticks 2/3 must agree",
            docs.len()
        );
        let mut parser = Ui::default();
        if let Some(pack) = &pack {
            parser.set_style_pack(pack).unwrap();
        }
        let parsed: Vec<_> = docs
            .iter()
            .map(|d| {
                parser
                    .parse_document(&d.doc.source, &d.doc.text, Vec::new())
                    .unwrap_or_else(|e| panic!("{} style:{style}: parse: {e:?}", d.path))
            })
            .collect();
        for (index, doc) in docs.iter().enumerate() {
            if document.is_some_and(|selected| !doc.path.ends_with(selected)) {
                continue;
            }
            let before = failures.len();
            eprintln!(
                "document geometry: {style} document {}/{} {} ({} cells complete, {:.1}s)",
                index + 1,
                docs.len(),
                doc.path,
                cells,
                started.elapsed().as_secs_f32()
            );
            let mut fixtures = doc.doc.fixtures.clone();
            // Confirm currently declares only default; explicitly exercise its
            // long explanation too, without changing the shipped document.
            if doc.path == "src/ui/screens/confirm.svelte"
                && !fixtures.iter().any(|(n, _)| n == "long")
            {
                let mut long = doc.props.clone();
                long["body"] = serde_json::json!("Progress since the last save is lost. Return to the menu only when you are ready to leave this expedition. ".repeat(12));
                fixtures.push(("long".into(), long));
            }
            for (state, props) in fixtures {
                for (width, height, safe) in [
                    (
                        390,
                        844,
                        og_ui::Insets {
                            top: 47.0,
                            bottom: 34.0,
                            left: 0.0,
                            right: 0.0,
                        },
                    ),
                    (
                        844,
                        390,
                        og_ui::Insets {
                            top: 0.0,
                            bottom: 21.0,
                            left: 47.0,
                            right: 47.0,
                        },
                    ),
                ] {
                    cells += 1;
                    let cell = format!(
                        "{} fixture:{state} style:{style} {width}x{height} text:150%",
                        doc.path
                    );
                    let mut scene = RenderScene::new();
                    let mut ui = ui_with_font(add_test_font(&mut scene));
                    let mut theme = *ui.theme();
                    theme.text_scale = 1.5;
                    ui.set_theme(theme);
                    ui.set_icons(icons(None));
                    if let Some(pack) = &pack {
                        ui.set_style_pack(pack).unwrap();
                    }
                    for d in &parsed {
                        ui.define_document(d.clone());
                    }
                    ui.mount_with_values(
                        og_ui::reactive::SourceId::new("geometry", "fixture"),
                        &doc.doc.name,
                        &props,
                    )
                    .unwrap_or_else(|e| panic!("{cell}: mount: {e}"));
                    let mut viewport = Viewport::new(width as f32, height as f32);
                    viewport.safe_area = safe;
                    viewport.coarse_pointer = true;
                    if !settle_geometry(&mut ui, &messages, viewport)
                        .unwrap_or_else(|e| panic!("{cell}: build: {e}"))
                    {
                        failures.push(format!("{cell}: UNSETTLED: geometry/visibility differs between reduced-motion ticks 2 and 3"));
                    }
                    let (content, found) = document_geometry(&ui);
                    // The default establishes that this document's context is
                    // supported. Explicit alternative states may legitimately
                    // hide all content (empty collections, closed dialogs).
                    // They still build, render and check every present element.
                    if content == 0 && state == "default" {
                        failures.push(format!("{cell}: UNCHECKED: default fixture produced no measurable text, pictures or controls; world mass batches: {} (GPU mass instances have no element/line bounds in the native geometry interfaces)", ui.world_frame().batches.len()));
                    }
                    failures.extend(found.into_iter().map(|f| format!("{cell}: {f}")));
                    // Exercise the same renderer/font atlas path as the frame
                    // tests, but never compare pixels or record references.
                    if render {
                        let _rendered = render_ui(&ui, &mut scene, width, height);
                    }
                }
            }
            eprintln!(
                "document geometry: {style} {} done; {} findings",
                doc.path,
                failures.len() - before
            );
            for failure in &failures[before..] {
                eprintln!("{failure}");
            }
        }
    }
    eprintln!(
        "document geometry: {cells} cells, {} failures",
        failures.len()
    );
    assert!(
        cells > 0,
        "no document/pack matched the diagnostic selection"
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn document_geometry_scroll_controls() {
    let view = Rect::new(0.0, 0.0, 100.0, 100.0);
    let vertical = [(view, [0.0, 0.0], [0.0, 200.0])];
    assert!(lost_edges(Rect::new(0.0, 180.0, 80.0, 30.0), &vertical).is_empty());
    assert_eq!(
        lost_edges(Rect::new(0.0, -20.0, 80.0, 30.0), &vertical),
        ["top: 20.00px"]
    );
    assert_eq!(
        lost_edges(Rect::new(0.0, 180.0, 140.0, 30.0), &vertical),
        ["right: 40.00px"]
    );
    let sideways = [(view, [0.0; 2], [100.0, 0.0])];
    assert!(lost_edges(Rect::new(150.0, 0.0, 30.0, 20.0), &sideways).is_empty());
    assert_eq!(
        lost_edges(Rect::new(-20.0, 0.0, 30.0, 20.0), &sideways),
        ["left: 20.00px"]
    );
    // At a nonzero offset, the start remains reachable by scrolling back.
    let scrolled = [(view, [0.0, 80.0], [0.0, 200.0])];
    assert!(lost_edges(Rect::new(0.0, -80.0, 30.0, 20.0), &scrolled).is_empty());
    let nested = [(view, [0.0; 2], [0.0; 2]), (view, [0.0; 2], [100.0, 200.0])];
    assert_eq!(
        lost_edges(Rect::new(120.0, 0.0, 20.0, 20.0), &nested),
        ["left: 20.00px", "right: 40.00px"]
    );
}

/// Controls through the actual document/layout/scroll interfaces, not only the
/// interval arithmetic above. In particular CSS overflow-y must not excuse a
/// negative origin, or an overflow-x:hidden child's missing words.
#[test]
fn document_geometry_mounted_scroll_controls() {
    for (name, css, expected) in [
        ("reachable", "#Content { top: 180px; }", None),
        ("negative", "#Content { top: -20px; }", Some("top")),
        (
            "horizontal",
            "#Content { top: 180px; width: 140px; }",
            Some("right"),
        ),
    ] {
        let source = format!(
            r#"<meta name="component" content="GeometryControl">
<div id="Scroll"><button id="Content">Reach me</button><div id="Extent"></div></div>
<style>
#Scroll {{ position: relative; width: 100px; height: 100px; overflow-y: auto; overflow-x: hidden; }}
#Content {{ position: absolute; left: 0; width: 80px; height: 30px; padding: 0; }}
#Extent {{ width: 1px; height: 300px; }}
{css}
</style>"#
        );
        let d = crate::support::Doc::new(&source, &[]);
        let (_, failures) = document_geometry(&d.ui);
        let content: Vec<_> = failures
            .iter()
            .filter(|f| f.contains("/Content ("))
            .collect();
        match expected {
            None => assert!(content.is_empty(), "{name}: {content:?}"),
            Some(edge) => assert!(
                content.iter().any(|f| f.contains(edge)),
                "{name}: {failures:?}"
            ),
        }
    }
}

#[test]
fn document_geometry_reduced_motion_radial_settles() {
    let (docs, messages) = engine_documents(&["src/ui/components", "src/ui/screens"]);
    let doc = docs
        .iter()
        .find(|d| d.path.ends_with("/radial-menu.svelte"))
        .unwrap();
    let mut scene = RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    let mut theme = *ui.theme();
    theme.text_scale = 1.5;
    ui.set_theme(theme);
    for d in &docs {
        ui.define_document(d.doc.clone());
    }
    ui.mount_with_values(
        og_ui::reactive::SourceId::new("geometry", "settle"),
        &doc.doc.name,
        &doc.props,
    )
    .unwrap();
    let mut viewport = Viewport::new(390.0, 844.0);
    viewport.coarse_pointer = true;
    assert!(
        settle_geometry(&mut ui, &messages, viewport).unwrap(),
        "RadialMenu geometry must settle at reduced-motion ticks 2/3"
    );
    assert!(
        document_geometry(&ui).0 > 0,
        "settled RadialMenu must show content"
    );
}
