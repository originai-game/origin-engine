//! Default pack shapes, art references, icon bindings, motion slot
//! presets, and sound tables.

use crate::tokens::Tokens;
use serde_json::{Map, Value, json};

/// The shapes every look has (`templates/styles/shared/shapes/`, white
/// masks drawn 4x, coloured by the element's fill): a round slot's
/// cooldown `disc` and progress `ring` (stretched whole, for a radial
/// wipe), a square slot's `tile` at the slot radius (`--radius-field`),
/// and the HUD's bevels: `plate-cut` (`--hud-plate-cut`, 6) and
/// `panel-cut` (`--hud-panel-cut`, 12) cut the leading top and trailing
/// bottom corners, `chip-cut` the trailing top one; a d-pad's `cross`,
/// its outline `cross-rim` and its top arm `cross-arm` (each stretched
/// whole over the pad's box: one shape, so no scale opens a seam in it);
/// then the art.
pub(super) fn shapes() -> Value {
    let mut out = Map::new();
    for (name, slice, width) in [
        ("disc", 0, None),
        ("ring", 0, None),
        ("cross", 0, None),
        ("cross-rim", 0, None),
        ("cross-arm", 0, None),
        ("tile", 24, Some(6)),
        ("plate-cut", 24, Some(6)),
        ("panel-cut", 48, Some(12)),
        ("chip-cut", 24, Some(6)),
    ] {
        let mut image = json!({ "src": format!("@styles/shared/shapes/{name}.svg"), "slice": slice, "tint": true });
        if let Some(w) = width {
            image["width"] = json!(w);
        }
        out.insert(name.to_owned(), image);
    }
    out.extend(art());
    Value::Object(out)
}

/// The art every look has (`templates/styles/shared/art/`, generated
/// pictures drawn 4x): an item or ability's octagon `socket` and its
/// `socket-rim` (a white mask the element's fill colours: a rarity rim),
/// a radial `glow` (a white mask the fill colours) and a coin or plate's
/// `sheen` stretched over the box, and a card's ornate `card-frame` (its
/// centre left open for the card's art).
pub(super) fn art() -> Vec<(String, Value)> {
    [
        ("socket", json!({ "src": "@styles/shared/art/socket.png", "slice": 76, "width": 19 })),
        ("socket-rim", json!({ "src": "@styles/shared/art/socket-rim.png", "slice": 76, "width": 19, "fill": false, "tint": true })),
        ("glow", json!({ "src": "@styles/shared/art/glow.png", "slice": 0, "tint": true })),
        ("sheen", json!({ "src": "@styles/shared/art/sheen.png", "slice": 0 })),
        ("card-frame", json!({ "src": "@styles/shared/art/card-frame.png", "slice": 60, "width": 15, "fill": false })),
        ]
        .into_iter()
        .map(|(name, image)| (name.to_owned(), image))
        .chain(effects())
        .collect()
}

/// The UI effects' particle textures (`templates/styles/shared/fx/`): the
/// engine's emitters name them by id, and as images the fill colours them
/// (`nine-fx-sparkle bg-gold`), stretched whole.
pub(super) fn effects() -> Vec<(String, Value)> {
    let mut out = Vec::new();
    for name in ["sparkle", "glint", "shard", "coin", "ray", "ring"] {
        out.push((
            format!("fx-{name}"),
            json!({ "src": format!("@styles/shared/fx/{name}.png"), "slice": 0, "tint": true }),
        ));
    }
    out
}

