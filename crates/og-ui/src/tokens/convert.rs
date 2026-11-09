use super::*;

impl StyleTokens {
    pub fn space(&self, s: SpaceStep) -> f32 {
        self.space[s.index()]
    }
    pub fn size(&self, s: SizeStep) -> f32 {
        self.size[s.index()]
    }
    pub fn radius(&self, s: RadiusStep) -> f32 {
        self.radius[s.index()]
    }
    pub fn stroke(&self, s: StrokeWidth) -> f32 {
        self.stroke[s.index()]
    }
    pub fn elevation(&self, l: Level) -> &Elevation {
        &self.elevation[l.index()]
    }
    pub fn shape(&self, c: ElementClass) -> ShapeToken {
        self.shape[c.index()]
    }
    pub fn spring(&self, s: SpringName) -> SpringToken {
        self.motion.springs[s.index()]
    }
    pub fn ease(&self, e: EaseRole) -> Ease {
        self.motion.eases[e.index()]
    }
    pub fn feedback(&self, e: SoundEvent) -> &Feedback {
        &self.feedback[e.index()]
    }
    pub fn rarity(&self, r: Rarity) -> Swatch {
        self.colors.rarity[r.index()]
    }

    /// A complete set derived from a theme's v1 values: its palette and
    /// scheme (the colours, read on its panel), its radii (the shape
    /// language), its ramp (the type scale), its durations and the motion
    /// and shadows of its art. `icons` are the style's icon names (a status
    /// takes an icon cue when the set has a fitting one, a pattern
    /// otherwise). The default style's tokens are this of its palette;
    /// `og style upgrade` writes this for a version 1 style.
    pub fn derive(t: &Theme, icons: &[String]) -> StyleTokens {
        StyleTokens::derive_from(
            V1 {
                palette: &t.palette,
                space: &t.space,
                radius: &t.radius,
                type_ramp: &t.type_ramp,
                durations_ms: t.durations_ms,
                art: &t.art,
            },
            icons,
        )
    }

