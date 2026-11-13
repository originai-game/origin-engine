//! The probe's loopback WebSocket.

// napi registers the exports outside `cfg(test)` only.
#![cfg_attr(test, allow(dead_code))]

use crate::wire::EnvScope;
use napi::Env;
use napi_derive::napi;
use std::cell::RefCell;

thread_local! {
    pub(crate) static SERVERS: RefCell<std::collections::BTreeMap<u32, og_probe::server::ProbeServer>> =
        const { RefCell::new(std::collections::BTreeMap::new()) };
}

/// Serve an engine's probe on `127.0.0.1:<port>/probe` (JSON-RPC 2.0 over
/// WebSocket; port 0 picks a free one). A Result of the port.
#[napi(js_name = "og_node_probe_listen")]
pub fn og_node_probe_listen(engine: u32, port: u32) -> String {
    let addr = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port as u16));
    match og_probe::server::ProbeServer::bind(addr) {
        Ok(server) => {
            let port = server.port();
            SERVERS.with(|s| s.borrow_mut().insert(engine, server));
            format!(r#"{{"ok":true,"value":{port}}}"#)
        }
        Err(e) => og_runtime::host_impl::refuse(&format!(
            "cannot listen on 127.0.0.1:{port}: {e}; pick another --port or stop the other og run"
        )),
    }
}

/// Answer every call queued on the engine's socket, between ticks. Returns
/// how many were answered (0 without a server).
#[napi(js_name = "og_node_probe_serve")]
pub fn og_node_probe_serve(env: Env, engine: u32) -> u32 {
    let _env = EnvScope::enter(&env);
    let mut answered = 0;
    loop {
        let pending = SERVERS.with(|s| s.borrow().get(&engine).and_then(|srv| srv.poll()));
        let Some(pending) = pending else { break };
        let frame = og_runtime::answer_call(engine, &pending.call);
        pending.answer(frame);
        answered += 1;
    }
    answered
}

/// Stop serving an engine's probe.
#[napi(js_name = "og_node_probe_close")]
pub fn og_node_probe_close(engine: u32) -> bool {
    SERVERS.with(|s| s.borrow_mut().remove(&engine).is_some())
}
