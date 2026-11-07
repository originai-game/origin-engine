//! Art slots (docs/ui.md §3.3, §3.3): every image in a screen is a slot
//! with a spec the art pipeline reads, so generated art has a specified
//! place. A [`SlotSpec`] is plain data: a name, a [`SlotKind`], the aspect
//! (or the aspects allowed), a focal point cropping keeps, a safe
//! rectangle where nothing important may sit because text or controls are
//! laid over it, a fit, a mask and the style's treatments.
//! [`SlotSpec::new`] gives a kind's defaults and [`SlotSpec::validate`]
//! refuses a spec with named fixes.
//!
//! The kinds are Appendix F's, each with the aspects the brief asks for
//! ([`SlotKind::info`]); the art brief (`packages/knowledge/scripts/brief.ts`,
//! its `UI_KINDS`) carries the same names, aspects, focal points and safe
//! rectangles, held equal by this crate's tests. Store capsules follow
//! Steamworks' asset sizes as of 2024 (header 920×430, small 462×174, main
//! 1232×706, vertical 748×896, library 600×900, hero 3840×1240).
//!
//! Until its art lands a slot draws a placeholder from the style and the
//! kind ([`SlotKind::placeholder`]): a gradient of two of the palette's
//! roles, the kind's silhouette from the style's icon set, and the slot's
//! name in the default style only; never an empty grey box
//! ([`crate::Ui::art`]). [`document_slots`] reads every `art` element of a
//! document with its spec, what `og ui slots` lists.

use crate::tokens::{ElementClass, names};

/// A width-to-height ratio in whole numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aspect {
    pub w: u16,
    pub h: u16,
}

impl Aspect {
    pub const fn new(w: u16, h: u16) -> Aspect {
        Aspect { w, h }
    }

    pub fn ratio(self) -> f32 {
        f32::from(self.w) / f32::from(self.h)
    }

    /// The same shape within half a percent (920×430 is 92:43).
    pub fn same(self, o: Aspect) -> bool {
        (self.ratio() / o.ratio() - 1.0).abs() < 0.005
    }
}

impl std::fmt::Display for Aspect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.w, self.h)
    }
}

names! {
    /// A kind of generated art a slot holds (Appendix F).
    SlotKind {
        /// Key art and title backgrounds.
        KeyArt "keyArt",
        /// Store capsules.
        Capsule "capsule",
        /// The game's logo, transparent.
        Logo "logo",
        /// The app icon and game tile.
        AppIcon "appIcon",
        /// A character splash.
        CharacterSplash "characterSplash",
        /// A character's full body.
        FullBody "fullBody",
        /// Portraits and busts, with expression sets.
        Bust "bust",
        /// A visual novel's layered, transparent sprite.
        VnSprite "vnSprite",
        /// An event CG.
        EventCg "eventCg",
        /// Item and equipment icons, multi-cell items.
        ItemIcon "itemIcon",
        /// Ability, skill and buff icons.
        AbilityIcon "abilityIcon",
        /// A card's art window.
        CardArt "cardArt",
        /// A card's frame.
        CardFrame "cardFrame",
        /// Unit and building portraits.
        UnitPortrait "unitPortrait",
        /// Level, track and stage thumbnails.
        LevelThumb "levelThumb",
        /// A song's jacket.
        SongJacket "songJacket",
        /// A save slot's thumbnail.
        SaveThumb "saveThumb",
        /// Menu backgrounds.
        MenuBackground "menuBackground",
        /// A tiling menu pattern.
        MenuPattern "menuPattern",
        /// Event and gacha banners.
        Banner "banner",
        /// Player banners and calling cards.
        PlayerBanner "playerBanner",
        /// Emblems, crests, rank badges, flags.
        Emblem "emblem",
        /// Achievement icons.
        Achievement "achievement",
        /// Emotes and stickers.
        Emote "emote",
        /// Currency and resource icons.
        Currency "currency",
        /// Map glyphs and points of interest.
        MapGlyph "mapGlyph",
        /// Tech and talent node icons.
        TalentIcon "talentIcon",
        /// Tutorial and loading-tip art.
        Tutorial "tutorial",
        /// A cosmetic's preview, rendered from a viewport (§5.5), not generated.
        CosmeticPreview "cosmeticPreview",
        /// The style's frames (panels, cards, slots) as nine-slices.
        StyleFrame "styleFrame",
        /// The style's buttons, one nine-slice per state.
        StyleButton "styleButton",
        /// The style's ornaments: corners, flourishes, dividers.
        Ornament "ornament",
        /// The style's icon set, one sheet.
        IconSet "iconSet",
    }
}

