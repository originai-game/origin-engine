//! Juice: number count-ups, bursts, shakes and screen transitions. Each is a call
//! per tick whose progress lives in the UI state (ticks counted by the UI,
//! never a clock), so snapshots and replays carry it; `reduced_motion`
//! turns movement into fades (and bursts off).

use crate::draw::DrawCmd;
use crate::id::{Id, child_key};
use crate::layout::{Content, FontRole, NONE, Rect, Size, TextAlign};
use crate::motion::Fx;
use crate::style::{Srgba, TypeStep};
use crate::ui::{Anim, Ui};
use og_draw::TextureHandle;

/// A shake of the element built just before (a wrong answer, a locked door):
/// three decaying swings, as a refused press shakes a button.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShakeDesc {
    /// The first swing, CSS px (0: the style's `motion.shake`, else 6).
    pub amount: f32,
    /// 0: `motion.duration.slow`.
    pub duration_ms: f32,
    /// Also queue the style's error sound, as a refused press does.
    pub sound: bool,
}

/// The tag of a shake's memory (`Anim::h`), so [`Ui::end`] finds them.
pub(crate) const SHAKE: u64 = 0x5348_414b_4500_0001;

/// A number that counts to its value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CounterDesc {
    /// `None`: `text-body`.
    pub step: Option<TypeStep>,
    /// The numeric role (`Mono`) suits counts in a row; a big score takes `Display`.
    pub font: FontRole,
    /// `None`: `ink`.
    pub color: Option<Srgba>,
    pub align: TextAlign,
    pub width: Size,
    /// How long a change takes to count; 0: 900 ms.
    pub duration_ms: f32,
    /// Where the first count starts (`None`: at the value, no count).
    pub from: Option<f64>,
    /// Digits after the point.
    pub decimals: u32,
    pub prefix: String,
    pub suffix: String,
}

/// Rays and sparks thrown from the element built just before.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BurstDesc {
    /// Rays (0: 8).
    pub count: u32,
    /// `None`: the accent.
    pub color: Option<Srgba>,
    /// How far they fly from the element's centre, CSS px (0: 1.4 × its size).
    pub radius: f32,
    /// 0: `motion.duration.slow` + `base`.
    pub duration_ms: f32,
    /// Waits this long after the element first appears.
    pub delay_ms: f32,
}

/// How a screen transition covers and reveals.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScreenWipe {
    #[default]
    Fade,
    /// An edge sweeps across (from the left, or `from`).
    Wipe,
    /// A circle closes on the centre and opens again.
    Iris,
    /// Cells appear in a fixed scattered order.
    Dissolve,
    /// A sheet slides over and away.
    Slide,
}

/// Where a wipe or slide comes from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScreenTransitionDesc {
    pub kind: ScreenWipe,
    /// Cover and reveal together; 0: `motion.duration.slow` × 2.
    pub duration_ms: f32,
    /// `None`: `backdrop`.
    pub color: Option<Srgba>,
    /// Covers with a texture region `[u0, v0, u1, v1]` stretched over the
    /// surface (tinted by `color`) instead of a flat colour.
    pub texture: Option<(TextureHandle, [f32; 4])>,
    pub from: Side,
}

/// What the frame's cover draws (after everything else).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Cover {
    pub kind: ScreenWipe,
    pub from: Side,
    /// Eased coverage, 0–1.
    pub e: f32,
    /// Revealing (the second half).
    pub revealing: bool,
    pub color: Srgba,
    pub texture: Option<(TextureHandle, [f32; 4])>,
}

/// `12,840` with `decimals` digits after the point.
pub fn format_count(v: f64, decimals: u32) -> String {
    let neg = v < 0.0;
    let scale = 10f64.powi(decimals as i32);
    let r = (v.abs() * scale).round() as u128;
    let (int, frac) = (r / scale as u128, r % scale as u128);
    let digits = int.to_string();
    let mut grouped = String::new();
    for (k, c) in digits.chars().enumerate() {
        if k > 0 && (digits.len() - k) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(c);
    }
    let mut out = String::new();
    if neg && r > 0 {
        out.push('-');
    }
    out.push_str(&grouped);
    if decimals > 0 {
        out.push('.');
        out.push_str(&format!("{:0width$}", frac, width = decimals as usize));
    }
    out
}

