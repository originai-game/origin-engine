//! The accessibility tree over the shared-frames wire: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/a11y.rs.

use crate::support;
use og_ui::*;
use support::*;

#[path = "../../og-ui/tests/a11y/form.rs"]
mod form;
use form::*;

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_tree_is_incremental_and_actions_round_trip_on_the_tick() {
    use native_a11y::{
        Bridge,
        accesskit::{Action, ActionData, ActionRequest, Role, TreeId},
    };
    use og_render::shared_frames::{Msg, recv, send};
    let mut ui = ui_with_font(test_font_metrics());
    let mut f = Form {
        volume: 35.0,
        name: "Ada".into(),
        ..Default::default()
    };
    tick(&mut ui, &mut f, InputFrame::default());
    let mut bridge = Bridge::default();
    let update = bridge.update(&ui, [2.0, 3.0]).unwrap();
    let (volume, slider) = update
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Slider && n.numeric_value() == Some(35.0))
        .unwrap();
    let volume = *volume;
    assert_eq!(slider.min_numeric_value(), Some(0.0));
    assert_eq!(slider.max_numeric_value(), Some(100.0));
    let source = node(ui.a11y(), "Form/Volume/Track");
    assert_eq!(slider.bounds().unwrap().x0, f64::from(source.x) * 2.0);
    assert_eq!(
        slider.bounds().unwrap().y1,
        f64::from(source.y + source.height) * 3.0
    );
    let name = update
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::TextInput && n.value() == Some("Ada"))
        .unwrap()
        .0;
    let check = update
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::CheckBox && n.label() == Some("Show hints"))
        .unwrap()
        .0;
    let message = Msg::AccessibilityUpdate {
        version: 1,
        generation: 2,
        seq: 19,
        update: serde_json::to_value(&update).unwrap(),
    };
    let mut bytes = Vec::new();
    send(&mut bytes, &message).unwrap();
    assert_eq!(recv(&mut bytes.as_slice()).unwrap(), message);
    assert!(bridge.update(&ui, [2.0, 3.0]).is_none());
    let request = |target_node, action, data| ActionRequest {
        action,
        target_node,
        target_tree: TreeId::ROOT,
        data,
    };
    let dispatch = |ui: &mut Ui, request: ActionRequest| {
        let msg = Msg::AccessibilityAction {
            version: 1,
            request: serde_json::to_value(request).unwrap(),
        };
        let mut bytes = Vec::new();
        send(&mut bytes, &msg).unwrap();
        let Msg::AccessibilityAction { request, .. } = recv(&mut bytes.as_slice()).unwrap() else {
            panic!("action")
        };
        ui.queue_accessibility(
            bridge
                .action(serde_json::from_value(request).unwrap())
                .unwrap(),
        );
    };
    dispatch(
        &mut ui,
        request(
            volume,
            Action::SetValue,
            Some(ActionData::NumericValue(73.0)),
        ),
    );
    assert_eq!(f.volume, 35.0, "socket receipt does not change simulation");
    tick(&mut ui, &mut f, InputFrame::default());
    assert_eq!(f.volume, 75.0, "slider quantizes to its step");
    dispatch(
        &mut ui,
        request(
            name,
            Action::SetValue,
            Some(ActionData::Value("Grace".into())),
        ),
    );
    tick(&mut ui, &mut f, InputFrame::default());
    assert_eq!(f.name, "Grace");
    dispatch(&mut ui, request(check, Action::Click, None));
    tick(&mut ui, &mut f, InputFrame::default());
    assert!(f.check);
    let diff = bridge.update(&ui, [2.0, 3.0]).unwrap();
    assert!(diff.tree.is_none());
    assert!(diff.nodes.len() < update.nodes.len());
    assert_eq!(diff.focus, check);
    assert!(
        diff.nodes
            .iter()
            .any(|(id, n)| *id == volume && n.numeric_value() == Some(75.0))
    );
    ui.begin(InputFrame::default(), Viewport::new(800.0, 1000.0));
    ui.end();
    bridge.update(&ui, [2.0, 3.0]).unwrap();
    assert!(
        bridge.action(request(check, Action::Click, None)).is_none(),
        "deleted ids reject delayed actions"
    );
}
