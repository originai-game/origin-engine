use super::*;
use std::collections::BTreeMap;

/// The bindings under test as the hosts send their answers.
macro_rules! sent {
        ($($writer:ident $name:ident($($arg:ident: $t:ty),*);)*) => {$(
            fn $name($($arg: $t),*) -> String {
                crate::host::host_impl::$writer(super::$name($($arg),*))
            }
        )*};
    }
sent! {
    answer og_platform_store_get(world: u32, key: String);
    answer og_platform_save_slot(world: u32, desc: Option<String>);
    answer og_platform_load_slot(world: u32, desc: Option<String>);
    answer og_platform_products(world: u32, desc: Option<String>);
    answer og_platform_purchase(world: u32, desc: Option<String>);
    answer og_platform_consume(world: u32, purchase: String);
    answer og_platform_show_ad(world: u32, desc: Option<String>);
    answer og_platform_achievement_progress(world: u32, desc: Option<String>);
    answer og_platform_invite_send(world: u32, desc: Option<String>);
    answer og_platform_share(world: u32, desc: Option<String>);
    json og_platform_result(world: u32, request: u32);
    json og_platform_notice(world: u32, notice: u32);
}

fn value(answer: &str) -> Value {
    let v: Value = serde_json::from_str(answer).expect("JSON");
    assert_eq!(v["ok"], json!(true), "{answer}");
    v["value"].clone()
}

fn engine_at_60() -> u32 {
    let game = crate::og_game_game(Some(json!({ "tick": 60 }).to_string()));
    value(&crate::og_engine_create(game, "{}".into()))
        .as_u64()
        .expect("an engine id") as u32
}

fn host(engine: u32, time: f64, clock: &str) {
    og_platform_set_host(
        engine,
        json!({ "host": "test", "capabilities": ["store"], "time": time, "clock": clock })
            .to_string(),
    );
}

fn advance(engine: u32, steps: u32) {
    value(&crate::og_engine_advance(engine, steps));
}

#[test]
fn now_never_goes_back_on_one_clock_and_starts_afresh_on_another() {
    let e = engine_at_60();
    host(e, 1_000_000.0, "browser");
    assert_eq!(og_platform_now(e), 1_000_000.0);
    // A harness advance runs ahead of the host's clock.
    advance(e, 600);
    let ahead = og_platform_now(e);
    assert!(ahead >= 1_009_999.0, "{ahead}");
    // The host retells an older time on the same clock.
    host(e, 1_000_500.0, "browser");
    let retold = og_platform_now(e);
    assert!(
        retold >= ahead,
        "set_host moved now back: {ahead} → {retold}"
    );
    // An answer stamped earlier still (browser-stamped: no clock).
    let id = value(&og_platform_store_get(e, "k".into()))
        .as_u64()
        .unwrap() as u32;
    assert!(og_platform_answer(
        e,
        id,
        json!({ "ok": true, "data": null, "time": 1_000_600.0 }).to_string()
    ));
    advance(e, 1);
    let answered = og_platform_now(e);
    assert!(
        answered >= retold,
        "an answer moved now back: {retold} → {answered}"
    );
    let result: Value = serde_json::from_str(&og_platform_result(e, id)).unwrap();
    assert_eq!(result["ok"], json!(true), "{result}");
    // Another clock is another time line: it is taken as told.
    host(e, 5.0, "fixed");
    assert_eq!(og_platform_now(e), 5.0);
}

#[test]
fn calls_waiting_and_answers_are_bounded() {
    let e = engine_at_60();
    host(e, 1.0, "fixed");
    let first = value(&og_platform_store_get(e, "k".into()))
        .as_u64()
        .unwrap() as u32;
    // An answer past MAX_ANSWER answers its call with too_large.
    let long = json!({ "ok": true, "data": "x".repeat(MAX_ANSWER) }).to_string();
    assert!(og_platform_answer(e, first, long));
    advance(e, 1);
    let result: Value = serde_json::from_str(&og_platform_result(e, first)).unwrap();
    assert_eq!(result["ok"], json!(false), "{result}");
    assert_eq!(result["error"]["code"], json!("too_large"), "{result}");
    // MAX_PENDING calls wait; the next is refused by name.
    for _ in 0..MAX_PENDING {
        value(&og_platform_store_get(e, "k".into()));
    }
    let refused: Value = serde_json::from_str(&og_platform_store_get(e, "k".into())).unwrap();
    assert_eq!(refused["ok"], json!(false), "{refused}");
    let reason = refused["reason"].as_str().unwrap_or_default();
    assert!(reason.contains("waiting"), "{reason}");
    assert_eq!(og_platform_store_list(e), 0);
    // One-way telemetry is not held back by them.
    let sent: Value = serde_json::from_str(&crate::host::host_impl::answer(og_platform_telemetry(
        e,
        "t".into(),
        None,
    )))
    .unwrap();
    assert_eq!(sent["ok"], json!(true), "{sent}");
}

