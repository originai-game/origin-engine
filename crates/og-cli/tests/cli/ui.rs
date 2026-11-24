use super::*;

/// Serves `root`'s files over HTTP on a free local port (GET only, one
/// request per connection); returns `http://127.0.0.1:<port>`.
fn serve(root: PathBuf) -> String {
    use std::io::{BufRead, BufReader, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            let _ = reader.read_line(&mut line);
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header).unwrap_or(0) == 0 || header == "\r\n" {
                    break;
                }
            }
            let path = line
                .split(' ')
                .nth(1)
                .unwrap_or("/")
                .trim_start_matches('/');
            let file = root.join(path);
            let (status, body) = match std::fs::read(&file) {
                Ok(body) if !path.contains("..") => ("200 OK", body),
                _ => ("404 Not Found", Vec::new()),
            };
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(&body);
        }
    });
    url
}

/// `og vfx add <name>`, for a pack that is neither a directory nor bundled
/// nor installed, downloads it from this engine's release: a downloads tree
/// written by scripts/release-dist.ts's own code (the fixture
/// tests/fixtures/vfx-release, signed with a test key) served over HTTP.
/// The good pack installs as a local directory would (no preview/); a
/// tampered archive, a tampered SHA256SUMS, a signature by another key, an
/// archive entry with `..`, one that is a symlink, and a pack the release
/// does not list are each refused, naming what failed, and leave nothing in
/// the game or the temporary directory.
///
/// Ways it could fail: an archive installed without its signed sha256, a
/// SHA256SUMS trusted without its signature or with any key, an entry
/// written outside the pack, a link followed, or a half-installed pack or a
/// download left behind after a refusal.
#[test]
fn vfx_add_downloads_a_signed_pack_and_refuses_anything_else() {
    let dir = scratch_root();
    let project = dir.join("game");
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(project.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        project.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let tmp = dir.join("tmp");
    std::fs::create_dir_all(&tmp).unwrap();
    let channels = dir.join("channels");
    let url = serve(channels.clone());
    let version = env!("CARGO_PKG_VERSION");
    let fixture = repo().join("tests/fixtures/vfx-release");
    // One release per case, built and signed by the fixture's channel.ts; returns its key.
    let channel = |case: &str, evil: &[&str]| -> String {
        let out = Command::new("bun")
            .arg(fixture.join("channel.ts"))
            .arg(channels.join(case))
            .arg(version)
            .args(evil)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out));
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        stdout
            .lines()
            .find_map(|l| l.strip_prefix("pubkey "))
            .unwrap()
            .to_string()
    };
    let add = |case: &str, pack: &str, key: Option<&str>| {
        let mut c = og();
        c.args(["vfx", "add", pack])
            .arg(&project)
            .env("ORIGIN_ENGINE_DOWNLOADS", format!("{url}/{case}"))
            .env("TMPDIR", &tmp)
            .env_remove("ORIGIN_ENGINE_PUBKEY");
        if let Some(key) = key {
            c.env("ORIGIN_ENGINE_PUBKEY", key);
        }
        let out = c.output().unwrap();
        (out.status.code(), text(&out))
    };
    let nothing_left = |case: &str, out: &str| {
        let vfx = project.join("assets/vfx");
        let left: Vec<_> = std::fs::read_dir(&vfx)
            .map(|d| d.map(|e| e.unwrap().file_name()).collect())
            .unwrap_or_default();
        assert!(
            left.is_empty(),
            "{case}: {left:?} left in assets/vfx\n{out}"
        );
        let downloads: Vec<_> = std::fs::read_dir(&tmp)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .filter(|n| n.starts_with("og-vfx-"))
            .collect();
        assert!(
            downloads.is_empty(),
            "{case}: {downloads:?} left in TMPDIR\n{out}"
        );
        assert!(
            !tmp.join("glint.ogt").exists(),
            "{case}: an entry escaped\n{out}"
        );
    };

    let key = channel("good", &[]);
    let (code, out) = add("good", "spark-pack", Some(&key));
    assert_eq!(code, Some(0), "{out}");
    let installed = project.join("assets/vfx/spark-pack");
    for f in [
        "extension.json",
        "burst.ogt",
        "glint.ogt",
        "sparks.ogt",
        "spark.png",
    ] {
        assert_eq!(
            std::fs::read(installed.join(f)).unwrap(),
            std::fs::read(fixture.join("spark-pack").join(f)).unwrap(),
            "{f}"
        );
    }
    assert!(!installed.join("preview").exists(), "{out}");
    std::fs::remove_dir_all(project.join("assets/vfx")).unwrap();
    nothing_left("good", &out);

    // A pack the release does not offer.
    let (code, out) = add("good", "no-such-pack", Some(&key));
    assert_eq!(code, Some(1), "{out}");
    assert!(
        out.contains("has no downloadable vfx pack `no-such-pack` (it has spark-pack)"),
        "{out}"
    );
    nothing_left("unlisted", &out);

    // The engine's own key (no test key): the test release is not signed by it.
    let (code, out) = add("good", "spark-pack", None);
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("not the engine's F6CC11AC12997D89"), "{out}");
    nothing_left("another key", &out);

    let release = |case: &str| channels.join(case).join(version);
    let key = channel("archive", &[]);
    let archive = release("archive").join("vfx/spark-pack.tar.gz");
    let mut bytes = std::fs::read(&archive).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 1;
    std::fs::write(&archive, bytes).unwrap();
    let (code, out) = add("archive", "spark-pack", Some(&key));
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("the signed SHA256SUMS says"), "{out}");
    nothing_left("tampered archive", &out);

    let key = channel("sums", &[]);
    let sums = release("sums").join("SHA256SUMS");
    let text_sums = std::fs::read_to_string(&sums).unwrap();
    let line = text_sums
        .lines()
        .find(|l| l.ends_with("vfx/spark-pack.tar.gz"))
        .unwrap();
    std::fs::write(
        &sums,
        text_sums.replace(line, &format!("{}  vfx/spark-pack.tar.gz", "0".repeat(64))),
    )
    .unwrap();
    let (code, out) = add("sums", "spark-pack", Some(&key));
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("SHA256SUMS.minisig does not verify"), "{out}");
    nothing_left("tampered SHA256SUMS", &out);

    for (case, want) in [
        ("traversal", "the unsafe path \"spark-pack/../glint.ogt\""),
        ("symlink", "spark-pack/glint.ogt is a symlink"),
    ] {
        let key = channel(case, &["--evil", case]);
        let (code, out) = add(case, "spark-pack", Some(&key));
        assert_eq!(code, Some(1), "{case}: {out}");
        assert!(out.contains(want), "{case}: {out}");
        assert!(out.contains("nothing written"), "{case}: {out}");
        nothing_left(case, &out);
    }
    let _ = std::fs::remove_dir_all(&dir);
}
