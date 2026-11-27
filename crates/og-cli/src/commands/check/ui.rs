//! UI checks run by `og check`: Tailwind class resolution (OG9105),
//! missing companion declarations (OG9106), pack grounds (OG9107), and
//! component ground/ink lints (OG9108–OG9110).

use super::*;

/// The grounds and inks the TSX under `dirs` writes by hand
/// (`ui_grounds`), against the default pack under the game's own: printed
/// as warnings (`<file>(<line>,<col>): warning OG9108:`) or, with `json`,
/// as one `ui-grounds` event. Never fails a check.
pub(super) fn ui_grounds_lint(root: &Path, dirs: &[&Path], game: Option<&Path>, json: bool) {
    use og_ui_next::pack::{Layer, Pack, Scheme};
    let mut layers = vec![Layer::default_pack()];
    if let Some(layer) = game
        .and_then(|g| std::fs::read_to_string(g).ok())
        .and_then(|t| Layer::parse(&t).ok())
    {
        layers.push(layer);
    }
    // The game's theme recipe (`styles/game/theme.json`): its layer, and its
    // art slots as the grounds a child's ink is checked on (OG9107). One
    // that does not compile is the cook's error.
    let theme = game
        .map(|g| g.with_file_name(og_cook::cook::ui_themes::RECIPE))
        .and_then(|r| std::fs::read_to_string(&r).ok())
        .and_then(|t| og_cook::cook::ui_themes::compile(&t, "styles/game/theme.json").ok());
    if let Some(layer) = theme
        .as_ref()
        .and_then(|t| t.layer(Default::default()).ok())
    {
        layers.push(layer);
    }
    let art = theme
        .as_ref()
        .and_then(|t| crate::ui_grounds::ArtGrounds::of(t).ok());
    let pack = Pack::new(&layers);
    let colors = crate::ui_grounds::colors(&Scheme::ALL.map(|s| pack.theme_json(s)));
    // The pack's own scheme first: a colour written by hand is a role's
    // value in the scheme the UI was drawn in.
    let schemes = match pack.scheme {
        Some(Scheme::Light) => [Scheme::Light, Scheme::Dark],
        _ => [Scheme::Dark, Scheme::Light],
    };
    let scale = crate::ui_values::Scale::new(&schemes.map(|s| pack.theme_json(s)));
    let found = crate::ui_grounds::lint(root, dirs, &colors, &scale, art.as_ref());
    if found.is_empty() {
        return;
    }
    if json {
        let event = serde_json::json!({
            "event": "ui-grounds",
            "warnings": found.iter().map(|(file, f)| serde_json::json!({
                "file": file, "line": f.line, "col": f.col, "code": f.code, "class": f.class, "message": f.message,
                "fix": f.fix.as_ref().map(|(classes, _)| classes), "exact": f.fix.as_ref().map(|(_, exact)| exact),
            })).collect::<Vec<_>>(),
        });
        println!("{event}");
        return;
    }
    for (file, f) in &found {
        println!(
            "{file}({},{}): warning OG{}: {}",
            f.line, f.col, f.code, f.message
        );
    }
    let count = |code: u32| {
        let of: Vec<&str> = found
            .iter()
            .filter(|(_, f)| f.code == code)
            .map(|(file, _)| file.as_str())
            .collect();
        let mut files = of.clone();
        files.dedup();
        (of.len(), files.len())
    };
    let (grounds, in_files) = count(crate::ui_grounds::CODE);
    if grounds > 0 {
        println!(
            "og check: {grounds} ground or ink warning(s) in {in_files} file(s): text sits on a pack slot or an opaque theme ground, in that ground's ink (crates/og-ui-next/CONVENTIONS.md §Theme and packs, Grounds and inks)"
        );
    }
    let (values, in_files) = count(crate::ui_values::CODE);
    if values > 0 {
        println!(
            "og check: {values} hand-written value warning(s) in {in_files} file(s): a size, a colour, a shadow or a strength is a theme step or role, never a bracket, a literal or an opacity modifier (crates/og-ui-next/CONVENTIONS.md §Theme and packs, Steps and roles)"
        );
    }
}

