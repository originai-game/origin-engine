//! The world → `RenderScene` extraction and the renderer a host attached.
//!
//! Rendering reads the world after the tick and never writes it back
//! (AGENTS.md: rendering never feeds back into simulation). Everything here
//! takes the world shared, so the probe's `Capture` hook can extract the
//! frame of the current tick from the `&World` it is handed.
//!
//! What is drawn:
//! - entities with `og.Model` (a loaded glTF's default scene) at their world
//!   transform, composed from `og.Transform` down the `og.Parent` chain;
//! - entities with `og.Mesh` (geometry and material handles), when og-core
//!   can register its bool fields;
//! - the first `og.DirectionalLight` (else the renderer's default sun), the
//!   `og.PointLight` / `og.SpotLight` (og-render culls and clusters them);
//! - the `og.Camera` and `og.RenderSettings` resources;
//! - the text `og.render.text` queued during the tick, with cooked
//!   multi-page MSDF fonts mapped to one `MsdfFont` per page.

use super::codec::Layout;
use glam::{Mat4, Quat, Vec3};
use og_ecs::{Entity, QueryDesc, World};
use og_render::{FontHandle, MaterialHandle, MeshHandle, RenderScene};
use serde_json::Value;
use std::collections::BTreeMap;

/// A renderer the host attached to an engine (an OffscreenCanvas surface on
/// the web, an offscreen target on native).
pub trait RenderBackend {
    /// The IDL `RenderTier` (`webgpu`, `webgl2`, `metal`, `vulkan`,
    /// `vulkan-cpu`), og-render's `tier_name`, which also names the
    /// reference frames.
    fn tier(&self) -> String;
    /// The render target in device pixels.
    fn size(&self) -> (u32, u32);
    /// What `RenderOptions.antialias` resolved to: `off`, `msaa4` or `taa`.
    fn antialias(&self) -> String {
        "off".into()
    }
    /// The texture encodings the device takes (og-render's
    /// `RenderInfo::texture_formats`): loads transcode to them. `None`: RGBA8.
    fn texture_formats(&self) -> Option<og_render::TextureFormats> {
        None
    }
    /// Draw one frame of the scene to the display target.
    fn render(&mut self, scene: &RenderScene) -> Result<(), String>;
    /// Draw the scene offscreen at `width`×`height` (0×0: the target's size)
    /// and read it back. Native hosts block on the readback; the web host
    /// answers a frame it requested earlier (request on tick N, answer on
    /// N + 1) and `None` while the copy is in flight.
    fn capture(
        &mut self,
        scene: &RenderScene,
        width: u32,
        height: u32,
    ) -> Option<og_render::RgbaImage>;
}

/// One loaded font: a cooked MTSDF font as one `MsdfFont` per atlas page.
/// Every page's font carries every glyph's advance, and the plane bounds only
/// of its own page's glyphs (the others are blank), so drawing the same
/// string once per page lays every page out identically.
pub(crate) struct FontPages {
    pub pages: Vec<FontHandle>,
}

/// A queued `og.render.text`.
#[derive(Clone, Debug)]
pub(crate) struct QueuedText {
    pub font: usize,
    pub text: String,
    pub position: Vec3,
    pub size: f32,
    pub rotation: f32,
    pub color: [f32; 4],
    pub layer: i32,
    /// In the overlay (CSS pixels, y down), cut to this clip.
    pub screen: bool,
    pub clip: Option<[f32; 4]>,
    /// World text: an outline (width as a fraction of the distance range) and its colour.
    pub outline: f32,
    pub outline_color: [f32; 4],
}

/// One model's parts: per part the mesh, the material and the node's
/// transform within the model.
pub(crate) type Parts = Vec<(MeshHandle, MaterialHandle, Mat4)>;

