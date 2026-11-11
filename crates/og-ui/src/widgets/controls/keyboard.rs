//! The on-screen keyboard a text input opens when a pad gives it the focus
//! (docs/ui.md §2.3): a panel docked at the screen's bottom over a scrim,
//! the text with its caret, a candidate bar while kana compose, and rows
//! of keys a pad walks and a finger presses. Letters with shift and caps,
//! symbols, a number pad and the kana table with its voicing marks.

use super::*;

use crate::widgets::ModalDesc;

/// The keys an on-screen keyboard shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum KeyboardLayout {
    /// Latin letters (QWERTY), shift and caps.
    #[default]
    Letters,
    /// Digits and punctuation.
    Symbols,
    /// A number pad.
    Numeric,
    /// The kana table (gojūon); typed kana compose until committed.
    Kana,
}

const LAYOUTS: [KeyboardLayout; 4] = [
    KeyboardLayout::Letters,
    KeyboardLayout::Symbols,
    KeyboardLayout::Numeric,
    KeyboardLayout::Kana,
];

/// What an on-screen keyboard edits and how it types.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct KeyboardDesc {
    /// The layout it opens with.
    pub layout: KeyboardLayout,
    /// The kana layout is offered.
    pub kana: bool,
    /// The most characters (0: no limit).
    pub max_chars: u32,
    /// What is being typed; empty for none.
    pub title: String,
    pub placeholder: String,
    /// The enter key's label; empty is `Done`.
    pub submit_label: String,
    /// The text shows as dots; a number pad then takes digits only.
    pub password: bool,
}

