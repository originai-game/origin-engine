//! The rule the engine's built-in documents (`packages/engine/src/ui`)
//! keep so a style pack can restyle them by tokens alone: a document's
//! `<style>` writes no literal colour, length, duration or `url()` (the
//! pack rule of [`super::literals`], with no `:root` to hold values), its
//! markup writes no literal colour (`#hex` or a colour function in an
//! attribute or a binding; a colour a game passes in a prop is the game's), and
//! every part its markup names is listed in its `<meta name="parts">`, which
//! lists nothing else.

use std::collections::BTreeSet;
use std::path::Path;

/// The findings of every `.svelte` under `directory`, `<file>:<line>: …` or
/// `<file>: …` each.
pub fn findings(directory: &Path, out: &mut Vec<String>) -> Result<(), String> {
    let mut entries: Vec<_> = std::fs::read_dir(directory)
        .map_err(|e| format!("{}: {e}", directory.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            findings(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "svelte") {
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            document(&path.to_string_lossy().replace('\\', "/"), &text, out)?;
        }
    }
    Ok(())
}

fn document(file: &str, text: &str, out: &mut Vec<String>) -> Result<(), String> {
    const META: &str = "<meta name=\"parts\" content=\"";
    let declared: BTreeSet<&str> = text
        .find(META)
        .map(|at| &text[at + META.len()..])
        .and_then(|rest| rest.split_once('"'))
        .map(|(list, _)| list.split_whitespace().collect())
        .unwrap_or_default();
    let authored = og_ui::doc::schema::parts::authored_parts(file, text)?;
    let authored: BTreeSet<&str> = authored.iter().map(|(_, part)| part.as_str()).collect();
    for part in authored.difference(&declared) {
        out.push(format!(
            "{file}: part \"{part}\" is not listed in <meta name=\"parts\">"
        ));
    }
    // A bound `part={…}` names parts this reading cannot see.
    if !text.contains("part={") {
        for part in declared.difference(&authored) {
            out.push(format!(
                "{file}: <meta name=\"parts\"> lists \"{part}\", which no element names"
            ));
        }
    }
    let markup = text
        .split_once("<style>")
        .map_or(text, |(markup, _)| markup);
    for (index, line) in markup.lines().enumerate() {
        // A meta's fixtures and docs are data, not dressing.
        if line.trim_start().starts_with("<meta ") {
            continue;
        }
        if let Some(colour) = markup_colour(line) {
            out.push(format!(
                "{file}:{}: literal colour `{colour}` in markup; set it in <style> from a token",
                index + 1
            ));
        }
    }
    if let Some((before, rest)) = text.split_once("<style>") {
        let css = rest.split_once("</style>").map_or(rest, |(css, _)| css);
        // Blank lines stand for the markup, so a finding names the file's line.
        let padded = format!("{}{css}", "\n".repeat(before.matches('\n').count()));
        out.extend(
            super::literals::findings(file, &padded)
                .into_iter()
                .map(|f| {
                    f.replace(
                        "outside :root; set it as a :root token and use var()",
                        "in a built-in document; name a token with var()",
                    )
                }),
        );
    }
    Ok(())
}

/// The first literal colour a markup line writes: `#` and 3, 4, 6 or 8 hex
/// digits that is not a fragment reference (`href="#Face"`, `url(#Bed)`), or
/// a colour function.
fn markup_colour(line: &str) -> Option<&str> {
    for (at, _) in line.match_indices('#') {
        let digits = line[at + 1..]
            .bytes()
            .take_while(u8::is_ascii_alphanumeric)
            .count();
        let word = &line[at..at + 1 + digits];
        let reference = line[..at].ends_with("href=\"") || line[..at].ends_with("url(");
        if matches!(digits, 3 | 4 | 6 | 8)
            && word[1..].bytes().all(|b| b.is_ascii_hexdigit())
            && !reference
        {
            return Some(word);
        }
    }
    [
        "rgb(",
        "rgba(",
        "hsl(",
        "hsla(",
        "oklch(",
        "oklab(",
        "color-mix(",
    ]
    .into_iter()
    .find_map(|function| {
        line.find(function)
            .map(|at| &line[at..at + function.len() - 1])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ways it could fail: a literal in a style block missed or reported on
    /// the wrong line, a markup colour missed or a fragment reference taken for
    /// one, an unlisted part or a stale listing missed, a bound
    /// part reported as stale, or a token value reported.
    #[test]
    fn a_document_reports_literals_and_parts_out_of_step_with_its_meta() {
        let mut out = Vec::new();
        document(
            "a.svelte",
            "<meta name=\"component\" content=\"A\">\n<meta name=\"parts\" content=\"A.frame A.gone\">\n<div part=\"A.frame\"><p part=\"A.row\">x</p></div>\n<svg><use href=\"#Face\"></use><rect fill=\"#ffffffb3\"></rect></svg>\n<style>\n.a { gap: var(--space-sm); width: 50%; }\n.b { padding: 4px; }\n</style>\n",
            &mut out,
        )
        .unwrap();
        assert_eq!(
            out,
            [
                "a.svelte: part \"A.row\" is not listed in <meta name=\"parts\">",
                "a.svelte: <meta name=\"parts\"> lists \"A.gone\", which no element names",
                "a.svelte:4: literal colour `#ffffffb3` in markup; set it in <style> from a token",
                "a.svelte:7: padding: literal length `4px` in a built-in document; name a token with var()",
            ]
        );
        let mut out = Vec::new();
        document(
            "b.svelte",
            "<meta name=\"component\" content=\"B\">\n<meta name=\"parts\" content=\"B.frame\">\n<div part={\"B.frame[x=\" + x + \"]\"}></div>\n",
            &mut out,
        )
        .unwrap();
        assert_eq!(out, [] as [&str; 0]);
    }

    /// The engine's own documents keep the rule.
    #[test]
    fn built_in_documents_hold_no_literals_and_list_their_parts() {
        let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui");
        let mut out = Vec::new();
        findings(&ui, &mut out).unwrap();
        assert!(
            out.is_empty(),
            "{} finding(s):\n{}",
            out.len(),
            out.join("\n")
        );
    }
}
