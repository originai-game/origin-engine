//! The scenes og-ui's reference frames are drawn from (`references/<tier>/`
//! sample-screen, text-glow, glass, widget-set and text-fields-2; and
//! text-mixed-runs, whose fonts og-tests-cooked's `tests/text_gfx.rs` cooks), as scenes ready to
//! render: shared by the native tests (`tests/gfx.rs`, `tests/kit_gfx.rs`,
//! through `support`) and the browser tiers (crates/og-tests-web includes
//! this file, packages/engine/tests/web/reference-gfx.ts renders them), so
//! every tier draws the same frame. The screens themselves are
//! `screens.rs`'s. Nothing here reads a file: the test font comes in as
//! bytes ([`FontFiles`]).

pub use super::screens::*;
use og_core::glam::{Vec2, Vec3};
use og_render::{Background, ColorSpace, Mips, RenderDesc, RenderScene, TextureData, Tonemap};
use og_ui::*;

/// Adds the MSDF test font to a scene; the UI font for it, with the font's
/// own ascender and descender.
pub fn add_font(scene: &mut RenderScene, files: &FontFiles) -> UiFont {
    let img = image::load_from_memory(files.png)
        .unwrap_or_else(|e| panic!("noto-sans-sc-msdf.png: {e}"))
        .to_rgba8();
    let (width, height) = img.dimensions();
    let atlas = scene.add_texture(TextureData {
        width,
        height,
        rgba: img.into_raw(),
        color_space: ColorSpace::Linear,
        mips: Mips::None,
    });
    let (font, metrics) = msdf_font(files, atlas);
    let handle = scene.add_font(font);
    UiFont {
        handle: Some(handle),
        metrics,
    }
}

/// A UI frame ready to render: the Ui that laid it out (its frame is what
/// the tests assert on), the scene with that frame emitted as its overlay
/// over the theme's background, and the renderer description at the
/// viewport's device size.
pub struct Shot {
    pub ui: Ui,
    pub scene: RenderScene,
    pub desc: RenderDesc,
}

/// Emits `ui`'s frame into `scene`'s overlay on the theme's background
/// (no tonemap) and sizes the renderer to `vp` in device pixels.
pub fn shot(ui: Ui, mut scene: RenderScene, vp: Viewport) -> Shot {
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    on_backdrop(&ui, &mut scene);
    let desc = RenderDesc {
        width: (vp.width * vp.dpr) as u32,
        height: (vp.height * vp.dpr) as u32,
        ..RenderDesc::default()
    };
    Shot { ui, scene, desc }
}

/// Puts `scene` on `ui`'s theme backdrop with no tonemap: what a UI frame
/// is drawn over, so its colours come out as the theme states them.
pub fn on_backdrop(ui: &Ui, scene: &mut RenderScene) {
    let bg = ui.theme().palette.backdrop.linear();
    scene.lighting.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.look.post.tonemap = Tonemap::None;
}

/// `references/<tier>/sample-screen.png`: the sample screen after its
/// session, 800×500 at 1.5×.
pub fn sample_screen_shot(font: &FontFiles) -> (Shot, Sample) {
    let vp = viewport();
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    let mut s = Sample::default();
    sample_session(&mut ui, &mut s);
    (shot(ui, scene, vp), s)
}

/// `references/<tier>/widget-set.png`: the widget set after a hit, an open
/// select moved to Hard by Tab, and a toast, 900×520 at 1.5×.
pub fn widget_set_shot(font: &FontFiles) -> Shot {
    let vp = Viewport {
        width: 900.0,
        height: 520.0,
        dpr: 1.5,
        ..Viewport::default()
    };
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    let mut s = Kit {
        amount: 3.0,
        checks: [true, false],
        pick: 1,
        ..Kit::default()
    };
    let step = |ui: &mut Ui, s: &mut Kit, input: InputFrame| {
        ui.begin(input, vp);
        kit_screen(ui, s);
        ui.end();
    };
    step(&mut ui, &mut s, InputFrame::default());
    for _ in 0..4 {
        step(&mut ui, &mut s, InputFrame::default());
    }
    // Open the select with a press: its menu floats over the panel and
    // takes focus; Tab moves it to Hard with the ring and its gap.
    let sel = ui.element("Difficulty").expect("select").clone();
    let (x, y) = sel.center();
    let p = Vec2::new(x, y);
    step(
        &mut ui,
        &mut s,
        InputFrame {
            pointer: p,
            pointer_inside: true,
            down: true,
            pressed: true,
            ..InputFrame::default()
        },
    );
    step(
        &mut ui,
        &mut s,
        InputFrame {
            pointer: p,
            pointer_inside: true,
            released: true,
            ..InputFrame::default()
        },
    );
    step(
        &mut ui,
        &mut s,
        InputFrame {
            keys: vec![UiKey::Tab],
            ..InputFrame::hover(p.x, p.y)
        },
    );
    ui.toast(
        "Achievement unlocked",
        ToastDesc {
            tone: Tone::Success,
            ..ToastDesc::default()
        },
    );
    for _ in 0..20 {
        step(&mut ui, &mut s, InputFrame::hover(p.x, p.y));
    }
    shot(ui, scene, vp)
}

