use super::*;

/// A pack's checked declarations install outside assets, conflict atomically,
/// and are usable by the ordinary cook and native sheet loader.
#[test]
fn vfx_pack_installs_checked_pixel_effects_and_cooks() {
    use og_anim::cooked::BundleAnim;
    let dir = TempDir::new("og-pixel-pack");
    let write = |file: &str, text: &str| {
        let path = dir.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write("src/main.ts", "");
    write(
        "og-project.json",
        r#"{"version":1,"name":"Effects","engine":"0.9.0"}"#,
    );
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author';export default defineProject({id:'effects',entry:'src/main.ts'});",
    );
    write(
        "pack/extension.json",
        r#"{"format":1,"kind":"vfx","name":"tiny","version":"1.0.0","description":"Tiny effect"}"#,
    );
    let declaration = "import {file} from '@origingame/engine/author';export default file.vfx2d({recipe:{w:8,h:12,frames:3,layers:[{type:'burst'}]}});";
    write("pack/content/hit.ts", declaration);
    let add = || {
        og().args(["vfx", "add"])
            .arg(dir.join("pack"))
            .arg(&dir)
            .output()
            .unwrap()
    };
    write("content/vfx/tiny/keep.ts", "owned");
    let out = add();
    assert!(!out.status.success(), "{}", text(&out));
    assert!(!dir.join("assets/vfx/tiny").exists());
    assert_eq!(
        std::fs::read_to_string(dir.join("content/vfx/tiny/keep.ts")).unwrap(),
        "owned"
    );
    std::fs::remove_dir_all(dir.join("content/vfx/tiny")).unwrap();
    std::fs::remove_dir(dir.join("content/vfx")).unwrap();
    write("content/vfx", "owned parent");
    let out = add();
    assert!(!out.status.success(), "{}", text(&out));
    assert!(
        !dir.join("assets/vfx/tiny").exists(),
        "half-installed metadata survived"
    );
    std::fs::remove_file(dir.join("content/vfx")).unwrap();
    let out = add();
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        std::fs::read_to_string(dir.join("content/vfx/tiny/hit.ts")).unwrap(),
        declaration
    );
    assert!(!dir.join("assets/vfx/tiny/content").exists());
    let out = og().arg("cook").arg(&dir).output().unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let b = og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap();
    let sheet = b.load_flipbook("base/vfx/tiny/hit").unwrap();
    assert_eq!(sheet.size, [24, 12]);
    assert_eq!(sheet.texture, "base/vfx/tiny/hit#sheet");
    assert!(b.read(&sheet.texture).is_ok());
}

/// Font installation owns two roots atomically, with declarations outside
/// binary assets and no rewrite of the distributed source or its references.
#[test]
fn font_pack_installs_typed_content_and_cooks() {
    use og_i18n::BundleFontPacks;
    let dir = TempDir::new("og-font-pack");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        dir.join("og-project.json"),
        r#"{"version":1,"name":"Fonts","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(dir.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'fonts',entry:'src/main.ts'});").unwrap();
    let source = repo().join("templates/fonts/latin");
    let add = || {
        og().args(["font", "add"])
            .arg(&source)
            .arg(&dir)
            .output()
            .unwrap()
    };
    std::fs::create_dir_all(dir.join("content/fonts/latin")).unwrap();
    std::fs::write(dir.join("content/fonts/latin/keep.ts"), "owned").unwrap();
    let out = add();
    assert!(!out.status.success(), "{}", text(&out));
    assert!(!dir.join("assets/fonts/latin").exists());
    assert_eq!(
        std::fs::read_to_string(dir.join("content/fonts/latin/keep.ts")).unwrap(),
        "owned"
    );
    std::fs::remove_dir_all(dir.join("content/fonts/latin")).unwrap();
    let out = add();
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        std::fs::read(dir.join("content/fonts/latin/index.ts")).unwrap(),
        std::fs::read(source.join("content/fonts/latin/index.ts")).unwrap()
    );
    assert!(dir.join("assets/fonts/latin/NotoSans-Latin.ttf").is_file());
    assert!(!dir.join("assets/fonts/latin/content").exists());
    let out = og().arg("cook").arg(&dir).output().unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap();
    let bytes = bundle.read("base/fonts/latin/index").unwrap();
    let pack = bundle.font_pack("base/fonts/latin/index", &bytes).unwrap();
    assert_eq!(pack.faces, ["fonts/latin/NotoSans-Latin.ttf"]);
    assert!(pack.covers('é'));
}