/// A store as a host keeps it: key → (value, revision). Answers every
/// call taken, except the store.set calls `fail` names by key.
fn serve(e: u32, store: &mut BTreeMap<String, (Value, u64)>, fail: &[&str]) -> usize {
    let calls: Vec<Value> = serde_json::from_str(&og_platform_take(e)).unwrap();
    for c in &calls {
        let id = c["request"].as_u64().unwrap() as u32;
        let a = &c["args"];
        let key = a["key"].as_str().unwrap_or("").to_string();
        let answer = match c["method"].as_str().unwrap() {
            "store.get" => {
                let (value, revision) = store.get(&key).cloned().unwrap_or((Value::Null, 0));
                json!({ "ok": true, "data": { "key": key, "value": value, "revision": revision } })
            }
            "store.set" if fail.contains(&key.as_str()) => {
                json!({ "ok": false, "error": { "code": "offline", "message": "no network" } })
            }
            "store.set" => {
                let at = store.get(&key).map_or(0, |v| v.1);
                let create = a["create"].as_bool() == Some(true);
                let revision = a["revision"].as_u64().unwrap_or(0);
                if (create && at != 0) || (!create && revision != 0 && revision != at) {
                    json!({ "ok": false, "error": { "code": "conflict", "message": "moved", "revision": at } })
                } else {
                    store.insert(key.clone(), (a["value"].clone(), at + 1));
                    json!({ "ok": true, "data": { "key": key, "revision": at + 1 } })
                }
            }
            m => panic!("unexpected {m}"),
        };
        assert!(og_platform_answer(e, id, answer.to_string()));
    }
    calls.len()
}

/// [`serve`], every answer stamped with the host's `time` as og run's
/// host does.
fn serve_at(e: u32, store: &mut BTreeMap<String, (Value, u64)>, time: f64) {
    let calls: Vec<Value> = serde_json::from_str(&og_platform_take(e)).unwrap();
    for c in &calls {
        let id = c["request"].as_u64().unwrap() as u32;
        let a = &c["args"];
        let key = a["key"].as_str().unwrap_or("").to_string();
        let data = match c["method"].as_str().unwrap() {
            "store.get" => {
                let (value, revision) = store.get(&key).cloned().unwrap_or((Value::Null, 0));
                json!({ "key": key, "value": value, "revision": revision })
            }
            "store.set" => {
                let at = store.get(&key).map_or(0, |v| v.1);
                store.insert(key.clone(), (a["value"].clone(), at + 1));
                json!({ "key": key, "revision": at + 1 })
            }
            m => panic!("unexpected {m}"),
        };
        let answer = json!({ "ok": true, "data": data, "time": time });
        assert!(og_platform_answer(e, id, answer.to_string()));
    }
}

#[test]
fn a_slot_calls_answer_carries_the_hosts_time_as_a_store_answer_does() {
    // brasswick FINDINGS #7: under og run, store.get answered the
    // host's clock while slot.load and slot.save answered time 0.
    let e = engine_at_60();
    host(e, 1_000_000.0, "fixed");
    let mut store = BTreeMap::new();
    let mut settle_at = |id: u32, time: f64| {
        for _ in 0..8 {
            serve_at(e, &mut store, time);
            advance(e, 1);
            let r: Value = serde_json::from_str(&og_platform_result(e, id)).unwrap();
            if !r.is_null() {
                return r;
            }
        }
        panic!("request {id} was never answered");
    };
    let get = request_id(&og_platform_store_get(e, "progress".into()));
    assert_eq!(settle_at(get, 1_000_500.0)["time"], json!(1_000_500.0));
    let save = request_id(&og_platform_save_slot(
        e,
        Some(json!({ "slot": "slot1", "text": "og-save:1:abc", "meta": { "at": 1 } }).to_string()),
    ));
    let r = settle_at(save, 1_002_000.0);
    assert_eq!(r["ok"], json!(true), "{r}");
    assert_eq!(r["time"], json!(1_002_000.0), "slot.save: {r}");
    for meta_only in [true, false] {
        let load = request_id(&og_platform_load_slot(
            e,
            Some(json!({ "slot": "slot1", "metaOnly": meta_only }).to_string()),
        ));
        let r = settle_at(load, 1_003_000.0);
        assert_eq!(r["ok"], json!(true), "{r}");
        assert_eq!(
            r["time"],
            json!(1_003_000.0),
            "slot.load (metaOnly {meta_only}): {r}"
        );
    }
}

