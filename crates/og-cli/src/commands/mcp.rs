//! `og mcp`: the og tool as a Model Context Protocol server over stdio
//! (newline-delimited JSON-RPC 2.0, MCP's stdio transport), so an agent
//! calls it as tools instead of shell commands.
//!
//! The tools keep the contracts the agent already relies on (testing §9):
//!
//! - `probe` makes one probe call in process and answers exactly what
//!   `og probe` prints: the result as JSON, a refusal (`false`, `null`) as a
//!   result, a JSON-RPC error or an unreachable probe as an error. A
//!   `capture` result's `png` is the path it was written to (`out`, default
//!   `capture.png`), and the frame also comes back as an image block.
//! - `run`, `test`, `render`, `compare`, `api_find` and `api_show` run this
//!   same `og` with the arguments the command line takes (`--json` where the
//!   command has it) and answer its stdout as is, its exit status as
//!   `isError`, and stderr's tail when it failed. `args` passes any other
//!   flag through.
//! - `run` with a `port` starts the game in the background and answers the
//!   probe's `hello` once it is up; the games it started stop when the
//!   server's input closes (or with `probe commands {"script":"0:quit"}`).

use serde_json::{Map, Value, json};
use std::io::{BufRead, Write};
use std::process::{Child, Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

const USAGE: &str = "usage: og mcp";
/// The MCP revisions this server speaks, newest first.
const PROTOCOLS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];
/// How long `run` with a port waits for the probe to answer.
const STARTUP: Duration = Duration::from_secs(180);
/// The stderr a failed command's answer carries, from its end.
const STDERR_TAIL: usize = 4000;

pub fn run(args: &[&str]) -> ExitCode {
    if !args.is_empty() {
        eprintln!("og mcp: takes no arguments. {USAGE}");
        return ExitCode::from(2);
    }
    let mut server = Server::default();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        if let Some(answer) = server.handle(&line) {
            let _ = writeln!(stdout, "{answer}");
            let _ = stdout.flush();
        }
    }
    server.stop();
    ExitCode::SUCCESS
}

#[derive(Default)]
struct Server {
    /// Games `run` started in the background.
    games: Vec<Child>,
}