fn smooth(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    c * c * (3.0 - 2.0 * c)
}

/// A fixed scatter in 0–1 for cell `(x, y)` (an integer hash).
fn scatter(x: u32, y: u32) -> f32 {
    let mut h = x.wrapping_mul(0x9e37_79b9) ^ y.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 65536.0
}

impl Ui {
    /// A number that counts from where it was to `value` (ease-out cubic
    /// over `duration_ms`) each time `value` changes, and pops when it lands
    /// if the style says so. Returns the number shown this tick.
    pub fn counter(&mut self, id: impl Into<Id>, value: f64, desc: &CounterDesc) -> f64 {
        let t = self.env.theme;
        let value = if value.is_finite() { value } else { 0.0 };
        let i = self.leaf(
            "label",
            id.into(),
            crate::Element {
                width: desc.width,
                ..crate::Element::default()
            },
        );
        let key = self.tree.nodes[i].key;
        let (kf, kt) = (child_key(key, "count"), child_key(key, "to"));
        let now = self.interaction.state.tick;
        let fresh = !self.has_anim(kf);
        let mut from = self.anim(kf);
        let mut to = self.anim(kt);
        let ms = if desc.duration_ms > 0.0 {
            desc.duration_ms
        } else {
            900.0
        };
        let secs = f64::from(ms) / 1000.0;
        let tps = f64::from(self.env.ticks_per_second);
        let at = |from: &Anim, to: &Anim| -> f64 {
            let (a, b) = (f64::from_bits(from.h), f64::from_bits(to.h));
            let k = ((now - from.t) as f64 / tps / secs).clamp(0.0, 1.0);
            a + (b - a) * (1.0 - (1.0 - k) * (1.0 - k) * (1.0 - k))
        };
        if fresh {
            from = Anim {
                h: desc.from.unwrap_or(value).to_bits(),
                t: now,
                ..Anim::default()
            };
            to = Anim {
                h: value.to_bits(),
                ..Anim::default()
            };
        } else if f64::from_bits(to.h) != value {
            from = Anim {
                h: at(&from, &to).to_bits(),
                t: now,
                ..Anim::default()
            };
            to.h = value.to_bits();
        }
        self.set_anim(kf, from);
        self.set_anim(kt, to);
        let shown = at(&from, &to);
        let shown = if desc.decimals == 0 {
            shown.round()
        } else {
            shown
        };
        let text = format!(
            "{}{}{}",
            desc.prefix,
            format_count(shown, desc.decimals),
            desc.suffix
        );
        let n = &mut self.tree.nodes[i];
        n.content = Content::Text(crate::layout::TextContent {
            text: text.clone(),
            step: desc.step.unwrap_or(t.type_ramp.body),
            color: desc.color.unwrap_or(t.palette.ink),
            align: desc.align,
            wrap: false,
            font: desc.font,
            ellipsis: false,
            max_lines: 0,
            emphasis: 0.0,
            fit: Vec::new(),
            wrap_style: Default::default(),
            cut: None,
            vertical: None,
        });
        n.probe_text = text;
        // The landing pop: a bump of `pop` over `motion.duration.fast`.
        let pop = t.art.motion.pop;
        if pop > 0.0 && !t.reduced_motion && f64::from_bits(from.h) != value {
            let end = from.t as f64 + secs * tps;
            let b = (now as f64 - end) / f64::from(self.ticks_for(t.durations_ms.fast));
            if (0.0..1.0).contains(&b) {
                let b = b as f32;
                self.tree.nodes[i].fx = Some(Fx {
                    scale: 1.0 + pop * 4.0 * b * (1.0 - b),
                    ..Fx::default()
                });
            }
        }
        shown
    }

