//! `og probe <method> [params-json] [--port P] [--out file.png] [--raw]`: one
//! JSON-RPC 2.0 call to a running `og run --port P` (or `og dev`) over the
//! loopback WebSocket at `ws://127.0.0.1:<port>/probe`.
//!
//! Prints the result as JSON; `--raw` prints the whole answer frame. A
//! `capture` result's PNG is written to `--out` (default `capture.png`) and
//! the printed result carries its path instead of the base64. A refusal
//! (`false`, `null`) is a result and exits 0; a JSON-RPC error exits 1.

use std::process::ExitCode;
use tungstenite::Message;

pub const DEFAULT_PORT: u16 = 47800;
const USAGE: &str = "usage: og probe <method> [params-json] [--port P] [--out file.png] [--raw]";

/// One call: the answer frame as JSON.
pub fn call(
    port: u16,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let url = format!("ws://127.0.0.1:{port}/probe");
    let (mut socket, _) = tungstenite::connect(&url).map_err(|e| {
        format!("cannot reach the probe at {url} ({e}); start the game with `og run --port {port}` or `og dev`")
    })?;
    let frame =
        serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    socket
        .send(Message::text(frame.to_string()))
        .map_err(|e| format!("{url}: {e}"))?;
    loop {
        match socket.read().map_err(|e| format!("{url}: {e}"))? {
            Message::Text(t) => {
                let _ = socket.close(None);
                return serde_json::from_str(t.as_str())
                    .map_err(|e| format!("{url} answered something that is not JSON: {e}"));
            }
            Message::Close(_) => {
                return Err(format!("{url} closed the connection without an answer"))
            }
            _ => {}
        }
    }
}

fn decode_base64(s: &str) -> Option<Vec<u8>> {
    let val = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32)
    };
    let bytes: Vec<u8> = s.bytes().filter(|c| !c.is_ascii_whitespace()).collect();
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let mut n = 0u32;
        let mut pad = 0;
        for &c in chunk {
            if c == b'=' {
                pad += 1;
                n <<= 6;
            } else {
                n = (n << 6) | val(c)?;
            }
        }
        n <<= 6 * (4 - chunk.len() as u32);
        let b = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        out.extend_from_slice(&b[..3 - pad.min(2)]);
    }
    Some(out)
}

pub fn run(args: &[&str]) -> ExitCode {
    let mut port = std::env::var("OG_PROBE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let mut out = "capture.png".to_string();
    let mut raw = false;
    let mut positional = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--port" => {
                let Some(p) = args.get(i + 1).and_then(|p| p.parse().ok()) else {
                    eprintln!("og probe: --port takes a port number. {USAGE}");
                    return ExitCode::from(2);
                };
                port = p;
                i += 1;
            }
            "--out" => {
                let Some(o) = args.get(i + 1) else {
                    eprintln!("og probe: --out takes a path. {USAGE}");
                    return ExitCode::from(2);
                };
                out = o.to_string();
                i += 1;
            }
            "--raw" => raw = true,
            a => positional.push(a),
        }
        i += 1;
    }
    let (method, params) = match positional.as_slice() {
        [m] => (*m, serde_json::json!({})),
        [m, p] => match serde_json::from_str(p) {
            Ok(v) => (*m, v),
            Err(e) => {
                eprintln!("og probe: the params are not JSON ({e}); write them as one argument, such as '{{\"steps\": 60}}'");
                return ExitCode::from(2);
            }
        },
        _ => {
            eprintln!("og probe: {USAGE}");
            return ExitCode::from(2);
        }
    };
    let answer = match call(port, method, params) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("og probe: {e}");
            return ExitCode::FAILURE;
        }
    };
    if raw {
        println!("{answer}");
    } else if let Some(err) = answer.get("error") {
        eprintln!(
            "og probe: {method}: error {}: {}",
            err["code"],
            err["message"].as_str().unwrap_or("")
        );
        return ExitCode::FAILURE;
    } else {
        let mut result = answer.get("result").cloned().unwrap_or_default();
        if let Some(png) = result.get("png").and_then(|p| p.as_str()) {
            match decode_base64(png) {
                Some(bytes) => {
                    if let Err(e) = std::fs::write(&out, &bytes) {
                        eprintln!("og probe: cannot write {out}: {e}");
                        return ExitCode::FAILURE;
                    }
                    result["png"] = serde_json::json!(out);
                }
                None => {
                    eprintln!("og probe: the capture's PNG is not base64");
                    return ExitCode::FAILURE;
                }
            }
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_default()
        );
    }
    if answer.get("error").is_some() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
