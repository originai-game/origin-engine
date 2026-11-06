//! Command action execution.

use super::*;

impl Probe {
    pub(super) fn action(&mut self, world: &mut World, step: &CommandStep) -> Result<(), String> {
        let tick = world.time().tick;
        let args = &step.args;
        let arg = |i: usize, usage: &str| -> Result<&str, String> {
            args.get(i)
                .map(String::as_str)
                .filter(|a| !a.is_empty())
                .ok_or_else(|| format!("missing argument; write `{usage}`"))
        };
        let joined = args.join(":");
        match step.action.as_str() {
            #[cfg(not(feature = "publish"))]
            "screenshot" => {
                let mut path = None;
                let mut edge = None;
                for a in args {
                    if a == "full" {
                        edge = None;
                    } else if let Ok(n) = a.parse::<u32>() {
                        edge = Some(n.clamp(1, MAX_CAPTURE_EDGE));
                    } else if path.is_none() && !a.is_empty() {
                        path = output_path(Some(a))?;
                    } else {
                        return Err(format!(
                            "unexpected argument '{a}'; write `screenshot[:path][:full|:N]`"
                        ));
                    }
                }
                // No renderer is a host configuration, not a fault: the
                // step notes the frame it did not take and the run goes on
                // exactly as it would with one (rendering never feeds back).
                let Some(capture) = self.capture.as_mut() else {
                    let file = path.unwrap_or_else(|| format!("screenshot-{tick:06}.png"));
                    self.note(
                        tick,
                        false,
                        format!("{}s {FRAME_UNCHECKED} ({file})", step.at),
                    );
                    return Ok(());
                };
                let (width, height) = match edge {
                    None => (0, 0),
                    Some(n) => {
                        let (tw, th) = capture.target_size();
                        let (tw, th) = (f64::from(tw.max(1)), f64::from(th.max(1)));
                        if tw >= th {
                            (n, ((f64::from(n) * th / tw).round() as u32).max(1))
                        } else {
                            (((f64::from(n) * tw / th).round() as u32).max(1), n)
                        }
                    }
                };
                let frame = capture.capture(world, width, height);
                let Some(frame) = frame else {
                    if !capture.deferred() {
                        return Err(match capture.capture_error() {
                            Some(why) => format!("the renderer produced no frame: {why}"),
                            None => "the renderer produced no frame".into(),
                        });
                    }
                    // Its frame lands on a later call: written then.
                    self.shot = Some(Shot {
                        tick,
                        at: step.at,
                        path,
                        width,
                        height,
                        polls: 0,
                    });
                    return Ok(());
                };
                self.emit(
                    step.at,
                    Output {
                        tick,
                        action: step.action.clone(),
                        path,
                        payload: Payload::Png {
                            width: frame.width,
                            height: frame.height,
                            bytes: frame.png,
                        },
                    },
                );
                Ok(())
            }
            #[cfg(not(feature = "publish"))]
            "listen" => {
                let mut path = None;
                let mut seconds = 1.0f32;
                for a in args {
                    if let Ok(s) = a.parse::<f32>() {
                        seconds = s.clamp(0.0, 2.0);
                    } else if path.is_none() && !a.is_empty() {
                        path = output_path(Some(a))?;
                    }
                }
                let listen = self
                    .listen
                    .as_ref()
                    .ok_or("no audio in this host: nothing to listen to")?;
                let heard = listen(seconds).ok_or("the mixer has rendered nothing yet")?;
                self.emit(
                    step.at,
                    Output {
                        tick,
                        action: step.action.clone(),
                        path,
                        payload: Payload::Wav(heard.wav),
                    },
                );
                Ok(())
            }
            "touch_move_to" | "move_to_text" => {
                let target = if step.action == "touch_move_to" {
                    arg(0, "touch_move_to:<ui id>")?.to_owned()
                } else {
                    if joined.is_empty() {
                        return Err("missing argument; write `move_to_text:[=]text[@N]`".into());
                    }
                    joined.clone()
                };
                let elements = match &self.ui {
                    Some(ui) => ui.elements(world),
                    None => Vec::new(),
                };
                let found = if step.action == "touch_move_to" {
                    find_by_id(&elements, &target)
                } else {
                    find_by_text(&elements, &target)
                };
                let Some(element) = found else {
                    self.target_missed = true;
                    return Err(format!(
                        "target not found: '{target}' ({} UI elements; `dump_ui` lists them); the next touch_start/touch_end are skipped",
                        elements.len()
                    ));
                };
                self.target_missed = false;
                let (x, y) = element.center();
                self.pointer = (x, y);
                self.pointer_event(world, PointerAction::Move, PointerButton::None)
            }
            "touch_move" => {
                let xy = numbers(arg(0, "touch_move:x,y")?)?;
                let [x, y] = xy[..] else {
                    return Err("write `touch_move:x,y` with two numbers".into());
                };
                self.target_missed = false;
                self.pointer = (x, y);
                self.pointer_event(world, PointerAction::Move, PointerButton::None)
            }
            "touch_start" | "touch_end" => {
                if self.target_missed {
                    if step.action == "touch_end" {
                        self.target_missed = false;
                    }
                    return Err("skipped: the last touch target was not found".into());
                }
                let action = if step.action == "touch_start" {
                    PointerAction::Down
                } else {
                    PointerAction::Up
                };
                self.pointer_event(world, action, PointerButton::Left)
            }
            "mouse_move" => {
                let d = numbers(arg(0, "mouse_move:dx,dy")?)?;
                let [dx, dy] = d[..] else {
                    return Err("write `mouse_move:dx,dy` with two numbers".into());
                };
                let (x, y) = self.pointer;
                let event = InjectedInput::Pointer {
                    action: PointerAction::Move,
                    x,
                    y,
                    button: PointerButton::None,
                    id: 0,
                    movement_x: dx,
                    movement_y: dy,
                };
                if self.deliver(world, &event) {
                    Ok(())
                } else {
                    Err("the input layer refused the mouse move".into())
                }
            }
            "pointer_lock" => {
                let locked = match arg(0, "pointer_lock:<on|off>")? {
                    "on" => true,
                    "off" => false,
                    other => {
                        return Err(format!(
                            "'{other}' is not on or off; write `pointer_lock:<on|off>`"
                        ));
                    }
                };
                if self.deliver(world, &InjectedInput::PointerLock { locked }) {
                    Ok(())
                } else {
                    Err("the input layer refused the pointer lock report".into())
                }
            }
            "scroll" => {
                let d = numbers(arg(0, "scroll:dx,dy")?)?;
                let [dx, dy] = d[..] else {
                    return Err("write `scroll:dx,dy` with two numbers".into());
                };
                let (x, y) = self.pointer;
                let event = InjectedInput::Wheel {
                    x,
                    y,
                    delta_x: dx,
                    delta_y: dy,
                };
                if self.deliver(world, &event) {
                    Ok(())
                } else {
                    Err("the input layer refused the wheel event".into())
                }
            }
            "touch" => {
                let usage = "touch:<id>:<down|move|up>:<x,y | ui id>";
                let id: u32 = arg(0, usage)?
                    .parse()
                    .ok()
                    .filter(|id| (1..og_input::MAX_POINTERS as u32).contains(id))
                    .ok_or("the touch id is 1 to 4 (0 is the pointer touch_start drives)")?;
                let action = match arg(1, usage)? {
                    "down" => PointerAction::Down,
                    "move" => PointerAction::Move,
                    "up" => PointerAction::Up,
                    other => {
                        return Err(format!(
                            "'{other}' is not down, move or up; write `{usage}`"
                        ));
                    }
                };
                let target = step.args.get(2..).map(|a| a.join(":")).unwrap_or_default();
                let (x, y) = match numbers(&target).ok().as_deref() {
                    Some(&[x, y]) => (x, y),
                    _ => {
                        let elements = match &self.ui {
                            Some(ui) => ui.elements(world),
                            None => Vec::new(),
                        };
                        let element = find_by_id(&elements, &target).ok_or_else(|| {
                            format!("touch target not found: '{target}' (x,y or a UI id)")
                        })?;
                        element.center()
                    }
                };
                let event = InjectedInput::Pointer {
                    action,
                    x,
                    y,
                    button: if action == PointerAction::Move {
                        PointerButton::None
                    } else {
                        PointerButton::Left
                    },
                    id,
                    movement_x: 0.0,
                    movement_y: 0.0,
                };
                if self.deliver(world, &event) {
                    Ok(())
                } else {
                    Err(format!("the input layer refused touch {id}"))
                }
            }
            "gamepad" => {
                let usage = "gamepad:<a+b+...|->[@pad][:lx,ly[,rx,ry]][:lt,rt]";
                let (first, pad) = match arg(0, usage)?.split_once('@') {
                    Some((buttons, pad)) => (
                        buttons,
                        pad.parse::<u32>()
                            .ok()
                            .filter(|p| (*p as usize) < og_input::MAX_GAMEPADS)
                            .ok_or_else(|| {
                                format!("'@{pad}' is not a pad (0 to 7); write `{usage}`")
                            })?,
                    ),
                    None => (arg(0, usage)?, 0),
                };
                let mut state = og_input::GamepadState {
                    connected: first != "off",
                    pad,
                    ..Default::default()
                };
                if state.connected && first != "-" {
                    for name in first.split('+') {
                        let b = og_input::gamepad_button(name).ok_or_else(|| {
                            format!(
                                "'{name}' is not a standard gamepad button ({})",
                                og_input::GAMEPAD_BUTTONS.join(", ")
                            )
                        })?;
                        state.buttons |= 1 << b;
                    }
                }
                if let Some(axes) = step.args.get(1) {
                    let v = numbers(axes)?;
                    if v.len() != 2 && v.len() != 4 {
                        return Err(format!(
                            "the axes are lx,ly or lx,ly,rx,ry; write `{usage}`"
                        ));
                    }
                    state.axes[..v.len()].copy_from_slice(&v);
                }
                if let Some(t) = step.args.get(2) {
                    let v = numbers(t)?;
                    let [lt, rt] = v[..] else {
                        return Err(format!("the triggers are lt,rt; write `{usage}`"));
                    };
                    state.triggers = [lt, rt];
                }
                if self.deliver(world, &InjectedInput::Gamepad(state)) {
                    Ok(())
                } else {
                    Err("the input layer refused the gamepad state".into())
                }
            }
            "key_down" | "key_up" => {
                let (modifiers, held, key) = key_chord(arg(0, "key_down:[Mod+]<Key>")?)?;
                let down = step.action == "key_down";
                let action = if down { KeyAction::Down } else { KeyAction::Up };
                // The modifiers go down before the key and up after it.
                let modifier = |code: &str| InjectedInput::Key {
                    key: code.to_owned(),
                    action,
                    modifiers: Modifiers::default(),
                };
                let mut events: Vec<InjectedInput> = Vec::new();
                if down {
                    events.extend(held.iter().map(|c| modifier(c)));
                }
                events.push(InjectedInput::Key {
                    key: key.clone(),
                    action,
                    modifiers,
                });
                if !down {
                    events.extend(held.iter().rev().map(|c| modifier(c)));
                }
                for event in &events {
                    if !self.deliver(world, event) {
                        return Err(format!(
                            "the input layer refused key '{key}'; keys are KeyboardEvent.code names such as KeyW, Space, ArrowLeft"
                        ));
                    }
                }
                Ok(())
            }
            "type" => {
                if joined.is_empty() {
                    return Err("missing argument; write `type:<text>`".into());
                }
                // Per character, one tick apart from the next by two: the
                // key that types it (a press: down one tick, up the next,
                // Shift for capitals and symbols) and the character itself
                // as typed text (og.TextInput), which a text field inserts;
                // a character no key types (é, 中) is typed text alone.
                let mut at = tick;
                for c in joined.chars() {
                    if let Some((key, shift)) = key_for_char(c) {
                        let event = InjectedInput::Key {
                            key,
                            action: KeyAction::Press,
                            modifiers: Modifiers {
                                shift,
                                ..Modifiers::default()
                            },
                        };
                        self.seq += 1;
                        self.queue.insert((at, self.seq), Queued::Inject(event));
                    }
                    self.seq += 1;
                    self.queue.insert(
                        (at, self.seq),
                        Queued::Inject(InjectedInput::Text {
                            text: c.to_string(),
                            composing: false,
                        }),
                    );
                    at += 2;
                }
                Ok(())
            }
            "compose" | "commit" => {
                // An input method: `compose` shows its composition so far
                // (empty ends it), `commit` inserts the committed text.
                let event = InjectedInput::Text {
                    text: joined.clone(),
                    composing: step.action == "compose",
                };
                if step.action == "commit" && joined.is_empty() {
                    return Err("missing argument; write `commit:<text>`".into());
                }
                if self.deliver(world, &event) {
                    Ok(())
                } else {
                    Err("the input layer refused the text (no og.TextInput in this world)".into())
                }
            }
            "teleport" => {
                let id = arg(0, "teleport:<entity>:x,y,z")?;
                let e = self.entity_of(world, id)?;
                let mut xyz = numbers(arg(1, "teleport:<entity>:x,y,z")?)?;
                let tid = engine_type(world, "Transform")?;
                let current = world.get_bytes(e, tid).map_err(|e| e.reason().to_owned())?;
                let schema = world
                    .schema(tid)
                    .map_err(|e| e.reason().to_owned())?
                    .clone();
                let fi = schema
                    .field_index("position")
                    .ok_or("Transform has no position field")?;
                let lanes = schema.fields[fi].ty.lanes();
                if xyz.len() == lanes - 1 {
                    let layout = Layout::of(&schema);
                    let old = value::field_json(schema.fields[fi].ty, &current[layout.range(fi)]);
                    xyz.push(old[lanes - 1].as_f64().unwrap_or(0.0) as f32);
                }
                if xyz.len() != lanes {
                    return Err(format!("Transform.position takes {lanes} numbers"));
                }
                let parameter = format!("{}.position", schema.name);
                self.write_field(world, e, &parameter, &json!(xyz), false)?;
                if let Some(c) = self.capture.as_mut() {
                    c.teleported(world, e);
                }
                Ok(())
            }
            "frame_entity" => {
                let id = arg(0, "frame_entity:<entity>")?;
                let e = self.entity_of(world, id)?;
                let target = self.position_of(world, e)?;
                let cid = engine_type(world, "Camera")?;
                let schema = world
                    .schema(cid)
                    .map_err(|e| e.reason().to_owned())?
                    .clone();
                let mut bytes = world
                    .resource_bytes(cid)
                    .map_err(|_| "no Camera resource in the world".to_owned())?;
                let layout = Layout::of(&schema);
                let (Some(pi), Some(qi)) = (
                    schema.field_index("position"),
                    schema.field_index("quaternion"),
                ) else {
                    return Err("Camera needs position and quaternion fields".into());
                };
                if schema.fields[pi].ty != FieldType::Vec3
                    || schema.fields[qi].ty != FieldType::Quat
                {
                    return Err(
                        "Camera.position must be a vec3 and Camera.quaternion a quat".into(),
                    );
                }
                let eye: Vec3 =
                    og_core::bytemuck::pod_read_unaligned::<[f32; 3]>(&bytes[layout.range(pi)])
                        .into();
                let rotation = look_at(eye, target)
                    .ok_or("the camera is at the entity; nothing to look at")?;
                bytes[layout.range(qi)]
                    .copy_from_slice(og_core::bytemuck::bytes_of(&rotation.to_array()));
                world
                    .insert_resource_bytes(cid, &bytes)
                    .map_err(|e| e.reason().to_owned())
            }
            #[cfg(not(feature = "publish"))]
            "describe" => {
                let value = self.describe(world);
                self.emit(
                    step.at,
                    Output {
                        tick,
                        action: step.action.clone(),
                        path: output_path(args.first())?,
                        payload: Payload::Json(value),
                    },
                );
                Ok(())
            }
            #[cfg(not(feature = "publish"))]
            "expect" => self.expect_args(world, args, Some(step.at)).map(|_| ()),
            #[cfg(not(feature = "publish"))]
            "invariant" => self.add_invariant(args),
            #[cfg(not(feature = "publish"))]
            "apply_prefab" | "revert_prefab" => {
                // `<action>:<entity>[:<Component>[.<field>]][:base]`, over
                // the source files (docs/design.md §2.5, prefabs).
                let apply = step.action == "apply_prefab";
                let usage = if apply {
                    "apply_prefab:<entity>[:<Component>[.<field>]][:base]"
                } else {
                    "revert_prefab:<entity>[:<Component>[.<field>]]"
                };
                let id = arg(0, usage)?;
                let entity = self.entity_of(world, id)?;
                let base = apply && args.iter().skip(1).any(|a| a == "base");
                let target = args.iter().skip(1).find(|a| *a != "base").cloned();
                let edit = crate::PrefabEdit {
                    entity,
                    target,
                    apply,
                    base,
                    write: true,
                };
                let saved = self.prefab_edit(world, &edit)?;
                self.note(
                    tick,
                    false,
                    format!(
                        "{}: {}",
                        step.action,
                        saved
                            .files
                            .iter()
                            .map(|f| format!("{} ({} line(s) changed)", f.source, f.lines.len()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                );
                Ok(())
            }
            #[cfg(not(feature = "publish"))]
            "save_scene" => {
                // `save_scene[:path][:spawned]`: `spawned` is a flag, any
                // other argument the path.
                let spawned = args.iter().any(|a| a == "spawned");
                let path = output_path(args.iter().find(|a| *a != "spawned"))?;
                let saved = self.save_scene(world, path.is_none(), spawned)?;
                let scenes = saved["scenes"].as_array().cloned().unwrap_or_default();
                self.note(
                    tick,
                    false,
                    format!(
                        "save_scene: {}{}{}",
                        scenes
                            .iter()
                            .map(|s| format!(
                                "{} ({} line(s) changed{})",
                                s["source"].as_str().unwrap_or(""),
                                s["lines"].as_array().map_or(0, Vec::len),
                                match s["written"].as_str() {
                                    Some(w) if !w.is_empty() => format!(", written to {w}"),
                                    _ => String::new(),
                                }
                            ))
                            .collect::<Vec<_>>()
                            .join(", "),
                        match saved["spawned"].as_u64() {
                            Some(n) if spawned => format!(
                                "; {n} spawned entit{} under the marked section",
                                if n == 1 { "y" } else { "ies" }
                            ),
                            _ => String::new(),
                        },
                        match saved["skipped"].as_array() {
                            Some(s) if !s.is_empty() => format!(
                                "; not written: {}",
                                s.iter()
                                    .filter_map(Value::as_str)
                                    .collect::<Vec<_>>()
                                    .join("; ")
                            ),
                            _ => String::new(),
                        }
                    ),
                );
                // With a path, the boot scene's text goes there and each
                // other scene's beside it, at its path under assets/.
                if let Some(path) = path {
                    let dir = std::path::Path::new(&path)
                        .parent()
                        .map(|d| d.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    for (k, s) in scenes.iter().enumerate() {
                        let to = if k == 0 {
                            path.clone()
                        } else {
                            let source = s["source"].as_str().unwrap_or("");
                            let rel = source.strip_prefix("assets/").unwrap_or(source);
                            if dir.is_empty() {
                                rel.to_string()
                            } else {
                                format!("{dir}/{rel}")
                            }
                        };
                        self.emit(
                            step.at,
                            Output {
                                tick,
                                action: step.action.clone(),
                                path: Some(to),
                                payload: Payload::Text(s["text"].as_str().unwrap_or("").to_owned()),
                            },
                        );
                    }
                }
                Ok(())
            }
            #[cfg(not(feature = "publish"))]
            "dump_world" | "dump_ui" => {
                // `dump_ui:layers[:path]`: the layer stack alone.
                let layers =
                    step.action == "dump_ui" && args.first().is_some_and(|a| a == "layers");
                let args: &[String] = if layers { &args[1..] } else { args };
                let dump = if step.action == "dump_world" {
                    self.dump_world(world)
                } else if layers {
                    self.dump_layers(world)
                } else {
                    self.dump_ui(world)
                };
                self.emit(
                    step.at,
                    Output {
                        tick,
                        action: step.action.clone(),
                        path: output_path(args.first())?,
                        payload: Payload::Json(dump),
                    },
                );
                Ok(())
            }
            #[cfg(not(feature = "publish"))]
            "dump_entity" => {
                let id = arg(0, "dump_entity:<entity>[:path]")?;
                let e = self.entity_of(world, id)?;
                let dump = self.dump_entity(world, e);
                self.emit(
                    step.at,
                    Output {
                        tick,
                        action: step.action.clone(),
                        path: output_path(args.get(1))?,
                        payload: Payload::Json(dump),
                    },
                );
                Ok(())
            }
            "set" => {
                let (target, text) = joined.split_once('=').ok_or_else(|| {
                    "write `set:<entity>.<component>.<field>=<value> or set:<Resource>.<field>=<value>`"
                        .to_string()
                })?;
                self.tune(world, target, text)
            }
            #[cfg(not(feature = "publish"))]
            "get" => {
                let usage = "get:<Resource>[.<field>] or get:<entity>[.<component>[.<field>]]";
                let target = arg(0, usage)?.trim();
                let value = self.read_target(world, target)?;
                let mut shown = value.to_string();
                if shown.len() > 160 {
                    let mut end = 157;
                    while !shown.is_char_boundary(end) {
                        end -= 1;
                    }
                    shown.truncate(end);
                    shown.push_str("...");
                }
                self.note(tick, false, format!("get {target} = {shown}"));
                // Unnamed gets on one tick: get-<tick>.json, then -2, -3, ...
                let mut path = output_path(args.get(1))?;
                if path.is_none() {
                    self.gets = if self.gets.0 == tick {
                        (tick, self.gets.1 + 1)
                    } else {
                        (tick, 1)
                    };
                    if self.gets.1 > 1 {
                        path = Some(format!("get-{tick:06}-{}.json", self.gets.1));
                    }
                }
                self.emit(
                    step.at,
                    Output {
                        tick,
                        action: "get".into(),
                        path,
                        payload: Payload::Json(json!({
                            "kind": "get",
                            "target": target,
                            "tick": tick,
                            "value": value,
                        })),
                    },
                );
                Ok(())
            }
            "seed" => {
                let seed: u32 = arg(0, "seed:<n>")?
                    .trim()
                    .parse()
                    .map_err(|_| "the seed is a whole number in [0, 4294967295]".to_owned())?;
                *world
                    .resource_mut::<Rng>()
                    .map_err(|e| e.reason().to_owned())? = Rng::new(seed);
                Ok(())
            }
            "pause" => {
                self.paused = true;
                self.script_paused = true;
                Ok(())
            }
            "resume" => {
                self.paused = false;
                self.script_paused = false;
                Ok(())
            }
            "advance" => {
                let ticks: u64 = arg(0, "advance:<ticks>")?
                    .trim()
                    .parse()
                    .ok()
                    .filter(|t| *t <= PLAYTEST_MAX_TICKS)
                    .ok_or_else(|| {
                        format!("ticks is a whole number in [0, {PLAYTEST_MAX_TICKS}]")
                    })?;
                // The script's steps after this one that are already due
                // wait for the steps: `0:advance:300;0:dump_world;0:quit`
                // dumps tick 300 and quits there, instead of the nested pump
                // running them (quit included) before the first step.
                let now = world.time().tick;
                let later = self.queue.split_off(&(now + 1, 0));
                let due = std::mem::replace(&mut self.queue, later);
                let advanced = self.advance(world, ticks);
                let tick = world.time().tick;
                for ((_, seq), step) in due {
                    self.queue.insert((tick, seq), step);
                }
                advanced.map(|_| ()).map_err(|e| e.reason().to_owned())
            }
            "quit" => {
                self.quit = true;
                Ok(())
            }
            #[cfg(not(feature = "publish"))]
            "help" => {
                let mut text = String::new();
                for info in self.help() {
                    let origin = if info.core { "" } else { " (registered)" };
                    text.push_str(&format!("{}{origin}\n    {}\n", info.usage, info.doc));
                }
                self.note(tick, false, format!("help: {} actions", self.help().len()));
                self.emit(
                    step.at,
                    Output {
                        tick,
                        action: "help".into(),
                        path: output_path(args.first())?,
                        payload: Payload::Text(text),
                    },
                );
                Ok(())
            }
            // A publish build runs the actions that change the world (input,
            // set, teleport, the clock, registered commands: what og.ai's
            // commands and a timeline's event keys reach); the ones that only
            // observe it for a tool are not built in.
            #[cfg(feature = "publish")]
            "screenshot" | "listen" | "describe" | "expect" | "save_scene" | "apply_prefab"
            | "revert_prefab" | "dump_world" | "dump_ui" | "dump_entity" | "get" | "help" => Err(
                format!("`{}` is not built into a publish build", step.action),
            ),
            name => {
                let Some(registered) = self.commands.get(name) else {
                    return Err(format!(
                        "unknown action '{name}', skipped; `help` lists the actions"
                    ));
                };
                match registered.handler.clone() {
                    Handler::Act(handler) => handler(world, args),
                    Handler::Dump(handler, path_arg) => {
                        let (head, path) = if args.len() > path_arg {
                            (&args[..path_arg], output_path(args.get(path_arg))?)
                        } else {
                            (&args[..], None)
                        };
                        let value = handler(world, head)?;
                        self.emit(
                            step.at,
                            Output {
                                tick,
                                action: name.to_owned(),
                                path,
                                payload: Payload::Json(value),
                            },
                        );
                        Ok(())
                    }
                }
            }
        }
    }
}