/// The input prompt glyphs every look has (`templates/styles/shared/glyphs/`,
/// single-colour masks the text colour tints), by og-input's glyph names
/// (`Glyph::icon`): every key og-input binds (`key.<code lowercased>`; a
/// left and right modifier, a numpad digit or operator and its main key
/// share one), the Mac's `key.command`, the mouse's, touch's, and each pad
/// family's buttons, d-pad and sticks.
pub(super) fn icons() -> Value {
    let mut out = Map::new();
    let mut put = |name: String, file: &str| {
        out.insert(name, json!(format!("@styles/shared/glyphs/{file}.svg")));
    };
    for code in og_input::KEY_CODES {
        let lower = code.to_ascii_lowercase();
        let file = match *code {
            c if c.starts_with("Key") => format!("key-{}", &lower[3..]),
            c if c.starts_with("Digit") => format!("key-{}", &c[5..]),
            c if c.starts_with("Numpad") && c[6..].parse::<u8>().is_ok() => {
                format!("key-{}", &c[6..])
            }
            "ShiftLeft" | "ShiftRight" => "key-shift".into(),
            "ControlLeft" | "ControlRight" => "key-ctrl".into(),
            "AltLeft" | "AltRight" => "key-alt".into(),
            "MetaLeft" | "MetaRight" => "key-meta-win".into(),
            "NumpadSubtract" => "key-minus".into(),
            "NumpadDivide" => "key-slash".into(),
            "NumpadDecimal" => "key-period".into(),
            "NumpadEnter" => "key-enter".into(),
            "NumpadEqual" => "key-equal".into(),
            _ => format!("key-{lower}"),
        };
        put(format!("key.{lower}"), &file);
    }
    for (name, file) in [
        ("key.command", "key-meta-mac"),
        ("key.wasd", "key-wasd"),
        ("key.arrows", "key-arrows"),
        ("mouse.left", "mouse-left"),
        ("mouse.middle", "mouse-middle"),
        ("mouse.right", "mouse-right"),
        ("mouse.move", "mouse-move"),
        ("mouse.wheel", "mouse-wheel"),
        ("touch.tap", "touch-tap"),
        ("xbox.a", "face-a"),
        ("xbox.b", "face-b"),
        ("xbox.x", "face-x"),
        ("xbox.y", "face-y"),
        ("xbox.lb", "bumper-lb"),
        ("xbox.rb", "bumper-rb"),
        ("xbox.lt", "trigger-lt"),
        ("xbox.rt", "trigger-rt"),
        ("xbox.view", "view"),
        ("xbox.menu", "menu"),
        ("ps.cross", "ps-cross"),
        ("ps.circle", "ps-circle"),
        ("ps.square", "ps-square"),
        ("ps.triangle", "ps-triangle"),
        ("ps.l1", "bumper-l1"),
        ("ps.r1", "bumper-r1"),
        ("ps.l2", "trigger-l2"),
        ("ps.r2", "trigger-r2"),
        ("ps.create", "ps-create"),
        ("ps.options", "menu"),
        ("switch.a", "face-a"),
        ("switch.b", "face-b"),
        ("switch.x", "face-x"),
        ("switch.y", "face-y"),
        ("switch.l", "bumper-l"),
        ("switch.r", "bumper-r"),
        ("switch.zl", "trigger-zl"),
        ("switch.zr", "trigger-zr"),
        ("switch.minus", "minus"),
        ("switch.plus", "plus"),
        ("pad.south", "pad-south"),
        ("pad.east", "pad-east"),
        ("pad.west", "pad-west"),
        ("pad.north", "pad-north"),
        ("pad.l1", "bumper-l1"),
        ("pad.r1", "bumper-r1"),
        ("pad.l2", "trigger-l2"),
        ("pad.r2", "trigger-r2"),
        ("pad.select", "view"),
        ("pad.start", "menu"),
    ] {
        put(name.into(), file);
    }
    // Every family's d-pad, sticks and home button.
    for family in ["xbox", "ps", "switch", "pad"] {
        for (name, file) in [
            ("dpad", "dpad"),
            ("dpadUp", "dpad-up"),
            ("dpadDown", "dpad-down"),
            ("dpadLeft", "dpad-left"),
            ("dpadRight", "dpad-right"),
            ("stickL", "stick-l"),
            ("stickR", "stick-r"),
            ("ls", "stick-l-press"),
            ("rs", "stick-r-press"),
            ("home", "home"),
        ] {
            put(format!("{family}.{name}"), file);
        }
    }
    Value::Object(out)
}

/// The motion slots' defaults, from motion.css's timing table, so a pack
/// restyles them by its tokens or its own `variants`.
pub(super) struct Timing<'a>(pub(super) &'a Tokens);

