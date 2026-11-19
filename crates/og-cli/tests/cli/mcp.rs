use super::*;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::Stdio;

/// `og mcp` as an MCP client sees it: the handshake, the seven tools, and
/// a tool's answer being exactly the command's output (`api_find` is `og
/// api find --json`'s stdout); a probe nobody serves is a tool error, not
/// a protocol error.
#[test]
fn mcp_serves_the_og_commands_as_tools_with_their_own_output() {
    let mut server = og()
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = server.stdin.take().unwrap();
    let mut output = BufReader::new(server.stdout.take().unwrap());
    let mut ask = |msg: Value| -> Option<Value> {
        writeln!(input, "{msg}").unwrap();
        msg.get("id")?;
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        let answer: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(answer["id"], msg["id"], "{answer}");
        Some(answer)
    };
    let init = ask(json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}
    }}))
    .unwrap();
    assert_eq!(init["result"]["protocolVersion"], json!("2025-03-26"));
    assert!(init["result"]["capabilities"]["tools"].is_object());
    assert!(ask(json!({"jsonrpc": "2.0", "method": "notifications/initialized"})).is_none());

    let list = ask(json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})).unwrap();
    let mut names: Vec<&str> = list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "api_find", "api_show", "compare", "probe", "render", "run", "test"
        ]
    );

    let call = |name: &str, arguments: Value| json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": name, "arguments": arguments}});
    let found = ask(call(
        "api_find",
        json!({"words": "why is it not drawn", "limit": 3}),
    ))
    .unwrap();
    let direct = og()
        .args([
            "api",
            "find",
            "why is it not drawn",
            "--limit",
            "3",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(found["result"]["isError"], json!(false), "{found}");
    assert_eq!(
        found["result"]["content"][0]["text"],
        json!(String::from_utf8(direct.stdout).unwrap())
    );

    let unreachable = ask(call("probe", json!({"method": "hello", "port": 1}))).unwrap();
    assert_eq!(
        unreachable["result"]["isError"],
        json!(true),
        "{unreachable}"
    );
    let text = unreachable["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert!(
        text.starts_with("og probe: cannot reach the probe"),
        "{text}"
    );

    drop(input);
    assert!(server.wait().unwrap().success());
}