/// The render side of an engine, shared with the probe's capture hook.
pub(crate) struct RenderState {
    pub scene: RenderScene,
    pub backend: Option<Box<dyn RenderBackend>>,
    /// Material handles (`og.render.standardMaterial`, glTF materials) by index.
    pub materials: Vec<MaterialHandle>,
    /// Geometry handles (glTF primitives) by index.
    pub geometries: Vec<MeshHandle>,
    /// Model parts by asset handle bits.
    pub models: BTreeMap<u64, Parts>,
    /// Fonts by Font handle index.
    pub fonts: Vec<FontPages>,
    /// Text queued during the current tick, and that tick.
    pub texts: Vec<QueuedText>,
    pub texts_tick: u64,
    /// The effects games declare as data (`super::fx`), with the sprites
    /// and ribbons queued during the current tick.
    pub fx: super::fx::FxState,
    /// Textures by Texture handle index (`og.load.texture`).
    pub textures: Vec<og_render::TextureHandle>,
    /// Layouts of the engine records extraction reads.
    pub camera: Option<Layout>,
    pub settings: Option<Layout>,
    pub draw: DrawLayouts,
    /// The game's RenderOptions (antialias, shadowMapSize).
    pub options: Value,
    /// The play surface the host reported (`set_viewport`); `None` uses the
    /// render target at one CSS pixel per device pixel.
    pub viewport: Option<og_ui::Viewport>,
    /// The engine's UI, drawn over every frame, and its atlas once added.
    pub ui: Option<og_ui::SharedUi>,
    pub ui_atlas: Option<og_render::TextureHandle>,
    /// A UI font's first atlas page → every page, so text is drawn per page.
    pub ui_pages: BTreeMap<FontHandle, Vec<FontHandle>>,
    /// Warnings of the last extraction, for the engine's log.
    pub notes: Vec<String>,
    /// The feel modules' drawing state (`super::feel`).
    pub feel: super::feel::FeelRender,
    /// The atmosphere's and the physics extras' drawing state (`super::atmosphere`).
    pub atmo: super::atmosphere::AtmoState,
    /// The terrains loaded (`super::terrain`).
    pub terrain: super::terrain::TerrainState,
}

impl RenderState {
    pub fn new(options: Value) -> RenderState {
        RenderState {
            scene: RenderScene::new(),
            backend: None,
            materials: Vec::new(),
            geometries: Vec::new(),
            models: BTreeMap::new(),
            fonts: Vec::new(),
            texts: Vec::new(),
            texts_tick: 0,
            fx: super::fx::FxState::default(),
            textures: Vec::new(),
            camera: None,
            settings: None,
            draw: DrawLayouts::default(),
            options,
            viewport: None,
            ui: None,
            ui_atlas: None,
            ui_pages: BTreeMap::new(),
            notes: Vec::new(),
            feel: Default::default(),
            atmo: Default::default(),
            terrain: Default::default(),
        }
    }
}

fn f(v: &Value, key: &str, default: f32) -> f32 {
    v[key].as_f64().map_or(default, |x| x as f32)
}

fn vec3(v: &Value) -> Vec3 {
    Vec3::new(f(v, "x", 0.0), f(v, "y", 0.0), f(v, "z", 0.0))
}

fn color(v: &Value, default: f32) -> Vec3 {
    Vec3::new(f(v, "r", default), f(v, "g", default), f(v, "b", default))
}

fn on(v: &Value, key: &str, default: bool) -> bool {
    match &v[key] {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        _ => default,
    }
}

/// The value of a resource the world holds, decoded with its layout.
fn resource(world: &World, layout: Option<&Layout>) -> Option<Value> {
    let layout = layout?;
    let id = world.id_by_name(&layout.schema).ok()?;
    let bytes = world.resource_bytes(id).ok()?;
    Some(layout.decode(&bytes))
}

/// Local transforms and parents of every entity with `og.Transform`, for
/// composing world matrices.
pub struct Hierarchy {
    local: BTreeMap<u64, Mat4>,
    parent: BTreeMap<u64, u64>,
    world: BTreeMap<u64, Mat4>,
}

