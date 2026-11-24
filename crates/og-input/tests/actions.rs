//! Gameplay actions over the input resources, end to end: keys injected
//! between ticks drive the actions a game defined, and action state is
//! world state, so a snapshot restored into a fresh world replays the rest
//! to the same states and hash.

use og_ecs::{World, WorldDesc};
use og_input::{
    ActionDesc, ActionKind, ActionMap, ActionState, Binding, InjectedInput, InputLayer, InputSink,
    KeyAction, Modifiers, SharedActions,
};
use std::sync::{Arc, Mutex};

struct Rig {
    world: World,
    layer: InputLayer,
    map: SharedActions,
}

impl Rig {
    fn new(actions: &[ActionDesc]) -> Rig {
        let mut world = World::new(&WorldDesc::default()).unwrap();
        let mut layer = InputLayer::new();
        layer.attach(&mut world).unwrap();
        let map: SharedActions = Arc::new(Mutex::new(ActionMap::new()));
        og_input::install(&mut world, map.clone(), || None).unwrap();
        for a in actions {
            map.lock().unwrap().define(a.clone()).unwrap();
        }
        Rig { world, layer, map }
    }

    /// Delivers `events` between ticks, then runs one tick.
    fn tick(&mut self, events: &[InjectedInput]) {
        for e in events {
            assert!(self.layer.inject(&mut self.world, e), "refused: {e:?}");
        }
        self.world.step().unwrap();
        self.layer.end_tick(&mut self.world);
    }

    fn state(&self, name: &str) -> ActionState {
        self.map.lock().unwrap().state(name).unwrap()
    }
}

fn key(code: &str, action: KeyAction) -> InjectedInput {
    InjectedInput::Key {
        key: code.into(),
        action,
        modifiers: Modifiers::default(),
    }
}

fn down(code: &str) -> InjectedInput {
    key(code, KeyAction::Down)
}

fn up(code: &str) -> InjectedInput {
    key(code, KeyAction::Up)
}

fn action(name: &str, kind: ActionKind, bindings: Vec<Binding>) -> ActionDesc {
    ActionDesc {
        name: name.into(),
        kind,
        bindings,
        ..ActionDesc::default()
    }
}

#[test]
fn actions_are_world_state_and_replay() {
    let defs = [
        ActionDesc {
            hold_ms: 50.0,
            ..action("charge", ActionKind::Button, vec![Binding::control("KeyF")])
        },
        action(
            "move",
            ActionKind::Axis2,
            vec![Binding::plane("KeyW", "KeyS", "KeyA", "KeyD")],
        ),
    ];
    let steps: Vec<Vec<InjectedInput>> = vec![
        vec![down("KeyF"), down("KeyW")],
        vec![],
        vec![down("KeyA")],
        vec![],
        vec![up("KeyF")],
        vec![up("KeyW"), up("KeyA")],
    ];
    let mut a = Rig::new(&defs);
    let mut seen_a = Vec::new();
    for s in &steps[..2] {
        a.tick(s);
    }
    let snap = a.world.snapshot();
    let at_snap = a.world.world_hash();
    for s in &steps[2..] {
        a.tick(s);
        seen_a.push((a.state("charge"), a.state("move")));
    }
    // A fresh world restored from the snapshot: the actions and their
    // state (the hold under way) come with it.
    let mut b = Rig::new(&[]);
    b.world.restore(&snap).unwrap();
    // Restored, the block hashes as it was saved (every saved field,
    // the map's tick included, comes back).
    assert_eq!(b.world.world_hash(), at_snap);
    let mut seen_b = Vec::new();
    for s in &steps[2..] {
        b.tick(s);
        seen_b.push((b.state("charge"), b.state("move")));
    }
    assert_eq!(seen_a, seen_b);
    assert_eq!(a.world.world_hash(), b.world.world_hash());
    // The hold: 50 ms is three ticks at 60 Hz, reached on the fourth.
    assert!(seen_a[1].0.held, "{:?}", seen_a[1].0);
    assert!(!seen_a[0].0.held);
}
