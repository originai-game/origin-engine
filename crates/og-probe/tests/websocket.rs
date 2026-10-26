//! The loopback WebSocket at `/probe`: a real client, calls answered on the
//! world's thread between ticks, malformed frames answered by the transport.
#![cfg(all(feature = "ws", not(feature = "publish")))]

use crate::support;

use og_probe::server::{DEFAULT_PORT, ProbeServer};
use serde_json::{Value, json};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

fn ask(socket: &mut Socket, text: &str) -> Value {
    socket.send(Message::text(text)).unwrap();
    loop {
        if let Message::Text(t) = socket.read().unwrap() {
            return serde_json::from_str(&t).unwrap();
        }
    }
}

#[test]
fn a_real_client_is_answered_on_the_world_thread() {
    let server = ProbeServer::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let port = server.port();
    // The world's thread: a host loop that ticks and serves between ticks.
    let world = thread::spawn(move || {
        let mut h = support::build(7, true);
        h.probe.pause();
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut answered = 0;
        while answered < 5 && Instant::now() < deadline {
            answered += h.probe.serve(&mut h.world, &server);
            h.probe.frame(&mut h.world, 1).unwrap();
            thread::sleep(Duration::from_millis(1));
        }
        (h.probe.tick(&h.world), answered)
    });
    let (mut socket, _) = tungstenite::connect(format!("ws://127.0.0.1:{port}/probe")).unwrap();
    let hello = ask(
        &mut socket,
        r#"{"jsonrpc":"2.0","id":1,"method":"hello","params":{}}"#,
    );
    assert_eq!(hello["result"]["protocol"], json!(5));
    assert_eq!(hello["result"]["substrate"], json!("origin-engine"));
    // Answered by the transport; never reaches the world.
    assert_eq!(ask(&mut socket, "{")["error"]["code"], json!(-32700));
    socket.send(Message::binary(vec![1u8, 2, 3])).unwrap();
    let binary = loop {
        if let Message::Text(t) = socket.read().unwrap() {
            break serde_json::from_str::<Value>(&t).unwrap();
        }
    };
    assert_eq!(binary["error"]["code"], json!(-32600));
    let advanced = ask(
        &mut socket,
        r#"{"jsonrpc":"2.0","id":2,"method":"advance","params":{"steps":5}}"#,
    );
    assert_eq!(
        advanced["result"],
        json!(5),
        "paused: only advance moves the world"
    );
    let frame = ask(
        &mut socket,
        r#"{"jsonrpc":"2.0","id":"c","method":"capture","params":{"width":32,"height":18}}"#,
    );
    assert_eq!(
        (frame["id"].clone(), frame["result"]["tick"].clone()),
        (json!("c"), json!(5))
    );
    let unknown = ask(&mut socket, r#"{"jsonrpc":"2.0","id":3,"method":"nope"}"#);
    assert_eq!(unknown["error"]["code"], json!(-32601));
    let patched = ask(
        &mut socket,
        r#"{"jsonrpc":"2.0","id":4,"method":"patch","params":{"id":"player","parameter":"Mover.speed","value":99}}"#,
    );
    assert_eq!(
        patched["result"],
        json!(false),
        "a refusal crosses the wire as a result"
    );
    let (tick, answered) = world.join().unwrap();
    assert_eq!((tick, answered), (5, 5));
}

#[test]
fn other_paths_and_non_loopback_addresses_are_refused() {
    let server = ProbeServer::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    assert!(tungstenite::connect(format!("ws://127.0.0.1:{}/other", server.port())).is_err());
    assert!(ProbeServer::bind("0.0.0.0:0".parse().unwrap()).is_err());
    assert!(
        ProbeServer::start(0).unwrap().is_none(),
        "port 0 disables the server"
    );
    assert_eq!(DEFAULT_PORT, 47800);
}

#[test]
fn a_browser_page_is_refused_at_the_handshake() {
    use tungstenite::client::IntoClientRequest;
    let server = ProbeServer::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let mut request = format!("ws://127.0.0.1:{}/probe", server.port())
        .into_client_request()
        .unwrap();
    request
        .headers_mut()
        .insert("Origin", "https://any.example".parse().unwrap());
    match tungstenite::connect(request) {
        Err(tungstenite::Error::Http(response)) => {
            assert_eq!(response.status(), 403);
            let body = String::from_utf8(response.body().clone().unwrap_or_default()).unwrap();
            assert!(body.contains("Origin"), "{body}");
        }
        Err(e) => panic!("refused, but not by the Origin check: {e}"),
        Ok(_) => panic!("a request with an Origin header upgraded"),
    }
}