    /// Shakes the element built just before from this tick on: it swings
    /// sideways three times, decaying to rest over the duration, with no
    /// further call (the shake lives in the UI state, so snapshots carry
    /// it). A second shake restarts it. Still under reduced motion (the
    /// sound, when asked, still plays).
    pub fn shake(&mut self, desc: ShakeDesc) {
        let anchor = self.tree.last_closed;
        if anchor == NONE {
            self.out.warnings.push(
                "shake with no element before it: call ui.shake() right after the element".into(),
            );
            return;
        }
        let t = self.env.theme;
        let amount = if desc.amount > 0.0 {
            desc.amount
        } else if t.art.motion.shake > 0.0 {
            t.art.motion.shake
        } else {
            6.0
        };
        let ms = if desc.duration_ms > 0.0 {
            desc.duration_ms
        } else {
            t.durations_ms.slow
        };
        let key = child_key(self.tree.nodes[anchor].key, "shake");
        let ticks = self.ticks_for(ms) as f32;
        self.set_anim(
            key,
            Anim {
                a: amount,
                b: ticks,
                t: self.interaction.state.tick,
                h: SHAKE,
            },
        );
        if desc.sound {
            self.sound(crate::style_file::UiSound::Error);
        }
    }

    /// Moves every element with a shake in progress (called by `end`
    /// before the transitions apply): the swing of this tick, kept until
    /// it has come to rest.
    pub(crate) fn apply_shakes(&mut self, order: &[usize]) {
        if !self.interaction.state.anims.values().any(|a| a.h == SHAKE) {
            return;
        }
        let now = self.interaction.state.tick;
        let still = self.env.theme.reduced_motion;
        for &i in order {
            let key = child_key(self.tree.nodes[i].key, "shake");
            let Some(&a) = self
                .interaction
                .state
                .anims
                .get(&key)
                .filter(|a| a.h == SHAKE)
            else {
                continue;
            };
            let k = (now - a.t) as f32 / a.b.max(1.0);
            if k >= 1.0 {
                continue;
            }
            self.front.touched.insert(key);
            if still {
                continue;
            }
            // Three decaying swings, arithmetic only (a triangle wave).
            let x = k * 3.0;
            let tri = 4.0 * (x - x.floor() - 0.5).abs() - 1.0;
            let mut f = self.tree.nodes[i].fx.unwrap_or_default();
            f.dx += a.a * (1.0 - k) * tri;
            self.tree.nodes[i].fx = Some(f);
        }
    }

    /// Covers the whole frame by `amount` (0 clear to 1 covered) this frame,
    /// with no screen state: for a caller that times the cover itself (a timeline's transition
    /// track, a function of its time). `revealing` draws the second half's
    /// shape (a wipe leaving past the far side). Reduced motion fades.
    pub fn cover(&mut self, desc: &ScreenTransitionDesc, amount: f32, revealing: bool) {
        let amount = if amount.is_nan() {
            0.0
        } else {
            amount.clamp(0.0, 1.0)
        };
        if amount <= 0.0 {
            return;
        }
        let t = self.env.theme;
        self.front.cover = Some(Cover {
            kind: if t.reduced_motion {
                ScreenWipe::Fade
            } else {
                desc.kind
            },
            from: desc.from,
            e: smooth(amount),
            revealing,
            color: desc.color.unwrap_or(t.palette.backdrop),
            texture: desc.texture,
        });
    }

