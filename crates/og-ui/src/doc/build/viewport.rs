//! `<viewport>` in documents (docs/ui.md §3.3): a camera's render target
//! shown in the layout ([`crate::Ui::scene_viewport`]), its camera written
//! as `<model-viewer>` writes one (`camera-orbit`, `camera-target`,
//! `field-of-view`), its size bound back with Svelte's `bind:clientWidth`
//! and `bind:clientHeight`, and its pointer events carrying the ray into
//! its scene ([`super::retained`]'s input pass).
use super::*;
use crate::viewport::ViewportDesc;
use crate::world::WorldView;
use og_core::glam::Vec3;

/// The numbers of a `<model-viewer>` value (`45deg 75deg 3m`, `0m 1m 0m`),
/// or of a bound array.
fn numbers(text: &str) -> Vec<f32> {
    text.replace(['[', ']', ','], " ")
        .split_whitespace()
        .filter_map(|w| {
            w.trim_end_matches("deg")
                .trim_end_matches("rad")
                .trim_end_matches('m')
                .parse::<f32>()
                .ok()
        })
        .collect()
}

/// `<model-viewer>`'s camera-orbit (yaw, pitch from straight up, radius),
/// degrees and metres.
fn orbit_of(text: Option<&str>) -> [f32; 3] {
    match numbers(text.unwrap_or("0 75 3")).as_slice() {
        [a, b, c] => [*a, *b, *c],
        _ => [0.0, 75.0, 3.0],
    }
}

/// A `min-camera-orbit` or `max-camera-orbit` bound: each of yaw, pitch and
/// radius a number, or `auto` (`fallback`'s).
fn orbit_bound(text: Option<&str>, fallback: [f32; 3]) -> [f32; 3] {
    let mut out = fallback;
    for (k, w) in text.unwrap_or("").split_whitespace().take(3).enumerate() {
        if let Some(v) = numbers(w).first() {
            out[k] = *v;
        }
    }
    out
}

/// Its camera for an element `aspect` wide over tall: orbiting its
/// target at yaw, pitch (from straight up, as `<model-viewer>`) and
/// radius; `None` without a field of view.
pub(super) fn camera(r: &Resolved, aspect: f32) -> Option<WorldView> {
    camera_at(r, aspect, orbit_of(r.str("cameraOrbit")))
}

fn camera_at(r: &Resolved, aspect: f32, [yaw, pitch, radius]: [f32; 3]) -> Option<WorldView> {
    let fov = r
        .str("fieldOfView")
        .map_or(Some(50.0), |v| numbers(v).first().copied())?;
    let target = match numbers(r.str("cameraTarget").unwrap_or("0 1 0")).as_slice() {
        [x, y, z] => Vec3::new(*x, *y, *z),
        _ => Vec3::new(0.0, 1.0, 0.0),
    };
    let (yaw, pitch) = (yaw.to_radians(), pitch.to_radians());
    let eye = target
        + radius
            * Vec3::new(
                pitch.sin() * yaw.sin(),
                pitch.cos(),
                pitch.sin() * yaw.cos(),
            );
    Some(WorldView::look_at(
        eye,
        target,
        Vec3::Y,
        fov,
        aspect,
        0.05,
        500.0,
    ))
}

pub(crate) trait ViewportUi {
    /// `<model-viewer>`'s camera-controls on viewport `id`: a drag orbits
    /// it (and keeps turning, slowing, when flung), the wheel and two
    /// fingers zoom it, and with the focus Left and Right turn it 15° and
    /// Up and Down tilt it 10°, all within `min-camera-orbit` and
    /// `max-camera-orbit` (pitch 22.5°–157.5° and any yaw and radius by
    /// default). The orbit lives in the UI's memory, so snapshots carry it;
    /// a new authored orbit takes over. Answers the orbit this tick.
    fn orbit_controls(
        &mut self,
        id: &str,
        r: &Resolved,
        out: &crate::viewport::ViewportResponse,
        bound: bool,
    ) -> [f32; 3];

    #[allow(clippy::too_many_arguments)]
    fn build_viewport(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
        props: Props,
        path: String,
        key: u64,
        built: Built,
    ) -> Option<String>;
}

