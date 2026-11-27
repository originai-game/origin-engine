//! `og mcp`: the og tool as a Model Context Protocol server over stdio
//! (newline-delimited JSON-RPC 2.0, MCP's stdio transport), so an agent
//! calls it as tools instead of shell commands.
//!
//! The tools keep the contracts the agent already relies on (testing §9):
//!
//! - `probe` makes one probe call in process and answers exactly what
//!   `og probe` prints: the result as JSON, a refusal (`false`, `null`) as a
//!   result, a JSON-RPC error or an unreachable probe as an error. Captured
//!   PNGs (`capture`, `marks`, `capture_views`, `sweep`) are written to files
//!   and also returned as MCP image blocks; `listen`'s WAV is written to a file.
//! - The CLI tools (`run`, `test`, `render`, `compare`, `check`, `cook`,
//!   `build`, `create`, `explore`, `replay`, `ui`, `style`, `model`, `assets`,
//!   `prefab`, `doctor`, `api_find`, `api_show`) run this same `og` with the
//!   arguments the command line takes (`--json` where supported) and answer
//!   its stdout as is, its exit status as `isError`, and stderr's tail when
//!   it failed. `args` passes any other flag through.
//! - `run` with a `port` starts the game in the background, continuously
//!   drains its stderr into a ring buffer (`run_logs` reads it) so the pipe
//!   never blocks, and answers the probe's `hello` once it is up; the games
//!   it started stop when the server's input closes (or with `probe commands
//!   {"script":"0:quit"}`).

use serde_json::{Map, Value, json};
use std::collections::VecDeque;
use std::io::{BufRead, Read, Write};
use std::process::{Child, Command, ExitCode, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const USAGE: &str = "usage: og mcp";
/// The MCP revisions this server speaks, newest first.
const PROTOCOLS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];
/// How long `run` with a port waits for the probe to answer.
const STARTUP: Duration = Duration::from_secs(180);
/// The stderr a failed command's answer carries, from its end.
const STDERR_TAIL: usize = 4000;
/// Maximum stderr bytes kept per background game started by `run`.
const BG_STDERR_CAP: usize = 64 * 1024;

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

struct BackgroundGame {
    port: u16,
    child: Child,
    stderr: Arc<Mutex<VecDeque<u8>>>,
    exited: Option<String>,
}

