//! World-space UI (`docs/ui.md` §14): anchored elements, world panels,
//! mass elements and meta UI.
//!
//! - **Anchored elements** ([`Ui::world_anchor`]): an element (a subtree
//!   like any other) anchored to a world point or an entity, offset in
//!   world units and CSS px. It is laid out at its own size and placed
//!   at the end of the tick on its point projected with the tick's camera
//!   ([`Ui::set_world_view`]): scaled with distance or kept at its screen
//!   size, faded or hidden where the scene is in front of it (a depth test
//!   against the frame's view depth, on the GPU), given a level of detail
//!   by distance, decluttered against the others by priority (shifted,
//!   stacked or hidden), and, for a waypoint or a ping, clamped to the
//!   safe area's edge with an arrow toward its point. It draws under the
//!   screen's own elements.
//! - **World panels** ([`Ui::world_panel`]): a subtree laid out at its own
//!   size away from the screen and drawn into a render target (an
//!   rgba16float [`og_render::RenderTargetHandle`] colour) that a mesh's
//!   material shows, lit (`map`) or unlit (`emissiveMap`). The pointer
//!   reaches it through a ray: where the ray from the pointer meets the
//!   panel's quad, the panel's controls take the press.
//! - **Mass elements** ([`Ui::define_mass`], [`Ui::spawn_mass`],
//!   [`Ui::push_mass`]): og-render's pooled, instanced path for damage
//!   numbers, overhead bars, nameplates, loot labels and status icons at
//!   thousands a frame. A template is declared once; an instance is a few
//!   values; its motion is the template's timeline, evaluated in the
//!   shader; no instance is laid out.
//! - **Meta UI** ([`Ui::meta`]): a full-screen layer over the scene (a
//!   damage vignette, a low-health pulse, a material over the frame).
//!
//! Determinism (AGENTS.md): the simulation never reads any of this. What
//! must replay (spawned mass elements and particles, each anchor's eased
//! fade and declutter offset) is the world state ([`Ui::save_world_state`]),
//! which the host keeps beside the UI state; the view and the entities'
//! poses are inputs the host hands in every tick.

use crate::draw::DrawCmd;
use crate::element::{Affine, Transform};
use crate::id::Id;
use crate::layout::{Anchor, Element, Float, NONE, Rect, Size};
use crate::style::Srgba;
use crate::ui::Ui;
use og_core::glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use og_core::math::Pose;
use og_render::{
    TextureHandle, UiGeometry, UiMassBatch, UiMassInstance, UiMassSpace, UiMassTemplate, UiShape,
};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Where panels are laid out: far right of any screen, one slot apart.
const PANEL_X: f32 = 100_000.0;

/// A rectangle in a world panel's layout (off any screen, either side once
/// a right-to-left layout mirrors it).
pub(crate) fn off_screen(r: &Rect) -> bool {
    r.x.abs() > PANEL_X / 2.0
}
const PANEL_SLOT: f32 = 8_192.0;
/// The most instances a pool keeps (the oldest go first).
pub const MASS_POOL_LIMIT: usize = 16_384;

/// The camera anchored elements project with and panel rays start from:
/// the tick's camera, handed in by the host before [`Ui::end`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldView {
    /// Projection times view (right handed, depth 0..1 as og-render's).
    pub view_proj: Mat4,
    pub eye: Vec3,
    /// Unit vector the camera looks along.
    pub forward: Vec3,
    pub orthographic: bool,
}

impl WorldView {
    /// A perspective camera at `eye` looking at `target` (`fov_y` degrees,
    /// `aspect` width over height), built with og-core's math.
    pub fn look_at(
        eye: Vec3,
        target: Vec3,
        up: Vec3,
        fov_y: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> WorldView {
        // look_at_rh by hand (products only).
        let f = (target - eye).normalize_or(Vec3::NEG_Z);
        let side = f.cross(up).normalize_or(Vec3::X);
        let u = side.cross(f);
        let view = Mat4::from_cols(
            Vec4::new(side.x, u.x, -f.x, 0.0),
            Vec4::new(side.y, u.y, -f.y, 0.0),
            Vec4::new(side.z, u.z, -f.z, 0.0),
            Vec4::new(-side.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
        );
        let (s, c) = og_core::math::sin_cos(fov_y.to_radians() / 2.0);
        let f = c / s;
        let r = far / (near - far);
        let proj = Mat4::from_cols(
            Vec4::new(f / aspect.max(1e-6), 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, r, -1.0),
            Vec4::new(0.0, 0.0, r * near, 0.0),
        );
        WorldView {
            view_proj: proj * view,
            eye,
            forward: (target - eye).normalize_or(Vec3::NEG_Z),
            orthographic: false,
        }
    }

    /// `p` on a `w`×`h` CSS px surface: (x, y, depth along the view axis);
    /// `None` behind the eye (its mirrored direction is in
    /// [`WorldView::direction`]).
    pub fn project(&self, p: Vec3, w: f32, h: f32) -> Option<(f32, f32, f32)> {
        let c = self.view_proj * p.extend(1.0);
        let depth = (p - self.eye).dot(self.forward);
        if c.w <= 1e-5 || depth <= 0.0 {
            return None;
        }
        let n = c.truncate() / c.w;
        Some(((n.x + 1.0) * 0.5 * w, (1.0 - n.y) * 0.5 * h, depth))
    }

    /// The screen direction from the centre toward `p`, unit, y down; a
    /// point behind the eye points the way it would turn to. Clip space's
    /// x and y before the divide are the view's sideways and up offsets
    /// scaled by the projection, so their signs hold behind the eye too
    /// (dividing by a negative w would mirror them).
    pub fn direction(&self, p: Vec3, w: f32, h: f32) -> Vec2 {
        let c = self.view_proj * p.extend(1.0);
        // Clip space to CSS px proportions, y down.
        Vec2::new(c.x * w, -c.y * h).normalize_or(Vec2::Y)
    }

    /// The ray through CSS px (`x`, `y`) of a `w`×`h` surface.
    pub fn ray(&self, x: f32, y: f32, w: f32, h: f32) -> (Vec3, Vec3) {
        let inv = self.view_proj.inverse();
        let nx = x / w.max(1e-6) * 2.0 - 1.0;
        let ny = 1.0 - y / h.max(1e-6) * 2.0;
        let near = inv.project_point3(Vec3::new(nx, ny, 0.0));
        let far = inv.project_point3(Vec3::new(nx, ny, 1.0));
        (near, (far - near).normalize_or(self.forward))
    }
}

/// What an anchored element or a panel is placed on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorldTarget {
    Point(Vec3),
    /// An entity, by its bits: its world pose as the host hands it in
    /// ([`Ui::set_world_poses`]; [`Ui::world_entities`] lists those asked for).
    Entity(u64),
}

impl Default for WorldTarget {
    fn default() -> Self {
        WorldTarget::Point(Vec3::ZERO)
    }
}

/// How an anchored element's size follows its distance.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AnchorScale {
    /// Its laid-out size whatever the distance.
    #[default]
    Constant,
    /// 1 at `reference` world units, larger nearer and smaller farther,
    /// clamped to `min..max`.
    Distance { reference: f32, min: f32, max: f32 },
}

