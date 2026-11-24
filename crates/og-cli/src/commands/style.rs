//! A style is a directory under `assets/styles/` with a theme recipe
//! (`theme.json`), a pack (`pack.json`) laid over it, or both (og-ui-next's
//! CONVENTIONS.md §Theme and packs).
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og style init <name> [project] [--from <theme>]\n       og style validate <directory>\n       og style check-project [project] [art-contract options]\n       og style add <directory> [project]\n       og style list [project] [--json]";

pub fn run(args: &[&str]) -> ExitCode {
    let result = match args {
        ["new", ..] => Err::<(), String>("CSS packs are removed, so `og style new` is too; start a style with `og style init <name>` or add one with `og style add <directory>`".into()),
        ["init", rest @ ..] => init(rest),
        ["validate", rest @ ..] => validate(rest),
        ["check-project", rest @ ..] => return crate::style_check::run(rest),
        ["add", rest @ ..] => return crate::ui::extensions::add(rest, "style"),
        ["list", rest @ ..] => return crate::ui::extensions::list(rest, Some("style")),
        ["--help"] | ["help"] => { println!("{USAGE}"); Ok(()) },
        ["upgrade" | "art" | "pack", ..] => Err("legacy style.ogt conversion and atlas packing are removed; name the art in the pack's pack.json (og api show screens, §3)".into()),
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

/// `og style init <name> [project] [--from <theme>]`: a new style at
/// `assets/styles/<name>/theme.json`, a copy of one of the engine's theme
/// recipes (`minimal` by default) to edit. It never overwrites.
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
    {
        return Err(format!(
            "`{name}`: a style name is letters, digits, `-` and `_`"
        ));
    }
    let project = PathBuf::from(rest.first().copied().unwrap_or("."));
    if !project.join("project.ogt").is_file() {
        return Err(format!(
            "{}: no project.ogt; run it in a game's directory or pass the project",
            project.display()
        ));
    }
    let source = og_cook::cook::ui_themes::builtin_recipe(from)
        .ok_or_else(|| format!("no engine theme `{from}`; `og style list` names them"))?;
    let dir = project.join("assets/styles").join(name);
    let target = dir.join(og_cook::cook::ui_themes::RECIPE);
    if target.exists() || dir.join("pack.json").exists() {
        return Err(format!("{}: a style is already there", dir.display()));
    }
    let text =
        std::fs::read_to_string(&source).map_err(|e| format!("{}: {e}", source.display()))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    std::fs::write(&target, text).map_err(|e| format!("{}: {e}", target.display()))?;
    println!(
        "og style: wrote {} (from the {from} theme); point the look's `ui` at styles/{name} and check it with `og style validate {}`",
        target.display(),
        dir.display()
    );
    Ok(())
}

/// `og style validate <directory>`: the style's recipe compiles, its pack
/// parses, and none of the pack's variants paints a ground without its ink. Each problem is
/// a line; it exits 1 when there is one.
fn validate(args: &[&str]) -> Result<(), String> {
    use og_ui_next::pack::{Layer, grounds_without_ink};
    let [dir] = args else {
        return Err(USAGE.into());
    };
    let dir = Path::new(dir);
    let recipe = dir.join(og_cook::cook::ui_themes::RECIPE);
    let pack = dir.join("pack.json");
    if !recipe.is_file() && !pack.is_file() {
        return Err(format!("{}: no theme.json or pack.json", dir.display()));
    }
    let mut problems = Vec::new();
    let mut layers = vec![Layer::default_pack()];
    if let Ok(text) = std::fs::read_to_string(&recipe) {
        let label = recipe.to_string_lossy();
        match og_cook::cook::ui_themes::compile(&text, &label)
            .and_then(|t| t.layer(Default::default()).map_err(|e| e.to_string()))
        {
            Ok(layer) => layers.push(layer),
            Err(e) => problems.push(format!("{}: {e}", recipe.display())),
        }
    }
    if let Ok(text) = std::fs::read_to_string(&pack) {
        match Layer::parse(&text) {
            Ok(layer) => layers.push(layer),
            Err(e) => problems.push(format!("{}: {e}", pack.display())),
        }
    }
    // The theme compiler solves its own inks; the pack laid over it is
    // what can paint a ground without one.
    if problems.is_empty() && pack.is_file() {
        let top = layers.pop().expect("the pack's layer");
        for message in grounds_without_ink(&layers, &top) {
            problems.push(format!("{}: {message}", pack.display()));
        }
    }
    if problems.is_empty() {
        println!("og style: {} is valid", dir.display());
        return Ok(());
    }
    for p in &problems {
        println!("{p}");
    }
    Err(format!(
        "{} problem(s) in {}",
        problems.len(),
        dir.display()
    ))
}
