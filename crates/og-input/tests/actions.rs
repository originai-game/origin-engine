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
    ActionDesc, ActionKind, ActionMap, ActionState, Binding, CaptureDesc, ConflictPolicy, Device,
    GamepadState, InjectedInput, InputLayer, InputSink, KeyAction, Modifiers, PadDeadzone,
    RebindDesc, SharedActions, binding_glyphs, combined_glyph,
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
fn a_keyboard_composite_and_a_stick_drive_one_move_action() {
    let mv = action(
        "move",
        ActionKind::Axis2,
        vec![
            Binding::plane("KeyW", "KeyS", "KeyA", "KeyD"),
            Binding::control("pad.leftXY"),
        ],
    );
    let mut rig = Rig::new(&[mv]);
    // W and D: a diagonal of length 1, not √2; y is up.
    rig.tick(&[down("KeyW"), down("KeyD")]);
    let s = rig.state("move");
    let d = std::f32::consts::FRAC_1_SQRT_2;
    assert!(close(s.vector.x, d) && close(s.vector.y, d), "{s:?}");
    assert!(close(s.value, 1.0));
    assert_eq!(s.device, Device::KeyboardMouse);
    assert!(s.pressed && s.down, "a move past the threshold presses it");
    // Keys up; the stick pushed up and right, as the Gamepad API reports it (y down).
    rig.tick(&[
        up("KeyW"),
        up("KeyD"),
        pad(
            0,
            &[],
            [0.5, -0.8, 0.0, 0.0],
            [0.0, 0.0],
            Some(Device::PlayStation),
        ),
    ]);
    let s = rig.state("move");
    // Radial dead zone 0.15: length √(0.25 + 0.64), rescaled over 0.85, y up.
    let len = (0.5f32 * 0.5 + 0.8 * 0.8).sqrt();
    let k = ((len - 0.15) / 0.85) / len;
    assert!(
        close(s.vector.x, 0.5 * k) && close(s.vector.y, 0.8 * k),
        "{s:?}"
    );
    assert_eq!(s.device, Device::PlayStation);
    // A stick resting inside its dead zone reads nothing.
    rig.tick(&[pad(0, &[], [0.1, 0.08, 0.0, 0.0], [0.0, 0.0], None)]);
    let s = rig.state("move");
    assert_eq!(s.vector, og_core::glam::Vec2::ZERO);
    assert!(s.released && !s.down);
    // A player's wider dead zone on that pad: 0.5 swallows a push of 0.45.
    rig.map
        .lock()
        .unwrap()
        .set_deadzone(
            Some(0),
            PadDeadzone {
                stick: 0.5,
                trigger: 0.05,
            },
        )
        .unwrap();
    rig.tick(&[pad(0, &[], [0.45, 0.0, 0.0, 0.0], [0.0, 0.0], None)]);
    assert_eq!(rig.state("move").vector, og_core::glam::Vec2::ZERO);
}

#[test]
fn button_edges_follow_keys_taps_and_an_analog_threshold() {
    let jump = action(
        "jump",
        ActionKind::Button,
        vec![Binding::control("Space"), Binding::control("pad.a")],
    );
    let fire = action(
        "fire",
        ActionKind::Button,
        vec![Binding::control("pad.rightTrigger")],
    );
    let mut rig = Rig::new(&[jump, fire]);
    rig.tick(&[down("Space")]);
    let s = rig.state("jump");
    assert!(s.down && s.pressed && !s.released);
    rig.tick(&[]);
    let s = rig.state("jump");
    assert!(s.down && !s.pressed, "the edge lasted one tick");
    rig.tick(&[up("Space")]);
    let s = rig.state("jump");
    assert!(!s.down && s.released);
    // A tap shorter than a tick: both edges, not held.
    rig.tick(&[down("Space"), up("Space")]);
    let s = rig.state("jump");
    assert!(s.pressed && s.released && !s.down, "{s:?}");
    // The trigger presses at 0.5 of its travel past its 0.05 dead zone and
    // lets go under three quarters of that: 0.45 keeps it down.
    let t = |v: f32| pad(0, &[], [0.0; 4], [0.0, v], None);
    rig.tick(&[t(0.6)]);
    let s = rig.state("fire");
    assert!(s.pressed && s.down);
    assert!(close(s.value, (0.6 - 0.05) / 0.95));
    rig.tick(&[t(0.45)]);
    assert!(rig.state("fire").down, "no chatter around the threshold");
    rig.tick(&[t(0.3)]);
    assert!(rig.state("fire").released);
}

