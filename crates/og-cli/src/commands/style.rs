//! Checked theme/pack declarations, addressed by canonical content IDs.
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og style init <name> [project] [--from <theme>]\n       og style validate <content-id> [project]\n       og style inspect <builtin-id|default> --json\n       og style check-project [project] [art-contract options]\n       og style add <directory> [project]\n       og style list [project] [--json]";

pub fn run(args: &[&str]) -> ExitCode {
    let result = match args {
        ["init", rest @ ..] => init(rest),
        ["validate", rest @ ..] => validate(rest),
        ["inspect", id, "--json"] => inspect(id),
        ["check-project", rest @ ..] => return crate::style_check::run(rest),
        ["add", rest @ ..] => return crate::ui::extensions::add(rest, "style"),
        ["list", rest @ ..] => return crate::ui::extensions::list(rest, Some("style")),
        ["--help"] | ["help"] => {
            println!("{USAGE}");
            Ok(())
        }
        _ => Err(USAGE.into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("og style: {e}");
            ExitCode::FAILURE
        }
    }
}

fn init(args: &[&str]) -> Result<(), String> {
    let mut words = Vec::new();
    let mut from = "minimal";
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match *a {
            "--from" => from = it.next().ok_or("--from takes a theme name")?,
            a if a.starts_with('-') => return Err(format!("unknown option `{a}`\n{USAGE}")),
            a => words.push(a),
        }
    }
    let [name, rest @ ..] = words.as_slice() else {
        return Err(USAGE.into());
    };
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        || rest.len() > 1
    {
        return Err("a style name is letters, digits, `-` and `_`".into());
    }
    let root = Path::new(rest.first().copied().unwrap_or("."));
    if !root.join("og-project.json").is_file() {
        return Err("no og-project.json".into());
    }
    og_cook::cook::ui_themes::builtin_recipe(from)
        .ok_or_else(|| format!("no engine theme `{from}`"))?;
    let target = root.join("content/styles").join(format!("{name}.ts"));
    std::fs::create_dir_all(target.parent().unwrap()).map_err(|e| e.to_string())?;
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
        .map_err(|e| format!("{}: {e}", target.display()))?;
    let parent = serde_json::to_string(&format!("@themes/{from}")).unwrap();
    write!(file, "import {{ file }} from '@origingame/engine/author';\nexport default file.theme({{extends: {parent}}});\n").map_err(|e| e.to_string())?;
    println!(
        "og style: wrote {}; set look.ui to base/styles/{name}",
        target.display()
    );
    Ok(())
}

fn validate(args: &[&str]) -> Result<(), String> {
    use og_ui_next::pack::{Layer, grounds_without_ink};
    let [id, rest @ ..] = args else {
        return Err(USAGE.into());
    };
    if rest.len() > 1 {
        return Err(USAGE.into());
    }
    let root = Path::new(rest.first().copied().unwrap_or("."));
    let styles = og_cook::cook::ui_packs::project_styles(root)?;
    let style = styles
        .get(*id)
        .ok_or_else(|| format!("{id}: no checked theme/pack declaration"))?;
    // The theme compiler solves its own inks. Standalone packs can instead
    // paint a new ground without the text ink that belongs on it.
    if style.theme.is_none() {
        let problems = grounds_without_ink(&[Layer::default_pack()], &style.layer);
        if !problems.is_empty() {
            return Err(problems.join("\n"));
        }
    }
    println!("og style: {id} is valid");
    Ok(())
}

/// Built-in native tables for release tooling; never evaluate author TS in Bun.
fn inspect(id: &str) -> Result<(), String> {
    use og_ui_next::pack::{Layer, Pack, Scheme};
    let mut layers = vec![Layer::default_pack()];
    if id != "default" {
        layers.push(
            og_cook::cook::ui_packs::builtin(id, &std::env::temp_dir().join("og-ui-art"))?.layer,
        );
    }
    let pack = Pack::new(&layers);
    let light: serde_json::Value =
        serde_json::from_str(&pack.theme_json(Scheme::Light)).map_err(|e| e.to_string())?;
    let dark: serde_json::Value =
        serde_json::from_str(&pack.theme_json(Scheme::Dark)).map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::json!({"light":light,"dark":dark,"fonts":pack.fonts})
    );
    Ok(())
}
