use super::*;

/// `og ui add <directory>` installs an editable copy of a fixture kit;
/// eject then copies the installed copy's document (the game's edit with
/// it) into assets/ui/fixture/, with no second copy of the
/// kit's code, manifest, skill or sheet, and refuses to overwrite it
/// without --force.
///
/// Ways it could fail: the bundled document ejected over the game's edit,
/// the kit's TypeScript or sheet copied beside the document (a stray second
/// kit in the game's assets), or an existing ejected edit overwritten.
#[test]
fn ui_eject_takes_the_installed_kit_document_and_only_its_companions() {
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let run = |args: &[&str]| {
        let out = og().args(args).arg(&dir).output().unwrap();
        (out.status.code(), text(&out))
    };
    let source = scratch_root();
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("extension.json"), r#"{"format":1,"kind":"kit","name":"fixture","version":"1.0.0","description":"Document copy fixture"}"#).unwrap();
    std::fs::write(source.join("results.svelte"), "<meta name='screen' content='results'><column id='root'><text id='title'>Pos</text></column>").unwrap();
    let (code, out) = run(&["ui", "add", source.to_str().unwrap()]);
    assert_eq!(code, Some(0), "{out}");
    let installed = dir.join("assets/ui-kits/fixture/results.svelte");
    let edited = std::fs::read_to_string(&installed)
        .unwrap()
        .replace(">Pos</text>", ">Position</text>");
    std::fs::write(&installed, &edited).unwrap();

    let (code, out) = run(&["ui", "eject", "fixture/results"]);
    assert_eq!(code, Some(0), "{out}");
    let ejected = dir.join("assets/ui/fixture");
    assert_eq!(
        std::fs::read_to_string(ejected.join("results.svelte")).unwrap(),
        edited
    );
    for kit_only in [
        "index.ts",
        "types.ts",
        "extension.json",
        "src",
        "sheet",
        "fixtures",
    ] {
        assert!(
            !ejected.join(kit_only).exists(),
            "{kit_only} was ejected beside the document"
        );
    }
    let (code, out) = run(&["ui", "eject", "fixture/results"]);
    assert_eq!(code, Some(1), "{out}");
    assert_eq!(
        std::fs::read_to_string(ejected.join("results.svelte")).unwrap(),
        edited
    );
    std::fs::remove_dir_all(source).unwrap();
}