#[test]
fn contexts_give_a_shared_key_to_the_first_live_one() {
    let interact = ActionDesc {
        context: "gameplay".into(),
        ..action(
            "interact",
            ActionKind::Button,
            vec![Binding::control("KeyE")],
        )
    };
    let exit = ActionDesc {
        context: "vehicle".into(),
        ..action("exit", ActionKind::Button, vec![Binding::control("KeyE")])
    };
    let mut rig = Rig::new(&[interact, exit]);
    rig.map
        .lock()
        .unwrap()
        .set_contexts(vec!["vehicle".into(), "gameplay".into()]);
    rig.tick(&[down("KeyE")]);
    assert!(rig.state("exit").pressed);
    assert!(
        !rig.state("interact").down,
        "the vehicle's E wins over gameplay's"
    );
    rig.tick(&[up("KeyE")]);
    rig.map
        .lock()
        .unwrap()
        .set_contexts(vec!["gameplay".into()]);
    rig.tick(&[down("KeyE")]);
    assert!(rig.state("interact").pressed);
    assert!(!rig.state("exit").down, "a context not listed is off");
    rig.tick(&[up("KeyE")]);
    rig.map.lock().unwrap().set_contexts(Vec::new());
    rig.tick(&[down("KeyE")]);
    assert!(rig.state("interact").pressed && rig.state("exit").pressed);
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
fn a_capture_takes_the_next_input_without_firing_anything() {
    let jump = action("jump", ActionKind::Button, vec![Binding::control("Space")]);
    let crouch = action("crouch", ActionKind::Button, vec![Binding::control("KeyC")]);
    let look = action(
        "look",
        ActionKind::Axis2,
        vec![Binding::control("mouse.xy")],
    );
    let mut rig = Rig::new(&[jump, crouch, look]);
    // A stick held when the capture starts does not count until it returns.
    rig.tick(&[pad(
        0,
        &[],
        [0.0, 0.0, 0.9, 0.0],
        [0.0, 0.0],
        Some(Device::Xbox),
    )]);
    let start = |rig: &mut Rig, action: &str| {
        rig.map
            .lock()
            .unwrap()
            .start_capture(CaptureDesc {
                action: action.into(),
                slot: 0,
                part: String::new(),
                accept: og_input::CaptureAccept::Any,
                cancel: Vec::new(),
                timeout_ms: 0.0,
            })
            .unwrap();
    };
    start(&mut rig, "look");
    rig.tick(&[]);
    assert!(
        rig.map.lock().unwrap().capture().active,
        "the held stick was not taken"
    );
    rig.tick(&[pad(0, &[], [0.0; 4], [0.0, 0.0], None)]);
    rig.tick(&[pad(0, &[], [0.0, 0.0, 0.0, -0.7], [0.0, 0.0], None)]);
    let c = rig.map.lock().unwrap().capture().clone();
    assert!(c.done, "{c:?}");
    assert_eq!(
        c.control, "pad.rightXY",
        "a two-axis action captures the whole stick"
    );
    rig.tick(&[pad(0, &[], [0.0; 4], [0.0, 0.0], None)]);

    // A key for jump: the press is taken, and jump does not fire on it.
    start(&mut rig, "jump");
    rig.tick(&[down("KeyC")]);
    let c = rig.map.lock().unwrap().capture().clone();
    assert!(c.done && c.control == "KeyC");
    assert_eq!(c.conflicts.len(), 1, "crouch holds C: {:?}", c.conflicts);
    assert!(
        !rig.state("crouch").pressed,
        "the captured key moves nothing"
    );
    let applied = rig
        .map
        .lock()
        .unwrap()
        .apply_capture(ConflictPolicy::Swap)
        .unwrap();
    assert!(applied.applied);
    rig.tick(&[]);
    assert!(
        !rig.state("jump").down,
        "the key still held from the capture does not press its new action"
    );
    rig.tick(&[up("KeyC")]);
    rig.tick(&[down("KeyC")]);
    assert!(rig.state("jump").pressed);
    assert_eq!(
        rig.map.lock().unwrap().bindings("crouch").unwrap()[0].control,
        "Space"
    );
    rig.tick(&[up("KeyC")]);

    // Escape cancels and binds nothing.
    start(&mut rig, "jump");
    rig.tick(&[down("Escape")]);
    let c = rig.map.lock().unwrap().capture().clone();
    assert!(c.cancelled && !c.done);
    assert!(
        rig.map
            .lock()
            .unwrap()
            .apply_capture(ConflictPolicy::Allow)
            .is_err()
    );
}

#[test]
fn bindings_come_back_from_their_text() {
    let defs = [
        action("jump", ActionKind::Button, vec![Binding::control("Space")]),
        ActionDesc {
            hold_ms: 300.0,
            ..action(
                "sprint",
                ActionKind::Button,
                vec![Binding::control("ShiftLeft")],
            )
        },
        action(
            "look",
            ActionKind::Axis2,
            vec![
                Binding::control("mouse.xy"),
                Binding::control("pad.rightXY"),
            ],
        ),
    ];
    let mut map = ActionMap::new();
    for d in &defs {
        map.define(d.clone()).unwrap();
    }
    map.rebind(&RebindDesc {
        action: "jump".into(),
        slot: 1,
        part: String::new(),
        binding: Binding::control("pad.a"),
        policy: ConflictPolicy::Refuse,
    })
    .unwrap();
    // Sensitivity and inverted look are a binding's.
    map.rebind(&RebindDesc {
        action: "look".into(),
        slot: 0,
        part: String::new(),
        binding: Binding {
            scale: 0.25,
            invert_y: true,
            ..Binding::control("mouse.xy")
        },
        policy: ConflictPolicy::Refuse,
    })
    .unwrap();
    map.set_toggle("sprint", Some(true)).unwrap();
    map.set_deadzone(
        Some(3),
        PadDeadzone {
            stick: 0.3,
            trigger: 0.1,
        },
    )
    .unwrap();
    let text = map.save_bindings();

    // Loaded before the game defines its actions, then defined: the same.
    let mut fresh = ActionMap::new();
    assert!(fresh.load_bindings(&text).unwrap().is_empty());
    for d in &defs {
        fresh.define(d.clone()).unwrap();
    }
    assert_eq!(fresh.save_bindings(), text);
    assert_eq!(fresh.bindings("jump").unwrap()[1].control, "pad.a");
    let look = &fresh.bindings("look").unwrap()[0];
    assert_eq!((look.scale, look.invert_y), (0.25, true));
    assert_eq!(
        fresh.deadzone(3),
        PadDeadzone {
            stick: 0.3,
            trigger: 0.1
        }
    );
    assert!(
        fresh
            .infos()
            .iter()
            .any(|i| i.desc.name == "sprint" && i.toggle)
    );
    // A binding that no longer fits its action is a warning, not a failure.
    let bad = text.replace("pad.a", "pad.leftXY");
    let warnings = fresh.load_bindings(&bad).unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(
        fresh.bindings("jump").unwrap().len(),
        1,
        "jump is back to its default"
    );
    assert!(fresh.load_bindings("{\"version\": 9}").is_err());
}

#[test]
fn eight_pads_with_their_families_and_glyphs() {
    let jump = action(
        "jump",
        ActionKind::Button,
        vec![Binding::control("Space"), Binding::control("pad.a")],
    );
    let mv = action(
        "move",
        ActionKind::Axis2,
        vec![
            Binding::plane("KeyW", "KeyS", "KeyA", "KeyD"),
            Binding::control("pad.leftXY"),
        ],
    );
    let mut rig = Rig::new(&[jump, mv]);
    rig.tick(&[pad(7, &["a"], [0.0; 4], [0.0, 0.0], Some(Device::Switch))]);
    let pads = *rig.world.resource::<og_input::Gamepad>().unwrap();
    assert!(pads.is_connected(7));
    assert_eq!(pads.device(7), Some(Device::Switch));
    let s = rig.state("jump");
    assert!(s.pressed);
    assert_eq!(s.device, Device::Switch);
    let map = rig.map.lock().unwrap();
    assert_eq!(map.device(), Device::Switch);
    // Switch's south button is B.
    let g = map.glyphs("jump", None).unwrap();
    assert_eq!(
        (g[0].0.icon.as_str(), g[0].0.text.as_str()),
        ("switch.b", "B")
    );
    let keys = map.glyphs("move", Some(Device::KeyboardMouse)).unwrap();
    let one = combined_glyph(&keys, Device::KeyboardMouse).unwrap();
    assert_eq!(
        (one.icon.as_str(), one.text.as_str()),
        ("key.wasd", "W A S D")
    );
    assert!(binding_glyphs(&[Binding::control("Space")], Device::Touch).is_empty());
    drop(map);
    // A disconnect forgets the family.
    rig.tick(&[InjectedInput::Gamepad(GamepadState {
        pad: 7,
        ..Default::default()
    })]);
    assert_eq!(
        rig.world.resource::<og_input::Gamepad>().unwrap().device(7),
        None
    );
}

#[test]
fn a_pad_family_crosses_json_and_the_ring() {
    let e = pad(5, &["x"], [0.0; 4], [0.0, 0.0], Some(Device::PlayStation));
    let json = e.to_json();
    assert_eq!(json["device"], "playstation");
    assert_eq!(InjectedInput::from_json(&json).unwrap(), e);
    let slot = og_input::ring::encode_slot(&e);
    assert_eq!(og_input::ring::decode_slot(&slot).unwrap(), e);
    // keyboardMouse is no pad's family.
    let mut bad = json.clone();
    bad["device"] = "keyboardMouse".into();
    assert!(InjectedInput::from_json(&bad).is_none());
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

#[test]
fn touch_zones_drive_a_stick_and_a_button() {
    use og_input::{PointerAction, PointerButton, Surface, Zone};
    let left = Zone {
        x: 0.0,
        y: 0.0,
        width: 0.5,
        height: 1.0,
    };
    let right = Zone { x: 0.5, ..left };
    let mv = action(
        "move",
        ActionKind::Axis2,
        vec![Binding {
            zone: left,
            radius: 50.0,
            ..Binding::control("touch.xy")
        }],
    );
    let fire = action(
        "fire",
        ActionKind::Button,
        vec![Binding {
            zone: right,
            ..Binding::control("touch")
        }],
    );
    let mut rig = Rig::new(&[mv, fire]);
    rig.map.lock().unwrap().set_surface(Surface {
        width: 1000.0,
        height: 500.0,
        coarse: true,
    });
    let touch = |action, id, x, y| InjectedInput::Pointer {
        action,
        x,
        y,
        button: PointerButton::Left,
        id,
        movement_x: 0.0,
        movement_y: 0.0,
    };
    rig.tick(&[touch(PointerAction::Down, 0, 100.0, 400.0)]);
    assert!(
        !rig.state("fire").down,
        "a touch on the left half is the stick's"
    );
    // Dragged 30 right and 40 up over a radius of 50.
    rig.tick(&[
        touch(PointerAction::Move, 0, 130.0, 360.0),
        touch(PointerAction::Down, 1, 800.0, 400.0),
    ]);
    let s = rig.state("move");
    assert!(close(s.vector.x, 0.6) && close(s.vector.y, 0.8), "{s:?}");
    assert_eq!(s.device, Device::Touch);
    assert!(rig.state("fire").pressed);
    assert_eq!(rig.map.lock().unwrap().device(), Device::Touch);
    rig.tick(&[touch(PointerAction::Up, 0, 130.0, 360.0)]);
    assert!(rig.state("move").released);
    assert!(rig.state("fire").down, "the other touch still holds fire");
}
