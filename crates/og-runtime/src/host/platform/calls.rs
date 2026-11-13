//! The game's calls: the key-value store, settings, leaderboards, telemetry, points and AI marks.

use super::*;

pub(super) fn valid_key(key: &str) -> bool {
    (1..=64).contains(&key.len())
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// A board name, `main` when it is not a key.
pub(super) fn board_of(board: String) -> String {
    if valid_key(&board) {
        board
    } else {
        "main".to_string()
    }
}

pub fn og_platform_store_get(world: u32, key: String) -> Result<u32, String> {
    if !valid_key(&key) {
        return Err(format!(
            "'{key}' is not a store key: 1 to 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    request(world, "store.get", json!({ "key": key }))
}

pub fn og_platform_store_set(world: u32, desc: Option<String>) -> Result<u32, String> {
    let d: crate::host::wire::StoreSetDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    let key = d.key;
    if !valid_key(&key) {
        return Err(format!(
            "'{key}' is not a store key: 1 to 64 of letters, digits, '.', '_' and '-'"
        ));
    }
    let value = d.value;
    let size = value.to_string().len();
    if size > MAX_VALUE {
        return Err(format!(
            "the value is {size} bytes encoded; a store value is at most {MAX_VALUE}"
        ));
    }
    let revision = d.revision;
    let create = d.create;
    request(
        world,
        "store.set",
        json!({ "key": key, "value": value, "revision": revision, "create": create }),
    )
}

/// The key og.platform's settings helpers use.
pub const SETTINGS_KEY: &str = "og.settings";

pub fn og_platform_load_settings(world: u32) -> u32 {
    call(world, "store.get", json!({ "key": SETTINGS_KEY }), false).unwrap_or(0)
}

pub fn og_platform_save_settings(world: u32, value: String) -> Result<u32, String> {
    let v = parse(Some(&value))?;
    if !v.is_object() {
        return Err("settings are an object: { volume: 0.8, ... }".into());
    }
    if v.to_string().len() > MAX_VALUE {
        return Err(format!("settings are at most {MAX_VALUE} bytes encoded"));
    }
    request(
        world,
        "store.set",
        json!({ "key": SETTINGS_KEY, "value": v, "revision": 0 }),
    )
}

pub fn og_platform_store_list(world: u32) -> u32 {
    call(world, "store.list", json!({}), false).unwrap_or(0)
}

pub fn og_platform_leaderboard_submit(world: u32, desc: Option<String>) -> Result<u32, String> {
    let d: crate::host::wire::LeaderboardSubmitDesc =
        crate::host::wire::arg(desc.as_deref(), "desc")?;
    let score = match d.score {
        Some(s) if s.is_finite() => s,
        Some(s) => return Err(format!("desc.score: expected a finite number, got {s}")),
        None => return Err("desc.score: expected a finite number, got nothing".into()),
    };
    request(
        world,
        "leaderboard.submit",
        json!({ "board": board_of(d.board), "score": score, "order": d.order.as_str() }),
    )
}

pub(super) fn query(world: u32, method: &str, desc: Option<String>) -> u32 {
    let d: crate::host::wire::LeaderboardQueryDesc =
        crate::host::wire::arg(desc.as_deref(), "desc").unwrap_or_default();
    call(
        world,
        method,
        json!({ "board": board_of(d.board), "count": d.count.clamp(1, 100) }),
        false,
    )
    .unwrap_or(0)
}

pub fn og_platform_leaderboard_top(world: u32, desc: Option<String>) -> u32 {
    query(world, "leaderboard.top", desc)
}

pub fn og_platform_leaderboard_around(world: u32, desc: Option<String>) -> u32 {
    query(world, "leaderboard.around", desc)
}

pub fn og_platform_telemetry(
    world: u32,
    name: String,
    attrs: Option<String>,
) -> Result<(), String> {
    let named = !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_'));
    if !named {
        return Err(format!(
            "'{name}' is not an event name: 1 to 64 of letters, digits, '.' and '_'"
        ));
    }
    let attrs = {
        let a = attrs.as_deref().map(|a| parse(Some(a))).transpose()?;
        a.unwrap_or(json!({}))
    };
    let Some(o) = attrs.as_object() else {
        return Err("attrs is an object of strings, numbers, booleans or null".into());
    };
    if o.len() > 16 || o.values().any(|v| v.is_array() || v.is_object()) {
        return Err("attrs holds at most 16 strings, numbers, booleans or null".into());
    }
    let key_ok = |k: &str| {
        let mut b = k.bytes();
        b.next().is_some_and(|c| c.is_ascii_lowercase())
            && b.all(|c| c.is_ascii_alphanumeric() || c == b'_')
    };
    if let Some(k) = o.keys().find(|k| !key_ok(k)) {
        return Err(format!(
            "attribute '{k}' is not a name: a lowercase letter, then letters, digits and '_'"
        ));
    }
    if o.values()
        .any(|v| v.as_str().is_some_and(|t| t.chars().count() > 512))
    {
        return Err("an attribute's string is at most 512 characters".into());
    }
    match call(
        world,
        "telemetry",
        json!({ "name": name, "attrs": attrs }),
        true,
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn og_platform_point(world: u32, desc: Option<String>) -> Result<u32, String> {
    let d: crate::host::wire::PointDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    let reason = d.reason.trim().to_string();
    if reason.is_empty() || !valid_key(&reason) {
        return Err(format!(
            "'{reason}' is not a reason: 1 to 64 of letters, digits, '.', '_' and '-', as `level.completed`"
        ));
    }
    let amount = d.amount.clamp(1, 1000);
    request(
        world,
        "point",
        json!({ "reason": reason, "amount": amount }),
    )
}

pub fn og_platform_ai_mark(world: u32, desc: Option<String>) -> Result<u32, String> {
    let d: crate::host::wire::AiMarkDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    let kind = d.kind.trim().to_string();
    let label = d.label.trim().to_string();
    let kind_ok = kind.len() <= 32
        && kind.bytes().next().is_some_and(|c| c.is_ascii_lowercase())
        && kind
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'_' | b'-'));
    if !kind_ok || label.is_empty() || label.chars().count() > 120 {
        return Err("an AI mark needs a kind (1 to 32 of a-z, 0-9, '_' and '-', a letter first: `text`, `image`) and a label of 1 to 120 characters".into());
    }
    let mut args = json!({ "kind": kind, "label": label });
    if let Some(m) = Some(d.model.as_str()).filter(|m| !m.is_empty()) {
        if m.chars().count() > 80 {
            return Err("an AI mark's model is at most 80 characters".into());
        }
        args["model"] = json!(m);
    }
    request(world, "ai.mark", args)
}