impl Timing<'_> {
    fn num(&self, name: &str) -> f64 {
        let v = self.0.get(name).unwrap_or_else(|| panic!("{name}"));
        v.trim().parse().unwrap_or_else(|_| panic!("{name}: {v}"))
    }

    /// A duration in ms: its own (`50ms`) or the `--motion-duration-*` it names.
    fn ms(&self, name: &str) -> f64 {
        let ms = |v: &str| {
            v.trim()
                .strip_suffix("ms")
                .and_then(|n| n.trim().parse::<f64>().ok())
        };
        let v = self.0.get(name).unwrap_or_else(|| panic!("{name}"));
        ms(v)
            .or_else(|| {
                self.0
                    .get(&format!("--motion-duration-{}", v.trim()))
                    .and_then(ms)
            })
            .unwrap_or_else(|| panic!("{name}: {v} names no duration"))
    }

    /// A scale as a `scale-*` utility.
    fn scale(&self, name: &str) -> String {
        let s = self.num(name);
        let pct = s * 100.0;
        if (pct - pct.round()).abs() < 1e-6 {
            format!("scale-{}", pct.round())
        } else {
            format!("scale-[{s}]")
        }
    }

    /// NSH's `ease-spring-[stiffness,damping]` of a unit mass for a damping
    /// ratio ζ and a response T in seconds: k = (2π/T)², c = 4πζ/T.
    fn spring(zeta: f64, response: f64) -> String {
        let tau = std::f64::consts::TAU;
        let (k, c) = ((tau / response).powi(2), 2.0 * tau * zeta / response);
        format!("ease-spring-[{},{}]", k.round(), (c * 10.0).round() / 10.0)
    }

    /// `Press.feedback`: the press scale (`--motion-scale-press`) under
    /// `active:`, in over the press row's `in` on ease-out, and back on its
    /// spring (its damping, its duration as the response).
    pub(super) fn press(&self) -> String {
        let spring = Self::spring(
            self.num("--motion-timing-press-damping"),
            self.ms("--motion-timing-press-duration") / 1000.0,
        );
        format!(
            "transition-transform {spring} active:{} active:duration-{} active:ease-out",
            self.scale("--motion-scale-press"),
            self.ms("--motion-timing-press-in")
        )
    }

    /// `Screen.frame`: in from transparent at the enter scale on the enter
    /// row, out to transparent at the exit scale on the exit row.
    pub(super) fn screen(&self) -> String {
        let ease = |n: &str| self.ease(n);
        format!(
            "transition duration-{} ease-{} starting:opacity-0 starting:{} exit:opacity-0 exit:{} exit:duration-{} exit:ease-{}",
            self.ms("--motion-timing-enter-duration"),
            ease("--motion-timing-enter-easing"),
            self.scale("--motion-timing-enter-scale"),
            self.scale("--motion-timing-exit-scale"),
            self.ms("--motion-timing-exit-duration"),
            ease("--motion-timing-exit-easing"),
        )
    }

    /// `Modal.frame`: a pop in from transparent at the pop scale, back on
    /// the pop row's spring (its damping, its duration as the response),
    /// out as a screen leaves.
    pub(super) fn modal(&self) -> String {
        let spring = Self::spring(
            self.num("--motion-timing-pop-damping"),
            self.ms("--motion-timing-pop-duration") / 1000.0,
        );
        format!(
            "transition {spring} starting:opacity-0 starting:{} exit:opacity-0 exit:{} exit:duration-{} exit:ease-{}",
            self.scale("--motion-timing-pop-scale"),
            self.scale("--motion-timing-exit-scale"),
            self.ms("--motion-timing-exit-duration"),
            self.ease("--motion-timing-exit-easing"),
        )
    }

    /// `Modal.scrim`: the scrim a dialog dims the screen with, fading in on
    /// the enter row and out on the exit row, on its own while the dialog
    /// pops.
    pub(super) fn scrim(&self) -> String {
        format!(
            "absolute inset-0 bg-scrim transition-opacity duration-{} ease-{} starting:opacity-0 exit:opacity-0 exit:duration-{} exit:ease-{}",
            self.ms("--motion-timing-enter-duration"),
            self.ease("--motion-timing-enter-easing"),
            self.ms("--motion-timing-exit-duration"),
            self.ease("--motion-timing-exit-easing"),
        )
    }

    /// `List.cascade`: the rows a list puts on screen together enter one
    /// after another, the cascade row's step apart, spread over its cap.
    pub(super) fn cascade(&self) -> String {
        format!(
            "stagger-{}/{}",
            self.ms("--motion-timing-cascade-step"),
            self.ms("--motion-timing-cascade-cap")
        )
    }

    /// `Count.roll`: a number rolling to a new value on the count row.
    pub(super) fn count(&self) -> String {
        format!(
            "transition-count duration-{} ease-{}",
            self.ms("--motion-timing-count-duration"),
            self.ease("--motion-timing-count-easing")
        )
    }

    /// `Tab.indicator`, `Menu.indicator`: a selection's indicator, sliding
    /// on the glide row's spring from the one it replaces (`glide-<name>`).
    pub(super) fn indicator(&self, name: &str) -> String {
        let spring = Self::spring(
            self.num("--motion-timing-glide-damping"),
            self.ms("--motion-timing-glide-duration") / 1000.0,
        );
        format!("transition-layout {spring} glide-{name}")
    }

    /// `Bar.fill`: a fill easing to a new value on the bar row.
    pub(super) fn bar(&self) -> String {
        format!(
            "transition-layout duration-{} ease-{}",
            self.ms("--motion-timing-bar-duration"),
            self.ease("--motion-timing-bar-easing")
        )
    }

    /// `Bar.trail`: what was lost draining after the fill, using the
    /// theme's `duration.trail` and `duration.trail-hold` tokens.
    pub(super) fn trail(&self) -> String {
        format!(
            "transition-layout duration-trail delay-trail-hold ease-{}",
            self.ease("--motion-timing-bar-easing")
        )
    }

    /// An easing row's name, as its `ease-*` utility takes it.
    fn ease(&self, name: &str) -> String {
        self.0
            .get(name)
            .unwrap_or_else(|| panic!("{name}"))
            .trim()
            .to_owned()
    }
}

