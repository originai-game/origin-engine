//! og.input's gameplay actions (IDL `input`, og-input's `actions.rs`): the
//! binding calls over the world's action map, which the engine samples
//! before the game's systems (`modules::mount_front`).

use super::*;
use og_input::actions::lock;
use og_input::{
    ActionDesc, ActionMap, CaptureDesc, ConflictPolicy, Control, Device, Glyph, PadDeadzone,
    RebindDesc, Rumble,
};

/// Runs `f` on the world's action map; the reason when the world has none.
fn on_map<T>(world: u32, f: impl FnOnce(&mut ActionMap) -> T) -> Result<T, String> {
    let eng = engine(world)?;
    let shared = eng
        .state
        .borrow()
        .actions
        .clone()
        .ok_or("the world has no action map")?;
    let mut map = lock(&shared);
    Ok(f(&mut map))
}

fn desc_json(desc: Option<String>) -> Result<Value, String> {
    let v = parse(desc.as_deref())?;
    Ok(if v.is_null() { json!({}) } else { v })
}

fn device_of(device: Option<String>) -> Result<Option<Device>, String> {
    match device.as_deref() {
        None | Some("") => Ok(None),
        Some(d) => Device::from_name(d).map(Some).ok_or_else(|| {
            format!(
                "'{d}' is no device: write one of {}",
                Device::NAMES.join(", ")
            )
        }),
    }
}

fn input_device(d: Device) -> crate::host::wire::InputDevice {
    match d {
        Device::KeyboardMouse => wire::InputDevice::KeyboardMouse,
        Device::Xbox => wire::InputDevice::Xbox,
        Device::PlayStation => wire::InputDevice::Playstation,
        Device::Switch => wire::InputDevice::Switch,
        Device::Gamepad => wire::InputDevice::Gamepad,
        Device::Touch => wire::InputDevice::Touch,
    }
}

fn input_binding(b: &og_input::Binding) -> crate::host::wire::InputBinding {
    wire::InputBinding {
        control: b.control.clone(),
        negative: b.negative.clone(),
        positive: b.positive.clone(),
        up: b.up.clone(),
        down: b.down.clone(),
        left: b.left.clone(),
        right: b.right.clone(),
        scale: b.scale,
        invert_y: b.invert_y,
        deadzone: b.deadzone,
        zone: wire::TouchZone {
            x: b.zone.x,
            y: b.zone.y,
            width: b.zone.width,
            height: b.zone.height,
        },
        radius: b.radius,
    }
}

fn binding_conflicts(list: &[og_input::Conflict]) -> Vec<crate::host::wire::BindingConflict> {
    list.iter()
        .map(|c| wire::BindingConflict {
            action: c.action.clone(),
            slot: c.slot,
            control: c.control.clone(),
            action_map: c.context.clone(),
        })
        .collect()
}

fn rebind_answer(a: og_input::RebindAnswer) -> crate::host::wire::RebindAnswer {
    wire::RebindAnswer {
        applied: a.applied,
        conflicts: binding_conflicts(&a.conflicts),
    }
}

fn capture_state(c: &og_input::CaptureState) -> crate::host::wire::CaptureState {
    wire::CaptureState {
        active: c.active,
        done: c.done,
        cancelled: c.cancelled,
        action: c.action.clone(),
        slot: c.slot,
        part: c.part.clone(),
        control: c.control.clone(),
        binding: input_binding(&c.binding),
        conflicts: binding_conflicts(&c.conflicts),
    }
}

fn input_glyph(g: &Glyph, device: Device, control: &str) -> crate::host::wire::InputGlyph {
    wire::InputGlyph {
        icon: g.icon.clone(),
        text: g.text.clone(),
        device: input_device(device),
        control: control.to_string(),
    }
}

fn state_of(world: u32, name: &str) -> Option<og_input::ActionState> {
    on_map(world, |m| m.state(name)).ok().flatten()
}

pub fn og_input_define_action(world: u32, desc: Option<String>) -> Result<(), String> {
    desc_json(desc)
        .and_then(|d| ActionDesc::from_json(&d))
        .and_then(|d| on_map(world, |m| m.define(d))?)
}

