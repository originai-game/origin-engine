//! World-anchored elements in documents (docs/ui.md §3.3): `<anchor
//! entity={u.entity} offset="0 2.1 0" occlusion="fade" declutter="stack"
//! clamp="edge">…</anchor>`, the native anchor ([`Ui::open_world_anchor`])
//! with its children laid out as any box. The anchor is placed every tick
//! from the camera and the entity's pose; its subtree is built again only
//! when a binding it reads changes ([`Ui::keep_world_anchors`] places the
//! ones not built this tick).
use super::*;
use crate::world::{
    AnchorScale, Declutter, EdgeClamp, WorldAnchorDesc, WorldOcclusion, WorldPanelDesc, WorldTarget,
};
use og_core::glam::Vec3;

fn vec3(text: &str) -> Option<Vec3> {
    let n: Vec<f32> = text
        .replace(',', " ")
        .split_whitespace()
        .filter_map(|w| w.parse().ok())
        .collect();
    match n.as_slice() {
        [x, y, z] => Some(Vec3::new(*x, *y, *z)),
        _ => None,
    }
}

/// The anchor `r` describes; `Err` names a bad attribute.
pub(super) fn anchor_desc(r: &Resolved, el: Element) -> Result<WorldAnchorDesc, String> {
    let target = match (r.num("entity"), r.str("worldAt")) {
        (Some(e), _) => WorldTarget::Entity(e as u64),
        (None, Some(p)) => WorldTarget::Point(
            vec3(p).ok_or_else(|| format!("at=\"{p}\" is three numbers, x y z"))?,
        ),
        (None, None) => return Err("an <anchor> takes entity={…} or at=\"x y z\"".into()),
    };
    let offset = match r.str("worldOffset") {
        Some(v) => vec3(v).ok_or_else(|| format!("offset=\"{v}\" is three numbers"))?,
        None => Vec3::ZERO,
    };
    let occlusion = match r.str("occlusion") {
        None | Some("none") => WorldOcclusion::None,
        Some("fade") => WorldOcclusion::Fade { alpha: 0.35 },
        Some("hide") => WorldOcclusion::Hide,
        Some(o) => return Err(format!("occlusion=\"{o}\" is none, fade or hide")),
    };
    let declutter = match r.str("declutter") {
        None | Some("none") => Declutter::None,
        Some("shift") => Declutter::Shift,
        Some("stack") => Declutter::Stack,
        Some("hide") => Declutter::Hide,
        Some(o) => return Err(format!("declutter=\"{o}\" is none, shift, stack or hide")),
    };
    let clamp = match r.str("worldClamp") {
        None | Some("none") => None,
        Some("edge") => Some(EdgeClamp::default()),
        Some(o) => return Err(format!("clamp=\"{o}\" is none or edge")),
    };
    let scale = match r.num("worldScaleAt") {
        Some(d) if d > 0.0 => AnchorScale::Distance {
            reference: d,
            min: 0.5,
            max: 2.0,
        },
        _ => AnchorScale::Constant,
    };
    Ok(WorldAnchorDesc {
        target,
        offset,
        element: el,
        occlusion,
        declutter,
        clamp,
        scale,
        priority: r.num("priority").unwrap_or(0.0) as i32,
        max_distance: r.num("maxDistance").unwrap_or(0.0).max(0.0),
        ..WorldAnchorDesc::default()
    })
}

/// Three numbers or `None`; `what` names the attribute in the error.
fn three(r: &Resolved, prop: &str, what: &str) -> Result<Option<Vec3>, String> {
    r.str(prop)
        .map(|v| vec3(v).ok_or_else(|| format!("{what}=\"{v}\" is three numbers")))
        .transpose()
}

