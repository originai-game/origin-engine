use super::{DrawCmd, DrawItem, Ground, LayerRole, linear, mapped_shape, mul2};
use crate::element::Affine;
use crate::layout::Rect;
use crate::world::WorldCmd;
use og_core::glam::Vec2;
use og_draw::{Overlay, Text, UiGeometry, UiMaterial, UiMode, UiPaint, UiShape};

/// Where a frame's shapes and text go.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EmitDesc {
    /// The overlay layer of painter level 0; level n draws on `layer + n`.
    /// Put the UI above other overlay sprites with a high base. A UI layer
    /// ([`DrawCmd::BeginLayer`]) takes [`LAYER_SPAN`] overlay layers of its
    /// own from `layer` + [`LAYER_BASE`].
    pub layer: i32,
    /// The UI clock in seconds (`Ui::seconds`): patterns scroll and layer
    /// materials move by it.
    pub time: f32,
}

/// Past the main levels: where the UI's offscreen layers take their
/// overlay layers.
pub const LAYER_BASE: i32 = 1 << 19;
/// The overlay layers of one offscreen layer (its painter levels).
pub const LAYER_SPAN: i32 = 4096;
/// The most offscreen layers one frame draws; past them a layer's subtree
/// draws in place.
pub const MAX_LAYERS: u32 = 120;

/// A rectangle as og-render's `[x0, y0, width, height]` key.
fn key(r: &Rect) -> [u32; 4] {
    [
        r.x.to_bits(),
        r.y.to_bits(),
        r.width.to_bits(),
        r.height.to_bits(),
    ]
}

/// A cut to a shape in force: its geometry, its own record in the UI's
/// clip list and the record of the cut under it.
struct Cut {
    id: u32,
    record: u32,
    below: Option<u32>,
    geometry: UiGeometry,
    center: Vec2,
    half: Vec2,
    transform: [f32; 4],
}

/// A clip rectangle's key: its bits, the cut and the depth test in force.
type ClipKey = ([u32; 4], Option<u32>, Option<u32>);

struct Clips {
    known: std::collections::HashMap<ClipKey, u32>,
    cuts: Vec<Cut>,
    next_cut: u32,
    /// Depth tests in force (`WorldCmd::Occlude`): id, point, depth, alpha.
    occluders: Vec<(u32, Vec2, f32, f32)>,
}

impl Clips {
    /// The overlay clip index of a rectangle under the cuts in force,
    /// adding it (and its cut's shape) the first time.
    fn index(&mut self, overlay: &mut Overlay, clip: &Rect) -> u32 {
        let top = self.cuts.last();
        let occ = self.occluders.last().copied();
        let k = (key(clip), top.map(|c| c.id), occ.map(|o| o.0));
        if let Some(i) = self.known.get(&k) {
            return *i;
        }
        let i = overlay.add_clip([clip.x, clip.y, clip.width, clip.height]);
        if let Some((_, at, depth, alpha)) = occ {
            overlay.ui.occluders.push(og_draw::UiOccluder {
                rect: i,
                at,
                depth,
                alpha,
                bias: 0.25,
                soft: 0.5,
            });
        }
        if let Some(c) = top {
            overlay.ui.clips.push(og_draw::UiClip {
                rect: i,
                geometry: c.geometry.clone(),
                center: c.center,
                half: c.half,
                transform: c.transform,
                parent: c.below,
            });
        }
        self.known.insert(k, i);
        i
    }

    /// Starts a cut, its shape under `m`: its own record (on a clip
    /// rectangle of its own) is the parent of the cuts inside it.
    fn push(
        &mut self,
        overlay: &mut Overlay,
        geometry: UiGeometry,
        center: Vec2,
        half: Vec2,
        m: &Affine,
    ) {
        let below = self.cuts.last().map(|c| c.record);
        let rect = overlay.add_clip([-1e9, -1e9, 2e9, 2e9]);
        let (cx, cy) = m.apply(center.x, center.y);
        let center = Vec2::new(cx, cy);
        let transform = linear(m);
        overlay.ui.clips.push(og_draw::UiClip {
            rect,
            geometry: geometry.clone(),
            center,
            half,
            transform,
            parent: below,
        });
        self.next_cut += 1;
        self.cuts.push(Cut {
            id: self.next_cut,
            record: overlay.ui.clips.len() as u32 - 1,
            below,
            geometry,
            center,
            half,
            transform,
        });
    }
}