fn class_resolver(pack: &og_ui_next::pack::Pack) -> Result<og_ui_next::classes::Classes, String> {
    use og_ui_next::{
        classes::{Classes, nine_slice},
        pack::Scheme,
    };
    let (mut classes, _) = Classes::new(Some(&pack.theme_json(Scheme::Dark)))?;
    classes.set_variants(pack.variants().clone());
    classes.set_images(
        [Scheme::Light, Scheme::Dark]
            .into_iter()
            .flat_map(|s| {
                pack.images(s)
                    .iter()
                    .map(|(n, i)| (n.clone(), nine_slice(i, 0)))
            })
            .collect(),
    );
    Ok(classes)
}

/// Loads a style directory's layers: its compiled `theme.json`, then its
/// `pack.json` over it, as the cook lays them (either may be absent).
/// Returns `Err(())` when an existing file is malformed (deferring diagnostics to `og cook`).
fn push_style_dir(
    layers: &mut Vec<og_ui_next::pack::Layer>,
    dir: &Path,
    label: &str,
) -> Result<(), ()> {
    use og_ui_next::pack::Layer;
    let recipe_path = dir.join(og_cook::cook::ui_themes::RECIPE);
    if let Ok(text) = std::fs::read_to_string(&recipe_path) {
        let theme = og_cook::cook::ui_themes::compile(&text, label).map_err(|_| ())?;
        let layer = theme.layer(Default::default()).map_err(|_| ())?;
        layers.push(layer);
    }
    let pack_path = dir.join("pack.json");
    if let Ok(text) = std::fs::read_to_string(&pack_path) {
        let layer = Layer::parse(&text).map_err(|_| ())?;
        layers.push(layer);
    }
    Ok(())
}

/// Resolves the UI pack layers active for `project_root`: default pack, the
/// look's `ui` style or theme (from `project.ogt`'s `look`), and `styles/game`.
fn project_style_layers(project_root: &Path) -> Result<Vec<og_ui_next::pack::Layer>, ()> {
    use og_ui_next::pack::Layer;
    let mut layers = vec![Layer::default_pack()];
    let assets = project_root.join("assets");
    let look_style = og_cook::cook::look::project_ui_style(project_root).map_err(|_| ())?;
    if let Some(style_id) = look_style.as_deref() {
        if let Some(theme_name) = style_id.strip_prefix("@themes/") {
            if let Some(recipe_path) = og_cook::cook::ui_themes::builtin_recipe(theme_name)
                && let Ok(text) = std::fs::read_to_string(&recipe_path)
            {
                let label = format!("@themes/{theme_name}/{}", og_cook::cook::ui_themes::RECIPE);
                let theme = og_cook::cook::ui_themes::compile(&text, &label).map_err(|_| ())?;
                let layer = theme.layer(Default::default()).map_err(|_| ())?;
                layers.push(layer);
            }
        } else if style_id.starts_with("@styles/") {
            if let Some((_, bytes)) = og_cook::cook::look::style_files(style_id)
                .into_iter()
                .find(|(id, _)| id.ends_with("/pack.json"))
                && let Ok(text) = std::str::from_utf8(bytes)
            {
                let layer = Layer::parse(text).map_err(|_| ())?;
                layers.push(layer);
            }
        } else {
            push_style_dir(
                &mut layers,
                &assets.join(style_id),
                &format!("{style_id}/{}", og_cook::cook::ui_themes::RECIPE),
            )?;
        }
    }
    if look_style.as_deref() != Some("styles/game") {
        push_style_dir(
            &mut layers,
            &assets.join("styles/game"),
            "styles/game/theme.json",
        )?;
    }
    Ok(layers)
}