/// The world panel `r` describes (`<panel world entity size rotation
/// src>`): the box `el` laid out at its CSS size, shown on a quad `size`
/// world units across turned by `rotation` (degrees about x, y, z).
///
/// A `ground` lies flat: facing up (its top away from a camera south of
/// it), `rotation`'s y turning it about the vertical; the UI draws it.
pub(super) fn panel_desc(
    r: &Resolved,
    el: Element,
    texture: Option<og_render::TextureHandle>,
    ground: bool,
) -> Result<WorldPanelDesc, String> {
    let px = |s: Size, d: f32| match s {
        Size::Fixed(v) => v,
        _ => d,
    };
    let target = match (r.num("entity"), r.str("worldAt")) {
        (Some(e), _) => Some(WorldTarget::Entity(e as u64)),
        (None, Some(p)) => {
            Some(WorldTarget::Point(vec3(p).ok_or_else(|| {
                format!("at=\"{p}\" is three numbers, x y z")
            })?))
        }
        (None, None) => None,
    };
    let size = match r.str("worldSize") {
        Some(v) => {
            let n: Vec<f32> = v
                .split_whitespace()
                .filter_map(|w| w.parse().ok())
                .collect();
            match n.as_slice() {
                [w, h] if *w > 0.0 && *h > 0.0 => og_core::glam::Vec2::new(*w, *h),
                _ => {
                    return Err(format!(
                        "size=\"{v}\" is two positive numbers, width height"
                    ));
                }
            }
        }
        None => og_core::glam::Vec2::ONE,
    };
    let mut turn = three(r, "worldRotation", "rotation")?.unwrap_or(Vec3::ZERO);
    if ground {
        turn.x -= 90.0;
    }
    let rotation = og_core::glam::Quat::from_euler(
        og_core::glam::EulerRot::YXZ,
        turn.y.to_radians(),
        turn.x.to_radians(),
        turn.z.to_radians(),
    );
    Ok(WorldPanelDesc {
        target: texture,
        width: px(el.width, 480.0).max(1.0),
        height: px(el.height, 320.0).max(1.0),
        quad: target.map(|t| (t, rotation, size)),
        element: el,
        projected: ground,
    })
}

/// `-og-emitter: <rate> [square|circle|diamond] [<size>px] [<colour>…]`.
fn emitter_desc(v: &str) -> Result<Option<crate::particles::ParticlesDesc>, String> {
    use og_render::UiParticleShape;
    if v.trim() == "none" {
        return Ok(None);
    }
    let mut d = crate::particles::ParticlesDesc {
        fade: true,
        ..Default::default()
    };
    let mut rate = None;
    for w in v.split_whitespace() {
        match w {
            "square" => d.shape = UiParticleShape::Square,
            "circle" => d.shape = UiParticleShape::Circle,
            "diamond" => d.shape = UiParticleShape::Diamond,
            _ if rate.is_none() && w.parse::<f32>().is_ok() => rate = w.parse().ok(),
            _ => {
                if let Some(px) = w.strip_suffix("px").and_then(|n| n.parse::<f32>().ok()) {
                    d.size = og_core::glam::Vec2::splat(px);
                } else {
                    d.colors.push(srgba(w)?);
                }
            }
        }
    }
    d.rate = rate
        .ok_or("-og-emitter takes a rate a second first: `-og-emitter: 12 diamond 6px #ffd166`")?;
    Ok(Some(d))
}

/// `-og-meta: vignette [<colour>] | tint <colour> | none`.
fn meta_desc(v: &str) -> Result<Option<crate::world::MetaDesc>, String> {
    use crate::world::{MetaDesc, MetaKind};
    let mut w = v.split_whitespace();
    let kind = match (w.next(), w.next()) {
        (Some("none"), None) | (None, _) => return Ok(None),
        (Some("vignette"), c) => MetaKind::Vignette {
            color: c
                .map(srgba)
                .transpose()?
                .unwrap_or(crate::style::Srgba::rgb(160, 0, 0)),
            inner: 0.55,
            outer: 1.0,
        },
        (Some("tint"), Some(c)) => MetaKind::Tint { color: srgba(c)? },
        _ => return Err("-og-meta is vignette [<colour>], tint <colour> or none".into()),
    };
    Ok(Some(MetaDesc {
        kind,
        ..MetaDesc::default()
    }))
}

fn srgba(w: &str) -> Result<crate::style::Srgba, String> {
    let c = crate::svg::color(w)?;
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Ok(crate::style::Srgba {
        rgb: [b(c[0]), b(c[1]), b(c[2])],
        alpha: c[3],
    })
}