impl BackgroundGame {
    fn stderr_string(&self) -> String {
        let guard = self.stderr.lock().unwrap_or_else(|e| e.into_inner());
        let bytes: Vec<u8> = guard.iter().copied().collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    fn clear_stderr(&self) {
        let mut guard = self.stderr.lock().unwrap_or_else(|e| e.into_inner());
        guard.clear();
    }

    fn poll_exit(&mut self) {
        if self.exited.is_none()
            && let Ok(Some(status)) = self.child.try_wait()
        {
            self.exited = Some(status.to_string());
        }
    }
}

#[derive(Default)]
struct Server {
    /// Games `run` started in the background.
    games: Vec<BackgroundGame>,
    /// Cached API wiki index, loaded on the first `api_*` call and again
    /// when the wiki changed.
    wiki: Option<crate::api_lookup::WikiCache>,
}

impl Server {
    fn wiki(&mut self) -> Result<&crate::api_lookup::WikiCache, String> {
        if !self.wiki.as_ref().is_some_and(|w| w.is_current()) {
            self.wiki = None;
            self.wiki = Some(crate::api_lookup::WikiCache::open()?);
        }
        Ok(self.wiki.as_ref().expect("initialized"))
    }

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
                        "instructions": "The Origin Engine tool. `api_list`, `api_find` and `api_show` answer the API and its guides (`api_show start`, `api_show testing`); `create`, `check`, `cook`, `build`, `run`, `test`, `explore`, `replay`, `render`, `compare`, `frame`, `cost`, `ui`, `style`, `model`, `vfx`, `sfx`, `sprite`, `materials`, `i18n`, `flow`, `fmt`, `font`, `assets`, `prefab` and `doctor` operate the project; `probe` and `run_logs` inspect a running game (`api_show probe` lists the members).",
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
        let mut append_json = true;
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
            "run_logs" => return Ok(self.run_logs(a)),
            "api_find" => {
                let words = a
                    .get("words")
                    .and_then(Value::as_str)
                    .ok_or("api_find: words is the text to look for")?;
                let kind = a.get("kind").and_then(Value::as_str);
                if let Some(k) = kind
                    && !crate::api_lookup::KINDS.contains(&k)
                {
                    return Err(format!(
                        "api_find: kind must be one of {}",
                        crate::api_lookup::KINDS.join(", ")
                    ));
                }
                let limit = a
                    .get("limit")
                    .and_then(Value::as_u64)
                    .map_or(12, |n| n as usize);
                return Ok(match self.wiki() {
                    Ok(wiki) => {
                        let (body, found) = wiki.find_json(words, kind, limit);
                        json!({
                            "content": [{ "type": "text", "text": body.to_string() }],
                            "structuredContent": body,
                            "isError": !found,
                        })
                    }
                    Err(e) => failed(format!("og api find: {e}")),
                });
            }
            "api_show" => {
                let what = a
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or("api_show: name is the entry, module or guide")?;
                let every = a.get("all").and_then(Value::as_bool).unwrap_or(false);
                let guide = a.get("guide").and_then(Value::as_bool).unwrap_or(false);
                let as_json = a.get("json").and_then(Value::as_bool).unwrap_or(false);
                return Ok(match self.wiki() {
                    Ok(wiki) => match wiki.show_output(what, guide, every, as_json) {
                        Ok(text) => {
                            let mut out = json!({
                                "content": [{ "type": "text", "text": text }],
                                "isError": false,
                            });
                            if as_json && let Ok(v) = serde_json::from_str::<Value>(&text) {
                                out["structuredContent"] = v;
                            }
                            out
                        }
                        Err(e) => failed(e),
                    },
                    Err(e) => failed(format!("og api show: {e}")),
                });
            }
            "api_list" => {
                let kind = a.get("kind").and_then(Value::as_str);
                if let Some(k) = kind
                    && !matches!(k, "module" | "guide" | "file")
                {
                    return Err("api_list: kind must be `module`, `guide` or `file`".into());
                }
                return Ok(match self.wiki() {
                    Ok(wiki) => {
                        let body = wiki.list_json(kind);
                        json!({
                            "content": [{ "type": "text", "text": body.to_string() }],
                            "structuredContent": body,
                            "isError": false,
                        })
                    }
                    Err(e) => failed(format!("og api list: {e}")),
                });
            }
            "run" => {
                args.push("run".into());
                project(&mut args);
                args.push("--headless".into());
                push(&mut args, "--commands", "commands");
                push(&mut args, "--commands-file", "commandsFile");
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
                push(&mut args, "--skip", "skip");
                push(&mut args, "--affected", "affected");
                push(&mut args, "--jobs", "jobs");
                flag(&mut args, "--no-check", "noCheck");
                flag(&mut args, "--require-checked", "requireChecked");
            }
            "check" => {
                args.push("check".into());
                project(&mut args);
                push(&mut args, "--tool", "tool");
                flag(&mut args, "--require-checked", "requireChecked");
            }
            "cook" => {
                args.push("cook".into());
                project(&mut args);
                push(&mut args, "--threads", "threads");
                flag(&mut args, "--report", "report");
            }
            "build" => {
                args.push("build".into());
                project(&mut args);
                flag(&mut args, "--publish", "publish");
                push(&mut args, "--out", "out");
                flag(&mut args, "--no-check", "noCheck");
            }
            "create" => {
                args.push("create".into());
                if let Some(dir) = a.get("dir").and_then(Value::as_str) {
                    args.push(dir.into());
                }
                args.push("--no-prompt".into());
                push(&mut args, "--name", "name");
                push(&mut args, "--id", "id");
                push(&mut args, "--premise", "premise");
                push(&mut args, "--form", "form");
                push(&mut args, "--genre", "genre");
                push(&mut args, "--view", "view");
                push(&mut args, "--look", "look");
                push(&mut args, "--variant", "variant");
                push(&mut args, "--ui", "ui");
                push(&mut args, "--platform", "platform");
                push(&mut args, "--input", "input");
                push(&mut args, "--players", "players");
                push(&mut args, "--lang", "lang");
                push(&mut args, "--music", "music");
                flag(&mut args, "--force", "force");
                append_json = false;
            }
            "explore" => {
                args.push("explore".into());
                project(&mut args);
                push(&mut args, "--seconds", "seconds");
                push(&mut args, "--ticks", "ticks");
                push(&mut args, "--seed", "seed");
                push(&mut args, "--jobs", "jobs");
                push(&mut args, "--out", "out");
                push(&mut args, "--web", "web");
                flag(&mut args, "--first", "first");
                flag(&mut args, "--no-shrink", "noShrink");
            }
            "replay" => {
                let sub = a
                    .get("subcommand")
                    .and_then(Value::as_str)
                    .ok_or("replay: subcommand must be `record` or `compare`")?;
                args.extend(["replay".into(), sub.into()]);
                if sub == "compare" {
                    let trace = a
                        .get("trace")
                        .and_then(Value::as_str)
                        .ok_or("replay compare: trace is the trace.json path")?;
                    args.push(trace.into());
                }
                project(&mut args);
                push(&mut args, "--commands", "commands");
                push(&mut args, "--commands-file", "commandsFile");
                push(&mut args, "--ticks", "ticks");
                push(&mut args, "--seed", "seed");
                push(&mut args, "--out", "out");
            }
            "ui" => {
                let sub = a
                    .get("subcommand")
                    .and_then(Value::as_str)
                    .ok_or("ui: subcommand must be `check`, `preview`, `literals`, `list`, `eject` or `add`")?;
                args.extend(["ui".into(), sub.into()]);
                if let Some(target) = a.get("target").and_then(Value::as_str) {
                    args.push(target.into());
                }
                project(&mut args);
                push(&mut args, "--out", "out");
                flag(&mut args, "--force", "force");
                append_json = matches!(sub, "check" | "literals" | "list");
            }
            "style" => {
                let sub = a
                    .get("subcommand")
                    .and_then(Value::as_str)
                    .ok_or("style: subcommand must be `list`, `check-project`, `init`, `validate` or `add`")?;
                args.extend(["style".into(), sub.into()]);
                if let Some(target) = a.get("target").and_then(Value::as_str) {
                    args.push(target.into());
                }
                if sub != "validate" {
                    project(&mut args);
                }
                push(&mut args, "--from", "from");
                flag(&mut args, "--no-frames", "noFrames");
                append_json = matches!(sub, "list" | "check-project");
            }
            "model" => {
                let sub = a
                    .get("subcommand")
                    .and_then(Value::as_str)
                    .ok_or("model: subcommand is required (`og model help`)")?;
                args.extend(["model".into(), sub.into()]);
                if let Some(input) = a.get("input").and_then(Value::as_str) {
                    args.push(input.into());
                }
                push(&mut args, "--out", "out");
                append_json = false;
            }
            "assets" => {
                args.push("assets".into());
                if let Some(sub) = a.get("subcommand").and_then(Value::as_str) {
                    args.push(sub.into());
                }
                push(&mut args, "--project", "project");
                push(&mut args, "--kind", "kind");
                push(&mut args, "--group", "group");
                flag(&mut args, "--unused", "unused");
                flag(&mut args, "--refs", "refs");
                flag(&mut args, "--by", "by");
                flag(&mut args, "--dupes", "dupes");
                if let Some(query) = a.get("query").and_then(Value::as_str) {
                    args.push(query.into());
                }
            }
            "prefab" => {
                let sub = a
                    .get("subcommand")
                    .and_then(Value::as_str)
                    .ok_or("prefab: subcommand must be `check`, `show` or `extract`")?;
                args.extend(["prefab".into(), sub.into()]);
                if let Some(target) = a.get("target").and_then(Value::as_str) {
                    args.push(target.into());
                }
                if sub == "check" {
                    project(&mut args);
                } else {
                    push(&mut args, "--project", "project");
                }
                append_json = false;
            }
            "doctor" => {
                args.push("doctor".into());
                push(&mut args, "--project", "project");
                flag(&mut args, "--require-checked", "requireChecked");
            }
            "render" => {
                args.push("render".into());
                project(&mut args);
                push(&mut args, "--commands", "commands");
                push(&mut args, "--commands-file", "commandsFile");
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
            "frame" => {
                let png = a
                    .get("png")
                    .and_then(Value::as_str)
                    .ok_or("frame: png is the captured PNG path")?;
                args.extend(["frame".into(), "measure".into(), png.into()]);
                push(&mut args, "--region", "region");
                push(&mut args, "--against", "against");
                push(&mut args, "--units", "units");
                push(&mut args, "--metric", "metric");
                push(&mut args, "--rows", "rows");
                flag(&mut args, "--require-checked", "requireChecked");
            }
            "cost" => {
                args.push("cost".into());
                project(&mut args);
                push(&mut args, "--off", "off");
                push(&mut args, "--on", "on");
                push(&mut args, "--runs", "runs");
                push(&mut args, "--ticks", "ticks");
            }
            "vfx" => {
                let sub = a
                    .get("subcommand")
                    .and_then(Value::as_str)
                    .ok_or("vfx: subcommand must be `add`, `import` or `preview`")?;
                args.extend(["vfx".into(), sub.into()]);
                if let Some(target) = a.get("target").and_then(Value::as_str) {
                    args.push(target.into());
                }
                if sub == "add" {
                    project(&mut args);
                    flag(&mut args, "--any-engine", "anyEngine");
                } else if sub == "preview" {
                    push(&mut args, "--project", "project");
                    push(&mut args, "--out", "out");
                    push(&mut args, "--size", "size");
                    push(&mut args, "--quality", "quality");
                    push(&mut args, "--look", "look");
                    push(&mut args, "--scene", "scene");
                    push(&mut args, "--position", "position");
                    push(&mut args, "--camera", "camera");
                    flag(&mut args, "--project-look", "projectLook");
                    flag(&mut args, "--no-cost", "noCost");
                    flag(&mut args, "--together", "together");
                } else {
                    if let Some(out) = a.get("out").and_then(Value::as_str) {
                        args.push(out.into());
                    }
                    push(&mut args, "--root", "root");
                    flag(&mut args, "--hit", "hit");
                }
                append_json = sub == "preview";
            }
            "script" => {
                let file = a
                    .get("file")
                    .and_then(Value::as_str)
                    .ok_or("script: file is a TS module path relative to root")?;
                args.extend(["script".into(), file.into()]);
                push(&mut args, "--root", "root");
                push(&mut args, "--params", "params");
                push(&mut args, "--seed", "seed");
                push(&mut args, "--out", "out");
                if let Some(inputs) = a.get("inputs").and_then(Value::as_object) {
                    for (name, path) in inputs {
                        let path = path.as_str().ok_or("script.inputs values are paths")?;
                        args.extend(["--input".into(), format!("{name}={path}")]);
                    }
                }
            }
            "sfx" => {
                args.push("sfx".into());
                if let Some(preset) = a.get("preset").and_then(Value::as_str) {
                    args.push(preset.into());
                }
                push(&mut args, "--seed", "seed");
                push(&mut args, "--pitch", "pitch");
                push(&mut args, "--length", "length");
                push(&mut args, "--volume", "volume");
                push(&mut args, "--out", "out");
                append_json = false;
            }
            "sprite" => {
                let input = a
                    .get("input")
                    .and_then(Value::as_str)
                    .ok_or("sprite: input is the animated image or sheet.ogt path")?;
                args.extend(["sprite".into(), "inspect".into(), input.into()]);
                push(&mut args, "--key", "key");
                push(&mut args, "--tolerance", "tolerance");
                flag(&mut args, "--holes", "holes");
                push(&mut args, "--out", "out");
            }
            "materials" => {
                let sub = a
                    .get("subcommand")
                    .and_then(Value::as_str)
                    .ok_or("materials: subcommand must be `brief` or `make`")?;
                let library = a
                    .get("library")
                    .and_then(Value::as_str)
                    .ok_or("materials: library is the library .ogt path")?;
                args.extend(["materials".into(), sub.into(), library.into()]);
            }
            "i18n" => {
                args.extend(["i18n".into(), "extract".into()]);
                project(&mut args);
                flag(&mut args, "--write", "write");
                flag(&mut args, "--check", "check");
            }
            "flow" => {
                let sub = a
                    .get("subcommand")
                    .and_then(Value::as_str)
                    .ok_or("flow: subcommand must be `list` or `routes`")?;
                args.extend(["flow".into(), sub.into()]);
                project(&mut args);
            }
            "fmt" => {
                args.push("fmt".into());
                if let Some(path) = a.get("path").and_then(Value::as_str) {
                    args.push(path.into());
                }
                flag(&mut args, "--check", "check");
            }
            "font" => {
                let sub = a
                    .get("subcommand")
                    .and_then(Value::as_str)
                    .ok_or("font: subcommand must be `list` or `add`")?;
                args.extend(["font".into(), sub.into()]);
                if let Some(target) = a.get("target").and_then(Value::as_str) {
                    args.push(target.into());
                }
                project(&mut args);
                append_json = false;
            }
            _ => return Err(format!("unknown tool `{name}`")),
        }
        if let Some(extra) = a.get("args").and_then(Value::as_array) {
            args.extend(extra.iter().filter_map(Value::as_str).map(str::to_owned));
        }
        if append_json && !args.iter().any(|x| x == "--json") {
            args.push("--json".into());
        }
        if name == "run"
            && let Some(port) = a.get("port").and_then(Value::as_u64)
        {
            return Ok(self.start(&args, port as u16));
        }
        Ok(og(&args))
    }