/// What the scene in front of an anchored element does to it (a depth test
/// of its point against the frame's view depth, on the GPU).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum WorldOcclusion {
    #[default]
    None,
    /// Drawn at `alpha` behind the scene.
    Fade { alpha: f32 },
    /// Not drawn behind the scene.
    Hide,
}

/// How an anchored element that overlaps one of higher priority resolves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Declutter {
    /// It overlaps.
    #[default]
    None,
    /// Moved the shortest way clear of what it overlaps (else stacked).
    Shift,
    /// Moved above what it overlaps, one over another.
    Stack,
    /// Faded out while it overlaps.
    Hide,
}

/// Edge clamping (a waypoint, a ping): off the screen or behind the eye it
/// stays inside the safe area, on the edge toward its point, with an arrow
/// that points there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeClamp {
    /// Inset from the safe area's edges, CSS px.
    pub margin: f32,
    /// The arrow's colour (`None`: no arrow) and its length, CSS px.
    pub arrow: Option<Srgba>,
    pub arrow_size: f32,
}

impl Default for EdgeClamp {
    fn default() -> Self {
        EdgeClamp {
            margin: 24.0,
            arrow: Some(Srgba::WHITE),
            arrow_size: 14.0,
        }
    }
}

/// An anchored element ([`Ui::world_anchor`]).
#[derive(Clone, Debug, PartialEq)]
pub struct WorldAnchorDesc {
    pub target: WorldTarget,
    /// Added to the point, world units (above a head: `(0, 2, 0)`).
    pub offset: Vec3,
    /// Added on the screen, CSS px (y down).
    pub screen_offset: Vec2,
    /// The element's point that sits on the projected point.
    pub pivot: Anchor,
    /// Its box: direction, padding, gap, background, radius; `Fit` sizes.
    pub element: Element,
    pub scale: AnchorScale,
    pub occlusion: WorldOcclusion,
    /// Level-of-detail distances, ascending: level k from `lod[k - 1]` on
    /// ([`AnchorInfo::lod`]; build less at a higher level).
    pub lod: Vec<f32>,
    /// Hidden past this distance (0: never), fading over its last tenth.
    pub max_distance: f32,
    pub declutter: Declutter,
    /// Higher keeps its place; equal priorities favour the nearer.
    pub priority: i32,
    pub clamp: Option<EdgeClamp>,
    /// Ticks a fade (declutter, distance, entering) takes (0: 6).
    pub fade_ticks: u32,
}

impl Default for WorldAnchorDesc {
    fn default() -> Self {
        WorldAnchorDesc {
            target: WorldTarget::default(),
            offset: Vec3::ZERO,
            screen_offset: Vec2::ZERO,
            pivot: Anchor::Bottom,
            element: Element::column(),
            scale: AnchorScale::Constant,
            occlusion: WorldOcclusion::None,
            lod: Vec::new(),
            max_distance: 0.0,
            declutter: Declutter::None,
            priority: 0,
            clamp: None,
            fade_ticks: 0,
        }
    }
}

/// Where an anchored element is ([`Ui::world_anchor`]): its distance and
/// level of detail from the view as it is when built, the rest as the last
/// tick placed it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AnchorInfo {
    /// Its point is in front of the eye and within `max_distance`.
    pub in_view: bool,
    /// Its point projects inside the screen.
    pub on_screen: bool,
    /// Held on the screen's edge.
    pub clamped: bool,
    pub lod: u32,
    /// From the eye, world units.
    pub distance: f32,
    /// Where its pivot was drawn last tick, CSS px.
    pub x: f32,
    pub y: f32,
    /// The arrow's direction toward its point, radians clockwise from +x.
    pub angle: f32,
    pub scale: f32,
    /// Its eased opacity last tick (declutter, distance).
    pub alpha: f32,
}

/// A world panel ([`Ui::world_panel`]).
#[derive(Clone, Debug, PartialEq)]
pub struct WorldPanelDesc {
    /// The render target's colour (an rgba16float target).
    pub target: Option<TextureHandle>,
    /// The panel's layout size, CSS px (the target's pixels spread over it).
    pub width: f32,
    pub height: f32,
    /// The quad it is shown on: centred on the pose's position, `size`
    /// world units across its local +x and +y, facing +z (an
    /// og.render.planeGeometry of that size). `None`: not reachable by rays.
    pub quad: Option<(WorldTarget, Quat, Vec2)>,
    /// Its box (background, padding, direction); its size is the panel's.
    pub element: Element,
}

impl Default for WorldPanelDesc {
    fn default() -> Self {
        WorldPanelDesc {
            target: None,
            width: 480.0,
            height: 320.0,
            quad: None,
            element: Element::column(),
        }
    }
}

/// What the pointer does on a world panel this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PanelInfo {
    /// The ray from the pointer meets the panel's quad (in front of the
    /// screen's own elements); its controls take the press.
    pub pointed: bool,
    /// Where on the panel, CSS px from its top left.
    pub x: f32,
    pub y: f32,
    /// The hit's distance along the ray, world units.
    pub distance: f32,
}

/// A mass element template declared with [`Ui::define_mass`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MassHandle(pub u32);

/// A meta layer ([`Ui::meta`]): full screen, over the scene and under the
/// screen's elements.
#[derive(Clone, Debug, PartialEq)]
pub enum MetaKind {
    /// Darkens (or tints) the edges: transparent at `inner` of the half
    /// diagonal, `color` from `outer` on.
    Vignette {
        color: Srgba,
        inner: f32,
        outer: f32,
    },
    /// A flat tint over the frame.
    Tint { color: Srgba },
    /// The frame under it composited through a UI material (a ripple for
    /// sanity, a glitch for a hit): a backdrop of the scene in a layer.
    Material {
        material: og_render::UiMaterial,
        params: [[f32; 4]; og_render::UI_MATERIAL_PARAMS],
    },
}

/// A meta layer's desc ([`Ui::meta`]).
#[derive(Clone, Debug, PartialEq)]
pub struct MetaDesc {
    pub kind: MetaKind,
    /// 0..1: its opacity.
    pub intensity: f32,
    /// Pulses this many times a second (0: steady), between `intensity`
    /// and `intensity` × (1 − `pulse_depth`).
    pub pulse_hz: f32,
    pub pulse_depth: f32,
}

impl Default for MetaDesc {
    fn default() -> Self {
        MetaDesc {
            kind: MetaKind::Vignette {
                color: Srgba::rgb(160, 0, 0),
                inner: 0.55,
                outer: 1.0,
            },
            intensity: 1.0,
            pulse_hz: 0.0,
            pulse_depth: 0.5,
        }
    }
}

