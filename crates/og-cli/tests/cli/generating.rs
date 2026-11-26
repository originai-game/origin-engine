use super::*;

/// The public offline executor: dependencies and parameters invalidate a
/// content key, moving a project does not, and ambient authority is absent.
#[test]
fn script_tracks_its_closed_inputs_and_rejects_ambient_authority() {
    let root = TempDir::new("og-script");
    let run = |name: &str, params: &str| {
        og().args([
            "script",
            name,
            "--params",
            params,
            "--input",
            "bytes=input.bin",
            "--json",
        ])
        .current_dir(&root)
        .output()
        .unwrap()
    };
    let value = |out: &Output| -> serde_json::Value {
        assert!(out.status.success(), "{}", text(out));
        serde_json::from_slice(&out.stdout).unwrap()
    };
    std::fs::write(root.join("input.bin"), [7, 11]).unwrap();
    std::fs::write(root.join("shape.ts"), "export const offset: number = 3;\n").unwrap();
    std::fs::write(root.join("main.ts"), r#"
        import {offset} from './shape.ts';
        export default ({params, inputs}: {params: {scale: number}; inputs: Record<string, number[]>}) =>
            new Float32Array([offset * params.scale, inputs.bytes[1], Math.sin(0.7), 2 ** 0.3]);
    "#).unwrap();
    let first = value(&run("main.ts", "{\"scale\":2}"));
    assert_eq!(first["data"][0], 6.0);
    assert_eq!(first["data"][1], 11.0);
    assert_eq!(first, value(&run("main.ts", "{\"scale\":2}")));
    assert_eq!(first, value(&run("main.ts", "{\"scale\":2.0}")));
    assert_ne!(first["key"], value(&run("main.ts", "{\"scale\":4}"))["key"]);
    std::fs::write(root.join("shape.ts"), "export const offset: number = 5;\n").unwrap();
    let changed = value(&run("main.ts", "{\"scale\":2}"));
    assert_ne!(first["key"], changed["key"]);
    assert_eq!(changed["data"][0], 10.0);
    std::fs::write(root.join("input.bin"), [7, 13]).unwrap();
    let input = value(&run("main.ts", "{\"scale\":2}"));
    assert_ne!(changed["key"], input["key"]);
    assert_eq!(input["data"][1], 13.0);
    let moved = TempDir::new("og-script-moved");
    for file in ["main.ts", "shape.ts", "input.bin"] {
        std::fs::copy(root.join(file), moved.join(file)).unwrap();
    }
    let output = og()
        .args([
            "script",
            "main.ts",
            "--params",
            "{\"scale\":2}",
            "--input",
            "bytes=input.bin",
            "--json",
        ])
        .current_dir(&moved)
        .output()
        .unwrap();
    assert_eq!(input, value(&output));

    for code in [
        "export default () => Date.now()",
        "export default () => Math.random()",
        "export default () => fetch('https://example.invalid')",
        "export default () => process.env.HOME",
        "import fs from 'node:fs'; export default () => fs.readFileSync('/etc/passwd')",
        "export default () => (()=>{}).constructor('return 1')()",
        "export default () => eval('2 ** 0.3')",
        "export default async () => [1]",
        "export default () => [Infinity]",
        "export default () => { while (true) {} }",
    ] {
        std::fs::write(root.join("denied.ts"), code).unwrap();
        let out = run("denied.ts", "{}");
        assert!(
            !out.status.success(),
            "accepted forbidden generator {code}: {}",
            text(&out)
        );
        assert!(out.stdout.is_empty(), "partial result for {code}");
    }
    let out = run("../outside.ts", "{}");
    assert!(!out.status.success());
    assert!(text(&out).contains("leaves its input root"));
}
