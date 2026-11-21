//! A HUD of op nodes as the TS renderer makes it, headless: the batch that
//! builds it (applied and its first frame), an unchanged frame, one text
//! changed (a clock), and the pointer coming over a panel whose `hover:`
//! fill shows and leaving it, and a d-pad move of the focus down the
//! panels, and 200 anchored nameplates placed again as the camera moves
//! (their placing alone and with the frame), and 1000 DamageNumbers as the
//! component makes them (the default pack's classes, a real face) rising
//! as hits spawn and expire, each the median of N runs with its
//! allocations.
//!
//!     cargo run --release -p og-ui-next --example op_costs [N]

use og_ui::IconAtlas;
use og_ui_next::style::Record;
use og_ui_next::tree::{MAGIC, Tree};
use std::time::Instant;

#[global_allocator]
static HEAP: og_core::alloc::Counting<std::alloc::System> =
    og_core::alloc::Counting(std::alloc::System);

fn allocations() -> usize {
    og_core::alloc::heap().map_or(0, |h| h.allocations)
}

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

/// 30 panels of a label, a value and an icon under one root: 121 nodes.
fn hud() -> Batch {
    let mut b = Batch::default();
    b.op(1).u32(1).u8(0);
    b.op(4).u32(1).str("col p-4 gap-2");
    b.op(8).u32(0).u32(1).u32(u32::MAX);
    for k in 0..30u32 {
        let (p, l, v, i) = (10 + k * 4, 11 + k * 4, 12 + k * 4, 13 + k * 4);
        b.op(1).u32(p).u8(0);
        b.op(4)
            .u32(p)
            .str("row gap-1 rounded-lg bg-surface hover:bg-accent");
        b.op(2).u32(l).str("Gold");
        b.op(2).u32(v).str("06:54");
        b.op(1).u32(i).u8(5);
        b.op(6).u32(i).str("name").str("clock");
        b.op(7).u32(p).u8(0).u8(1);
        for c in [l, v, i] {
            b.op(8).u32(p).u32(c).u32(u32::MAX);
        }
        b.op(8).u32(1).u32(p).u32(u32::MAX);
    }
    b
}

/// 200 nameplates anchored to entities 1..=200, each a label.
fn nameplates() -> Batch {
    let mut b = Batch::default();
    for k in 0..200u32 {
        let (p, l) = (1000 + k * 2, 1001 + k * 2);
        b.op(1).u32(p).u8(0);
        b.op(4).u32(p).str("row rounded-lg bg-surface");
        b.op(6).u32(p).str("anchor").str(&(k + 1).to_string());
        b.op(6).u32(p).str("lift").str("0 2 0");
        b.op(2).u32(l).str("Goblin");
        b.op(8).u32(p).u32(l).u32(u32::MAX);
        b.op(8).u32(0).u32(p).u32(u32::MAX);
    }
    b
}

fn tree() -> Tree {
    let mut t = Tree::new();
    t.resize(1280.0, 720.0);
    t.set_resolver(Box::new(|class, _, states, _| {
        let mut s = taffy::Style::default();
        for c in class.split_whitespace() {
            if c == "row" {
                s.flex_direction = taffy::FlexDirection::Row;
            }
        }
        let hover =
            class.contains("hover:bg-accent") && states & og_ui_next::classes::state::HOVERED != 0;
        let mut r = Record {
            layout: rune_taffy::CompactStyle::from(&s),
            ..Record::default()
        };
        if hover {
            r.skin.fill = [0.2, 0.4, 0.9, 1.0];
        }
        r
    }));
    t
}

