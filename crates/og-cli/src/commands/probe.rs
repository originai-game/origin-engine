//! `og probe <method> [params-json] [--port P] [--out file.png] [--timeout S] [--raw]`: one
//! JSON-RPC 2.0 call to a running `og run --port P` (or `og dev`) over the
//! loopback WebSocket at `ws://127.0.0.1:<port>/probe`.
//!
//! Prints the result as JSON; `--raw` prints the whole answer frame. A
//! `capture` or `marks` result's PNG is written to `--out` (default
//! `capture.png`) and the printed result carries its path instead of the
//! base64; each frame of `capture_views` or `sweep` is written beside it as
//! `<stem>-<view>.png` (`capture-0.png`, ...) and carries its path the same
//! way; a `listen` result's WAV is written to `--out` (default `listen.wav`).
//! A refusal (`false`, `null`) is a result and exits 0; a JSON-RPC error exits 1.

use std::net::{SocketAddr, TcpStream};
use std::process::ExitCode;
use std::time::Duration;
use tungstenite::Message;

pub const DEFAULT_PORT: u16 = 47800;
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
const USAGE: &str =
    "usage: og probe <method> [params-json] [--port P] [--out file.png] [--timeout S] [--raw]";

/// One call with the default (or `OG_PROBE_TIMEOUT`) socket timeout.
pub fn call(
    port: u16,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let secs = std::env::var("OG_PROBE_TIMEOUT")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|&s| s > 0)
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    call_with_timeout(port, method, params, Duration::from_secs(secs))
}

/// One call with an explicit connect/read/write timeout: the answer frame as JSON.
pub fn call_with_timeout(
    port: u16,
    method: &str,
    params: serde_json::Value,
    timeout: Duration,
) -> Result<serde_json::Value, String> {
    let url = format!("ws://127.0.0.1:{port}/probe");
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let stream = TcpStream::connect_timeout(&addr, timeout).map_err(|e| {
        format!("cannot reach the probe at {url} ({e}); start the game with `og run --port {port}` or `og dev`")
    })?;
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    let (mut socket, _) = tungstenite::client(&url, stream).map_err(|e| {
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
                return Err(format!("{url} closed the connection without an answer"));
            }
            _ => {}
        }
    }
}

/// Decodes and writes base64 PNG/WAV payloads (`capture`, `marks`,
/// `capture_views`, `sweep`, `listen`) to files and replaces the base64
/// strings in `result` with their file paths. Returns the raw base64 PNGs
/// written (in order) so MCP can also return them as image blocks.
pub(super) fn extract_binaries(
    method: &str,
    result: &mut serde_json::Value,
    out: &str,
) -> Result<Vec<String>, String> {
    let mut images = Vec::new();
    if let Some(png) = result
        .get("png")
        .and_then(|p| p.as_str())
        .map(str::to_owned)
    {
        let bytes =
            decode_base64(&png).ok_or_else(|| "the capture's PNG is not base64".to_string())?;
        std::fs::write(out, &bytes).map_err(|e| format!("cannot write {out}: {e}"))?;
        result["png"] = serde_json::json!(out);
        images.push(png);
    }
    if matches!(method, "capture_views" | "sweep")
        && let Some(frames) = result.get_mut("frames").and_then(|f| f.as_array_mut())
    {
        let at = std::path::Path::new(out);
        let stem = at.file_stem().and_then(|s| s.to_str()).unwrap_or("capture");
        for (i, frame) in frames.iter_mut().enumerate() {
            let Some(png) = frame.get("png").and_then(|p| p.as_str()).map(str::to_owned) else {
                continue;
            };
            let bytes =
                decode_base64(&png).ok_or_else(|| format!("frame {i}'s PNG is not base64"))?;
            let view = frame
                .get("view")
                .and_then(|v| v.as_u64())
                .unwrap_or(i as u64);
            let file = at.with_file_name(format!("{stem}-{view}.png"));
            std::fs::write(&file, &bytes)
                .map_err(|e| format!("cannot write {}: {e}", file.display()))?;
            frame["png"] = serde_json::json!(file.to_string_lossy());
            images.push(png);
        }
    }
    if let Some(wav) = result
        .get("wav")
        .and_then(|w| w.as_str())
        .map(str::to_owned)
    {
        let wav_out = if out == "capture.png" {
            "listen.wav"
        } else {
            out
        };
        let bytes =
            decode_base64(&wav).ok_or_else(|| "the listen WAV is not base64".to_string())?;
        std::fs::write(wav_out, &bytes).map_err(|e| format!("cannot write {wav_out}: {e}"))?;
        result["wav"] = serde_json::json!(wav_out);
    }
    Ok(images)
}

pub(super) fn decode_base64(s: &str) -> Option<Vec<u8>> {
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
    let mut timeout_secs = std::env::var("OG_PROBE_TIMEOUT")
        .ok()
        .and_then(|t| t.parse::<u64>().ok())
        .filter(|&s| s > 0)
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
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
            "--timeout" => {
                let Some(s) = args
                    .get(i + 1)
                    .and_then(|t| t.parse::<u64>().ok())
                    .filter(|&s| s > 0)
                else {
                    eprintln!("og probe: --timeout takes positive seconds. {USAGE}");
                    return ExitCode::from(2);
                };
                timeout_secs = s;
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
                eprintln!(
                    "og probe: the params are not JSON ({e}); write them as one argument, such as '{{\"steps\": 60}}'"
                );
                return ExitCode::from(2);
            }
        },
        _ => {
            eprintln!("og probe: {USAGE}");
            return ExitCode::from(2);
        }
    };
    let answer = match call_with_timeout(port, method, params, Duration::from_secs(timeout_secs)) {
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
        if let Err(e) = extract_binaries(method, &mut result, &out) {
            eprintln!("og probe: {e}");
            return ExitCode::FAILURE;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_binaries_handles_sweep_and_listen() {
        let dir = std::env::temp_dir().join(format!("og-probe-bin-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("sweep.png");
        let b64 = "aGVsbG8="; // "hello"
        let mut sweep = serde_json::json!({
            "frames": [
                { "value": 1, "png": b64 },
                { "value": 2, "png": b64 }
            ]
        });
        let imgs = extract_binaries("sweep", &mut sweep, out.to_str().unwrap()).unwrap();
        assert_eq!(imgs.len(), 2);
        assert!(
            sweep["frames"][0]["png"]
                .as_str()
                .unwrap()
                .ends_with("sweep-0.png")
        );
        assert!(
            sweep["frames"][1]["png"]
                .as_str()
                .unwrap()
                .ends_with("sweep-1.png")
        );
        assert_eq!(std::fs::read(dir.join("sweep-0.png")).unwrap(), b"hello");

        let wav_out = dir.join("out.wav");
        let mut listen = serde_json::json!({ "wav": b64, "peak": 0.5 });
        let imgs = extract_binaries("listen", &mut listen, wav_out.to_str().unwrap()).unwrap();
        assert!(imgs.is_empty());
        assert_eq!(listen["wav"], wav_out.to_str().unwrap());
        assert_eq!(std::fs::read(&wav_out).unwrap(), b"hello");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