impl Hierarchy {
    pub(crate) fn of(world: &World) -> Hierarchy {
        let mut h = Hierarchy {
            local: BTreeMap::new(),
            parent: BTreeMap::new(),
            world: BTreeMap::new(),
        };
        if let Ok(tid) = world.id_by_name("og.Transform") {
            let fields = ["position", "quaternion", "scale"];
            let refs: Vec<_> = fields
                .iter()
                .filter_map(|n| world.field_by_id(tid, n).ok())
                .collect();
            let q = QueryDesc {
                read: vec!["og.Transform".into()],
                ..QueryDesc::default()
            };
            if let (Ok(all), 3) = (world.query(&q), refs.len()) {
                for a in all {
                    let (Ok(p), Ok(r), Ok(s)) = (
                        a.read::<Vec3>(refs[0]),
                        a.read::<Quat>(refs[1]),
                        a.read::<Vec3>(refs[2]),
                    ) else {
                        continue;
                    };
                    for (i, e) in a.entities().iter().enumerate() {
                        h.local.insert(
                            e.to_bits(),
                            Mat4::from_scale_rotation_translation(s[i], r[i], p[i]),
                        );
                    }
                }
            }
        }
        if let Ok(pid) = world.id_by_name("og.Parent")
            && let Ok(field) = world.field_by_id(pid, "entity")
        {
            let q = QueryDesc {
                read: vec!["og.Parent".into()],
                ..QueryDesc::default()
            };
            if let Ok(all) = world.query(&q) {
                for a in all {
                    if let Ok(p) = a.read::<u64>(field) {
                        for (e, parent) in a.entities().iter().zip(p.iter()) {
                            h.parent.insert(e.to_bits(), *parent);
                        }
                    }
                }
            }
        }
        h
    }

    /// The world matrix of an entity: its local transform after its parents'.
    pub(crate) fn world_of(&mut self, e: u64) -> Mat4 {
        if let Some(m) = self.world.get(&e) {
            return *m;
        }
        let mut chain = vec![e];
        let mut cur = e;
        while let Some(&p) = self.parent.get(&cur) {
            if chain.contains(&p) || chain.len() > 256 {
                break;
            }
            chain.push(p);
            cur = p;
        }
        let mut m = Mat4::IDENTITY;
        for id in chain.iter().rev() {
            if let Some(w) = self.world.get(id) {
                m = *w;
                continue;
            }
            m *= self.local.get(id).copied().unwrap_or(Mat4::IDENTITY);
            self.world.insert(*id, m);
        }
        m
    }
}

/// Decoded rows of one component: `(entity bits, value)`.
fn rows(world: &World, layout: &Layout) -> Vec<(u64, Value)> {
    let Ok(id) = world.id_by_name(&layout.schema) else {
        return Vec::new();
    };
    let q = QueryDesc {
        read: vec![layout.schema.clone()],
        ..QueryDesc::default()
    };
    let mut out = Vec::new();
    let Ok(all) = world.query(&q) else {
        return out;
    };
    let entities: Vec<Entity> = all.flat_map(|a| a.entities().to_vec()).collect();
    for e in entities {
        if let Ok(bytes) = world.get_bytes(e, id) {
            out.push((e.to_bits(), layout.decode(&bytes)));
        }
    }
    out
}

/// The layouts of the drawing components the engine registered.
#[derive(Default)]
pub(crate) struct DrawLayouts {
    pub model: Option<Layout>,
    pub mesh: Option<Layout>,
    pub directional: Option<Layout>,
    pub point: Option<Layout>,
    pub spot: Option<Layout>,
    /// The effect records (`super::fx`).
    pub fx: super::fx::FxLayouts,
    /// The atmosphere's and the physics extras' records (`super::atmosphere`).
    pub atmo: super::atmosphere::AtmoLayouts,
    /// og.Terrain and og.TerrainScatter (`super::terrain`).
    pub terrain: Option<Layout>,
    pub scatter: Option<Layout>,
}

