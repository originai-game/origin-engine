//! Shared elements of view transitions (CSS View Transitions 1 and 2,
//! docs/ui.md §2.1). When a screen's layer starts its view transition
//! ([`crate::layers`]), every element with a `view-transition-name` that
//! last frame drew is captured: its draw commands, frozen as they were,
//! are its old image. Each name is a `::view-transition-group`: in the top
//! layer it moves from the old element's box to the new one's (the new
//! element with that name, drawn live each frame), and its image pair
//! cross-fades the two with `plus-lighter`. The live elements taking part
//! are hidden where they stand until their group ends. A shared element's
//! image is its own drawing: the shared elements inside it are groups of
//! their own, left out of it. Groups nest by `view-transition-group`
//! (Level 2): a nested group is drawn in its parent group's
//! `::view-transition-group-children`, over its image pair, moving from
//! where it was in the old parent's padding box to where it is in the new
//! one's, and carried by its parent's box, transform, opacity and clips. The groups are
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
    /// Its `view-transition-group` (empty: normal).
    pub group: String,
    /// The group its own nests in (the spec's resolved group name), set
    /// once the frame's shared elements are known ([`Ui::nest_named`]).
    pub parent: Option<String>,
    pub range: Range<usize>,
    /// Its box on the screen.
    pub screen: Rect,
    /// Its border's widths on the screen (left, top, right, bottom): its
    /// padding box, where its nested groups are placed and clipped.
    pub inset: [f32; 4],
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

/// A shared element's own drawing in `cmds`: without the shared elements
/// inside it (each a group of its own, as the spec captures them) and the
/// opacity its ancestors gave it.
fn image(cmds: &[(DrawCmd, Affine)], n: &Named, all: &[Named]) -> Vec<(DrawCmd, Affine)> {
    let mut inner: Vec<&Range<usize>> = all
        .iter()
        .filter(|m| m.key != n.key && !m.range.is_empty())
        .map(|m| &m.range)
        .filter(|r| r.start >= n.range.start && r.end <= n.range.end)
        .collect();
    if inner.is_empty() {
        return own(&cmds[n.range.clone()], n.inherited);
    }
    inner.sort_by_key(|r| r.start);
    let (mut kept, mut at) = (Vec::new(), n.range.start);
    for r in inner {
        if r.start >= at {
            kept.extend_from_slice(&cmds[at..r.start]);
        }
        at = at.max(r.end);
    }
    kept.extend_from_slice(&cmds[at..n.range.end]);
    own(&kept, n.inherited)
}

/// A node's border widths on the screen (left, top, right, bottom).
pub(crate) fn border_inset(n: &crate::layout::Node) -> [f32; 4] {
    let Some(d) = &n.decor else {
        return [0.0; 4];
    };
    let scale = |screen: f32, layout: f32| {
        if layout > 0.0 { screen / layout } else { 1.0 }
    };
    let (kx, ky) = (
        scale(n.screen.width, n.rect.width),
        scale(n.screen.height, n.rect.height),
    );
    let w = |k: usize| d.borders[k].map_or(0.0, |l| l.width);
    [w(3) * kx, w(0) * ky, w(1) * kx, w(2) * ky]
}