impl Ui {
    /// An element's `-og-emitter` and `-og-meta`, on node `i`.
    pub(super) fn world_effects(
        &mut self,
        b: &Build<'_>,
        n: &DocNode,
        r: &Resolved,
        i: usize,
        path: &str,
    ) {
        let emitter = r.str("emitter").map(emitter_desc).transpose();
        let meta = r.str("meta").map(meta_desc).transpose();
        match (emitter, meta) {
            (Ok(e), Ok(m)) => self.document_effects(i, e.flatten(), m.flatten()),
            (Err(e), _) | (_, Err(e)) => self
                .warnings
                .push(format!("{}:{} ({path}): {e}", b.doc.source, n.line)),
        }
    }

    /// A `<panel world>`: its subtree drawn into the render target `src`
    /// names (shown by the game on a plane), and rays from the pointer
    /// that meet its quad reach its controls.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_world_panel(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        r: &Resolved,
        props: Props,
        path: String,
        key: u64,
        built: Built,
        build_children: impl FnOnce(&mut Ui, &mut Build<'_>, Option<Props>),
    ) -> Option<String> {
        let el = Self::box_of(
            r,
            Element {
                direction: Direction::Column,
                ..Element::default()
            },
            self.em_of(r),
        );
        let texture = r
            .num("viewportSrc")
            .and_then(|h| b.env.texture(f64::from(h)));
        let ground = matches!(n.kind, NodeKind::Element("ground"));
        let desc = match panel_desc(r, el, if ground { None } else { texture }, ground) {
            Ok(d) => d,
            Err(e) => {
                self.warnings
                    .push(format!("{}:{} ({path}): {e}", b.doc.source, n.line));
                return None;
            }
        };
        self.open_world_panel_keyed(path.as_str().into(), Some(key), desc);
        let i = self.parent();
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        self.nodes[i].props = p;
        if let Some(name) = r.str("name").filter(|n| !n.is_empty()) {
            self.nodes[i].a11y.name = Some(name.to_string());
        }
        self.keep_world_panel(i, key);
        self.note_built(built);
        build_children(self, b, None);
        let resp = self.close();
        self.fire(b, n, key, &path, resp);
        Some(path)
    }