/// How the art fills the slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fit {
    /// Fills it, cropped around the focal point.
    #[default]
    Cover,
    /// Whole, letterboxed.
    Contain,
    /// Stretched to it.
    Fill,
    /// Its corners kept, its edges and centre stretched (style art).
    NineSlice,
    /// Repeated at its own size.
    Tile,
}

/// What the art is cut to.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Mask {
    #[default]
    None,
    /// The slot's rectangle with this corner radius, CSS px.
    Round(f32),
    Circle,
    Squircle,
    Hexagon,
    /// The style's corner for an element class.
    Shape(ElementClass),
    /// The art's own alpha (transparent art).
    Alpha,
}

/// A look the style lays over the art.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Treatment {
    /// Darkened towards the edges, so text over it reads.
    Vignette,
    /// Mapped onto two of the palette's colours.
    Duotone,
    /// The style's frame round it.
    Frame,
    /// A glow in its rarity's colour.
    RarityGlow,
}

/// The aspect a slot takes.
#[derive(Clone, Debug, PartialEq)]
pub enum SlotAspect {
    One(Aspect),
    /// Any of these, the art chosen by the slot's laid-out shape.
    AnyOf(Vec<Aspect>),
    /// Anything between the two, inclusive.
    Range(Aspect, Aspect),
    /// Whatever the art has (logos, ornaments).
    Free,
}

/// What a kind is, for its slots' defaults and the art brief.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KindInfo {
    /// The aspects it is drawn at, the first the default; empty: free.
    pub aspects: &'static [Aspect],
    /// Any aspect between these two is also allowed.
    pub range: Option<(Aspect, Aspect)>,
    /// Drawn on a clear background.
    pub transparent: bool,
    /// Repeats edge to edge.
    pub tiling: bool,
    /// Where the subject sits (0–1, from the top left).
    pub focal: [f32; 2],
    /// Where text or controls usually lie over it (x0, y0, x1, y1, 0–1).
    pub safe: Option<[f32; 4]>,
    pub fit: Fit,
    /// The smallest CSS px size it must read at (0: no floor).
    pub legible_px: u16,
    /// Drawn as a nine-slice with declared insets.
    pub nine_slice: bool,
    /// The states drawn, one image each (style buttons).
    pub states: &'static [&'static str],
    /// Made by an image generator (false: rendered in the engine).
    pub generated: bool,
}

const fn info(
    aspects: &'static [Aspect],
    transparent: bool,
    focal: [f32; 2],
    fit: Fit,
) -> KindInfo {
    KindInfo {
        aspects,
        range: None,
        transparent,
        tiling: false,
        focal,
        safe: None,
        fit,
        legible_px: 0,
        nine_slice: false,
        states: &[],
        generated: true,
    }
}

const CENTRE: [f32; 2] = [0.5, 0.5];
/// The multi-cell shapes of a grid inventory's items (columns × rows).
pub const ITEM_CELLS: [[u8; 2]; 4] = [[1, 1], [1, 2], [2, 3], [2, 4]];
/// The states a style button is drawn in.
pub const BUTTON_STATES: [&str; 5] = ["normal", "hover", "pressed", "disabled", "focused"];

