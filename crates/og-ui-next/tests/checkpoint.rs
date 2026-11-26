//! Determinism and world-anchor contracts for `og-ui-next`:
//! 1. `Tree::checkpoint` / `Tree::restore` captures and restores focus,
//!    keyboard modality, scroll offsets, `<Input>` text/caret, UI sound PRNG,
//!    and virtual cursor state without emitting spurious JS events, and
//!    survives JSON serialization cleanly.
//! 2. World anchors honour `maxDistance` depth culling, `scaleAt` / `scaleMin`
//!    / `scaleMax` distance scaling, and `lod` (`data-lod-{near,mid,far}`).

use og_draw::glyphs::Atlas;
use og_ui_next::icons::IconAtlas;
use og_ui_next::tree::{
    Cue, CursorPad, CursorPointer, Sighting, SoundTable, Tree, TreeCheckpoint, VirtualCursorDesc,
};
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Default)]
struct Batch(Vec<u8>, u32);

impl Batch {
    fn op(&mut self, code: u8) -> &mut Self {
        self.1 += 1;
        self.0.push(code);
        self
    }
    fn u32(&mut self, v: u32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn u8(&mut self, v: u8) -> &mut Self {
        self.0.push(v);
        self
    }
    fn str(&mut self, s: &str) -> &mut Self {
        self.u32(s.len() as u32);
        self.0.extend_from_slice(s.as_bytes());
        self
    }
    fn bytes(&self) -> Vec<u8> {
        let mut out = self.1.to_le_bytes().to_vec();
        out.extend_from_slice(&self.0);
        out
    }
}

fn pack_tree() -> Tree {
    let mut t = Tree::new();
    t.resize(800.0, 600.0);
    let pack = og_ui_next::pack::Pack::new(&[og_ui_next::pack::Layer::default_pack()]);
    let theme = pack.theme_json(og_ui_next::pack::Scheme::Dark);
    let classes =
        std::cell::RefCell::new(og_ui_next::classes::Classes::new(Some(&theme)).unwrap().0);
    let env = og_ui_next::classes::Env {
        window: [800.0, 600.0],
        dark: true,
        ..Default::default()
    };
    t.set_resolver(Box::new(move |class, style, states, data| {
        classes.borrow_mut().record_data(
            class,
            style,
            &og_ui_next::classes::Env { states, ..env },
            data,
        )
    }));
    t
}

fn face() -> og_text::fonts::Collection {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/latin/NotoSans-Latin.ttf"
    );
    let mut fonts = og_text::fonts::Collection::new();
    let face = fonts
        .add(std::sync::Arc::new(std::fs::read(path).unwrap()))
        .unwrap();
    let family = format!("\"{}\"", fonts.family_name(face).unwrap());
    fonts.set_generic(og_text::fonts::GenericFamily::SansSerif, &family);
    fonts.set_fallback(None, &[face]);
    fonts
}