/// `references/<tier>/text-fields-2.png`, 520×330 at 1.5×, two UIs (each
/// with its own focus) in one frame: in the first, "one two three" typed a
/// character a tick into the second field and undone back to "one two ",
/// then "select" in the first field selected by a double click (its caret
/// at the word's end); in the second, a press on the area's first line
/// dragged 30 px below it and held there 6 ticks, the area scrolled about
/// three lines (a line cut part way at its top) and its selection over
/// every line it passed, the press still held.
pub fn text_fields_shot(font: &FontFiles) -> Shot {
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let (ui, area) = text_fields_uis(&font);
    let vp = text_fields_viewport();
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    // The area's UI under the fields' (each band leaves the other's empty).
    area.frame().emit(&EmitDesc::default(), &mut overlay);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    on_backdrop(&ui, &mut scene);
    let desc = RenderDesc {
        width: (vp.width * vp.dpr) as u32,
        height: (vp.height * vp.dpr) as u32,
        ..RenderDesc::default()
    };
    Shot { ui, scene, desc }
}

/// A dark card of display-size labels, with or without a style's text
/// glow, as a scene ready to render.
pub fn glow_scene(font: &FontFiles, glow: bool, labels: usize) -> (RenderScene, RenderDesc) {
    let vp = Viewport {
        dpr: 1.0,
        ..Viewport::new(480.0, 200.0f32.max(16.0 + 40.0 * labels as f32))
    };
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    let mut theme = *ui.theme();
    if glow {
        theme.art.text_glows[0] = Some(og_ui::style_file::Glow {
            size: 2.0,
            ink: og_ui::style_file::Ink::Primary,
            alpha: 0.8,
        });
    }
    ui.set_theme(theme);
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp);
        let t = *ui.theme();
        ui.column(
            "Card",
            Element {
                width: Size::grow(),
                height: Size::grow(),
                padding: Insets::all(t.space.lg),
                background: Some(t.palette.backdrop),
                ..Element::default()
            },
            |ui| {
                for k in 0..labels {
                    ui.label(
                        ui_idi("Title", k as u32),
                        "Wall 42",
                        LabelDesc {
                            // A shader fixture, independent of the style's
                            // responsive ramp. Tight tracking overlaps quads.
                            step: Some(TypeStep {
                                size: 34.0,
                                line_height: 38.0,
                                tracking_em: -0.1,
                                cjk_line_height: 44.0,
                                cjk_tracking_em: 0.0,
                                features: og_ui::shape::Features::NONE,
                                weight: 400,
                                uppercase: false,
                            }),
                            ..LabelDesc::default()
                        },
                    );
                }
            },
        );
        ui.end();
    }
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, vp.dpr);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    on_backdrop(&ui, &mut scene);
    let desc = RenderDesc {
        width: vp.width as u32,
        height: vp.height as u32,
        ..RenderDesc::default()
    };
    (scene, desc)
}