/// Commands of the UI pass that only world-space UI makes (inside
/// [`DrawCmd::World`]).
#[derive(Clone, Debug, PartialEq)]
pub enum WorldCmd {
    /// What follows, until [`WorldCmd::EndOcclude`], is depth tested at
    /// `at` (CSS px) against `depth` (world units along the view axis).
    Occlude {
        at: (f32, f32),
        depth: f32,
        alpha: f32,
    },
    EndOcclude,
    /// Mass elements (particles of an element), on its level and clip.
    Mass {
        batch: Box<UiMassBatch>,
        bounds: Rect,
        clip: Rect,
    },
    /// What follows, until [`WorldCmd::EndPanel`], draws into `target`,
    /// `rect` over the whole of it.
    BeginPanel {
        target: TextureHandle,
        rect: Rect,
    },
    EndPanel,
}

/// Mass elements of a frame that no element owns: the templates' pools and
/// this tick's immediate instances, emitted under the screen's elements.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldFrame {
    /// The tick mass elements' ages count from.
    pub tick: u32,
    pub batches: Vec<UiMassBatch>,
}

impl WorldFrame {
    /// Appends the batches to an overlay, one layer under `layer`.
    pub fn emit(&self, layer: i32, overlay: &mut og_render::Overlay) {
        overlay.ui.tick = self.tick;
        for b in &self.batches {
            overlay.ui.mass.push(UiMassBatch {
                layer: layer - 1,
                ..b.clone()
            });
        }
    }
}

/// An anchored element this frame.
#[derive(Clone, Debug)]
struct AnchorEntry {
    node: usize,
    desc: WorldAnchorDesc,
}

#[derive(Clone, Debug)]
struct PanelEntry {
    node: usize,
    desc: WorldPanelDesc,
    rect: Rect,
}

/// A panel as last placed: what rays test.
#[derive(Clone, Copy, Debug)]
struct PanelHit {
    key: u64,
    rect: Rect,
    center: Vec3,
    rotation: Quat,
    size: Vec2,
}

/// What a node's commands are wrapped in.
#[derive(Clone, Debug, Default)]
pub(crate) struct Marks {
    pub enter: Vec<DrawCmd>,
    pub leave: Vec<DrawCmd>,
}

/// An anchor's eased values (world state).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Smooth {
    pub alpha: f32,
    pub dx: f32,
    pub dy: f32,
}

/// What replays: spawned mass elements by template, emitters' particles,
/// anchors' eased values.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WorldState {
    pub pools: BTreeMap<u32, Vec<UiMassInstance>>,
    pub particles: BTreeMap<u64, crate::particles::EmitterState>,
    pub smooth: BTreeMap<u64, Smooth>,
}

/// World-space UI's part of [`Ui`].
#[derive(Clone, Debug, Default)]
pub(crate) struct WorldUi {
    pub view: Option<WorldView>,
    pub poses: BTreeMap<u64, Pose>,
    /// Entities asked for this frame.
    wanted: Vec<u64>,
    anchors: Vec<AnchorEntry>,
    panels: Vec<PanelEntry>,
    last_panels: Vec<PanelHit>,
    /// The panel the pointer is on this tick.
    pub pointed: Option<(u64, PanelInfo)>,
    pub emitters: Vec<crate::particles::EmitterEntry>,
    metas: Vec<(usize, MetaDesc)>,
    pub templates: Vec<(String, Arc<UiMassTemplate>, UiMassSpace)>,
    immediate: BTreeMap<u32, Vec<UiMassInstance>>,
    pub marks: BTreeMap<u64, Marks>,
    last: BTreeMap<u64, AnchorInfo>,
    pub frame: WorldFrame,
    pub state: WorldState,
    /// The pointer last tick (a viewport's drag).
    pub last_pointer: Vec2,
}

impl WorldUi {
    /// A world point of a target, with the pose's rotation.
    fn pose_of(&self, t: WorldTarget) -> Option<(Vec3, Quat)> {
        match t {
            WorldTarget::Point(p) => Some((p, Quat::IDENTITY)),
            WorldTarget::Entity(e) => self.poses.get(&e).map(|&(p, q, _)| (p, q)),
        }
    }

    fn want(&mut self, t: WorldTarget) {
        if let WorldTarget::Entity(e) = t
            && !self.wanted.contains(&e)
        {
            self.wanted.push(e);
        }
    }

    /// The area and clip a world layer is laid out in (`end`'s layout).
    pub(crate) fn area(&self, node: usize, screen: Rect) -> (Rect, Rect) {
        match self.panels.iter().find(|p| p.node == node) {
            Some(p) => (p.rect, p.rect),
            None => (screen, screen),
        }
    }

    pub(crate) fn is_panel_node(&self, node: usize) -> bool {
        self.panels.iter().any(|p| p.node == node)
    }
}

/// `p`'s level of detail.
fn lod_of(lod: &[f32], distance: f32) -> u32 {
    lod.iter().take_while(|&&d| distance >= d).count() as u32
}

/// The scale an anchor takes at `distance`.
fn scale_of(s: AnchorScale, distance: f32, ortho: bool) -> f32 {
    match s {
        AnchorScale::Constant => 1.0,
        AnchorScale::Distance { .. } if ortho => 1.0,
        AnchorScale::Distance {
            reference,
            min,
            max,
        } => (reference / distance.max(1e-3)).clamp(min.min(max), max.max(min)),
    }
}

/// Where a box of half size `half` stands clamped inside `area` along the
/// ray from `area`'s centre in `dir`.
fn clamp_to_edge(area: Rect, half: Vec2, dir: Vec2) -> Vec2 {
    let (cx, cy) = area.center();
    let hx = (area.width / 2.0 - half.x).max(0.0);
    let hy = (area.height / 2.0 - half.y).max(0.0);
    let tx = if dir.x.abs() > 1e-6 {
        hx / dir.x.abs()
    } else {
        f32::INFINITY
    };
    let ty = if dir.y.abs() > 1e-6 {
        hy / dir.y.abs()
    } else {
        f32::INFINITY
    };
    let t = tx.min(ty);
    Vec2::new(cx + dir.x * t, cy + dir.y * t)
}