    pub(crate) fn derive_from(t: V1, icons: &[String]) -> StyleTokens {
        let p = t.palette;
        let ground = p.panel_raised.over(p.backdrop);
        // A non-text colour reads 3:1 on the panel (WCAG 1.4.11): the seed
        // keeps its hue and moves along its own lightness until it does.
        let read = |hex: &str| {
            let c = parse_color(hex).expect("a seed colour");
            c.readable_on(ground, 3.0).unwrap_or(p.ink)
        };
        let icon = |name: &str, or: Pattern| {
            icons
                .iter()
                .position(|n| n == name)
                .map_or(Cue::Pattern(or), |i| Cue::Icon(i as u16))
        };
        let swatch = |hex: &str, cue: Cue| Swatch {
            color: read(hex),
            cue,
        };
        let colors = ColorTokens {
            status: [
                icon("check", Pattern::Dots),
                icon("bolt", Pattern::Stripes),
                icon("close", Pattern::Crosshatch),
                Cue::Pattern(Pattern::Waves),
            ],
            rarity: [
                swatch("#9e9e9e", Cue::Pips(1)),
                swatch("#3fbf4f", Cue::Pips(2)),
                swatch("#2f7fef", Cue::Pips(3)),
                swatch("#a335ee", Cue::Pips(4)),
                swatch("#ff8000", Cue::Pips(5)),
                swatch("#e8364f", Cue::Pips(6)),
            ],
            team: [
                "#3d7eff", "#ff4747", "#3fbf5f", "#ffc61a", "#a347ff", "#ff8a1f", "#1fc7c7",
                "#ff5cb0",
            ]
            .map(|h| swatch(h, Cue::None)),
            damage: [
                "#b8b0a0", "#ff6a2b", "#62cfff", "#ffd83a", "#7ed957", "#ffe9a0", "#8a5cf6",
                "#ff4fd8",
            ]
            .map(|h| swatch(h, Cue::None)),
            series: [
                "#4e79a7", "#f28e2b", "#e15759", "#76b7b2", "#59a14f", "#edc948", "#b07aa1",
                "#ff9da7",
            ]
            .map(read),
        };
        let r = t.type_ramp;
        let step = |s: TypeStep, weight: u16, font: FontRole, phone: f32| {
            let desk = TypeToken {
                size: s.size,
                line_height: s.line_height,
                tracking_em: s.tracking_em,
                cjk_tracking_em: s.cjk_tracking_em,
                weight,
                font,
                features: s.features,
                uppercase: s.uppercase,
            };
            let small = |k: f32| TypeToken {
                size: (s.size * k).round().max(12.0),
                line_height: (s.line_height * k).round().max(16.0),
                ..desk
            };
            // phonePortrait, phoneLandscape, tablet, desktop.
            [small(phone), small((1.0 + phone) / 2.0), desk, desk]
        };
        let typography = TypeScale {
            steps: [
                step(r.display_lg, 700, FontRole::Display, 0.8),
                step(r.display_sm, 700, FontRole::Display, 0.875),
                step(r.xl, 600, FontRole::Display, 1.0),
                step(r.body, 400, FontRole::Body, 1.0),
                // A button's and a tab's label: version 1 set them in body.
                step(r.body, 600, FontRole::Body, 1.0),
                step(r.caption, 400, FontRole::Body, 1.0),
                step(r.lg, 600, FontRole::Mono, 1.0),
                step(r.body, 600, FontRole::Body, 1.0),
                step(r.display_xl2, 700, FontRole::Display, 0.75),
            ],
        };
        let s = t.space;
        let rad = t.radius;
        let shadow = |dy: f32, blur: f32, a: f32| Shadow {
            dx: 0.0,
            dy,
            blur,
            spread: 0.0,
            // Quantised to a byte, as a file writes it.
            color: parse_color(&hex(p.shadow.fade(a))).expect("a colour"),
        };
        let level = |s: Option<Shadow>, or: Shadow| {
            let mut e = Elevation::FLAT;
            e.shadows[0] = s.unwrap_or(or);
            e.count = 1;
            e
        };
        let art = t.art;
        let overlay = level(
            art.shadows[StylePart::Panel.index()],
            shadow(12.0, 32.0, 0.45),
        );
        let mut top = overlay;
        top.shadows[0].dy = (top.shadows[0].dy * 1.5).round();
        top.shadows[0].blur = (top.shadows[0].blur * 1.5).round();
        let elevation = [
            level(art.shadows[StylePart::Card.index()], shadow(1.0, 3.0, 0.3)),
            level(
                art.shadows[StylePart::Tooltip.index()].or(art.shadows[StylePart::Hud.index()]),
                shadow(4.0, 12.0, 0.35),
            ),
            overlay,
            top,
        ];
        let corner = |radius: f32| ShapeToken {
            corner: if radius <= 0.0 {
                CornerShape::Square
            } else {
                CornerShape::Round
            },
            radius,
        };
        let shape = ElementClass::map_all(|c| {
            corner(match c {
                ElementClass::Panel
                | ElementClass::Dialog
                | ElementClass::Card
                | ElementClass::Tooltip => rad.panel,
                ElementClass::Button
                | ElementClass::Field
                | ElementClass::Chip
                | ElementClass::Badge
                | ElementClass::Tab
                | ElementClass::Hud => rad.control,
                ElementClass::Slot | ElementClass::Avatar => rad.small,
                ElementClass::Bar => rad.bar,
            })
        });
        let d = t.durations_ms;
        let springs = [
            SpringToken {
                duration_ms: d.fast,
                bounce: 0.0,
            },
            SpringToken {
                duration_ms: d.base,
                bounce: 0.0,
            },
            SpringToken {
                duration_ms: d.slow,
                bounce: 0.15,
            },
            SpringToken {
                duration_ms: d.spring,
                bounce: 0.3,
            },
        ];
        let mut overshoot = springs
            .iter()
            .map(|s| s.overshoot())
            .chain(
                [art.motion.hover_spring, art.motion.press_spring]
                    .into_iter()
                    .flatten()
                    .map(|s| SpringToken::from_spring(s).overshoot()),
            )
            .fold(0.0f32, f32::max);
        overshoot = ((overshoot * 20.0).ceil() / 20.0).min(1.0);
        let motion = MotionTokens {
            springs,
            eases: [
                Ease::CubicBezier([0.2, 0.0, 0.0, 1.0]),
                Ease::CubicBezier(crate::theme::EASE_OUT),
                Ease::CubicBezier(crate::theme::EASE_EXIT),
                Ease::CubicBezier([0.34, 1.4, 0.64, 1.0]),
                Ease::CubicBezier(crate::theme::EASE_OUT),
                Ease::CubicBezier(crate::theme::EASE_EXIT),
            ],
            durations: d,
            stagger_ms: art.motion.stagger_ms,
            stagger_cap_ms: 300.0,
            overshoot,
        };
        let haptic = |e: SoundEvent| {
            use HapticPreset as H;
            use SoundEvent as E;
            match e {
                E::Press | E::Select | E::Cancel | E::Back | E::Tab => Some(H::Tick),
                E::ToggleOn | E::ToggleOff | E::SliderTick | E::RevealUncommon => Some(H::Tick),
                E::Confirm | E::Notification | E::RevealRare => Some(H::Tap),
                E::Warning => Some(H::Warning),
                E::Error => Some(H::Error),
                E::Success => Some(H::Success),
                E::Reward | E::RevealEpic => Some(H::Double),
                E::RevealLegendary => Some(H::Thud),
                E::RevealMythic => Some(H::Buzz),
                _ => None,
            }
            .map_or(Haptic::NONE, Haptic::preset)
        };
        let feedback = SoundEvent::map_all(|e| Feedback {
            haptic: haptic(e),
            ..Feedback::SILENT
        });
        StyleTokens {
            colors,
            typography,
            space: [s.xxs, s.xs, s.sm, s.md, s.lg, s.xl, s.xxl, s.xl3],
            size: [16.0, 24.0, 32.0, 48.0, 64.0, 96.0, 44.0],
            radius: [
                rad.small,
                rad.control,
                rad.panel,
                (rad.panel * 1.5).round(),
                (rad.panel * 2.0).round(),
                9999.0,
            ],
            stroke: [1.0, 1.5, 2.0, 3.0, 2.0],
            elevation,
            shape,
            motion,
            feedback,
        }
    }
}