/// Check against the default pack under the project look's theme/style and the
/// game's own pack, using the same class resolver as runtime rather than a
/// parallel parser contract.
pub(super) fn unknown_classes(p: &Project, report: &mut Report) {
    // The game's code and its UI components, where the template's
    // tsconfig reads TSX.
    let mut files = Vec::new();
    crate::lockstep_lint::sources(p.entry.parent().unwrap_or(&p.root), &mut files);
    crate::lockstep_lint::sources(&p.root.join("assets/ui"), &mut files);
    files.retain(|f| f.extension().is_some_and(|e| e == "tsx" || e == "jsx"));
    files.sort();
    files.dedup();
    if files.is_empty() {
        return;
    }
    let Ok(layers) = project_style_layers(&p.root) else {
        // The cook refuses the malformed pack/theme and says why.
        return;
    };
    let pack = og_ui_next::pack::Pack::new(&layers);
    let Ok(mut classes) = class_resolver(&pack) else {
        return;
    };
    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let file = typescript_path(path.strip_prefix(&p.root).unwrap_or(&path));
        for u in
            nsh_jsx::unknown_classes_with(&path.to_string_lossy(), &text, |s| classes.unknown(s))
        {
            let message = format!("unknown class `{}`", u.class);
            report.raw.push_str(&format!(
                "{file}({},{}): error OG{UNKNOWN_CLASS}: {message}\n",
                u.line, u.column
            ));
            report.diagnostics.push(Diagnostic {
                file: Some(file.clone()),
                line: Some(u.line as u32),
                col: Some(u.column as u32),
                code: UNKNOWN_CLASS,
                message,
            });
        }
    }
}

/// Each of the game's style packs (`assets/styles/<name>/pack.json`) whose
/// variants paint the ground under the text without its ink
/// (`og_ui_next::pack::grounds_without_ink`), laid over the default pack and
/// the style's own compiled `theme.json`.
pub(super) fn pack_grounds(p: &Project, report: &mut Report) {
    use og_ui_next::pack::{Layer, grounds_without_ink};
    let Ok(dirs) = std::fs::read_dir(p.root.join("assets/styles")) else {
        return;
    };
    let mut styles: Vec<_> = dirs
        .flatten()
        // A directory symlink is never followed.
        .filter(|d| d.file_type().is_ok_and(|t| t.is_dir()))
        .map(|d| d.path())
        .collect();
    styles.sort();
    for dir in styles {
        // The theme compiler solves its own inks; the pack over it is what
        // can paint a ground without one.
        let path = dir.join("pack.json");
        if !path.is_file() {
            continue;
        }
        let file = typescript_path(path.strip_prefix(&p.root).unwrap_or(&path));
        let mut layers = Vec::new();
        // One that does not parse is the cook's error.
        if push_style_dir(&mut layers, &dir, &file).is_err() {
            continue;
        }
        let mut under = vec![Layer::default_pack()];
        let Some(layer) = layers.pop() else {
            continue;
        };
        under.extend(layers);
        for message in grounds_without_ink(&under, &layer) {
            report
                .raw
                .push_str(&format!("{file}: error OG{PACK_GROUND_INK}: {message}\n"));
            report.diagnostics.push(Diagnostic {
                file: Some(file.clone()),
                line: None,
                col: None,
                code: PACK_GROUND_INK,
                message,
            });
        }
    }
}

/// The diagnostic code of a pack variant painting a ground without its ink.
const PACK_GROUND_INK: u32 = 9107;

/// The diagnostic code of an unknown class in a game's TSX.
const UNKNOWN_CLASS: u32 = 9105;

/// The diagnostic code of a string table a source imports that is not there.
const MISSING_TABLE: u32 = 9106;

/// Companion declarations a source imports for their effect (locale TS tables
/// or the not-yet-migrated built-in particle documents),
/// each with its 1-based line and column.
fn table_imports(text: &str) -> Vec<(u32, u32, &str)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        // A statement, not an import a comment or a string quotes.
        let Some(stmt) = line.trim_start().strip_prefix("import") else {
            continue;
        };
        let mut rest = stmt.trim_start();
        // A default import of the table (`import strings from "./x.en.strings.ts"`).
        if !rest.starts_with(['"', '\''])
            && let Some(at) = rest.find(" from ")
        {
            rest = rest[at + " from ".len()..].trim_start();
        }
        let Some(quote) = rest.chars().next().filter(|c| *c == '"' || *c == '\'') else {
            continue;
        };
        let body = &rest[1..];
        let Some(end) = body.find(quote) else {
            continue;
        };
        let spec = &body[..end];
        if spec.ends_with(".ogt") || spec.ends_with(".strings.ts") {
            let col = line.len() - rest.len() + 1;
            out.push((i as u32 + 1, col as u32 + 1, spec));
        }
    }
    out
}

