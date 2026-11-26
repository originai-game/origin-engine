//! The installed engine's pure schema module, not a cook-only implementation
//! of its record/column factories. Only these four trusted sources are lowered.
use super::*;
use std::sync::OnceLock;

pub const MODULE: &str = "@origingame/engine/schema";

pub fn modules() -> Result<&'static BTreeMap<String, String>, String> {
    static MODULES: OnceLock<Result<BTreeMap<String, String>, String>> = OnceLock::new();
    MODULES
        .get_or_init(|| {
            let mut modules = BTreeMap::new();
            for (path, text) in [
                (
                    MODULE,
                    include_str!("../../../../../packages/engine/src/schema.ts"),
                ),
                (
                    "@origingame/engine/records.ts",
                    include_str!("../../../../../packages/engine/src/records.ts"),
                ),
                (
                    "@origingame/engine/generated/exports.ts",
                    include_str!("../../../../../packages/engine/src/generated/exports.ts"),
                ),
                (
                    "@origingame/engine/generated/consts.ts",
                    include_str!("../../../../../packages/engine/src/generated/consts.ts"),
                ),
            ] {
                let allocator = Allocator::default();
                let parsed = Parser::new(&allocator, text, SourceType::ts()).parse();
                if !parsed.diagnostics.is_empty() {
                    return Err(format!("{path}: {:?}", parsed.diagnostics));
                }
                let mut program = parsed.program;
                let semantics = SemanticBuilder::new().build(&program);
                if !semantics.diagnostics.is_empty() {
                    return Err(format!("{path}: {:?}", semantics.diagnostics));
                }
                let transformed =
                    Transformer::new(&allocator, Path::new(path), &TransformOptions::default())
                        .build_with_scoping(semantics.semantic.into_scoping(), &mut program);
                if !transformed.diagnostics.is_empty() {
                    return Err(format!("{path}: {:?}", transformed.diagnostics));
                }
                modules.insert(path.into(), Codegen::new().build(&program).code);
            }
            Ok(modules)
        })
        .as_ref()
        .map_err(Clone::clone)
}

pub fn builtin(path: &str) -> bool {
    path == BUILTIN
        || path == author::MODULE
        || path == MODULE
        || path.starts_with("@origingame/engine/generated/")
        || path == "@origingame/engine/records.ts"
}
