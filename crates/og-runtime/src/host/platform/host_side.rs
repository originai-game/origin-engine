//! The host's side (lifecycle members): notices in, the queue out, answers in.

use super::*;

/// Unasked host input: `{ kind, data, time? }`, applied at the next tick's
/// start (og.PlatformNotice). False for a malformed notice, an unknown kind
/// or data over the answers' cap.
pub fn og_platform_notify(engine: u32, notice: String) -> bool {
    if notice.len() > services::MAX_NOTICE {
        return false;
    }
    let Ok(v) = serde_json::from_str::<Value>(&notice) else {
        return false;
    };
    if !v["kind"]
        .as_str()
        .is_some_and(|k| NOTICE_KINDS.contains(&k))
    {
        return false;
    }
    with(engine, |p| p.notices_in.push(v));
    true
}

/// The calls made since the last take: `[{ request, method, args, tick }]`.
pub fn og_platform_take(engine: u32) -> String {
    with(engine, |p| {
        let ids = std::mem::take(&mut p.outbox);
        Value::Array(
            ids.into_iter()
                .filter_map(|id| {
                    let r = p.log.get(&id)?;
                    Some(json!({ "request": id, "method": r.method, "args": r.args, "tick": r.tick }))
                })
                .collect(),
        )
    })
    .to_string()
}

/// Answers one call; applied at the next tick's start. False for an
/// unknown, one-way or already answered request, or a malformed answer.
/// An answer longer than [`MAX_ANSWER`] answers the call with a
/// `too_large` error instead.
pub fn og_platform_answer(engine: u32, request: u32, answer: String) -> bool {
    let v = if answer.len() > MAX_ANSWER {
        json!({ "ok": false, "error": { "code": "too_large", "message": format!(
            "the host's answer is {} bytes encoded; an answer is at most {MAX_ANSWER}",
            answer.len()
        ) } })
    } else {
        let Ok(v) = serde_json::from_str::<Value>(&answer) else {
            return false;
        };
        v
    };
    if v["ok"].as_bool().is_none() {
        return false;
    }
    with(engine, |p| {
        let open = p
            .log
            .get(&request)
            .is_some_and(|r| r.answer.is_none() && !r.oneway && !r.internal)
            && !p.inbox.iter().any(|(id, _)| *id == request);
        if open {
            p.inbox.push((request, v));
        }
        open
    })
}

/// What answers this engine's calls: `{ host, capabilities, time?, clock? }`.
/// The time (UTC ms) is the host's now, between ticks: `now` reads it from
/// the current tick on.
pub fn og_platform_set_host(engine: u32, info: String) {
    let v: Value = serde_json::from_str(&info).unwrap_or(Value::Null);
    let tick = tick_now(engine).unwrap_or(0);
    let ms = tick_ms(engine);
    with(engine, |p| {
        if let Some(t) = time_of(&v) {
            rebase(p, t, tick, v["clock"].as_str(), ms);
        }
        if let Some(c) = v["clock"].as_str() {
            p.clock = Some(c.to_string());
        }
        p.host = v["host"].as_str().map(str::to_string);
        p.capabilities = v["capabilities"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|c| c.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
    });
}
