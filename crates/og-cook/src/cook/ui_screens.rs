//! project.ogt's `ui` block (docs/ui.md §6): the screens `og ui check`
//! and `og ui preview` visit when none is named, each a name
//! and the `og run --commands` script that reaches it:
//!
//! ```text
//! ui {
//!   screen "hud" "0:key:Enter"
//!   screen "shop" "0:key:Enter;30:key:KeyS"
//! }
//! ```
//!
//! The cook refuses a malformed block, like a malformed `conform`, so a
//! screen list that would silently check nothing never ships.

use og_core::ogt::Block;

/// One declared screen: its name and the commands that reach it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screen {
    pub name: String,
    pub commands: String,
}

const SHAPE: &str = "screen \"<name>\" \"<commands>\", e.g. screen \"hud\" \"0:key:Enter\"";

/// The screens of project.ogt's `ui` block, in order; empty without one.
/// An error names the line and the fix.
pub fn screens(body: &Block) -> Result<Vec<Screen>, String> {
    let Some(e) = body.get("ui") else {
        return Ok(Vec::new());
    };
    let Some(block) = &e.block else {
        return Err(format!("{}: ui is a block: ui {{ {SHAPE} }}", e.line));
    };
    let mut out: Vec<Screen> = Vec::new();
    for e in &block.entries {
        if e.key != "screen" {
            return Err(format!(
                "{}: ui has no `{}`; it takes {SHAPE}",
                e.line, e.key
            ));
        }
        if e.block.is_some() {
            return Err(format!("{}: ui's screen is one line: {SHAPE}", e.line));
        }
        let text = |i: usize| e.values.get(i).and_then(|v| v.as_str()).map(str::trim);
        let name = match text(0) {
            Some(n) if !n.is_empty() => n,
            _ => {
                return Err(format!(
                    "{}: ui's screen has no name; write {SHAPE}",
                    e.line
                ));
            }
        };
        let commands = match text(1) {
            Some(c) if !c.is_empty() => c,
            _ => {
                return Err(format!(
                    "{}: ui's screen \"{name}\" has no commands; write the og run --commands script that reaches it: screen \"{name}\" \"0:key:Enter\"",
                    e.line
                ));
            }
        };
        if e.values.len() > 2 {
            return Err(format!(
                "{}: ui's screen \"{name}\" takes a name and one commands string; join the commands with `;` in one string",
                e.line
            ));
        }
        if out.iter().any(|s| s.name == name) {
            return Err(format!(
                "{}: ui declares screen \"{name}\" twice; rename one or remove it",
                e.line
            ));
        }
        out.push(Screen {
            name: name.to_string(),
            commands: commands.to_string(),
        });
    }
    Ok(out)
}