/// Serves and ticks until request `id` is answered.
fn settle(e: u32, id: u32, store: &mut BTreeMap<String, (Value, u64)>, fail: &[&str]) -> Value {
    for _ in 0..8 {
        serve(e, store, fail);
        advance(e, 1);
        let r: Value = serde_json::from_str(&og_platform_result(e, id)).unwrap();
        if !r.is_null() {
            return r;
        }
    }
    panic!("request {id} was never answered");
}

fn request_id(answer: &str) -> u32 {
    value(answer).as_u64().unwrap() as u32
}

#[test]
fn a_slot_takes_a_text_over_a_key_in_banks_and_a_save_cut_short_leaves_the_last_one() {
    let e = engine_at_60();
    host(e, 1.0, "fixed");
    let mut store = BTreeMap::new();
    // Never saved: no text, no meta.
    let id = request_id(&og_platform_load_slot(
        e,
        Some(json!({ "slot": "slot1" }).to_string()),
    ));
    let r = settle(e, id, &mut store, &[]);
    assert_eq!(r["ok"], json!(true), "{r}");
    assert_eq!(r["data"]["text"], Value::Null, "{r}");
    // 150 KB with quotes and non-ASCII: three parts, each a store value.
    let text: String = (0..150_000).map(|i| ['a', '"', '灯', 'z'][i % 4]).collect();
    let save = |e, text: &str, place: u32| {
        request_id(&og_platform_save_slot(
            e,
            Some(json!({ "slot": "slot1", "text": text, "meta": { "place": place } }).to_string()),
        ))
    };
    let r = settle(e, save(e, &text, 1), &mut store, &[]);
    assert_eq!(r["ok"], json!(true), "{r}");
    let parts = r["data"]["parts"].as_u64().unwrap();
    assert!(parts >= 3, "{r}");
    assert!(store.contains_key("slot1.a0") && !store.contains_key("slot1.b0"));
    for (k, (v, _)) in &store {
        assert!(v.to_string().len() <= MAX_VALUE, "{k} is over a key");
    }
    // Only the slot call's answer reaches the game as an event; its own
    // record was never sent to the host.
    let dump = dump(e);
    let slot_call = dump["calls"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["method"] == "slot.save")
        .unwrap();
    assert_eq!(slot_call["state"], json!("answered"));
    // A second save goes in the other bank; one of its parts refused,
    // the manifest still names the first, whole.
    let other: String = text.chars().rev().collect();
    let r = settle(e, save(e, &other, 2), &mut store, &["slot1.b1"]);
    assert_eq!(r["ok"], json!(false), "{r}");
    assert_eq!(r["error"]["code"], json!("offline"), "{r}");
    assert!(
        r["error"]["message"]
            .as_str()
            .unwrap()
            .contains("writing slot 'slot1'"),
        "{r}"
    );
    let id = request_id(&og_platform_load_slot(
        e,
        Some(json!({ "slot": "slot1" }).to_string()),
    ));
    let r = settle(e, id, &mut store, &[]);
    assert_eq!(r["data"]["text"].as_str(), Some(text.as_str()));
    assert_eq!(r["data"]["meta"], json!({ "place": 1 }));
    // Saved again in full: the b bank, and the meta alone reads without the parts.
    let r = settle(e, save(e, &other, 2), &mut store, &[]);
    assert_eq!(r["ok"], json!(true), "{r}");
    assert_eq!(store["slot1"].0["bank"], json!("b"));
    let id = request_id(&og_platform_load_slot(
        e,
        Some(json!({ "slot": "slot1", "metaOnly": true }).to_string()),
    ));
    let r = settle(e, id, &mut store, &[]);
    assert_eq!(
        [r["data"]["meta"].clone(), r["data"]["text"].clone()],
        [json!({ "place": 2 }), Value::Null]
    );
    let id = request_id(&og_platform_load_slot(
        e,
        Some(json!({ "slot": "slot1" }).to_string()),
    ));
    assert_eq!(
        settle(e, id, &mut store, &[])["data"]["text"].as_str(),
        Some(other.as_str())
    );
    // A part changed under the manifest is refused, not joined.
    store.get_mut("slot1.b0").unwrap().0 = json!("tampered");
    let id = request_id(&og_platform_load_slot(
        e,
        Some(json!({ "slot": "slot1" }).to_string()),
    ));
    let r = settle(e, id, &mut store, &[]);
    assert_eq!(r["error"]["code"], json!("invalid"), "{r}");
    // Refused at the call: a bad name, a text over eight parts.
    let bad: Value = serde_json::from_str(&og_platform_save_slot(
        e,
        Some(json!({ "slot": "a b", "text": "x" }).to_string()),
    ))
    .unwrap();
    assert_eq!(bad["ok"], json!(false));
    let huge = "x".repeat(MAX_PARTS * MAX_VALUE);
    let bad: Value = serde_json::from_str(&og_platform_save_slot(
        e,
        Some(json!({ "slot": "s", "text": huge }).to_string()),
    ))
    .unwrap();
    assert!(
        bad["reason"].as_str().unwrap().contains("at most 8"),
        "{bad}"
    );
}