/// `rect`'s padding box: in from it by `inset`.
fn padding(rect: &Rect, inset: &[f32; 4]) -> Rect {
    Rect::new(
        rect.x + inset[0],
        rect.y + inset[1],
        (rect.width - inset[0] - inset[2]).max(0.0),
        (rect.height - inset[1] - inset[3]).max(0.0),
    )
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
    /// The group its own nested in, in the old view.
    pub parent: Option<String>,
    pub screen: Rect,
    pub inset: [f32; 4],
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
                    "parent": c.parent, "inset": c.inset,
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
                parent: serde_json::from_value(c["parent"].clone()).ok()?,
                inset: serde_json::from_value(c["inset"].clone()).ok()?,
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

/// The cut to box `at` along the axes that clip (across, down); `None`
/// when neither does.
fn clip_to(at: &Rect, axes: [bool; 2]) -> Option<DrawCmd> {
    if axes == [false, false] {
        return None;
    }
    let half = |clips: bool, len: f32| if clips { len / 2.0 } else { 1e9 };
    Some(DrawCmd::PushClip {
        geometry: og_draw::UiGeometry::Rect {
            radii: [0.0; 4],
            corner: og_draw::UiCorner::Round,
        },
        center: (at.x + at.width / 2.0, at.y + at.height / 2.0),
        half: (half(axes[0], at.width), half(axes[1], at.height)),
    })
}

fn begin_layer() -> DrawCmd {
    DrawCmd::BeginLayer {
        material: og_draw::UiMaterial::Plain,
        params: [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS],
        mask: None,
        expand: 0.0,
        rect: Rect::default(),
        filter: None,
        role: Default::default(),
    }
}

fn end_layer(rect: Rect, opacity: f32, blend: og_draw::UiBlend) -> DrawCmd {
    DrawCmd::EndLayer {
        geometry: None,
        rect,
        opacity,
        blend,
        clip: EVERYWHERE,
        ground: None,
    }
}

/// A group of the running transition this frame.
struct Group<'a> {
    old: Option<&'a Capture>,
    live: Option<&'a Named>,
    style: GroupStyle,
    /// The group it nests in, and those nested in it (indices).
    parent: Option<usize>,
    children: Vec<usize>,
    /// Its box this frame, before its parents' transforms.
    at: Rect,
}

impl Group<'_> {
    /// Its padding box in the old view and the new one (the other view's
    /// when it is in one only).
    fn boxes(&self) -> (Rect, Rect) {
        let old = self.old.map(|c| padding(&c.screen, &c.inset));
        let new = self.live.map(|n| padding(&n.screen, &n.inset));
        let any = old.or(new).unwrap_or_default();
        (old.unwrap_or(any), new.unwrap_or(any))
    }

    /// Its border's widths (the new element's, else the old's).
    fn inset(&self) -> [f32; 4] {
        self.live
            .map(|n| n.inset)
            .or(self.old.map(|c| c.inset))
            .unwrap_or_default()
    }
}

/// Places group `k` and those nested in it at `t`: a group moves from the
/// old element's box to the new one's; a nested one does so relative to
/// its parent's padding box (`within`: where that box is now, and where
/// it was in the old view and is in the new one), so it rides along.
fn place(groups: &mut [Group], k: usize, within: Option<(Rect, Rect, Rect)>, t: f32) {
    let g = &groups[k];
    let rel = |r: &Rect, base: Option<Rect>| match base {
        Some(b) => Rect::new(r.x - b.x, r.y - b.y, r.width, r.height),
        None => *r,
    };
    let (from, to) = (within.map(|w| w.1), within.map(|w| w.2));
    let mut at = match (g.old, g.live) {
        (Some(o), Some(n)) => lerp(
            &rel(&o.screen, from),
            &rel(&n.screen, to),
            progress(&g.style.group, t),
        ),
        (Some(o), None) => rel(&o.screen, from),
        (None, Some(n)) => rel(&n.screen, to),
        (None, None) => Rect::default(),
    };
    if let Some((now, ..)) = within {
        (at.x, at.y) = (at.x + now.x, at.y + now.y);
    }
    groups[k].at = at;
    let (old, new) = groups[k].boxes();
    let inside = (padding(&at, &groups[k].inset()), old, new);
    for c in groups[k].children.clone() {
        place(groups, c, Some(inside), t);
    }
}