/// The first spot clear of `placed` for `b` (moved by `shift` mode), or
/// `None`: overlapping; `reach` bounds the move.
fn declutter(mode: Declutter, b: Rect, placed: &[Rect], gap: f32, reach: f32) -> Option<Vec2> {
    let clear = |r: &Rect| !placed.iter().any(|p| p.overlaps(r));
    if clear(&b) {
        return Some(Vec2::ZERO);
    }
    let moved = |d: Vec2| Rect::new(b.x + d.x, b.y + d.y, b.width, b.height);
    if mode == Declutter::Shift {
        // The shortest way clear of each box it meets, nearest first.
        let mut tries: Vec<Vec2> = Vec::new();
        for p in placed.iter().filter(|p| p.overlaps(&b)) {
            tries.push(Vec2::new(0.0, p.y - gap - b.bottom()));
            tries.push(Vec2::new(0.0, p.bottom() + gap - b.y));
            tries.push(Vec2::new(p.x - gap - b.right(), 0.0));
            tries.push(Vec2::new(p.right() + gap - b.x, 0.0));
        }
        og_core::sort::sort_by_key(&mut tries, |d| (d.length() * 16.0) as u32);
        if let Some(d) = tries
            .into_iter()
            .find(|&d| d.length() <= reach && clear(&moved(d)))
        {
            return Some(d);
        }
    }
    if matches!(mode, Declutter::Shift | Declutter::Stack) {
        // Up over what it meets, again and again.
        let mut d = Vec2::ZERO;
        for _ in 0..16 {
            let r = moved(d);
            let Some(top) = placed
                .iter()
                .filter(|p| p.overlaps(&r))
                .map(|p| p.y)
                .reduce(f32::min)
            else {
                return Some(d);
            };
            d.y = top - gap - b.bottom();
            if -d.y > reach {
                return None;
            }
        }
    }
    None
}

impl Ui {
    // ── inputs from the host ─────────────────────────────────────────────

    /// The camera this tick's anchored elements project with and panel
    /// rays start from (the host sets it before [`Ui::end`]; `None`: none,
    /// so anchors are hidden).
    pub fn set_world_view(&mut self, view: Option<WorldView>) {
        self.world.view = view;
    }

    pub fn world_view(&self) -> Option<WorldView> {
        self.world.view
    }

    /// The entities' world poses (position, rotation, scale) for anchors and
    /// panels on entities: replaces the last ones.
    pub fn set_world_poses(&mut self, poses: &[(u64, Pose)]) {
        self.world.poses = poses.iter().copied().collect();
    }

    /// The entities this frame's anchors and panels are on, for the host to
    /// hand their poses in before [`Ui::end`].
    pub fn world_entities(&self) -> Vec<u64> {
        let mut v = self.world.wanted.clone();
        v.sort_unstable();
        v
    }

    // ── anchored elements ────────────────────────────────────────────────

    /// Where an anchor stands as of now: its distance and level from the
    /// current view, the rest from last tick.
    fn anchor_info(&self, key: u64, desc: &WorldAnchorDesc) -> AnchorInfo {
        let mut info = self.world.last.get(&key).copied().unwrap_or_default();
        if let (Some(view), Some((p, _))) = (self.world.view, self.world.pose_of(desc.target)) {
            let p = p + desc.offset;
            info.distance = (p - view.eye).length();
            info.lod = lod_of(&desc.lod, info.distance);
        }
        info
    }

    /// Opens an anchored element; its children follow until [`Ui::close`].
    pub fn open_world_anchor(&mut self, id: impl Into<Id>, desc: WorldAnchorDesc) -> AnchorInfo {
        let mut el = desc.element;
        // Laid out at its own size: never stretched to the screen.
        if matches!(el.width, Size::Grow { .. } | Size::Percent(_)) {
            el.width = Size::fit();
        }
        if matches!(el.height, Size::Grow { .. } | Size::Percent(_)) {
            el.height = Size::fit();
        }
        let i = self.open_layer("world", id.into(), el, Float::World, NONE);
        self.world.want(desc.target);
        let info = self.anchor_info(self.nodes[i].key, &desc);
        self.world.anchors.push(AnchorEntry { node: i, desc });
        info
    }

    /// An anchored element built by `f`, which reads where it stands (its
    /// level of detail, its distance, whether it is clamped).
    pub fn world_anchor(
        &mut self,
        id: impl Into<Id>,
        desc: WorldAnchorDesc,
        f: impl FnOnce(&mut Ui, &AnchorInfo),
    ) -> AnchorInfo {
        let info = self.open_world_anchor(id, desc);
        f(self, &info);
        self.close();
        info
    }

    // ── world panels ─────────────────────────────────────────────────────

    /// Opens a world panel; its children follow until [`Ui::close`].
    pub fn open_world_panel(&mut self, id: impl Into<Id>, desc: WorldPanelDesc) -> PanelInfo {
        let mut el = desc.element;
        el.width = Size::Fixed(desc.width.max(1.0));
        el.height = Size::Fixed(desc.height.max(1.0));
        let k = self.world.panels.len() as f32;
        let rect = Rect::new(
            PANEL_X + k * PANEL_SLOT,
            0.0,
            desc.width.max(1.0),
            desc.height.max(1.0),
        );
        let i = self.open_layer("panel", id.into(), el, Float::World, NONE);
        if let Some((t, ..)) = desc.quad {
            self.world.want(t);
        }
        let key = self.nodes[i].key;
        self.world.panels.push(PanelEntry {
            node: i,
            desc,
            rect,
        });
        match self.world.pointed {
            Some((k, info)) if k == key => info,
            _ => PanelInfo::default(),
        }
    }

    /// A world panel built by `f`.
    pub fn world_panel(
        &mut self,
        id: impl Into<Id>,
        desc: WorldPanelDesc,
        f: impl FnOnce(&mut Ui, &PanelInfo),
    ) -> PanelInfo {
        let info = self.open_world_panel(id, desc);
        f(self, &info);
        self.close();
        info
    }

    // ── mass elements ────────────────────────────────────────────────────

    /// Declares a mass element template (configuration, like a timeline): a
    /// second declaration of a name replaces it and keeps its handle.
    /// Text and numbers without a font take the UI's body font.
    pub fn define_mass(
        &mut self,
        name: &str,
        template: UiMassTemplate,
        space: UiMassSpace,
    ) -> MassHandle {
        let mut t = template;
        if t.font.is_none() {
            t.font = self.fonts.body.handle;
        }
        let t = Arc::new(t);
        if let Some(k) = self.world.templates.iter().position(|(n, ..)| n == name) {
            self.world.templates[k] = (name.to_owned(), t, space);
            return MassHandle(k as u32);
        }
        self.world.templates.push((name.to_owned(), t, space));
        MassHandle(self.world.templates.len() as u32 - 1)
    }

    /// A template by name.
    pub fn mass_handle(&self, name: &str) -> Option<MassHandle> {
        self.world
            .templates
            .iter()
            .position(|(n, ..)| n == name)
            .map(|k| MassHandle(k as u32))
    }

    /// Starts an instance of a template on this tick: it plays its
    /// template's timeline over its lifetime and leaves the pool after it.
    pub fn spawn_mass(&mut self, h: MassHandle, instance: UiMassInstance) {
        let Some((_, t, _)) = self.world.templates.get(h.0 as usize) else {
            self.warnings
                .push(format!("spawn_mass: no template {}; define it first", h.0));
            return;
        };
        if t.lifetime == 0 {
            // No life to play: shown this tick only.
            self.push_mass(h, instance);
            return;
        }
        let pool = self.world.state.pools.entry(h.0).or_default();
        if pool.len() >= MASS_POOL_LIMIT {
            pool.remove(0);
        }
        pool.push(UiMassInstance {
            start: self.state.tick as u32,
            ..instance
        });
    }