#[test]
fn a_host_notice_from_the_framing_page_passes_through_unread() {
    let e = engine_at_60();
    host(e, 1.0, "fixed");
    let data = json!({ "name": "hero.show", "scene": "lantern-shrine" });
    assert!(og_platform_notify(
        e,
        json!({ "kind": "host", "data": data }).to_string()
    ));
    advance(e, 1);
    let n: Value = serde_json::from_str(&og_platform_notice(e, 1)).unwrap();
    assert_eq!(n["kind"], json!("host"), "{n}");
    assert_eq!(n["data"], data, "{n}");
}

#[test]
fn a_notice_is_applied_at_the_next_tick_numbered_and_kept() {
    let e = engine_at_60();
    host(e, 1.0, "fixed");
    assert!(!og_platform_notify(
        e,
        json!({ "kind": "lottery" }).to_string()
    ));
    assert!(!og_platform_notify(e, "not json".into()));
    let data = json!({ "purchase": "p-1", "product": "gems", "quantity": 1, "state": "completed" });
    assert!(og_platform_notify(
        e,
        json!({ "kind": "purchase", "data": data, "time": 5000.0 }).to_string()
    ));
    // Host input, like an answer: nothing until the tick's start.
    assert_eq!(og_platform_notice(e, 1), "null");
    let applied = tick_now(e).unwrap();
    advance(e, 1);
    let n: Value = serde_json::from_str(&og_platform_notice(e, 1)).unwrap();
    assert_eq!(n["kind"], json!("purchase"), "{n}");
    assert_eq!(n["data"], data, "{n}");
    assert_eq!(n["time"], json!(5000.0), "{n}");
    assert_eq!(n["tick"], json!(applied), "{n}");
    let dump = dump(e);
    assert_eq!(dump["notices"][0]["data"], data, "{dump}");
    // The last KEEP_NOTICES are kept.
    for _ in 0..KEEP_NOTICES {
        assert!(og_platform_notify(
            e,
            json!({ "kind": "invite" }).to_string()
        ));
    }
    advance(e, 1);
    assert_eq!(og_platform_notice(e, 1), "null");
    assert_ne!(og_platform_notice(e, KEEP_NOTICES as u32 + 1), "null");
}

