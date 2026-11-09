//! Shared elements of view transitions (CSS View Transitions 1 and 2,
//! docs/ui.md §2.1). When a screen's layer starts its view transition
//! ([`crate::layers`]), every element with a `view-transition-name` that
//! last frame drew is captured: its draw commands, frozen as they were,
//! are its old image. Each name is a `::view-transition-group`: in the top
//! layer it moves from the old element's box to the new one's (the new
//! element with that name, drawn live each frame), and its image pair
//! cross-fades the two with `plus-lighter`. The live elements taking part
//! are hidden where they stand until their group ends. The groups are
//! styled by the rules of the incoming screen (the outgoing one when only
//! a layer leaves), as [`crate::doc::view_transition::ViewTransition::group`]
//! cascades them.
//!
//! A frame's shared elements are recorded where they draw in its command
//! list (`Ui::commands`), so capturing them costs nothing until a
//! transition starts, and the hidden ones are cut only from the frame's
//! items: the command list and what later frames reuse from it are
//! untouched. Two elements with one name in the old or the new view skip
//! the transition (the spec's `InvalidStateError`), as reduced motion does
//! (unless the style keeps motion).

use crate::doc::view_transition::{FADE_IN, FADE_OUT, GROUP_ANIM, GroupStyle};
use crate::draw::DrawCmd;
use crate::element::{Affine, CHANNELS, Transform};
use crate::layout::{NONE, Rect};
use crate::motion::css::{CssAnimation, Keyframe, KeyframeTrack, Keyframes};
use crate::ui::Ui;
use serde_json::{Value, json};
use std::ops::Range;

/// A shared element as a frame drew it: where its commands are in the
/// frame's list.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Named {
    /// Its node this frame.
    pub node: usize,
    pub key: u64,
    /// Its `view-transition-name` (`auto` and `match-element`: its key's).
    pub name: String,
    pub classes: Vec<String>,
    pub range: Range<usize>,
    /// Its box on the screen.
    pub screen: Rect,
    /// The opacity its ancestors give it (in its commands' alpha): a
    /// snapshot is the element's own drawing, without them.
    pub inherited: f32,
}

/// `cmds` without the opacity `inherited` their ancestors gave them.
fn own(cmds: &[(DrawCmd, Affine)], inherited: f32) -> Vec<(DrawCmd, Affine)> {
    cmds.iter()
        .filter(|(c, _)| !matches!(c, DrawCmd::World(_)))
        .map(|(c, m)| {
            let mut c = c.clone();
            if inherited > 0.0 && inherited < 1.0 {
                crate::ui::fade_cmd(&mut c, 1.0 / inherited);
            }
            (c, *m)
        })
        .collect()
}

/// The name `auto` and `match-element` give an element: its identity,
/// which no `::view-transition-*(name)` can write (only `*` and classes
/// select it).
pub(crate) fn element_name(key: u64) -> String {
    format!("#{key:x}")
}

/// An old element: its frozen drawing, each command with its map onto
/// the screen.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Capture {
    pub key: u64,
    pub name: String,
    pub classes: Vec<String>,
    pub screen: Rect,
    pub cmds: Vec<(DrawCmd, Affine)>,
}

/// A frame's groups (drawn over its trees) and the ranges of its trees
/// the live elements taking part draw (left out of its items).
pub(crate) type Groups = (Vec<(DrawCmd, Affine)>, Vec<Range<usize>>);

/// The view transition running.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Running {
    /// The tick it started.
    pub since: u64,
    /// The screen whose rules style its groups.
    pub screen: String,
    pub old: Vec<Capture>,
    /// Skipped (a name twice): its layers end their transitions at once.
    pub skipped: bool,
}

impl Running {
    /// As [`Ui::save_state`] carries it.
    pub(crate) fn save(&self) -> Value {
        let old: Vec<Value> = self
            .old
            .iter()
            .map(|c| {
                let mut b = Vec::new();
                let mut w = crate::motion::codec::W(&mut b);
                w.u32(c.cmds.len() as u32);
                for (cmd, m) in &c.cmds {
                    w.cmd(cmd);
                    w.affine(m);
                }
                w.rect(&c.screen);
                json!({
                    "key": c.key.to_string(), "name": c.name, "classes": c.classes,
                    "drawing": b.iter().map(|x| format!("{x:02x}")).collect::<String>(),
                })
            })
            .collect();
        json!({ "since": self.since, "screen": self.screen, "skipped": self.skipped, "old": old })
    }

