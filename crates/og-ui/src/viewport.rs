//! The viewport element (`docs/ui.md` §3.3): a camera's view of a scene
//! layer (a character on a turntable, an item to inspect, a car in a
//! garage) shown in the layout like an image, sized by it, and taking
//! input through: a press or a drag inside it becomes a ray into that
//! scene and a drag in CSS px, so a menu can orbit, pick and rotate.
//!
//! The view itself is the engine's secondary camera (og.CameraView) into a
//! render target on its own layers; the element shows the target's colour
//! and asks for the pixel size that fills it ([`ViewportResponse::pixels`]).

use crate::id::Id;
use crate::layout::{Content, Element, Rect, Size};
use crate::style::Srgba;
use crate::ui::Ui;
use crate::world::WorldView;
use og_core::glam::{Vec2, Vec3};
use og_draw::TextureHandle;

/// A viewport ([`Ui::scene_viewport`]).
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportDesc {
    /// The render target's colour its camera draws into.
    pub texture: Option<TextureHandle>,
    pub width: Size,
    pub height: Size,
    /// Width over height (0: free).
    pub aspect: f32,
    /// Its camera, for the rays (`aspect` of the projection: the element's).
    pub view: Option<WorldView>,
    /// Drawn under the view (and alone without a texture).
    pub background: Option<Srgba>,
    pub radius: f32,
    /// What the probe and the accessibility tree call it.
    pub label: String,
    /// It takes the focus, and Left and Right (Up and Down) step it
    /// ([`ViewportResponse::adjust`]): `<model-viewer>`'s camera-controls.
    pub controls: bool,
}

impl Default for ViewportDesc {
    fn default() -> Self {
        ViewportDesc {
            texture: None,
            width: Size::Fixed(320.0),
            height: Size::Fixed(320.0),
            aspect: 0.0,
            view: None,
            background: None,
            radius: 0.0,
            label: String::new(),
            controls: false,
        }
    }
}

/// What the pointer does in a viewport this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewportResponse {
    /// Where it was laid out last tick, CSS px.
    pub rect: Rect,
    /// The target size that fills it at the device's pixel ratio.
    pub pixels: (u32, u32),
    pub hovered: bool,
    /// A press that started in it is held.
    pub pressed: bool,
    /// Pressed and released in it.
    pub clicked: bool,
    /// The pointer's move while held this tick, CSS px (orbit, turn).
    pub drag: Vec2,
    /// The wheel over it (zoom).
    pub wheel: Vec2,
    /// The pointer in it, fractions of its box (0, 0: top left).
    pub at: Option<Vec2>,
    /// The ray into its scene through the pointer (origin, unit direction).
    pub ray: Option<(Vec3, Vec3)>,
    /// With `controls`, focused: Left and Right (-1, 1), Up and Down.
    pub adjust: i32,
    pub adjust_y: i32,
}

impl Ui {
    /// A viewport: a camera's view shown in the layout, input passed in.
    pub fn scene_viewport(&mut self, id: impl Into<Id>, desc: ViewportDesc) -> ViewportResponse {
        let i = self.leaf(
            "viewport",
            id.into(),
            Element {
                width: desc.width,
                height: desc.height,
                aspect: desc.aspect.max(0.0),
                interactive: true,
                focusable: desc.controls,
                adjustable: desc.controls,
                arrows: if desc.controls {
                    crate::ARROW_LEFT | crate::ARROW_RIGHT | crate::ARROW_UP | crate::ARROW_DOWN
                } else {
                    0
                },
                background: desc.background,
                radius: desc.radius,
                ..Element::default()
            },
        );
        if let Some(t) = desc.texture {
            self.nodes[i].content = Content::Image {
                texture: Some(t),
                uv: [0.0, 0.0, 1.0, 1.0],
                tint: Srgba::WHITE,
                object: Default::default(),
            };
        }
        self.nodes[i].probe_text = desc.label.clone();
        let key = self.nodes[i].key;
        let r = self.respond(key);
        let mut out = ViewportResponse {
            hovered: r.hovered,
            pressed: r.pressed,
            clicked: r.clicked,
            adjust: r.adjust,
            adjust_y: r.adjust_y,
            ..ViewportResponse::default()
        };
        let Some(hit) = self.hit(key) else {
            return out;
        };
        let rect = hit.local;
        let dpr = self.env.viewport.dpr();
        out.rect = rect;
        out.pixels = (
            (rect.width * dpr).round().max(1.0) as u32,
            (rect.height * dpr).round().max(1.0) as u32,
        );
        let p = self.interaction.input.pointer;
        let (lx, ly) = hit.inv.apply(p.x, p.y);
        let inside = rect.contains(lx, ly);
        if r.pressed {
            out.drag = p - self.world.last_pointer;
        }
        if inside && self.interaction.input.pointer_inside {
            out.wheel = self.interaction.input.wheel;
            let at = Vec2::new(
                (lx - rect.x) / rect.width.max(1e-6),
                (ly - rect.y) / rect.height.max(1e-6),
            );
            out.at = Some(at);
            if let Some(v) = desc.view {
                out.ray = Some(v.ray(at.x, at.y, 1.0, 1.0));
            }
        }
        out
    }
}
