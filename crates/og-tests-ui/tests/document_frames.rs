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

// A wide world stage keeps metre-sized ground marks and the fixtures' nearby
// units in view in portrait, while the camera stays within prompt range.
// The GPU and CPU projections must use the identical field of view.
const GEOMETRY_WORLD_FOV: f32 = 80.0;

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

fn geometry_steps(ui: &Ui, node: usize, travel: &[(u64, f32)]) -> Vec<(Rect, [f32; 2], [f32; 2])> {
    let nodes = ui.tree.nodes();
    let mut steps = Vec::new();
    let movement = |key| {
        travel
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, dy)| (Rect::new(-1e9, -1e9, 2e9, 2e9), [0.0; 2], [0.0, *dy]))
    };
    if let Some(step) = movement(nodes[node].key) {
        steps.push(step);
    }
    let mut at = nodes[node].parent;
    while let Some(n) = nodes.get(at) {
        if let Some(step) = movement(n.key) {
            steps.push(step);
        }
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
    document_geometry_with_travel(ui, &[])
}

fn document_geometry_with_travel(ui: &Ui, travel: &[(u64, f32)]) -> (usize, Vec<String>) {
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
        // Noninteractive progress indicators are content, not decorative art.
        // Their semantic box covers the ring/bones even without a text label.
        let indicator = matches!(n.a11y.role, Some("progressbar" | "meter"));
        if !text && !picture && !indicator && !n.focusable && !n.el.interactive {
            continue;
        }
        let r = n.placed.screen;
        if !r.x.is_finite()
            || !r.y.is_finite()
            || !r.width.is_finite()
            || !r.height.is_finite()
            || r.width <= 0.0
            || r.height <= 0.0
        {
            failures.push(format!(
                "{} ({kind}) has no finite positive content box: {r:?}",
                ui.tree.path_of(i)
            ));
            continue;
        }
        content += 1;
        let lost = lost_edges(r, &geometry_steps(ui, i, travel));
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
        // Missing artwork draws a diagnostic path string, not player copy.
        // Do not mistake that debug overlay for a clipped caption. Real text
        // elements remain checked independently, including beside an art node.
        if ui.tree.kinds()[i].0 == "art" {
            continue;
        }
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
        steps.extend(geometry_steps(ui, i, travel));
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

/// Reconstruct the mass shader's semantic quads from the actual instance,
/// template timeline, camera and cooked glyph planes. A batch count alone
/// says nothing about visibility or safe-area coverage.
fn mass_geometry(ui: &Ui, scene: &RenderScene) -> (usize, Vec<String>) {
    use og_core::glam::{Vec2, Vec3};
    use og_draw::{GlyphKey, UiMassKind, UiMassSpace};
    let viewport = ui.viewport();
    let safe = Rect::new(
        viewport.safe_area.left,
        viewport.safe_area.top,
        viewport.width - viewport.safe_area.horizontal(),
        viewport.height - viewport.safe_area.vertical(),
    );
    let mut checked = 0;
    let mut failures = Vec::new();
    // A ground's composite rect is local to its panel. Its projected corners,
    // not that local layout box, determine what can be seen on the phone.
    let mut overlay = og_draw::Overlay::new(viewport.width, viewport.height, 1.0);
    ui.frame().emit(&og_ui::EmitDesc::default(), &mut overlay);
    let declared = ui.frame().items.iter().filter(|item| matches!(&item.cmd, DrawCmd::EndLayer { ground: Some(_), opacity, .. } if *opacity > 0.0)).count();
    let projected = overlay
        .ui
        .layers
        .iter()
        .filter(|layer| layer.ground.is_some())
        .count();
    if declared > projected {
        failures.push(format!(
            "UNCHECKED: {} world/plane composites have no projected context",
            declared - projected
        ));
    }
    for (key, layer) in overlay.ui.layers.iter().enumerate() {
        let Some(ground) = &layer.ground else {
            continue;
        };
        if ground.plane {
            continue;
        }
        let Some(corners) = ground.corners else {
            failures.push(format!("ground {key}: composite crosses camera plane"));
            continue;
        };
        let lo = corners
            .iter()
            .copied()
            .fold(Vec2::splat(f32::INFINITY), Vec2::min);
        let hi = corners
            .iter()
            .copied()
            .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
        let bounds = Rect::new(lo.x, lo.y, hi.x - lo.x, hi.y - lo.y);
        if !lo.is_finite() || !hi.is_finite() || bounds.width <= 0.0 || bounds.height <= 0.0 {
            failures.push(format!(
                "ground {key}: invalid projected composite {corners:?}"
            ));
            continue;
        }
        checked += 1;
        let lost = lost_edges(bounds, &[(safe, [0.0; 2], [0.0; 2])]);
        if !lost.is_empty() {
            failures.push(format!("ground {key}: {lost:?} unreachable: {bounds:?}"));
        }
    }
    ui.world_frame().emit(0, &mut overlay);
    for (b, batch) in overlay.ui.mass.iter().enumerate() {
        let tpl = &batch.template;
        for (j, instance) in batch.instances.iter().enumerate() {
            let label = format!("mass batch:{b} instance:{j}");
            let Some(age) = ui.world_frame().tick.checked_sub(instance.start) else {
                continue;
            };
            let t = if tpl.lifetime == 0 {
                0.0
            } else {
                age as f32 / tpl.lifetime as f32
            };
            if t > 1.0 {
                continue;
            }
            let (mut offset, mut scale, alpha) = tpl.sample(t);
            scale *= instance.scale;
            if alpha * instance.color[3] <= 0.002 || scale <= 0.0 {
                continue;
            }
            let travel = if tpl.drag { 1.0 - (1.0 - t).powi(2) } else { t };
            offset += Vec2::new(instance.aux[0], instance.aux[1])
                + Vec2::new(instance.velocity[0], instance.velocity[1]) * travel;
            let hash = (og_core::hash::pcg(instance.seed) & 0xffff) as f32 / 65535.0;
            offset.x += ((hash * 2.0 - 1.0) * tpl.drift + instance.aux[3]) * t;
            offset.y += tpl.gravity * t * t;
            let point = Vec3::from_array(instance.position);
            let (mut at, distance_scale) = if batch.space == UiMassSpace::World {
                let Some(view) = ui.world_view() else {
                    failures.push(format!("{label}: UNCHECKED: no world camera"));
                    continue;
                };
                let clip = view.view_proj * point.extend(1.0);
                let Some((x, y, depth)) = view.project(point, viewport.width, viewport.height)
                else {
                    failures.push(format!("{label}: world point behind camera: {point:?}"));
                    continue;
                };
                if !(0.0..=1.0).contains(&(clip.z / clip.w)) {
                    failures.push(format!(
                        "{label}: world point outside camera depth: {point:?}"
                    ));
                    continue;
                }
                let ds = if tpl.distance.x > 0.0 {
                    (tpl.distance.x / depth.max(1e-4)).clamp(tpl.distance.y, tpl.distance.z)
                } else {
                    1.0
                };
                (Vec2::new(x, y), ds)
            } else {
                (point.truncate(), 1.0)
            };
            at += offset * distance_scale;
            scale *= distance_scale;
            let mut boxes = Vec::new();
            match &tpl.kind {
                UiMassKind::Bar { .. } => {
                    let size = tpl.size * scale;
                    boxes.push(Rect::new(
                        at.x - tpl.pivot.x * size.x,
                        at.y - tpl.pivot.y * size.y,
                        size.x,
                        size.y,
                    ));
                }
                UiMassKind::Number { .. } | UiMassKind::Text { .. } => {
                    let text = match &tpl.kind {
                        UiMassKind::Number {
                            decimals,
                            plus,
                            compact,
                        } => og_draw::format_number(instance.value, *decimals, *plus, *compact),
                        UiMassKind::Text { strings } => strings
                            .get(instance.value.max(0.0) as usize)
                            .map(|s| s.chars().take(og_draw::UI_MASS_QUADS as usize).collect())
                            .unwrap_or_default(),
                        _ => unreachable!(),
                    };
                    let Some(font) = tpl.font.and_then(|font| scene.font(font)) else {
                        failures.push(format!("{label}: UNCHECKED: mass text has no cooked font"));
                        continue;
                    };
                    let kerns = matches!(tpl.kind, UiMassKind::Text { .. });
                    let mut pen = 0.0;
                    let mut previous = None;
                    let mut planes = Vec::new();
                    for ch in text.chars() {
                        if kerns && let Some(prev) = previous {
                            pen += font
                                .kerning
                                .iter()
                                .find(|k| k.left == prev && k.right == ch)
                                .map_or(0.0, |k| k.adjust);
                        }
                        let Some(glyph) = font.glyphs.iter().find(|g| g.key == GlyphKey::Char(ch))
                        else {
                            failures.push(format!("{label}: UNCHECKED: missing glyph {ch:?}"));
                            continue;
                        };
                        let [l, bottom, r, top] = glyph.plane_bounds;
                        if r > l {
                            planes.push((pen + l, bottom, pen + r, top));
                        }
                        pen += glyph.advance;
                        previous = Some(ch);
                    }
                    let em = tpl.size.x * scale;
                    let left = at.x - tpl.pivot.x * pen * em;
                    let baseline = at.y - tpl.pivot.y * em + 0.8 * em;
                    for (l, bottom, r, top) in planes {
                        boxes.push(Rect::new(
                            left + l * em,
                            baseline - top * em,
                            (r - l) * em,
                            (top - bottom) * em,
                        ));
                    }
                }
                _ => {
                    failures.push(format!(
                        "{label}: UNCHECKED: unsupported mass kind {:?}",
                        tpl.kind
                    ));
                    continue;
                }
            }
            let mut steps = vec![(safe, [0.0; 2], [0.0; 2])];
            if let Some(index) = batch.clip {
                if let Some(&[x, y, w, h]) = overlay.clips.get(index as usize) {
                    steps.insert(0, (Rect::new(x, y, w, h), [0.0; 2], [0.0; 2]));
                } else {
                    failures.push(format!(
                        "{label}: UNCHECKED: missing emitted mass clip {index}"
                    ));
                }
                if overlay.ui.clips.iter().any(|clip| clip.rect == index) {
                    failures.push(format!("{label}: UNCHECKED: nonrectangular mass clip"));
                }
            }
            if boxes.is_empty() {
                failures.push(format!("{label}: UNCHECKED: no visible mass quads"));
            }
            for bounds in boxes {
                if ![bounds.x, bounds.y, bounds.width, bounds.height]
                    .into_iter()
                    .all(f32::is_finite)
                    || bounds.width <= 0.0
                    || bounds.height <= 0.0
                {
                    failures.push(format!("{label}: invalid mass quad {bounds:?}"));
                    continue;
                }
                checked += 1;
                let lost = lost_edges(bounds, &steps);
                if !lost.is_empty() {
                    failures.push(format!("{label}: {lost:?} unreachable: {bounds:?}"));
                }
            }
        }
    }
    (checked, failures)
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

fn geometry_positions(ui: &Ui) -> Vec<(String, Rect, Rect, String)> {
    geometry_nodes(ui)
        .into_iter()
        .map(|i| {
            let n = &ui.tree.nodes()[i];
            (
                ui.tree.path_of(i),
                n.placed.screen,
                n.placed.clip,
                n.probe_text.clone(),
            )
        })
        .collect()
}

/// Reduced motion removes CSS intros, but finite value animations (counter)
/// still play. If ticks 2/3 disagree, sample at the end of a two-second window,
/// not the first coincidental plateau (digits can have identical widths).
/// Continuing motion remains a failure; no component names are exempted.
fn settle_geometry(
    ui: &mut Ui,
    messages: &Messages,
    mut viewport: Viewport,
) -> Result<bool, String> {
    viewport.reduced_motion = true;
    let mut previous = Vec::new();
    let last = 2 * ui.ticks_per_second() + 2;
    for tick in 0..=last {
        ui.begin(InputFrame::default(), viewport);
        // Inside InteractionPrompt's default 6 m visibility distance.
        ui.set_world_view(Some(og_ui::WorldView::look_at(
            og_core::glam::Vec3::new(0.0, 3.0, 5.0),
            og_core::glam::Vec3::new(0.0, 1.0, 0.0),
            og_core::glam::Vec3::Y,
            GEOMETRY_WORLD_FOV,
            viewport.width / viewport.height,
            0.1,
            100.0,
        )));
        ui.build_retained_documents(messages)?;
        let poses: Vec<_> = ui
            .world_entities()
            .into_iter()
            .map(|entity| (entity, og_core::math::Pose::default()))
            .collect();
        ui.set_world_poses(&poses);
        ui.end();
        let positions = geometry_positions(ui);
        if tick == 2 && previous == positions {
            return Ok(true);
        }
        if tick == last {
            if previous != positions {
                for (a, b) in previous
                    .iter()
                    .zip(&positions)
                    .filter(|(a, b)| a != b)
                    .take(8)
                {
                    eprintln!("unsettled geometry tick {tick}: {a:?} -> {b:?}");
                }
            }
            return Ok(previous == positions);
        }
        previous = positions;
    }
    unreachable!()
}

/// A game-driven roll is not a scroller. Recognize the linear CSS binding,
/// then prove its travel by changing the real mount value at two distances.
/// No document/id/prop names are special; unsupported or non-linear state
/// changes remain unchecked rather than granting hypothetical reachability.
fn translation_travel(
    ui: &mut Ui,
    source: &str,
    props: &serde_json::Value,
    messages: &Messages,
    viewport: Viewport,
    mut inspect: impl FnMut(&Ui),
) -> Result<Vec<(u64, f32)>, String> {
    let compact: String = source.split_whitespace().collect();
    let Some(after) = compact.split("translate:0bind(\"-").nth(1) else {
        return Ok(Vec::new());
    };
    let Some((field, _)) = after.split_once("\")") else {
        return Err("incomplete translation binding".into());
    };
    if !field.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("unsupported translation expression".into());
    }
    let Some(value) = props.get(field).and_then(serde_json::Value::as_f64) else {
        return Err("translation binding has no numeric fixture value".into());
    };
    let base: std::collections::BTreeMap<_, _> = geometry_nodes(ui)
        .into_iter()
        .map(|i| {
            let n = &ui.tree.nodes()[i];
            (n.key, (n.placed.screen, n.probe_text.clone()))
        })
        .collect();
    let distance = base
        .values()
        .map(|(r, _)| r.bottom())
        .fold(viewport.height, f32::max)
        + viewport.height;
    let mount = og_ui::reactive::SourceId::new("geometry", "fixture");
    let result = (|| {
        let mut roots = Vec::new();
        for fraction in [1.0, 0.5] {
            let mut values = props.clone();
            values[field] = serde_json::json!(value + f64::from(distance * fraction));
            ui.set_mount_values(&mount, &values)?;
            if !settle_geometry(ui, messages, viewport)? {
                return Err("translated fixture does not settle".into());
            }
            inspect(ui);
            if geometry_nodes(ui).len() != base.len() {
                return Err("translation changes visibility/topology".into());
            }
            let mut observed_roots = 0;
            for i in geometry_nodes(ui) {
                let n = &ui.tree.nodes()[i];
                let Some((before, text)) = base.get(&n.key) else {
                    return Err("translation changes element keys".into());
                };
                let now = n.placed.screen;
                if (now.width - before.width).abs() > 0.5
                    || (now.height - before.height).abs() > 0.5
                    || (now.x - before.x).abs() > 0.5
                    || *text != n.probe_text
                {
                    return Err("translation changes more than vertical position".into());
                }
                let dy = before.y - now.y;
                let parent_dy = ui
                    .tree
                    .nodes()
                    .get(n.parent)
                    .and_then(|parent| {
                        base.get(&parent.key)
                            .map(|(r, _)| r.y - parent.placed.screen.y)
                    })
                    .unwrap_or(0.0);
                if dy > 1.0 && parent_dy.abs() <= 0.5 {
                    observed_roots += 1;
                    if n.scrolls() || n.decor.as_ref().is_some_and(|d| d.clip) {
                        return Err("moving clip needs its own state model".into());
                    }
                    if fraction == 1.0 {
                        roots.push((n.key, dy));
                    } else if !roots
                        .iter()
                        .any(|(key, full)| *key == n.key && (dy - full * 0.5).abs() <= 1.0)
                    {
                        return Err("translation is not linear at midpoint".into());
                    }
                } else if dy < -0.5 {
                    return Err("translation moves content in the wrong direction".into());
                } else if parent_dy > 1.0 && (dy - parent_dy).abs() > 0.5 {
                    return Err("translation is not a rigid subtree movement".into());
                }
            }
            if fraction == 0.5 && observed_roots != roots.len() {
                return Err("translation roots differ at midpoint".into());
            }
        }
        if roots.is_empty() {
            return Err("translation binding produced no measured travel".into());
        }
        Ok(roots)
    })();
    ui.set_mount_values(&mount, props)?;
    if !settle_geometry(ui, messages, viewport)? {
        return Err("restored translation fixture does not settle".into());
    }
    if geometry_nodes(ui).into_iter().any(|i| {
        let n = &ui.tree.nodes()[i];
        base.get(&n.key)
            .is_none_or(|(r, text)| *r != n.placed.screen || *text != n.probe_text)
    }) {
        return Err("translation fixture did not restore its original geometry".into());
    }
    result
}

fn render_geometry_frame(ui: &Ui, scene: &mut RenderScene, width: u32, height: u32) {
    let mut overlay = og_draw::Overlay::new(width as f32, height as f32, 1.0);
    ui.frame().emit(&og_ui::EmitDesc::default(), &mut overlay);
    ui.world_frame().emit(0, &mut overlay);
    scene.ui.overlay = Some(overlay);
    on_backdrop(ui, scene);
    let view = ui.world_view().expect("geometry camera");
    scene.frame.camera = og_render::Camera {
        position: view.eye,
        target: view.eye + view.forward,
        up: og_core::glam::Vec3::Y,
        projection: og_render::Projection::Perspective {
            fov_y_degrees: GEOMETRY_WORLD_FOV,
        },
        near: 0.1,
        far: 100.0,
        ..Default::default()
    };
    let _rendered = render_frame(scene, &sized(width, height));
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
            "document geometry: pack {style}, {} documents; reduced-motion ticks 2/3 or final two ticks after 2s must agree",
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
                        failures.push(format!("{cell}: UNSETTLED: geometry/visibility/text still differs after 2s reduced-motion playback"));
                    }
                    let travel = translation_travel(
                        &mut ui,
                        &doc.doc.text,
                        &props,
                        &messages,
                        viewport,
                        |ui| {
                            if render {
                                render_geometry_frame(ui, &mut scene, width, height);
                            }
                        },
                    )
                    .unwrap_or_else(|error| {
                        failures.push(format!("{cell}: UNCHECKED: translated context: {error}"));
                        Vec::new()
                    });
                    let (content, mut found) = document_geometry_with_travel(&ui, &travel);
                    let (mass_content, mass_found) = mass_geometry(&ui, &scene);
                    found.extend(mass_found);
                    // The default establishes that this document's context is
                    // supported. Explicit alternative states may legitimately
                    // hide all content (empty collections, closed dialogs).
                    // They still build, render and check every present element.
                    if content + mass_content == 0 && state == "default" {
                        failures.push(format!("{cell}: UNCHECKED: default fixture produced no measurable text, semantic boxes or mass quads"));
                    }
                    failures.extend(found.into_iter().map(|f| format!("{cell}: {f}")));
                    // Exercise the same renderer/font atlas path as the frame
                    // tests, but never compare pixels or record references.
                    if render {
                        render_geometry_frame(&ui, &mut scene, width, height);
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

#[test]
fn document_geometry_semantic_indicator_controls() {
    for (left, expected) in [(0, false), (-20, true), (1200, true)] {
        let source = format!(
            r#"<meta name="component" content="IndicatorControl">
<div role="progressbar" id="Indicator"></div>
<style>#Indicator {{ position: absolute; left: {left}px; top: 0; width: 30px; height: 30px; }}</style>"#
        );
        let d = crate::support::Doc::new(&source, &[]);
        let (checked, failures) = document_geometry(&d.ui);
        assert_eq!(checked, 1, "a nontext progressbar is measured");
        assert_eq!(!failures.is_empty(), expected, "{left}: {failures:?}");
    }
}

#[test]
fn document_geometry_results_counter_settles() {
    let (docs, messages) = engine_documents(&["src/ui/components", "src/ui/screens"]);
    let doc = docs
        .iter()
        .find(|d| d.path == "src/ui/screens/results.svelte")
        .unwrap();
    let mut scene = RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    for d in &docs {
        ui.define_document(d.doc.clone());
    }
    ui.mount_with_values(
        og_ui::reactive::SourceId::new("geometry", "results"),
        &doc.doc.name,
        &doc.props,
    )
    .unwrap();
    let mut viewport = Viewport::new(390.0, 844.0);
    viewport.reduced_motion = true;
    let mut previous = Vec::new();
    for tick in 0..=60 {
        ui.begin(InputFrame::default(), viewport);
        ui.build_retained_documents(&messages).unwrap();
        ui.end();
        if tick == 59 {
            previous = geometry_positions(&ui);
        }
    }
    assert_eq!(
        previous,
        geometry_positions(&ui),
        "finite counter must stop"
    );
    assert!(
        ui.tree.nodes().iter().any(|n| n.probe_text == "48,210"),
        "check final stat, not merely a same-width digit plateau"
    );
    assert!(settle_geometry(&mut ui, &messages, viewport).unwrap());
}

#[test]
fn document_geometry_world_mass_controls() {
    use og_core::glam::{Vec2, Vec3};
    use og_draw::{UiMassInstance, UiMassKind, UiMassSpace, UiMassTemplate};
    for (point, valid) in [
        (Vec3::ZERO, true),
        (Vec3::new(100.0, 0.0, 0.0), false),
        (Vec3::new(0.0, 0.0, 10.0), false),
    ] {
        let mut scene = RenderScene::new();
        let mut ui = ui_with_font(add_test_font(&mut scene));
        ui.begin(InputFrame::default(), Viewport::new(390.0, 844.0));
        ui.set_world_view(Some(og_ui::WorldView::look_at(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::Y,
            50.0,
            390.0 / 844.0,
            0.1,
            100.0,
        )));
        let handle = ui.define_mass(
            "control",
            UiMassTemplate {
                kind: UiMassKind::Bar {
                    back: [1.0; 4],
                    ghost: [1.0; 4],
                    radius: 0.0,
                },
                size: Vec2::new(40.0, 8.0),
                ..Default::default()
            },
            UiMassSpace::World,
        );
        ui.push_mass(
            handle,
            UiMassInstance {
                position: point.to_array(),
                color: [1.0; 4],
                scale: 1.0,
                ..Default::default()
            },
        );
        ui.end();
        let (checked, failures) = mass_geometry(&ui, &scene);
        assert_eq!(failures.is_empty(), valid, "{point:?}: {failures:?}");
        if valid {
            assert_eq!(checked, 1);
        }
    }
}

#[test]
fn document_geometry_world_number_and_missing_font_controls() {
    use og_draw::{UiMassInstance, UiMassKind, UiMassSpace, UiMassTemplate};
    let mut scene = RenderScene::new();
    let mut ui = ui_with_font(add_test_font(&mut scene));
    ui.begin(InputFrame::default(), Viewport::new(390.0, 844.0));
    let handle = ui.define_mass(
        "number",
        UiMassTemplate {
            kind: UiMassKind::Number {
                decimals: 0,
                plus: false,
                compact: false,
            },
            ..Default::default()
        },
        UiMassSpace::Screen,
    );
    ui.push_mass(
        handle,
        UiMassInstance {
            position: [195.0, 422.0, 0.0],
            value: 1240.0,
            color: [1.0; 4],
            scale: 1.0,
            ..Default::default()
        },
    );
    ui.end();
    let (checked, failures) = mass_geometry(&ui, &scene);
    assert_eq!(checked, 4, "measure every digit's cooked glyph plane");
    assert!(failures.is_empty(), "{failures:?}");
    let (checked, failures) = mass_geometry(&ui, &RenderScene::new());
    assert_eq!(checked, 0, "missing font must not be declared measured");
    assert!(
        failures.iter().any(|f| f.contains("no cooked font")),
        "{failures:?}"
    );
}

#[test]
fn document_geometry_ground_controls() {
    for (at, valid) in [("0 0 0", true), ("100 0 0", false)] {
        let source = format!(
            r##"<meta name="component" content="GroundControl">
<div id="Root"><ground id="Ground" at="{at}" size="1 1"><svg id="Mark" viewBox="0 0 100 100"><circle cx="50" cy="50" r="40" fill="#ffffff"></circle></svg></ground></div>
<style>#Ground, #Mark {{ width: 100px; height: 100px; }}</style>"##
        );
        let mut d = crate::support::Doc::new(&source, &[]);
        assert!(
            settle_geometry(
                &mut d.ui,
                &Messages(Default::default()),
                Viewport::new(390.0, 844.0)
            )
            .unwrap()
        );
        let (checked, failures) = mass_geometry(&d.ui, &RenderScene::new());
        if valid {
            assert!(checked > 0, "ground must really emit a projected composite");
        }
        assert_eq!(failures.is_empty(), valid, "{at}: {failures:?}");
    }
}

#[test]
fn document_geometry_continuing_counter_is_not_settled() {
    let mut d = crate::support::Doc::new(
        r#"<meta name="component" content="LongCounterControl"><og-counter value="100000" from="0" duration="10000"></og-counter>"#,
        &[],
    );
    assert!(
        !settle_geometry(
            &mut d.ui,
            &Messages(Default::default()),
            Viewport::new(390.0, 844.0)
        )
        .unwrap(),
        "continuing value/layout motion must remain a defect, not silently sample its first same-width plateau"
    );
}

#[test]
fn document_geometry_bound_translation_controls() {
    for (left, reachable) in [(0, true), (120, false)] {
        let source = format!(
            r#"<meta name="component" content="TranslationControl">
<meta name="prop:advance" content="number">
<div id="Window"><div id="Moving"><button id="Content">Reach me</button></div></div>
<style>
#Window {{ position: relative; width: 100px; height: 100px; overflow: hidden; }}
#Moving {{ position: relative; width: 100px; height: 500px; translate: 0 bind("-advance"); }}
#Content {{ position: absolute; left: {left}px; top: 300px; width: 80px; height: 30px; padding: 0; }}
</style>"#
        );
        let mut scene = RenderScene::new();
        let mut ui = ui_with_font(add_test_font(&mut scene));
        let doc = ui
            .parse_document("translation.svelte", &source, Vec::new())
            .unwrap();
        let name = doc.name.clone();
        ui.define_document(doc);
        let props = serde_json::json!({"advance": 0});
        ui.mount_with_values(
            og_ui::reactive::SourceId::new("geometry", "fixture"),
            &name,
            &props,
        )
        .unwrap();
        let viewport = Viewport::new(390.0, 844.0);
        let messages = Messages(Default::default());
        assert!(settle_geometry(&mut ui, &messages, viewport).unwrap());
        assert!(
            document_geometry(&ui)
                .1
                .iter()
                .any(|f| f.contains("/Content (") && f.contains("bottom"))
        );
        let mut sampled = 0;
        let travel = translation_travel(&mut ui, &source, &props, &messages, viewport, |_| {
            sampled += 1
        })
        .unwrap();
        assert_eq!(
            sampled, 2,
            "sample endpoint and midpoint through real mount values"
        );
        assert_eq!(travel.len(), 1);
        let (_, failures) = document_geometry_with_travel(&ui, &travel);
        assert_eq!(
            !failures.iter().any(|f| f.contains("/Content (")),
            reachable,
            "{left}: {failures:?}"
        );
    }
}

#[test]
fn document_geometry_art_diagnostic_is_not_player_copy() {
    let mut scene = RenderScene::new();
    let source = r#"<meta name="component" content="ArtControl">
<div id="Window"><art id="VeryLongDiagnosticArtworkIdentifier" kind="logo"></art><p id="Copy">Actual player copy must still be checked</p></div>
<style>
#Window { width: 100px; height: 100px; overflow: hidden; }
art { width: 30px; height: 30px; }
#Copy { width: 500px; height: 30px; white-space: nowrap; }
</style>"#;
    let d = crate::support::Doc::with_font(source, &[], add_test_font(&mut scene));
    assert!(og_ui::drawn_lines(d.ui.frame().items.iter().map(|i| &i.cmd)).into_iter().any(|cmd|
        matches!(cmd, DrawCmd::Text { text, .. } if text.contains("VeryLongDiagnosticArtworkIdentifier"))),
        "the control must really draw the synthetic missing-art caption");
    let (_, failures) = document_geometry(&d.ui);
    assert!(
        !failures
            .iter()
            .any(|f| f.contains("VeryLongDiagnosticArtworkIdentifier")),
        "{failures:?}"
    );
    assert!(
        failures
            .iter()
            .any(|f| f.contains("/Copy") && f.contains("right")),
        "actual clipped copy must not be excused: {failures:?}"
    );
}