macro_rules! map_all {
    ($($ty:ident),*) => {$(
        impl $ty {
            /// `f` of every value, in order.
            pub fn map_all<U>(mut f: impl FnMut($ty) -> U) -> [U; $ty::COUNT] {
                std::array::from_fn(|i| f($ty::ALL[i]))
            }
        }
    )*};
}
map_all!(ElementClass, SoundEvent);

/// The version 1 values a token set is derived from.
pub(crate) struct V1<'a> {
    pub palette: &'a crate::Palette,
    pub space: &'a crate::Space,
    pub radius: &'a crate::Radius,
    pub type_ramp: &'a crate::TypeRamp,
    pub durations_ms: Durations,
    pub art: &'a crate::Art,
}

// ───────────────────────────────────────────────────────────── writing

pub(crate) fn num_value(v: f32) -> Value {
    num(v)
}

fn num(v: f32) -> Value {
    Value::Number(Number::from_f32(round_to(v, 1000.0)).expect("a finite token"))
}

fn word(w: &str) -> Value {
    Value::Word(w.to_string())
}

/// `#rrggbb` or `#rrggbbaa`.
pub fn hex(c: Srgba) -> String {
    let [r, g, b] = c.rgb;
    if c.alpha >= 1.0 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!(
            "#{r:02x}{g:02x}{b:02x}{:02x}",
            (c.alpha * 255.0).round() as u8
        )
    }
}