    /// `run_logs`: read the stderr ring buffer and lifecycle status of
    /// background game(s) started by `run` with `port`.
    fn run_logs(&mut self, a: &Map<String, Value>) -> Value {
        let port = a.get("port").and_then(Value::as_u64).map(|p| p as u16);
        let clear = a.get("clear").and_then(Value::as_bool).unwrap_or(false);
        let mut entries = Vec::new();
        for g in &mut self.games {
            g.poll_exit();
            if port.is_some_and(|p| p != g.port) {
                continue;
            }
            let stderr = g.stderr_string();
            if clear {
                g.clear_stderr();
            }
            entries.push(json!({
                "port": g.port,
                "running": g.exited.is_none(),
                "exit": g.exited,
                "stderr": stderr,
            }));
        }
        let body = json!({ "games": entries });
        json!({
            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&body).unwrap_or_default() }],
            "structuredContent": body,
            "isError": false,
        })
    }

    /// `run` with a port: the game in the background, continuously draining
    /// stderr into a bounded buffer, answered by its probe's `hello` once up.
    fn start(&mut self, args: &[String], port: u16) -> Value {
        // Reap any previously exited games on the same port.
        self.games.retain_mut(|g| {
            g.poll_exit();
            !(g.port == port && g.exited.is_some())
        });
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
        let stderr_buf = Arc::new(Mutex::new(VecDeque::with_capacity(4096)));
        if let Some(mut pipe) = child.stderr.take() {
            let sink = Arc::clone(&stderr_buf);
            let _ = std::thread::Builder::new()
                .name(format!("og-mcp-stderr-{port}"))
                .spawn(move || {
                    let mut chunk = [0u8; 4096];
                    while let Ok(n) = pipe.read(&mut chunk) {
                        if n == 0 {
                            break;
                        }
                        let mut guard = sink.lock().unwrap_or_else(|e| e.into_inner());
                        for &b in &chunk[..n] {
                            if guard.len() >= BG_STDERR_CAP {
                                guard.pop_front();
                            }
                            guard.push_back(b);
                        }
                    }
                });
        }
        let deadline = Instant::now() + STARTUP;
        loop {
            if let Ok(Some(status)) = child.try_wait() {
                // Give the stderr reader thread a brief moment to finish draining.
                std::thread::sleep(Duration::from_millis(20));
                let err = {
                    let guard = stderr_buf.lock().unwrap_or_else(|e| e.into_inner());
                    let bytes: Vec<u8> = guard.iter().copied().collect();
                    String::from_utf8_lossy(&bytes).into_owned()
                };
                return failed(format!(
                    "og run exited ({status}) before its probe answered\n{}",
                    tail(&err)
                ));
            }
            if let Ok(answer) =
                super::probe::call_with_timeout(port, "hello", json!({}), Duration::from_secs(2))
                && let Some(hello) = answer.get("result")
            {
                let hello = hello.clone();
                self.games.push(BackgroundGame {
                    port,
                    child,
                    stderr: stderr_buf,
                    exited: None,
                });
                return json!({
                    "content": [{ "type": "text", "text": hello.to_string() }],
                    "structuredContent": hello,
                    "isError": false,
                });
            }
            if Instant::now() > deadline {
                let _ = child.kill();
                let _ = child.wait();
                return failed(format!(
                    "og run: no probe on port {port} after {}s",
                    STARTUP.as_secs()
                ));
            }
            std::thread::sleep(Duration::from_millis(250));
        }
    }

    fn stop(&mut self) {
        for mut g in self.games.drain(..) {
            let _ = g.child.kill();
            let _ = g.child.wait();
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
    let timeout = a
        .get("timeout")
        .and_then(Value::as_u64)
        .filter(|&s| s > 0)
        .map(Duration::from_secs);
    let answer = match timeout {
        Some(t) => super::probe::call_with_timeout(port, method, params, t),
        None => super::probe::call(port, method, params),
    };
    let answer = match answer {
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
    let out = a
        .get("out")
        .and_then(Value::as_str)
        .unwrap_or("capture.png");
    let images = match super::probe::extract_binaries(method, &mut result, out) {
        Ok(imgs) => imgs,
        Err(e) => return failed(format!("og probe: {e}")),
    };
    let text = serde_json::to_string_pretty(&result).unwrap_or_default();
    let mut content = vec![json!({ "type": "text", "text": text })];
    for png in images {
        content.push(json!({ "type": "image", "data": png, "mimeType": "image/png" }));
    }
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
            "One call to the probe of a running game (`og run --port P`, `og dev`), as `og probe`: the member's result as JSON (a refusal, false or null, is a result); a JSON-RPC error or an unreachable probe is an error. Captured PNGs (`capture`, `marks`, `capture_views`, `sweep`) are written to `out` (or `<stem>-<view>.png`) and also returned as image blocks; `listen`'s WAV is written to `out` (default `listen.wav`). `api_show probe` lists the members: describe, visible, dump, capture, capture_views, marks, sweep, listen, pick, pick_ui, set, get, logs, run, commands...",
            json!({
                "method": s("The member, as the wire names it: `describe`, `visible`, `capture_views`, `set`, `get`, `logs`, `dump`..."),
                "params": { "type": "object", "description": "Its params, such as {\"id\": \"player\"}." },
                "port": n("The probe's port (default OG_PROBE_PORT, else 47800)."),
                "out": s("Where a capture PNG (default capture.png) or listen WAV (default listen.wav) is written."),
                "timeout": n("Socket timeout in seconds (default OG_PROBE_TIMEOUT, else 30)."),
            }),
            &["method"],
        ),
        tool(
            "run_logs",
            "Read the stderr output and running/exited status of background game(s) started by `run` with `port`.",
            json!({
                "port": n("Filter to one background game's probe port; omit for all."),
                "clear": b("Clear the buffered stderr after reading."),
            }),
            &[],
        ),
        tool(
            "run",
            "`og run --headless --json`: cook, check, bundle and run the game with a command script; its JSON on stdout. With `port`, the game starts in the background and the answer is its probe's hello: call `probe` on that port, read stderr with `run_logs`, and stop it with probe commands {\"script\": \"0:quit\"}.",
            json!({
                "project": project, "commands": s("The command script, `t:action[:args];...` (testing §7)."),
                "commandsFile": s("UTF-8 script file, relative to the server's working directory; use instead of commands for long scripts."),
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
                "skip": s("Skip suite files matching this substring."),
                "affected": s("Select changed dependencies since this git revision."),
                "jobs": n("Playtest files at once."), "noCheck": b("Skip the type check."),
                "requireChecked": b("Fail if GPU checks remain unchecked."), "args": args,
            }),
            &[],
        ),
        tool(
            "check",
            "`og check --json`: type-check the game code with tsrs/tsgo and engine lints (OG9xxx).",
            json!({
                "project": project, "tool": s("Checker override: `tsrs` or `tsgo`."),
                "requireChecked": b("Fail on unchecked checks."), "args": args,
            }),
            &[],
        ),
        tool(
            "cook",
            "`og cook --json`: cook assets/ into cooked/ incrementally.",
            json!({
                "project": project, "threads": n("Worker threads."),
                "report": b("Include slowest assets report."), "args": args,
            }),
            &[],
        ),
        tool(
            "build",
            "`og build --json`: cook, type-check and export the web build to dist/.",
            json!({
                "project": project, "publish": b("Build the publish PWA output."),
                "out": s("Output directory."), "noCheck": b("Skip the type check."), "args": args,
            }),
            &[],
        ),
        tool(
            "create",
            "`og create --no-prompt`: scaffold a new game project with creation choices.",
            json!({
                "dir": s("Target directory."), "name": s("Game display name."),
                "id": s("Game slug."), "premise": s("One-line premise."),
                "form": s("Game form choice."), "genre": s("Genre choice."),
                "view": s("Camera view choice."), "look": s("Catalogue look preset."),
                "variant": s("Look variant."), "ui": s("UI style."),
                "platform": s("Target platform."), "input": s("Primary input."),
                "players": s("Player count mode."), "lang": s("Comma-separated locales, e.g. en,zh."),
                "music": s("Music style."),
                "force": b("Overwrite an existing directory."), "args": args,
            }),
            &[],
        ),
        tool(
            "explore",
            "`og explore --json`: Go-Explore state search from snapshots to find invariant violations, throws, and reachable coverage.",
            json!({
                "project": project, "seconds": n("Time budget in seconds."),
                "ticks": n("Tick budget."), "seed": n("RNG seed."),
                "jobs": n("Parallel workers."), "out": s("Output directory."),
                "web": s("Web verification mode: `auto`, `on`, or `off`."),
                "first": b("Stop at first violation or error."),
                "noShrink": b("Skip repro shrinking."), "args": args,
            }),
            &[],
        ),
        tool(
            "replay",
            "`og replay record|compare --json`: record per-tick world hashes or compare a trace for determinism.",
            json!({
                "subcommand": s("`record` or `compare`."),
                "trace": s("Trace JSON path (required for `compare`)."),
                "project": project, "commands": s("Command script for `record`."),
                "commandsFile": s("Command script file for `record`."),
                "ticks": n("Ticks for `record`."), "seed": n("Seed for `record`."),
                "out": s("Output trace.json path for `record`."), "args": args,
            }),
            &["subcommand"],
        ),
        tool(
            "ui",
            "`og ui <subcommand>`: check, preview, literals, list, eject, or add UI components/screens/materials.",
            json!({
                "subcommand": s("`check`, `preview`, `literals`, `list`, `eject`, or `add`."),
                "target": s("Component/screen/material target when required by the subcommand."),
                "project": project, "out": s("Output directory."),
                "force": b("Overwrite existing files on `eject`."), "args": args,
            }),
            &["subcommand"],
        ),
        tool(
            "style",
            "`og style <subcommand>`: list, check-project, init, validate, or add UI styles and art-contract checks.",
            json!({
                "subcommand": s("`list`, `check-project`, `init`, `validate`, or `add`."),
                "target": s("Style name (for `init`) or directory (for `validate`/`add`)."),
                "from": s("Base built-in theme recipe for `init` (default: `minimal`)."),
                "project": project, "noFrames": b("Skip frame captures on `check-project`."),
                "args": args,
            }),
            &["subcommand"],
        ),
        tool(
            "model",
            "`og model <subcommand>`: procedural model & mesh tools (`build`, `measure`, `preview`, `import`, `repair`, `decimate`, `unwrap`, `place`, `collide`, `compress`, `process`, `rig`, `export`).",
            json!({
                "subcommand": s("Model subcommand (`build`, `measure`, `preview`, `help`, ...)."),
                "input": s("Input `.shape.json` or mesh path."),
                "out": s("Output path."), "args": args,
            }),
            &["subcommand"],
        ),
        tool(
            "assets",
            "`og assets --json`: query cooked asset metadata, unused/duplicate assets, references, or move assets (`subcommand: \"mv\"`).",
            json!({
                "subcommand": s("Optional subcommand, e.g. `mv`."),
                "project": project, "kind": s("Filter by asset kind."),
                "group": s("Filter by streaming group."), "query": s("Search text."),
                "unused": b("List unreferenced assets."), "refs": b("Include dependencies."),
                "by": b("Include reverse dependents."), "dupes": b("Find duplicate assets."),
                "args": args,
            }),
            &[],
        ),
        tool(
            "prefab",
            "`og prefab check|show|extract`: validate, inspect, or extract prefabs.",
            json!({
                "subcommand": s("`check`, `show`, or `extract`."),
                "target": s("Prefab name (for `show`) or scene (for `extract`, pass remaining args in `args`)."),
                "project": project, "args": args,
            }),
            &["subcommand"],
        ),
        tool(
            "doctor",
            "`og doctor --json`: verify bun, engine package, native addon, wasm, renderer, and project pin compatibility.",
            json!({
                "project": project, "requireChecked": b("Fail if GPU checks are unavailable."),
                "args": args,
            }),
            &[],
        ),
        tool(
            "render",
            "`og render --json`: the game drawn frame by frame headless into PNG frames or a video, with frames.json.",
            json!({
                "project": project, "commands": s("The command script."), "timeline": s("A timeline to play."),
                "commandsFile": s("UTF-8 script file, relative to the server's working directory; use instead of commands for long scripts."),
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
            "frame",
            "`og frame measure --json`: measure WCAG contrast, high-pass energy, mean colour, and row luminance ramp on a captured PNG.",
            json!({
                "png": s("Captured PNG path."),
                "region": s("Measured box `x,y,w,h`."), "against": s("Background box `x,y,w,h`."),
                "units": s("`px` or `fraction`."), "metric": s("Comma-separated metrics (`contrast,highpass,mean,ramp`)."),
                "rows": n("Number of horizontal bands for `ramp`."),
                "requireChecked": b("Fail if checks cannot run."), "args": args,
            }),
            &["png"],
        ),
        tool(
            "cost",
            "`og cost --json`: measure median GPU frame span difference with a feature off vs on.",
            json!({
                "project": project, "off": s("`og run` options with the feature off."),
                "on": s("`og run` options with the feature on."),
                "runs": n("Alternating runs per side."), "ticks": s("Measured tick range `A..B`."),
                "args": args,
            }),
            &["off"],
        ),
        tool(
            "vfx",
            "`og vfx add|import|preview`: install effect packs, import Effekseer effects, or preview `type vfx`/`vfx2d` assets.",
            json!({
                "subcommand": s("`add`, `import`, or `preview`."),
                "target": s("Pack name/dir (`add`), Effekseer file (`import`), or effect `.ogt` (`preview`)."),
                "project": project, "out": s("Output directory."),
                "size": s("Preview WxH."), "quality": s("`low`, `medium`, or `high`."),
                "projectLook": b("Use og-project.json's look for preview shots and costs, not black-stage bounds."),
                "look": s("Explicit look asset, exclusive with projectLook."), "scene": s("Scene asset for preview shots/costs; applies its resources after the look."),
                "position": s("Effect world position x,y,z; contextual previews do not auto-raise the effect."),
                "camera": s("World-space px,py,pz,tx,ty,tz,fov; otherwise auto-frame at the effect position."),
                "root": s("Assets root for `import`."), "hit": b("Add hit light/shake on `import`."),
                "noCost": b("Skip GPU cost run on `preview`."), "together": b("Preview together on a grid."),
                "anyEngine": b("Allow newer `minEngine` on `add`."), "args": args,
            }),
            &["subcommand"],
        ),
        tool(
            "script",
            "Execute a deterministic TS generator with the cook's executor and cache. Returns its key and source data, or writes data to an explicitly chosen out file. No network, process, clock or undeclared file access.",
            json!({
                "file": s("TS module path relative to root; default export is a synchronous function of ctx.params, ctx.seed and ctx.inputs."),
                "root": s("Input root directory (default current directory; use assets to share a terrain cook key)."),
                "params": {"type": "object"}, "seed": n("Seed (default 1)."),
                "inputs": {"type": "object", "additionalProperties": {"type": "string"}},
                "out": s("Explicit JSON output file; omitted means no project files are written."),
            }),
            &["file"],
        ),
        tool(
            "sfx",
            "`og sfx`: list procedural sound presets, write a TS content declaration or listening WAV, or write a TS library.",
            json!({
                "preset": s("Preset name (omit to list all presets) or `library`."),
                "seed": n("Synth RNG seed."), "pitch": { "type": "number" },
                "length": { "type": "number" }, "volume": { "type": "number" },
                "out": s("Output `content/**/*.ts` declaration or `.wav` path."), "args": args,
            }),
            &[],
        ),
        tool(
            "sprite",
            "`og sprite inspect --json`: measure animated sprite or flipbook sheet frames, drift, loop seam, and alpha fringe.",
            json!({
                "input": s("Path to `.gif`, `.webp`, `.png`, or flipbook `.ogt`."),
                "key": s("Background key: `auto`, `off`, or `#rrggbb`."),
                "tolerance": { "type": "number" }, "holes": b("Also key enclosed background pockets."),
                "out": s("Write measured frames sheet PNG."), "args": args,
            }),
            &["input"],
        ),
        tool(
            "materials",
            "`og materials brief|make --json`: list art-brief images for a material library or build periodic PBR maps.",
            json!({
                "subcommand": s("`brief` or `make`."),
                "library": s("Path to material library `.ogt`."), "args": args,
            }),
            &["subcommand", "library"],
        ),
        tool(
            "i18n",
            "`og i18n extract --json`: check or extract translation keys from dialogues, quests, and tables.",
            json!({
                "project": project, "write": b("Add missing keys to the base locale table."),
                "check": b("Exit non-zero if any locale table is incomplete."), "args": args,
            }),
            &[],
        ),
        tool(
            "flow",
            "`og flow list|routes --json`: inspect interactive-film flows and reachable endings.",
            json!({
                "subcommand": s("`list` or `routes`."), "project": project, "args": args,
            }),
            &["subcommand"],
        ),
        tool(
            "fmt",
            "`og fmt --json`: format `.ogt` files into canonical layout or check formatting.",
            json!({
                "path": s("File, directory, or project root (default: current directory)."),
                "check": b("Check without writing."), "args": args,
            }),
            &[],
        ),
        tool(
            "font",
            "`og font list|add`: list bundled/installed script font packs or install one into a project.",
            json!({
                "subcommand": s("`list` or `add`."),
                "target": s("Font pack name or directory (for `add`)."),
                "project": project, "args": args,
            }),
            &["subcommand"],
        ),
        tool(
            "api_find",
            "`og api find --json` (in-process cached index): rank API and guide entries matching words.",
            json!({
                "words": s("A name, a purpose, a typo."),
                "kind": s("module, fn, record, type, member, const, value, column or guide."),
                "limit": n("How many."),
            }),
            &["words"],
        ),
        tool(
            "api_show",
            "`og api show` (in-process cached index): one API entry, module, UI component or guide (`og.physics.castRay`, `probe`, `Button`, `testing`, `start`, `saves §2`).",
            json!({
                "name": s("The entry, module, component or guide."), "all": b("Every member of a module, or every section of a large guide."),
                "guide": b("The guide, when a module has the same name."),
                "json": b("Return structured JSON."),
            }),
            &["name"],
        ),
        tool(
            "api_list",
            "`og api list --json` (in-process cached index): list every API module and guide with summaries.",
            json!({
                "kind": { "type": "string", "enum": ["module", "guide", "file"] },
            }),
            &[],
        ),
    ])
}