    /// An `<anchor mass>`: a mass element of the template its `mass` names
    /// (defined from the element: a number, or with `max` a bar; its first
    /// CSS animation's `@keyframes` its timeline), held in the tree by a
    /// hidden leaf so the retained document keeps it.
    fn build_mass(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        r: &Resolved,
        name: &str,
        path: &str,
        key: u64,
    ) {
        use og_render::{UiMassInstance, UiMassKey, UiMassKind, UiMassSpace, UiMassTemplate};
        let target = match (r.num("entity"), r.str("worldAt")) {
            (Some(e), _) => WorldTarget::Entity(e as u64),
            (None, Some(p)) => match vec3(p) {
                Some(v) => WorldTarget::Point(v),
                None => {
                    self.warnings.push(format!(
                        "{}:{} ({path}): at=\"{p}\" is three numbers, x y z",
                        b.doc.source, n.line
                    ));
                    return;
                }
            },
            (None, None) => {
                self.warnings.push(format!(
                    "{}:{} ({path}): an <anchor mass> takes entity={{…}} or at=\"x y z\"",
                    b.doc.source, n.line
                ));
                return;
            }
        };
        let offset = r.str("worldOffset").and_then(vec3).unwrap_or(Vec3::ZERO);
        let em = self.em_of(r);
        let value = r.num("massValue").unwrap_or(0.0);
        let max = r.num("massMax");
        let px = |s: Option<Size>, d: f32| match s {
            Some(Size::Fixed(v)) => v,
            _ => d,
        };
        let (kind, size, shown) = match max {
            Some(m) => (
                UiMassKind::Bar {
                    back: [0.0, 0.0, 0.0, 0.6],
                    ghost: [1.0, 1.0, 1.0, 0.7],
                    radius: 2.0,
                },
                og_core::glam::Vec2::new(px(r.size("width"), 60.0), px(r.size("height"), 6.0)),
                if m > 0.0 {
                    (value / m).clamp(0.0, 1.0)
                } else {
                    0.0
                },
            ),
            None => (
                UiMassKind::Number {
                    decimals: 0,
                    plus: false,
                    compact: false,
                },
                og_core::glam::Vec2::splat(em),
                value,
            ),
        };
        // The timeline: the first animation's keyframes, its translate,
        // scale and opacity tracks sampled at every key any of them has.
        let mut lifetime = 0;
        let mut keys = Vec::new();
        if let Some(a) = r.animations.first()
            && let Some(frames) = self.docs.keyframes.get(&a.name).cloned()
        {
            lifetime =
                ((a.duration_ms * self.ticks_per_second as f32 / 1000.0).round() as u32).max(1);
            let track = |prop: crate::element::Prop, t: f32, rest: f32| {
                let Some(tr) = frames.tracks.iter().find(|tr| tr.prop == prop) else {
                    return rest;
                };
                let mut pts: Vec<(f32, f32)> =
                    tr.keys.iter().map(|k| (k.offset, k.value)).collect();
                if pts.first().is_none_or(|p| p.0 > 0.0) {
                    pts.insert(0, (0.0, rest));
                }
                if pts.last().is_none_or(|p| p.0 < 1.0) {
                    pts.push((1.0, rest));
                }
                let k = pts.iter().position(|p| p.0 >= t).unwrap_or(pts.len() - 1);
                if k == 0 {
                    return pts[0].1;
                }
                let (a, b) = (pts[k - 1], pts[k]);
                let f = if b.0 > a.0 {
                    (t - a.0) / (b.0 - a.0)
                } else {
                    1.0
                };
                a.1 + (b.1 - a.1) * f
            };
            let mut at: Vec<f32> = frames
                .tracks
                .iter()
                .flat_map(|tr| tr.keys.iter().map(|k| k.offset))
                .chain([0.0, 1.0])
                .collect();
            at.sort_by(f32::total_cmp);
            at.dedup();
            use crate::element::Prop;
            for t in at {
                let scale = if frames.tracks.iter().any(|tr| tr.prop == Prop::Scale) {
                    track(Prop::Scale, t, 1.0)
                } else {
                    track(Prop::ScaleX, t, 1.0)
                };
                keys.push(UiMassKey {
                    t,
                    offset: og_core::glam::Vec2::new(
                        track(Prop::X, t, 0.0),
                        track(Prop::Y, t, 0.0),
                    ),
                    scale,
                    alpha: track(Prop::Opacity, t, 1.0),
                    ..UiMassKey::default()
                });
            }
        }
        let handle = self.define_mass(
            name,
            UiMassTemplate {
                kind,
                size,
                pivot: og_core::glam::Vec2::new(0.5, 1.0),
                lifetime,
                keys,
                ..UiMassTemplate::default()
            },
            UiMassSpace::World,
        );
        let color = r.color("color").map_or([1.0; 4], |c| c.linear());
        // Its place in the tree, so the retained document keeps it.
        let i = self.leaf(
            "mass",
            crate::Id::from(path.rsplit('/').next().unwrap_or(path)),
            Element {
                width: Size::Fixed(0.0),
                height: Size::Fixed(0.0),
                ..Element::default()
            },
        );
        self.nodes[i].hidden = true;
        let owner = self.nodes[i].key;
        debug_assert_eq!(owner, key, "the leaf takes the element's key");
        self.document_mass(
            key,
            owner,
            lifetime > 0,
            crate::world::KeptMass {
                handle,
                target,
                offset,
                instance: UiMassInstance {
                    value: shown,
                    color,
                    scale: 1.0,
                    seed: key as u32,
                    ..UiMassInstance::default()
                },
                owner,
            },
        );
    }