fn color(c: Srgba) -> Value {
    Value::String(hex(c))
}

fn line(key: &str, values: Vec<Value>) -> Entry {
    Entry {
        key: key.to_string(),
        values,
        block: None,
        line: 0,
    }
}

fn block(key: &str, values: Vec<Value>, entries: Vec<Entry>) -> Entry {
    Entry {
        key: key.to_string(),
        values,
        block: Some(Block { entries }),
        line: 0,
    }
}

fn cue_values(cue: Cue, icons: &[String]) -> Vec<Value> {
    match cue {
        Cue::None => vec![],
        Cue::Icon(i) => vec![
            word("icon"),
            Value::String(icons.get(usize::from(i)).cloned().unwrap_or_default()),
        ],
        Cue::Pattern(x) => vec![word("pattern"), word(x.name())],
        Cue::Pips(n) => vec![word("pips"), num(f32::from(n))],
    }
}

pub(crate) fn shadow_values(s: &Shadow) -> Vec<Value> {
    vec![
        num(s.dx),
        num(s.dy),
        num(s.blur),
        num(s.spread),
        color(s.color),
    ]
}

impl StyleTokens {
    /// The tokens as the blocks of a version 2 style file (`colors`,
    /// `typography`, `space`, `size`, `radius`, `stroke`, `elevation`,
    /// `shape`), and the token lines of its `motion` block and its
    /// `sounds` block given each event's sound ids.
    pub fn entries(
        &self,
        icons: &[String],
        sounds: &[Vec<String>; SoundEvent::COUNT],
    ) -> (Vec<Entry>, Vec<Entry>, Entry) {
        let c = &self.colors;
        let mut colors = Vec::new();
        for s in Status::ALL {
            colors.push(line(s.name(), cue_values(c.status[s.index()], icons)));
        }
        let sw = |k: &str, s: &Swatch| {
            let mut v = vec![color(s.color)];
            v.extend(cue_values(s.cue, icons));
            line(k, v)
        };
        for r in Rarity::ALL {
            colors.push(sw(r.name(), &c.rarity[r.index()]));
        }
        for (k, s) in c.team.iter().enumerate() {
            colors.push(sw(&format!("team{}", k + 1), s));
        }
        for d in DamageType::ALL {
            colors.push(sw(d.name(), &c.damage[d.index()]));
        }
        for (k, s) in c.series.iter().enumerate() {
            colors.push(line(&format!("series{}", k + 1), vec![color(*s)]));
        }
        let mut typo = Vec::new();
        for r in TypeRole::ALL {
            let at = self.typography.steps[r.index()];
            // A button at its label's step is the default: written only when it differs.
            if *r == TypeRole::Button && at == self.typography.steps[TypeRole::Label.index()] {
                continue;
            }
            let base = at[Breakpoint::Desktop as usize];
            // A token's step names a role, never a document's family.
            let font = match base.font.base() {
                FontRole::Display => "display",
                FontRole::Mono => "numeric",
                FontRole::Rounded => "rounded",
                _ => "body",
            };
            let overrides: Vec<Entry> = Breakpoint::ALL
                .iter()
                .filter(|b| {
                    let t = at[**b as usize];
                    (t.size, t.line_height) != (base.size, base.line_height)
                })
                .map(|b| {
                    let t = at[*b as usize];
                    line(b.name(), vec![num(t.size), num(t.line_height)])
                })
                .collect();
            let mut values = vec![
                num(base.size),
                num(base.line_height),
                num(base.tracking_em),
                num(f32::from(base.weight)),
                word(font),
            ];
            values.extend(base.features.tags().map(word));
            if base.cjk_tracking_em != 0.0 {
                values.extend([word("cjkTracking"), num(base.cjk_tracking_em)]);
            }
            if base.uppercase {
                values.extend([word("case"), word("upper")]);
            }
            typo.push(if overrides.is_empty() {
                line(r.name(), values)
            } else {
                block(r.name(), values, overrides)
            });
        }
        let lengths = |key: &str, names: &[&str], v: &[f32]| {
            block(
                key,
                vec![],
                names
                    .iter()
                    .zip(v)
                    .map(|(n, v)| line(n, vec![num(*v)]))
                    .collect(),
            )
        };
        let elevation = Level::ALL
            .iter()
            .map(|l| {
                let e = &self.elevation[l.index()];
                if e.count == 0 {
                    line(l.name(), vec![word("none")])
                } else {
                    block(
                        l.name(),
                        vec![],
                        e.shadows()
                            .iter()
                            .map(|s| line("shadow", shadow_values(s)))
                            .collect(),
                    )
                }
            })
            .collect();
        let shape = ElementClass::ALL
            .iter()
            .map(|k| {
                let s = self.shape[k.index()];
                line(k.name(), vec![word(s.corner.name()), num(s.radius)])
            })
            .collect();
        let tokens = vec![
            block("colors", vec![], colors),
            block("typography", vec![], typo),
            lengths(
                "space",
                &names_of(SpaceStep::ALL, SpaceStep::name),
                &self.space,
            ),
            lengths("size", &names_of(SizeStep::ALL, SizeStep::name), &self.size),
            lengths(
                "radius",
                &names_of(RadiusStep::ALL, RadiusStep::name),
                &self.radius,
            ),
            lengths(
                "stroke",
                &names_of(StrokeWidth::ALL, StrokeWidth::name),
                &self.stroke,
            ),
            block("elevation", vec![], elevation),
            block("shape", vec![], shape),
        ];
        let m = &self.motion;
        let mut motion = Vec::new();
        for s in SpringName::ALL {
            let t = m.springs[s.index()];
            motion.push(line(
                "spring",
                vec![word(s.name()), num(t.duration_ms), num(t.bounce)],
            ));
        }
        for n in EaseRole::ALL {
            let mut v = vec![word(n.name())];
            match m.eases[n.index()] {
                Ease::Linear => v.push(word("linear")),
                Ease::CubicBezier(b) => {
                    v.extend(b.map(|x| {
                        Value::Number(Number::from_f32(x).expect("finite timing controls"))
                    }))
                }
                ease => v.push(Value::String(ease.to_text())),
            }
            motion.push(line("ease", v));
        }
        let d = m.durations;
        for (n, v) in DURATIONS
            .iter()
            .zip([d.instant, d.fast, d.base, d.slow, d.spring, d.bounce])
        {
            motion.push(line("duration", vec![word(n), num(v)]));
        }
        motion.push(line("stagger", vec![num(m.stagger_ms)]));
        motion.push(line("staggerCap", vec![num(m.stagger_cap_ms)]));
        motion.push(line("overshoot", vec![num(m.overshoot)]));
        let feedback = SoundEvent::ALL
            .iter()
            .map(|e| {
                let f = &self.feedback[e.index()];
                let ids = &sounds[e.index()];
                let mut v: Vec<Value> = if ids.is_empty() {
                    vec![word("none")]
                } else {
                    ids.iter().map(|s| Value::String(s.clone())).collect()
                };
                if f.volume != [1.0, 1.0] {
                    v.extend([word("volume"), num(f.volume[0]), num(f.volume[1])]);
                }
                if f.pitch != [1.0, 1.0] {
                    v.extend([word("pitch"), num(f.pitch[0]), num(f.pitch[1])]);
                }
                v.push(word("haptic"));
                match HapticPreset::ALL
                    .iter()
                    .find(|h| Haptic::preset(**h) == f.haptic)
                {
                    _ if f.haptic.count == 0 => v.push(word("none")),
                    Some(h) => v.push(word(h.name())),
                    None => {
                        for p in f.haptic.pulses() {
                            v.extend([num(p.start_ms), num(p.ms), num(p.low), num(p.high)]);
                        }
                    }
                }
                line(e.name(), v)
            })
            .collect();
        (tokens, motion, block("sounds", vec![], feedback))
    }
}