    /// Reads what [`Running::save`] wrote.
    pub(crate) fn restore(v: &Value) -> Option<Running> {
        let mut old = Vec::new();
        for c in v["old"].as_array()? {
            let hex = c["drawing"].as_str()?;
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|k| u8::from_str_radix(hex.get(k..k + 2)?, 16).ok())
                .collect::<Option<_>>()?;
            let mut r = crate::motion::codec::R {
                bytes: &bytes,
                at: 0,
                legacy_glyphs: false,
                affine_only: false,
            };
            let mut cmds = Vec::new();
            for _ in 0..r.u32()? {
                let cmd = r.cmd()?;
                cmds.push((cmd, r.affine()?));
            }
            old.push(Capture {
                key: c["key"].as_str()?.parse().ok()?,
                name: c["name"].as_str()?.to_string(),
                classes: serde_json::from_value(c["classes"].clone()).ok()?,
                screen: r.rect()?,
                cmds,
            });
        }
        Some(Running {
            since: v["since"].as_u64()?,
            screen: v["screen"].as_str()?.to_string(),
            old,
            skipped: v["skipped"].as_bool()?,
        })
    }
}

/// A list's channel values at `t` ms after the transition started, over
/// the identity (no move, scale 1, opaque).
fn sample(ui: &Ui, list: &[CssAnimation], t: f32) -> [f32; CHANNELS] {
    let mut row = [0.0; CHANNELS];
    (row[2], row[3], row[7]) = (1.0, 1.0, 1.0);
    for a in list {
        let built;
        let frames = match crate::doc::view_transition::builtin_keyframes(&a.name) {
            Some(f) => {
                built = f;
                &built
            }
            None => match ui.docs.keyframes.get(&a.name) {
                Some(f) => &**f,
                None => continue,
            },
        };
        for (c, v) in a.sample_active(frames, t - a.delay_ms, &row, false).values {
            row[c] = v;
        }
    }
    row
}

/// How far the group's box has moved from the old box to the new one
/// (its UA animation's eased progress; 1 when it has no effect).
fn progress(list: &[CssAnimation], t: f32) -> f32 {
    let Some(a) = list.iter().rev().find(|a| a.name == GROUP_ANIM) else {
        return 1.0;
    };
    let key = |offset, value| Keyframe {
        offset,
        value,
        ease: None,
        composite: None,
    };
    let frames = Keyframes {
        tracks: vec![KeyframeTrack {
            prop: crate::element::Prop::Opacity,
            keys: vec![key(0.0, 0.0), key(1.0, 1.0)],
        }],
        ..Keyframes::default()
    };
    let base = [1.0; CHANNELS];
    a.sample_active(&frames, t - a.delay_ms, &base, false)
        .values
        .first()
        .map_or(1.0, |&(_, v)| v)
}

/// When the last of `lists` ends, ms after they start.
fn ends(lists: &[&[CssAnimation]]) -> f32 {
    lists
        .iter()
        .map(|l| crate::layers::ends(l))
        .fold(0.0, f32::max)
}

fn lerp(a: &Rect, b: &Rect, p: f32) -> Rect {
    let m = |x: f32, y: f32| x + (y - x) * p;
    Rect::new(
        m(a.x, b.x),
        m(a.y, b.y),
        m(a.width, b.width),
        m(a.height, b.height),
    )
}

/// The transform channels of a sampled row about `rect`.
fn transform(row: &[f32; CHANNELS], rect: &Rect) -> Affine {
    Transform {
        x: row[0],
        y: row[1],
        scale_x: row[2],
        scale_y: row[3],
        rotate: row[4],
        skew_x: row[5],
        skew_y: row[6],
        ..Transform::IDENTITY
    }
    .matrix(rect)
}

