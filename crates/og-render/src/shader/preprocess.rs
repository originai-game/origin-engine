//! Lightweight WGSL preprocessor: `//#import <name>` dependency resolution,
//! `//#if` / `//#ifdef` / `//#ifndef` / `//#else` / `//#endif` conditional
//! compilation, and source line mapping for diagnostics.
//!
//! Kept zero-dependency and deterministic so both build-time validation and
//! runtime module assembly can compose shader fragments without manual string
//! concatenation or duplicate definitions.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

/// A resolved source span in the preprocessed output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceSegment {
    /// 1-indexed first line in the preprocessed output.
    pub out_start: usize,
    /// Original module/file name.
    pub file: String,
    /// 1-indexed line in the original file corresponding to `out_start`.
    pub src_start: usize,
    /// Number of contiguous lines from `file`.
    pub line_count: usize,
}

/// Maps 1-indexed line numbers in preprocessed WGSL back to `(file, line)`.
#[derive(Clone, Debug, Default)]
pub(crate) struct LineMap {
    segments: Vec<SourceSegment>,
}

impl LineMap {
    fn push(&mut self, out_line: usize, file: &str, src_line: usize) {
        if let Some(last) = self.segments.last_mut()
            && last.file == file
            && last.out_start + last.line_count == out_line
            && last.src_start + last.line_count == src_line
        {
            last.line_count += 1;
            return;
        }
        self.segments.push(SourceSegment {
            out_start: out_line,
            file: file.to_string(),
            src_start: src_line,
            line_count: 1,
        });
    }

    /// Resolves a 1-indexed output line to `(original_file, original_line)`.
    pub(crate) fn resolve(&self, out_line: usize) -> Option<(&str, usize)> {
        let idx = self
            .segments
            .partition_point(|seg| seg.out_start <= out_line);
        let seg = self.segments.get(idx.checked_sub(1)?)?;
        let offset = out_line - seg.out_start;
        if offset < seg.line_count {
            Some((&seg.file, seg.src_start + offset))
        } else {
            None
        }
    }
}

/// Preprocessed WGSL text paired with its original-source line map.
#[derive(Clone, Debug)]
pub(crate) struct Preprocessed {
    pub wgsl: String,
    pub lines: LineMap,
}

/// Preprocessor error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PreprocessError {
    UnknownImport {
        from: String,
        line: usize,
        target: String,
    },
    CircularImport {
        chain: Vec<String>,
    },
    UnbalancedCondition {
        file: String,
        line: usize,
        directive: &'static str,
    },
}

impl std::fmt::Display for PreprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreprocessError::UnknownImport { from, line, target } => {
                write!(f, "{from}:{line}: unknown //#import `{target}`")
            }
            PreprocessError::CircularImport { chain } => {
                write!(f, "circular //#import: {}", chain.join(" -> "))
            }
            PreprocessError::UnbalancedCondition {
                file,
                line,
                directive,
            } => write!(f, "{file}:{line}: unbalanced {directive}"),
        }
    }
}

impl std::error::Error for PreprocessError {}

/// Preprocesses `root_name` using `sources` as the named module lookup table
/// and `defines` as the active boolean compile-time flags.
pub(crate) fn preprocess(
    root_name: &str,
    sources: &HashMap<&str, &str>,
    defines: &HashSet<&str>,
) -> Result<Preprocessed, PreprocessError> {
    let mut out = String::new();
    let mut lines = LineMap::default();
    let mut out_line = 1usize;
    let mut included = HashSet::<String>::new();
    let mut stack = Vec::<String>::new();
    expand(
        root_name,
        sources,
        defines,
        &mut included,
        &mut stack,
        &mut out,
        &mut lines,
        &mut out_line,
    )?;
    Ok(Preprocessed { wgsl: out, lines })
}