pub fn og_input_actions(world: u32) -> Vec<crate::host::wire::ActionInfo> {
    on_map(world, |m| {
        m.infos()
            .iter()
            .map(|i| wire::ActionInfo {
                name: i.desc.name.clone(),
                kind: match i.desc.kind {
                    og_input::ActionKind::Button => wire::ActionKind::Button,
                    og_input::ActionKind::Axis => wire::ActionKind::Axis,
                    og_input::ActionKind::Axis2 => wire::ActionKind::Axis2,
                },
                action_map: i.desc.context.clone(),
                label: if i.desc.label.is_empty() {
                    i.desc.name.clone()
                } else {
                    i.desc.label.clone()
                },
                bindings: i.bindings.iter().map(input_binding).collect(),
                defaults: i.desc.bindings.iter().map(input_binding).collect(),
                locked: i.desc.locked,
                toggle: i.toggle,
                rebound: i.rebound,
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default()
}

pub fn og_input_action(world: u32, name: String) -> Option<crate::host::wire::ActionState> {
    state_of(world, &name).map(|s| wire::ActionState {
        down: s.down,
        pressed: s.pressed,
        released: s.released,
        held: s.held,
        value: s.value,
        vector: s.vector,
        device: input_device(s.device),
    })
}

pub fn og_input_is_action_pressed(world: u32, name: String) -> bool {
    state_of(world, &name).is_some_and(|s| s.down)
}

pub fn og_input_is_action_just_pressed(world: u32, name: String) -> bool {
    state_of(world, &name).is_some_and(|s| s.pressed)
}

pub fn og_input_is_action_just_released(world: u32, name: String) -> bool {
    state_of(world, &name).is_some_and(|s| s.released)
}

pub fn og_input_action_held(world: u32, name: String) -> bool {
    state_of(world, &name).is_some_and(|s| s.held)
}

pub fn og_input_action_value(world: u32, name: String) -> f64 {
    state_of(world, &name).map_or(0.0, |s| f64::from(s.value))
}

pub fn og_input_action_vector(world: u32, name: String) -> glam::Vec2 {
    state_of(world, &name).map(|s| s.vector).unwrap_or_default()
}

/// A two-axis action as a horizontal world vector seen from `camera`.
pub fn og_input_move_relative(world: u32, name: String, camera: String) -> glam::Vec3 {
    let input = og_input_action_vector(world, name);
    og_core::facing::move_relative(input, super::rotation_arg(&camera, "camera"))
}

pub fn og_input_set_action_maps(world: u32, contexts: String) {
    let list: Vec<String> = wire::arg(Some(&contexts), "actionMaps").unwrap_or_default();
    let _ = on_map(world, |m| m.set_contexts(list));
}

pub fn og_input_action_maps(world: u32) -> Vec<String> {
    on_map(world, |m| m.contexts().to_vec()).unwrap_or_default()
}

pub fn og_input_rebind(
    world: u32,
    desc: Option<String>,
) -> Result<crate::host::wire::RebindAnswer, String> {
    let r = desc_json(desc)
        .and_then(|d| RebindDesc::from_json(&d))
        .and_then(|d| on_map(world, |m| m.rebind(&d))?);
    r.map(rebind_answer)
}

pub fn og_input_conflicts(
    world: u32,
    desc: Option<String>,
) -> Result<Vec<crate::host::wire::BindingConflict>, String> {
    let r = desc_json(desc)
        .and_then(|d| RebindDesc::from_json(&d))
        .and_then(|d| {
            on_map(world, |m| {
                // What the slot would become, then what it would share.
                let mut probe = m.clone();
                probe.rebind(&RebindDesc {
                    policy: ConflictPolicy::Refuse,
                    ..d
                })
            })?
        });
    r.map(|a| binding_conflicts(&a.conflicts))
}

pub fn og_input_reset_bindings(world: u32, action: Option<String>) -> Result<(), String> {
    on_map(world, |m| {
        m.reset(action.as_deref().filter(|a| !a.is_empty()))
    })
    .and_then(|r| r)
}

pub fn og_input_set_toggle(world: u32, action: String, on: bool) -> Result<(), String> {
    on_map(world, |m| m.set_toggle(&action, Some(on))).and_then(|r| r)
}

pub fn og_input_save_bindings(world: u32) -> String {
    on_map(world, |m| m.save_bindings()).unwrap_or_default()
}

pub fn og_input_load_bindings(world: u32, text: String) -> Result<serde_json::Value, String> {
    on_map(world, |m| m.load_bindings(&text))
        .and_then(|r| r)
        .map(|w| json!(w))
}

pub fn og_input_start_capture(world: u32, desc: Option<String>) -> Result<(), String> {
    desc_json(desc)
        .and_then(|d| CaptureDesc::from_json(&d))
        .and_then(|d| on_map(world, |m| m.start_capture(d))?)
}

pub fn og_input_capture(world: u32) -> crate::host::wire::CaptureState {
    on_map(world, |m| capture_state(m.capture()))
        .unwrap_or_else(|_| capture_state(&og_input::CaptureState::default()))
}

pub fn og_input_cancel_capture(world: u32) {
    let _ = on_map(world, |m| m.cancel_capture());
}

pub fn og_input_apply_capture(
    world: u32,
    policy: Option<String>,
) -> Result<crate::host::wire::RebindAnswer, String> {
    let policy = match policy.as_deref() {
        None | Some("") => ConflictPolicy::Refuse,
        Some(p) => match ConflictPolicy::from_name(p) {
            Some(p) => p,
            None => return Err(format!("policy is refuse, swap, replace or allow, not {p}")),
        },
    };
    on_map(world, |m| m.apply_capture(policy))
        .and_then(|r| r)
        .map(rebind_answer)
}

/// A binding's glyphs, each with its control's name, and the device they
/// are drawn for.
type Glyphs = (Vec<(Glyph, String)>, Device);

/// The glyphs of `action`'s first binding on `device` (the device used
/// last when `None`), with the device they are drawn for.
fn glyphs_of(world: u32, action: &str, device: Option<String>) -> Result<Option<Glyphs>, String> {
    let device = device_of(device)?;
    on_map(world, |m| {
        let d = device.unwrap_or(m.device());
        m.glyphs(action, Some(d)).map(|g| (g, d))
    })
}

pub fn og_input_glyph(
    world: u32,
    action: String,
    device: Option<String>,
) -> Option<wire::InputGlyph> {
    let (parts, d) = glyphs_of(world, &action, device).ok()??;
    let g = og_input::combined_glyph(&parts, d)?;
    let names: Vec<&str> = parts.iter().map(|(_, n)| n.as_str()).collect();
    Some(input_glyph(&g, d, &names.join(" ")))
}

pub fn og_input_glyphs(
    world: u32,
    action: String,
    device: Option<String>,
) -> Vec<wire::InputGlyph> {
    match glyphs_of(world, &action, device) {
        Ok(Some((parts, d))) => parts
            .iter()
            .map(|(g, name)| input_glyph(g, d, name))
            .collect(),
        _ => Vec::new(),
    }
}

pub fn og_input_control_glyph(control: String, device: Option<String>) -> Option<wire::InputGlyph> {
    let device = device_of(device).ok()?;
    let c = Control::parse(&control)?;
    let d = device.unwrap_or(match c.class() {
        Device::Gamepad => Device::Xbox,
        other => other,
    });
    Some(input_glyph(&c.glyph(d), d, &control))
}

pub fn og_input_device(world: u32) -> String {
    on_map(world, |m| m.device().name().to_string())
        .unwrap_or_else(|_| Device::KeyboardMouse.name().to_string())
}

pub fn og_input_pad_device(world: u32, pad: u32) -> Option<String> {
    let device = input_res(
        world,
        |g: &og_input::Gamepad| g.is_connected(pad).then(|| g.device(pad)).flatten(),
        None,
    );
    device.map(|d| d.name().to_string())
}

pub fn og_input_set_deadzones(world: u32, desc: Option<String>) -> Result<(), String> {
    wire::arg::<wire::DeadzoneDesc>(desc.as_deref(), "desc").and_then(|d| {
        let dz = PadDeadzone {
            stick: d.stick,
            trigger: d.trigger,
        };
        let pad = usize::try_from(d.pad).ok();
        on_map(world, |m| m.set_deadzone(pad, dz))?
    })
}

pub fn og_input_set_sensitivity(world: u32, desc: Option<String>) -> Result<(), String> {
    wire::arg::<wire::SensitivityDesc>(desc.as_deref(), "desc").and_then(|d| {
        let device =
            device_of(Some(d.device.as_str().to_string()))?.unwrap_or(Device::KeyboardMouse);
        let sense = og_input::Sensitivity {
            scale: d.scale,
            invert_x: d.invert_x,
            invert_y: d.invert_y,
        };
        on_map(world, |m| m.set_sensitivity(&d.action, device, sense))?
    })
}

pub fn og_input_sensitivity(
    world: u32,
    action: String,
    device: Option<String>,
) -> crate::host::wire::SensitivityDesc {
    let device = device_of(device).ok().flatten();

    on_map(world, |m| {
        let d = device.unwrap_or(m.device());
        let s = m.sensitivity(&action, d);
        // The class's own name: any pad family is the pads'.
        let class = if d.is_pad() { d } else { d.class() };
        wire::SensitivityDesc {
            action: action.clone(),
            device: input_device(class),
            scale: s.scale,
            invert_x: s.invert_x,
            invert_y: s.invert_y,
        }
    })
    .unwrap_or_else(|_| wire::SensitivityDesc {
        action: action.clone(),
        device: wire::InputDevice::KeyboardMouse,
        scale: 1.0,
        invert_x: false,
        invert_y: false,
    })
}

pub fn og_input_set_keyboard_layout(labels: String) {
    let list: Vec<wire::KeyLabel> = wire::arg(Some(&labels), "labels").unwrap_or_default();
    og_input::set_key_labels(list.into_iter().map(|e| (e.code, e.label)));
}

pub fn og_input_keyboard_layout() -> Vec<crate::host::wire::KeyLabel> {
    og_input::key_labels()
        .into_iter()
        .map(|(code, label)| wire::KeyLabel {
            code: code.to_string(),
            label: label.to_string(),
        })
        .collect::<Vec<_>>()
}

pub fn og_input_set_mouse(world: u32, desc: Option<String>) -> Result<(), String> {
    (|| -> Result<(), String> {
        let eng = engine(world)?;
        // A patch: only the fields the desc names change.
        let v = parse(desc.as_deref())?;
        let d: wire::MouseDesc = wire::value(&v, "desc")?;
        let given = |k: &str| v.get(k).is_some_and(|x| !x.is_null());
        let mut rs = eng
            .render
            .try_borrow_mut()
            .map_err(|_| "the renderer is busy drawing".to_string())?;
        if given("raw") {
            rs.display.mouse.raw = d.raw;
        }
        Ok(())
    })()
}

pub fn og_input_mouse(world: u32) -> crate::host::wire::MouseInfo {
    let m = engine(world)
        .ok()
        .and_then(|eng| eng.render.try_borrow().ok().map(|rs| rs.display.mouse))
        .unwrap_or_default();
    wire::MouseInfo {
        raw: m.raw,
        raw_active: m.raw_active,
        raw_unsupported: m.raw_unsupported,
    }
}

pub fn og_input_rumble(world: u32, desc: Option<String>) -> Result<(), String> {
    wire::arg::<wire::RumbleDesc>(desc.as_deref(), "desc").and_then(|d| {
        let r = Rumble {
            pad: d.pad,
            low: d.low,
            high: d.high,
            ms: d.ms,
        };
        on_map(world, |m| m.rumble(r))?
    })
}

/// The rumbles the game asked for since the last frame answer, as the
/// answer's `rumble`: `[pad, low, high, ms]` each (pad -1: every pad).
pub(crate) fn rumble_played(eng: &Engine) -> Vec<[f64; 4]> {
    let Some(shared) = eng.state.borrow().actions.clone() else {
        return Vec::new();
    };
    lock(&shared)
        .take_rumble()
        .iter()
        .map(|r| {
            [
                f64::from(r.pad),
                f64::from(r.low),
                f64::from(r.high),
                f64::from(r.ms),
            ]
        })
        .collect()
}
