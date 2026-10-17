//! The `og` binary as a user runs it: exit codes and what it prints.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

fn og() -> Command {
    Command::new(env!("CARGO_BIN_EXE_og"))
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// A fresh directory that holds a copy of the repository's IDL, as a root.
fn scratch_root() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "og-cli-test-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let idl = dir.join(og_api_path());
    std::fs::create_dir_all(idl.parent().unwrap()).unwrap();
    std::fs::copy(repo().join(og_api_path()), &idl).unwrap();
    dir
}

fn og_api_path() -> &'static str {
    "crates/og-api/api.ogt"
}

#[test]
fn version_names_og_and_the_engine() {
    let out = og().arg("--version").output().unwrap();
    assert!(out.status.success());
    let v = env!("CARGO_PKG_VERSION");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        format!("og {v} (origin-engine {v})")
    );
}

#[test]
fn api_check_passes_on_the_repository() {
    let out = og()
        .args(["api", "check"])
        .current_dir(repo())
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("match crates/og-api/api.ogt"));
}

#[test]
fn api_check_detects_a_hand_edit_and_a_stray_file_and_gen_repairs_them() {
    let root = scratch_root();
    let root_arg = root.to_str().unwrap();
    let gen = og()
        .args(["api", "gen", "--root", root_arg])
        .output()
        .unwrap();
    assert!(gen.status.success(), "{}", text(&gen));
    let clean = og()
        .args(["api", "check", "--root", root_arg])
        .output()
        .unwrap();
    assert!(clean.status.success(), "{}", text(&clean));

    // A hand edit in the middle of the declarations.
    let dts = root.join("packages/engine/types/index.d.ts");
    let original = std::fs::read_to_string(&dts).unwrap();
    std::fs::write(
        &dts,
        original.replacen("readonly tick: number;", "readonly tick: string;", 1),
    )
    .unwrap();
    // A file nobody generates in a generated directory.
    std::fs::write(root.join("packages/engine/schema/notes.json"), "{}").unwrap();

    let out = og()
        .args(["api", "check", "--root", root_arg])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    let report = text(&out);
    assert!(report.contains("2 file(s) differ"), "{report}");
    assert!(
        report.contains("packages/engine/types/index.d.ts: differs"),
        "{report}"
    );
    assert!(
        report.contains("packages/engine/schema/notes.json: not generated from api.ogt"),
        "{report}"
    );
    assert!(
        !report.contains("bindings.json"),
        "untouched files are not reported: {report}"
    );

    std::fs::remove_file(root.join("packages/engine/schema/notes.json")).unwrap();
    assert!(og()
        .args(["api", "gen", "--root", root_arg])
        .status()
        .unwrap()
        .success());
    assert!(og()
        .args(["api", "check", "--root", root_arg])
        .status()
        .unwrap()
        .success());
    assert_eq!(std::fs::read_to_string(&dts).unwrap(), original);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn api_gen_refuses_an_invalid_idl_and_names_the_line() {
    let root = scratch_root();
    let idl = root.join(og_api_path());
    let text_in = std::fs::read_to_string(&idl).unwrap();
    std::fs::write(
        &idl,
        text_in.replacen("returns Result.Entity", "returns Result.Entty", 1),
    )
    .unwrap();
    let out = og()
        .args(["api", "gen", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out).contains("unknown type `Entty`"),
        "{}",
        text(&out)
    );
    assert!(
        !root.join("packages/engine/types/index.d.ts").exists(),
        "nothing is written for an invalid IDL"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Project commands outside a project name the fix and do nothing; usage
/// and unknown commands exit 2. (A real cook and playtest of the template,
/// which needs the native addon, is `packages/engine/tests/template.test.ts`.)
#[test]
fn project_commands_without_a_project_name_the_fix() {
    let empty = scratch_root();
    for cmd in ["build", "cook", "test"] {
        let out = og().arg(cmd).current_dir(&empty).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "og {cmd}");
        let t = text(&out);
        assert!(t.contains("no project.ogt"), "og {cmd}: {t}");
        assert!(t.contains("npm create @origingame/game"), "og {cmd}: {t}");
        assert!(!empty.join("cooked").exists() && !empty.join("dist").exists());
    }
    let probe = og().arg("probe").output().unwrap();
    assert_eq!(probe.status.code(), Some(2));
    assert!(
        text(&probe).contains("usage: og probe <method>"),
        "{}",
        text(&probe)
    );

    let missing = og()
        .args(["run", "--headless", "/nonexistent/game.ts", "--ticks", "1"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(
        text(&missing).contains("neither a game entry nor a project directory"),
        "{}",
        text(&missing)
    );

    let unknown = og().arg("frobnicate").output().unwrap();
    assert_eq!(unknown.status.code(), Some(2));
    assert!(text(&unknown).contains("unknown command `frobnicate`"));
    let _ = std::fs::remove_dir_all(&empty);
}

/// The pins as the repository's own files state them.
fn pins() -> (String, String) {
    let toolchain = std::fs::read_to_string(repo().join("rust-toolchain.toml")).unwrap();
    let rust = toolchain
        .lines()
        .find_map(|l| l.strip_prefix("channel = "))
        .unwrap()
        .trim_matches('"')
        .to_string();
    let lock = std::fs::read_to_string(repo().join("Cargo.lock")).unwrap();
    let at = lock.find("name = \"wasm-bindgen\"\n").unwrap();
    let bindgen = lock[at..]
        .lines()
        .nth(1)
        .unwrap()
        .trim_start_matches("version = ")
        .trim_matches('"')
        .to_string();
    (rust, bindgen)
}

#[test]
fn doctor_reports_the_pins_and_each_tool() {
    let (rust, _) = pins();
    let out = og().arg("doctor").current_dir(repo()).output().unwrap();
    let t = text(&out);
    assert!(
        t.contains(&format!("pinned      rust {rust} (rust-toolchain.toml)")),
        "{t}"
    );
    for tool in ["rustc", "cargo", "wasm-bindgen", "bun", "node"] {
        assert!(
            t.lines().any(|l| l.split_whitespace().nth(1) == Some(tool)),
            "no line for {tool}:\n{t}"
        );
    }
}

#[test]
fn doctor_names_a_fix_for_every_missing_tool() {
    let empty = std::env::temp_dir().join(format!("og-cli-empty-path-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let (rust, bindgen) = pins();
    let out = og()
        .arg("doctor")
        .current_dir(repo())
        .env("PATH", &empty)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir(&empty);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    let t = text(&out);
    let rustup = format!("rustup toolchain install {rust}");
    let cli = format!("cargo install wasm-bindgen-cli --version {bindgen} --locked");
    for (tool, fix) in [
        ("rustc", rustup.as_str()),
        ("cargo", rustup.as_str()),
        ("wasm-bindgen", cli.as_str()),
        ("bun", "https://bun.sh/install"),
        ("node", "https://nodejs.org"),
    ] {
        let at = t
            .lines()
            .position(|l| l.starts_with("missing") && l.split_whitespace().nth(1) == Some(tool))
            .unwrap_or_else(|| panic!("{tool} not reported missing:\n{t}"));
        let next = t.lines().nth(at + 1).unwrap_or_default();
        assert!(
            next.contains("fix:") && next.contains(fix),
            "{tool}: `{next}`"
        );
    }
    assert!(t.contains("5 problem(s)"), "{t}");
}

/// `og api gen --wiki` writes the wiki pages alone, one per module plus the
/// index, and removes a page whose module the IDL no longer declares.
#[test]
fn api_gen_wiki_writes_only_the_wiki_and_removes_stale_pages() {
    let root = scratch_root();
    let wiki = root.join("packages/knowledge/wiki");
    std::fs::create_dir_all(&wiki).unwrap();
    std::fs::write(wiki.join("retired.md"), "an old module's page\n").unwrap();
    let out = og()
        .args(["api", "gen", "--wiki", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    assert!(!wiki.join("retired.md").exists(), "{}", text(&out));
    let idl = std::fs::read_to_string(root.join(og_api_path())).unwrap();
    let modules: Vec<&str> = idl
        .lines()
        .filter_map(|l| l.strip_prefix("module ")?.strip_suffix(" {"))
        .collect();
    let mut pages: Vec<String> = std::fs::read_dir(&wiki)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    pages.sort();
    let mut want: Vec<String> = modules.iter().map(|m| format!("{m}.md")).collect();
    want.push("index.md".into());
    want.sort();
    assert_eq!(pages, want);
    // Nothing outside the wiki was written.
    assert!(!root.join("packages/engine").exists());
    assert!(!root.join("crates/og-host-web").exists());
    let _ = std::fs::remove_dir_all(&root);
}
