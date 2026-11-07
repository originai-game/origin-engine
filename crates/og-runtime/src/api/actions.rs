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

fn glyph_json(g: &Glyph, device: Device, control: &str) -> Value {
    json!({ "icon": g.icon, "text": g.text, "device": device.name(), "control": control })
}

fn state_of(world: u32, name: &str) -> Option<og_input::ActionState> {
    on_map(world, |m| m.state(name)).ok().flatten()
}

pub fn og_input_define_action(world: u32, desc: Option<String>) -> String {
    let r = desc_json(desc)
        .and_then(|d| ActionDesc::from_json(&d))
        .and_then(|d| on_map(world, |m| m.define(d))?);
    result_void(r)
}

pub fn og_input_actions(world: u32) -> String {
    on_map(world, |m| {
        Value::Array(m.infos().iter().map(|i| i.to_json()).collect())
    })
    .unwrap_or_else(|_| json!([]))
    .to_string()
}

pub fn og_input_action(world: u32, name: String) -> String {
    state_of(world, &name).map_or_else(|| "null".into(), |s| s.to_json().to_string())
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

pub fn og_input_action_vector(world: u32, name: String) -> String {
    let v = state_of(world, &name).map(|s| s.vector).unwrap_or_default();
    json!({ "x": f64::from(v.x), "y": f64::from(v.y) }).to_string()
}

pub fn og_input_set_action_maps(world: u32, contexts: String) {
    let list: Vec<String> = parse(Some(&contexts))
        .ok()
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|c| c.as_str().map(str::to_string))
        .collect();
    let _ = on_map(world, |m| m.set_contexts(list));
}

pub fn og_input_action_maps(world: u32) -> String {
    on_map(world, |m| json!(m.contexts()))
        .unwrap_or_else(|_| json!([]))
        .to_string()
}

pub fn og_input_rebind(world: u32, desc: Option<String>) -> String {
    let r = desc_json(desc)
        .and_then(|d| RebindDesc::from_json(&d))
        .and_then(|d| on_map(world, |m| m.rebind(&d))?);
    result(r.map(|a| a.to_json()))
}

pub fn og_input_conflicts(world: u32, desc: Option<String>) -> String {
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
    result(r.map(|a| Value::Array(a.conflicts.iter().map(|c| c.to_json()).collect())))
}

pub fn og_input_reset_bindings(world: u32, action: Option<String>) -> String {
    result_void(
        on_map(world, |m| {
            m.reset(action.as_deref().filter(|a| !a.is_empty()))
        })
        .and_then(|r| r),
    )
}

pub fn og_input_set_toggle(world: u32, action: String, on: bool) -> String {
    result_void(on_map(world, |m| m.set_toggle(&action, Some(on))).and_then(|r| r))
}

pub fn og_input_save_bindings(world: u32) -> String {
    on_map(world, |m| m.save_bindings()).unwrap_or_default()
}

pub fn og_input_load_bindings(world: u32, text: String) -> String {
    result(
        on_map(world, |m| m.load_bindings(&text))
            .and_then(|r| r)
            .map(|w| json!(w)),
    )
}

pub fn og_input_start_capture(world: u32, desc: Option<String>) -> String {
    let r = desc_json(desc)
        .and_then(|d| CaptureDesc::from_json(&d))
        .and_then(|d| on_map(world, |m| m.start_capture(d))?);
    result_void(r)
}

pub fn og_input_capture(world: u32) -> String {
    on_map(world, |m| m.capture().to_json())
        .unwrap_or_else(|_| og_input::CaptureState::default().to_json())
        .to_string()
}

pub fn og_input_cancel_capture(world: u32) {
    let _ = on_map(world, |m| m.cancel_capture());
}

