//! Native district entities and scene outputs, shared with the author cooker.
use super::parse::PropCollider;
use crate::cook::source_schemas::{
    ContentRef,
    world::scene::{EntityInput, HlodInput, SceneInput, StreamInput, StreamTriggerInput},
};
use glam::{DQuat, DVec3};
use serde_json::{Map, Value, json};

/// Keep the builder's tenth-of-a-millimetre quantization on every host.
fn numbers(xs: &[f64]) -> Value {
    json!(
        xs.iter()
            .map(|x| {
                let r = (x * 10_000.0).round() / 10_000.0;
                if r == 0.0 { 0.0 } else { r }
            })
            .collect::<Vec<_>>()
    )
}

fn entity(name: &str, p: DVec3, q: DQuat) -> EntityInput {
    let mut transform = Map::from_iter([("position".into(), numbers(&p.to_array()))]);
    if q != DQuat::IDENTITY {
        transform.insert("quaternion".into(), numbers(&q.to_array()));
    }
    EntityInput {
        name: name.into(),
        components: [("og.Transform".into(), transform)].into(),
        ..Default::default()
    }
}

fn collider_entity(name: &str, c: &PropCollider) -> EntityInput {
    let (lift, fields) = match c {
        PropCollider::Box(s) => (s[1] / 2.0, json!({"shape":"box","size":numbers(s)})),
        PropCollider::Capsule(r, h) => (
            r + h / 2.0,
            json!({"shape":"capsule","radius":r,"height":h}),
        ),
    };
    let mut e = entity(name, DVec3::new(0.0, lift, 0.0), DQuat::IDENTITY);
    e.components
        .insert("og.Collider".into(), fields.as_object().unwrap().clone());
    e
}

pub(super) fn placed(
    name: &str,
    model: &str,
    p: DVec3,
    q: DQuat,
    c: Option<&PropCollider>,
) -> EntityInput {
    let mut e = entity(name, p, q);
    e.model = Some(ContentRef { id: model.into() });
    e.static_model = Some(true);
    if let Some(c) = c {
        e.children
            .push(collider_entity(&format!("{name}/collider"), c));
    }
    e
}

pub(super) fn collider(name: &str, p: DVec3, q: DQuat, c: &PropCollider) -> EntityInput {
    let PropCollider::Box(s) = c else {
        unreachable!("a building collides as a box")
    };
    let mut e = entity(name, p, q);
    e.components.insert(
        "og.Collider".into(),
        json!({"shape":"box","size":numbers(s)})
            .as_object()
            .unwrap()
            .clone(),
    );
    e
}

pub(super) fn cell(
    name: &str,
    scene: &str,
    at: DVec3,
    size: f64,
    height: f64,
    load: f64,
) -> EntityInput {
    let mut e = entity(name, at, DQuat::IDENTITY);
    e.scene = Some(ContentRef { id: scene.into() });
    e.components.insert(
        "og.Volume".into(),
        json!({"size":numbers(&[size,2.0*height,size])})
            .as_object()
            .unwrap()
            .clone(),
    );
    e.components.insert(
        "og.StreamCell".into(),
        json!({"load":load}).as_object().unwrap().clone(),
    );
    e
}

pub(super) fn scene(
    id: &str,
    hlod: Option<&HlodInput>,
    group: Option<(&str, &[String])>,
    entities: &[EntityInput],
) -> SceneInput {
    let mut scene = SceneInput {
        name: Some(id.into()),
        entities: entities.into(),
        hlod: hlod.cloned(),
        ..Default::default()
    };
    if let Some((name, assets)) = group {
        scene.streams.insert(
            name.into(),
            StreamInput {
                assets: assets
                    .iter()
                    .map(|id| ContentRef { id: id.clone() })
                    .collect(),
                trigger: Some(StreamTriggerInput::Cell),
                priority: None,
            },
        );
    }
    scene
}
