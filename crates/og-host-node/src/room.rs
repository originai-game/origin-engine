//! Rooms on the native host: `og run --room ws://127.0.0.1:P/<room>`
//! connects an engine to `og relay` (og-net's client on a thread of its
//! own). Between frames the run pumps the link: the engine's room ops go to
//! the relay, the relay's events to the engine (og_room_deliver), in order.

// napi exports, called from JS only (as render.rs): unused in the lib test build.
#![cfg_attr(test, allow(dead_code))]

use napi_derive::napi;
use og_net::client::RelayClient;
use std::cell::RefCell;
use std::collections::BTreeMap;

thread_local! {
    static LINKS: RefCell<BTreeMap<u32, RelayClient>> = const { RefCell::new(BTreeMap::new()) };
}

fn json_str(s: &str) -> String {
    serde_json::Value::String(s.to_string()).to_string()
}

/// Connect an engine's rooms to a relay (`ws://host:port/<room>` joins that
/// room at once). A Result.
#[napi(js_name = "og_node_room_connect")]
pub fn og_node_room_connect(engine: u32, url: String) -> String {
    match RelayClient::connect(&url) {
        Ok(client) => {
            og_runtime::og_room_set_host(engine, r#"{"host":"relay"}"#.into());
            LINKS.with(|l| l.borrow_mut().insert(engine, client));
            r#"{"ok":true,"value":null}"#.to_string()
        }
        Err(e) => format!(r#"{{"ok":false,"reason":{}}}"#, json_str(&e)),
    }
}

/// Carry the engine's room ops to the relay and the relay's events to the
/// engine. Returns the events delivered (0 without a link).
#[napi(js_name = "og_node_room_pump")]
pub fn og_node_room_pump(engine: u32) -> u32 {
    LINKS.with(|l| {
        let links = l.borrow();
        let Some(client) = links.get(&engine) else {
            return 0;
        };
        let ops: Vec<serde_json::Value> =
            serde_json::from_str(&og_runtime::og_room_take(engine)).unwrap_or_default();
        for op in &ops {
            client.send(op);
        }
        let mut n = 0;
        for event in client.poll() {
            og_runtime::og_room_deliver(engine, event.to_string());
            n += 1;
        }
        // What the events made lockstep send goes now, not a frame later.
        let ops: Vec<serde_json::Value> =
            serde_json::from_str(&og_runtime::og_room_take(engine)).unwrap_or_default();
        for op in &ops {
            client.send(op);
        }
        n
    })
}

/// Close an engine's relay link (its last ops are sent first).
#[napi(js_name = "og_node_room_close")]
pub fn og_node_room_close(engine: u32) -> bool {
    og_node_room_pump(engine);
    LINKS.with(|l| l.borrow_mut().remove(&engine).is_some())
}

thread_local! {
    static RELAYS: RefCell<Vec<og_net::relay::Relay>> = const { RefCell::new(Vec::new()) };
}

/// Start an `og relay` inside this process, on 127.0.0.1 (0 picks a free
/// port), for as long as the process lives: a playtest's room (PlaytestDesc.room)
/// runs its peers through one. A Result with the port.
#[napi(js_name = "og_node_relay_start")]
pub fn og_node_relay_start(port: u32) -> String {
    let Ok(port) = u16::try_from(port) else {
        return r#"{"ok":false,"reason":"the port is 0 (a free port) to 65535"}"#.into();
    };
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    match og_net::relay::Relay::bind(addr) {
        Ok(relay) => {
            let port = relay.port();
            RELAYS.with(|r| r.borrow_mut().push(relay));
            format!(r#"{{"ok":true,"value":{port}}}"#)
        }
        Err(e) => format!(
            r#"{{"ok":false,"reason":{}}}"#,
            json_str(&format!("cannot listen on {addr}: {e}"))
        ),
    }
}