impl SlotKind {
    /// The kind's aspects, focal point, safe rectangle and fit (Appendix F).
    pub fn info(self) -> KindInfo {
        use SlotKind as K;
        match self {
            K::KeyArt => KindInfo {
                safe: Some([0.0, 0.0, 0.4, 1.0]),
                ..info(
                    &[
                        Aspect { w: 16, h: 9 },
                        Aspect { w: 21, h: 9 },
                        Aspect { w: 9, h: 16 },
                    ],
                    false,
                    [0.66, 0.45],
                    Fit::Cover,
                )
            },
            K::Capsule => info(
                &[
                    Aspect { w: 92, h: 43 },
                    Aspect { w: 77, h: 29 },
                    Aspect { w: 616, h: 353 },
                    Aspect { w: 187, h: 224 },
                    Aspect { w: 2, h: 3 },
                    Aspect { w: 96, h: 31 },
                ],
                false,
                [0.5, 0.45],
                Fit::Cover,
            ),
            K::Logo => info(&[], true, CENTRE, Fit::Contain),
            K::AppIcon => KindInfo {
                legible_px: 32,
                ..info(&[Aspect { w: 1, h: 1 }], false, CENTRE, Fit::Cover)
            },
            K::CharacterSplash => KindInfo {
                safe: Some([0.0, 0.78, 1.0, 1.0]),
                ..info(
                    &[Aspect { w: 16, h: 9 }, Aspect { w: 9, h: 16 }],
                    false,
                    [0.5, 0.3],
                    Fit::Cover,
                )
            },
            K::FullBody => KindInfo {
                range: Some((Aspect { w: 1, h: 2 }, Aspect { w: 9, h: 16 })),
                ..info(
                    &[Aspect { w: 9, h: 16 }, Aspect { w: 1, h: 2 }],
                    true,
                    [0.5, 0.18],
                    Fit::Contain,
                )
            },
            K::Bust => info(
                &[Aspect { w: 1, h: 1 }, Aspect { w: 3, h: 4 }],
                true,
                [0.5, 0.35],
                Fit::Cover,
            ),
            K::VnSprite => info(&[Aspect { w: 1, h: 2 }], true, [0.5, 0.15], Fit::Contain),
            K::EventCg => KindInfo {
                safe: Some([0.0, 0.72, 1.0, 1.0]),
                ..info(&[Aspect { w: 16, h: 9 }], false, [0.5, 0.4], Fit::Cover)
            },
            K::ItemIcon => KindInfo {
                legible_px: 32,
                ..info(
                    &[
                        Aspect { w: 1, h: 1 },
                        Aspect { w: 1, h: 2 },
                        Aspect { w: 2, h: 3 },
                    ],
                    true,
                    CENTRE,
                    Fit::Contain,
                )
            },
            K::AbilityIcon => KindInfo {
                legible_px: 24,
                ..info(&[Aspect { w: 1, h: 1 }], false, CENTRE, Fit::Cover)
            },
            K::CardArt => info(&[Aspect { w: 4, h: 3 }], false, [0.5, 0.45], Fit::Cover),
            K::CardFrame => info(&[Aspect { w: 5, h: 7 }], true, CENTRE, Fit::Contain),
            K::UnitPortrait => KindInfo {
                legible_px: 32,
                ..info(&[Aspect { w: 1, h: 1 }], false, [0.5, 0.4], Fit::Cover)
            },
            K::LevelThumb => info(
                &[Aspect { w: 16, h: 9 }, Aspect { w: 4, h: 3 }],
                false,
                CENTRE,
                Fit::Cover,
            ),
            K::SongJacket => info(&[Aspect { w: 1, h: 1 }], false, CENTRE, Fit::Cover),
            K::SaveThumb => info(&[Aspect { w: 16, h: 9 }], false, CENTRE, Fit::Cover),
            K::MenuBackground => KindInfo {
                safe: Some([0.0, 0.0, 0.45, 1.0]),
                ..info(
                    &[Aspect { w: 16, h: 9 }, Aspect { w: 21, h: 9 }],
                    false,
                    [0.68, 0.45],
                    Fit::Cover,
                )
            },
            K::MenuPattern => KindInfo {
                tiling: true,
                ..info(&[Aspect { w: 1, h: 1 }], false, CENTRE, Fit::Tile)
            },
            K::Banner => KindInfo {
                range: Some((Aspect { w: 16, h: 9 }, Aspect { w: 2, h: 1 })),
                safe: Some([0.0, 0.0, 0.45, 1.0]),
                ..info(
                    &[
                        Aspect { w: 2, h: 1 },
                        Aspect { w: 16, h: 9 },
                        Aspect { w: 3, h: 1 },
                    ],
                    false,
                    [0.68, 0.5],
                    Fit::Cover,
                )
            },
            K::PlayerBanner => KindInfo {
                range: Some((Aspect { w: 4, h: 1 }, Aspect { w: 5, h: 1 })),
                safe: Some([0.0, 0.55, 0.6, 1.0]),
                ..info(
                    &[
                        Aspect { w: 4, h: 1 },
                        Aspect { w: 5, h: 1 },
                        Aspect { w: 1, h: 2 },
                    ],
                    false,
                    [0.75, 0.4],
                    Fit::Cover,
                )
            },
            K::Emblem => KindInfo {
                legible_px: 32,
                ..info(&[Aspect { w: 1, h: 1 }], true, CENTRE, Fit::Contain)
            },
            K::Achievement => KindInfo {
                legible_px: 32,
                ..info(&[Aspect { w: 1, h: 1 }], false, CENTRE, Fit::Cover)
            },
            K::Emote => KindInfo {
                legible_px: 28,
                ..info(&[Aspect { w: 1, h: 1 }], true, CENTRE, Fit::Contain)
            },
            K::Currency => KindInfo {
                legible_px: 16,
                ..info(&[Aspect { w: 1, h: 1 }], true, CENTRE, Fit::Contain)
            },
            K::MapGlyph => KindInfo {
                legible_px: 16,
                ..info(&[Aspect { w: 1, h: 1 }], true, CENTRE, Fit::Contain)
            },
            K::TalentIcon => KindInfo {
                legible_px: 32,
                ..info(&[Aspect { w: 1, h: 1 }], false, CENTRE, Fit::Cover)
            },
            K::Tutorial => info(
                &[Aspect { w: 16, h: 9 }, Aspect { w: 4, h: 3 }],
                false,
                CENTRE,
                Fit::Cover,
            ),
            K::CosmeticPreview => KindInfo {
                generated: false,
                ..info(
                    &[Aspect { w: 1, h: 1 }, Aspect { w: 3, h: 4 }],
                    false,
                    [0.5, 0.45],
                    Fit::Cover,
                )
            },
            K::StyleFrame => KindInfo {
                nine_slice: true,
                ..info(&[], true, CENTRE, Fit::NineSlice)
            },
            K::StyleButton => KindInfo {
                nine_slice: true,
                states: &BUTTON_STATES,
                ..info(&[], true, CENTRE, Fit::NineSlice)
            },
            K::Ornament => info(&[], true, CENTRE, Fit::Contain),
            K::IconSet => KindInfo {
                legible_px: 24,
                ..info(&[Aspect { w: 1, h: 1 }], true, CENTRE, Fit::Contain)
            },
        }
    }