#[test]
fn tree_checkpoint_restores_focus_scroll_input_sound_rng_and_vcursor() {
    let mut tree = pack_tree();
    let mut fonts = face();
    let mut atlas = Atlas::default();
    let icons = IconAtlas::default();

    let mut sounds: SoundTable = BTreeMap::new();
    let mut root_sounds = BTreeMap::new();
    root_sounds.insert(
        "press".to_string(),
        Cue {
            clips: vec![101, 102, 103, 104],
            volume: [0.5, 1.0],
            pitch: [0.8, 1.2],
        },
    );
    sounds.insert(String::new(), root_sounds);
    tree.set_sounds(Rc::new(sounds));

    // Tags: 0=View, 1=Text, 3=Scroll, 4=Input.
    let mut b = Batch::default();
    // Root column (id 1)
    b.op(1).u32(1).u8(0);
    b.op(4).u32(1).str("flex-col size-full");
    // Button 1 (id 2)
    b.op(1).u32(2).u8(0);
    b.op(4).u32(2).str("w-32 h-12 bg-surface");
    b.op(7).u32(2).u8(0).u8(1); // listen Press
    b.op(8).u32(1).u32(2).u32(u32::MAX);
    // Input (id 3)
    b.op(1).u32(3).u8(4);
    b.op(4).u32(3).str("w-48 h-12");
    b.op(7).u32(3).u8(9).u8(1); // listen Change
    b.op(8).u32(1).u32(3).u32(u32::MAX);
    // Scroll (id 4) with tall child (id 5)
    b.op(1).u32(4).u8(3);
    b.op(4).u32(4).str("flex-col w-64 h-32 overflow-y-auto");
    b.op(1).u32(5).u8(0);
    b.op(4).u32(5).str("shrink-0 w-64 h-96 bg-tile");
    b.op(8).u32(4).u32(5).u32(u32::MAX);
    b.op(8).u32(1).u32(4).u32(u32::MAX);
    b.op(8).u32(0).u32(1).u32(u32::MAX);
    assert!(tree.apply(&b.bytes()).is_ok());

    tree.set_time(0.1, false);
    tree.present(&mut fonts, &mut atlas, &icons);

    // Move focus to Button (id 2) then Input (id 3) via Tab, type text, scroll,
    // advance virtual cursor, and trigger one press sound to advance sound_rng.
    assert!(tree.key("Tab", false, false, false, false));
    assert_eq!(tree.focused_id(), Some(2));
    // Enter presses focused button and advances sound_rng.
    assert!(tree.key("Enter", false, false, false, false));
    let _ = tree.take_sounds();

    assert!(tree.key("Tab", false, false, false, false));
    assert_eq!(tree.focused_id(), Some(3));
    assert!(tree.type_text("hero"));

    // Scroll inside the scroll container (starts around y = 96..224).
    tree.wheel(40.0, 140.0, 0.0, 64.0);

    // Step virtual cursor via pad stick.
    let desc = Some(VirtualCursorDesc::default());
    let mut pad = CursorPad {
        stick: [0.8, 0.4],
        ..Default::default()
    };
    let mut ptr = CursorPointer::default();
    tree.virtual_cursor(desc, &mut pad, &mut ptr, 60);
    tree.set_time(0.25, false);
    tree.present(&mut fonts, &mut atlas, &icons);

    // Capture checkpoint and verify JSON round-trip.
    let cp = tree.checkpoint();
    assert_eq!(cp.focus, Some(3));
    assert!(cp.focus_keyboard);
    assert!(!cp.scrolls.is_empty());
    assert_eq!(cp.inputs.len(), 1);
    assert_eq!(cp.inputs[0].value, "hero");
    assert_eq!(cp.inputs[0].caret, 4);
    assert_ne!(cp.sound_rng, 0);
    assert!(cp.vcursor.shown);

    let json = serde_json::to_string(&cp).expect("serialize checkpoint");
    let roundtrip: TreeCheckpoint = serde_json::from_str(&json).expect("deserialize checkpoint");
    assert_eq!(roundtrip, cp);

    // Record the exact next press sound after checkpoint in the original run.
    let _ = tree.take_sounds();
    assert!(tree.key("Tab", true, false, false, false)); // Shift+Tab back to Button (id 2)
    assert!(tree.key("Enter", false, false, false, false));
    let expected_sounds = tree.take_sounds();
    assert_eq!(expected_sounds.len(), 1);

    // Mutate everything further.
    assert!(tree.key("Tab", false, false, false, false));
    assert!(tree.type_text("-mutated"));
    tree.wheel(40.0, 140.0, 0.0, 96.0);
    let mut pad2 = CursorPad {
        stick: [-0.9, 0.9],
        ..Default::default()
    };
    tree.virtual_cursor(desc, &mut pad2, &mut ptr, 60);
    tree.set_time(1.0, false);
    tree.present(&mut fonts, &mut atlas, &icons);
    assert_ne!(tree.checkpoint(), cp);

    // Drain events, then restore checkpoint: must emit zero spurious events and
    // reproduce the exact checkpoint and subsequent sound pick.
    tree.events.clear();
    let _ = tree.take_sounds();
    tree.restore(&roundtrip);
    assert!(tree.events.is_empty(), "restore must not emit UI events");
    assert_eq!(tree.checkpoint(), cp);

    assert!(tree.key("Tab", true, false, false, false));
    assert!(tree.key("Enter", false, false, false, false));
    let replayed_sounds = tree.take_sounds();
    assert_eq!(replayed_sounds, expected_sounds);
}