    /// Shows an instance of a template this tick only (an overhead bar, a
    /// nameplate, a status icon: rebuilt every tick with its values).
    pub fn push_mass(&mut self, h: MassHandle, instance: UiMassInstance) {
        if self.world.templates.get(h.0 as usize).is_none() {
            self.warnings
                .push(format!("push_mass: no template {}; define it first", h.0));
            return;
        }
        self.world
            .immediate
            .entry(h.0)
            .or_default()
            .push(UiMassInstance {
                start: self.state.tick as u32,
                ..instance
            });
    }

    /// The frame's own mass elements (the templates' pools and this tick's
    /// instances), to emit with [`WorldFrame::emit`] beside the UI frame.
    pub fn world_frame(&self) -> &WorldFrame {
        &self.world.frame
    }

    /// The live instances a template's pool holds.
    pub fn mass_count(&self, h: MassHandle) -> usize {
        self.world.state.pools.get(&h.0).map_or(0, Vec::len)
    }

    // ── meta UI ──────────────────────────────────────────────────────────

    /// A meta layer: full screen, over the scene and under the screen's
    /// elements, never taking input.
    pub fn meta(&mut self, id: impl Into<Id>, desc: MetaDesc) {
        let vp = self.viewport;
        let el = Element {
            width: Size::Fixed(vp.width),
            height: Size::Fixed(vp.height),
            ..Element::default()
        };
        let i = self.open_layer("meta", id.into(), el, Float::World, NONE);
        self.nodes[i].props.inert = true;
        self.close();
        self.world.metas.push((i, desc));
    }

    // ── the frame ────────────────────────────────────────────────────────

