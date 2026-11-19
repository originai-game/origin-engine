//! Shared worlds on the native host: `og run --shared ws://127.0.0.1:P/`
//! (with `--shared-token-file`) gives an engine a static session provider,
//! one fixed endpoint and token; an `https://` endpoint is WebTransport,
//! its certificate checked against `--shared-cert-hash` when given. Between frames the run pumps the link: the
//! engine's ops (og_shared_take) open, carry and close one og-shared client
//! connection, and what the connection says goes back (og_shared_deliver),
//! in order. Each `connect` op opens a new connection; reconnecting is the
//! engine's.

// napi exports, called from JS only (as room.rs): unused in the lib test build.
#![cfg_attr(test, allow(dead_code))]

use napi_derive::napi;
use og_shared::client::{Client, ClientEvent};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;

struct Link {
    url: String,
    token: String,
    /// For an `https://` URL: the certificate's SHA-256s (none: the system's roots).
    hashes: Vec<[u8; 32]>,
    client: Option<Client>,
}

thread_local! {
    static LINKS: RefCell<BTreeMap<u32, Link>> = const { RefCell::new(BTreeMap::new()) };
}

/// Give an engine a static session provider: `url` (`ws://`, `wss://`, or
/// `https://` for WebTransport), the token sent as it is, and for
/// WebTransport the certificate's SHA-256 in hex (empty: the system's
/// roots). The engine hears that sessions are carried.
#[napi(js_name = "og_node_shared_attach")]
pub fn og_node_shared_attach(engine: u32, url: String, token: String, cert_hash: String) -> bool {
    let wt = url.starts_with("https://");
    if !(wt || url.starts_with("ws://") || url.starts_with("wss://")) {
        return false;
    }
    let hashes = match (wt, cert_hash.is_empty()) {
        (_, true) => Vec::new(),
        (true, false) => match og_shared::wt::parse_hash(&cert_hash) {
            Ok(h) => vec![h],
            Err(_) => return false,
        },
        (false, false) => return false,
    };
    LINKS.with(|l| {
        l.borrow_mut().insert(
            engine,
            Link {
                url,
                token,
                hashes,
                client: None,
            },
        )
    });
    og_runtime::og_shared_deliver(
        engine,
        json!({ "type": "host", "host": "static" }).to_string(),
    )
}

fn carry(engine: u32, link: &mut Link) {
    let ops: Vec<Value> =
        serde_json::from_str(&og_runtime::og_shared_take(engine)).unwrap_or_default();
    for op in ops {
        match op["op"].as_str() {
            Some("connect") => {
                link.client = Some(if link.url.starts_with("https://") {
                    Client::connect_wt(&link.url, link.hashes.clone())
                } else {
                    Client::connect(&link.url)
                })
            }
            Some("send") => {
                if let (Some(c), Some(b)) = (
                    &link.client,
                    op["bytes"].as_str().and_then(og_core::base64::decode),
                ) {
                    c.send(b);
                }
            }
            Some("close") => link.client = None,
            _ => {}
        }
    }
}

/// Carry the engine's ops to its connection and what the connection says
/// back. Returns the events delivered (0 without a link).
#[napi(js_name = "og_node_shared_pump")]
pub fn og_node_shared_pump(engine: u32) -> u32 {
    LINKS.with(|l| {
        let mut links = l.borrow_mut();
        let Some(link) = links.get_mut(&engine) else {
            // No provider here: a join is told so (og.SharedLeft `unavailable`).
            let ops: Vec<Value> =
                serde_json::from_str(&og_runtime::og_shared_take(engine)).unwrap_or_default();
            if !ops.iter().any(|op| op["op"] == "connect") {
                return 0;
            }
            og_runtime::og_shared_deliver(
                engine,
                json!({ "type": "host", "host": "none" }).to_string(),
            );
            return 1;
        };
        carry(engine, link);
        let mut n = 0;
        let events = link.client.as_ref().map(Client::poll).unwrap_or_default();
        for e in events {
            let v = match e {
                ClientEvent::Open => {
                    let transport = if link.url.starts_with("https://") {
                        "wt"
                    } else {
                        "ws"
                    };
                    json!({ "type": "open", "transport": transport, "token": link.token })
                }
                ClientEvent::Message(b) => {
                    json!({ "type": "message", "bytes": og_core::base64::encode(&b) })
                }
                ClientEvent::Closed(reason) => {
                    link.client = None;
                    json!({ "type": "closed", "reason": reason })
                }
            };
            og_runtime::og_shared_deliver(engine, v.to_string());
            n += 1;
        }
        // What the events made the engine send goes now (the hello after open).
        carry(engine, link);
        n
    })
}

/// The engine is destroyed: its connection closes.
pub(crate) fn forget_engine(engine: u32) {
    LINKS.with(|l| l.borrow_mut().remove(&engine));
}
