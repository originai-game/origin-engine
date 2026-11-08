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

/// Its camera for an element `aspect` wide over tall: orbiting its
/// target at yaw, pitch (from straight up, as `<model-viewer>`) and
/// radius; `None` without a field of view.
pub(super) fn camera(r: &Resolved, aspect: f32) -> Option<WorldView> {
    let fov = r
        .str("fieldOfView")
        .map_or(Some(50.0), |v| numbers(v).first().copied())?;
    let target = match numbers(r.str("cameraTarget").unwrap_or("0 1 0")).as_slice() {
        [x, y, z] => Vec3::new(*x, *y, *z),
        _ => Vec3::new(0.0, 1.0, 0.0),
    };
    let [yaw, pitch, radius] = match numbers(r.str("cameraOrbit").unwrap_or("0 75 3")).as_slice() {
        [a, b, c] => [a.to_radians(), b.to_radians(), *c],
        _ => [0.0, 75f32.to_radians(), 3.0],
    };
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

impl Ui {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_viewport(
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
        let out = self.scene_viewport(
            id,
            ViewportDesc {
                texture: r
                    .num("viewportSrc")
                    .and_then(|h| b.env.texture(f64::from(h))),
                width: r.size("width").unwrap_or(Size::Fixed(320.0)),
                height: r.size("height").unwrap_or(Size::Fixed(320.0)),
                aspect: r.num("aspect").unwrap_or(0.0),
                view: camera(r, aspect),
                background: r.color("background"),
                radius: r.num("radius").unwrap_or(0.0),
                label: r.str("name").unwrap_or_default().to_string(),
            },
        );
        let i = self.last_closed;
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        self.nodes[i].props = p;
        if let Some(name) = r.str("name").filter(|n| !n.is_empty()) {
            self.nodes[i].a11y.name = Some(name.to_string());
        }
        // Svelte's dimension bindings: written when its box changes (a
        // resized node is built again), so the game sizes its target.
        for (prop, v) in [
            ("clientWidth", out.rect.width),
            ("clientHeight", out.rect.height),
        ] {
            if super::nodes::bound(n, prop) && r.num(prop) != Some(v) {
                self.write_back(b, n, prop, serde_json::json!(v));
            }
        }
        self.note_built(built);
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
            ui.reactive
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
            ui.reactive
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
}