/// Draws group `k` (placed) under `outer`, its parents' transforms: its
/// image pair, then the groups nested in it in its
/// `::view-transition-group-children`, all in its opacity and its clip.
#[allow(clippy::too_many_arguments)]
fn draw(
    ui: &Ui,
    groups: &[Group],
    k: usize,
    outer: Affine,
    t: f32,
    trees: &[(DrawCmd, Affine)],
    all: &[Named],
    tail: &mut Vec<(DrawCmd, Affine)>,
) {
    let Group {
        old,
        live,
        style,
        children,
        at,
        ..
    } = &groups[k];
    let g = sample(ui, &style.group, t);
    let p = sample(ui, &style.pair, t);
    let group = outer.mul(&transform(&g, at));
    let around = group.mul(&transform(&p, at));
    let both = old.is_some() && live.is_some();
    let nested = !children.is_empty();
    tail.push((begin_layer(), Affine::IDENTITY));
    // `overflow` on the group cuts all it holds (its nested groups too);
    // on the image pair, its images.
    let group_cut = clip_to(at, style.group_clip);
    if let Some(cut) = &group_cut {
        tail.push((cut.clone(), group));
    }
    // With groups nested over it, the pair is a layer of its own (its
    // opacity is not theirs); else the group's layer is the pair's too.
    if nested {
        tail.push((begin_layer(), Affine::IDENTITY));
    }
    let pair_cut = clip_to(at, style.pair_clip);
    if let Some(cut) = &pair_cut {
        tail.push((cut.clone(), around));
    }
    // The new image is drawn live: this frame's commands, without what
    // its ancestors fade (none to recover while they are at 0).
    let live_cmds = live
        .filter(|n| n.inherited > 0.0)
        .map(|n| image(trees, n, all));
    let images = [
        (old.map(|o| (o.screen, &o.cmds[..])), &style.old),
        (
            live.zip(live_cmds.as_deref()).map(|(n, c)| (n.screen, c)),
            &style.new,
        ),
    ];
    for (image, list) in images {
        let Some((from, cmds)) = image else { continue };
        let row = sample(ui, list, t);
        if row[7] <= 0.0 {
            continue;
        }
        // As wide as the group, its height by its own aspect, from the
        // group's top.
        let height = if from.width > 0.0 {
            from.height * at.width / from.width
        } else {
            at.height
        };
        let to = Rect::new(at.x, at.y, at.width, height);
        let map = around
            .mul(&transform(&row, &to))
            .mul(&Affine::rect_to_rect(&from, &to));
        let blend = if both && list.iter().any(|a| a.name == FADE_IN || a.name == FADE_OUT) {
            og_draw::UiBlend::Add
        } else {
            og_draw::UiBlend::Normal
        };
        tail.push((begin_layer(), Affine::IDENTITY));
        tail.extend(cmds.iter().map(|(c, m)| (c.clone(), map.mul(m))));
        tail.push((end_layer(to, row[7].min(1.0), blend), Affine::IDENTITY));
    }
    if pair_cut.is_some() {
        tail.push((DrawCmd::PopClip, Affine::IDENTITY));
    }
    let opacity = if nested {
        tail.push((
            end_layer(*at, p[7].clamp(0.0, 1.0), og_draw::UiBlend::Normal),
            Affine::IDENTITY,
        ));
        // The children: the group's padding box, its own animations'
        // transform and opacity, and its `overflow` clip.
        let pad = padding(at, &groups[k].inset());
        let c = sample(ui, &style.children, t);
        let inner = group.mul(&transform(&c, &pad));
        let faded = c[7] < 1.0;
        if faded {
            tail.push((begin_layer(), Affine::IDENTITY));
        }
        let cut = clip_to(&pad, style.children_clip);
        if let Some(cut) = &cut {
            tail.push((cut.clone(), inner));
        }
        if c[7] > 0.0 {
            for &child in children {
                draw(ui, groups, child, inner, t, trees, all, tail);
            }
        }
        if cut.is_some() {
            tail.push((DrawCmd::PopClip, Affine::IDENTITY));
        }
        if faded {
            tail.push((
                end_layer(pad, c[7].max(0.0), og_draw::UiBlend::Normal),
                Affine::IDENTITY,
            ));
        }
        g[7]
    } else {
        g[7] * p[7]
    };
    if group_cut.is_some() {
        tail.push((DrawCmd::PopClip, Affine::IDENTITY));
    }
    tail.push((
        end_layer(*at, opacity.clamp(0.0, 1.0), og_draw::UiBlend::Normal),
        Affine::IDENTITY,
    ));
}

