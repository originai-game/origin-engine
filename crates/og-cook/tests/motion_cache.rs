use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn motion_installs_only_when_used_and_verifies_cache_before_offline_cooking() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("motion-cache");
    let _ = std::fs::remove_dir_all(&root);
    for dir in ["assets", "src", "engine", "cache"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    let weights = b"asymmetric pinned motion weights";
    let digest = ring::digest::digest(&ring::digest::SHA256, weights);
    let sha: String = digest.as_ref().iter().map(|b| format!("{b:02x}")).collect();
    let pin = |url: &str| {
        std::fs::write(
            root.join("engine/motion.json"),
            serde_json::json!({
                "url": url, "bytes": weights.len(), "sha256": sha,
            })
            .to_string(),
        )
        .unwrap();
    };
    let cook = || {
        Command::new(env!("CARGO_BIN_EXE_og-cook"))
            .arg(&root)
            .env("OG_ENGINE_PACKAGE", root.join("engine"))
            .env("ORIGIN_ENGINE_CACHE", root.join("cache"))
            .env_remove("OG_MOTION_MODEL")
            .output()
            .unwrap()
    };
    let dead = TcpListener::bind("127.0.0.1:0").unwrap();
    let unavailable = format!("http://{}/model", dead.local_addr().unwrap());
    drop(dead);
    pin(&unavailable);
    std::fs::write(root.join("src/main.ts"), "export function start() {}").unwrap();
    assert!(
        cook().status.success(),
        "a game without motion must not download weights"
    );
    assert!(!root.join("cooked/motion/model.ogb").exists());
    std::fs::write(root.join("src/main.ts"), "og.motion.capabilities(world);").unwrap();
    let offline = cook();
    assert!(!offline.status.success());
    assert!(String::from_utf8_lossy(&offline.stderr).contains("offline or download refused"));

    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    pin(&format!("http://{}/model", server.local_addr().unwrap()));
    let worker = std::thread::spawn(move || {
        let (mut socket, _) = server.accept().unwrap();
        let mut request = [0; 4096];
        assert!(socket.read(&mut request).unwrap() > 0);
        write!(
            socket,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            weights.len()
        )
        .unwrap();
        socket.write_all(weights).unwrap();
    });
    let installed = cook();
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );
    worker.join().unwrap();
    let cached = root.join(format!("cache/models/motion/{sha}.safetensors"));
    assert_eq!(std::fs::read(&cached).unwrap(), weights);
    let pack = std::fs::read(root.join("cooked/motion/model.ogb")).unwrap();
    let files = og_cook::runtime::unpack(&pack).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].0, "motion/model.safetensors");
    assert_eq!(files[0].1.as_slice(), weights);
    pin(&unavailable);
    assert!(
        cook().status.success(),
        "the verified cache must work offline"
    );
    assert!(
        !root.join("engine/motion").exists(),
        "never mutate the engine install"
    );
    std::fs::write(&cached, vec![0; weights.len()]).unwrap();
    let corrupt = cook();
    assert!(
        !corrupt.status.success(),
        "same-length corrupted cache must not be used"
    );
    assert!(String::from_utf8_lossy(&corrupt.stderr).contains("verified cache"));
    assert!(
        std::fs::read_dir(cached.parent().unwrap())
            .unwrap()
            .all(|e| !e.unwrap().path().to_string_lossy().ends_with(".part"))
    );
    std::fs::remove_dir_all(root).unwrap();
}
