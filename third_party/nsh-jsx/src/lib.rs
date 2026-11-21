//! NSH's JSX compiler and class check, for Origin Engine's `og build` and
//! `og check` (PATCHED.md lists what differs from NSH).

mod classes;
mod compile;
mod hot;
mod jsx;
mod props;

pub use nsh_tailwind::tailwind::Theme;

/// A `.tsx`/`.jsx`/`.ts` module compiled to JS: TypeScript removed, the JSX
/// compiled to the renderer's calls (`@origingame/engine/ui/rt`).
pub struct Module {
    pub code: String,
    /// The source map (JSON) to the module's own text.
    pub map: String,
    /// Warnings: components that take their props apart, which loses
    /// reactivity.
    pub warnings: Vec<String>,
}

/// Compiles `source`, the module at `path` (its extension picks TypeScript
/// and JSX). An error is rendered with a code frame.
pub fn compile(path: &str, source: &str) -> Result<Module, String> {
    let p = std::path::Path::new(path);
    let source_type = oxc::span::SourceType::from_path(p).map_err(|e| format!("{path}: {e}"))?;
    let modules = compile::NativeModules::default();
    let out = compile::compile_native(p, source_type, source, None, None, None, &modules)
        .map_err(|e| compile::strip_ansi(&e))?;
    Ok(Module {
        code: out.code,
        map: out.map.to_json_string(),
        warnings: out.destructured.iter().map(|d| d.message()).collect(),
    })
}

/// One class in a JSX `class`/`classList` that `theme` does not know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownClass {
    /// 1-based line and column.
    pub line: usize,
    pub column: usize,
    pub class: String,
}

/// The unknown classes of `source`'s JSX (the module at `path`): the
/// static parts of each `class` and `classList`.
pub fn unknown_classes(path: &str, source: &str, theme: &Theme) -> Vec<UnknownClass> {
    let Ok(source_type) = oxc::span::SourceType::from_path(path) else {
        return Vec::new();
    };
    classes::check(source, source_type, theme)
        .into_iter()
        .map(|u| UnknownClass {
            line: u.line,
            column: u.column,
            class: u.class,
        })
        .collect()
}