/// The default pack's sounds: each event's clip of the shared set
/// (`@styles/shared/sfx/default/`, mastered by
/// templates/tools/master-ui-sounds.ts so the events are balanced), with a
/// small pitch spread on the repeated ones.
pub(super) fn sounds() -> Value {
    const SFX: &str = "@styles/shared/sfx/default";
    let sound = |clip: &str, volume: Value, pitch: Option<[f32; 2]>| {
        let mut s = json!({ "clips": [format!("{SFX}/{clip}.ogg")], "volume": volume });
        if let Some(p) = pitch {
            s["pitch"] = json!(p);
        }
        s
    };
    let jitter = Some([0.98, 1.02]);
    let hover = sound("tap", json!(0.12), Some([1.18, 1.26]));
    let mut out = Map::new();
    for (event, clip, volume, pitch) in [
        ("press", "tap", json!([0.45, 0.5]), Some([0.96, 1.04])),
        ("select", "tab", json!(0.5), jitter),
        ("confirm", "confirm", json!(0.5), None),
        ("cancel", "back", json!(0.5), None),
        ("back", "back", json!(0.5), None),
        ("open", "open", json!(0.45), None),
        ("close", "close", json!(0.45), None),
        ("error", "error", json!(0.5), None),
        ("reward", "reward", json!(0.5), None),
        ("toggle", "toggle", json!(0.5), jitter),
        ("tab", "tab", json!(0.5), jitter),
        ("levelup", "levelup", json!(0.5), None),
        ("purchase", "purchase", json!(0.5), None),
        ("countdown", "countdown", json!(0.5), None),
        ("tick", "tick", json!(0.35), Some([0.97, 1.03])),
        ("step", "step", json!(0.4), jitter),
        ("swipe", "swipe", json!(0.45), jitter),
        ("notify", "notify", json!(0.5), None),
        ("unlock", "unlock", json!(0.55), None),
    ] {
        out.insert(event.into(), sound(clip, volume, pitch));
    }
    out.insert("hover".into(), hover.clone());
    out.insert("Button.plate".into(), json!({ "hover": hover }));
    out.insert(
        "Button.plate.primary".into(),
        json!({ "hover": hover, "press": sound("confirm", json!(0.45), jitter) }),
    );
    Value::Object(out)
}