    /// Whether `aspect` is one this kind is drawn at.
    pub fn allows(self, aspect: Aspect) -> bool {
        let i = self.info();
        i.aspects.is_empty()
            || i.aspects.iter().any(|k| k.same(aspect))
            || i.range.is_some_and(|(lo, hi)| {
                let (lo, hi) = (lo.ratio().min(hi.ratio()), lo.ratio().max(hi.ratio()));
                (lo * 0.995..=hi * 1.005).contains(&aspect.ratio())
            })
    }
}

/// A slot's spec (see the module docs).
#[derive(Clone, Debug, PartialEq)]
pub struct SlotSpec {
    /// Unique in its screen: `hero`, `shop/item3`.
    pub name: String,
    pub kind: SlotKind,
    pub aspect: SlotAspect,
    /// The point cropping keeps (0–1 from the top left).
    pub focal: [f32; 2],
    /// Where nothing important may sit (x0, y0, x1, y1, 0–1): text or
    /// controls are laid over it.
    pub safe: Option<[f32; 4]>,
    pub fit: Fit,
    pub mask: Mask,
    pub treatments: Vec<Treatment>,
    /// A grid item's cells, columns × rows (item icons; one of [`ITEM_CELLS`]).
    pub cells: Option<[u8; 2]>,
    /// A nine-slice's insets as fractions of the art (left, top, right, bottom).
    pub insets: Option<[f32; 4]>,
    /// The states drawn (style buttons).
    pub states: Vec<String>,
    /// The set it is drawn with, so one generation keeps their look
    /// consistent: the kind's name unless the document names another.
    pub set: String,
}