/// The default pack's class resolver, dark, at 1280x720.
fn pack_tree() -> Tree {
    let mut t = tree();
    let pack = og_ui_next::pack::Pack::new(&[og_ui_next::pack::Layer::default_pack()]);
    let theme = pack.theme_json(og_ui_next::pack::Scheme::Dark);
    let classes =
        std::cell::RefCell::new(og_ui_next::classes::Classes::new(Some(&theme)).unwrap().0);
    let env = og_ui_next::classes::Env {
        window: [1280.0, 720.0],
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

/// DamageNumbers' class, as hud/damage-numbers.tsx gives each number.
const DAMAGE: &str = "-translate-x-1/2 animate-float text-[1.286rem] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-2xs";

/// Hit `k` spawned: a text node anchored to entity `k`, its number, in
/// the root.
fn spawn(b: &mut Batch, k: u32) {
    let (p, l) = (100_000 + k * 2, 100_001 + k * 2);
    b.op(1).u32(p).u8(1);
    b.op(4).u32(p).str(DAMAGE);
    b.op(6)
        .u32(p)
        .str("anchor")
        .str(&(k % 5000 + 1).to_string());
    if k % 3 == 0 {
        b.op(5).u32(p).str("color: #ffc93c");
    }
    b.op(2).u32(l).str(&format!("{}", 100 + k % 9000));
    b.op(8).u32(p).u32(l).u32(u32::MAX);
    b.op(8).u32(0).u32(p).u32(u32::MAX);
}

/// Hit `k` expired: out of the root and released.
fn expire(b: &mut Batch, k: u32) {
    let (p, l) = (100_000 + k * 2, 100_001 + k * 2);
    b.op(9).u32(0).u32(p);
    b.op(10).u32(l);
    b.op(10).u32(p);
}

/// A face with its file: every glyph laid out and painted.
fn face() -> og_ui::Fonts {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/latin/NotoSans-Latin.ttf"
    );
    let file =
        og_text::shape::FontFile::new(std::sync::Arc::new(std::fs::read(path).unwrap())).unwrap();
    let atlas = og_draw::MsdfFont {
        atlas: og_draw::TextureHandle::from_slot(1).unwrap(),
        distance_range: 4.0,
        foreground: false,
        silhouette: false,
        line_height: 1.2,
        glyphs: Vec::new(),
        kerning: Vec::new(),
    };
    og_ui::Fonts {
        body: og_ui::UiFont {
            handle: Some(og_draw::FontHandle::from_index(0)),
            metrics: og_text::text::FontMetrics::from_msdf(&atlas).with_file(Some(file)),
        },
        ..Default::default()
    }
}

fn median(mut v: Vec<(f64, usize)>) -> (f64, usize) {
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v[v.len() / 2]
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(21);
    let fonts = og_ui::Fonts::default();
    let icons = IconAtlas::default();
    let build = hud().bytes();
    let (mut builds, mut idle, mut changed) = (Vec::new(), Vec::new(), Vec::new());
    let mut hovered = Vec::new();
    let mut moved = Vec::new();
    for _ in 0..n {
        let mut t = tree();
        let a = allocations();
        let s = Instant::now();
        t.apply(&build).unwrap();
        t.frame(&fonts, &icons);
        builds.push((s.elapsed().as_secs_f64() * 1e3, allocations() - a));
        let a = allocations();
        let s = Instant::now();
        t.frame(&fonts, &icons);
        idle.push((s.elapsed().as_secs_f64() * 1e3, allocations() - a));
        for minute in 0..3u32 {
            let mut b = Batch::default();
            b.op(3).u32(12).str(&format!("06:{:02}", 55 + minute));
            let bytes = b.bytes();
            let a = allocations();
            let s = Instant::now();
            t.apply(&bytes).unwrap();
            t.frame(&fonts, &icons);
            changed.push((s.elapsed().as_secs_f64() * 1e3, allocations() - a));
        }
        // Over panel 10 and off it; the first round meets its hover
        // record, the others measure.
        let [x, y, _, h] = t.op_box(10).unwrap();
        for round in 0..4 {
            for (px, py) in [(x + 2.0, y + h / 2.0), (1270.0, 710.0)] {
                let a = allocations();
                let s = Instant::now();
                t.pointer(px, py, false, false);
                t.frame(&fonts, &icons);
                if round > 0 {
                    hovered.push((s.elapsed().as_secs_f64() * 1e3, allocations() - a));
                }
            }
        }
    }
    for _ in 0..n {
        let mut t = tree();
        t.apply(&build).unwrap();
        t.frame(&fonts, &icons);
        for k in 0..8 {
            let a = allocations();
            let s = Instant::now();
            t.nav(og_input::frame::Nav::Down);
            t.frame(&fonts, &icons);
            // The first move meets the stops' capacity.
            if k > 0 {
                moved.push((s.elapsed().as_secs_f64() * 1e3, allocations() - a));
            }
        }
    }
    let (mut placed, mut anchored) = (Vec::new(), Vec::new());
    let plates = nameplates().bytes();
    for _ in 0..n {
        let mut t = tree();
        t.apply(&build).unwrap();
        t.apply(&plates).unwrap();
        t.frame(&fonts, &icons);
        // The camera pans a px a frame; every tenth plate is off screen.
        for f in 0..6 {
            let project = |target, _lift: [f32; 3], _occlusion: bool| {
                let og_ui_next::tree::AnchorTarget::Entity(e) = target else {
                    return None;
                };
                (e % 10 != 0).then_some(og_ui_next::tree::Sighting {
                    at: [
                        (e as f32 * 6.0 + f as f32) % 1200.0,
                        (e as f32 * 3.0) % 700.0,
                    ],
                    ..Default::default()
                })
            };
            let a = allocations();
            let s = Instant::now();
            t.place_anchors(project);
            let p = (s.elapsed().as_secs_f64() * 1e3, allocations() - a);
            t.frame(&fonts, &icons);
            // The first frames meet the paint's capacity.
            if f > 1 {
                placed.push(p);
                anchored.push((s.elapsed().as_secs_f64() * 1e3, allocations() - a));
            }
        }
    }
    // 1000 DamageNumbers live at 60 Hz: each floats 0.8 s, so 21 spawn
    // and 21 expire a frame, and every unit moves a px a frame. A frame
    // is the batch, the placing and the frame; the moving-only frame has
    // no batch. The first 120 frames reach the steady state.
    let (mut damage, mut drift) = (Vec::new(), Vec::new());
    let mut paint_quads = 0;
    let face = face();
    for _ in 0..n {
        let mut t = pack_tree();
        let mut b = Batch::default();
        for k in 0..1000 {
            spawn(&mut b, k);
        }
        t.apply(&b.bytes()).unwrap();
        t.frame(&face, &icons);
        let mut next = 1000u32;
        for f in 0..180u32 {
            let project = |target, _lift: [f32; 3], _occlusion: bool| {
                let og_ui_next::tree::AnchorTarget::Entity(e) = target else {
                    return None;
                };
                Some(og_ui_next::tree::Sighting {
                    at: [
                        (e as f32 * 7.0 + f as f32) % 1240.0 + 20.0,
                        (e as f32 * 3.0) % 680.0 + 20.0,
                    ],
                    ..Default::default()
                })
            };
            t.set_time(f64::from(f) / 60.0, false);
            let moving_only = f % 4 == 3;
            let mut b = Batch::default();
            if !moving_only {
                // Four frames' hits in three, so 1000 stay live.
                for _ in 0..28 {
                    spawn(&mut b, next);
                    expire(&mut b, next - 1000);
                    next += 1;
                }
            }
            let bytes = b.bytes();
            let a = allocations();
            let s = Instant::now();
            if !moving_only {
                t.apply(&bytes).unwrap();
            }
            t.place_anchors(project);
            t.frame(&face, &icons);
            let m = (s.elapsed().as_secs_f64() * 1e3, allocations() - a);
            if f >= 120 {
                if moving_only {
                    drift.push(m);
                } else {
                    damage.push(m);
                }
                paint_quads = t.drawn.quads.len();
            }
        }
    }
    println!("1000 damage numbers: {paint_quads} quads painted a frame");
    assert_eq!(&MAGIC[..], b"OGOP");
    for (what, v) in [
        ("build", builds),
        ("unchanged", idle),
        ("one text", changed),
        ("hover", hovered),
        ("d-pad", moved),
        ("place 200", placed),
        ("200 moved", anchored),
        ("1000 dmg", damage),
        ("1000 drift", drift),
    ] {
        let (ms, allocs) = median(v);
        println!("{what:>10}: {ms:.4} ms, {allocs} allocations");
    }
}
