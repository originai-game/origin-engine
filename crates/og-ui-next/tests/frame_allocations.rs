//! What a steady frame allocates, counted on this thread: a HUD of 1000
//! DamageNumbers as the component makes them (the default pack's classes,
//! a real face), 28 spawning and 28 expiring a frame, stays within 1056
//! allocations a frame (the cooked-atlas path's count), and a frame with
//! nothing changed allocates none. Broken by a freed text's leaf not going
//! back for a new one to reuse (1420 a frame: every new number shaped and
//! painted into empty vectors).

use og_ui_next::icons::IconAtlas;
use og_ui_next::tree::{AnchorTarget, Sighting, Tree};

#[global_allocator]
static HEAP: og_core::alloc::Counting<std::alloc::System> =
    og_core::alloc::Counting(std::alloc::System);

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

/// The default pack's classes, dark, at 1280x720.
fn pack_tree() -> Tree {
    let mut t = Tree::new();
    t.resize(1280.0, 720.0);
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

/// The default Latin face as the sans.
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
fn a_thousand_damage_numbers_stay_in_their_budget_and_an_unchanged_frame_allocates_none() {
    let icons = IconAtlas::default();
    let mut atlas = og_draw::glyphs::Atlas::default();
    let mut fonts = face();
    let mut t = pack_tree();
    let mut b = Batch::default();
    for k in 0..1000 {
        spawn(&mut b, k);
    }
    t.apply(&b.bytes()).unwrap();
    t.frame(&mut fonts, &mut atlas, &icons);
    let mut next = 1000u32;
    let mut worst = 0;
    for f in 0..40u32 {
        let mut b = Batch::default();
        for _ in 0..28 {
            spawn(&mut b, next);
            expire(&mut b, next - 1000);
            next += 1;
        }
        let bytes = b.bytes();
        t.set_time(f64::from(f) / 60.0, false);
        let before = og_core::alloc::thread_allocations();
        t.apply(&bytes).unwrap();
        t.place_anchors(|target, _, _| {
            let AnchorTarget::Entity(e) = target else {
                return None;
            };
            Some(Sighting {
                at: [
                    (e as f32 * 7.0 + f as f32) % 1240.0 + 20.0,
                    (e as f32 * 3.0) % 680.0 + 20.0,
                ],
                ..Default::default()
            })
        });
        t.frame(&mut fonts, &mut atlas, &icons);
        // The first frames fill the spare leaves and the vectors' room.
        if f >= 20 {
            worst = worst.max(og_core::alloc::thread_allocations() - before);
        }
    }
    assert!(
        worst <= 1056,
        "{worst} allocations in a frame of 1000 damage numbers"
    );
    // Past every float's end, with the clock then held: nothing moves and
    // nothing changed.
    t.set_time(60.0, false);
    t.frame(&mut fonts, &mut atlas, &icons);
    t.frame(&mut fonts, &mut atlas, &icons);
    let before = og_core::alloc::thread_allocations();
    t.frame(&mut fonts, &mut atlas, &icons);
    assert_eq!(
        og_core::alloc::thread_allocations() - before,
        0,
        "an unchanged frame"
    );
}