/// Each companion a TS or TSX source under `dirs` imports relatively that does
/// not exist: an error naming the importer. The cook fails the whole asset
/// that bundles it (every component's table with it), so a bad import stops
/// at its author's check instead.
pub(super) fn missing_tables(root: &Path, dirs: &[PathBuf], report: &mut Report) {
    let mut files = Vec::new();
    for d in dirs {
        crate::lockstep_lint::sources(d, &mut files);
    }
    files.sort();
    files.dedup();
    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let dir = path.parent().unwrap_or(root);
        for (line, col, spec) in table_imports(&text) {
            if !spec.starts_with('.') || dir.join(spec).is_file() {
                continue;
            }
            let file = typescript_path(path.strip_prefix(root).unwrap_or(&path));
            let message = format!(
                "`{spec}` does not exist relative to this file: import a typed locale declaration under content/"
            );
            report.raw.push_str(&format!(
                "{file}({line},{col}): error OG{MISSING_TABLE}: {message}\n"
            ));
            report.diagnostics.push(Diagnostic {
                file: Some(file),
                line: Some(line),
                col: Some(col),
                code: MISSING_TABLE,
                message,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_check_uses_the_runtime_contract_without_accepting_typos() {
        use og_ui_next::pack::{Layer, Pack};
        let pack = Pack::new(&[Layer::default_pack()]);
        let mut classes = class_resolver(&pack).unwrap();
        let found = nsh_jsx::unknown_classes_with(
            "a.tsx",
            r#"<View class="scheme-dark scheme-light aspect-[3.2] data-active:text-white aspect-3.2 scheme-dakr" />"#,
            |s| classes.unknown(s),
        );
        assert_eq!(
            found.iter().map(|f| f.class.as_str()).collect::<Vec<_>>(),
            ["aspect-3.2", "scheme-dakr"]
        );
    }

    /// The engine's own TSX components (packages/engine/src/ui) use only
    /// classes the default pack's theme knows, as a game's must
    /// ([`unknown_classes`]): one that styles nothing fails here, where
    /// no game's check reaches it.
    #[test]
    fn the_engines_components_use_only_known_classes() {
        use og_ui_next::pack::{Layer, Pack};
        let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui");
        let pack = Pack::new(&[Layer::default_pack()]);
        let mut classes = class_resolver(&pack).unwrap();
        let mut files = Vec::new();
        crate::lockstep_lint::sources(&ui, &mut files);
        files.retain(|f| f.extension().is_some_and(|e| e == "tsx"));
        assert!(files.len() > 20, "the components are there");
        let mut unknown = Vec::new();
        for path in files {
            let text = std::fs::read_to_string(&path).unwrap();
            for u in nsh_jsx::unknown_classes_with(&path.to_string_lossy(), &text, |s| {
                classes.unknown(s)
            }) {
                let file = path
                    .strip_prefix(&ui)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                unknown.push(format!("{file}:{}: {}", u.line, u.class));
            }
        }
        assert!(
            unknown.is_empty(),
            "unknown classes:\n{}",
            unknown.join("\n")
        );
    }

    /// A class that fixes a visual value no theme reaches: a duration,
    /// delay, opacity, line width or scale given as a number instead of
    /// the theme's names (`duration-base`, `opacity-dim`, `ring-regular`,
    /// `scale-press`), a fixed Tailwind radius (`rounded`, `rounded-sm`..`3xl`)
    /// outside `theme.radius`, or a hardcoded cut-corner nine-slice
    /// (`nine-panel-cut`, `nine-plate-cut`, `nine-chip-cut`) that bypasses the
    /// pack's shape slots. `0`, `100` (opacity, scale) and arbitrary values,
    /// which [`literal_text`] counts, are not.
    fn literal_class(class: &str) -> bool {
        let utility = class.rsplit(':').next().unwrap_or(class);
        let utility = utility.trim_start_matches('-');
        if matches!(
            utility,
            "rounded"
                | "rounded-sm"
                | "rounded-md"
                | "rounded-lg"
                | "rounded-xl"
                | "rounded-2xl"
                | "rounded-3xl"
                | "nine-panel-cut"
                | "nine-plate-cut"
                | "nine-chip-cut"
                | "nine-socket"
        ) {
            return true;
        }
        // A fixed radius on one side or corner (`rounded-t-lg`,
        // `rounded-tl-md`, `rounded-s`): the radius roles have no sides.
        if let Some(rest) = utility.strip_prefix("rounded-") {
            let (side, size) = rest.split_once('-').unwrap_or((rest, ""));
            let sided = matches!(
                side,
                "t" | "r"
                    | "b"
                    | "l"
                    | "s"
                    | "e"
                    | "tl"
                    | "tr"
                    | "bl"
                    | "br"
                    | "ss"
                    | "se"
                    | "es"
                    | "ee"
            );
            if sided && matches!(size, "" | "xs" | "sm" | "md" | "lg" | "xl" | "2xl" | "3xl") {
                return true;
            }
        }
        let Some((name, value)) = utility.rsplit_once('-') else {
            return false;
        };
        if value.is_empty() || !value.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return false;
        }
        let edge = matches!(name, "ring" | "inset-ring" | "outline")
            || name == "border"
            || name.strip_prefix("border-").is_some_and(|s| s.len() == 1);
        match name {
            "duration" | "delay" => true,
            "opacity" | "scale" | "scale-x" | "scale-y" => value != "0" && value != "100",
            _ => edge && value != "0",
        }
    }

    /// Visual literals in a component's source outside its classes' names:
    /// a hex colour in a string and a length or time in px or ms (an
    /// inline style, an arbitrary class value, a timer).
    fn literal_text(text: &str) -> usize {
        let b = text.as_bytes();
        let mut n = 0;
        let mut i = 0;
        while i < b.len() {
            if b[i] == b'#' && i > 0 && matches!(b[i - 1], b'"' | b'\'' | b'`' | b'[') {
                let len = b[i + 1..]
                    .iter()
                    .take_while(|c| c.is_ascii_hexdigit())
                    .count();
                if matches!(len, 3 | 4 | 6 | 8) {
                    n += 1;
                }
            }
            if b[i].is_ascii_digit() && (i == 0 || !b[i - 1].is_ascii_alphanumeric()) {
                let mut j = i;
                while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'.') {
                    j += 1;
                }
                let unit = |u: &[u8]| {
                    b[j..].starts_with(u)
                        && b.get(j + u.len())
                            .is_none_or(|c| !c.is_ascii_alphanumeric())
                };
                let zero = b[i..j].iter().all(|&c| c == b'0' || c == b'.');
                if !zero && (unit(b"px") || unit(b"ms")) {
                    n += 1;
                }
                i = j;
                continue;
            }
            i += 1;
        }
        n
    }

    /// No engine component brings in a visual value no theme can change
    /// (xiaomao, 2026-11-24): every duration, opacity, line width, scale,
    /// colour and length is a token or a pack slot a theme sets. The files
    /// that still have some are listed in `component-literals.json` with
    /// their count; a count may fall, and then the list is lowered with it
    /// (`OG_BLESS=1`), but never rise, and a new file has none.
    #[test]
    fn no_engine_component_adds_a_visual_literal() {
        let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui");
        let list =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/commands/component-literals.json");
        let mut files = Vec::new();
        crate::lockstep_lint::sources(&ui, &mut files);
        files.retain(|f| f.extension().is_some_and(|e| e == "tsx"));
        let mut counts = std::collections::BTreeMap::new();
        for path in files {
            let text = std::fs::read_to_string(&path).unwrap();
            // A component's fixtures are the game's data (a ramp's colours,
            // a car's), passed in as props: not the component's look.
            let text = text.split("\nexport const fixtures").next().unwrap_or("");
            // Doc comments name values (`--ability-slot-size`, `48 px`).
            let code: String = text
                .lines()
                .filter(|l| {
                    let t = l.trim_start();
                    !(t.starts_with("//") || t.starts_with("/*") || t.starts_with('*'))
                })
                .collect::<Vec<_>>()
                .join("\n");
            let classes = nsh_jsx::unknown_classes_with(&path.to_string_lossy(), &code, |s| {
                s.split_whitespace()
                    .filter(|c| literal_class(c))
                    .map(str::to_owned)
                    .collect()
            });
            let n = classes.len() + literal_text(&code);
            if n > 0 {
                let file = path.strip_prefix(&ui).unwrap().display().to_string();
                counts.insert(file, n);
            }
        }
        if std::env::var_os("OG_BLESS").is_some() {
            let text = serde_json::to_string_pretty(&counts).unwrap();
            std::fs::write(&list, text + "\n").unwrap();
            return;
        }
        let allowed: std::collections::BTreeMap<String, usize> =
            serde_json::from_str(&std::fs::read_to_string(&list).unwrap()).unwrap();
        let mut wrong = Vec::new();
        for (file, &n) in &counts {
            let a = allowed.get(file).copied().unwrap_or(0);
            if n > a {
                wrong.push(format!(
                    "{file}: {n} visual literals, {a} allowed: use the theme's tokens"
                ));
            }
        }
        for (file, &a) in &allowed {
            let n = counts.get(file).copied().unwrap_or(0);
            if n < a {
                wrong.push(format!(
                    "{file}: {n} visual literals, the list says {a}: lower it (OG_BLESS=1)"
                ));
            }
        }
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }

    #[test]
    fn literals_are_numbers_no_theme_names() {
        let found: Vec<_> = "duration-317 duration-base hover:opacity-45 opacity-dim opacity-100 \
             inset-ring-2 ring-regular ring-accent border-l-3 border-0 scale-96 scale-press \
             rounded-2 rounded-md rounded-control nine-plate-cut nine-ring p-4 size-12 rotate-45 delay-75"
            .split(' ')
            .filter(|c| literal_class(c))
            .collect();
        assert_eq!(
            found,
            [
                "duration-317",
                "hover:opacity-45",
                "inset-ring-2",
                "border-l-3",
                "scale-96",
                "rounded-md",
                "nine-plate-cut",
                "delay-75"
            ]
        );
        assert_eq!(
            literal_text(r##"style={{ top: "48px" }} c="#ffd24a" 0px 1.5px 120ms 12 #idx"##),
            4
        );
    }

    /// No engine component draws its own border, ring or outline on an
    /// element a nine-sliced frame draws (`nine-*`): the frame is the
    /// element's edge, as CSS `border-image` is, so a rectangle round it
    /// is a second edge that misses the art's shape (a cut or rounded
    /// corner). Its room for content is the frame's own inset.
    #[test]
    fn no_engine_component_edges_a_framed_element_twice() {
        use og_ui_next::classes::Env;
        use og_ui_next::pack::{Layer, Pack};
        let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui");
        let pack = Pack::new(&[Layer::default_pack()]);
        let mut classes = class_resolver(&pack).unwrap();
        let env = Env {
            window: [390.0, 844.0],
            dark: true,
            rtl: false,
            touch: true,
            reduce_motion: false,
            states: 0,
        };
        let mut files = Vec::new();
        crate::lockstep_lint::sources(&ui, &mut files);
        files.retain(|f| f.extension().is_some_and(|e| e == "tsx"));
        // The edges `s` draws besides its nine-* frame, if it draws one.
        let mut edges = |s: &str| {
            let r = classes.record(s, &env).skin;
            if r.image.slice == [0.0; 4] {
                return Vec::new();
            }
            let mut edges = Vec::new();
            if r.border.iter().any(|&w| w > 0.0) {
                edges.push("border");
            }
            let ring = r.shadows[..r.shadow_count as usize]
                .iter()
                .any(|sh| !sh.inset && sh.blur == 0.0 && sh.spread > 0.0);
            if ring {
                edges.push("ring");
            }
            if r.outline.width > 0.0 {
                edges.push("outline");
            }
            if edges.is_empty() {
                Vec::new()
            } else {
                vec![edges.join(" ")]
            }
        };
        let mut twice = Vec::new();
        // The pack slots a theme frames with nine-slice art (plus any slot a
        // component still puts beside `nine-*`).
        let mut framed_slots: std::collections::BTreeSet<String> = [
            "Button.plate",
            "Button.plate.primary",
            "Button.plate.danger",
            "Panel.frame",
            "Plate.frame",
            "Chip.frame",
            "Slot.frame",
            "Field.frame",
            "Field.area",
            "Row.frame",
            "Row.frame.selected",
            "Row.frame.muted",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        for path in files {
            let text = std::fs::read_to_string(&path).unwrap();
            for line in text.lines().filter(|l| l.contains("nine-")) {
                for call in line.split("packClass(").skip(1) {
                    let args: Vec<_> = call
                        .split(')')
                        .next()
                        .unwrap_or("")
                        .split(',')
                        .map(|a| a.trim().trim_matches('"'))
                        .collect();
                    if args
                        .iter()
                        .all(|a| !a.is_empty() && a.chars().all(|c| c.is_ascii_alphanumeric()))
                    {
                        framed_slots.insert(args.join("."));
                    }
                }
            }
            for u in nsh_jsx::unknown_classes_with(&path.to_string_lossy(), &text, &mut edges) {
                let file = path
                    .strip_prefix(&ui)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                twice.push(format!("{file}:{}: {}", u.line, u.class));
            }
        }
        // A slot carries the look a theme gives it, so it is held to the
        // same rule over a frame: the slots themes frame, and any slot that
        // draws a frame itself.
        for (key, value) in pack.variants() {
            let framed = framed_slots.contains(key) || value.contains("nine-");
            for e in edges(&format!("nine-plate-cut {value}")) {
                if framed {
                    twice.push(format!("pack slot {key}: {e}"));
                }
            }
        }
        assert!(
            twice.is_empty(),
            "a nine-* frame with its own edge as well ({}):\n{}",
            twice.len(),
            twice.join("\n")
        );
    }

    /// No engine component sets an ink on a theme's art that the art does
    /// not carry (OG9107 on a child): under a theme of every material, the
    /// text inside an art slot reads in the slot's solved ink, or in one
    /// that holds 4.5:1 on that art too.
    #[test]
    fn no_engine_component_inks_over_a_themes_art() {
        let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui");
        let materials = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../templates/themes/dimensions/material");
        let mut low = std::collections::BTreeSet::new();
        let mut names: Vec<String> = std::fs::read_dir(&materials)
            .unwrap()
            .map(|e| {
                e.unwrap()
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        for m in names {
            for source in ["#d9a531", "#5cc8ff", "#ff5c9a"] {
                let recipe = format!(
                    r##"{{ "material": "{m}", "shape": "soft", "color": {{ "source": "{source}", "neutral": "#4c5643" }} }}"##
                );
                let theme = og_cook::cook::ui_themes::compile(&recipe, "test").unwrap();
                let art = crate::ui_grounds::ArtGrounds::of(&theme).unwrap();
                let colors = crate::ui_grounds::colors(
                    &art.colors
                        .clone()
                        .map(|c| serde_json::json!({ "colors": c }).to_string()),
                );
                let scale = crate::ui_values::Scale::new(&[]);
                for (file, f) in crate::ui_grounds::lint(&ui, &[&ui], &colors, &scale, Some(&art)) {
                    if f.code == crate::ui_grounds::ART_INK {
                        low.insert(format!(
                            "{file}:{}: {} ({m} {source}: {})",
                            f.line,
                            f.class,
                            f.message.split(" (under").next().unwrap_or("")
                        ));
                    }
                }
            }
        }
        // one line per class, its first material
        let mut seen = std::collections::BTreeSet::new();
        let low: Vec<_> = low
            .into_iter()
            .filter(|l| seen.insert(l.split(" (").next().unwrap().to_owned()))
            .collect();
        assert!(
            low.is_empty(),
            "{} inks on art:\n{}",
            low.len(),
            low.join("\n")
        );
    }
}