    /// Starts the world part of a frame: the pointer retargeted onto a
    /// world panel its ray meets, before anything is hit tested.
    pub(crate) fn world_begin(&mut self, input: &mut crate::input::InputFrame) {
        let w = &mut self.world;
        w.last_pointer = self.state.pointer;
        w.wanted.clear();
        w.anchors.clear();
        w.panels.clear();
        w.emitters.clear();
        w.metas.clear();
        w.immediate.clear();
        w.marks.clear();
        w.pointed = None;
        let Some(view) = w.view else {
            return;
        };
        if !input.pointer_inside || w.last_panels.is_empty() {
            return;
        }
        // The screen's own elements are over the world: a pointer on one
        // stays there.
        let p = input.pointer;
        let on_screen = self.state.hits.iter().any(|h| {
            !off_screen(&h.rect) && h.contains(p.x, p.y) && h.flags & crate::ui::HIT_POINTER != 0
        });
        if on_screen {
            return;
        }
        let vp = self.viewport;
        let (o, d) = view.ray(p.x, p.y, vp.width, vp.height);
        let mut best: Option<(f32, PanelHit, Vec2)> = None;
        for ph in &w.last_panels {
            let n = ph.rotation * Vec3::Z;
            let den = n.dot(d);
            if den.abs() < 1e-6 {
                continue;
            }
            let t = n.dot(ph.center - o) / den;
            if t <= 0.0 || best.is_some_and(|(bt, ..)| bt <= t) {
                continue;
            }
            let local = ph.rotation.inverse() * (o + d * t - ph.center);
            let u = local.x / ph.size.x + 0.5;
            let v = 0.5 - local.y / ph.size.y;
            if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
                best = Some((t, *ph, Vec2::new(u, v)));
            }
        }
        if let Some((t, ph, uv)) = best {
            let at = Vec2::new(
                ph.rect.x + uv.x * ph.rect.width,
                ph.rect.y + uv.y * ph.rect.height,
            );
            input.pointer = at;
            w.pointed = Some((
                ph.key,
                PanelInfo {
                    pointed: true,
                    x: uv.x * ph.rect.width,
                    y: uv.y * ph.rect.height,
                    distance: t,
                },
            ));
        }
    }

    /// Places this frame's world layers once laid out: anchors on their
    /// projected points (scaled, faded, decluttered, clamped), panels in
    /// their targets, meta layers over the screen. Called by `end` before
    /// transforms are applied.
    pub(crate) fn world_place(&mut self) {
        self.world_anchors();
        self.world_panels();
        self.world_metas();
    }

    /// Spawns particles where their elements are drawn, ages the pools and
    /// makes the frame's own batches. Called by `end` once transforms are
    /// applied.
    pub(crate) fn world_finish(&mut self) {
        let tick = self.state.tick as u32;
        self.world_particles(tick);
        // Pools: instances past their life leave.
        let templates = &self.world.templates;
        self.world.state.pools.retain(|&k, pool| {
            let Some((_, t, _)) = templates.get(k as usize) else {
                return false;
            };
            pool.retain(|i| tick.wrapping_sub(i.start) < t.lifetime);
            !pool.is_empty()
        });
        // The frame's own batches: pools, then this tick's instances.
        let mut batches = Vec::new();
        for (k, (_, t, space)) in self.world.templates.iter().enumerate() {
            let k = k as u32;
            let mut instances = self.world.state.pools.get(&k).cloned().unwrap_or_default();
            if let Some(now) = self.world.immediate.get(&k) {
                instances.extend_from_slice(now);
            }
            if instances.is_empty() {
                continue;
            }
            batches.push(UiMassBatch {
                template: t.clone(),
                space: *space,
                instances,
                layer: 0,
                clip: None,
            });
        }
        self.world.frame = WorldFrame { tick, batches };
    }

    fn world_anchors(&mut self) {
        let vp = self.viewport;
        let screen = Rect::new(0.0, 0.0, vp.width, vp.height);
        let safe = Rect::new(
            vp.safe_area.left,
            vp.safe_area.top,
            vp.width - vp.safe_area.horizontal(),
            vp.height - vp.safe_area.vertical(),
        );
        let anchors = std::mem::take(&mut self.world.anchors);
        // Where each lands before decluttering.
        struct Placed {
            entry: usize,
            key: u64,
            at: Vec2,
            pivot: (f32, f32),
            scale: f32,
            alpha: f32,
            depth: f32,
            info: AnchorInfo,
            rect: Rect,
            arrow: Option<(f32, Srgba, f32)>,
        }
        let mut placed: Vec<Placed> = Vec::new();
        let view = self.world.view;
        for (e, a) in anchors.iter().enumerate() {
            let n = &self.nodes[a.node];
            let key = n.key;
            let r = n.rect;
            let d = &a.desc;
            let mut info = AnchorInfo {
                scale: 1.0,
                ..AnchorInfo::default()
            };
            let (Some(view), Some((p, _))) = (view, self.world.pose_of(d.target)) else {
                placed.push(Placed {
                    entry: e,
                    key,
                    at: Vec2::ZERO,
                    pivot: d.pivot.fractions(),
                    scale: 1.0,
                    alpha: 0.0,
                    depth: 0.0,
                    info,
                    rect: Rect::default(),
                    arrow: None,
                });
                continue;
            };
            let p = p + d.offset;
            let distance = (p - view.eye).length();
            info.distance = distance;
            info.lod = lod_of(&d.lod, distance);
            let mut alpha = 1.0;
            if d.max_distance > 0.0 {
                let fade = d.max_distance * 0.1;
                alpha = ((d.max_distance - distance) / fade.max(1e-3)).clamp(0.0, 1.0);
            }
            let scale = scale_of(d.scale, distance, view.orthographic);
            info.scale = scale;
            let projected = view.project(p, vp.width, vp.height);
            let half = Vec2::new(r.width, r.height) * scale / 2.0;
            let mut pivot = d.pivot.fractions();
            let mut arrow = None;
            let (at, depth) = match projected {
                Some((x, y, depth)) => {
                    let at = Vec2::new(x, y) + d.screen_offset;
                    info.on_screen = screen.contains(x, y);
                    info.in_view = alpha > 0.0;
                    (at, depth)
                }
                None => {
                    info.in_view = false;
                    (Vec2::new(-1e5, -1e5), 0.0)
                }
            };
            let mut at = at;
            if let Some(c) = d.clamp {
                let area = Rect::new(
                    safe.x + c.margin,
                    safe.y + c.margin,
                    (safe.width - 2.0 * c.margin).max(0.0),
                    (safe.height - 2.0 * c.margin).max(0.0),
                );
                let inside = projected.is_some()
                    && at.x - half.x >= area.x
                    && at.x + half.x <= area.right()
                    && at.y - half.y >= area.y
                    && at.y + half.y <= area.bottom();
                if !inside {
                    let dir = view.direction(p, vp.width, vp.height);
                    let reach = c.arrow.map_or(0.0, |_| c.arrow_size);
                    at = clamp_to_edge(area, half + Vec2::splat(reach), dir);
                    pivot = (0.5, 0.5);
                    info.clamped = true;
                    info.angle = og_core::math::atan2(dir.y, dir.x);
                    if let Some(color) = c.arrow {
                        arrow = Some((info.angle, color, c.arrow_size));
                    }
                }
                if alpha > 0.0 {
                    info.in_view = true;
                }
            } else if projected.is_none() {
                alpha = 0.0;
            } else {
                // Culled once its whole box is off the screen.
                let b = Rect::new(
                    at.x - pivot.0 * half.x * 2.0,
                    at.y - pivot.1 * half.y * 2.0,
                    half.x * 2.0,
                    half.y * 2.0,
                );
                if !b.overlaps(&screen) {
                    alpha = 0.0;
                }
            }
            let rect = Rect::new(
                at.x - pivot.0 * half.x * 2.0,
                at.y - pivot.1 * half.y * 2.0,
                half.x * 2.0,
                half.y * 2.0,
            );
            placed.push(Placed {
                entry: e,
                key,
                at,
                pivot,
                scale,
                alpha,
                depth,
                info,
                rect,
                arrow,
            });
        }
        // Declutter by priority, then nearness, then build order.
        let mut by_rank: Vec<usize> = (0..placed.len()).collect();
        by_rank.sort_by(|&x, &y| {
            let (a, b) = (&placed[x], &placed[y]);
            anchors[b.entry]
                .desc
                .priority
                .cmp(&anchors[a.entry].desc.priority)
                .then(a.info.distance.total_cmp(&b.info.distance))
                .then(x.cmp(&y))
        });
        let gap = self.theme.space.xs;
        let mut taken: Vec<Rect> = Vec::new();
        let mut moves = vec![(Vec2::ZERO, 1.0f32); placed.len()];
        for &k in &by_rank {
            let pl = &placed[k];
            if pl.alpha <= 0.0 || pl.rect.is_empty() {
                continue;
            }
            let mode = anchors[pl.entry].desc.declutter;
            if mode == Declutter::None {
                taken.push(pl.rect);
                continue;
            }
            let reach = 3.0 * pl.rect.height.max(pl.rect.width.min(160.0));
            match declutter(mode, pl.rect, &taken, gap, reach) {
                Some(d) => {
                    let r = pl.rect;
                    taken.push(Rect::new(r.x + d.x, r.y + d.y, r.width, r.height));
                    moves[k] = (d, 1.0);
                }
                None => moves[k] = (Vec2::ZERO, 0.0),
            }
        }
        // Eased toward this tick's values (world state), applied.
        let mut seen = Vec::new();
        for (k, pl) in placed.iter().enumerate() {
            let d = &anchors[pl.entry].desc;
            let (shift, keep) = moves[k];
            let target_alpha = pl.alpha * keep;
            let fresh = !self.world.state.smooth.contains_key(&pl.key);
            let s = self.world.state.smooth.entry(pl.key).or_default();
            let step = if d.fade_ticks == 0 {
                1.0 / 6.0
            } else {
                1.0 / d.fade_ticks.clamp(1, 120) as f32
            };
            if fresh {
                s.dx = shift.x;
                s.dy = shift.y;
                s.alpha = 0.0;
            }
            s.alpha = if s.alpha < target_alpha {
                (s.alpha + step).min(target_alpha)
            } else {
                (s.alpha - step).max(target_alpha)
            };
            // Shifts glide (a third of the way a tick), so a crowd settles.
            s.dx += (shift.x - s.dx) * 0.34;
            s.dy += (shift.y - s.dy) * 0.34;
            if (shift.x - s.dx).abs() < 0.05 {
                s.dx = shift.x;
            }
            if (shift.y - s.dy).abs() < 0.05 {
                s.dy = shift.y;
            }
            let s = *s;
            seen.push(pl.key);
            let node = anchors[pl.entry].node;
            let r = self.nodes[node].rect;
            let px = r.x + pl.pivot.0 * r.width;
            let py = r.y + pl.pivot.1 * r.height;
            let at = pl.at + Vec2::new(s.dx, s.dy);
            let n = &mut self.nodes[node];
            n.props.transform = Transform {
                x: at.x - px,
                y: at.y - py,
                scale_x: pl.scale,
                scale_y: pl.scale,
                pivot_x: pl.pivot.0,
                pivot_y: pl.pivot.1,
                ..n.props.transform
            };
            n.props.opacity *= s.alpha;
            if s.alpha < 0.05 {
                n.props.inert = true;
            }
            let mut info = pl.info;
            info.x = at.x;
            info.y = at.y;
            info.alpha = s.alpha;
            self.world.last.insert(pl.key, info);
            let mut marks = Marks::default();
            if let Some(view) = self.world.view
                && d.occlusion != WorldOcclusion::None
                && !pl.info.clamped
                && pl.depth > 0.0
            {
                let _ = view;
                let alpha = match d.occlusion {
                    WorldOcclusion::Fade { alpha } => alpha,
                    _ => 0.0,
                };
                marks.enter.push(DrawCmd::World(Box::new(WorldCmd::Occlude {
                    at: (pl.at.x, pl.at.y),
                    depth: pl.depth,
                    alpha,
                })));
                marks
                    .leave
                    .push(DrawCmd::World(Box::new(WorldCmd::EndOcclude)));
            }
            if let Some((angle, color, size)) = pl.arrow
                && s.alpha > 0.0
            {
                // On the box's edge toward the point, pointing out.
                let half = Vec2::new(r.width, r.height) * pl.scale / 2.0;
                let dir = Vec2::new(og_core::math::cos(angle), og_core::math::sin(angle));
                let tx = if dir.x.abs() > 1e-6 {
                    half.x / dir.x.abs()
                } else {
                    f32::INFINITY
                };
                let ty = if dir.y.abs() > 1e-6 {
                    half.y / dir.y.abs()
                } else {
                    f32::INFINITY
                };
                let c = at + dir * (tx.min(ty) + size * 0.6);
                let mut shape = UiShape::rect(
                    [c.x - size / 2.0, c.y - size / 2.0, size, size],
                    0.0,
                    color.fade(s.alpha).linear(),
                );
                // A triangle whose first vertex points along `angle`.
                shape.geometry = UiGeometry::Polygon {
                    sides: 3,
                    inner: 1.0,
                    rounding: 1.5,
                    rotation: angle + std::f32::consts::FRAC_PI_2,
                };
                marks.leave.push(DrawCmd::Shape {
                    shape: Box::new(shape),
                    clip: screen,
                });
            }
            if !marks.enter.is_empty() || !marks.leave.is_empty() {
                self.world.marks.insert(pl.key, marks);
            }
        }
        self.world.state.smooth.retain(|k, _| seen.contains(k));
        self.world.last.retain(|k, _| seen.contains(k));
        self.world.anchors = anchors;
    }

    fn world_panels(&mut self) {
        let mut hits = Vec::new();
        for p in &self.world.panels {
            let key = self.nodes[p.node].key;
            if let Some(target) = p.desc.target {
                self.world.marks.insert(
                    key,
                    Marks {
                        enter: vec![DrawCmd::World(Box::new(WorldCmd::BeginPanel {
                            target,
                            rect: self.nodes[p.node].rect,
                        }))],
                        leave: vec![DrawCmd::World(Box::new(WorldCmd::EndPanel))],
                    },
                );
            }
            if let Some((t, rotation, size)) = p.desc.quad
                && let Some((center, q)) = self.world.pose_of(t)
            {
                hits.push(PanelHit {
                    key,
                    rect: self.nodes[p.node].rect,
                    center,
                    rotation: q * rotation,
                    size,
                });
            }
        }
        self.world.last_panels = hits;
    }

    fn world_metas(&mut self) {
        let metas = std::mem::take(&mut self.world.metas);
        let vp = self.viewport;
        let seconds = self.seconds() as f32;
        for (i, m) in &metas {
            let pulse = if m.pulse_hz > 0.0 {
                let s = og_core::math::sin(seconds * m.pulse_hz * std::f32::consts::TAU);
                1.0 - m.pulse_depth.clamp(0.0, 1.0) * (0.5 - 0.5 * s)
            } else {
                1.0
            };
            let alpha = (m.intensity * pulse).clamp(0.0, 1.0);
            let screen = Rect::new(0.0, 0.0, vp.width, vp.height);
            let cmds = meta_cmds(&m.kind, screen, alpha);
            let key = self.nodes[*i].key;
            self.world.marks.insert(
                key,
                Marks {
                    enter: cmds,
                    leave: Vec::new(),
                },
            );
        }
        self.world.metas = metas;
    }

    /// The order with world anchors and meta layers drawn (and hit) under
    /// the screen's own elements: right after the root, before its children.
    pub(crate) fn world_order(&self, order: &mut Vec<usize>) {
        if order.is_empty() {
            return;
        }
        let under = |i: usize| -> bool {
            let n = &self.nodes[i];
            n.float == Float::World && !self.world.is_panel_node(i)
        };
        // Subtrees of world layers below the root (each a run of the order).
        let mut moved = Vec::new();
        let mut rest = Vec::with_capacity(order.len());
        let mut k = 0;
        while k < order.len() {
            let i = order[k];
            if under(i) {
                let end = crate::layout::subtree_ends(&self.nodes, &order[k..])[0] + k;
                moved.extend_from_slice(&order[k..end]);
                k = end;
            } else {
                rest.push(i);
                k += 1;
            }
        }
        if moved.is_empty() {
            return;
        }
        let mut out = Vec::with_capacity(order.len());
        out.push(rest[0]);
        out.extend(moved);
        out.extend_from_slice(&rest[1..]);
        *order = out;
    }

    /// What a node's commands are wrapped in (an occlusion test, a panel's
    /// target, an edge arrow, a meta layer's paint, particles).
    pub(crate) fn world_marks(&self, key: u64) -> Option<&Marks> {
        self.world.marks.get(&key)
    }

    // ── state ────────────────────────────────────────────────────────────

    /// The world state's bytes: spawned mass elements and particles, each
    /// anchor's eased values. The host keeps them beside the UI state.
    pub fn save_world_state(&self) -> Vec<u8> {
        let s = &self.world.state;
        let mut out = Vec::new();
        out.extend_from_slice(b"OGUW");
        out.extend_from_slice(&1u32.to_le_bytes());
        let put = |out: &mut Vec<u8>, v: u64| out.extend_from_slice(&v.to_le_bytes());
        put(&mut out, s.pools.len() as u64);
        for (k, pool) in &s.pools {
            put(&mut out, u64::from(*k));
            put(&mut out, pool.len() as u64);
            out.extend_from_slice(bytemuck::cast_slice(pool));
        }
        put(&mut out, s.particles.len() as u64);
        for (k, e) in &s.particles {
            put(&mut out, *k);
            e.save(&mut out);
        }
        put(&mut out, s.smooth.len() as u64);
        for (k, v) in &s.smooth {
            put(&mut out, *k);
            for f in [v.alpha, v.dx, v.dy] {
                out.extend_from_slice(&f.to_le_bytes());
            }
        }
        out
    }

    /// Restores [`Ui::save_world_state`]'s bytes (empty: the empty state).
    pub fn restore_world_state(&mut self, bytes: &[u8]) -> og_core::Result<()> {
        self.world.state = decode_world(bytes).ok_or_else(|| {
            og_core::Error::new(
                og_core::ErrorKind::Parse,
                "og.ui world state: unreadable bytes",
            )
        })?;
        Ok(())
    }
}