    /// An `<anchor>`: opened as the native anchor, kept for placement while
    /// its subtree is not built again.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_anchor(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        r: &Resolved,
        props: Props,
        path: String,
        key: u64,
        built: Built,
        build_children: impl FnOnce(&mut Ui, &mut Build<'_>, Option<Props>),
    ) -> Option<String> {
        if let Some(name) = r.str("mass").filter(|m| !m.is_empty()) {
            let name = name.to_owned();
            self.build_mass(b, n, r, &name, &path, key);
            self.note_built(built);
            return Some(path);
        }
        let el = Self::box_of(
            r,
            Element {
                direction: Direction::Column,
                ..Element::default()
            },
            self.em_of(r),
        );
        let desc = match anchor_desc(r, el) {
            Ok(d) => d,
            Err(e) => {
                self.warnings
                    .push(format!("{}:{} ({path}): {e}", b.doc.source, n.line));
                return None;
            }
        };
        self.open_world_anchor_keyed(path.as_str().into(), Some(key), desc.clone());
        let i = self.parent();
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        self.nodes[i].props = p;
        if let Some(name) = r.str("name").filter(|n| !n.is_empty()) {
            self.nodes[i].a11y.name = Some(name.to_string());
        }
        self.world.kept.insert(self.nodes[i].key, (desc, key));
        self.note_built(built);
        build_children(self, b, None);
        let resp = self.close();
        self.fire(b, n, key, &path, resp);
        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::UiDocument;
    use crate::reactive::SourceId;
    use crate::reactive::session::Command;
    use crate::world::WorldView;
    use crate::{InputFrame, Ui, UiDesc, Viewport};
    use og_core::glam::{Quat, Vec3};
    use serde_json::json;
    use std::collections::BTreeMap;

    const TAGS: &str = r#"
<meta name="component" content="tags">
<div id="Root">
  {#each $units as u (u.id)}
    <anchor id="Tag" entity={u.entity} offset="0 2 0"><p id="Name">{u.name}</p></anchor>
  {/each}
</div>
"#;

    /// Anchors follow their entities every tick without being built again;
    /// a changed name builds that tag alone; a removed row's anchor leaves.
    ///
    /// Ways it could fail: an anchor placed only on the tick it was built
    /// (frozen when the camera moves), every anchor rebuilt per tick, or a
    /// removed anchor still placed.
    #[test]
    fn anchors_follow_entities_and_build_only_what_changed() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("tags.svelte", TAGS).unwrap());
        let units = SourceId::new("test", "units");
        let rows = |names: &[&str]| -> Vec<serde_json::Map<String, serde_json::Value>> {
            names
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    json!({"id": format!("u{i}"), "entity": i + 1, "name": n})
                        .as_object()
                        .unwrap()
                        .clone()
                })
                .collect()
        };
        ui.reactive
            .execute(Command::Collection {
                source: units.clone(),
                key: "id".into(),
                items: rows(&["Ash", "Birch"]),
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "tags"),
            "tags",
            BTreeMap::from([("units".to_string(), units.clone())]),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui, eye_x: f32| {
            ui.set_world_view(Some(WorldView::look_at(
                Vec3::new(eye_x, 3.0, 10.0),
                Vec3::new(eye_x, 1.0, 0.0),
                Vec3::Y,
                50.0,
                800.0 / 600.0,
                0.1,
                100.0,
            )));
            ui.set_world_poses(
                &[1u64, 2]
                    .iter()
                    .map(|&e| {
                        (
                            e,
                            (
                                Vec3::new(e as f32 * 2.0 - 3.0, 0.0, 0.0),
                                Quat::IDENTITY,
                                Vec3::ONE,
                            ),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end().stats
        };
        for _ in 0..3 {
            tick(&mut ui, 0.0);
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let xs = |ui: &Ui| {
            let mut v: Vec<f32> = ui
                .probe_elements()
                .into_iter()
                .filter(|e| e.kind == "world")
                .map(|e| e.x)
                .collect();
            v.sort_by(f32::total_cmp);
            v
        };
        let before = xs(&ui);
        assert_eq!(before.len(), 2, "{before:?}");
        // The camera moves: both tags move, nothing is built again.
        let moved = tick(&mut ui, 1.0);
        assert_eq!((moved.bindings, moved.reconciled), (0, 0), "{moved:?}");
        let after = xs(&ui);
        assert!(
            after.iter().zip(&before).all(|(a, b)| a < b),
            "{before:?} → {after:?}"
        );
        // One name: that tag alone.
        ui.reactive
            .execute(Command::Patch {
                source: units.clone(),
                key: "u1".into(),
                fields: json!({"name": "Cedar"}).as_object().unwrap().clone(),
            })
            .unwrap();
        let named = tick(&mut ui, 1.0);
        assert!(
            named.bindings >= 1 && (1..=2).contains(&named.reconciled),
            "{named:?}"
        );
        assert_eq!(xs(&ui).len(), 2);
        let names: Vec<String> = ui
            .probe_elements()
            .into_iter()
            .filter(|e| e.path.ends_with("/Name"))
            .map(|e| e.text)
            .collect();
        assert!(names.contains(&"Cedar".to_string()), "{names:?}");
        let idle = tick(&mut ui, 1.0);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
        // A row leaves: its anchor leaves.
        ui.reactive
            .execute(Command::Remove {
                source: units,
                key: "u1".into(),
            })
            .unwrap();
        for _ in 0..2 {
            tick(&mut ui, 1.0);
        }
        assert_eq!(xs(&ui).len(), 1);
    }

    /// A world element's layer is keyed by its own identity, so what it
    /// holds has the keys the retained document gives it, two levels down
    /// as at the top, and one can be its document's root.
    ///
    /// Ways it could fail: a layer keyed from its id under the root (its
    /// children's keys then match no retained element: presses, focus and
    /// patches miss them), or a root anchor refused or colliding.
    #[test]
    fn a_world_layer_keys_its_subtree_as_the_document_does() {
        let mut ui = Ui::new(UiDesc::default());
        let doc = r#"<meta name="component" content="sign">
<anchor id="Sign" at="0 1 0"><div id="Box"><div id="Line"><p id="Word">Hi</p></div></div></anchor>"#;
        ui.define_document(UiDocument::parse("sign.svelte", doc).unwrap());
        ui.mount_document(
            SourceId::new("test", "sign"),
            "sign",
            BTreeMap::new(),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        for _ in 0..2 {
            ui.set_world_view(Some(WorldView::look_at(
                Vec3::new(0.0, 3.0, 10.0),
                Vec3::ZERO,
                Vec3::Y,
                50.0,
                800.0 / 600.0,
                0.1,
                100.0,
            )));
            ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let path = "test:sign/Box/Line/Word";
        let word = ui
            .nodes
            .iter()
            .position(|n| n.id == "Word")
            .expect("the word is built");
        assert_eq!(ui.nodes[word].key, crate::id::key_of_path(path));
        assert!(ui.element(path).is_some(), "{path}");
        let layer = ui.nodes[ui.nodes[ui.nodes[word].parent].parent].parent;
        assert_eq!(ui.nodes[layer].parent, crate::layout::NONE);
        assert_eq!(ui.nodes[layer].key, crate::id::key_of_path("test:sign"));
    }

    /// A keyed each of `<anchor mass>`: a row with an animation spawns one
    /// mass element when it appears and never again; a row without one
    /// stands as a bar pushed every tick with nothing built, and leaves
    /// with its row. The keyframes are the template's timeline.
    ///
    /// Ways it could fail: a spawn on every build (numbers repeat), a bar
    /// that needs its element rebuilt each tick, or one left after its row.
    #[test]
    fn mass_anchors_spawn_once_and_bars_follow_their_rows() {
        let doc = r#"<meta name="component" content="hits">
<div id="Root">
  {#each $hits as h (h.id)}<anchor id="Hit" entity={h.entity} mass="damage" value={h.amount}></anchor>{/each}
  {#each $units as u (u.id)}<anchor id="Bar" entity={u.entity} mass="hp" value={u.hp} max="100"></anchor>{/each}
</div>
<style>
#Hit { animation: rise 1s linear; }
@keyframes rise { from { translate: 0 0; } to { translate: 0 -40px; opacity: 0; } }
</style>"#;
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("hits.svelte", doc).unwrap());
        let rows = |v: serde_json::Value| -> Vec<serde_json::Map<String, serde_json::Value>> {
            v.as_array()
                .unwrap()
                .iter()
                .map(|r| r.as_object().unwrap().clone())
                .collect()
        };
        let hits = SourceId::new("test", "hits");
        let units = SourceId::new("test", "units");
        for (source, items) in [
            (&hits, json!([{"id": "a", "entity": 1, "amount": 12}])),
            (&units, json!([{"id": "u", "entity": 1, "hp": 50}])),
        ] {
            ui.reactive
                .execute(Command::Collection {
                    source: source.clone(),
                    key: "id".into(),
                    items: rows(items),
                })
                .unwrap();
        }
        ui.mount_document(
            SourceId::new("test", "hits"),
            "hits",
            BTreeMap::from([
                ("hits".to_string(), hits.clone()),
                ("units".to_string(), units.clone()),
            ]),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui| {
            ui.set_world_view(Some(WorldView::look_at(
                Vec3::new(0.0, 3.0, 10.0),
                Vec3::ZERO,
                Vec3::Y,
                50.0,
                800.0 / 600.0,
                0.1,
                100.0,
            )));
            ui.set_world_poses(&[(1u64, (Vec3::ZERO, Quat::IDENTITY, Vec3::ONE))]);
            ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end().stats
        };
        for _ in 0..3 {
            tick(&mut ui);
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let damage = ui.mass_handle("damage").expect("the damage template");
        let hp = ui.mass_handle("hp").expect("the hp template");
        assert_eq!(ui.mass_count(damage), 1);
        let t = &ui.world.templates[damage.0 as usize].1;
        assert_eq!(t.lifetime, ui.ticks_per_second);
        let last = t.keys.last().expect("keys");
        assert_eq!((last.t, last.offset.y, last.alpha), (1.0, -40.0, 0.0));
        let bars = |ui: &Ui| {
            ui.world_frame()
                .batches
                .iter()
                .filter(|b| matches!(b.template.kind, og_render::UiMassKind::Bar { .. }))
                .map(|b| b.instances.len())
                .sum::<usize>()
        };
        let idle = tick(&mut ui);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
        assert_eq!(bars(&ui), 1);
        // A second hit spawns one more; the first is not spawned again.
        ui.reactive
            .execute(Command::Insert {
                source: hits.clone(),
                row: json!({"id": "b", "entity": 1, "amount": 30})
                    .as_object()
                    .unwrap()
                    .clone(),
                before: None,
            })
            .unwrap();
        tick(&mut ui);
        assert_eq!(ui.mass_count(damage), 2);
        assert!(hp != damage);
        // The unit leaves: its bar does.
        ui.reactive
            .execute(Command::Remove {
                source: units,
                key: "u".into(),
            })
            .unwrap();
        for _ in 0..2 {
            tick(&mut ui);
        }
        assert_eq!(bars(&ui), 0);
    }

    /// `-og-emitter` and `-og-meta` stay registered on the ticks their
    /// element is not built again, and leave with it; `-og-burst` in a
    /// keyframe is a burst event.
    ///
    /// Ways it could fail: an emitter that stops on the first idle tick (it
    /// was registered only while built), one that outlives its element, or
    /// a burst dropped by the keyframe parser.
    #[test]
    fn css_emitters_and_metas_last_while_their_element_stands() {
        let doc = r#"<meta name="component" content="fx">
<div id="Root">
  {#if $on}<div id="Gem"></div>{/if}
  <div id="Hurt"></div>
</div>
<style>
#Gem { width: 40px; height: 40px; -og-emitter: 30 diamond 6px #ffd166; animation: pop 1s; }
#Hurt { -og-meta: vignette #aa0000; opacity: 0.5; }
@keyframes pop { 50% { -og-burst: 24 48px #ffffff; } }
</style>"#;
        let mut ui = Ui::new(UiDesc::default());
        let parsed = UiDocument::parse("fx.svelte", doc).unwrap();
        let pop = &parsed.keyframes.iter().find(|(k, _)| k == "pop").unwrap().1;
        assert!(matches!(
            pop.events.as_slice(),
            [(o, crate::motion::EventKind::Burst { count: 24, .. })] if *o == 0.5
        ));
        ui.define_document(parsed);
        let on = SourceId::new("test", "on");
        ui.reactive
            .execute(Command::Signal {
                source: on.clone(),
                initial: json!(true),
                readable: false,
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "fx"),
            "fx",
            BTreeMap::from([("on".to_string(), on.clone())]),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end().stats
        };
        for _ in 0..3 {
            tick(&mut ui);
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let idle = tick(&mut ui);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
        let particles = |ui: &Ui| {
            ui.world
                .state
                .particles
                .values()
                .map(|e| e.spawned)
                .sum::<u32>()
        };
        let before = particles(&ui);
        for _ in 0..10 {
            tick(&mut ui);
        }
        assert!(particles(&ui) > before, "the emitter stopped while idle");
        assert!(ui.world.kept_effects.len() == 2);
        ui.reactive
            .execute(Command::Set {
                source: on,
                value: json!(false),
                force: false,
            })
            .unwrap();
        for _ in 0..2 {
            tick(&mut ui);
        }
        assert_eq!(
            ui.world.kept_effects.len(),
            1,
            "the gem's emitter outlived it"
        );
    }
}