/// `og ui add materials` installs the material presets (each a `type
/// ui-material` naming its WGSL beside it), and the game cooks every one:
/// the source is read relative to the material file, as a kit's relative
/// references are promised to survive installation (docs/ui.md §5). Each
/// cooked material carries its code and its parameters in `param` order.
///
/// Ways it could fail: `source` read only from `assets/` (every preset
/// missing its WGSL once installed), a preset whose WGSL does not compile,
/// or a preset past the four parameters a layer carries.
#[test]
fn the_material_presets_install_and_cook_with_their_wgsl_beside_them() {
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let out = og()
        .args(["ui", "add", "materials"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let out = og().arg("cook").arg(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap();
    for (preset, first) in [
        ("foil", "amount"),
        ("holo", "amount"),
        ("burn", "progress"),
        ("rarity", "color"),
    ] {
        let id = format!("ui-kits/materials/{preset}.ogt");
        let m = bundle
            .ui_material(&id, bundle.read(&id).unwrap().as_slice())
            .unwrap_or_else(|e| panic!("{id}: {e}"));
        assert!(m.code.contains("fn material("), "{id}");
        assert_eq!(m.params.len(), 4, "{id}");
        assert_eq!(m.params[0].name, first, "{id}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// `og vfx add <directory>` installs an effect pack (`kind vfx`) into
/// assets/vfx/<name>/, where its composed effects cook with the emitters
/// they name by the installed path; a second add refuses, and a pack with
/// code (`entry`) is not a valid vfx pack.
///
/// Ways it could fail: the pack copied into a kit's or a style's folder,
/// an existing installation replaced, or a vfx pack allowed to ship code.
#[test]
fn a_vfx_pack_installs_into_assets_vfx_and_cooks() {
    let dir = scratch_root();
    let project = dir.join("game");
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(project.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        project.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let pack = dir.join("hits");
    std::fs::create_dir_all(&pack).unwrap();
    let manifest = |extra: &str| {
        format!(
            "{{\"format\":1,\"kind\":\"vfx\",\"name\":\"hits\",\"version\":\"1.0.0\",\"description\":\"Hit effects\"{extra}}}"
        )
    };
    std::fs::write(
        pack.join("extension.json"),
        manifest(",\"minEngine\":\"0.1.0\",\"keywords\":[\"spark\"],\"tags\":{\"style\":\"stylised\",\"dim\":\"3d\",\"category\":[\"hit-spark\"],\"element\":[]}"),
    )
    .unwrap();
    std::fs::create_dir_all(pack.join("preview")).unwrap();
    std::fs::write(pack.join("preview/hit.png"), b"not installed").unwrap();
    std::fs::write(
        pack.join("sparks.ogt"),
        "type particles\nversion 1\n---\nrate 10\n",
    )
    .unwrap();
    std::fs::write(
        pack.join("hit.ogt"),
        "type vfx\nversion 1\n---\nduration 0.5\nimpact 0.1\nemitter \"vfx/hits/sparks.ogt\" {\n  at 0\n}\n",
    )
    .unwrap();
    let add = || {
        let out = og()
            .args(["vfx", "add"])
            .arg(&pack)
            .arg(&project)
            .output()
            .unwrap();
        (out.status.code(), text(&out))
    };
    let (code, out) = add();
    assert_eq!(code, Some(0), "{out}");
    assert!(project.join("assets/vfx/hits/hit.ogt").is_file(), "{out}");
    assert!(!project.join("assets/vfx/hits/preview").exists(), "{out}");
    std::fs::remove_dir_all(project.join("assets/vfx")).unwrap();
    // A pack for an engine newer than this one installs only on request.
    std::fs::write(
        pack.join("extension.json"),
        manifest(",\"minEngine\":\"99.0.0\""),
    )
    .unwrap();
    let (code, out) = add();
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("needs engine 99.0.0"), "{out}");
    assert!(!project.join("assets/vfx/hits").exists(), "{out}");
    let out = og()
        .args(["vfx", "add"])
        .arg(&pack)
        .arg(&project)
        .arg("--any-engine")
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    std::fs::write(pack.join("extension.json"), manifest(",\"minEngine\":\"0.1.0\",\"keywords\":[\"spark\"],\"tags\":{\"style\":\"stylised\",\"dim\":\"3d\",\"category\":[\"hit-spark\"],\"element\":[]}")).unwrap();
    let (code, out) = add();
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("already exists"), "{out}");

    let out = og().arg("cook").current_dir(&project).output().unwrap();
    assert!(out.status.success(), "{}", text(&out));

    std::fs::remove_dir_all(project.join("assets/vfx")).unwrap();
    for bad in [
        ",\"minEngine\":\"unreleased\"",
        ",\"keywords\":[\"Spark\"]",
        ",\"tags\":{\"style\":\"anime\",\"dim\":\"3d\"}",
    ] {
        std::fs::write(pack.join("extension.json"), manifest(bad)).unwrap();
        let (code, out) = add();
        assert_eq!(code, Some(1), "{bad}: {out}");
        assert!(out.contains("invalid extension manifest"), "{bad}: {out}");
    }
    std::fs::write(
        pack.join("extension.json"),
        manifest(",\"entry\":\"index.ts\",\"types\":\"index.d.ts\""),
    )
    .unwrap();
    std::fs::write(pack.join("index.ts"), "").unwrap();
    std::fs::write(pack.join("index.d.ts"), "").unwrap();
    let (code, out) = add();
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("invalid extension manifest"), "{out}");
}

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

/// `og check` types each screen from its document (`og ui types`): a
/// viewmodel pushed or replaced against the props, a list's items as its
/// fixtures show them, and the viewmodel og.ui.screenVm answers; a name
/// no document has stays untyped; an edit to a document retypes the next
/// check.
///
/// Ways it could fail: the engine's string overload chosen before the
/// game's documents (nothing caught), an unknown screen refused, or types
/// written once and left stale.
#[test]
fn og_check_types_viewmodels_from_the_documents() {
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("assets/ui")).unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let shop = |gold: &str| {
        format!(
            "<meta name='screen' content='shop' data-back='closeShop'>\n<meta name='prop:gold' content='{gold}' data-doc='Gold.'>\n<meta name='prop:items' content='list' data-doc='For sale.'>\n\
             <meta name='fixture:rich' content='{{\"gold\":1,\"items\":[{{\"name\":\"Sword\",\"price\":40}},{{\"name\":\"Shield\",\"price\":30,\"sale\":true}}]}}'>\n\
             <column id='Root'><button id='Buy' onclick={{buy}}>Buy</button><Badge id='Gold' value={{1}}/></column>"
        )
    };
    std::fs::write(dir.join("assets/ui/shop.svelte"), shop("number")).unwrap();
    std::fs::write(
        dir.join("assets/ui/badge.svelte"),
        "<meta name='component' content='Badge'><meta name='prop:value' content='number' data-doc='The number.'><row id='Root'><button id='Tap' onclick={tapped}>{value}</button></row>",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/main.ts"),
        r#"import { og } from "@origingame/engine";
export default og.game({
  start: (world) => {
    og.ui.push(world, "shop", { gold: 1, items: [{ name: "a", price: 1 }] });
    og.ui.push(world, "shop", { gold: "1", items: [] });
    og.ui.replace(world, "shop", { gold: 1, items: [{ name: "a", cost: 1 }] });
    og.ui.push(world, "elsewhere", { anything: 1 });
    const vm = og.ui.screenVm(world, "shop");
    if (vm) Math.round(vm.gold);
  },
});
"#,
    )
    .unwrap();
    let errors = || -> Vec<(u64, u64)> {
        let out = og().args(["check", "--json"]).arg(&dir).output().unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{e}: {}", text(&out)));
        report["diagnostics"]
            .as_array()
            .unwrap_or_else(|| panic!("{report}"))
            .iter()
            .map(|d| (d["line"].as_u64().unwrap(), d["code"].as_u64().unwrap()))
            .collect()
    };
    // The string gold and the item's unknown field; nothing else.
    assert_eq!(errors(), [(5, 2322), (6, 2353)]);
    std::fs::write(dir.join("assets/ui/shop.svelte"), shop("string")).unwrap();
    assert_eq!(
        errors(),
        [(4, 2322), (6, 2322), (6, 2353), (9, 2345)],
        "gold is a string now: the numbers are the wrong ones"
    );
}

/// `og ui slots --json` gives each art slot its whole generation spec:
/// the kind's facts (transparency, legible size), the multi-cell split,
/// the nine-slice insets and the set it is drawn with, whose reference is
/// the first member with its art set, in any of the project's screens.
///
/// Ways it could fail: `itemCells`, `slice` or `artSet` read by the
/// builder but not by `og ui slots` (two parsers drifting), a reference
/// that names a member still on its placeholder, or a set split per file.
#[test]
fn ui_slots_json_gives_each_slot_its_generation_spec_and_set() {
    let dir = scratch_root();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("assets/ui")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"T\"\nentry \"src/main.ts\"\n",
    )
    .unwrap();
    let screen = |name: &str, body: &str| {
        std::fs::write(
            dir.join(format!("assets/ui/{name}.svelte")),
            format!("<meta name='screen' content='{name}'>\n<column id='Root'>\n{body}</column>\n"),
        )
        .unwrap();
    };
    screen(
        "bag",
        "<art id='Sword' kind='itemIcon' item-cells='2 3'/>\n<art id='Frame' kind='styleFrame' fit='nineSlice' slice='0.2 0.3' art-set='hud'/>\n",
    );
    screen("shop", "<art id='Shield' kind='itemIcon' src={7}/>\n");
    let out = og()
        .args(["ui", "slots", "--json"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let rows: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let slot = |path: &str| {
        rows.as_array()
            .unwrap()
            .iter()
            .find(|s| s["path"] == path)
            .cloned()
            .unwrap_or_else(|| panic!("no {path} in {rows}"))
    };
    let sword = slot("bag/Root/Sword");
    assert_eq!(sword["cells"], serde_json::json!([2, 3]), "{sword}");
    assert_eq!(sword["transparent"], true);
    assert_eq!(sword["mask"], "alpha");
    assert_eq!(sword["art"], "placeholder");
    assert_eq!(sword["set"]["name"], "itemIcon");
    assert_eq!(
        sword["set"]["members"],
        serde_json::json!(["bag/Root/Sword", "shop/Root/Shield"])
    );
    assert_eq!(sword["set"]["reference"], "shop/Root/Shield");
    assert_eq!(sword["source"], "ui/bag.svelte:3");
    let frame = slot("bag/Root/Frame");
    assert_eq!(
        frame["insets"],
        serde_json::json!([0.2, 0.3, 0.2, 0.3]),
        "{frame}"
    );
    assert_eq!(frame["cells"], serde_json::Value::Null);
    assert_eq!(
        frame["set"],
        serde_json::json!({ "name": "hud", "members": ["bag/Root/Frame"], "reference": null })
    );
    assert_eq!(frame["problems"], serde_json::json!([]), "{frame}");
}
