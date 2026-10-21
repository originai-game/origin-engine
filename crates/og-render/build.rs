//! The WGSL the renderer compiles, as it ships: every file of `shaders/`
//! copied to `$OUT_DIR/shaders/` with its `//` comments removed, which the
//! `wgsl!` macro (src/lib.rs) includes in place of the file itself.
//!
//! A comment runs from `//` to the end of its line (WGSL has no string
//! literals, and these files use no block comments: a `/*` in a line's
//! code fails the build), so removing it, and the spaces before it, leaves
//! every token of the module in place and every line where it was: naga
//! parses the same module, and its messages name the same lines. The
//! comments were about 100 kB of the lanes wasm and 27 kB of its brotli
//! (tests/evidence/P4/wasm-size/README.md).

use std::path::Path;

fn main() {
    let dir = Path::new("shaders");
    println!("cargo:rerun-if-changed=shaders");
    let out = Path::new(&std::env::var("OUT_DIR").expect("cargo sets OUT_DIR")).join("shaders");
    std::fs::create_dir_all(&out).expect("OUT_DIR/shaders");
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .expect("og-render's shaders/")
        .map(|e| e.expect("a shaders/ entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "wgsl"))
        .collect();
    entries.sort();
    for path in entries {
        let src = std::fs::read_to_string(&path).expect("a WGSL file");
        let stripped = strip(&src).unwrap_or_else(|line| {
            panic!(
                "{}:{line}: a block comment; the WGSL build step (build.rs) removes only `//` comments",
                path.display()
            )
        });
        let name = path.file_name().expect("a file name");
        std::fs::write(out.join(name), stripped).expect("a stripped WGSL file");
    }
}

/// `src` without its `//` comments and the spaces before them; every line
/// stays, so a comment line is left empty. Err(line) where a line's code
/// opens a block comment.
fn strip(src: &str) -> Result<String, usize> {
    let mut out = String::with_capacity(src.len());
    for (n, line) in src.split_inclusive('\n').enumerate() {
        let (body, end) = match line.strip_suffix('\n') {
            Some(b) => (b, "\n"),
            None => (line, ""),
        };
        let code = match body.find("//") {
            Some(at) => body[..at].trim_end(),
            None => body,
        };
        if code.contains("/*") {
            return Err(n + 1);
        }
        out.push_str(code);
        out.push_str(end);
    }
    Ok(out)
}