#[test]
fn world_anchor_honours_max_distance_scale_at_and_lod() {
    let mut tree = pack_tree();
    let mut fonts = face();
    let mut atlas = Atlas::default();
    let icons = IconAtlas::default();

    let mut b = Batch::default();
    b.op(1).u32(1).u8(0);
    b.op(4)
        .u32(1)
        .str("w-20 h-10 bg-surface data-lod-near:w-24 data-lod-mid:w-16 data-lod-far:w-8");
    b.op(6).u32(1).str("anchor").str("1 2 3");
    b.op(6).u32(1).str("maxDistance").str("50");
    b.op(6).u32(1).str("scaleAt").str("10");
    b.op(6).u32(1).str("scaleMin").str("0.5");
    b.op(6).u32(1).str("scaleMax").str("2");
    b.op(6).u32(1).str("lod").str("10,30");
    b.op(8).u32(0).u32(1).u32(u32::MAX);
    assert!(tree.apply(&b.bytes()).is_ok());

    let mut place_at_depth = |tree: &mut Tree, depth: f32, time: f64| {
        tree.set_time(time, false);
        tree.place_anchors(|_, _, _| {
            Some(Sighting {
                at: [400.0, 300.0],
                depth,
                behind: false,
                occluded: false,
            })
        });
        tree.present(&mut fonts, &mut atlas, &icons);
    };

    // Depth 5 -> near (w-24 = 96px), scale = clamp(10/5, 0.5, 2) = 2.0.
    place_at_depth(&mut tree, 5.0, 0.1);
    assert_eq!(tree.drawn.quads.len(), 1);
    let q_near = tree.drawn.quads[0];
    assert!(
        (q_near.rect[2] - 96.0).abs() < 0.5,
        "near width: {:?}",
        q_near.rect
    );
    let xf_near = tree.drawn.transforms[q_near.xf as usize];
    assert!((xf_near[0] - 2.0).abs() < 1e-3, "near scale x: {xf_near:?}");
    assert!((xf_near[3] - 2.0).abs() < 1e-3, "near scale y: {xf_near:?}");

    // Depth 20 -> mid (w-16 = 64px), scale = clamp(10/20, 0.5, 2) = 0.5.
    place_at_depth(&mut tree, 20.0, 0.2);
    assert_eq!(tree.drawn.quads.len(), 1);
    let q_mid = tree.drawn.quads[0];
    assert!(
        (q_mid.rect[2] - 64.0).abs() < 0.5,
        "mid width: {:?}",
        q_mid.rect
    );
    let xf_mid = tree.drawn.transforms[q_mid.xf as usize];
    assert!((xf_mid[0] - 0.5).abs() < 1e-3, "mid scale x: {xf_mid:?}");

    // Depth 40 -> far (w-8 = 32px), scale clamped at scaleMin = 0.5.
    place_at_depth(&mut tree, 40.0, 0.3);
    assert_eq!(tree.drawn.quads.len(), 1);
    let q_far = tree.drawn.quads[0];
    assert!(
        (q_far.rect[2] - 32.0).abs() < 0.5,
        "far width: {:?}",
        q_far.rect
    );

    // Depth 60 -> past maxDistance (50), hidden completely.
    place_at_depth(&mut tree, 60.0, 0.4);
    assert!(
        tree.drawn.quads.is_empty(),
        "anchor past maxDistance must paint no quads"
    );
}

#[test]
fn subtree_materials_keep_nested_parameters_filters_and_can_be_removed() {
    use og_draw::paint::{PaintEffect, PaintMaterial};
    let mut tree = pack_tree();
    tree.set_materials(Box::new(|text| {
        Ok(PaintMaterial {
            shader: og_draw::ui::UiMaterialHandle(0),
            params: [[
                text.parse::<f32>().map_err(|e| e.to_string())?,
                0.0,
                0.0,
                0.0,
            ]; 4],
        })
    }));
    let mut b = Batch::default();
    b.op(1).u32(1).u8(0);
    b.op(4)
        .u32(1)
        .str("w-64 h-32 bg-surface opacity-50 blur-sm");
    b.op(6).u32(1).str("material").str("0.2");
    b.op(1).u32(2).u8(0);
    b.op(4).u32(2).str("w-16 h-8 bg-tile");
    b.op(6).u32(2).str("material").str("0.8");
    b.op(8).u32(1).u32(2).u32(u32::MAX);
    b.op(8).u32(0).u32(1).u32(u32::MAX);
    tree.apply(&b.bytes()).unwrap();
    let mut fonts = face();
    let mut atlas = Atlas::default();
    let icons = IconAtlas::default();
    tree.present(&mut fonts, &mut atlas, &icons);
    assert_eq!(tree.drawn.groups.len(), 2);
    let inner = tree.drawn.groups[0];
    let outer = tree.drawn.groups[1];
    assert_eq!(inner.material.unwrap().params[0][0], 0.8);
    assert_eq!(outer.material.unwrap().params[0][0], 0.2);
    assert!(inner.quad > outer.quad && inner.end <= outer.end);
    assert!(
        tree.drawn
            .effects
            .iter()
            .any(|f| matches!(f, PaintEffect::Blur(s) if *s > 0.0))
    );
    assert!((tree.drawn.quads[outer.quad as usize].color[3] - 0.5).abs() < 0.001);

    let mut clear = Batch::default();
    clear.op(6).u32(1).str("material").str("");
    clear.op(6).u32(2).str("material").str("");
    tree.apply(&clear.bytes()).unwrap();
    tree.present(&mut fonts, &mut atlas, &icons);
    assert!(tree.drawn.groups.iter().all(|g| g.material.is_none()));
    assert!(
        !tree.drawn.effects.is_empty(),
        "removing materials keeps the blur"
    );
}