#[test]
fn services_validate_their_arguments_and_name_their_methods() {
    let e = engine_at_60();
    og_platform_set_host(
        e,
        json!({ "host": "test", "capabilities": ["purchase.buy", "ad.show"] }).to_string(),
    );
    assert!(og_platform_supports(e, "ad.show".into()));
    assert!(!og_platform_supports(e, "share".into()));
    let refused = |answer: String| {
        let v: Value = serde_json::from_str(&answer).unwrap();
        assert_eq!(v["ok"], json!(false), "{v}");
    };
    refused(og_platform_purchase(
        e,
        Some(json!({ "product": "a b" }).to_string()),
    ));
    refused(og_platform_consume(e, String::new()));
    refused(og_platform_show_ad(
        e,
        Some(json!({ "kind": "popup" }).to_string()),
    ));
    refused(og_platform_achievement_progress(
        e,
        Some(json!({ "id": "a", "percent": 101 }).to_string()),
    ));
    refused(og_platform_invite_send(
        e,
        Some(json!({ "payload": "x".repeat(1100) }).to_string()),
    ));
    refused(og_platform_share(
        e,
        Some(json!({ "text": "" }).to_string()),
    ));
    refused(og_platform_share(
        e,
        Some(json!({ "text": "hi", "path": "https://x.y?a" }).to_string()),
    ));
    refused(og_platform_products(
        e,
        Some(json!({ "ids": vec!["a"; 101] }).to_string()),
    ));
    value(&og_platform_purchase(
        e,
        Some(json!({ "product": "gems.100", "quantity": 500 }).to_string()),
    ));
    value(&og_platform_show_ad(e, None));
    og_platform_achievements(e);
    let calls: Vec<Value> = serde_json::from_str(&og_platform_take(e)).unwrap();
    let shapes: Vec<(String, Value)> = calls
        .iter()
        .map(|c| (c["method"].as_str().unwrap().to_string(), c["args"].clone()))
        .collect();
    assert_eq!(
        shapes,
        vec![
            (
                "purchase.buy".into(),
                json!({ "product": "gems.100", "quantity": 99 })
            ),
            (
                "ad.show".into(),
                json!({ "kind": "rewarded", "placement": "default" })
            ),
            ("achievement.list".into(), json!({})),
        ]
    );
}

fn snapshot_of(e: u32) -> String {
    value(&crate::host::host_impl::answer(crate::og_world_snapshot(
        e, None,
    )))
    .as_str()
    .unwrap()
    .to_string()
}

#[test]
fn a_purchase_is_granted_once_across_a_restore_and_a_replayed_notice() {
    let e = engine_at_60();
    host(e, 1.0, "fixed");
    // A world that never grants keeps its type table: no og.PlatformGrants.
    let untouched = engine_at_60();
    assert!(
        !crate::save_snapshot_bytes(&snapshot_of(untouched))
            .ok()
            .and_then(|b| og_ecs::snapshot_type_names(&b).ok())
            .is_some_and(|n| n.iter().any(|t| t == GRANTS))
    );
    // The notice that the purchase completed: the game grants it.
    let notice = json!({ "kind": "purchase", "data": { "purchase": "p-7", "product": "gems", "quantity": 1, "state": "completed" } });
    let grant_from_notice = |n: u32| {
        let v: Value = serde_json::from_str(&og_platform_notice(e, n)).unwrap();
        let id = v["data"]["purchase"].as_str().unwrap().to_string();
        value(&crate::host::host_impl::answer(og_platform_grant(e, id)))
            .as_bool()
            .unwrap()
    };
    assert!(og_platform_notify(e, notice.to_string()));
    advance(e, 1);
    assert!(grant_from_notice(1), "the first grant is new");
    assert!(og_platform_granted(e, "p-7".into()));
    // Saved after the grant, before consume.
    let saved = snapshot_of(e);
    assert!(
        og_ecs::snapshot_type_names(&crate::save_snapshot_bytes(&saved).unwrap())
            .unwrap()
            .iter()
            .any(|t| t == GRANTS)
    );
    value(&og_platform_consume(e, "p-7".into()));
    // A fresh engine (a relaunch) restores the save, and the host sends
    // the notice again: still granted once.
    let relaunch = engine_at_60();
    host(relaunch, 1.0, "fixed");
    assert!(!og_platform_granted(relaunch, "p-7".into()));
    let restored: Value = serde_json::from_str(&crate::host::host_impl::answer(
        crate::og_world_restore(relaunch, saved, None),
    ))
    .unwrap();
    assert_eq!(restored["ok"], json!(true), "{restored}");
    assert!(og_platform_granted(relaunch, "p-7".into()));
    assert!(og_platform_notify(relaunch, notice.to_string()));
    advance(relaunch, 1);
    let v: Value = serde_json::from_str(&og_platform_notice(relaunch, 1)).unwrap();
    let again = value(&crate::host::host_impl::answer(og_platform_grant(
        relaunch,
        v["data"]["purchase"].as_str().unwrap().to_string(),
    )));
    assert_eq!(again, json!(false), "a replayed notice granted twice");
    // Another purchase is new; a malformed id is refused.
    assert_eq!(
        value(&crate::host::host_impl::answer(og_platform_grant(
            relaunch,
            "p-8".into()
        ))),
        json!(true)
    );
    let bad: Value = serde_json::from_str(&crate::host::host_impl::answer(og_platform_grant(
        relaunch,
        "no spaces".into(),
    )))
    .unwrap();
    assert_eq!(bad["ok"], json!(false));
}