/// What an on-screen keyboard did this tick.
pub(crate) struct KeyboardResult {
    pub changed: bool,
    /// Done was pressed.
    pub submitted: bool,
    /// Closed without submitting (Back, a press on the scrim).
    pub closed: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Key {
    Char(char),
    /// A digit that spans two keys (the PIN pad's 0).
    WideChar(char),
    Shift,
    Back(f32),
    Space(f32),
    Enter(f32),
    Voice,
    Semi,
    Small,
    Left,
    Right,
    Layout(KeyboardLayout, &'static str, f32),
}

/// The empty cells of the kana table.
const GAP: char = '\u{3000}';

impl Key {
    fn wide(self) -> f32 {
        match self {
            Key::WideChar(_) => 2.0,
            Key::Shift => 1.5,
            Key::Back(w) | Key::Space(w) | Key::Enter(w) | Key::Layout(_, _, w) => w,
            _ => 1.0,
        }
    }

    /// What it types: its accessible name and its glyph.
    fn name(self) -> String {
        match self {
            Key::Char(GAP) => String::new(),
            Key::Char(c) | Key::WideChar(c) => c.to_string(),
            Key::Layout(_, label, _) => label.into(),
            Key::Space(_) => "Space".into(),
            Key::Back(_) => "Backspace".into(),
            Key::Enter(_) => "Done".into(),
            Key::Shift => "Shift".into(),
            Key::Voice => "゛".into(),
            Key::Semi => "゜".into(),
            Key::Small => "小".into(),
            Key::Left => "Caret left".into(),
            Key::Right => "Caret right".into(),
        }
    }
}

fn chars(s: &str) -> Vec<Key> {
    s.chars().map(Key::Char).collect()
}

/// The kana table in rows: the gojūon read across by consonant (a row is a
/// vowel), as a pad keyboard lays it out.
const KANA_ROWS: [&str; 5] = [
    "あかさたなはまやらわ",
    "いきしちにひみ\u{3000}り\u{3000}",
    "うくすつぬふむゆるを",
    "えけせてねへめ\u{3000}れん",
    "おこそとのほもよろー",
];

/// A layout's keys in rows, the last the action row; `digits`: a masked
/// number (a PIN) takes digits only.
fn layout_rows(layout: KeyboardLayout, shifted: bool, kana: bool, digits: bool) -> Vec<Vec<Key>> {
    use KeyboardLayout as L;
    let bottom = |to: L, label: &'static str| {
        let mut row = vec![Key::Layout(to, label, 1.5)];
        if kana && layout != L::Kana {
            row.push(Key::Layout(L::Kana, "かな", 1.25));
        }
        if layout == L::Kana {
            row.push(Key::Layout(L::Letters, "ABC", 1.25));
        }
        row.extend([
            Key::Left,
            Key::Space(if layout == L::Kana { 3.0 } else { 4.0 }),
            Key::Right,
            Key::Enter(2.0),
        ]);
        row
    };
    match layout {
        L::Symbols => {
            let mut third = chars("!\"':;/?,.");
            third.push(Key::Back(1.5));
            vec![
                chars("1234567890"),
                chars("@#$%&*-+()"),
                third,
                bottom(L::Letters, "ABC"),
            ]
        }
        L::Numeric if digits => vec![
            chars("123"),
            chars("456"),
            chars("789"),
            vec![Key::WideChar('0'), Key::Back(1.0)],
            vec![Key::Enter(3.0)],
        ],
        L::Numeric => vec![
            chars("123"),
            chars("456"),
            chars("789"),
            chars("-0."),
            vec![Key::Left, Key::Back(1.0), Key::Right],
            vec![Key::Enter(3.0)],
        ],
        L::Kana => {
            let mut rows: Vec<Vec<Key>> = KANA_ROWS.iter().map(|r| chars(r)).collect();
            let mut marks = vec![Key::Voice, Key::Semi, Key::Small];
            marks.extend(chars("、。？！「」"));
            marks.push(Key::Back(2.0));
            rows.push(marks);
            rows.push(bottom(L::Symbols, "?123"));
            rows
        }
        L::Letters => {
            let up = |s: &str| {
                if shifted {
                    s.to_uppercase()
                } else {
                    s.to_string()
                }
            };
            let mut third = vec![Key::Shift];
            third.extend(chars(&up("zxcvbnm")));
            third.push(Key::Back(1.5));
            vec![
                chars(&up("qwertyuiop")),
                chars(&up("asdfghjkl")),
                third,
                bottom(L::Symbols, "?123"),
            ]
        }
    }
}

/// A kana's voiced (゛), semi-voiced (゜) or small form, cycling back to the
/// plain one; `None` when it has none.
fn kana_mark(c: char, mark: Key) -> Option<char> {
    let shift = |c: char, by: i32| char::from_u32((c as i32 + by) as u32);
    let voiced = |c: char| "かきくけこさしすせそたちつてとはひふへほ".contains(c);
    let semi = |c: char| "はひふへほ".contains(c);
    let small = |c: char| "あいうえおつやゆよわ".contains(c);
    let before = |c: char, by: i32| shift(c, -by).filter(|&p| p != c);
    match mark {
        Key::Voice => match c {
            'う' => Some('ゔ'),
            'ゔ' => Some('う'),
            c if voiced(c) => shift(c, 1),
            c if before(c, 1).is_some_and(voiced) => before(c, 1),
            // A semi-voiced kana takes the voice mark (ぱ → ば).
            c if before(c, 2).is_some_and(semi) => before(c, 1),
            _ => None,
        },
        Key::Semi => match c {
            c if semi(c) => shift(c, 2),
            c if before(c, 2).is_some_and(semi) => before(c, 2),
            c if before(c, 1).is_some_and(semi) => shift(c, 1),
            _ => None,
        },
        _ => match c {
            c if small(c) => shift(c, -1),
            c if shift(c, 1).is_some_and(small) => shift(c, 1),
            _ => None,
        },
    }
}

/// Hiragana as katakana (the candidate the kana layout always offers).
fn to_katakana(s: &str) -> String {
    s.chars()
        .map(|c| match c as u32 {
            k @ 0x3041..=0x3096 => char::from_u32(k + 0x60).unwrap_or(c),
            _ => c,
        })
        .collect()
}

impl Ui {
    /// An on-screen keyboard editing `value` while `open`: a panel docked
    /// at the screen's bottom over a scrim, the text with its caret, a
    /// candidate bar while kana compose, and keys a pad walks (the focus
    /// starts on the first) and a finger presses. Y erases, LB and RB move
    /// the caret, Back closes. Parts: `Keyboard.panel`, `Keyboard.preview`,
    /// `Keyboard.key[wide][action][active][primary]`,
    /// `Keyboard.candidates`, `Keyboard.candidate`. A dialog named by its
    /// title; keys are buttons named by what they type. Probe:
    /// `<id>/Panel/Content/Keys/Row#r/Key#k`.
    pub(crate) fn on_screen_keyboard(
        &mut self,
        id: &str,
        open: bool,
        value: &mut String,
        desc: &KeyboardDesc,
    ) -> KeyboardResult {
        use KeyboardLayout as L;
        let t = self.env.theme;
        let vp = self.env.viewport;
        let was = self.memory(&format!("{id}~was"), 0.0) > 0.0;
        let asked = LAYOUTS.iter().position(|&l| l == desc.layout).unwrap_or(0) as f64;
        if open && !was {
            self.remember(&format!("{id}~caret"), -1.0);
            self.remember(&format!("{id}~shift"), 0.0);
            self.remember(&format!("{id}~layout"), asked);
        }
        self.remember(&format!("{id}~was"), if open { 1.0 } else { 0.0 });
        let mut layout =
            LAYOUTS[(self.memory(&format!("{id}~layout"), asked) as usize).min(LAYOUTS.len() - 1)];
        if layout == L::Kana && !desc.kana {
            layout = L::Letters;
        }
        let mut shift = self.memory(&format!("{id}~shift"), 0.0) as u8;
        let count = value.chars().count();
        let caret_at = self.memory(&format!("{id}~caret"), -1.0);
        let mut caret = if caret_at < 0.0 {
            count
        } else {
            (caret_at as usize).min(count)
        };
        let comp_key = format!("{id}~comp");
        let mut comp = if open {
            self.front
                .kept_texts
                .get(&crate::id::key_of_path(&comp_key))
                .cloned()
                .unwrap_or_default()
        } else {
            String::new()
        };
        let composing = layout == L::Kana;
        let max = desc.max_chars as usize;
        let (mut changed, mut submitted) = (false, false);
        let mut error = false;
        // Inserts at the caret; false past `max_chars`.
        let insert = |value: &mut String, caret: &mut usize, s: &str| -> bool {
            let mut cs: Vec<char> = value.chars().collect();
            let add: Vec<char> = s.chars().collect();
            if max > 0 && cs.len() + add.len() > max {
                return false;
            }
            let at = (*caret).min(cs.len());
            cs.splice(at..at, add.iter().copied());
            *value = cs.into_iter().collect();
            *caret = at + add.len();
            true
        };
        macro_rules! put {
            ($s:expr) => {
                if insert(value, &mut caret, $s) {
                    changed = true;
                } else {
                    error = true;
                }
            };
        }
        let mut pressed: Option<Key> = None;
        let mut candidate: Option<String> = None;
        let rows = layout_rows(layout, shift > 0, desc.kana, desc.password);
        let wide = vp.width >= 720.0;
        let safe_w = vp.width - vp.safe_area.horizontal();
        let panel_w = if wide { safe_w.min(980.0) } else { safe_w };
        let touch = self.touch_frame();
        let gap = t.space.xxs;
        let base = (if touch { TOUCH } else { 40.0 })
            .max(56.0f32.min((vp.width.min(980.0) - 2.0 * t.space.md) / 11.5 - gap));
        let room = (vp.height - vp.safe_area.top - vp.safe_area.bottom) * 0.6;
        let key_h = (if touch { TOUCH } else { 32.0 })
            .max(
                self.by_text(base)
                    .min((room - gap * (rows.len() as f32 - 1.0)) / rows.len() as f32),
            )
            .round();
        let units = rows
            .iter()
            .map(|r| r.iter().map(|k| k.wide()).sum::<f32>())
            .fold(1.0, f32::max);
        let fits = (panel_w - 2.0 * t.space.md - gap * (units - 1.0)) / units;
        let unit = (self.by_text(base) * if layout == L::Numeric { 2.4 } else { 1.0 })
            .min(fits)
            .floor();
        let name = if !desc.title.is_empty() {
            desc.title.clone()
        } else if !desc.placeholder.is_empty() {
            desc.placeholder.clone()
        } else {
            "Keyboard".into()
        };
        let modal = self.open_modal(
            id,
            open,
            &ModalDesc {
                width: Some(Size::px(panel_w)),
                transition: Some(crate::motion::TransitionKind::SlideUp),
                bare: true,
                ..ModalDesc::default()
            },
        );
        // Docked at the bottom of the screen, as a phone's keyboard is.
        let panel = self.tree.parent();
        let scrim = self.tree.nodes[panel].parent;
        self.tree.el_mut(scrim).align = Align::End;
        if modal.visible {
            let (ink, faint, primary) = (
                self.role("ink"),
                self.role("inkFaint"),
                self.role("primary"),
            );
            let (control, strong, on_primary) = (
                self.role("control"),
                self.role("controlStrong"),
                self.role("onPrimary"),
            );
            self.open(
                "Content",
                Element {
                    direction: Direction::Column,
                    width: Size::grow(),
                    padding: Insets::all(t.space.md),
                    gap: t.space.sm,
                    radius: t.radius.panel,
                    background: Some(self.role("panelRaised").srgba()),
                    part: crate::dress::PartRef::parse(
                        "keyboard::part(panel)",
                        ElementClass::Panel,
                    )
                    .ok(),
                    ..Element::default()
                },
            );
            if !desc.title.is_empty() {
                self.text(
                    "Title",
                    &desc.title,
                    Text {
                        step: Some(t.type_ramp.sm),
                        color: Ink::Role("inkMuted"),
                        nowrap: true,
                        ellipsis: true,
                        ..Text::default()
                    },
                );
            }
            // The text with its caret, and the composition underlined at it.
            self.open(
                "Preview",
                Element {
                    direction: Direction::Row,
                    width: Size::grow(),
                    height: Size::px(self.sizes().control_md.max(44.0)),
                    padding: Insets::xy(t.space.md, 0.0),
                    cross: Cross::Center,
                    radius: t.radius.control,
                    background: Some(self.role("well").srgba()),
                    part: crate::dress::PartRef::parse(
                        "keyboard::part(preview)",
                        ElementClass::Field,
                    )
                    .ok(),
                    ..Element::default()
                },
            );
            let shown: Vec<char> = if desc.password {
                vec!['•'; count]
            } else {
                value.chars().collect()
            };
            let body = |color| Text {
                color: Ink::Role(color),
                nowrap: true,
                ..Text::default()
            };
            let before: String = shown[..caret.min(shown.len())].iter().collect();
            let after: String = shown[caret.min(shown.len())..].iter().collect();
            if !before.is_empty() {
                self.text("Before", &before, body("ink"));
            }
            if !comp.is_empty() {
                self.open(
                    "Composition",
                    Element {
                        direction: Direction::Column,
                        cross: Cross::Stretch,
                        ..Element::default()
                    },
                );
                self.text("Text", &comp, body("primary"));
                self.open(
                    "Underline",
                    Element {
                        width: Size::grow(),
                        height: Size::px(2.0),
                        background: Some(primary.srgba()),
                        ..Element::default()
                    },
                );
                self.close();
                self.close();
            }
            let blink = self.still() || (self.seconds() * 1.9) as i64 % 2 == 0;
            self.open(
                "Caret",
                Element {
                    width: Size::px(2.0),
                    height: Size::px(self.by_text(22.0)),
                    background: blink.then(|| self.role("focus").srgba()),
                    ..Element::default()
                },
            );
            self.close();
            if value.is_empty() && comp.is_empty() && !desc.placeholder.is_empty() {
                self.text("Placeholder", &desc.placeholder, body("inkFaint"));
            }
            if !after.is_empty() {
                self.text("After", &after, body("ink"));
            }
            if max > 0 {
                self.spacer("Gap", Some(Size::grow()));
                self.text(
                    "Count",
                    &format!("{count}/{max}"),
                    Text {
                        step: Some(t.type_ramp.caption),
                        color: Ink::Role(if count >= max { "danger" } else { "inkMuted" }),
                        nowrap: true,
                        ..Text::default()
                    },
                );
            }
            self.close();
            self.semantics(&Semantics {
                role: Some("textbox".into()),
                name: Some(name.clone()),
                value: Some(if desc.password {
                    String::new()
                } else {
                    value.clone()
                }),
                ..Semantics::default()
            });
            let mut candidates = Vec::new();
            if composing && !comp.is_empty() {
                candidates.push(comp.clone());
                let kata = to_katakana(&comp);
                if kata != comp {
                    candidates.push(kata);
                }
            }
            if !candidates.is_empty() {
                self.open(
                    "Candidates",
                    Element {
                        direction: Direction::Row,
                        width: Size::grow(),
                        gap: t.space.xs,
                        part: crate::dress::PartRef::parse(
                            "keyboard::part(candidates)",
                            ElementClass::Panel,
                        )
                        .ok(),
                        ..Element::default()
                    },
                );
                for (i, c) in candidates.iter().enumerate() {
                    let b = self.ctl(
                        crate::ui_idi("Candidate", i as u32),
                        Ctl {
                            part: "keyboard::part(candidate)",
                            height: Some(Size::px(self.by_text(if touch { TOUCH } else { 36.0 }))),
                            padding: Insets::xy(t.space.md, 0.0),
                            fill: Some(if i == 0 {
                                mix(control, primary, 0.16)
                            } else {
                                control
                            }),
                            wash_by: 0.1,
                            semantics: Some(Semantics {
                                role: Some("button".into()),
                                name: Some(c.clone()),
                                ..Semantics::default()
                            }),
                            ..Ctl::default()
                        },
                        |ui, _| ui.text("Text", c, body("ink")),
                    );
                    if b.r.clicked {
                        candidate = Some(c.clone());
                    }
                }
                self.close();
            }
            // The keys: a row of buttons whose widths are units of the narrowest.
            self.open(
                "Keys",
                Element {
                    direction: Direction::Column,
                    width: Size::grow(),
                    gap,
                    cross: Cross::Center,
                    ..Element::default()
                },
            );
            for (r, row) in rows.iter().enumerate() {
                self.open(
                    crate::ui_idi("Row", r as u32),
                    Element {
                        direction: Direction::Row,
                        gap,
                        align: Align::Center,
                        ..Element::default()
                    },
                );
                for (i, &k) in row.iter().enumerate() {
                    let blank = k == Key::Char(GAP);
                    let action = !matches!(k, Key::Char(_) | Key::WideChar(_));
                    let active = (k == Key::Shift && shift > 0)
                        || matches!(k, Key::Layout(to, _, _) if to == layout);
                    let is_enter = matches!(k, Key::Enter(_));
                    let w = (unit * k.wide() + gap * (k.wide() - 1.0)).round();
                    let label = if k == Key::Shift && shift == 2 {
                        "Caps lock".to_string()
                    } else {
                        k.name()
                    };
                    let b = self.ctl(
                        crate::ui_idi("Key", i as u32),
                        Ctl {
                            part: "keyboard::part(key)",
                            flags: vec![
                                (k.wide() > 1.0).then_some("wide"),
                                action.then_some("action"),
                                active.then_some("active"),
                                is_enter.then_some("primary"),
                            ],
                            disabled: blank,
                            width: Some(Size::px(w)),
                            height: Some(Size::px(key_h)),
                            touch: 0.0,
                            fill: Some(if is_enter {
                                primary
                            } else if active {
                                mix(control, primary, 0.25)
                            } else if action {
                                strong
                            } else {
                                control
                            }),
                            wash_by: 0.12,
                            press: 0.94,
                            lift: 0.0,
                            dim: if blank { 0.0 } else { 0.5 },
                            semantics: Some(Semantics {
                                role: Some("button".into()),
                                name: Some(label.clone()),
                                ..Semantics::default()
                            }),
                            ..Ctl::default()
                        },
                        |ui, _| {
                            if blank {
                                return;
                            }
                            let fg = if is_enter { on_primary } else { ink };
                            let icon = ui.by_text(20.0);
                            let text = |ui: &mut Ui, s: &str, color: C| {
                                ui.text(
                                    "Label",
                                    s,
                                    Text {
                                        color: Ink::C(color),
                                        nowrap: true,
                                        align: TextAlign::Center,
                                        ..Text::default()
                                    },
                                );
                            };
                            match k {
                                Key::Back(_) => ui.drawn("Icon", "backspace", icon, fg, None),
                                Key::Shift => ui.drawn(
                                    "Icon",
                                    "shift",
                                    icon,
                                    if shift > 0 { primary } else { fg },
                                    None,
                                ),
                                Key::Left => ui.drawn("Icon", "chevronStart", icon, fg, None),
                                Key::Right => ui.drawn("Icon", "chevronEnd", icon, fg, None),
                                Key::Enter(_) => {
                                    let s = if !desc.submit_label.is_empty() {
                                        desc.submit_label.as_str()
                                    } else if comp.is_empty() {
                                        "Done"
                                    } else {
                                        "Enter"
                                    };
                                    text(ui, s, fg);
                                }
                                Key::Space(_) => text(
                                    ui,
                                    if layout == L::Kana { "空白" } else { "space" },
                                    faint,
                                ),
                                _ => text(ui, &k.name(), fg),
                            }
                        },
                    );
                    if b.r.clicked && !blank {
                        pressed = Some(k);
                    }
                }
                self.close();
            }
            self.close();
            self.close();
        }
        self.close_modal();
        // A pad's shortcuts: Y (the details action) erases, the bumpers
        // move the caret.
        if open && pressed.is_none() {
            pressed = if self.action("details") {
                Some(Key::Back(1.0))
            } else if self.action("tabPrev") {
                Some(Key::Left)
            } else if self.action("tabNext") {
                Some(Key::Right)
            } else {
                None
            };
        }
        if let Some(c) = candidate {
            comp.clear();
            put!(&c);
            self.feedback(SoundEvent::Select);
        }
        if let Some(k) = pressed {
            match k {
                Key::Char(c) | Key::WideChar(c) => {
                    if composing && (c.is_alphabetic() || c == 'ー') {
                        comp.push(c);
                    } else {
                        if !comp.is_empty() {
                            put!(&std::mem::take(&mut comp));
                        }
                        put!(&c.to_string());
                    }
                    if shift == 1 {
                        shift = 0;
                    }
                }
                Key::Voice | Key::Semi | Key::Small => {
                    match comp.chars().last().and_then(|c| kana_mark(c, k)) {
                        Some(to) => {
                            comp.pop();
                            comp.push(to);
                        }
                        None => error = true,
                    }
                }
                Key::Space(_) => {
                    if !comp.is_empty() {
                        put!(&std::mem::take(&mut comp));
                    }
                    put!(" ");
                }
                Key::Back(_) => {
                    if comp.pop().is_none() && caret > 0 {
                        let mut cs: Vec<char> = value.chars().collect();
                        cs.remove(caret - 1);
                        *value = cs.into_iter().collect();
                        caret -= 1;
                        changed = true;
                    }
                }
                Key::Left if comp.is_empty() => caret = caret.saturating_sub(1),
                Key::Right if comp.is_empty() => caret = (caret + 1).min(value.chars().count()),
                Key::Left | Key::Right => {}
                Key::Enter(_) => {
                    if comp.is_empty() {
                        submitted = true;
                    } else {
                        put!(&std::mem::take(&mut comp));
                    }
                }
                Key::Shift => {
                    // A second press within 0.35 s locks the capitals.
                    let now = self.seconds();
                    let last = self.memory(&format!("{id}~shiftAt"), -10.0);
                    let quick = now - last < 0.35;
                    shift = match shift {
                        0 if quick => 2,
                        0 => 1,
                        1 if quick => 2,
                        _ => 0,
                    };
                    self.remember(&format!("{id}~shiftAt"), now);
                }
                Key::Layout(to, _, _) => {
                    if !comp.is_empty() {
                        put!(&std::mem::take(&mut comp));
                    }
                    layout = to;
                    shift = 0;
                }
            }
            self.feedback(match k {
                Key::Back(_) => SoundEvent::Close,
                Key::Enter(_) => SoundEvent::Confirm,
                _ => SoundEvent::Press,
            });
        }
        if error {
            self.feedback(SoundEvent::Error);
        }
        let closed = modal.dismissed && open;
        if closed {
            self.feedback(SoundEvent::Close);
        }
        self.remember(
            &format!("{id}~layout"),
            LAYOUTS.iter().position(|&l| l == layout).unwrap_or(0) as f64,
        );
        self.remember(&format!("{id}~caret"), caret as f64);
        self.remember(&format!("{id}~shift"), f64::from(shift));
        self.front
            .kept_texts
            .insert(crate::id::key_of_path(&comp_key), comp);
        KeyboardResult {
            changed,
            submitted,
            closed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kana_marks_cycle_and_cross() {
        assert_eq!(kana_mark('か', Key::Voice), Some('が'));
        assert_eq!(kana_mark('が', Key::Voice), Some('か'));
        assert_eq!(kana_mark('は', Key::Semi), Some('ぱ'));
        assert_eq!(kana_mark('ぱ', Key::Semi), Some('は'));
        assert_eq!(
            kana_mark('ば', Key::Semi),
            Some('ぱ'),
            "voiced to semi-voiced"
        );
        assert_eq!(
            kana_mark('ぱ', Key::Voice),
            Some('ば'),
            "semi-voiced to voiced"
        );
        assert_eq!(kana_mark('つ', Key::Small), Some('っ'));
        assert_eq!(kana_mark('っ', Key::Small), Some('つ'));
        assert_eq!(kana_mark('う', Key::Voice), Some('ゔ'));
        assert_eq!(kana_mark('な', Key::Voice), None);
        assert_eq!(to_katakana("かな"), "カナ");
    }
}