const EVERYWHERE: Rect = Rect {
    x: -1e9,
    y: -1e9,
    width: 2e9,
    height: 2e9,
};

fn begin_layer() -> DrawCmd {
    DrawCmd::BeginLayer {
        material: og_render::UiMaterial::Plain,
        params: [[0.0; 4]; og_render::UI_MATERIAL_PARAMS],
        mask: None,
        expand: 0.0,
        rect: Rect::default(),
        filter: None,
        role: Default::default(),
    }
}

fn end_layer(rect: Rect, opacity: f32, blend: og_render::UiBlend) -> DrawCmd {
    DrawCmd::EndLayer {
        geometry: None,
        rect,
        opacity,
        blend,
        clip: EVERYWHERE,
        ground: None,
    }
}

impl Ui {
    /// A layer's view transition starts this tick: the first to start
    /// captures last frame's shared elements; the incoming screen's rules
    /// style the groups. Reduced motion starts none (unless the style
    /// keeps motion).
    pub(crate) fn start_view_transition(&mut self, screen: &str, joining: bool) {
        let tick = self.state.tick;
        if let Some(r) = &mut self.state.view_transition
            && r.since == tick
        {
            if joining {
                r.screen = screen.to_string();
            }
            return;
        }
        self.state.view_transition = None;
        if self.theme.reduced_motion && self.theme.motion.reduced != crate::motion::Reduced::Keep {
            return;
        }
        let last = &self.kept.last_cmds;
        let old = self
            .kept
            .named
            .iter()
            .filter(|n| n.range.end <= last.len())
            .map(|n| Capture {
                key: n.key,
                name: n.name.clone(),
                classes: n.classes.clone(),
                screen: n.screen,
                cmds: own(&last[n.range.clone()], n.inherited),
            })
            .collect();
        self.state.view_transition = Some(Running {
            since: tick,
            screen: screen.to_string(),
            old,
            skipped: false,
        });
    }

    /// Whether the view transition that started at `since` was skipped:
    /// its layers end theirs at once.
    pub(crate) fn view_transition_skipped(&self, since: u64) -> bool {
        self.state
            .view_transition
            .as_ref()
            .is_some_and(|r| r.skipped && r.since == since)
    }