/// A space of levels: its base overlay layer and its UI layer.
struct Space {
    base: i32,
    layer: Option<u32>,
    role: LayerRole,
    /// Draws nothing (an element image with no layer to draw into).
    hidden: bool,
}

impl Space {
    fn new(base: i32, layer: Option<u32>) -> Space {
        Space {
            base,
            layer,
            role: LayerRole::Draw,
            hidden: false,
        }
    }
}

/// A UI shape, laid on `layer` and clip `clip`.
fn placed(mut shape: UiShape, layer: i32, clip: u32) -> UiShape {
    shape.layer = layer;
    shape.clip = Some(clip);
    shape
}

fn rect_shape(rect: Rect, radius: f32, color: [f32; 4]) -> UiShape {
    UiShape::rect([rect.x, rect.y, rect.width, rect.height], radius, color)
}

/// `item` moved by `d` CSS px: its map translated, its clip and depth
/// test point with it.
/// A frame's commands drawn `unit` times larger (the og-ui unit, docs/ui.md
/// §15.2): laid out in units, emitted in CSS px. A command that keeps
/// rectangles upright takes the scale into its values ([`crate::ui::bake`]);
/// a turned, skewed or UI-pass command takes it over its map, its
/// screen-space clip, layer rectangle and points scaled with it. Blur
/// radii, a mass element's size and where screen-space ones stand grow too.
pub(crate) fn zoom_items(mut items: Vec<DrawItem>, unit: f32) -> Vec<DrawItem> {
    if unit == 1.0 {
        return items;
    }
    let s = Affine::scale(unit, unit);
    for item in &mut items {
        let cmd = std::mem::replace(&mut item.cmd, DrawCmd::PopClip);
        let (mut cmd, rest) = if item.transform.is_identity() {
            crate::ui::bake(cmd, s)
        } else {
            (cmd, s.mul(&item.transform))
        };
        // Not baked: its clip is the screen's, and the screen is scaled.
        if !rest.is_identity()
            && let Some(c) = cmd.clip_mut()
        {
            *c = s.map_rect(c);
        }
        match &mut cmd {
            DrawCmd::Glass { blur, .. } => *blur *= unit,
            DrawCmd::Shape { shape, .. } => {
                if let UiMode::Backdrop { blur, .. } = &mut shape.mode {
                    *blur *= unit;
                }
            }
            DrawCmd::BeginLayer {
                rect,
                expand,
                filter,
                ..
            } => {
                *rect = s.map_rect(rect);
                *expand *= unit;
                if let Some(f) = filter {
                    f.0 = s.map_rect(&f.0);
                    crate::svg::filter::map(&mut f.1, &s);
                }
            }
            DrawCmd::World(w) => match &mut **w {
                WorldCmd::Occlude { at, .. } => *at = (at.0 * unit, at.1 * unit),
                WorldCmd::Mass { batch, bounds, .. } => {
                    *bounds = s.map_rect(bounds);
                    zoom_mass(batch, unit);
                }
                WorldCmd::BeginPanel { rect, .. } => *rect = s.map_rect(rect),
                _ => {}
            },
            _ => {}
        }
        item.cmd = cmd;
        item.transform = rest;
    }
    items
}

/// A mass batch drawn `unit` times larger ([`zoom_items`]): each
/// instance's size, and for screen-space ones where it stands and how far
/// it travels.
pub(crate) fn zoom_mass(b: &mut og_draw::UiMassBatch, unit: f32) {
    let screen = b.space == og_draw::UiMassSpace::Screen;
    for i in &mut b.instances {
        i.scale *= unit;
        if screen {
            i.position[0] *= unit;
            i.position[1] *= unit;
            i.aux[0] *= unit;
            i.aux[1] *= unit;
            i.aux[3] *= unit;
            i.velocity[0] *= unit;
            i.velocity[1] *= unit;
        }
    }
}

