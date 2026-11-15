//! CSS directories are the authoring format; no manifest or generated atlas.
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[cfg(test)]
mod builtin;
mod contrast;
mod generate;
mod literals;

const USAGE: &str = "usage: og style new <name> [--look <preset>] [--scheme dark|light] [--finish flat|gradient|glass|hologram|neon|paper|bevel|brush] [--out <directory>]\n       og style check <directory|pack|default> [--json]\n          (outside :root a declaration names its colours, lengths, durations and url()s with var(); set the value as a :root token; @font-face src and -og-sound may name url()s)\n       og style check-project [project] [art-contract options]\n       og style add <directory> [project]\n       og style list [project] [--json]";

fn files(
    directory: &Path,
    root: &Path,
    namespace: &str,
    sources: &mut Vec<(String, String)>,
) -> Result<(), String> {
    for entry in
        std::fs::read_dir(directory).map_err(|e| format!("{}: {e}", directory.display()))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        if ty.is_symlink() {
            return Err(format!("{}: symlinks are not supported", path.display()));
        }
        if ty.is_dir() {
            files(&path, root, namespace, sources)?;
        } else if path.extension().is_some_and(|e| e == "css") {
            let id = format!(
                "{namespace}/{}",
                path.strip_prefix(root).unwrap().to_string_lossy()
            );
            sources.push((
                id,
                std::fs::read_to_string(&path).map_err(|e| e.to_string())?,
            ));
        }
    }
    Ok(())
}

fn document_parts(directory: &Path, out: &mut Vec<(String, String)>) -> Result<(), String> {
    for entry in
        std::fs::read_dir(directory).map_err(|e| format!("{}: {e}", directory.display()))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        if ty.is_symlink() {
            continue;
        }
        if ty.is_dir() {
            document_parts(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "svelte") {
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            out.extend(og_ui::doc::schema::parts::authored_parts(
                &path.to_string_lossy(),
                &text,
            )?);
        }
    }
    Ok(())
}