/// Rebuild the per-frame part of `rs.scene` from the world as of its tick.
pub(crate) fn extract(world: &World, rs: &mut RenderState) {
    let layouts = &rs.draw;
    let camera = resource(world, rs.camera.as_ref());
    let settings = resource(world, rs.settings.as_ref());
    let mut h = Hierarchy::of(world);
    rs.atmo.palettes.clear();
    let scene = &mut rs.scene;
    let (models, geometries, materials) = (&rs.models, &rs.geometries, &rs.materials);
    scene.instances.clear();
    scene.lights.clear();
    scene.texts.clear();
    scene.sprites.clear();
    scene.overlay = None;
    scene.joint_palettes.clear();
    scene.skinned_instances.clear();
    scene.sprite_order.clear();
    scene.sprite_fx.clear();
    scene.text_outlines.clear();
    scene.tile_layers.clear();
    scene.trail_layers.clear();
    let time = world.time();
    scene.tick = time.tick;
    scene.tick_rate = if time.dt > 0.0 { 1.0 / time.dt } else { 60.0 };
    // og.render.sprite and og.render.text are immediate mode: what the last
    // tick submitted (or a command since it) is drawn, and a tick that
    // submitted nothing draws nothing. The queues are cleared lazily by the
    // next submission, so a queue from an older tick is stale here.
    let now = world.time().tick;
    let fresh = rs.texts_tick == now || rs.texts_tick + 1 == now;

    // Camera.
    let mut exposure = 1.0f32;
    if let Some(c) = &camera {
        let q = &c["quaternion"];
        let rot = Quat::from_xyzw(
            f(q, "x", 0.0),
            f(q, "y", 0.0),
            f(q, "z", 0.0),
            f(q, "w", 1.0),
        )
        .normalize();
        let position = vec3(&c["position"]);
        let cam = &mut scene.camera;
        cam.position = position;
        cam.target = position + rot * Vec3::NEG_Z;
        cam.up = rot * Vec3::Y;
        cam.near = f(c, "near", 0.1);
        cam.far = f(c, "far", 2000.0);
        let zoom = f(c, "zoom", 1.0).max(0.01);
        cam.projection = if on(c, "orthographic", false) {
            og_render::Projection::Orthographic {
                height: f(c, "height", 10.0) / zoom,
            }
        } else {
            // three.js zoom narrows the field of view: tan(fov/2) / zoom.
            let half = (f(c, "fov", 50.0).to_radians() / 2.0).tan() / zoom;
            og_render::Projection::Perspective {
                fov_y_degrees: (2.0 * half.atan()).to_degrees(),
            }
        };
        exposure = f(c, "exposure", 1.0).max(0.0);
        scene.environment.background = og_render::Background::Color(color(&c["background"], 0.0));
    }

    // Settings: environment, sky, bloom, exposure, tone mapping.
    if let Some(s) = &settings {
        scene.environment.intensity = f(s, "environmentIntensity", 0.55);
        if on(s, "sky", false) {
            scene.environment.background = og_render::Background::Sky;
        }
        scene.post.bloom = on(s, "bloom", false).then(|| og_render::Bloom {
            threshold: f(s, "bloomThreshold", 1.0),
            knee: f(s, "bloomKnee", 0.5),
            intensity: f(s, "bloomIntensity", 0.6),
        });
        scene.post.tonemap = if on(s, "toneMapped", true) {
            og_render::Tonemap::Aces
        } else {
            og_render::Tonemap::None
        };
        scene.camera.exposure = if on(s, "autoExposure", false) {
            og_render::Exposure::Auto {
                compensation: exposure.max(1e-6).log2(),
                min_ev: -10.0,
                max_ev: 10.0,
            }
        } else {
            og_render::Exposure::linear(exposure)
        };
    } else {
        scene.camera.exposure = og_render::Exposure::linear(exposure);
    }

    // Models: posed by og-anim when the entity has a Skeleton.
    let posed = super::fx::skeletons(world, rs);
    let mut keys: Vec<(u64, u32)> = Vec::new();
    let model_rows = layouts
        .model
        .as_ref()
        .map(|m| rows(world, m))
        .unwrap_or_default();
    let model_parts: Vec<(u64, u64, Parts)> = model_rows
        .into_iter()
        .filter_map(|(e, v)| {
            let asset = v["asset"].as_f64().unwrap_or(0.0) as u64;
            models.get(&asset).map(|p| (e, asset, p.clone()))
        })
        .collect();
    let mesh_rows = layouts
        .mesh
        .as_ref()
        .map(|m| rows(world, m))
        .unwrap_or_default();
    let (geometries, materials) = (geometries.clone(), materials.clone());
    // Flashing meshes draw through a flash copy of their material.
    let mut flashed: BTreeMap<u64, MaterialHandle> = BTreeMap::new();
    for (e, v) in &mesh_rows {
        let amount = v["flash"].as_f64().unwrap_or(0.0) as f32;
        if amount > 0.0 {
            let mi = (v["material"].as_f64().unwrap_or(0.0) as u64 & 0xffff_ffff) as usize;
            if let Some(m) =
                super::feel::flash_material(rs, mi, amount, color(&v["flashColor"], 1.0))
            {
                flashed.insert(*e, m);
            }
        }
    }
    for (e, asset, parts) in model_parts {
        let m = h.world_of(e);
        if posed.contains(&e) && super::fx::posed_model(rs, e, asset, m, &parts, &mut keys) {
            continue;
        }
        for (k, &(mesh, material, local)) in parts.iter().enumerate() {
            rs.scene.add_instance(mesh, material, m * local);
            keys.push((e, k as u32));
        }
    }
    let scene = &mut rs.scene;
    // Meshes.
    {
        for (e, v) in mesh_rows {
            if !on(&v, "visible", true) {
                continue;
            }
            let index = |key: &str| (v[key].as_f64().unwrap_or(0.0) as u64 & 0xffff_ffff) as usize;
            let (Some(&g), Some(&m)) = (
                geometries.get(index("geometry")),
                materials.get(index("material")),
            ) else {
                continue;
            };
            let w = h.world_of(e);
            scene.add_instance(g, flashed.get(&e).copied().unwrap_or(m), w);
            keys.push((e, 0));
        }
    }
    // Lights: the first directional light, then point and spot lights in
    // entity order (deterministic); og-render culls them to the view and
    // keeps at most MAX_LIGHTS.
    let forward = |m: Mat4| m.transform_vector3(Vec3::NEG_Z).normalize_or_zero();
    let layouts = &rs.draw;
    if let Some(dl) = &layouts.directional
        && let Some((e, v)) = rows(world, dl).into_iter().next()
    {
        let m = h.world_of(e);
        let light = &mut scene.light;
        let dir = forward(m);
        light.direction_to_light = if dir == Vec3::ZERO { Vec3::Y } else { -dir };
        light.color = color(&v["color"], 1.0);
        light.intensity = f(&v, "intensity", 3.0);
        light.cast_shadows = on(&v, "castShadow", true);
        light.max_shadow_distance = f(&v, "shadowDistance", 200.0);
        scene.environment.sky.sun_direction = light.direction_to_light.normalize_or_zero();
    }
    let mut locals: Vec<(u64, og_render::Light)> = Vec::new();
    if let Some(pl) = &layouts.point {
        for (e, v) in rows(world, pl) {
            let m = h.world_of(e);
            locals.push((
                e,
                og_render::Light {
                    kind: og_render::LightKind::Point,
                    position: m.transform_point3(Vec3::ZERO),
                    color: color(&v["color"], 1.0),
                    intensity: f(&v, "intensity", 10.0),
                    range: f(&v, "distance", 10.0),
                    cast_shadows: on(&v, "castShadow", false),
                },
            ));
        }
    }
    if let Some(sl) = &layouts.spot {
        for (e, v) in rows(world, sl) {
            let m = h.world_of(e);
            locals.push((
                e,
                og_render::Light {
                    kind: og_render::LightKind::Spot {
                        direction: forward(m),
                        angle_degrees: f(&v, "angle", std::f32::consts::FRAC_PI_6).to_degrees(),
                        penumbra: f(&v, "penumbra", 0.0),
                    },
                    position: m.transform_point3(Vec3::ZERO),
                    color: color(&v["color"], 1.0),
                    intensity: f(&v, "intensity", 10.0),
                    range: f(&v, "distance", 10.0),
                    cast_shadows: on(&v, "castShadow", false),
                },
            ));
        }
    }
    locals.sort_by_key(|(e, _)| Entity::from_bits(*e).index());
    scene.lights = locals.into_iter().map(|(_, l)| l).collect();

    // The effects games declare: the post stack, shake and 2D lighting,
    // then the effect components.
    let tick = time.tick;
    super::fx::settings(world, rs);
    super::fx::components(world, rs, &mut h, tick);
    // Cloth meshes are instances too: keyed for per-object motion.
    keys.extend(super::atmosphere::extract(world, rs, &mut h));
    // Scattered copies are instances too, keyed by their terrain cell.
    keys.extend(super::terrain::extract(world, rs, &mut h));
    let vp = super::fx::viewport(rs);
    // The camera's shake, punch, kick and zones (on the frame only), then
    // og.Sprite rows, floating texts and tile layers.
    let aspect = match rs.backend.as_ref().map(|b| b.size()) {
        Some((w, ht)) if ht > 0 => w as f32 / ht as f32,
        _ if vp.1 > 0.0 => vp.0 / vp.1,
        _ => 16.0 / 9.0,
    };
    super::feel::camera_frame(world, rs, aspect);
    super::feel::extract(world, rs, &mut h);

    // Text, page by page; screen text into the overlay.
    let queued_texts: &[QueuedText] = if fresh { &rs.texts } else { &[] };
    for t in queued_texts {
        let Some(font) = rs.fonts.get(t.font) else {
            continue;
        };
        let scene = &mut rs.scene;
        for &page in &font.pages {
            let text = og_render::Text {
                font: page,
                text: t.text.clone(),
                position: t.position,
                size: t.size,
                rotation: t.rotation,
                color: t.color,
                layer: t.layer,
                clip: None,
            };
            if t.screen {
                let o = scene
                    .overlay
                    .get_or_insert_with(|| og_render::Overlay::new(vp.0, vp.1, 0.0));
                let clip = t.clip.map(|c| o.add_clip(c));
                o.texts.push(og_render::Text { clip, ..text });
            } else {
                if t.outline > 0.0 {
                    scene.text_outlines.push(og_render::TextOutline {
                        text: scene.texts.len() as u32,
                        width: t.outline,
                        color: t.outline_color,
                    });
                }
                scene.texts.push(text);
            }
        }
    }

    // Sprites and ribbons queued this tick.
    super::fx::queued(rs, fresh, vp);
    super::fx::motion(rs, &keys, tick);

    // The UI, over everything else, in the overlay.
    super::modules::emit(rs);
}