impl ViewportUi for Ui {
    fn orbit_controls(
        &mut self,
        id: &str,
        r: &Resolved,
        out: &crate::viewport::ViewportResponse,
        bound: bool,
    ) -> [f32; 3] {
        let authored = orbit_of(r.str("cameraOrbit"));
        let lo = orbit_bound(r.str("minCameraOrbit"), [f32::NEG_INFINITY, 22.5, 0.0]);
        let hi = orbit_bound(
            r.str("maxCameraOrbit"),
            [f32::INFINITY, 157.5, f32::INFINITY],
        );
        let mem =
            |ui: &mut Ui, k: &str, v: f32| ui.memory(&format!("{id}~{k}"), f64::from(v)) as f32;
        let seen = [
            mem(self, "seen-yaw", authored[0]),
            mem(self, "seen-pitch", authored[1]),
            mem(self, "seen-radius", authored[2]),
        ];
        let mut o = [
            mem(self, "yaw", authored[0]),
            mem(self, "pitch", authored[1]),
            mem(self, "radius", authored[2]),
        ];
        let mut v = [mem(self, "vyaw", 0.0), mem(self, "vpitch", 0.0)];
        let spread = mem(self, "spread", 0.0);
        if seen != authored {
            o = authored;
            v = [0.0, 0.0];
        }
        let reduced = self.theme().reduced_motion;
        let hit = self
            .interaction
            .hit(self.tree.nodes[self.tree.last_closed].key)
            .copied();
        let pinch = super::gestures::two_touches(&self.interaction.input, |x, y| {
            hit.is_some_and(|h| h.contains(x, y))
        });
        if let Some((_, now)) = pinch {
            if spread > 0.0 {
                o[2] *= spread / now;
            }
            v = [0.0, 0.0];
        } else if out.pressed {
            // The scene turns with the finger.
            v = [-out.drag.x * 0.4, -out.drag.y * 0.3];
            o[0] += v[0];
            o[1] += v[1];
        } else if !reduced && v[0].abs().max(v[1].abs()) > 0.02 {
            o[0] += v[0];
            o[1] += v[1];
            v = [v[0] * 0.9, v[1] * 0.9];
        } else {
            v = [0.0, 0.0];
        }
        o[2] *= 1.0 + out.wheel.y * 0.0015;
        o[0] += 15.0 * out.adjust as f32;
        o[1] += 10.0 * out.adjust_y as f32;
        for k in 0..3 {
            o[k] = o[k].clamp(lo[k], hi[k].max(lo[k]));
        }
        let put = |ui: &mut Ui, k: &str, x: f32| ui.remember(&format!("{id}~{k}"), f64::from(x));
        for (k, x) in [
            ("yaw", o[0]),
            ("pitch", o[1]),
            ("radius", o[2]),
            ("vyaw", v[0]),
            ("vpitch", v[1]),
        ] {
            put(self, k, x);
        }
        put(self, "spread", pinch.map_or(0.0, |(_, s)| s));
        // What it writes back is the authored orbit next tick.
        let next = if bound { o } else { authored };
        for (k, x) in [
            ("seen-yaw", next[0]),
            ("seen-pitch", next[1]),
            ("seen-radius", next[2]),
        ] {
            put(self, k, x);
        }
        o
    }

