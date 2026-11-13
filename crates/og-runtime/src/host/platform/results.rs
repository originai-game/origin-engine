//! What a call came to: its result, the answers still withheld, the host's info and a notice.

use super::*;

/// An answer's or a hello's time: finite UTC milliseconds above 0.
pub(super) fn time_of(v: &Value) -> Option<f64> {
    v["time"].as_f64().filter(|t| t.is_finite() && *t > 0.0)
}

pub(super) fn result_of(id: u32, r: &Record) -> Option<PlatformResult> {
    let (answer, tick) = r.answer.as_ref()?;
    let ok = answer["ok"].as_bool() == Some(true);
    let error = (!ok).then(|| {
        let e = &answer["error"];
        let code = e["code"].as_str().unwrap_or("failed");
        PlatformError {
            code: code.to_string(),
            message: e["message"].as_str().unwrap_or("").to_string(),
            retryable: e["retryable"]
                .as_bool()
                .unwrap_or(RETRYABLE.contains(&code)),
            revision: e["revision"].as_u64().unwrap_or(0) as u32,
        }
    });
    Some(PlatformResult {
        request: id,
        method: r.method.clone(),
        ok,
        data: if ok {
            answer.get("data").cloned().unwrap_or(Value::Null)
        } else {
            Value::Null
        },
        error,
        tick: *tick as u32,
        time: time_of(answer).unwrap_or(0.0),
    })
}

/// Whether the running system may not hear platform answers and notices:
/// inside a running lockstep or rollback session, a system that writes
/// shared state (anything but the session's own records, og-net's
/// SESSION_RESOURCES and SESSION_EVENTS). An answer is this peer's alone,
/// so applying it to shared state would make this world differ from the
/// other peers'; such a system hears nothing, and the engine logs it once.
pub(crate) fn withheld(world: u32) -> bool {
    let Some(f) = running(world) else {
        return false;
    };
    if !crate::bind::net::session_running(world) {
        return false;
    }
    let Ok(eng) = engine(world) else {
        return false;
    };
    let session = |n: &str| {
        og_net::state::SESSION_RESOURCES.contains(&n) || og_net::state::SESSION_EVENTS.contains(&n)
    };
    let mut st = eng.state.borrow_mut();
    let Some(s) = st.systems.get(f.system) else {
        return false;
    };
    let Some(write) = s.writes.iter().find(|w| !session(w)).cloned() else {
        return false;
    };
    let system = s.name.clone();
    if st
        .undeclared_logged
        .insert(format!("{system} platform session"))
    {
        let tick = tick_now(world).unwrap_or(0);
        st.log.push(json!({ "tick": tick, "warning": true, "text": format!(
            "system '{system}' writes '{write}', shared state, so in this lockstep session it hears no platform answer or notice: they are this peer's alone. Read them in a system that writes none, and carry what every peer must see in og.PlayerInput (og.net.submit)"
        ) }));
    }
    true
}

pub fn og_platform_result(world: u32, request: u32) -> Option<PlatformResult> {
    if withheld(world) {
        return None;
    }
    with(world, |p| {
        p.log.get(&request).and_then(|r| result_of(request, r))
    })
}

pub fn og_platform_info(world: u32) -> PlatformInfo {
    with(world, |p| PlatformInfo {
        host: p.host.clone().unwrap_or_else(|| "none".into()),
        capabilities: p.capabilities.clone(),
        pending: p
            .log
            .values()
            .filter(|r| r.answer.is_none() && !r.oneway)
            .count() as u32,
        clock: p.clock.clone().unwrap_or_else(|| "none".into()),
    })
}

pub fn og_platform_notice(world: u32, notice: u32) -> Option<PlatformNoticeData> {
    if withheld(world) {
        return None;
    }
    with(world, |p| {
        p.notices.get(&notice).map(|n| PlatformNoticeData {
            notice,
            kind: n.kind.clone(),
            data: n.data.clone(),
            tick: n.tick as u32,
            time: n.time,
        })
    })
}