fn check(args: &[&str]) -> Result<(), String> {
    let (name, json) = match args {
        [name] => (*name, false),
        [name, "--json"] => (*name, true),
        _ => return Err(USAGE.into()),
    };
    // `default` is the core default style, with no pack; a name that is no
    // directory is a pack of the engine's styles (`fantasy`).
    if name == "default" {
        return report(json, Vec::new(), None, contrast::Textures::of(None, "pack"));
    }
    let shipped =
        og_cook::cook::look::styles_dir().map(|d| d.join(name.trim_start_matches("@styles/")));
    let directory = match Path::new(name) {
        d if d.is_dir() => d.to_path_buf(),
        _ => shipped.filter(|d| d.is_dir()).ok_or_else(|| {
            format!("{name} is neither a directory nor a pack of the engine's styles")
        })?,
    };
    let directory = directory.as_path();
    let asset_root = directory
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == "assets"));
    let namespace = asset_root
        .map(|p| {
            directory
                .strip_prefix(p)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_else(|| "pack".into());
    let prefix = format!("{namespace}/");
    let mut sources = Vec::new();
    files(directory, directory, &namespace, &mut sources)?;
    sources.sort();
    let mut authored = Vec::new();
    document_parts(asset_root.unwrap_or(directory), &mut authored)?;
    // The engine's built-in components name parts a pack dresses too.
    if let Some(ui) = og_cook::cook::ui_components::ui_dir() {
        document_parts(ui, &mut authored)?;
    }
    let mut findings = Vec::new();
    for (id, text) in &sources {
        if let Err(error) = og_ui::style_pack::StylePack::validate_source(id, text, sources.clone())
        {
            findings.push(error.replace(&prefix, &format!("{}/", directory.display())));
        }
        findings.extend(
            og_ui::doc::schema::parts::validate_source(id, text, &authored)
                .into_iter()
                .map(|error| error.replace(&prefix, &format!("{}/", directory.display()))),
        );
        findings.extend(literals::findings(
            &format!(
                "{}/{}",
                directory.display(),
                id.strip_prefix(&prefix).unwrap_or(id)
            ),
            text,
        ));
        for url in og_ui::doc::at_rules::urls(text) {
            if url.starts_with('#') || url.starts_with("data:") {
                continue;
            }
            let resolved = og_ui::doc::at_rules::resolve(id, &url);
            let path = if !url.starts_with('/') {
                directory
                    .join(id.strip_prefix(&prefix).unwrap())
                    .parent()
                    .unwrap()
                    .join(&url)
            } else if let Some(relative) = resolved.strip_prefix("@styles/") {
                og_cook::cook::look::styles_dir()
                    .map(|p| p.join(relative))
                    .unwrap_or_default()
            } else {
                asset_root
                    .map(|p| p.join(&resolved))
                    .unwrap_or_else(|| PathBuf::from(&resolved))
            };
            if !path.is_file() {
                let line = text[..text.find(&url).unwrap_or(0)]
                    .bytes()
                    .filter(|b| *b == b'\n')
                    .count()
                    + 1;
                findings.push(format!(
                    "{}:{}: unresolved url({url})",
                    directory.join(id.strip_prefix(&prefix).unwrap()).display(),
                    line
                ));
            }
        }
    }
    let textures = contrast::Textures::of(Some(directory), &namespace);
    match og_ui::style_pack::StylePack::parse(&namespace, sources) {
        Ok(pack) => report(json, findings, Some(&pack), textures),
        Err(e) => {
            if findings.is_empty() {
                findings.push(e);
            }
            report(json, findings, None, textures)
        }
    }
}

/// Prints the findings and the contrast facts of `pack`'s sheets (the
/// core default's when it has none). A check with findings prints no text
/// pairs: drawing every sheet twice is most of a check's time, and a pack
/// that fails is fixed before its contrast is read.
fn report(
    json: bool,
    findings: Vec<String>,
    pack: Option<&og_ui::style_pack::StylePack>,
    textures: contrast::Textures,
) -> Result<(), String> {
    let pairs = if findings.is_empty() {
        contrast::pairs(pack, &textures)?
    } else {
        Vec::new()
    };
    let vision = contrast::vision(pack)?;
    if json {
        println!(
            "{}",
            serde_json::json!({
                "errors": findings,
                "contrast": pairs.iter().map(contrast::Pair::json).collect::<Vec<_>>(),
                "colorVision": vision,
            })
        );
    } else {
        for finding in &findings {
            println!("{finding}");
        }
        let (core, documents): (Vec<_>, Vec<_>) =
            pairs.iter().partition(|p| p.sheet.starts_with("sheet:"));
        println!(
            "contrast: each text the sheets draw on what is drawn under it, in both schemes: WCAG 2's ratio and APCA's Lc (the bars: og api show screens, §5)"
        );
        print_contrast(&core);
        print_documents(&documents);
        print_vision(&vision);
    }
    if findings.is_empty() {
        Ok(())
    } else {
        Err(format!("{} error(s)", findings.len()))
    }
}

/// One line per ink on one ground in one scheme, lowest ratio first, with
/// how many texts and which sheets draw it.
fn print_contrast(pairs: &[&contrast::Pair]) {
    let mut groups: Vec<(&contrast::Pair, Vec<&contrast::Pair>)> = Vec::new();
    for &p in pairs {
        match groups.iter_mut().find(|(g, _)| {
            g.scheme == p.scheme && g.ink == p.ink && g.ground == p.ground && g.over == p.over
        }) {
            Some((_, all)) => all.push(p),
            None => groups.push((p, vec![p])),
        }
    }
    groups.sort_by(|a, b| a.0.wcag.total_cmp(&b.0.wcag));
    let sheets: std::collections::BTreeSet<&str> = pairs.iter().map(|p| p.sheet.as_str()).collect();
    println!(
        "the core-control sheet: {} texts on its {} pages, {} ink-on-ground pairs",
        pairs.len(),
        sheets.len(),
        groups.len()
    );
    for (p, all) in &groups {
        let mut names: Vec<&str> = all.iter().map(|q| q.sheet.as_str()).collect();
        names.dedup();
        let more = if names.len() > 2 {
            format!(" +{}", names.len() - 2)
        } else {
            String::new()
        };
        let halo = all
            .iter()
            .filter_map(|q| q.halo)
            .reduce(f32::min)
            .map(|h| format!(", {h:.2}:1 on its halo"));
        let flags = [
            all.iter()
                .all(|q| q.disabled)
                .then(|| ", disabled".to_string()),
            halo,
        ]
        .into_iter()
        .flatten()
        .collect::<String>();
        println!(
            "  {:>5.2}:1  Lc {:>6.1}  {:5}  {} on {} ({}{flags})  {} text(s): \"{}\"  {}{more}",
            p.wcag,
            p.apca,
            p.scheme,
            contrast::hex(p.ink),
            contrast::hex(p.ground),
            p.over,
            all.len(),
            p.text.trim().chars().take(32).collect::<String>(),
            names.iter().take(2).copied().collect::<Vec<_>>().join(", "),
        );
    }
}

/// Each built-in document's lowest pair in each scheme, lowest first (every
/// pair is in `--json`).
fn print_documents(pairs: &[&contrast::Pair]) {
    let mut lowest: Vec<&contrast::Pair> = Vec::new();
    for &p in pairs {
        match lowest
            .iter_mut()
            .find(|l| l.sheet == p.sheet && l.scheme == p.scheme)
        {
            Some(l) if p.wcag < l.wcag => *l = p,
            Some(_) => {}
            None => lowest.push(p),
        }
    }
    lowest.sort_by(|a, b| a.wcag.total_cmp(&b.wcag));
    println!(
        "the built-in documents, from their fixtures: {} texts; each document's lowest pair per scheme",
        pairs.len()
    );
    for p in lowest {
        let count = pairs
            .iter()
            .filter(|q| q.sheet == p.sheet && q.scheme == p.scheme)
            .count();
        println!(
            "  {:>5.2}:1  Lc {:>6.1}  {:5}  {} on {} ({}{}{})  \"{}\" of {count} text(s)  {}",
            p.wcag,
            p.apca,
            p.scheme,
            contrast::hex(p.ink),
            contrast::hex(p.ground),
            p.over,
            if p.disabled { ", disabled" } else { "" },
            p.halo
                .map(|h| format!(", {h:.2}:1 on its halo"))
                .unwrap_or_default(),
            p.text.trim().chars().take(32).collect::<String>(),
            p.sheet,
        );
    }
}

/// The closest pair of each colour set under each vision.
fn print_vision(vision: &[serde_json::Value]) {
    println!(
        "colour vision: the closest two of each set as each vision sees them (ΔEOK in OKLab, WCAG 2 ratio between them)"
    );
    for v in vision {
        let closest = v["pairs"].as_array().into_iter().flatten().min_by(|a, b| {
            a["deltaE"]
                .as_f64()
                .unwrap_or(0.0)
                .total_cmp(&b["deltaE"].as_f64().unwrap_or(0.0))
        });
        if let Some(c) = closest {
            println!(
                "  {:5} {:5} {:12}  {}/{}  {} {}  ΔEOK {}  {}:1",
                v["scheme"].as_str().unwrap_or_default(),
                v["set"].as_str().unwrap_or_default(),
                v["vision"].as_str().unwrap_or_default(),
                c["a"].as_str().unwrap_or_default(),
                c["b"].as_str().unwrap_or_default(),
                c["seenA"].as_str().unwrap_or_default(),
                c["seenB"].as_str().unwrap_or_default(),
                c["deltaE"],
                c["ratio"],
            );
        }
    }
}

pub fn run(args: &[&str]) -> ExitCode {
    let result = match args {
        ["new", rest @ ..] => generate::run(rest),
        ["check", rest @ ..] => check(rest),
        ["check-project", rest @ ..] => return crate::style_check::run(rest),
        ["add", rest @ ..] => return crate::ui::extensions::add(rest, "style"),
        ["list", rest @ ..] => return crate::ui::extensions::list(rest, Some("style")),
        ["--help"] | ["help"] => { println!("{USAGE}"); Ok(()) },
        ["upgrade" | "art" | "pack", ..] => Err("legacy style.ogt conversion and atlas packing are removed; author CSS images, border-image and mask-image directly".into()),
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