    /// The running view transition's groups this frame, drawn over the
    /// frame's trees, and the ranges of `trees` the live elements taking
    /// part draw (left out of the frame's items). `None` when none runs
    /// (or it has just ended).
    pub(crate) fn view_transition_frame(&mut self, trees: &[(DrawCmd, Affine)]) -> Option<Groups> {
        let run = self.state.view_transition.take()?;
        if run.skipped {
            // Kept while its layers may still read it.
            if self.docs.layers.layers.iter().any(|l| l.since == run.since) {
                self.state.view_transition = Some(run);
            }
            return None;
        }
        let style = self
            .docs
            .docs
            .get(&run.screen)
            .and_then(|d| d.layer.transition.clone())?;
        let t = self.state.tick.saturating_sub(run.since) as f32 * 1000.0
            / self.ticks_per_second.max(1) as f32;
        // The new view: this frame's shared elements but a leaving layer's.
        let leaving: Vec<String> = self
            .docs
            .layers
            .layers
            .iter()
            .filter(|l| l.leaving)
            .map(crate::layers::layer_id)
            .collect();
        let in_leaving = |ui: &Ui, mut j: usize| {
            while j != NONE {
                if ui.kinds.get(j).is_some_and(|k| k.0 == "layer") {
                    return leaving.contains(&ui.nodes[j].id);
                }
                j = ui.nodes[j].parent;
            }
            false
        };
        let new: Vec<Named> = self
            .kept
            .named
            .iter()
            .filter(|n| leaving.is_empty() || !in_leaving(self, n.node))
            .cloned()
            .collect();
        let twice = |names: Vec<&String>| {
            names
                .iter()
                .enumerate()
                .find(|(k, n)| names[..*k].contains(n))
                .map(|(_, n)| (*n).clone())
        };
        let duplicate = twice(run.old.iter().map(|c| &c.name).collect())
            .map(|n| (n, "old"))
            .or_else(|| twice(new.iter().map(|n| &n.name).collect()).map(|n| (n, "new")));
        if let Some((name, view)) = duplicate {
            let name = if name.starts_with('#') {
                "auto"
            } else {
                name.as_str()
            };
            self.warnings.push(format!(
                "view-transition-name '{name}' is on two elements of the {view} view: the view transition is skipped (each name names one element)"
            ));
            self.state.view_transition = Some(Running {
                skipped: true,
                old: Vec::new(),
                ..run
            });
            return None;
        }
        // The groups: the old view's names in its order, then the new
        // view's others.
        let mut names: Vec<&String> = run.old.iter().map(|c| &c.name).collect();
        for n in &new {
            if !names.contains(&&n.name) {
                names.push(&n.name);
            }
        }
        let mut tail = Vec::new();
        let mut end = 0.0f32;
        let blend = |both: bool, list: &[CssAnimation]| {
            if both && list.iter().any(|a| a.name == FADE_IN || a.name == FADE_OUT) {
                og_render::UiBlend::Add
            } else {
                og_render::UiBlend::Normal
            }
        };
        for name in names {
            let old = run.old.iter().find(|c| &c.name == name);
            let live = new.iter().find(|n| &n.name == name);
            let classes = live
                .map(|n| &n.classes)
                .or(old.map(|c| &c.classes))
                .cloned()
                .unwrap_or_default();
            let GroupStyle {
                group,
                pair,
                old: old_anim,
                new: new_anim,
            } = style.group(name, &classes);
            end = end.max(ends(&[&group, &pair, &old_anim, &new_anim]));
            let at = match (old, live) {
                (Some(o), Some(n)) => lerp(&o.screen, &n.screen, progress(&group, t)),
                (Some(o), None) => o.screen,
                (None, Some(n)) => n.screen,
                (None, None) => continue,
            };
            let g = sample(self, &group, t);
            let p = sample(self, &pair, t);
            let around = transform(&g, &at).mul(&transform(&p, &at));
            let both = old.is_some() && live.is_some();
            tail.push((begin_layer(), Affine::IDENTITY));
            // The new image is drawn live: this frame's commands, without
            // what its ancestors fade (none to recover while they are at 0).
            let live_cmds = live
                .filter(|n| n.inherited > 0.0)
                .map(|n| own(&trees[n.range.clone()], n.inherited));
            let images = [
                (old.map(|o| (o.screen, &o.cmds[..])), &old_anim),
                (
                    live.zip(live_cmds.as_deref()).map(|(n, c)| (n.screen, c)),
                    &new_anim,
                ),
            ];
            for (image, list) in images {
                let Some((from, cmds)) = image else { continue };
                let row = sample(self, list, t);
                if row[7] <= 0.0 {
                    continue;
                }
                // As wide as the group, its height by its own aspect, from
                // the group's top.
                let height = if from.width > 0.0 {
                    from.height * at.width / from.width
                } else {
                    at.height
                };
                let to = Rect::new(at.x, at.y, at.width, height);
                let map = around
                    .mul(&transform(&row, &to))
                    .mul(&Affine::rect_to_rect(&from, &to));
                tail.push((begin_layer(), Affine::IDENTITY));
                tail.extend(cmds.iter().map(|(c, m)| (c.clone(), map.mul(m))));
                tail.push((
                    end_layer(to, row[7].min(1.0), blend(both, list)),
                    Affine::IDENTITY,
                ));
            }
            tail.push((
                end_layer(
                    at,
                    (g[7] * p[7]).clamp(0.0, 1.0),
                    og_render::UiBlend::Normal,
                ),
                Affine::IDENTITY,
            ));
        }
        if t >= end {
            return None;
        }
        let hidden = self
            .kept
            .named
            .iter()
            .filter(|n| {
                new.iter().any(|m| m.key == n.key) || run.old.iter().any(|c| c.key == n.key)
            })
            .map(|n| n.range.clone())
            .collect();
        self.state.view_transition = Some(run);
        Some((tail, hidden))
    }
}