/// Installed material declarations and captured WGSL form one usable pack;
/// conflicts in any source root refuse the whole install without overwrites.
#[test]
fn material_pack_installs_checked_sources_and_cooks() {
    let dir = TempDir::new("og-material-pack");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/main.ts"), "export default 0;\n").unwrap();
    std::fs::write(
        dir.join("og-project.json"),
        r#"{"version":1,"name":"Materials","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(dir.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'materials',entry:'src/main.ts'});").unwrap();
    let add = || {
        og().args(["ui", "add", "materials"])
            .arg(&dir)
            .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
            .output()
            .unwrap()
    };
    std::fs::create_dir_all(dir.join("sources/materials/materials")).unwrap();
    std::fs::write(dir.join("sources/materials/materials/keep.wgsl"), "owned").unwrap();
    let out = add();
    assert!(!out.status.success(), "{}", text(&out));
    assert!(!dir.join("assets/materials/materials").exists());
    assert!(!dir.join("content/materials/materials").exists());
    assert_eq!(
        std::fs::read_to_string(dir.join("sources/materials/materials/keep.wgsl")).unwrap(),
        "owned"
    );
    std::fs::remove_dir_all(dir.join("sources/materials/materials")).unwrap();
    // A failure after staging metadata/content must roll those roots back.
    std::fs::remove_dir(dir.join("sources/materials")).unwrap();
    std::fs::write(dir.join("sources/materials"), "owned parent").unwrap();
    let out = add();
    assert!(!out.status.success(), "{}", text(&out));
    assert!(!dir.join("assets/materials/materials").exists());
    assert!(!dir.join("content/materials/materials").exists());
    assert_eq!(
        std::fs::read_to_string(dir.join("sources/materials")).unwrap(),
        "owned parent"
    );
    std::fs::remove_file(dir.join("sources/materials")).unwrap();
    let out = add();
    assert!(out.status.success(), "{}", text(&out));
    let out = add();
    assert!(!out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("nothing written"));
    for command in ["check", "cook"] {
        let out = og()
            .arg(command)
            .arg(&dir)
            .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out));
    }
    let bundle = og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap();
    for name in ["burn", "foil", "holo", "rarity"] {
        let source = format!("sources/materials/materials/{name}.wgsl");
        assert_eq!(
            std::fs::read(dir.join(&source)).unwrap(),
            std::fs::read(repo().join(format!(
                "packages/engine/extensions/materials/sources/{name}.wgsl"
            )))
            .unwrap()
        );
        let id = format!("base/materials/materials/{name}");
        let material = bundle.ui_material(&id, &bundle.read(&id).unwrap()).unwrap();
        assert_eq!(
            material.code,
            std::fs::read_to_string(dir.join(source)).unwrap()
        );
        assert_eq!(material.params.len(), 4);
    }
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
        project.join("og-project.json"),
        r#"{"version":1,"name":"T","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(project.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'test',entry:'src/main.ts'});").unwrap();
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
    std::fs::remove_dir_all(project.join("content/vfx")).unwrap();
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

/// The public check report names source facts, preserves a successful exit
/// for warnings, and does not mistake a built-in wrapper or unknown art for
/// a handwritten replacement/missing texture.
#[test]
fn check_reports_screen_facts_without_refusing_the_game() {
    let dir = TempDir::new("og-screen-check");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("og-project.json"),
        r#"{"version":1,"name":"Screens","engine":"0.9.0"}"#,
    )
    .unwrap();
    std::fs::write(dir.join("project.ts"), "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'screens',entry:'src/main.tsx'});").unwrap();
    let check = |source: &str, json: bool| {
        std::fs::write(dir.join("src/main.tsx"), source).unwrap();
        let out = og()
            .arg("check")
            .arg(&dir)
            .args(json.then_some("--json"))
            .env("OG_ENGINE_PACKAGE", repo().join("packages/engine"))
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out));
        String::from_utf8(out.stdout).unwrap()
    };
    let events = |out: &str| -> Vec<serde_json::Value> {
        out.lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    };
    let source = r#"
