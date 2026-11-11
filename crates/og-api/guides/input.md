# Input

How a game reads the player: gameplay actions over keys, the mouse, touch and pads in action maps; mouse-look with the pointer lock; touch and virtual controls; gamepads with dead zones and rumble; the player's rebinding and the host-only session around it; and intents, injected input and replay. Everything the simulation reads is world state taken once per tick, so it replays and snapshots with the world.

The API: `og api show input`. The touch controls a screen draws are in
`screens §4`; online play hands input on as in `networking §3`.

## 1. Actions and action maps

Name what the player does (`jump`, `move`, `look`) and read that, never the
keys. Actions are read once per tick, before the game's systems, from
og.Input, og.Pointers and og.Gamepad, so they replay with them; in lockstep
they are this peer's own and cross the room only through og.net.submit.

```ts
og.input.defineAction(world, { name: "move", kind: "axis2", bindings: [
  { up: "KeyW", left: "KeyA", down: "KeyS", right: "KeyD" },
  { control: "pad.leftXY" },
  { control: "touch.xy", zone: { width: 0.5 } },
] });
og.input.defineAction(world, { name: "jump", kind: "button", bindings: [{ control: "Space" }, { control: "pad.a" }] });

// in a system
const move = og.input.actionVector(world, "move");    // x right, y up
if (og.input.isActionJustPressed(world, "jump")) jump();
```

Controls are named by strings:

| Control | Gives |
|---|---|
| a key code, `KeyW`, `Space` | a button (the physical key, KeyboardEvent.code) |
| `mouse.left`, `mouse.middle`, `mouse.right` | a button |
| `mouse.x`, `mouse.y`, `mouse.xy` | the mouse's movement this tick, CSS px |
| `wheel.x`, `wheel.y` | the wheel, in notches of 100 CSS px |
| `pad.a` to `pad.home` | a pad button, by GamepadButton's names, on any pad |
| `pad.leftX`, `pad.leftY`, `pad.rightX`, `pad.rightY` | one stick axis |
| `pad.leftTrigger`, `pad.rightTrigger` | a trigger, 0 to 1 |
| `pad.leftXY`, `pad.rightXY`, `pad.dpad` | a stick or the d-pad as two axes |
| `touch` | a touch held |
| `touch.xy` | a touch stick: a drag from where the touch started |
| any one-axis control plus `+` or `-` | its half, 0 to 1 (`pad.leftX+`): a direction as a button |

- Every y is up-positive in actions: W, the stick pushed up, the mouse
  moved up, the wheel scrolled up and a touch dragged up all read +1,
  whatever the raw og.Gamepad says (its sticks are y down).
- A composite (A and D as one axis, WASD as two) names its parts instead of
  a control; a binding with neither is an unbound slot a player may fill.
- An analog control presses a button action at its threshold (0.5 by
  default) and lets go under three quarters of it, so a trigger held near
  the line does not chatter.