#[allow(clippy::too_many_arguments)]
fn expand(
    name: &str,
    sources: &HashMap<&str, &str>,
    defines: &HashSet<&str>,
    included: &mut HashSet<String>,
    stack: &mut Vec<String>,
    out: &mut String,
    lines: &mut LineMap,
    out_line: &mut usize,
) -> Result<(), PreprocessError> {
    if stack.iter().any(|s| s == name) {
        let mut chain = stack.clone();
        chain.push(name.to_string());
        return Err(PreprocessError::CircularImport { chain });
    }
    if !included.insert(name.to_string()) {
        return Ok(());
    }
    let Some(&text) = sources.get(name) else {
        return Err(PreprocessError::UnknownImport {
            from: stack.last().cloned().unwrap_or_else(|| "<root>".into()),
            line: 1,
            target: name.to_string(),
        });
    };
    stack.push(name.to_string());

    // Conditional stack: (parent_active, branch_active, else_seen)
    let mut cond: Vec<(bool, bool, bool)> = Vec::new();
    let active = |c: &[(bool, bool, bool)]| c.last().is_none_or(|&(p, b, _)| p && b);

    for (idx, raw_line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw_line.trim();
        if let Some(rest) = trimmed.strip_prefix("//#") {
            let rest = rest.trim();
            if let Some(flag) = rest
                .strip_prefix("ifdef ")
                .or_else(|| rest.strip_prefix("if "))
            {
                let parent = active(&cond);
                let flag = flag.trim();
                let cond_val = if let Some(neg) = flag.strip_prefix('!') {
                    !defines.contains(neg.trim())
                } else {
                    defines.contains(flag)
                };
                cond.push((parent, cond_val, false));
                continue;
            }
            if let Some(flag) = rest.strip_prefix("ifndef ") {
                let parent = active(&cond);
                let cond_val = !defines.contains(flag.trim());
                cond.push((parent, cond_val, false));
                continue;
            }
            if rest == "else" {
                let Some(top) = cond.last_mut() else {
                    return Err(PreprocessError::UnbalancedCondition {
                        file: name.to_string(),
                        line: line_no,
                        directive: "//#else",
                    });
                };
                if top.2 {
                    return Err(PreprocessError::UnbalancedCondition {
                        file: name.to_string(),
                        line: line_no,
                        directive: "//#else",
                    });
                }
                top.1 = !top.1;
                top.2 = true;
                continue;
            }
            if rest == "endif" {
                if cond.pop().is_none() {
                    return Err(PreprocessError::UnbalancedCondition {
                        file: name.to_string(),
                        line: line_no,
                        directive: "//#endif",
                    });
                }
                continue;
            }
            if let Some(target) = rest.strip_prefix("import ") {
                if active(&cond) {
                    let target = target.trim().trim_matches('"');
                    if !sources.contains_key(target) {
                        return Err(PreprocessError::UnknownImport {
                            from: name.to_string(),
                            line: line_no,
                            target: target.to_string(),
                        });
                    }
                    expand(
                        target, sources, defines, included, stack, out, lines, out_line,
                    )?;
                }
                continue;
            }
        }
        if active(&cond) {
            out.push_str(raw_line);
            out.push('\n');
            lines.push(*out_line, name, line_no);
            *out_line += 1;
        }
    }

    if !cond.is_empty() {
        return Err(PreprocessError::UnbalancedCondition {
            file: name.to_string(),
            line: text.lines().count().max(1),
            directive: "//#if",
        });
    }
    stack.pop();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_imports_and_maps_lines() {
        let mut sources = HashMap::new();
        sources.insert("common.wgsl", "const A: f32 = 1.0;\nconst B: f32 = 2.0;\n");
        sources.insert(
            "helper.wgsl",
            "//#import common.wgsl\nfn h() -> f32 { return A; }\n",
        );
        sources.insert(
            "main.wgsl",
            "//#import common.wgsl\n//#import helper.wgsl\n//#if FOO\nfn f() -> f32 { return B; }\n//#else\nfn f() -> f32 { return 0.0; }\n//#endif\n",
        );
        let defines = HashSet::from(["FOO"]);
        let out = preprocess("main.wgsl", &sources, &defines).unwrap();
        assert_eq!(
            out.wgsl,
            "const A: f32 = 1.0;\nconst B: f32 = 2.0;\nfn h() -> f32 { return A; }\nfn f() -> f32 { return B; }\n"
        );
        assert_eq!(out.lines.resolve(1), Some(("common.wgsl", 1)));
        assert_eq!(out.lines.resolve(2), Some(("common.wgsl", 2)));
        assert_eq!(out.lines.resolve(3), Some(("helper.wgsl", 2)));
        assert_eq!(out.lines.resolve(4), Some(("main.wgsl", 4)));
    }

    #[test]
    fn detects_circular_imports() {
        let mut sources = HashMap::new();
        sources.insert("a.wgsl", "//#import b.wgsl\n");
        sources.insert("b.wgsl", "//#import a.wgsl\n");
        let err = preprocess("a.wgsl", &sources, &HashSet::new()).unwrap_err();
        assert_eq!(
            err,
            PreprocessError::CircularImport {
                chain: vec!["a.wgsl".into(), "b.wgsl".into(), "a.wgsl".into()]
            }
        );
    }
}