    /// The cover's draw commands over the whole surface.
    pub(crate) fn cover_commands(&self, c: &Cover, cmds: &mut Vec<DrawCmd>) {
        let vp = self.env.viewport;
        let (w, h) = (vp.width, vp.height);
        let screen = Rect::new(0.0, 0.0, w, h);
        let piece = |r: Rect, alpha: f32, cmds: &mut Vec<DrawCmd>| {
            let r = r.intersect(&screen);
            if r.is_empty() || alpha <= 0.0 {
                return;
            }
            match c.texture {
                Some((tex, [u0, v0, u1, v1])) => {
                    let u = |x: f32| u0 + (u1 - u0) * x / w;
                    let v = |y: f32| v0 + (v1 - v0) * y / h;
                    cmds.push(DrawCmd::Image {
                        rect: r,
                        texture: Some(tex),
                        uv: [u(r.x), v(r.y), u(r.right()), v(r.bottom())],
                        color: c.color.fade(alpha).linear(),
                        clip: screen,
                    });
                }
                None => cmds.push(DrawCmd::Quad {
                    rect: r,
                    radius: 0.0,
                    color: c.color.fade(alpha).linear(),
                    clip: screen,
                }),
            }
        };
        let e = c.e;
        match c.kind {
            ScreenWipe::Fade => piece(screen, e, cmds),
            ScreenWipe::Wipe | ScreenWipe::Slide => {
                // Covering grows from the side; revealing leaves past the far side.
                // A slide moves a whole sheet; a wipe moves an edge.
                let (along, total) = match c.from {
                    Side::Left | Side::Right => (true, w),
                    Side::Top | Side::Bottom => (false, h),
                };
                let size = total * e;
                let start = match (c.kind, c.revealing) {
                    (ScreenWipe::Wipe, false) => 0.0,
                    (ScreenWipe::Wipe, true) => total - size,
                    (_, false) => size - total,
                    (_, true) => total - size,
                };
                let len = if c.kind == ScreenWipe::Wipe {
                    size
                } else {
                    total
                };
                let flip = matches!(c.from, Side::Right | Side::Bottom);
                let s0 = if flip { total - start - len } else { start };
                let r = if along {
                    Rect::new(s0, 0.0, len, h)
                } else {
                    Rect::new(0.0, s0, w, len)
                };
                piece(r, 1.0, cmds);
            }
            ScreenWipe::Iris => {
                let (cx, cy) = (w / 2.0, h / 2.0);
                let big = (cx * cx + cy * cy).sqrt() + 2.0;
                let r = big * (1.0 - e);
                let band = 2.0;
                let mut y = 0.0;
                while y < h {
                    let yc = y + band / 2.0;
                    let dy = (yc - cy).abs();
                    if dy >= r {
                        piece(Rect::new(0.0, y, w, band), 1.0, cmds);
                    } else {
                        let half = (r * r - dy * dy).sqrt();
                        piece(Rect::new(0.0, y, cx - half, band), 1.0, cmds);
                        piece(Rect::new(cx + half, y, w - cx - half, band), 1.0, cmds);
                    }
                    y += band;
                }
            }
            ScreenWipe::Dissolve => {
                let cell = 24.0;
                let (cols, rows) = ((w / cell).ceil() as u32, (h / cell).ceil() as u32);
                for y in 0..rows {
                    for x in 0..cols {
                        let a = ((e * 1.25 - scatter(x, y)) * 6.0).clamp(0.0, 1.0);
                        if a > 0.0 {
                            piece(
                                Rect::new(x as f32 * cell, y as f32 * cell, cell, cell),
                                a,
                                cmds,
                            );
                        }
                    }
                }
            }
        }
    }
}

impl Ui {
    /// Bursts a timeline's events started, drawn on their elements.
    pub(crate) fn apply_bursts(&mut self, order: &[usize]) {
        if self.interaction.state.motion.bursts.is_empty() {
            return;
        }
        let bursts = self.interaction.state.motion.bursts.clone();
        for &i in order {
            let key = self.tree.nodes[i].key;
            for b in bursts.iter().filter(|b| b.key == key) {
                let desc = BurstDesc {
                    count: b.count,
                    color: b.color,
                    radius: b.radius,
                    duration_ms: b.duration_ms,
                    ..BurstDesc::default()
                };
                self.burst_particles(i, &desc, b.since);
            }
        }
    }
}