fn shifted(item: &DrawItem, d: Vec2) -> DrawItem {
    let mut out = item.clone();
    out.transform = Affine::translate(d.x, d.y).mul(&item.transform);
    if let Some(c) = out.cmd.clip_mut() {
        c.x += d.x;
        c.y += d.y;
    }
    if let DrawCmd::World(w) = &mut out.cmd
        && let WorldCmd::Occlude { at, .. } = &mut **w
    {
        *at = (at.0 + d.x, at.1 + d.y);
    }
    out
}

/// How far under a frame's level 0 its grounds' composites draw: under the
/// world frame's mass elements ([`crate::WorldFrame::emit`], one under).
const GROUND_BELOW: i32 = 2;

/// The levels of the planes of each 3D rendering context, farthest
/// first (CSS Transforms 2 §6.2): a context's composites are consecutive
/// in their space and each is a level of its own there (`collect`), so
/// handing their levels out by depth draws them back to front.
fn depth_order(
    items: &[DrawItem],
    grounds: &[crate::GroundQuad],
) -> std::collections::HashMap<u64, i32> {
    // (space, context) → each plane's (level, depth, key).
    type Planes = Vec<(i32, f32, u64)>;
    let mut groups: std::collections::BTreeMap<(u32, u64), Planes> = Default::default();
    let (mut spaces, mut next) = (vec![0u32], 0u32);
    for item in items {
        match &item.cmd {
            DrawCmd::BeginLayer { .. } => {
                next += 1;
                spaces.push(next);
            }
            DrawCmd::EndLayer { ground, .. } => {
                spaces.pop();
                if let Some(Ground::Plane(key)) = ground
                    && let Some(g) = grounds.iter().find(|g| g.key == *key)
                    && g.context != 0
                {
                    let space = spaces.last().copied().unwrap_or(0);
                    groups
                        .entry((space, g.context))
                        .or_default()
                        .push((item.level, g.depth, *key));
                }
            }
            _ => {}
        }
    }
    let mut out = std::collections::HashMap::new();
    for (_, mut planes) in groups {
        let mut levels: Vec<i32> = planes.iter().map(|p| p.0).collect();
        levels.sort_unstable();
        // Stable: equal depths keep their order in the tree.
        planes.sort_by(|a, b| a.1.total_cmp(&b.1));
        for (level, p) in levels.into_iter().zip(planes) {
            out.insert(p.2, level);
        }
    }
    out
}

