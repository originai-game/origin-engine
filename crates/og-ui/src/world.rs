//! World-space UI (`docs/ui.md` §1): anchored elements, world panels,
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

mod frame;
mod placement;

pub(crate) use frame::Reader;
use placement::lod_of;
pub(crate) use placement::mapped;

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
    /// Without `clamp`: while its point is in the safe area its box is kept
    /// inside it (a tag near the edge slides in rather than being cut), and
    /// it fades out as the point leaves the safe area.
    pub contain: bool,
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
            contain: false,
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
    /// Drawn by the UI itself onto where its quad is seen (a ground decal,
    /// a sign), in a band under the screen's elements, instead of into
    /// `target`: no render target or mesh. Inert: no ray reaches it, and
    /// its controls take no focus. Laid over the scene's surface as a
    /// decal ([`og_render::UiGround`]).
    pub projected: bool,
}

impl Default for WorldPanelDesc {
    fn default() -> Self {
        WorldPanelDesc {
            target: None,
            width: 480.0,
            height: 320.0,
            quad: None,
            element: Element::column(),
            projected: false,
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
    /// What follows, until [`WorldCmd::EndAnchor`], is the anchored element
    /// `key`: [`crate::UiFrame::emit_shifted`] moves it with its point.
    Anchor {
        key: u64,
    },
    EndAnchor,
}

/// Where an anchored element was placed at the end of the tick
/// ([`Ui::world_anchor_points`]): what a host that draws between ticks
/// projects again to move it with its point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnchorPoint {
    /// The element's key ([`WorldCmd::Anchor`]).
    pub key: u64,
    pub target: WorldTarget,
    /// Added to the target's point, world units.
    pub offset: Vec3,
    /// Held on the screen's edge (it stays there).
    pub clamped: bool,
}

/// A document's mass element (`<anchor mass>`): its template, where it
/// stands and its instance, and the element that owns it.
#[derive(Clone, Debug)]
pub(crate) struct KeptMass {
    pub handle: MassHandle,
    pub target: WorldTarget,
    pub offset: Vec3,
    pub instance: UiMassInstance,
    pub owner: u64,
}

/// A world panel the UI draws itself, as seen this tick: its layout `rect`
/// (CSS px), where that rect's top left, top right, bottom right and
/// bottom left corners land on the screen when all four are in front of
/// the camera, the screen `bounds` its decal can cover (the whole screen
/// while a corner of it is behind the camera), and its drape onto the
/// scene ([`og_render::UiDrape`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundQuad {
    pub key: u64,
    pub rect: Rect,
    pub corners: Option<[Vec2; 4]>,
    pub bounds: Rect,
    pub drape: og_render::UiDrape,
    /// A CSS 3D plane's 3D rendering context (a key; 0: none) and its
    /// depth toward the viewer there, CSS px.
    pub context: u64,
    pub depth: f32,
}

impl GroundQuad {
    /// Where the part `r` of the rect lands, its corners in the same order:
    /// through the plane's projective map, as the rect's own corners do.
    /// `None` while a corner is behind the camera.
    pub fn corners_of(&self, r: Rect) -> Option<[Vec2; 4]> {
        let m = og_render::square_to_quad(&self.corners?)?;
        let at = |x: f32, y: f32| {
            let u = (x - self.rect.x) / self.rect.width.max(1e-6);
            let v = (y - self.rect.y) / self.rect.height.max(1e-6);
            let w = m[2][0] * u + m[2][1] * v + m[2][2];
            Vec2::new(
                m[0][0] * u + m[0][1] * v + m[0][2],
                m[1][0] * u + m[1][1] * v + m[1][2],
            ) / w
        };
        let (x1, y1) = (r.x + r.width, r.y + r.height);
        Some([at(r.x, r.y), at(x1, r.y), at(x1, y1), at(r.x, y1)])
    }
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
    /// This tick's anchored elements as placed.
    pub placed: Vec<AnchorPoint>,
    pub frame: WorldFrame,
    pub state: WorldState,
    /// The pointer last tick (a viewport's drag).
    pub last_pointer: Vec2,
    /// Mounted documents' anchors by key, placed every tick whether or not
    /// their element was built again ([`Ui::keep_world_anchors`]).
    /// Each with the key of the in-flow element that owns it.
    pub kept: BTreeMap<u64, (WorldAnchorDesc, u64)>,
    /// Mounted documents' standing mass elements (`<anchor mass>` without
    /// an animation) by key: pushed every tick while their owner stands.
    pub kept_mass: BTreeMap<u64, KeptMass>,
    /// Mass elements built this tick (an immediate document's).
    pub mass_built: Vec<u64>,
    /// The one-shot mass elements already spawned, by key, while their
    /// row stands: a row spawns once.
    pub mass_spawned: std::collections::BTreeSet<u64>,
    /// Documents' emitter and meta effects by element key, registered
    /// every tick while their element stands (it is not built again).
    pub kept_effects: BTreeMap<u64, (Option<crate::particles::ParticlesDesc>, Option<MetaDesc>)>,
    /// This tick's projected panels as seen ([`crate::UiFrame::grounds`]).
    pub grounds: Vec<GroundQuad>,
    /// The keys of the CSS 3D plane roots (each a layer laid onto its
    /// plane, its quad among `grounds` each tick); kept across ticks.
    pub planes: std::collections::BTreeSet<u64>,
    /// Mounted documents' world panels by key, with the rect each was laid
    /// out in, kept for rays and placement as anchors are.
    kept_panels: BTreeMap<u64, (WorldPanelDesc, Rect, u64)>,
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

    /// The nodes this frame's world panels are laid out at.
    pub(crate) fn panel_nodes(&self) -> Vec<usize> {
        self.panels.iter().map(|p| p.node).collect()
    }
}