/// A scene of fine vertical stripes (4 px black, 4 px white) under an
/// overlay holding `glass` panels, 320×200 at 1×.
pub fn stripes_scene(glass: &[og_render::Glass]) -> (RenderScene, RenderDesc) {
    let (w, h) = (320u32, 200u32);
    let mut scene = RenderScene::new();
    scene.frame.camera =
        og_render::Camera::orthographic_2d(Vec2::new(w as f32 / 2.0, h as f32 / 2.0), h as f32);
    for k in 0..(w / 8) {
        scene.sprite.sprites.push(og_render::Sprite {
            position: Vec3::new(k as f32 * 8.0 + 2.0, h as f32 / 2.0, 0.0),
            size: Vec2::new(4.0, h as f32),
            color: [1.0, 1.0, 1.0, 1.0],
            ..og_render::Sprite::default()
        });
    }
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let mut overlay = og_render::Overlay::new(w as f32, h as f32, 1.0);
    overlay.glass.extend_from_slice(glass);
    scene.ui.overlay = Some(overlay);
    let desc = RenderDesc {
        width: w,
        height: h,
        ..RenderDesc::default()
    };
    (scene, desc)
}

/// The glass panel of `references/<tier>/glass.png`: a 160×120 rounded
/// rectangle at (80, 40) over the stripes, blur 16, no tint.
pub fn glass_panel() -> og_render::Glass {
    og_render::Glass {
        position: Vec2::new(80.0, 40.0),
        size: Vec2::new(160.0, 120.0),
        radius: 24.0,
        blur: 16.0,
        tint: [0.0, 0.0, 0.0, 0.0],
        alpha: 1.0,
        layer: 0,
        clip: None,
    }
}

/// Overlay content around the reference glass panel: an orange bar on the
/// overlay layer below it, crossing the panel (y 92..108, x 40..280), and
/// a small cyan square on the layer above it, inside the panel at
/// (190, 60), 16 px. The panel blurs the bar where it covers it and
/// leaves the square sharp.
pub fn glass_overlay_content() -> Vec<og_render::Sprite> {
    let solid = |x: f32, y: f32, w: f32, h: f32, color: [f32; 4], layer: i32| og_render::Sprite {
        texture: None,
        position: Vec3::new(x, y, 0.0),
        size: Vec2::new(w, h),
        anchor: Vec2::new(0.0, 0.0),
        color,
        layer,
        ..og_render::Sprite::default()
    };
    vec![
        solid(40.0, 92.0, 240.0, 16.0, [1.0, 0.45, 0.0, 1.0], -1),
        solid(190.0, 60.0, 16.0, 16.0, [0.0, 0.8, 1.0, 1.0], 1),
    ]
}

/// The scene of `references/<tier>/glass.png`: the stripes, the panel
/// (`glass_panel`) and the overlay content below and above it
/// (`glass_overlay_content`).
pub fn glass_reference_scene() -> (RenderScene, RenderDesc) {
    let (mut scene, desc) = stripes_scene(&[glass_panel()]);
    if let Some(o) = scene.ui.overlay.as_mut() {
        o.sprites.extend(glass_overlay_content());
    }
    (scene, desc)
}

/// The colour vision frame (docs/ui.md §3.2) at `width`×`height`: five
/// swatches a dichromat confuses in pairs, over a scene colour, so the
/// filter's test sees both the overlay and the scene. Not a reference: its
/// tests (`tests/color_vision.rs`, packages/engine/tests/web/color-vision.ts)
/// compare each filtered frame with the CPU filter of the unfiltered one.
pub fn color_vision_scene(font: &FontFiles, width: u32, height: u32) -> (RenderScene, RenderDesc) {
    let doc = r#"<meta name="component" content="chips">
<row id="Root"><panel id="R"/><panel id="G"/><panel id="B"/><panel id="Y"/><panel id="M"/></row>
<style>
#Root { padding: 24px; gap: 12px; flex-wrap: wrap; }
panel { width: 120px; height: 120px; }
#R { background: #d03028; } #G { background: #2fa84f; } #B { background: #2f6fe0; }
#Y { background: #e8c83a; } #M { background: #a03ad0; }
</style>"#;
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    ui.define_document(UiDocument::parse("ui/chips.svelte", doc).expect("chips parse"));
    let vp = Viewport::new(width as f32, height as f32);
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp);
        ui.component("T", "chips", &mut serde_json::json!({}), &bind::NoEnv);
        ui.end();
    }
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    scene.lighting.environment.background = Background::Color(Vec3::new(0.6, 0.15, 0.05));
    scene.look.post.tonemap = Tonemap::None;
    (
        scene,
        RenderDesc {
            width,
            height,
            ..Default::default()
        },
    )
}