impl SlotSpec {
    /// A slot of `kind` with the kind's defaults: its first aspect (all
    /// of them for a free or several-aspect kind), focal point, safe
    /// rectangle and fit; a style frame or button a quarter of the art as
    /// its insets, a button its states.
    pub fn new(name: &str, kind: SlotKind) -> SlotSpec {
        let i = kind.info();
        SlotSpec {
            name: name.into(),
            kind,
            aspect: match i.aspects {
                [] => SlotAspect::Free,
                [one] => SlotAspect::One(*one),
                [first, ..] => SlotAspect::One(*first),
            },
            focal: i.focal,
            safe: i.safe,
            fit: i.fit,
            mask: if i.transparent {
                Mask::Alpha
            } else {
                Mask::None
            },
            treatments: Vec::new(),
            cells: (kind == SlotKind::ItemIcon).then_some([1, 1]),
            insets: i.nine_slice.then_some([0.25; 4]),
            states: i.states.iter().map(|s| s.to_string()).collect(),
            set: kind.name().into(),
        }
    }

    /// An `art` element's spec: the kind's defaults, then its props as
    /// `word` and `nums` give them (`kind`, `ratio`, `focal`, `safe`,
    /// `fit`, `itemCells`, `slice`, `artSet`; cells without a ratio are
    /// drawn at their own aspect). A prop of the wrong shape
    /// keeps the default; [`SlotSpec::validate`] names what is wrong.
    pub fn from_props(
        path: &str,
        word: impl Fn(&str) -> Option<String>,
        nums: impl Fn(&str) -> Option<Vec<f32>>,
    ) -> SlotSpec {
        let kind = word("kind")
            .and_then(|k| SlotKind::from_name(&k))
            .unwrap_or(SlotKind::ALL[0]);
        let mut s = SlotSpec::new(path, kind);
        if let Some(n) = nums("ratio").filter(|n| n.len() == 2) {
            s.aspect = SlotAspect::One(Aspect::new(n[0] as u16, n[1] as u16));
        }
        if let Some(n) = nums("focal").filter(|n| n.len() == 2) {
            s.focal = [n[0], n[1]];
        }
        if let Some(n) = nums("safe").filter(|n| n.len() == 4) {
            s.safe = Some([n[0], n[1], n[2], n[3]]);
        }
        if let Some(f) = word("fit") {
            s.fit = match f.as_str() {
                "contain" => Fit::Contain,
                "fill" => Fit::Fill,
                "nineSlice" => Fit::NineSlice,
                "tile" => Fit::Tile,
                _ => Fit::Cover,
            };
        }
        if let Some(n) = nums("itemCells").filter(|n| n.len() == 2) {
            s.cells = Some([n[0] as u8, n[1] as u8]);
            if nums("ratio").is_none() {
                s.aspect = SlotAspect::One(Aspect::new(n[0] as u16, n[1] as u16));
            }
        }
        if let Some(n) = nums("slice") {
            s.insets = match n[..] {
                [a] => Some([a; 4]),
                [x, y] => Some([x, y, x, y]),
                [l, t, r, b] => Some([l, t, r, b]),
                _ => s.insets,
            };
        }
        if let Some(w) = word("artSet") {
            s.set = w;
        }
        s
    }

