//! og-ui's materials frame (`references/<tier>/materials.png`), rendered
//! headless and compared under og-tests' tolerance (`OG_UI_CAPTURE=1`
//! records this tier's): a card in the dark default style, a row of the
//! UI kit's iconoir icons cooked by og-cook's icon cook (a distance-field
//! set drawn as glyphs, at 24 and 48 px, in two inks) and one inline in
//! rich text, two pictures keeping 16:9 across growing columns, and a list
//! two touches scrolled by 60 px.
//!
//! It needs the cook (native only), so the browser tiers do not run it.
//!
//! Ways it could fail: an icon missing or drawn as a box (a missing glyph,
//! asserted 0), the icons blurred or cut (the field's range or the em
//! square wrong), the picture out of proportion, the two-finger scroll
//! not applied.

use crate::support;

use og_cook::runtime::{Bundle, CookedFont};
use og_cook::{CookOptions, cook_project};
use og_core::glam::{Vec2, Vec3};
use og_render::{
    Background, ColorSpace, Glyph, Mips, MsdfFont, RenderDesc, RenderScene, TextureData, Tonemap,
};
use og_ui::*;
use og_ui_next::fonts::Fonts;
use og_ui_next::icons::IconAtlas;
use std::path::PathBuf;
use support::*;

/// The kit's iconoir set and body font cooked into a scratch project.
fn cooked() -> (CookedFont, CookedFont) {
    let root = og_tests::harness::out_dir("og-ui-materials");
    let _ = std::fs::remove_dir_all(&root);
    let kit = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/ui-text/assets");
    let icons = root.join("assets/icons/iconoir");
    std::fs::create_dir_all(&icons).expect("icons dir");
    for e in std::fs::read_dir(kit.join("icons/iconoir")).expect("iconoir") {
        let e = e.expect("entry");
        std::fs::copy(e.path(), icons.join(e.file_name())).expect("copy svg");
    }
    std::fs::copy(
        kit.join("icons/iconoir.ogt"),
        root.join("assets/icons/iconoir.ogt"),
    )
    .expect("copy set");
    let font = "NotoSansSC-Regular-subset.ttf";
    std::fs::create_dir_all(root.join("assets/fonts")).expect("fonts dir");
    std::fs::copy(
        kit.join("fonts").join(font),
        root.join("assets/fonts").join(font),
    )
    .expect("copy font");
    std::fs::write(
        root.join("assets/fonts").join(format!("{font}.ogt")),
        "type import\nversion 1\n---\nfont {\n  charset latin\n  px_range 4\n}\n",
    )
    .expect("settings");
    let report = cook_project(&root, &CookOptions::default()).expect("cook");
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).expect("bundle");
    (
        bundle
            .load_font("icons/iconoir.ogt")
            .expect("the icon set loads"),
        bundle
            .load_font(&format!("fonts/{font}"))
            .expect("the font loads"),
    )
}

/// A cooked font (one page) in the scene, as og-runtime adds it.
fn add(scene: &mut RenderScene, f: &CookedFont) -> UiFont {
    assert_eq!(f.pages.len(), 1, "{}: one page", f.id);
    let atlas = scene.add_texture(TextureData {
        mips: Mips::None,
        color_space: ColorSpace::Linear,
        ..f.pages[0].clone()
    });
    let m = &f.meta;
    let font = MsdfFont {
        atlas,
        distance_range: m.px_range,
        foreground: false,
        silhouette: false,
        line_height: m.ascender - m.descender + m.line_gap,
        glyphs: f
            .glyphs
            .iter()
            .filter_map(|g| {
                Some(Glyph {
                    key: g.key()?,
                    advance: g.advance,
                    plane_bounds: [g.plane_left, g.plane_bottom, g.plane_right, g.plane_top],
                    atlas_bounds: [
                        g.atlas_x as f32,
                        g.atlas_y as f32,
                        (g.atlas_x + g.atlas_w) as f32,
                        (g.atlas_y + g.atlas_h) as f32,
                    ],
                })
            })
            .collect(),
        kerning: Vec::new(),
    };
    let metrics = FontMetrics::from_msdf(&font);
    UiFont {
        handle: Some(scene.add_font(font)),
        metrics,
    }
}