impl Server {
    /// One message in; the answer frame, none for a notification.
    fn handle(&mut self, line: &str) -> Option<Value> {
        let msg: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => return Some(error(Value::Null, -32700, &format!("parse error: {e}"))),
        };
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));
        let id = id?;
        Some(match method {
            "initialize" => {
                let asked = params["protocolVersion"].as_str().unwrap_or("");
                let version = PROTOCOLS
                    .iter()
                    .find(|p| **p == asked)
                    .unwrap_or(&PROTOCOLS[0]);
                result(
                    id,
                    json!({
                        "protocolVersion": version,
                        "capabilities": { "tools": {} },
                        "serverInfo": { "name": "og", "version": env!("CARGO_PKG_VERSION") },
                        "instructions": "The Origin Engine tool. `api_find` and `api_show` answer the API and its guides (`api_show start`, `api_show testing`); `run`, `test` and `render` drive the game; `probe` calls a running game's probe (`api_show probe` lists the members).",
                    }),
                )
            }
            "ping" => result(id, json!({})),
            "tools/list" => result(id, json!({ "tools": tools() })),
            "tools/call" => {
                let name = params["name"].as_str().unwrap_or("");
                let empty = Map::new();
                let a = params["arguments"].as_object().unwrap_or(&empty);
                match self.call(name, a) {
                    Ok(r) => result(id, r),
                    Err(message) => error(id, -32602, &message),
                }
            }
            _ => error(id, -32601, &format!("method not found: {method}")),
        })
    }

    /// One tool call: its MCP result, or why the call is malformed.
    fn call(&mut self, name: &str, a: &Map<String, Value>) -> Result<Value, String> {
        let mut args: Vec<String> = Vec::new();
        let push = |args: &mut Vec<String>, flag: &str, key: &str| {
            if let Some(v) = a.get(key).filter(|v| !v.is_null()) {
                args.push(flag.into());
                args.push(match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                });
            }
        };
        let flag = |args: &mut Vec<String>, flag: &str, key: &str| {
            if a.get(key).and_then(Value::as_bool) == Some(true) {
                args.push(flag.into());
            }
        };
        let project = |args: &mut Vec<String>| {
            if let Some(p) = a.get("project").and_then(Value::as_str) {
                args.push(p.into());
            }
        };
        match name {
            "probe" => return Ok(probe(a)),
            "run" => {
                args.push("run".into());
                project(&mut args);
                args.push("--headless".into());
                push(&mut args, "--commands", "commands");
                push(&mut args, "--ticks", "ticks");
                push(&mut args, "--seed", "seed");
                push(&mut args, "--size", "size");
                push(&mut args, "--out", "out");
                push(&mut args, "--port", "port");
                flag(&mut args, "--no-check", "noCheck");
            }
            "test" => {
                args.push("test".into());
                project(&mut args);
                push(&mut args, "--filter", "filter");
                push(&mut args, "--jobs", "jobs");
                flag(&mut args, "--no-check", "noCheck");
            }
            "render" => {
                args.push("render".into());
                project(&mut args);
                push(&mut args, "--commands", "commands");
                push(&mut args, "--timeline", "timeline");
                push(&mut args, "--fps", "fps");
                push(&mut args, "--from", "from");
                push(&mut args, "--to", "to");
                push(&mut args, "--size", "size");
                push(&mut args, "--out", "out");
                push(&mut args, "--contact-sheet", "contactSheet");
                flag(&mut args, "--keyframes", "keyframes");
                flag(&mut args, "--no-check", "noCheck");
            }
            "compare" => {
                args.push("compare".into());
                for key in ["reference", "render"] {
                    let path = a
                        .get(key)
                        .and_then(Value::as_str)
                        .ok_or_else(|| format!("compare: {key} is a PNG's path"))?;
                    args.push(path.into());
                }
                push(&mut args, "--mask", "mask");
                flag(&mut args, "--align", "align");
                push(&mut args, "--sheet", "sheet");
                push(&mut args, "--flip", "flip");
                push(&mut args, "--regions", "regions");
            }
            "api_find" => {
                let words = a
                    .get("words")
                    .and_then(Value::as_str)
                    .ok_or("api_find: words is the text to look for")?;
                args.extend(["api".into(), "find".into(), words.into()]);
                push(&mut args, "--kind", "kind");
                push(&mut args, "--limit", "limit");
            }
            "api_show" => {
                let what = a
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or("api_show: name is the entry, module or guide")?;
                args.extend(["api".into(), "show".into(), what.into()]);
                flag(&mut args, "--all", "all");
                flag(&mut args, "--guide", "guide");
            }
            _ => return Err(format!("unknown tool `{name}`")),
        }
        if let Some(extra) = a.get("args").and_then(Value::as_array) {
            args.extend(extra.iter().filter_map(Value::as_str).map(str::to_owned));
        }
        if name != "api_show" {
            args.push("--json".into());
        }
        if name == "run"
            && let Some(port) = a.get("port").and_then(Value::as_u64)
        {
            return Ok(self.start(&args, port as u16));
        }
        Ok(og(&args))
    }

    /// `run` with a port: the game in the background, answered by its
    /// probe's `hello` once it is up.
    fn start(&mut self, args: &[String], port: u16) -> Value {
        let child = Command::new(exe())
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();
        let mut child = match child {
            Ok(c) => c,
            Err(e) => return failed(format!("og run: cannot start: {e}")),
        };
        let deadline = Instant::now() + STARTUP;
        loop {
            if let Ok(Some(status)) = child.try_wait() {
                let mut err = String::new();
                if let Some(mut s) = child.stderr.take() {
                    let _ = std::io::Read::read_to_string(&mut s, &mut err);
                }
                return failed(format!(
                    "og run exited ({status}) before its probe answered\n{}",
                    tail(&err)
                ));
            }
            if let Ok(answer) = super::probe::call(port, "hello", json!({}))
                && let Some(hello) = answer.get("result")
            {
                let hello = hello.clone();
                self.games.push(child);
                return json!({
                    "content": [{ "type": "text", "text": hello.to_string() }],
                    "structuredContent": hello,
                    "isError": false,
                });
            }
            if Instant::now() > deadline {
                let _ = child.kill();
                return failed(format!(
                    "og run: no probe on port {port} after {}s",
                    STARTUP.as_secs()
                ));
            }
            std::thread::sleep(Duration::from_millis(250));
        }
    }

    fn stop(&mut self) {
        for mut c in self.games.drain(..) {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

/// The probe tool: what `og probe <method> [params] [--port P] [--out F]` prints.
fn probe(a: &Map<String, Value>) -> Value {
    let Some(method) = a.get("method").and_then(Value::as_str) else {
        return failed(
            "probe: method is the member to call (`og api show probe` lists them)".into(),
        );
    };
    let params = a.get("params").cloned().unwrap_or(json!({}));
    let port = a
        .get("port")
        .and_then(Value::as_u64)
        .map(|p| p as u16)
        .or_else(|| std::env::var("OG_PROBE_PORT").ok()?.parse().ok())
        .unwrap_or(super::probe::DEFAULT_PORT);
    let answer = match super::probe::call(port, method, params) {
        Ok(a) => a,
        Err(e) => return failed(format!("og probe: {e}")),
    };
    if let Some(err) = answer.get("error") {
        return failed(format!(
            "og probe: {method}: error {}: {}",
            err["code"],
            err["message"].as_str().unwrap_or("")
        ));
    }
    let mut result = answer.get("result").cloned().unwrap_or_default();
    let mut content = Vec::new();
    if let Some(png) = result.get("png").and_then(Value::as_str).map(str::to_owned) {
        let out = a
            .get("out")
            .and_then(Value::as_str)
            .unwrap_or("capture.png");
        let Some(bytes) = super::probe::decode_base64(&png) else {
            return failed("og probe: the capture's PNG is not base64".into());
        };
        if let Err(e) = std::fs::write(out, &bytes) {
            return failed(format!("og probe: cannot write {out}: {e}"));
        }
        result["png"] = json!(out);
        content.push(json!({ "type": "image", "data": png, "mimeType": "image/png" }));
    }
    let text = serde_json::to_string_pretty(&result).unwrap_or_default();
    content.insert(0, json!({ "type": "text", "text": text }));
    let mut r = json!({ "content": content, "isError": false });
    if result.is_object() {
        r["structuredContent"] = result;
    }
    r
}

/// This `og` with `args`: its stdout as the answer, its exit status as
/// `isError` (with stderr's tail).
fn og(args: &[String]) -> Value {
    let out = match Command::new(exe()).args(args).stdin(Stdio::null()).output() {
        Ok(o) => o,
        Err(e) => return failed(format!("og {}: cannot start: {e}", args[0])),
    };
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut content = vec![json!({ "type": "text", "text": stdout })];
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        content.push(json!({ "type": "text", "text": tail(&stderr) }));
    }
    json!({ "content": content, "isError": !out.status.success() })
}

fn exe() -> std::path::PathBuf {
    std::env::current_exe().unwrap_or_else(|_| "og".into())
}

fn tail(s: &str) -> &str {
    let mut at = s.len().saturating_sub(STDERR_TAIL);
    while !s.is_char_boundary(at) {
        at += 1;
    }
    &s[at..]
}

fn failed(text: String) -> Value {
    json!({ "content": [{ "type": "text", "text": text }], "isError": true })
}

fn result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// The tools and their input schemas.
fn tools() -> Value {
    let s = |d: &str| json!({ "type": "string", "description": d });
    let n = |d: &str| json!({ "type": "integer", "description": d });
    let b = |d: &str| json!({ "type": "boolean", "description": d });
    let args = json!({ "type": "array", "items": { "type": "string" }, "description": "Further flags, as on the command line." });
    let project = s("The project directory (default: the current one).");
    let tool = |name: &str, description: &str, properties: Value, required: &[&str]| {
        json!({
            "name": name,
            "description": description,
            "inputSchema": { "type": "object", "properties": properties, "required": required },
        })
    };
    json!([
        tool(
            "probe",
            "One call to the probe of a running game (`og run --port P`, `og dev`), as `og probe`: the member's result as JSON (a refusal, false or null, is a result); a JSON-RPC error or an unreachable probe is an error. A capture's png is written to `out` and also returned as an image. `api_show probe` lists the members: describe, visible, dump (world, entity, ui, drawn), capture, captureViews (`capture_views`), marks, pick, run, commands...",
            json!({
                "method": s("The member, as the wire names it: `describe`, `visible`, `capture_views`, `dump`..."),
                "params": { "type": "object", "description": "Its params, such as {\"id\": \"player\"}." },
                "port": n("The probe's port (default OG_PROBE_PORT, else 47800)."),
                "out": s("Where a capture's PNG is written (default capture.png)."),
            }),
            &["method"],
        ),
        tool(
            "run",
            "`og run --headless --json`: cook, check, bundle and run the game with a command script; its JSON on stdout. With `port`, the game starts in the background and the answer is its probe's hello: call `probe` on that port, and stop it with probe commands {\"script\": \"0:quit\"}.",
            json!({
                "project": project, "commands": s("The command script, `t:action[:args];...` (testing §7)."),
                "ticks": n("Ticks to run."), "seed": n("The seed."), "size": s("WxH."),
                "out": s("Where screenshots and dumps land (default og-out/)."),
                "port": n("Serve the probe on this port and keep running."),
                "noCheck": b("Skip the type check."), "args": args,
            }),
            &[],
        ),
        tool(
            "test",
            "`og test --json`: the bun tests and each tests/*.og.ts playtest; JSON Lines on stdout (check, playtest per file, test last; testing §9). An error when the test event's ok is false.",
            json!({
                "project": project, "filter": s("Suite files by a project-relative path substring."),
                "jobs": n("Playtest files at once."), "noCheck": b("Skip the type check."), "args": args,
            }),
            &[],
        ),
        tool(
            "render",
            "`og render --json`: the game drawn frame by frame headless into PNG frames or a video, with frames.json.",
            json!({
                "project": project, "commands": s("The command script."), "timeline": s("A timeline to play."),
                "fps": n("Frames a second."), "from": { "type": "number" }, "to": { "type": "number" },
                "size": s("WxH."), "out": s("A directory, out.mp4 or out.webm."),
                "contactSheet": n("Columns of a contact sheet."), "keyframes": b("Also the key frames."),
                "noCheck": b("Skip the type check."), "args": args,
            }),
            &[],
        ),
        tool(
            "compare",
            "`og compare --json`: a captured frame against a reference: silhouette IoU and ΔE00 per mask region, and FLIP.",
            json!({
                "reference": s("The reference PNG."), "render": s("The captured PNG."),
                "mask": s("A mask PNG."), "align": b("Align first."), "sheet": s("Write a sheet PNG."),
                "flip": s("Write the FLIP map PNG."), "regions": n("Worst regions to list."), "args": args,
            }),
            &["reference", "render"],
        ),
        tool(
            "api_find",
            "`og api find --json`: the API and guide entries for some words, ranked.",
            json!({
                "words": s("A name, a purpose, a typo."),
                "kind": s("fn, record, type, member, const, value, column or guide."),
                "limit": n("How many."), "args": args,
            }),
            &["words"],
        ),
        tool(
            "api_show",
            "`og api show`: one API entry, module or guide in full (`og.physics.castRay`, `probe`, `testing`, `start`, `saves §2`).",
            json!({
                "name": s("The entry, module or guide."), "all": b("Every member of a module."),
                "guide": b("The guide, when a module has the same name."), "args": args,
            }),
            &["name"],
        ),
    ])
}