pub fn og_input_apply_capture(world: u32, policy: Option<String>) -> String {
    let policy = match policy.as_deref() {
        None | Some("") => ConflictPolicy::Refuse,
        Some(p) => match ConflictPolicy::from_name(p) {
            Some(p) => p,
            None => return refuse(format!("policy is refuse, swap, replace or allow, not {p}")),
        },
    };
    result(
        on_map(world, |m| m.apply_capture(policy))
            .and_then(|r| r)
            .map(|a| a.to_json()),
    )
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

pub fn og_input_glyph(world: u32, action: String, device: Option<String>) -> String {
    match glyphs_of(world, &action, device) {
        Ok(Some((parts, d))) => match og_input::combined_glyph(&parts, d) {
            Some(g) => {
                let names: Vec<&str> = parts.iter().map(|(_, n)| n.as_str()).collect();
                glyph_json(&g, d, &names.join(" ")).to_string()
            }
            None => "null".into(),
        },
        _ => "null".into(),
    }
}

pub fn og_input_glyphs(world: u32, action: String, device: Option<String>) -> String {
    match glyphs_of(world, &action, device) {
        Ok(Some((parts, d))) => Value::Array(
            parts
                .iter()
                .map(|(g, name)| glyph_json(g, d, name))
                .collect(),
        )
        .to_string(),
        _ => "[]".into(),
    }
}

pub fn og_input_control_glyph(control: String, device: Option<String>) -> String {
    let Ok(device) = device_of(device) else {
        return "null".into();
    };
    let Some(c) = Control::parse(&control) else {
        return "null".into();
    };
    let d = device.unwrap_or(match c.class() {
        Device::Gamepad => Device::Xbox,
        other => other,
    });
    glyph_json(&c.glyph(d), d, &control).to_string()
}

pub fn og_input_device(world: u32) -> String {
    on_map(world, |m| m.device().name().to_string())
        .unwrap_or_else(|_| Device::KeyboardMouse.name().to_string())
}

pub fn og_input_pad_device(world: u32, pad: u32) -> String {
    let device = input_res(
        world,
        |g: &og_input::Gamepad| g.is_connected(pad).then(|| g.device(pad)).flatten(),
        None,
    );
    device.map_or_else(|| "null".into(), |d| json!(d.name()).to_string())
}

pub fn og_input_set_deadzones(world: u32, desc: Option<String>) -> String {
    let r = desc_json(desc).and_then(|d| {
        let num = |k: &str, def: f64| d[k].as_f64().unwrap_or(def);
        let pad = num("pad", -1.0);
        let dz = PadDeadzone {
            stick: num("stick", 0.15) as f32,
            trigger: num("trigger", 0.05) as f32,
        };
        let pad = if pad < 0.0 { None } else { Some(pad as usize) };
        on_map(world, |m| m.set_deadzone(pad, dz))?
    });
    result_void(r)
}

pub fn og_input_set_sensitivity(world: u32, desc: Option<String>) -> String {
    let r = desc_json(desc).and_then(|d| {
        let action = d["action"].as_str().unwrap_or_default().to_string();
        let device =
            device_of(d["device"].as_str().map(str::to_string))?.unwrap_or(Device::KeyboardMouse);
        let sense = og_input::Sensitivity {
            scale: d["scale"].as_f64().map_or(1.0, |x| x as f32),
            invert_x: d["invertX"].as_bool().unwrap_or(false),
            invert_y: d["invertY"].as_bool().unwrap_or(false),
        };
        on_map(world, |m| m.set_sensitivity(&action, device, sense))?
    });
    result_void(r)
}

pub fn og_input_sensitivity(world: u32, action: String, device: Option<String>) -> String {
    let device = device_of(device).ok().flatten();
    on_map(world, |m| {
        let d = device.unwrap_or(m.device());
        let s = m.sensitivity(&action, d);
        // The class's own name: any pad family is the pads'.
        let class = if d.is_pad() { d } else { d.class() };
        json!({ "action": action, "device": class.name(), "scale": s.scale, "invertX": s.invert_x, "invertY": s.invert_y })
    })
    .unwrap_or_else(|_| json!({ "action": action, "device": "keyboardMouse", "scale": 1.0, "invertX": false, "invertY": false }))
    .to_string()
}

pub fn og_input_set_keyboard_layout(labels: String) {
    let v: Value = serde_json::from_str(&labels).unwrap_or(Value::Null);
    og_input::set_key_labels(v.as_array().into_iter().flatten().filter_map(|e| {
        Some((
            e["code"].as_str()?.to_string(),
            e["label"].as_str()?.to_string(),
        ))
    }));
}

pub fn og_input_keyboard_layout() -> String {
    Value::Array(
        og_input::key_labels()
            .into_iter()
            .map(|(code, label)| json!({ "code": code, "label": label }))
            .collect(),
    )
    .to_string()
}

pub fn og_input_set_mouse(world: u32, desc: Option<String>) -> String {
    let r = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let d = desc_json(desc)?;
        let mut rs = eng
            .render
            .try_borrow_mut()
            .map_err(|_| "the renderer is busy drawing".to_string())?;
        if let Some(raw) = d["raw"].as_bool() {
            rs.display.mouse.raw = raw;
        }
        Ok(())
    })();
    result_void(r)
}

pub fn og_input_mouse(world: u32) -> String {
    let m = engine(world)
        .ok()
        .and_then(|eng| eng.render.try_borrow().ok().map(|rs| rs.display.mouse))
        .unwrap_or_default();
    json!({ "raw": m.raw, "rawActive": m.raw_active, "rawUnsupported": m.raw_unsupported })
        .to_string()
}

pub fn og_input_rumble(world: u32, desc: Option<String>) -> String {
    let r = desc_json(desc).and_then(|d| {
        let num = |k: &str, def: f64| d[k].as_f64().unwrap_or(def);
        let r = Rumble {
            pad: num("pad", -1.0) as i32,
            low: num("low", 0.0) as f32,
            high: num("high", 0.0) as f32,
            ms: num("ms", 200.0) as f32,
        };
        on_map(world, |m| m.rumble(r))?
    });
    result_void(r)
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