import { View, Text } from "@origingame/engine/ui";
export function TitleScreen() { return <View><Text>Title</Text></View>; }
export const MainMenu = () => <View />;
export function PauseMenu() { return View({}); }
export function SettingsScreen() { return <View />; }
export function SaveLoadScreen() { return <View />; }
export function Results() { return <View />; }
export function GameOver() { return <View />; }
export function ResultsStats() { return <Text>Not a screen name</Text>; }
"#;
    let out = events(&check(source, true));
    let warnings = out.iter().find(|e| e["event"] == "ui-screens").unwrap()["warnings"]
        .as_array()
        .unwrap();
    assert_eq!(warnings.len(), 7, "{out:?}");
    for (warning, builtin) in warnings.iter().zip([
        "title",
        "main-menu",
        "pause",
        "settings",
        "save-load",
        "results",
        "game-over",
    ]) {
        assert_eq!(warning["code"], 9113);
        assert!(
            warning["message"]
                .as_str()
                .unwrap()
                .contains(&format!("og ui eject screens/{builtin}"))
        );
    }
    let out = check(source, false);
    assert!(out.contains("warning OG9113:"), "{out}");

    let source = r#"
import { View, mount } from "@origingame/engine/ui";
import { TitleScreen as Builtin, type TitleScreenProps } from "@origingame/engine/ui/screens/title.tsx";
import * as Screens from "@origingame/engine/ui/screens/title.tsx";
import type { World } from "@origingame/engine";
export const TitleScreen = (p: TitleScreenProps) => <View><Builtin {...p} /></View>;
export function art(p: TitleScreenProps, world: World) {
  const zero = <Builtin keyArt={0} logo={0} name="T" legal="" version="1" />;
  // @ts-expect-error The omitted logo is deliberately checked separately from type errors.
  const missing = <Screens.TitleScreen keyArt={p.keyArt} name="T" legal="" version="1" />;
  // @ts-expect-error The later spread deliberately overwrites both literal props.
  const unknown = <Builtin keyArt={0} logo={0} {...p} />;
  const explicit = <Builtin {...p} keyArt={0} />;
  const mounted = mount(world, Builtin, { ...p, logo: 0 });
  const supplied = Builtin(p);
  function shadow(Builtin: (p: object) => unknown) { return Builtin({}); }
  return [zero, missing, unknown, explicit, mounted, supplied, shadow];
}
"#;
    let out = events(&check(source, true));
    let warnings = out.iter().find(|e| e["event"] == "ui-screens").unwrap()["warnings"]
        .as_array()
        .unwrap();
    assert_eq!(warnings.len(), 5, "{out:?}");
    assert!(warnings.iter().all(|w| w["code"] == 9112), "{out:?}");
    assert_eq!(
        warnings
            .iter()
            .filter(|w| w["message"].as_str().unwrap().contains("is omitted"))
            .count(),
        1
    );
    let out = events(&check(
        r#"
import { View } from "@origingame/engine/ui";
import { TitleScreen as Builtin, type TitleScreenProps } from "@origingame/engine/ui/screens/title.tsx";
export function TitleScreen(p: TitleScreenProps) { return <View><Builtin keyArt={p.keyArt} logo={p.logo} name={p.name} version={p.version} legal={p.legal} /></View>; }
"#,
        true,
    ));
    assert!(out.iter().all(|e| e["event"] != "ui-screens"), "{out:?}");

    // Capability facts must follow the typed shader entry, not stale OGT discovery.
    std::fs::create_dir_all(dir.join("content/shaders")).unwrap();
    let shader = og_render::example_full_shader();
    std::fs::write(dir.join("content/shaders/unlit.ts"), format!(
        "import {{file}} from '@origingame/engine/author';export default file.shader({{mode:{{kind:'full',lit:false}},code:{},params:[{{name:'amount',kind:'float'}},{{name:'edge',kind:'float'}},{{name:'scale',kind:'float'}},{{name:'glow',kind:'vec4'}}]}});",
        serde_json::to_string(&shader.code).unwrap()
    )).unwrap();
    let out = events(&check("export default 0;", true));
    let warnings = out
        .iter()
        .find(|e| e["event"] == "shader-capabilities")
        .expect("typed full-unlit shader was not inspected")["warnings"]
        .as_array()
        .unwrap();
    assert_eq!(warnings.len(), 1, "{out:?}");
    assert_eq!(warnings[0]["mode"], "full unlit");
    assert!(
        warnings[0]["file"]
            .as_str()
            .unwrap()
            .ends_with("content/shaders/unlit.ts")
    );
}