    /// Every problem with the spec, each with its fix; empty when it is sound.
    pub fn validate(&self) -> Vec<String> {
        let mut out = Vec::new();
        let n = &self.name;
        let i = self.kind.info();
        let k = self.kind.name();
        let mut push = |s: String| out.push(format!("slot '{n}': {s}"));
        if n.is_empty()
            || !n
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '/'))
        {
            push("a slot's name is letters, digits, -, _ and /".into());
        }
        let allowed = || {
            let mut s: Vec<String> = i.aspects.iter().map(|x| x.to_string()).collect();
            if let Some((lo, hi)) = i.range {
                s.push(format!("{lo} to {hi}"));
            }
            s.join(", ")
        };
        let check = |x: Aspect, push: &mut dyn FnMut(String)| {
            if x.w == 0 || x.h == 0 {
                push(format!("the aspect {x} has a zero side"));
            } else if !self.kind.allows(x) {
                push(format!(
                    "{k} art is drawn at {}, not {x}: pick one of those",
                    allowed()
                ));
            }
        };
        match &self.aspect {
            SlotAspect::One(x) => check(*x, &mut push),
            SlotAspect::AnyOf(xs) => {
                if xs.is_empty() {
                    push("lists no aspect: give one or more".into());
                }
                for x in xs {
                    check(*x, &mut push);
                }
            }
            SlotAspect::Range(lo, hi) => {
                check(*lo, &mut push);
                check(*hi, &mut push);
            }
            SlotAspect::Free => {
                if !i.aspects.is_empty() {
                    push(format!("{k} art has a set aspect ({}): name it", allowed()));
                }
            }
        }
        let unit = |v: f32| (0.0..=1.0).contains(&v);
        if !unit(self.focal[0]) || !unit(self.focal[1]) {
            push("the focal point is 0 to 1 on each axis, from the top left".into());
        }
        if let Some([x0, y0, x1, y1]) = self.safe {
            if ![x0, y0, x1, y1].into_iter().all(unit) || x0 >= x1 || y0 >= y1 {
                push("the safe rectangle is x0 y0 x1 y1 inside 0 to 1, x0 < x1 and y0 < y1".into());
            } else if (x0..=x1).contains(&self.focal[0]) && (y0..=y1).contains(&self.focal[1]) {
                push(
                    "the focal point lies in the safe rectangle, under the text or controls: move it out, or move the rectangle"
                        .into(),
                );
            }
        }
        if self.fit == Fit::NineSlice && !i.nine_slice {
            push(format!(
                "{k} art is not a nine-slice: fit it cover or contain"
            ));
        }
        if i.nine_slice {
            match self.insets {
                None => push("a nine-slice declares its insets (left top right bottom, fractions of the art)".into()),
                Some([l, t, r, b]) => {
                    if [l, t, r, b].iter().any(|v| *v <= 0.0) || l + r >= 1.0 || t + b >= 1.0 {
                        push("the insets are above 0 and leave a middle (left + right and top + bottom under 1)".into());
                    }
                }
            }
        } else if self.insets.is_some() {
            push(format!("{k} art is not a nine-slice: remove its insets"));
        }
        if self.fit == Fit::Tile && !i.tiling {
            push(format!("{k} art does not tile: fit it cover or contain"));
        }
        for s in &self.states {
            if !i.states.contains(&s.as_str()) {
                push(if i.states.is_empty() {
                    format!("{k} art has no states: remove '{s}'")
                } else {
                    format!("'{s}' is not a state of {k} ({})", i.states.join(", "))
                });
            }
        }
        if i.states.contains(&"normal") && !self.states.iter().any(|s| s == "normal") {
            push("a style button draws its normal state".into());
        }
        match (self.kind, self.cells) {
            (SlotKind::ItemIcon, Some(c)) => {
                if !ITEM_CELLS.contains(&c) {
                    push(format!(
                        "an item is 1×1, 1×2, 2×3 or 2×4 cells, not {}×{}",
                        c[0], c[1]
                    ));
                } else if let SlotAspect::One(x) = self.aspect
                    && !x.same(Aspect::new(c[0].into(), c[1].into()))
                {
                    push(format!(
                        "a {}×{} item is drawn at {}:{}, not {x}",
                        c[0], c[1], c[0], c[1]
                    ));
                }
            }
            (SlotKind::ItemIcon, None) => push("an item icon names its cells (1×1 for one)".into()),
            (_, Some(_)) => push(format!("{k} art has no cells: remove them")),
            _ => {}
        }
        if let Mask::Round(r) = self.mask
            && r < 0.0
        {
            push("a round mask's radius is 0 or more".into());
        }
        if self.mask == Mask::Alpha && !i.transparent {
            push(format!(
                "{k} art is opaque: it has no alpha to mask by; use a shape mask"
            ));
        }
        out
    }
}