/// A 64×36 picture: a vertical dusk gradient with a sun disc.
fn picture(scene: &mut RenderScene) -> og_render::TextureHandle {
    let (w, h) = (64u32, 36u32);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let t = y as f32 / (h - 1) as f32;
            let (dx, dy) = (x as f32 - 44.0, y as f32 - 14.0);
            let sun = dx * dx + dy * dy < 36.0;
            let c = if sun {
                [255, 200, 120]
            } else {
                [
                    (40.0 + 200.0 * t) as u8,
                    (30.0 + 90.0 * t) as u8,
                    (90.0 - 30.0 * t) as u8,
                ]
            };
            rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    scene.add_texture(TextureData {
        width: w,
        height: h,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    })
}

const ICONS: [&str; 16] = [
    "heart",
    "star",
    "coins",
    "trophy",
    "crown",
    "gift",
    "shield",
    "flash",
    "fire-flame",
    "potion",
    "magic-wand",
    "map",
    "compass",
    "clock",
    "gamepad",
    "settings",
];

fn vp() -> Viewport {
    Viewport {
        width: 640.0,
        height: 400.0,
        dpr: 1.5,
        ..Viewport::default()
    }
}

/// The card: a row of [`ICONS`] (the first 48 px, every fifth in the
/// primary ink, the rest muted), a line with two inline icons, two 16:9
/// pictures in growing columns and a scrolled list of ten small buttons.
/// `src={1}` is the picture ([`Pictures`]).
const CARD: &str = r#"<meta name="component" content="materials">
<panel id="Card">
  <row id="Icons">
    {#each $icons as icon (icon.id)}
      <og-icon id="Icon" icon={icon.name} iconSize={icon.size} class:accent={icon.accent}/>
    {/each}
  </row>
  <text id="Rich">Collect <og-icon name="coins"/> coins, then claim the <og-icon name="trophy"/> trophy.</text>
  <row id="Pictures">
    <column id="Left"><img id="Picture" src={1} alt=""/></column>
    <column id="Middle"><img id="Map" src={1} alt=""/></column>
    <scroll id="Quests">
      {#each $quests as quest (quest.id)}
        <button id="Quest" class="small">{quest.label}</button>
      {/each}
    </scroll>
  </row>
</panel>
<style>
#Card { width: stretch; height: stretch; }
#Icons { gap: var(--space-sm); align-items: center; }
#Icons og-icon { color: var(--ink-muted); }
#Icons og-icon.accent { color: var(--primary); }
#Pictures { width: stretch; gap: var(--space-md); }
#Left, #Middle { flex: 1 1 0; }
#Picture, #Map { width: stretch; aspect-ratio: 16 / 9; }
#Quests { width: 140px; height: 150px; gap: var(--space-xs); }
.small { width: stretch; }
.small, .small::part(base) { height: var(--control-sm); }
.small::part(base) { padding: 0 var(--space-md); }
</style>"#;

/// The picture every `src` of [`CARD`] names.
struct Pictures(og_render::TextureHandle);

impl bind::Env for Pictures {
    fn texture(&self, _: f64) -> Option<og_render::TextureHandle> {
        Some(self.0)
    }
}

/// Mounts [`CARD`] in `ui` (built with [`Pictures`]).
fn mount(ui: &mut Ui) {
    use serde_json::json;
    let icons: Vec<_> = ICONS
        .iter()
        .enumerate()
        .map(|(i, name)| {
            json!({
                "id": format!("i{i}"),
                "name": name,
                "size": if i == 0 { 48 } else { 24 },
                "accent": i % 5 == 0,
            })
        })
        .collect();
    let quests: Vec<_> = (0..10)
        .map(|k| json!({ "id": format!("q{k}"), "label": format!("Quest {}", k + 1) }))
        .collect();
    mount_screen_document(
        ui,
        "materials",
        CARD,
        &[("icons", json!(icons)), ("quests", json!(quests))],
    );
}

/// Two touches at `a` and `b` (pointer 0 is og.Input's pointer).
fn touches(p: &mut og_input::Pointers, at: [Vec2; 2]) -> InputFrame {
    p.end_tick();
    for (id, pos) in at.into_iter().enumerate() {
        let action = if p.down(id as u32) {
            og_input::PointerAction::Move
        } else {
            og_input::PointerAction::Down
        };
        p.apply(&og_input::InjectedInput::Pointer {
            action,
            x: pos.x,
            y: pos.y,
            button: og_input::PointerButton::Left,
            id: id as u32,
            movement_x: 0.0,
            movement_y: 0.0,
        })
        .expect("a pointer id under five");
    }
    InputFrame {
        pointer: p.position(0),
        pointer_inside: true,
        down: true,
        pressed: p.is_pressed(0),
        pointers: *p,
        ..InputFrame::default()
    }
}

#[test]
#[ignore = "requires GPU"]
fn the_materials_frame_matches_its_reference() {
    let (set, body) = cooked();
    // The cook names exactly the icons the set declares (each `icon "<name>" ["<svg>"]` line), no more and no fewer:
    // the set grows with the kit, so the count is read from it rather than fixed here.
    let declared: Vec<String> = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/ui-text/assets/icons/iconoir.ogt"),
    )
    .expect("the kit's set")
    .lines()
    .filter_map(|l| {
        l.strip_prefix("icon \"")?
            .split('"')
            .next()
            .map(str::to_owned)
    })
    .collect();
    let mut cooked_names = set.names.clone();
    cooked_names.sort();
    let mut want = declared;
    want.sort();
    assert_eq!(
        cooked_names, want,
        "the cooked set names every icon the set declares"
    );
    // Every name the kit's screens draw (its old PNG grid's 32) is in the set, and the phone chrome's grid.
    assert!(
        set.names.iter().any(|n| n == "view-grid"),
        "view-grid is in the set"
    );
    for name in [
        "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield", "potion", "gem",
        "key", "map", "bolt", "clock", "trophy", "home", "plus", "close", "check", "play", "back",
        "bag", "bomb", "crown", "flag", "apple", "leaf", "fire", "drop", "skull", "music",
        "person",
    ] {
        assert!(set.names.iter().any(|n| n == name), "{name} is in the set");
    }
    let mut scene = RenderScene::new();
    let icons = add(&mut scene, &set);
    let font = add(&mut scene, &body);
    let tex = picture(&mut scene);
    let mut ui = Ui::new(UiDesc {
        fonts: Fonts {
            body: font,
            ..Fonts::default()
        },
        ..UiDesc::default()
    });
    ui.set_icons(IconAtlas {
        font: Some(icons),
        names: set.names.clone(),
        ..IconAtlas::default()
    });
    mount(&mut ui);
    let step = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, vp());
        ui.build_retained_documents(&Pictures(tex))
            .expect("the card builds");
        ui.end();
    };
    for _ in 0..30 {
        step(&mut ui, InputFrame::default());
    }
    let center = |ui: &Ui, path: &str| {
        let (x, y) = ui.element(path).expect(path).center();
        Vec2::new(x, y)
    };
    // Two fingers up the list by 60.
    let list = center(&ui, "Pictures/Quests");
    let mut q = og_input::Pointers::default();
    for dy in [0.0, 30.0, 60.0] {
        step(
            &mut ui,
            touches(
                &mut q,
                [
                    list + Vec2::new(-20.0, 40.0 - dy),
                    list + Vec2::new(20.0, 40.0 - dy),
                ],
            ),
        );
    }
    for _ in 0..3 {
        step(&mut ui, InputFrame::default());
    }
    let pic = ui.element("Pictures/Left/Picture").expect("picture");
    assert!(
        (pic.width / pic.height - 16.0 / 9.0).abs() < 0.02,
        "the picture keeps 16:9 ({}x{})",
        pic.width,
        pic.height
    );
    // The frame.
    let v = vp();
    let mut overlay = og_render::Overlay::new(v.width, v.height, v.dpr);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.lighting.environment.background = Background::Color(Vec3::new(bg[0], bg[1], bg[2]));
    scene.look.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: (v.width * v.dpr) as u32,
        height: (v.height * v.dpr) as u32,
        ..RenderDesc::default()
    };
    let frame = render_frame(&scene, &desc);
    assert_eq!(
        frame.stats.missing_glyphs, 0,
        "every icon and character is in its font"
    );
    matches_reference("materials", &frame.image, &frame.info, Against::OwnTier);
}
