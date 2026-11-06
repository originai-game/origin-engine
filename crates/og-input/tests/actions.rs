//! Gameplay actions over the input resources, as the engine samples them
//! at the start of each tick. The failure modes written down first: a
//! keyboard composite and a stick that read on different scales or y
//! senses; a stick inside its dead zone that moves; a diagonal that is
//! faster than a straight line; a tap between two ticks that leaves no
//! edge; an analog button that chatters around its threshold; a key two
//! contexts bind firing both; a rebinding that silently steals a key; a
//! capture whose key also fires the action; bindings that do not come
//! back from their text; a pad past the fourth refused, or its family
//! lost; action state a snapshot does not carry.

use og_ecs::{World, WorldDesc};
use og_input::{
    ActionDesc, ActionKind, ActionMap, ActionState, Binding, ConflictPolicy, Device, GamepadState,
    InjectedInput, InputLayer, InputSink, KeyAction, Modifiers, RebindDesc, SharedActions,
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

fn pad(
    pad: u32,
    buttons: &[&str],
    axes: [f32; 4],
    triggers: [f32; 2],
    device: Option<Device>,
) -> InjectedInput {
    InjectedInput::Gamepad(GamepadState {
        pad,
        connected: true,
        buttons: buttons
            .iter()
            .map(|b| 1 << og_input::gamepad_button(b).expect("a standard button"))
            .sum(),
        axes,
        triggers,
        device,
    })
}

fn action(name: &str, kind: ActionKind, bindings: Vec<Binding>) -> ActionDesc {
    ActionDesc {
        name: name.into(),
        kind,
        bindings,
        ..ActionDesc::default()
    }
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

#[test]
fn rebinding_reports_conflicts_and_swaps_or_replaces() {
    let mut map = ActionMap::new();
    map.define(action(
        "jump",
        ActionKind::Button,
        vec![Binding::control("Space")],
    ))
    .unwrap();
    map.define(action(
        "crouch",
        ActionKind::Button,
        vec![Binding::control("KeyC")],
    ))
    .unwrap();
    map.define(ActionDesc {
        locked: true,
        ..action(
            "pause",
            ActionKind::Button,
            vec![Binding::control("Escape")],
        )
    })
    .unwrap();
    map.define(ActionDesc {
        context: "menu".into(),
        ..action("back", ActionKind::Button, vec![Binding::control("KeyC")])
    })
    .unwrap();
    let to = |control: &str, policy| RebindDesc {
        action: "jump".into(),
        slot: 0,
        part: String::new(),
        binding: Binding::control(control),
        policy,
    };
    // Refused: nothing changes, and the answer names crouch (not the menu's
    // back, which is another context).
    let a = map.rebind(&to("KeyC", ConflictPolicy::Refuse)).unwrap();
    assert!(!a.applied);
    assert_eq!(a.conflicts.len(), 1);
    assert_eq!(
        (a.conflicts[0].action.as_str(), a.conflicts[0].slot),
        ("crouch", 0)
    );
    assert_eq!(map.bindings("jump").unwrap()[0].control, "Space");
    // Swap: crouch takes jump's old Space.
    assert!(
        map.rebind(&to("KeyC", ConflictPolicy::Swap))
            .unwrap()
            .applied
    );
    assert_eq!(map.bindings("jump").unwrap()[0].control, "KeyC");
    assert_eq!(map.bindings("crouch").unwrap()[0].control, "Space");
    // Replace: jump takes Space back and crouch is left unbound.
    assert!(
        map.rebind(&to("Space", ConflictPolicy::Replace))
            .unwrap()
            .applied
    );
    assert_eq!(map.bindings("crouch").unwrap()[0].control, "");
    // A locked action's control does not move, and a locked action is not rebound.
    assert!(map.rebind(&to("Escape", ConflictPolicy::Swap)).is_err());
    assert!(
        map.rebind(&RebindDesc {
            action: "pause".into(),
            ..to("KeyP", ConflictPolicy::Allow)
        })
        .is_err()
    );
    // A stick on a button action, or a part a button has not, is refused with the fix.
    let e = map
        .rebind(&to("pad.leftXY", ConflictPolicy::Allow))
        .unwrap_err();
    assert!(e.contains("two axes"), "{e}");
    // One part of a composite: WASD's up becomes Z, and its conflict is named.
    map.define(action(
        "move",
        ActionKind::Axis2,
        vec![Binding::plane("KeyW", "KeyS", "KeyA", "KeyD")],
    ))
    .unwrap();
    let a = map
        .rebind(&RebindDesc {
            action: "move".into(),
            slot: 0,
            part: "up".into(),
            binding: Binding::control("Space"),
            policy: ConflictPolicy::Refuse,
        })
        .unwrap();
    assert_eq!(a.conflicts[0].action, "jump");
    map.rebind(&RebindDesc {
        action: "move".into(),
        slot: 0,
        part: "up".into(),
        binding: Binding::control("KeyZ"),
        policy: ConflictPolicy::Refuse,
    })
    .unwrap();
    let b = &map.bindings("move").unwrap()[0];
    assert_eq!((b.up.as_str(), b.down.as_str()), ("KeyZ", "KeyS"));
    // Reset brings the defaults back.
    map.reset(None).unwrap();
    assert_eq!(map.bindings("crouch").unwrap()[0].control, "KeyC");
    assert_eq!(map.bindings("move").unwrap()[0].up, "KeyW");
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

/// A player's sensitivity is per action and per device class: the
/// mouse's scales the mouse's movement, a pad's the stick, and neither
/// the other's; inversion turns keys over too, but keys are never scaled
/// past a whole press. It comes back from the bindings' text, a reset
/// forgets it, and a world whose player set none saves the bytes it did
/// before sensitivity existed (no `senses` key), so its hash is unmoved.
///
/// Ways it could fail: a single sensitivity for every device (the stick
/// reads 2× too); the scale applied to WASD or arrows (a diagonal past
/// 1); invert on the raw screen y instead of the action's up-positive y;
/// a sensitivity lost from the saved text or kept across a reset; the
/// state block written with an empty `senses`, which moves every hash.
#[test]
fn sensitivity_and_inversion_are_per_action_and_device_class() {
    let look = action(
        "look",
        ActionKind::Axis2,
        vec![
            Binding::control("mouse.xy"),
            Binding::control("pad.rightXY"),
            Binding::plane("ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"),
        ],
    );
    let jump = action("jump", ActionKind::Button, vec![Binding::control("Space")]);
    let mut rig = Rig::new(&[look, jump]);
    let plain = rig.map.lock().unwrap().save_state();
    assert!(
        !String::from_utf8_lossy(&plain).contains("senses"),
        "a world without sensitivities saves no senses key"
    );
    {
        let mut map = rig.map.lock().unwrap();
        let mouse = og_input::Sensitivity {
            scale: 2.0,
            invert_y: true,
            ..Default::default()
        };
        map.set_sensitivity("look", Device::KeyboardMouse, mouse)
            .unwrap();
        // Any pad family is every pad's.
        let stick = og_input::Sensitivity {
            scale: 0.5,
            ..Default::default()
        };
        map.set_sensitivity("look", Device::PlayStation, stick)
            .unwrap();
        assert!(
            map.set_sensitivity("jump", Device::KeyboardMouse, mouse)
                .is_err()
        );
        assert!(
            map.set_sensitivity(
                "look",
                Device::Touch,
                og_input::Sensitivity {
                    scale: 0.0,
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert_eq!(map.sensitivity("look", Device::Xbox).scale, 0.5);
        assert_eq!(map.sensitivity("look", Device::Touch).scale, 1.0);
    }
    // The mouse moved right 4 and up 3 (screen y -3): up is +3 in actions,
    // inverted -3, then twice.
    rig.tick(&[InjectedInput::Pointer {
        action: og_input::PointerAction::Move,
        x: 10.0,
        y: 10.0,
        button: og_input::PointerButton::None,
        id: 0,
        movement_x: 4.0,
        movement_y: -3.0,
    }]);
    let v = rig.state("look").vector;
    assert!(close(v.x, 8.0) && close(v.y, -6.0), "mouse: {v:?}");
    // The right stick pushed right: past the pad's 0.15 dead zone,
    // rescaled to reach 1, then half (not twice), and not inverted.
    rig.tick(&[pad(0, &[], [0.0, 0.0, 0.8, 0.0], [0.0, 0.0], None)]);
    let v = rig.state("look").vector;
    let expected = 0.5 * (0.8 - 0.15) / (1.0 - 0.15);
    assert!(close(v.x, expected) && close(v.y, 0.0), "stick: {v:?}");
    rig.tick(&[pad(0, &[], [0.0; 4], [0.0, 0.0], None)]);
    // The up arrow: keyboard and mouse's inversion, no scale (a whole press).
    rig.tick(&[down("ArrowUp")]);
    let v = rig.state("look").vector;
    assert!(close(v.x, 0.0) && close(v.y, -1.0), "arrow: {v:?}");
    rig.tick(&[up("ArrowUp")]);

    // The text carries them; a reset forgets them.
    let text = rig.map.lock().unwrap().save_bindings();
    let mut other = ActionMap::new();
    let (look, jump) = (
        action(
            "look",
            ActionKind::Axis2,
            vec![Binding::control("mouse.xy")],
        ),
        action("jump", ActionKind::Button, vec![]),
    );
    other.define(look).unwrap();
    other.define(jump).unwrap();
    assert_eq!(other.load_bindings(&text).unwrap(), Vec::<String>::new());
    let s = other.sensitivity("look", Device::KeyboardMouse);
    assert_eq!((s.scale, s.invert_x, s.invert_y), (2.0, false, true));
    assert_eq!(other.sensitivity("look", Device::Switch).scale, 0.5);
    // And the state block, for a snapshot.
    let bytes = rig.map.lock().unwrap().save_state();
    let back = rig.map.lock().unwrap().restore_state(&bytes).unwrap();
    assert_eq!(back.sensitivity("look", Device::KeyboardMouse).scale, 2.0);
    other.reset(None).unwrap();
    assert_eq!(
        other.sensitivity("look", Device::KeyboardMouse),
        og_input::Sensitivity::default()
    );
    // Every sensitivity back at 1: the world saves what it did before.
    let mut map = rig.map.lock().unwrap();
    map.set_sensitivity("look", Device::KeyboardMouse, Default::default())
        .unwrap();
    map.set_sensitivity("look", Device::Gamepad, Default::default())
        .unwrap();
    assert!(!String::from_utf8_lossy(&map.save_state()).contains("senses"));
}