pub(crate) fn emit(
    items: &[DrawItem],
    desc: &EmitDesc,
    overlay: &mut Overlay,
    shift: &dyn Fn(u64) -> Option<Vec2>,
    grounds: &[crate::GroundQuad],
) {
    // The anchored elements open (`WorldCmd::Anchor`): each one's shift.
    let mut anchors: Vec<Vec2> = Vec::new();
    let mut clips = Clips {
        known: std::collections::HashMap::new(),
        cuts: Vec::new(),
        next_cut: 0,
        occluders: Vec::new(),
    };
    overlay.ui.time = desc.time;
    let sorted = depth_order(items, grounds);
    let mut spaces = vec![Space::new(desc.layer, None)];
    for item in items {
        if let DrawCmd::World(w) = &item.cmd {
            match **w {
                WorldCmd::Anchor { key } => {
                    let outer = anchors.last().copied().unwrap_or(Vec2::ZERO);
                    anchors.push(outer + shift(key).unwrap_or(Vec2::ZERO));
                    continue;
                }
                WorldCmd::EndAnchor => {
                    anchors.pop();
                    continue;
                }
                _ => {}
            }
        }
        let moved;
        let item = match anchors.last() {
            Some(&d) if d != Vec2::ZERO => {
                moved = shifted(item, d);
                &moved
            }
            _ => item,
        };
        let space = spaces.last().expect("the frame's space");
        // An element image with no layer draws nothing; its spaces still
        // open and close.
        let hidden = space.hidden;
        if hidden
            && !matches!(
                item.cmd,
                DrawCmd::BeginLayer { .. } | DrawCmd::EndLayer { .. }
            )
        {
            continue;
        }
        let layer = space.base + item.level;
        let m = &item.transform;
        // A shape laid on this item's layer and clip, under its map.
        let put = |overlay: &mut Overlay, s: UiShape, c: u32| {
            overlay.ui.shapes.push(placed(mapped_shape(s, m), layer, c));
        };
        match &item.cmd {
            DrawCmd::PushClip {
                geometry,
                center,
                half,
            } => clips.push(
                overlay,
                geometry.clone(),
                Vec2::new(center.0, center.1),
                Vec2::new(half.0, half.1),
                m,
            ),
            DrawCmd::PopClip => {
                clips.cuts.pop();
            }
            DrawCmd::World(w) => match &**w {
                WorldCmd::Occlude { at, depth, alpha } => {
                    clips.next_cut += 1;
                    let id = clips.next_cut;
                    clips
                        .occluders
                        .push((id, Vec2::new(at.0, at.1), *depth, *alpha));
                }
                WorldCmd::EndOcclude => {
                    clips.occluders.pop();
                }
                WorldCmd::Mass { batch, clip, .. } => {
                    let c = clips.index(overlay, clip);
                    overlay.ui.mass.push(og_draw::UiMassBatch {
                        layer,
                        clip: Some(c),
                        ..(**batch).clone()
                    });
                }
                WorldCmd::BeginPanel { target, rect } => {
                    let k = overlay.ui.layers.len() as u32;
                    if k >= MAX_LAYERS {
                        // No room: its items draw nowhere visible (off the screen).
                        spaces.push(Space::new(layer, None));
                        continue;
                    }
                    let base = desc.layer + LAYER_BASE + k as i32 * LAYER_SPAN;
                    overlay.ui.layers.push(og_draw::UiLayer {
                        capture: [base, base + LAYER_SPAN - 1],
                        rect: [rect.x, rect.y, rect.width, rect.height],
                        material: UiMaterial::Plain,
                        params: [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS],
                        mask: None,
                        target: Some(*target),
                        ground: None,
                        filter: None,
                        backdrop: false,
                    });
                    spaces.push(Space::new(base, Some(k)));
                }
                WorldCmd::EndPanel => {
                    // Composited nowhere: the mesh shows its target.
                    spaces.pop();
                }
                // Read above.
                WorldCmd::Anchor { .. } | WorldCmd::EndAnchor => {}
            },
            DrawCmd::BeginLayer {
                material,
                params,
                mask,
                rect,
                filter,
                role,
                ..
            } => {
                let k = overlay.ui.layers.len() as u32;
                // An element image's filter: the nearest layer around it
                // whose filter reads that image and has no layer for it
                // yet, else one opened since the innermost (a backdrop's,
                // ended before its element's children).
                let reader = match role {
                    LayerRole::Image(n) => {
                        let stand_in = crate::svg::filter::element_layer(*n);
                        let reads = |j: &u32| {
                            overlay.ui.layers[*j as usize]
                                .filter
                                .as_ref()
                                .is_some_and(|f| {
                                    f.primitives.iter().any(|p| {
                                        matches!(p.op, og_draw::UiFilterOp::Layer { layer } if layer == stand_in)
                                    })
                                })
                        };
                        let floor = spaces.last().and_then(|s| s.layer).map_or(0, |l| l + 1);
                        spaces
                            .iter()
                            .rev()
                            .filter_map(|s| s.layer)
                            .chain((floor..k).rev())
                            .find(reads)
                    }
                    _ => None,
                };
                if k >= MAX_LAYERS
                    || rect.is_empty()
                    || hidden
                    || matches!(role, LayerRole::Image(_)) && reader.is_none()
                {
                    // Drawn in place, over what its composite would be over;
                    // an element image or a backdrop, nowhere.
                    spaces.push(Space {
                        hidden: hidden || *role != LayerRole::Draw,
                        ..Space::new(layer, None)
                    });
                    continue;
                }
                if let (LayerRole::Image(n), Some(j)) = (role, reader)
                    && let Some(f) = &mut overlay.ui.layers[j as usize].filter
                {
                    // The filter reads this layer for its n-th element.
                    for p in &mut f.primitives {
                        if let og_draw::UiFilterOp::Layer { layer } = &mut p.op
                            && *layer == crate::svg::filter::element_layer(*n)
                        {
                            *layer = k;
                        }
                    }
                }
                let base = desc.layer + LAYER_BASE + k as i32 * LAYER_SPAN;
                overlay.ui.layers.push(og_draw::UiLayer {
                    capture: [base, base + LAYER_SPAN - 1],
                    rect: [rect.x, rect.y, rect.width, rect.height],
                    material: *material,
                    params: *params,
                    mask: mask.clone(),
                    target: None,
                    ground: None,
                    filter: filter.as_ref().map(|f| f.1.clone()),
                    backdrop: *role == LayerRole::Backdrop,
                });
                spaces.push(Space {
                    role: *role,
                    ..Space::new(base, Some(k))
                });
            }
            DrawCmd::EndLayer {
                geometry,
                rect,
                opacity,
                blend,
                clip,
                ground,
            } => {
                let Some(done) = spaces.pop() else {
                    continue;
                };
                let Some(k) = done.layer else {
                    continue;
                };
                // An element image composites nowhere: its filter reads it.
                if matches!(done.role, LayerRole::Image(_)) {
                    continue;
                }
                if let Some(Ground::Plane(key)) = ground {
                    // Laid through its map where it stands; nowhere when it
                    // shows its hidden back face (no quad).
                    let Some(g) = grounds.iter().find(|g| g.key == *key) else {
                        continue;
                    };
                    let [lx, ly, lw, lh] = overlay.ui.layers[k as usize].rect;
                    let corners = g.corners_of(Rect::new(lx, ly, lw, lh));
                    overlay.ui.layers[k as usize].ground = Some(og_draw::UiGround {
                        corners,
                        drape: g.drape,
                        plane: true,
                    });
                    let bounds = match corners {
                        Some(c) => {
                            let (lo, hi) = c
                                .iter()
                                .fold((c[0], c[0]), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
                            Rect::new(lo.x, lo.y, hi.x - lo.x, hi.y - lo.y).outset(1.0)
                        }
                        None => *clip,
                    }
                    .intersect(clip);
                    let parent = spaces.last().map_or(desc.layer, |s| s.base);
                    let c = clips.index(overlay, clip);
                    let mut s = rect_shape(bounds, 0.0, [1.0; 4]);
                    s.paint = UiPaint::Layer(k);
                    s.opacity = *opacity;
                    s.blend = *blend;
                    let level = sorted.get(key).copied().unwrap_or(item.level);
                    overlay.ui.shapes.push(placed(s, parent + level, c));
                    continue;
                }
                if let Some(Ground::Panel(key)) = ground {
                    // Draped where its panel is seen, under the screen's
                    // levels and the world frame's mass elements; nowhere
                    // when it is not seen (no layer composited is drawn).
                    let Some(g) = grounds.iter().find(|g| g.key == *key) else {
                        continue;
                    };
                    let [lx, ly, lw, lh] = overlay.ui.layers[k as usize].rect;
                    overlay.ui.layers[k as usize].ground = Some(og_draw::UiGround {
                        corners: g.corners_of(Rect::new(lx, ly, lw, lh)),
                        drape: g.drape,
                        plane: false,
                    });
                    let c = clips.index(overlay, clip);
                    let mut s = rect_shape(g.bounds, 0.0, [1.0; 4]);
                    s.paint = UiPaint::Layer(k);
                    s.opacity = *opacity;
                    s.blend = *blend;
                    overlay
                        .ui
                        .shapes
                        .push(placed(s, desc.layer - GROUND_BELOW, c));
                    continue;
                }
                let parent = spaces.last().map_or(desc.layer, |s| s.base);
                let [lx, ly, lw, lh] = overlay.ui.layers[k as usize].rect;
                // A composite's shape is the element's, under its map; the
                // whole layer is its rectangle on the screen.
                let (geometry, r, map) = match geometry {
                    Some(g) => (g.clone(), *rect, *m),
                    None => (
                        UiGeometry::Rect {
                            radii: [0.0; 4],
                            corner: og_draw::UiCorner::Round,
                        },
                        Rect::new(lx, ly, lw, lh),
                        Affine::IDENTITY,
                    ),
                };
                let c = clips.index(overlay, clip);
                let s = UiShape {
                    geometry,
                    center: Vec2::new(r.x + r.width / 2.0, r.y + r.height / 2.0),
                    half: Vec2::new(r.width / 2.0, r.height / 2.0),
                    transform: [1.0, 0.0, 0.0, 1.0],
                    paint: UiPaint::Layer(k),
                    mode: UiMode::Fill,
                    opacity: *opacity,
                    blend: *blend,
                    layer: parent + item.level,
                    clip: Some(c),
                };
                overlay.ui.shapes.push(mapped_shape(s, &map));
            }
            DrawCmd::Glass {
                rect,
                radius,
                blur,
                saturation,
                tint,
                material,
                alpha,
                clip,
            } => {
                let c = clips.index(overlay, clip);
                let mut s = rect_shape(*rect, *radius, *tint);
                s.mode = UiMode::Backdrop {
                    blur: *blur,
                    saturation: *saturation,
                    brightness: 1.0,
                    glass: *material,
                };
                s.opacity = *alpha;
                put(overlay, s, c);
            }
            DrawCmd::Quad {
                rect,
                radius,
                color,
                clip,
            } => {
                if rect.is_empty() {
                    continue;
                }
                let c = clips.index(overlay, clip);
                put(overlay, rect_shape(*rect, *radius, *color), c);
            }
            DrawCmd::Image {
                rect,
                texture,
                uv,
                color,
                clip,
            } => {
                let c = clips.index(overlay, clip);
                let mut s = rect_shape(*rect, 0.0, *color);
                if let Some(t) = texture {
                    s.paint = UiPaint::Image(og_draw::UiImage {
                        texture: *t,
                        uv: *uv,
                        size: Vec2::new(rect.width, rect.height),
                        density: 1.0,
                        tint: *color,
                        fit: og_draw::UiFit::Fill,
                        focal: Vec2::splat(0.5),
                        luminance: false,
                        offset: Default::default(),
                        nearest: false,
                    });
                }
                put(overlay, s, c);
            }
            DrawCmd::Bar {
                center,
                w,
                h,
                angle,
                color,
                clip,
            } => {
                let c = clips.index(overlay, clip);
                let (sin, cos) = angle.sin_cos();
                let mut s = rect_shape(
                    Rect::new(center.0 - w / 2.0, center.1 - h / 2.0, *w, *h),
                    0.0,
                    *color,
                );
                s.transform = [cos, sin, -sin, cos];
                put(overlay, s, c);
            }
            DrawCmd::Shape { shape, clip } => {
                let c = clips.index(overlay, clip);
                put(overlay, (**shape).clone(), c);
            }
            DrawCmd::Vector {
                vector,
                rect,
                transform,
                tint,
                clip,
            } => {
                let c = clips.index(overlay, clip);
                // Its rectangle's centre moved by the map, its local space
                // (about that centre) composed with the map's linear part.
                let (cx, cy) = m.apply(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
                overlay.ui.vectors.push(og_draw::UiVectorDraw {
                    vector: *vector,
                    rect: [
                        cx - rect.width / 2.0,
                        cy - rect.height / 2.0,
                        rect.width,
                        rect.height,
                    ],
                    transform: mul2(linear(m), *transform),
                    tint: *tint,
                    layer,
                    clip: Some(c),
                });
            }
            DrawCmd::Text { .. } => {
                let at = overlay.texts.len() as u32;
                if emit_text(&item.cmd, layer, &mut clips, overlay) && !m.is_identity() {
                    // No effects: the map alone.
                    overlay.ui.text_effects.push(og_draw::UiTextEffects {
                        text: at,
                        transform: linear(m),
                        ..Default::default()
                    });
                    map_text(overlay, at, m);
                }
            }
            DrawCmd::StyledText { text, effects } => {
                let at = overlay.texts.len() as u32;
                if emit_text(text, layer, &mut clips, overlay) {
                    let mut fx = (**effects).clone();
                    fx.text = at;
                    if !m.is_identity() {
                        // The map over the text's own (an italic's slant).
                        let own = if fx.transform == [0.0; 4] {
                            [1.0, 0.0, 0.0, 1.0]
                        } else {
                            fx.transform
                        };
                        fx.transform = mul2(linear(m), own);
                        let b = Rect::new(
                            fx.fill_box[0],
                            fx.fill_box[1],
                            fx.fill_box[2],
                            fx.fill_box[3],
                        );
                        let b = m.map_rect(&b);
                        fx.fill_box = [b.x, b.y, b.width, b.height];
                        map_text(overlay, at, m);
                    }
                    overlay.ui.text_effects.push(fx);
                }
            }
        }
    }
}

/// A styled text's effects under an upright map (translation and scale)
/// baked into its text: offsets scaled per axis, widths by the text's
/// height scale (as its size), the gradient's box moved.
pub(crate) fn scale_effects(fx: &mut og_draw::UiTextEffects, m: &Affine) {
    let sy = m.d.abs();
    for s in &mut fx.shadows {
        s.offset = Vec2::new(s.offset.x * m.a, s.offset.y * m.d);
        s.blur *= sy;
    }
    if let Some(o) = &mut fx.outline {
        o.width *= sy;
    }
    let b = Rect::new(
        fx.fill_box[0],
        fx.fill_box[1],
        fx.fill_box[2],
        fx.fill_box[3],
    );
    let b = m.map_rect(&b);
    fx.fill_box = [b.x, b.y, b.width, b.height];
}

/// Moves text `at`'s pen by the map: its glyphs take the map's linear part
/// from its [`og_draw::UiTextEffects::transform`].
fn map_text(overlay: &mut Overlay, at: u32, m: &Affine) {
    let t = &mut overlay.texts[at as usize];
    let (x, y) = m.apply(t.position.x, t.position.y);
    t.position.x = x;
    t.position.y = y;
}

/// Pushes a text command's text; false when it is not one.
fn emit_text(cmd: &DrawCmd, layer: i32, clips: &mut Clips, overlay: &mut Overlay) -> bool {
    let DrawCmd::Text {
        x,
        baseline,
        size,
        text,
        color,
        font,
        clip,
        tracking,
        weight,
        glow,
        placements,
        ..
    } = cmd
    else {
        return false;
    };
    let c = clips.index(overlay, clip);
    overlay.texts.push(Text {
        font: *font,
        text: text.clone(),
        position: og_core::glam::Vec3::new(*x, *baseline, 0.0),
        size: *size,
        rotation: 0.0,
        color: *color,
        layer,
        clip: Some(c),
        tracking: *tracking,
        weight: *weight,
        glow: glow.map(|(color, radius)| og_draw::TextGlow { color, radius }),
        placements: placements.clone(),
    });
    true
}

/// The lines of text a frame shows, each a [`DrawCmd::Text`] (a styled
/// one's text unwrapped), without the style's drops under them
/// (`textShadow`): a drop is its line pushed before it with the same
/// words, size, height and clip, a step off (at most 6 % of the size, or
/// a pixel font's pixel, and half its blur) in the shadow's colour. What
/// reads a line's ink (`og ui check`) reads the line, never its drop: a
/// blue "Lv. 125" over its black drop measured 1:1.
pub fn drawn_lines<'a>(cmds: impl IntoIterator<Item = &'a DrawCmd>) -> Vec<&'a DrawCmd> {
    let lines: Vec<(&DrawCmd, &String, f32, Rect, Rect)> = cmds
        .into_iter()
        .map(|c| match c {
            DrawCmd::StyledText { text, .. } => &**text,
            c => c,
        })
        .filter_map(|c| match c {
            DrawCmd::Text {
                text,
                size,
                bounds,
                clip,
                ..
            } => Some((c, text, *size, *bounds, *clip)),
            _ => None,
        })
        .collect();
    lines
        .iter()
        .enumerate()
        .filter(|&(i, &(_, text, size, b, clip))| {
            let reach = (size * 0.2).max(2.0);
            !lines[i + 1..].iter().any(|&(_, t, s, o, c)| {
                let (dx, dy) = (b.x - o.x, b.y - o.y);
                t == text
                    && s == size
                    && c == clip
                    && o.height == b.height
                    && (dx != 0.0 || dy != 0.0)
                    && dx.abs() <= reach
                    && dy.abs() <= reach
            })
        })
        .map(|(_, l)| l.0)
        .collect()
}