/// How a kind's placeholder is drawn ([`SlotKind::placeholder`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placeholder {
    /// The gradient's two palette roles, top to bottom.
    pub from: &'static str,
    pub to: &'static str,
    /// The silhouette: an icon of the style's set (the standard names).
    pub icon: &'static str,
}

impl SlotKind {
    /// Its placeholder: a scene kind (key art, backgrounds, banners) a
    /// deep wash under a star, a character kind its accent wash under a
    /// person, an item kind the well under its object.
    pub fn placeholder(self) -> Placeholder {
        use SlotKind as K;
        let (from, to, icon) = match self {
            K::KeyArt | K::MenuBackground | K::EventCg | K::Capsule => {
                ("primarySoft", "well", "star")
            }
            K::Banner | K::PlayerBanner => ("primarySoft", "panel", "flag"),
            K::MenuPattern => ("well", "panel", "leaf"),
            K::Logo => ("panel", "well", "crown"),
            K::AppIcon => ("primarySoft", "panelRaised", "star"),
            K::CharacterSplash | K::FullBody | K::Bust | K::VnSprite | K::UnitPortrait => {
                ("primarySoft", "well", "person")
            }
            K::ItemIcon => ("well", "panelRaised", "gem"),
            K::AbilityIcon | K::TalentIcon => ("primarySoft", "well", "bolt"),
            K::CardArt => ("primarySoft", "well", "sword"),
            K::CardFrame | K::StyleFrame | K::StyleButton | K::Ornament => {
                ("panelRaised", "well", "shield")
            }
            K::LevelThumb | K::MapGlyph => ("well", "panel", "map"),
            K::SongJacket => ("primarySoft", "well", "music"),
            K::SaveThumb => ("well", "panel", "clock"),
            K::Emblem => ("panelRaised", "well", "shield"),
            K::Achievement => ("primarySoft", "panelRaised", "trophy"),
            K::Emote => ("primarySoft", "panelRaised", "heart"),
            K::Currency => ("well", "panelRaised", "coin"),
            K::Tutorial => ("well", "panel", "key"),
            K::CosmeticPreview => ("well", "panelRaised", "person"),
            K::IconSet => ("well", "panelRaised", "gear"),
        };
        Placeholder { from, to, icon }
    }
}

/// An `art` element a document declares: where, its spec as written
/// (a bound prop is taken at the kind's default) and whether its texture
/// is set (bound or given) or it still draws its placeholder.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentSlot {
    /// `<document>/<ids…>`, as the probe names the element.
    pub path: String,
    pub line: usize,
    pub spec: SlotSpec,
    pub textured: bool,
}

/// Every `art` element of `doc`, in document order, with its spec read
/// from its literal props ([`SlotSpec::from_props`]).
pub fn document_slots(doc: &crate::doc::UiDocument) -> Vec<DocumentSlot> {
    use crate::doc::{NodeKind, PropValue};
    let mut out = Vec::new();
    for (i, n) in doc.nodes.iter().enumerate() {
        if n.kind != NodeKind::Element("art") {
            continue;
        }
        let mut ids = Vec::new();
        let mut j = i;
        while j < doc.nodes.len() {
            let m = &doc.nodes[j];
            if !m.id.is_empty() {
                ids.push(m.id.clone());
            }
            j = m.parent;
        }
        ids.push(doc.name.clone());
        ids.reverse();
        let path = ids.join("/");
        let lit = |name: &str| -> Option<Vec<String>> {
            let p = n.props.iter().rev().find(|p| p.name() == name)?;
            match p.value {
                PropValue::Lit(_) => Some(
                    p.source
                        .split_whitespace()
                        .map(|w| w.trim_matches('"').to_string())
                        .collect(),
                ),
                _ => None,
            }
        };
        let nums = |name: &str| -> Option<Vec<f32>> {
            lit(name)?.iter().map(|w| w.parse().ok()).collect()
        };
        let spec = SlotSpec::from_props(&path, |p| lit(p)?.into_iter().next(), nums);
        out.push(DocumentSlot {
            path,
            line: n.line,
            spec,
            textured: n.props.iter().any(|p| p.name() == "texture"),
        });
    }
    out
}