/// Reads a world state's bytes.
fn decode_world(bytes: &[u8]) -> Option<WorldState> {
    let mut s = WorldState::default();
    if bytes.is_empty() {
        return Some(s);
    }
    let mut r = Reader { b: bytes, at: 0 };
    if r.take(4)? != b"OGUW" || r.u32()? != 1 {
        return None;
    }
    for _ in 0..r.u64()? {
        let k = r.u64()? as u32;
        let n = r.u64()? as usize;
        let size = std::mem::size_of::<UiMassInstance>();
        let raw = r.take(n.checked_mul(size)?)?;
        let pool: Vec<UiMassInstance> = raw
            .chunks_exact(size)
            .map(bytemuck::pod_read_unaligned)
            .collect();
        s.pools.insert(k, pool);
    }
    for _ in 0..r.u64()? {
        let k = r.u64()?;
        s.particles
            .insert(k, crate::particles::EmitterState::load(&mut r)?);
    }
    for _ in 0..r.u64()? {
        let k = r.u64()?;
        s.smooth.insert(
            k,
            Smooth {
                alpha: r.f32()?,
                dx: r.f32()?,
                dy: r.f32()?,
            },
        );
    }
    (r.at == bytes.len()).then_some(s)
}

/// Little-endian reads over a byte slice.
pub(crate) struct Reader<'a> {
    pub b: &'a [u8],
    pub at: usize,
}