    #[allow(clippy::too_many_arguments)]
    fn build_viewport(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
        props: Props,
        path: String,
        key: u64,
        built: Built,
    ) -> Option<String> {
        let last = self.element(&path).map(|e| (e.width, e.height));
        let aspect = last.map_or(1.0, |(w, h)| w / h.max(1.0));
        let controls = r.flag("cameraControls");
        let out = self.scene_viewport(
            id,
            ViewportDesc {
                texture: r.handle("viewportSrc").and_then(|h| b.env.texture(h)),
                width: r.size("width").unwrap_or(Size::Fixed(320.0)),
                height: r.size("height").unwrap_or(Size::Fixed(320.0)),
                aspect: r.num("aspect").unwrap_or(0.0),
                view: camera(r, aspect),
                background: r.color("background"),
                radius: r.num("radius").unwrap_or(0.0),
                label: r.str("name").unwrap_or_default().to_string(),
                controls,
            },
        );
        let i = self.tree.last_closed;
        if controls {
            let bound = super::nodes::bound(n, "cameraOrbit");
            let o = self.orbit_controls(id, r, &out, bound);
            let text = format!("{}deg {}deg {}m", o[0], o[1], o[2]);
            if bound && r.str("cameraOrbit") != Some(text.as_str()) {
                self.docs
                    .write_back(b, n, "cameraOrbit", serde_json::json!(text));
            }
        }
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        *self.tree.props_mut(i) = p;
        if let Some(name) = r.str("name").filter(|n| !n.is_empty()) {
            self.tree.nodes[i].a11y.name = Some(name.to_string());
        }
        // Svelte's dimension bindings: written when its box changes (a
        // resized node is built again), so the game sizes its target.
        for (prop, v) in [
            ("clientWidth", out.rect.width),
            ("clientHeight", out.rect.height),
        ] {
            if super::nodes::bound(n, prop) && r.num(prop) != Some(v) {
                self.docs.write_back(b, n, prop, serde_json::json!(v));
            }
        }
        self.docs.note_built(built);
        let resp = crate::ui::Response {
            hovered: out.hovered,
            pressed: out.pressed,
            clicked: out.clicked,
            ..Default::default()
        };
        self.fire(b, n, key, &path, resp);
        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::UiDocument;
    use crate::doc::build::ext::*;
    use crate::reactive::SourceId;
    use crate::reactive::session::Command;
    use crate::{InputFrame, Ui, UiDesc, Viewport};
    use og_core::glam::Vec2;
    use serde_json::json;
    use std::collections::BTreeMap;

    const VIEW: &str = r#"
<meta name="component" content="garage">
<div id="Root">
  <viewport id="Car" camera-orbit="90deg 90deg 3m" camera-target="0m 1m 0m"
    bind:clientWidth={$w} bind:clientHeight={$h} onpointerdown={pick} aria-label="Car"/>
</div>
<style>#Root { padding: 0; } #Car { width: 200px; height: 100px; }</style>
"#;

    #[test]
    fn a_viewport_binds_its_size_and_rays_its_presses() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("garage.svelte", VIEW).unwrap());
        let mut inputs = BTreeMap::new();
        for name in ["w", "h"] {
            let source = SourceId::new("test", name);
            ui.docs
                .reactive
                .execute(Command::Signal {
                    source: source.clone(),
                    initial: json!(0),
                    readable: false,
                })
                .unwrap();
            inputs.insert(name.to_string(), source);
        }
        ui.mount_document(
            SourceId::new("test", "garage"),
            "garage",
            inputs,
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui, at: Vec2, down: bool, was: bool| {
            ui.begin(
                InputFrame {
                    pointer: at,
                    pointer_inside: true,
                    down,
                    pressed: down && !was,
                    released: !down && was,
                    ..Default::default()
                },
                Viewport::new(400.0, 300.0),
            );
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            ui.take_doc_events()
        };
        let away = Vec2::new(390.0, 290.0);
        for _ in 0..4 {
            tick(&mut ui, away, false, false);
        }
        let read = |ui: &mut Ui, n: &str| {
            ui.docs
                .reactive
                .execute(Command::Read {
                    source: SourceId::new("test", n),
                })
                .unwrap()["value"]
                .clone()
        };
        assert_eq!(
            (read(&mut ui, "w"), read(&mut ui, "h")),
            (json!(200.0), json!(100.0))
        );
        let stats = tick(&mut ui, away, false, false);
        assert!(stats.is_empty());
        let idle = ui.frame().stats;
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
        let car = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path == "test:garage/Car")
            .unwrap();
        let centre = Vec2::new(car.x + car.width / 2.0, car.y + car.height / 2.0);
        tick(&mut ui, centre, false, false);
        let events = tick(&mut ui, centre, true, false);
        let pick = events.iter().find(|e| e.name == "pick").expect("a pick");
        let ray = &pick.value["ray"];
        // From yaw 90°, level, 3 m from (0, 1, 0): at (3, 1, 0), looking -x.
        let o: Vec<f64> = serde_json::from_value(ray["origin"].clone()).unwrap();
        let d: Vec<f64> = serde_json::from_value(ray["direction"].clone()).unwrap();
        assert!(
            (o[0] - 3.0).abs() < 0.1 && (o[1] - 1.0).abs() < 0.1 && o[2].abs() < 0.1,
            "{ray}"
        );
        assert!(d[0] < -0.99, "{ray}");
    }

    /// `camera-controls`: a drag orbits within `max-camera-orbit`, a fling
    /// keeps turning and slows to rest, and two fingers fire `onpinch` with
    /// their scale and zoom the orbit.
    ///
    /// Ways it could fail: a fling that stops on release (the viewport not
    /// built while idle), a pitch past its limit, a pinch that never fires
    /// or zooms the wrong way.
    #[test]
    fn camera_controls_orbit_fling_clamp_and_pinch() {
        let doc = r#"<meta name="component" content="turntable">
<div id="Root"><viewport id="View" camera-controls camera-orbit="0deg 75deg 4m"
  max-camera-orbit="auto 80deg 6m" bind:cameraOrbit={$orbit} onpinch={zoom}/></div>
<style>#Root { padding: 0; } #View { width: 300px; height: 200px; }</style>"#;
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("turntable.svelte", doc).unwrap());
        let orbit = SourceId::new("test", "orbit");
        ui.docs
            .reactive
            .execute(Command::Signal {
                source: orbit.clone(),
                initial: json!("0deg 75deg 4m"),
                readable: false,
            })
            .unwrap();
        ui.mount_document(
            SourceId::new("test", "turntable"),
            "turntable",
            BTreeMap::from([("orbit".to_string(), orbit.clone())]),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui, input: InputFrame| {
            ui.begin(input, Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let events = ui.take_doc_events();
            let o = ui
                .docs
                .reactive
                .execute(Command::Read {
                    source: orbit.clone(),
                })
                .unwrap()["value"]
                .as_str()
                .unwrap()
                .to_string();
            let n: Vec<f32> = o
                .split_whitespace()
                .map(|w| w.trim_end_matches(char::is_alphabetic).parse().unwrap())
                .collect();
            (n, events)
        };
        let at = |x: f32, y: f32, down: bool, was: bool| InputFrame {
            pointer: Vec2::new(x, y),
            pointer_inside: true,
            down,
            pressed: down && !was,
            released: !down && was,
            ..Default::default()
        };
        for _ in 0..3 {
            tick(&mut ui, at(150.0, 100.0, false, false));
        }
        tick(&mut ui, at(150.0, 100.0, true, false));
        let mut o = Vec::new();
        for k in 1..=4 {
            o = tick(
                &mut ui,
                at(150.0 - 20.0 * k as f32, 100.0 - 20.0 * k as f32, true, true),
            )
            .0;
        }
        assert!(o[0] > 20.0, "a drag to the left turns it: {o:?}");
        assert_eq!(o[1], 80.0, "the pitch stops at its limit: {o:?}");
        let released = tick(&mut ui, at(70.0, 20.0, false, true)).0;
        let later = tick(&mut ui, at(70.0, 20.0, false, false)).0;
        assert!(
            later[0] > released[0],
            "a fling keeps turning: {released:?} {later:?}"
        );
        for _ in 0..120 {
            tick(&mut ui, at(70.0, 20.0, false, false));
        }
        let rest = tick(&mut ui, at(70.0, 20.0, false, false)).0;
        assert_eq!(
            rest,
            tick(&mut ui, at(70.0, 20.0, false, false)).0,
            "it comes to rest"
        );
        // Two fingers spreading: a pinch of scale 2 about their centre, the
        // orbit's distance halved.
        let touches = |d: f32| {
            let p = og_input::Pointers {
                held: 0b110,
                start1: Vec2::new(130.0, 100.0),
                start2: Vec2::new(170.0, 100.0),
                position1: Vec2::new(150.0 - d, 100.0),
                position2: Vec2::new(150.0 + d, 100.0),
                ..Default::default()
            };
            InputFrame {
                pointer: Vec2::new(150.0, 100.0),
                pointer_inside: true,
                pointers: p,
                ..Default::default()
            }
        };
        tick(&mut ui, touches(20.0));
        let (o, events) = tick(&mut ui, touches(40.0));
        let pinch = events
            .iter()
            .find(|e| e.kind == "emit" && e.name == "zoom")
            .unwrap_or_else(|| panic!("no pinch: {events:?}"));
        assert!(
            (pinch.value["scale"].as_f64().unwrap() - 2.0).abs() < 1e-3,
            "{:?}",
            pinch.value
        );
        assert!(
            (pinch.value["x"].as_f64().unwrap() - 150.0).abs() < 1.0,
            "{:?}",
            pinch.value
        );
        assert!((o[2] - rest[2] / 2.0).abs() < 1e-3, "{rest:?} {o:?}");
    }
}
