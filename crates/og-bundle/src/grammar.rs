//! Shared line-table validation for the remaining internal text formats.

pub use og_core::ogt::line::Line;

/// Validate a file block against its published field vocabulary.
pub fn check(block: &og_core::ogt::Block, fields: &[Line], at: &str) -> Result<(), String> {
    for e in &block.entries {
        if !fields.iter().any(|f| f.key == e.key) {
            return Err(format!("line {}: {at} has no '{}'", e.line, e.key));
        }
    }
    Ok(())
}

/// The row matching a key and element label.
pub fn find<'a>(lines: &'a [Line], key: &str, labels: &[&str]) -> Option<&'a Line> {
    lines.iter().find(|l| {
        l.key == key && (l.within.is_empty() || l.within.iter().any(|w| labels.contains(w)))
    })
}

/// Accepted keys for an element label, each once, for a diagnostic.
pub fn keys(lines: &[Line], labels: &[&str]) -> String {
    let mut out: Vec<&str> = Vec::new();
    for l in lines {
        if (l.within.is_empty() || l.within.iter().any(|w| labels.contains(w)))
            && !out.contains(&l.key)
        {
            out.push(l.key);
        }
    }
    out.join(", ")
}