impl Reader<'_> {
    pub fn take(&mut self, n: usize) -> Option<&[u8]> {
        let s = self.b.get(self.at..self.at.checked_add(n)?)?;
        self.at += n;
        Some(s)
    }
    pub fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    pub fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    pub fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
}

/// A meta layer's commands over `screen` at `alpha`.
fn meta_cmds(kind: &MetaKind, screen: Rect, alpha: f32) -> Vec<DrawCmd> {
    let r = [screen.x, screen.y, screen.width, screen.height];
    match kind {
        MetaKind::Tint { color } => vec![DrawCmd::Shape {
            shape: Box::new(UiShape::rect(r, 0.0, color.fade(alpha).linear())),
            clip: screen,
        }],
        MetaKind::Vignette {
            color,
            inner,
            outer,
        } => {
            let c = color.linear();
            let mut shape = UiShape::rect(r, 0.0, c);
            // A radial gradient from the centre out to the corners.
            shape.paint = og_render::UiPaint::Gradient(og_render::UiGradient {
                kind: og_render::UiGradientKind::Radial,
                from: Vec2::new(0.5, 0.5),
                to: Vec2::new(1.0, 1.0),
                stops: vec![
                    og_render::UiStop {
                        offset: inner.clamp(0.0, 1.0),
                        color: [c[0], c[1], c[2], 0.0],
                    },
                    og_render::UiStop {
                        offset: outer.clamp(0.0, 1.0).max(*inner + 1e-3),
                        color: [c[0], c[1], c[2], c[3]],
                    },
                ],
                ..Default::default()
            });
            shape.opacity = alpha;
            vec![DrawCmd::Shape {
                shape: Box::new(shape),
                clip: screen,
            }]
        }
        MetaKind::Material { material, params } => {
            // The frame under it as a backdrop, in a layer composited
            // through the material.
            let mut back = UiShape::rect(r, 0.0, [1.0; 4]);
            back.mode = og_render::UiMode::Backdrop {
                blur: 0.0,
                saturation: 1.0,
                brightness: 1.0,
            };
            // No tint over the scene: the backdrop alone.
            back.paint = og_render::UiPaint::Solid([1.0, 1.0, 1.0, 0.0]);
            vec![
                DrawCmd::BeginLayer {
                    material: *material,
                    params: *params,
                    mask: None,
                    expand: 0.0,
                    rect: screen,
                },
                DrawCmd::Shape {
                    shape: Box::new(back),
                    clip: screen,
                },
                DrawCmd::EndLayer {
                    geometry: None,
                    rect: screen,
                    opacity: alpha,
                    blend: og_render::UiBlend::Normal,
                    clip: screen,
                },
            ]
        }
    }
}

/// Maps a point by an element's map (particles spawn where it is drawn).
pub(crate) fn mapped(m: &Affine, p: Vec2) -> Vec2 {
    let (x, y) = m.apply(p.x, p.y);
    Vec2::new(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_clamp_meets_the_side_its_direction_points_to() {
        let area = Rect::new(0.0, 0.0, 200.0, 100.0);
        // Right: x at the right edge less the half width.
        let p = clamp_to_edge(area, Vec2::new(10.0, 5.0), Vec2::new(1.0, 0.0));
        assert_eq!(p, Vec2::new(190.0, 50.0));
        // Steeply down-left: the bottom edge first.
        let d = Vec2::new(-0.2, 1.0).normalize();
        let p = clamp_to_edge(area, Vec2::new(10.0, 5.0), d);
        assert!(
            (p.y - 95.0).abs() < 1e-3 && p.x < 100.0 && p.x > 80.0,
            "{p:?}"
        );
    }

    #[test]
    fn declutter_shifts_the_short_way_and_stacks_when_boxed_in() {
        let a = Rect::new(0.0, 0.0, 100.0, 20.0);
        // Overlapping a by 4 px at its bottom: down by 4 + gap is shortest.
        let b = Rect::new(0.0, 16.0, 100.0, 20.0);
        assert_eq!(
            declutter(Declutter::Shift, b, &[a], 2.0, 100.0),
            Some(Vec2::new(0.0, 6.0))
        );
        // Stack: always above.
        assert_eq!(
            declutter(Declutter::Stack, b, &[a], 2.0, 100.0),
            Some(Vec2::new(0.0, -38.0))
        );
        assert_eq!(declutter(Declutter::Hide, b, &[a], 2.0, 100.0), None);
        // Clear: no move.
        let c = Rect::new(0.0, 40.0, 100.0, 20.0);
        assert_eq!(
            declutter(Declutter::Hide, c, &[a], 2.0, 100.0),
            Some(Vec2::ZERO)
        );
    }

    #[test]
    fn a_point_behind_the_eye_points_the_way_to_turn() {
        let v = WorldView::look_at(
            Vec3::ZERO,
            Vec3::NEG_Z,
            Vec3::Y,
            60.0,
            16.0 / 9.0,
            0.1,
            100.0,
        );
        // Behind and to the right: not projected, and the arrow points right.
        let p = Vec3::new(3.0, 0.0, 5.0);
        assert!(v.project(p, 1280.0, 720.0).is_none());
        assert!(v.direction(p, 1280.0, 720.0).x > 0.9);
        // In front and up: y on the screen is up (negative).
        let q = Vec3::new(0.0, 3.0, -5.0);
        let (_, y, depth) = v.project(q, 1280.0, 720.0).unwrap();
        assert!(y < 360.0 && (depth - 5.0).abs() < 1e-4);
    }
}
