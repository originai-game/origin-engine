use super::*;
use serde_json::{Value, json};

/// `og probe capture_views` writes each frame beside --out as
/// `<stem>-<view>.png` and prints its path in place of the base64, as
/// `capture`'s png: the frames never reach stdout.
#[test]
fn capture_views_writes_each_frame_beside_out_and_prints_its_path() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    // Two "PNGs" of different bytes, base64: "PNG0" and "PNG1".
    let server = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut ws = tungstenite::accept(stream).unwrap();
        let call: Value = serde_json::from_str(ws.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(call["method"], json!("capture_views"));
        let frame = |view: u32, png: &str| json!({"view": view, "width": 2, "height": 2, "scale": 1, "png": png});
        let answer = json!({"jsonrpc": "2.0", "id": call["id"], "result": {
            "tick": 7, "frames": [frame(0, "UE5HMA=="), frame(1, "UE5HMQ==")]
        }});
        ws.send(tungstenite::Message::text(answer.to_string()))
            .unwrap();
        let _ = ws.read();
    });
    let dir = TempDir::new("og-probe-views");
    let out = dir.join("shot.png");
    let run = og()
        .args(["probe", "capture_views", r#"{"views": []}"#, "--port"])
        .arg(port.to_string())
        .arg("--out")
        .arg(&out)
        .output()
        .unwrap();
    server.join().unwrap();
    assert!(run.status.success(), "{}", text(&run));
    let printed: Value = serde_json::from_slice(&run.stdout).unwrap();
    for (i, bytes) in [b"PNG0", b"PNG1"].iter().enumerate() {
        let file = dir.join(format!("shot-{i}.png"));
        assert_eq!(&std::fs::read(&file).unwrap(), bytes);
        let frame = &printed["frames"][i];
        assert_eq!(frame["png"], json!(file.to_string_lossy()), "{printed}");
        assert_eq!(frame["view"], json!(i));
    }
    assert_eq!(printed["tick"], json!(7));
    assert!(!out.exists(), "nothing is written at --out itself");
}