impl Ui {
    /// Where each of this frame's shared elements' groups nests, by their
    /// `view-transition-group` (the spec's resolved group name): `normal`
    /// and `contain` in the nearest shared ancestor's that is not normal,
    /// `nearest` in the nearest shared ancestor's, a name in the ancestor's
    /// of that name (else as normal). With every one normal, none nests.
    pub(crate) fn nest_named(&mut self) {
        let named = &mut self.kept.named;
        if named.iter().all(|n| n.group.is_empty()) {
            return;
        }
        let index: std::collections::HashMap<usize, usize> =
            named.iter().enumerate().map(|(k, n)| (n.node, k)).collect();
        for k in 0..named.len() {
            // Its shared ancestors, nearest first.
            let mut up = Vec::new();
            let mut j = self.nodes[named[k].node].parent;
            while j != NONE {
                if let Some(&m) = index.get(&j) {
                    up.push(m);
                }
                j = self.nodes[j].parent;
            }
            let name = |m: &usize| named[*m].name.clone();
            let containing = up.iter().find(|&&m| !named[m].group.is_empty()).map(name);
            named[k].parent = match named[k].group.as_str() {
                "" | "contain" => containing,
                "nearest" => up.first().map(name),
                g if up.iter().any(|&m| named[m].name == g) => Some(g.to_string()),
                _ => containing,
            };
        }
    }

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
        let (last, named) = (&self.kept.last_cmds, &self.kept.named);
        let old = named
            .iter()
            .filter(|n| n.range.end <= last.len())
            .map(|n| Capture {
                key: n.key,
                name: n.name.clone(),
                classes: n.classes.clone(),
                parent: n.parent.clone(),
                screen: n.screen,
                inset: n.inset,
                cmds: image(last, n, named),
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
        // view's others; each nested in its parent (the new view's, else
        // the old's).
        let mut names: Vec<&String> = run.old.iter().map(|c| &c.name).collect();
        for n in &new {
            if !names.contains(&&n.name) {
                names.push(&n.name);
            }
        }
        let mut groups: Vec<Group> = names
            .iter()
            .map(|name| {
                let old = run.old.iter().find(|c| &c.name == *name);
                let live = new.iter().find(|n| &n.name == *name);
                let classes = live
                    .map(|n| &n.classes)
                    .or(old.map(|c| &c.classes))
                    .cloned()
                    .unwrap_or_default();
                Group {
                    old,
                    live,
                    style: style.group(name, &classes),
                    parent: None,
                    children: Vec::new(),
                    at: Rect::default(),
                }
            })
            .collect();
        for k in 0..groups.len() {
            let parent = match (groups[k].live, groups[k].old) {
                (Some(n), _) => n.parent.as_ref(),
                (None, Some(c)) => c.parent.as_ref(),
                (None, None) => None,
            };
            if let Some(p) = parent.and_then(|p| names.iter().position(|n| *n == p))
                && p != k
            {
                groups[k].parent = Some(p);
                groups[p].children.push(k);
            }
        }
        let mut end = 0.0f32;
        for g in &groups {
            let s = &g.style;
            end = end.max(ends(&[&s.group, &s.pair, &s.old, &s.new, &s.children]));
        }
        let all = &self.kept.named;
        let mut tail = Vec::new();
        for k in 0..groups.len() {
            if groups[k].parent.is_none() {
                place(&mut groups, k, None, t);
                draw(self, &groups, k, Affine::IDENTITY, t, trees, all, &mut tail);
            }
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