/// A cooked font as one `MsdfFont` per atlas page (Render's question 1: the
/// mapping lives here). Pages are uploaded linear, without mips.
pub(crate) fn add_font(scene: &mut RenderScene, font: &og_cook::runtime::CookedFont) -> FontPages {
    let meta = &font.meta;
    let line_height = meta.ascender - meta.descender + meta.line_gap;
    let mut pages = Vec::with_capacity(font.pages.len());
    for (p, data) in font.pages.iter().enumerate() {
        let atlas = scene.add_texture(og_render::TextureData {
            mips: og_render::Mips::None,
            color_space: og_render::ColorSpace::Linear,
            ..data.clone()
        });
        let glyphs = font
            .glyphs
            .iter()
            .filter_map(|g| {
                let codepoint = char::from_u32(g.codepoint)?;
                let own = g.page as usize == p;
                Some(og_render::Glyph {
                    codepoint,
                    advance: g.advance,
                    plane_bounds: if own {
                        [g.plane_left, g.plane_bottom, g.plane_right, g.plane_top]
                    } else {
                        [0.0; 4]
                    },
                    atlas_bounds: if own {
                        [
                            g.atlas_x as f32,
                            g.atlas_y as f32,
                            (g.atlas_x + g.atlas_w) as f32,
                            (g.atlas_y + g.atlas_h) as f32,
                        ]
                    } else {
                        [0.0; 4]
                    },
                })
            })
            .collect();
        pages.push(scene.add_font(og_render::MsdfFont {
            atlas,
            distance_range: meta.px_range,
            line_height,
            glyphs,
        }));
    }
    FontPages { pages }
}