/// `user-select-<width>`: a drag from the wrapped paragraph's second line
/// into the next paragraph, its selection under the text, drawn by the
/// host's own og-ui (wasm in Chrome, native on Metal). Panics where the
/// drag did not select what it must.
pub fn user_select_scene(font: &FontFiles, width: u32, height: u32) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    ui.define_document(UiDocument::parse("ui/sel.svelte", USER_SELECT_DOC).expect("sel parses"));
    let vp = Viewport::new(width as f32, height as f32);
    let step = |ui: &mut Ui, p: Vec2, down: bool, pressed: bool, released: bool| {
        ui.begin(
            InputFrame {
                pointer: p,
                pointer_inside: true,
                down,
                pressed,
                released,
                ..InputFrame::default()
            },
            vp,
        );
        ui.component("T", "sel", &mut serde_json::json!({}), &bind::NoEnv);
        ui.end();
    };
    step(&mut ui, Vec2::ZERO, false, false, false);
    step(&mut ui, Vec2::ZERO, false, false, false);
    let at = |ui: &Ui, id: &str, fx: f32, fy: f32| {
        let e = ui.element(id).expect(id);
        Vec2::new(e.x + e.width * fx, e.y + e.height * fy)
    };
    let (from, to) = (at(&ui, "Wrap", 0.3, 0.3), at(&ui, "Second", 0.5, 0.5));
    step(&mut ui, from, true, true, false);
    for k in 1..=4 {
        step(
            &mut ui,
            from + (to - from) * (k as f32 / 4.0),
            true,
            false,
            false,
        );
    }
    step(&mut ui, to, false, false, true);
    let selected = ui.probe_extra()["selection"]["text"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(
        selected.contains("jumps over the lazy dog\nSecond"),
        "the drag selected {selected:?}"
    );
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    on_backdrop(&ui, &mut scene);
    (
        scene,
        RenderDesc {
            width,
            height,
            ..Default::default()
        },
    )
}

/// `text-path-<width>`: [`TEXT_PATH_DOC`] at `width`×`height`.
pub fn text_path_scene(font: &FontFiles, width: u32, height: u32) -> (RenderScene, RenderDesc) {
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    ui.define_document(UiDocument::parse("ui/tp.svelte", TEXT_PATH_DOC).expect("tp parses"));
    let vp = Viewport::new(width as f32, height as f32);
    for _ in 0..2 {
        ui.begin(InputFrame::default(), vp);
        ui.component("T", "tp", &mut serde_json::json!({}), &bind::NoEnv);
        ui.end();
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let mut overlay = og_render::Overlay::new(vp.width, vp.height, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    on_backdrop(&ui, &mut scene);
    (
        scene,
        RenderDesc {
            width,
            height,
            ..Default::default()
        },
    )
}

/// The reference frames, by their file name under `references/<tier>/`.
pub const REFERENCE_SCENES: [&str; 9] = [
    "sample-screen",
    "text-glow",
    "glass",
    "widget-set",
    "text-fields-2",
    "user-select-1280",
    "user-select-390",
    "text-path-1280",
    "text-path-390",
];

/// The scene and renderer description of reference frame `name`.
pub fn reference_scene(name: &str, font: &FontFiles) -> Result<(RenderScene, RenderDesc), String> {
    Ok(match name {
        "sample-screen" => {
            let (s, _) = sample_screen_shot(font);
            (s.scene, s.desc)
        }
        "text-glow" => glow_scene(font, true, 1),
        "glass" => glass_reference_scene(),
        "widget-set" => {
            let s = widget_set_shot(font);
            (s.scene, s.desc)
        }
        "text-fields-2" => {
            let s = text_fields_shot(font);
            (s.scene, s.desc)
        }
        "color-vision-1280" => color_vision_scene(font, 1280, 720),
        "color-vision-390" => color_vision_scene(font, 390, 844),
        "user-select-1280" => user_select_scene(font, 1280, 720),
        "user-select-390" => user_select_scene(font, 390, 844),
        "text-path-1280" => text_path_scene(font, 1280, 720),
        "text-path-390" => text_path_scene(font, 390, 844),
        _ => {
            return Err(format!(
                "no og-ui reference scene {name} (it knows {})",
                REFERENCE_SCENES.join(", ")
            ));
        }
    })
}