- Action maps (`gameplay`, `menu`, `vehicle`, or the game's own) let two
  actions share a control. og.input.setActionMaps lists the live ones by
  priority: with `["vehicle", "gameplay"]` E exits the car instead of
  interacting. Empty is every action map, none winning.
- og.input.defineAction may be called again: a redefinition keeps the
  player's bindings while they still fit, and the action's state while its
  kind stays. The map is world state, so snapshots carry it.

## 2. Keyboard, mouse and mouse-look

- Bindings name physical keys, so WASD is the same four keys on every
  layout; only their glyphs change (Z Q S D on AZERTY). The web shell tells
  the layout with og.input.setKeyboardLayout from
  navigator.keyboard.getLayoutMap() in Chrome and Edge; in Firefox and
  Safari it learns each key from the character it types, so a key reads as
  on a US layout until it is pressed once. Native runs are the US layout.
- Prompts read og.input.glyph on the device used last (og.input.device),
  live: a player who picks up a pad sees pad glyphs on the next tick.
- Mouse-look: set og.PointerLock.wanted from a system that writes
  og.PointerLock, and turn the camera by og.Pointers.movement each tick.

```ts
world.resource(og.PointerLock).value!.wanted = playing;
const m = world.resource(og.Pointers).value!.movement;  // CSS px, y down
yaw -= m.x * 0.0025;
```

  A `look` action bound to `mouse.xy` gives the same movement with the
  player's sensitivity and inversion applied, y up.

- Browsers grant the lock only to a gesture: the shell asks for it on the
  next press on the canvas (so the click that starts play takes it) and
  releases it when `wanted` goes false. The player's Escape releases it
  too: `locked` goes false while `wanted` stays true, so show a "click to
  look" prompt then.
- While locked, og.Input.pointer and og.Pointers' positions stand still
  (the UI's hover stays where it was), the mouse's buttons still press, and
  only the movement moves. `locked` follows the page's pointerlockchange.
- Native runs (`og run`, `og dev`) are headless: `locked` stays false and
  og.Pointers.movement stays zero unless a script injects
  `{ kind: "pointerLock", locked: true }` and pointer events with
  movementX/Y.
- Raw input (og.input.setMouse, on by default) asks the browser for the
  movement without the OS's pointer acceleration (requestPointerLock with
  unadjustedMovement). Chrome and Edge grant it on Windows, macOS and
  ChromeOS; Firefox, Safari and Linux refuse it and the lock keeps the
  OS's acceleration, which og.input.mouse reports as `rawUnsupported`
  (show the Raw input switch as unavailable). It takes effect at the next
  lock. It is the player's setting: keep it with og.platform.saveSettings.

## 3. Touch and virtual controls

- og.Pointers holds up to five pointers at once: pointer 0 is og.Input's
  pointer (the mouse, the first touch), 1 to 4 further touches in the order
  they landed, each taking the lowest free id. A touch stick and a touch
  button can be held together.
- A touch control claims the pointers that started on it: read
  where the pointer started (og.Pointers' `start`), not the position now, to decide which control a
  finger belongs to.
- `touch` and `touch.xy` bindings take a zone, fractions of the play
  surface from the top left; a zone of no size is all of it. A
  `touch.xy` stick reaches full deflection after its radius (64 CSS px
  when 0).
- The document controls of `screens §4` (a joystick, a d-pad, a touch
  button) press actions directly; game code reads the same actions as for
  keys and pads.

## 4. Gamepads

- Up to eight pads, in the W3C standard layout. On the web the shell polls
  them each display frame and sends their state through the input ring, so
  a pad is world state and replays. A pad the browser does not map
  (Firefox, Linux, many third-party pads) is mapped through the shell's
  table; the shell also maps pads to keys for games that read only keys.
- og.Gamepad's sticks and triggers are raw, y down. Dead zones belong to
  the actions:

| Dead zone | Default | Applies |
|---|---|---|
| a pad's stick (og.input.setDeadzones) | 0.15, radial over both axes | every action reading that pad |
| a pad's trigger | 0.05 | every action reading that pad |
| a binding's `deadzone` | 0: the pad's | over the pad's when above 0 |
| an action's `deadzone` | 0 | on the combined value (radial for two axes), after each binding's own |

  Inside a dead zone a value reads 0; the rest is rescaled to reach 1.
  Every dead zone stops at 0.95. The pads' are the player's (a settings
  slider), kept by og.input.saveBindings.
- og.input.padDevice gives a pad's family as the host told it from the
  pad's id (Xbox, PlayStation, Switch or generic); glyphs follow it. A pad
  whose family the host could not tell is drawn as an Xbox pad. A Switch
  pad's south button is B.
- og.input.rumble is output for the host, never world state: a replay does
  not repeat it, and a native run records it in the frame answer's
  `rumble`. Short, light pulses read best: 80 to 200 ms, the high motor
  for taps, the low motor for impacts; `ms` 0 stops the pad.

## 5. Rebinding, settings and the session

A controls screen lists og.input.actions and changes them for the player.

```ts
og.input.startCapture(world, { action: "jump", slot: 0 });   // "press a key"
// each tick:
const c = og.input.capture(world);
if (c.done) {
  const r = og.input.applyCapture(world);                  // refuse: shows conflicts
  if (r.ok && !r.value.applied) askPlayer(r.value.conflicts);
}
// on leaving the screen:
og.platform.saveSettings(world, { ...settings, controls: og.input.saveBindings(world) });
```

- From the next tick a capture blinds every action until it ends, and what
  was pressed for it moves no action until it is let go. `Escape` and
  `pad.back` cancel it unless the capture names its own cancel controls.
- A rebinding sharing a control with another action of its action map is
  refused by default and answers the conflicts; `swap`, `replace` and
  `allow` resolve them. Locked actions (pause, a debug key) cannot be
  rebound.
- The player's settings are bindings, toggles (a hold's alternative, XAG
  107), dead zones and per-device sensitivity and inversion (a look
  action's mouse and its stick each have their own). og.input.saveBindings
  writes the ones that differ from the defaults as text; keep it with
  og.platform.saveSettings or in a save, and give it back to
  og.input.loadBindings at start, before or after the actions are defined.
  A world whose player set no sensitivity saves and hashes as before.
- Session state is the host's, not the simulation's: og.PointerLock's
  `locked`, the mouse settings (og.input.setMouse), the keyboard layout
  and rumble. A restore keeps it, and lockstep peers never compare it
  (`networking §6`).

## 6. Intents, injected input and replay

- UI controls never write gameplay state: they send a reflected event with
  og.input.intent, delivered at the next tick and recorded with its name,
  packed payload and layout hash. A pause layer sends og.PauseIntent the
  same way. Inside a system, declare the event among its writes.
- A replay tool drains og.input.takeIntents regularly and later gives each
  back with og.input.replayIntent between ticks, when `world.time.tick`
  equals its tick. A recorded intent holds no presentation state, so it
  replays independently of presentation checkpoints.
- The shell collects pointer, keyboard, wheel and text input on the main
  thread; the probe's `input()` and playtest scripts inject through the
  same path, so injected input is indistinguishable from the player's.
- Key and pointer events carry `at`, when they happened within the tick
  (world time `time.elapsed + at * time.dt`): the shell stamps it from the
  browser event's timeStamp, the command channel from the step's time.
  Use it for timing-critical presses (rhythm, fighting inputs); it is part
  of the event, so a replay carries it.
- og.Input's edges (pressed, released) last exactly one tick; a press and a
  release between two ticks set both. An injected `press` or `click` is
  down, then up on the next tick.
- Text arrives as TextInput events, apart from the keys: a text longer than
  255 bytes comes as several in order. A text field types from them once a
  world has had one.
