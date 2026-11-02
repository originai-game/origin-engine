//! `og style pack <dir> [--out <dir>]`: a UI style's art drawn by hand, one
//! PNG per part, packed into the atlas a style file cuts from.
//!
//! The input directory holds:
//!
//! - `<part>.9.png` for each nine-slice: `panel`, `card`, `button`,
//!   `buttonHover`, `buttonPressed`, `buttonDisabled`, `field`, `slot`,
//!   `tooltip`, `capsule`, `track`, `fill`, and `soft` (the blurred
//!   rectangle shadows and glows use). A nine-patch, as Android draws them:
//!   the art inside a one-pixel border whose top row marks the columns that
//!   stretch and whose left column marks the rows that stretch (opaque dark
//!   pixels); the corner insets are what lies outside the marks;
//! - any other `<name>.png` (`corner`, `divider`, `flourish`, `tile`, a
//!   picture of the game's own): packed as it is.
//!
//! It writes `<out>/atlas.png` (the images on shelves, tallest first, two
//! pixels apart with their edge pixels extruded into the gap so filtering
//! never reads a neighbour), `<out>/atlas.png.ogt` when missing, and
//! `<out>/style.ogt`: an existing one keeps everything but its `atlas` line,
//! its `slice` and `soft` lines (rewritten from the nine-patches) and the
//! regions of its `corner`, `divider`, `flourish` and `tile` lines (moved to
//! where those images now sit); without one it starts a style of the slices.
//! `--out` defaults to the input directory.
//!
//! `og style list` names the built-in styles; `og style eject <slug>
//! [<name>] [--project <dir>] [--force]` writes one, byte for byte from the
//! copy compiled into `og` (its `style.ogt`, atlas, icons, fonts with their
//! import settings, licences), into the game's `assets/styles/<name>/`, a
//! game's own style to change at will. Its Chinese companions stay the
//! shared built-in `@styles/cjk/` fonts, which the cook bundles for a game
//! that shows Chinese.
//!
//! `og style new <name> [--look <family>] [--premise <text>] [choices]`
//! generates a style (og-ui's `style_gen`, docs/ui.md §12.5): it starts from
//! the seed style nearest the look family, takes the flags over it (the
//! palette seed, scheme, shape language, ornament level, motion personality
//! and finish), and writes `assets/styles/<name>/` of the game (or
//! `--out <dir>`): its `style.ogt` (palette, tokens, procedural dressings,
//! sound sets, haptics), the `type sfx` presets its sounds name, and
//! `art-requests.json`, the art slots it asks for (frames, buttons,
//! ornaments, icons) which the art brief's `styleArtBriefs` turns into
//! prompts. Its fonts and icons are the built-in packs' (`@styles/...`),
//! which the cook bundles. `og style new --seeds [--out <dir>]` rewrites the
//! seed styles of `templates/styles` from their recorded choices.
//!
//! `og style upgrade <style.ogt> [--out <file>] [--check]` writes a
//! version 1 style as version 2 (og-ui's `upgrade_style`, docs/ui.md
//! §12): every version 1 value kept, so it draws the same; its part lines
//! become dressings, its radii the shape language, its durations motion
//! tokens; the new tokens derived from its palette and scheme. In place
//! unless `--out`; `--check` writes nothing and fails while it is version 1.

use image::{Rgba, RgbaImage};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og style list\n       og style new <name> [--look <family>] [--premise <text>] [--seed <#rrggbb>] [--scheme dark|light] [--shape <s>] [--ornament <o>] [--motion <m>] [--finish <f>] [--project <dir> | --out <dir>] [--force]\n       og style new --seeds [--out <dir>]\n       og style upgrade <style.ogt> [--out <file>] [--check]\n       og style eject <slug> [<name>] [--project <dir>] [--force]\n       og style pack <dir> [--out <dir>]\n       og style art <pack dir> [--art <dir>] [--check]\n       og style check [project] [--title <commands>] [--play <commands>] [--results <commands>] [--size WxH] [--no-frames] [--critic] [--json]";

/// The parts a nine-patch can dress (og-ui's `StylePart` names) and `soft`.
const SLICES: [&str; 13] = [
    "panel",
    "card",
    "button",
    "buttonHover",
    "buttonPressed",
    "buttonDisabled",
    "field",
    "slot",
    "tooltip",
    "capsule",
    "track",
    "fill",
    "soft",
];

/// Pixels between packed images (each image's edge is extruded into it).
const GAP: u32 = 2;

/// One image to pack, with its nine-slice insets when it is a nine-patch.
struct Piece {
    name: String,
    image: RgbaImage,
    /// left, top, right, bottom in texels.
    insets: Option<[u32; 4]>,
    at: (u32, u32),
}

pub fn run(args: &[&str]) -> ExitCode {
    match args {
        ["check", rest @ ..] => crate::style_check::run(rest),
        ["list"] => {
            for slug in og_cook::cook::look::style_slugs() {
                let about = ABOUT
                    .iter()
                    .find(|(s, _)| *s == slug)
                    .map_or("", |(_, a)| a);
                println!("{slug:<10} {about}");
            }
            ExitCode::SUCCESS
        }
        ["new", rest @ ..] => match new(rest) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("og style new: {e}");
                ExitCode::from(1)
            }
        },
        ["eject", rest @ ..] => match eject(rest) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("og style eject: {e}");
                ExitCode::from(1)
            }
        },
        ["upgrade", rest @ ..] => match upgrade(rest) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("og style upgrade: {e}");
                ExitCode::from(1)
            }
        },
        ["art", rest @ ..] => match crate::style_art::run(rest) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("og style art: {e}");
                ExitCode::from(1)
            }
        },
        ["pack", rest @ ..] => match pack(rest) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("og style pack: {e}");
                ExitCode::from(1)
            }
        },
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// Each built-in style's signature (og-taste §1's table), for `og style list`.
const ABOUT: &[(&str, &str)] = &[
    (
        "cartoon",
        "thick outlines, cream panels, squash on press, pops (casual, match-3, party, idle)",
    ),
    (
        "fantasy",
        "parchment, gold corners and flourishes, a serif display (RPGs, card battlers, quests)",
    ),
    (
        "neon",
        "dark glass, glowing tubes, italic display (arcade, rhythm, racing, synthwave)",
    ),
    (
        "paper",
        "off-white sheets, ink outlines, soft shadows, one red (narrative, puzzle, cosy, word games)",
    ),
    (
        "pixel",
        "square frames, 2 px bevels, hard shadows, a bitmap face (platformers, roguelikes, retro)",
    ),
    (
        "scifi",
        "hairlines, cut corners, brackets, scanlines, mono numerals (space, tactics, hacking, mech)",
    ),
    // The seed styles og style new generated (no atlas: tokens and
    // procedural dressings), by look family.
    (
        "gilded",
        "ornate fantasy: notched gold-lit frames, corner ornaments, heavy motion",
    ),
    (
        "storybook",
        "storybook: soft paper sheets, ink lines, ornaments, floaty motion",
    ),
    (
        "tactical",
        "tactical military: cut corners, flat olive, tracked capitals, mechanical motion",
    ),
    (
        "holo",
        "sci-fi holographic: cut glowing frames, inner glow, snappy motion",
    ),
    (
        "cyber",
        "cyberpunk neon: notched neon tubes on black, snappy motion",
    ),
    (
        "bubbly",
        "casual cartoon: chunky round buttons, lit gradients, hard shadows, bouncy motion",
    ),
    (
        "kawaii",
        "kawaii gacha: pastel capsules, frosted glass, bouncy motion",
    ),
    (
        "arcade",
        "pixel retro: square bevels, 2 px lines, mechanical motion",
    ),
    (
        "craft",
        "paper and craft: round cut paper, dashed fields, snappy motion",
    ),
    (
        "dread",
        "diegetic horror: sharp ink-stained sheets, heavy motion, low sounds",
    ),
    (
        "broadcast",
        "sports broadcast: sharp lit bars, tracked capitals, snappy motion",
    ),
    (
        "inkwash",
        "ink and eastern brush: soft washes, brush lines, calm motion",
    ),
];

fn style_name_ok(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.starts_with(['@', '.', '-'])
        || name.contains(['/', '\\', '"'])
        || name.chars().any(char::is_whitespace)
    {
        return Err(format!(
            "'{name}' is not a style name: one directory name under assets/styles/ (letters, digits, - and _)"
        ));
    }
    Ok(())
}

/// The licence note of a generated style: what it borrows and from where.
fn generated_licences(c: &og_ui::style_gen::StyleChoices) -> String {
    let mut s = format!(
        "# {}\n\nGenerated by `og style new` (docs/ui.md §12.5) for the {} look: its\n`style.ogt` and `sfx-*.ogt` are the engine's own work, under the\nengine's licence. Its sounds are synthesised by the cook from `type sfx`\npresets; no recording is bundled.\n\nIt draws with built-in fonts and icons, which stay where they are and\nkeep their licences (each pack's `LICENSES.md` and `OFL.txt`):\n\n",
        c.name, c.look
    );
    let mut refs: Vec<&str> = c.fonts.iter().map(String::as_str).collect();
    if let Some(i) = &c.icons {
        refs.push(i);
    }
    refs.sort();
    refs.dedup();
    for r in refs {
        s.push_str(&format!("- `{r}`\n"));
    }
    s
}

/// Writes a generated style's files into `dir`.
fn write_generated(
    dir: &Path,
    c: &og_ui::style_gen::StyleChoices,
    force: bool,
) -> Result<Vec<String>, String> {
    let g = og_ui::style_gen::generate(c)?;
    let requests = serde_json::to_string_pretty(&og_ui::style_gen::art_requests_json(c))
        .map_err(|e| e.to_string())?;
    let mut files: Vec<(String, String)> = vec![("style.ogt".into(), g.text)];
    files.extend(g.sounds);
    files.push(("LICENSES.md".into(), generated_licences(c)));
    files.push(("art-requests.json".into(), format!("{requests}\n")));
    let existing: Vec<&str> = files
        .iter()
        .filter(|(f, _)| dir.join(f).exists())
        .map(|(f, _)| f.as_str())
        .collect();
    if !existing.is_empty() && !force {
        return Err(format!(
            "{}/ already has {}; nothing written (--force overwrites)",
            dir.display(),
            existing.join(", ")
        ));
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for (f, text) in &files {
        let path = dir.join(f);
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(files.into_iter().map(|(f, _)| f).collect())
}

/// `og style new`.
fn new(args: &[&str]) -> Result<(), String> {
    use og_ui::style_gen::{Finish, Motion, Ornament, ShapeLanguage, StyleChoices, seeds};
    let usage = USAGE
        .lines()
        .filter(|l| l.contains(" new "))
        .collect::<Vec<_>>()
        .join("\n");
    let mut it = args.iter();
    let (mut name, mut look, mut premise) = (None, None, None);
    let (mut project, mut out, mut force, mut all) = (PathBuf::from("."), None, false, false);
    let mut flags: Vec<(&str, &str)> = Vec::new();
    while let Some(a) = it.next() {
        match *a {
            "--force" => force = true,
            "--seeds" => all = true,
            "--project" => project = PathBuf::from(it.next().ok_or(usage.clone())?),
            "--out" => out = Some(PathBuf::from(it.next().ok_or(usage.clone())?)),
            "--look" => look = Some(*it.next().ok_or(usage.clone())?),
            "--premise" => premise = Some(*it.next().ok_or(usage.clone())?),
            f @ ("--seed" | "--scheme" | "--shape" | "--ornament" | "--motion" | "--finish") => {
                flags.push((f, it.next().ok_or(usage.clone())?));
            }
            a if a.starts_with('-') => return Err(usage),
            a if name.is_none() => name = Some(a),
            _ => return Err(usage),
        }
    }
    if all {
        if name.is_some() || look.is_some() || !flags.is_empty() {
            return Err(usage);
        }
        let root = out.unwrap_or_else(|| PathBuf::from("templates/styles"));
        for c in seeds() {
            let dir = root.join(&c.name);
            let files = write_generated(&dir, &c, true)?;
            println!(
                "og style new: {} ({}) -> {}/ ({} files)",
                c.name,
                c.look,
                dir.display(),
                files.len()
            );
        }
        return Ok(());
    }
    let name = name.ok_or(usage.clone())?;
    style_name_ok(name)?;
    let look = look.unwrap_or("minimal modern");
    let mut c = StyleChoices::for_look(name, look, premise.unwrap_or("a game"));
    for (flag, v) in flags {
        let bad = |what: &str, names: String| format!("{flag} {v}: not a {what} ({names})");
        match flag {
            "--seed" => {
                c.seed = og_ui::style_file::parse_color(v)
                    .ok_or_else(|| format!("--seed {v}: a colour, #rrggbb"))?;
                // A new seed recolours everything round it.
                c.ground = None;
                c.accent = None;
            }
            "--scheme" => {
                let dark = match v {
                    "dark" => true,
                    "light" => false,
                    _ => return Err(bad("scheme", "dark, light".into())),
                };
                if dark != (c.scheme == og_ui::Scheme::Dark) {
                    c.ground = None;
                }
                c.scheme = if dark {
                    og_ui::Scheme::Dark
                } else {
                    og_ui::Scheme::Light
                };
            }
            "--shape" => {
                c.shape = ShapeLanguage::from_name(v)
                    .ok_or_else(|| bad("shape language", ShapeLanguage::names()))?
            }
            "--ornament" => {
                c.ornament = Ornament::from_name(v)
                    .ok_or_else(|| bad("ornament level", Ornament::names()))?
            }
            "--motion" => {
                c.motion = Motion::from_name(v)
                    .ok_or_else(|| bad("motion personality", Motion::names()))?
            }
            _ => c.finish = Finish::from_name(v).ok_or_else(|| bad("finish", Finish::names()))?,
        }
    }
    let (dir, shown) = match out {
        Some(d) => (d.clone(), d.display().to_string()),
        None => {
            if !project.join("project.ogt").is_file() {
                return Err(format!(
                    "{}: no project.ogt; run it in a game's directory, pass --project <dir> or --out <dir>",
                    project.display()
                ));
            }
            let rel = format!("assets/styles/{name}");
            (project.join(&rel), rel)
        }
    };
    let files = write_generated(&dir, &c, force)?;
    println!(
        "og style new: '{name}', a {} style (shape {}, ornament {}, motion {}, finish {}, seed {}), into {shown}/",
        c.look,
        c.shape.name(),
        c.ornament.name(),
        c.motion.name(),
        c.finish.name(),
        og_ui::tokens::hex(c.seed)
    );
    for f in &files {
        println!("  {shown}/{f}");
    }
    let requests = og_ui::style_gen::art_requests(&c);
    println!(
        "its art slots ({}), drawn procedurally until their art lands (art-requests.json, the art brief's styleArtBriefs):",
        requests.len()
    );
    for r in &requests {
        println!("  {} ({})", r.name, r.kind.name());
    }
    println!("name it in the game's look: ui \"styles/{name}/style.ogt\"");
    Ok(())
}

/// `og style eject <slug> [<name>] [--project <dir>] [--force]`.
fn eject(args: &[&str]) -> Result<(), String> {
    let usage = "usage: og style eject <slug> [<name>] [--project <dir>] [--force]";
    let (mut words, mut project, mut force) = (Vec::new(), PathBuf::from("."), false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match *a {
            "--force" => force = true,
            "--project" => project = PathBuf::from(it.next().ok_or(usage)?),
            a if a.starts_with('-') => return Err(usage.into()),
            a => words.push(a),
        }
    }
    let (slug, name) = match words.as_slice() {
        [slug] => (*slug, *slug),
        [slug, name] => (*slug, *name),
        _ => return Err(usage.into()),
    };
    let slugs = og_cook::cook::look::style_slugs();
    if !slugs.contains(&slug) {
        return Err(format!(
            "no built-in style '{slug}' (built-in: {})",
            slugs.join(", ")
        ));
    }
    if name.is_empty()
        || name.starts_with(['@', '.'])
        || name.contains(['/', '\\', '"'])
        || name.chars().any(char::is_whitespace)
    {
        return Err(format!(
            "'{name}' is not a style name: one directory name under assets/styles/ (letters, digits, - and _)"
        ));
    }
    if !project.join("project.ogt").is_file() {
        return Err(format!(
            "{}: no project.ogt; run it in a game's directory or pass --project <dir>",
            project.display()
        ));
    }
    let prefix = format!("@styles/{slug}/");
    let mut files: Vec<(&str, &[u8])> =
        og_cook::cook::look::style_files(&format!("{prefix}style.ogt"));
    files.extend(
        og_cook::cook::look::STYLE_LICENCES
            .iter()
            .filter(|(id, _)| id.starts_with(&prefix))
            .copied(),
    );
    files.sort();
    let rel_dir = format!("assets/styles/{name}");
    let dir = project.join(&rel_dir);
    let targets: Vec<(String, &[u8])> = files
        .iter()
        .map(|(id, bytes)| (id[prefix.len()..].to_string(), *bytes))
        .collect();
    let existing: Vec<&str> = targets
        .iter()
        .filter(|(f, _)| dir.join(f).exists())
        .map(|(f, _)| f.as_str())
        .collect();
    if !existing.is_empty() && !force {
        return Err(format!(
            "{rel_dir}/ already has {}; nothing written (--force overwrites)",
            existing.join(", ")
        ));
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    println!(
        "og style eject: the built-in '{slug}' into {rel_dir}/ ({} files)",
        targets.len()
    );
    for (f, bytes) in &targets {
        let path = dir.join(f);
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("  {rel_dir}/{f}");
    }
    println!(
        "name it in the game's look: ui \"styles/{name}/style.ogt\"\nthen change its palette roles, radius, ramp and fonts in {rel_dir}/style.ogt (og-ui §6); its cjk fonts stay the built-in @styles/cjk/ ones"
    );
    Ok(())
}

/// `og style upgrade <style.ogt> [--out <file>] [--check]`.
fn upgrade(args: &[&str]) -> Result<(), String> {
    let usage = "usage: og style upgrade <style.ogt> [--out <file>] [--check]";
    let (path, out, check) = match args {
        [p] => (*p, *p, false),
        [p, "--check"] => (*p, *p, true),
        [p, "--out", o] => (*p, *o, false),
        _ => return Err(usage.into()),
    };
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let file = og_ui::parse_style(&text).map_err(|e| format!("{path}: {e}"))?;
    if file.version >= 2 {
        println!(
            "og style upgrade: {path} is version {} already",
            file.version
        );
        return Ok(());
    }
    if check {
        return Err(format!(
            "{path} is version 1: run `og style upgrade {path}` (version 1 loading is removed in a later minor version)"
        ));
    }
    let upgraded = og_ui::upgrade_style(&text).map_err(|e| format!("{path}: {e}"))?;
    std::fs::write(out, &upgraded).map_err(|e| format!("{out}: {e}"))?;
    println!(
        "og style upgrade: {path} -> {out} (version 2: {} dressing(s); its new tokens are derived, so make them the style's own: colors, typography, shape, elevation, motion, sounds)",
        file.dressings.len()
    );
    Ok(())
}

fn pack(args: &[&str]) -> Result<(), String> {
    let (dir, out) = match args {
        [dir] => (PathBuf::from(dir), PathBuf::from(dir)),
        [dir, "--out", out] => (PathBuf::from(dir), PathBuf::from(out)),
        _ => return Err(USAGE.into()),
    };
    let mut pieces = read_pieces(&dir)?;
    if pieces.is_empty() {
        return Err(format!(
            "{} holds no PNG: draw one nine-patch per part (panel.9.png, button.9.png, ...)",
            dir.display()
        ));
    }
    let (w, h) = shelve(&mut pieces);
    let atlas = compose(&pieces, w, h);
    std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let atlas_path = out.join("atlas.png");
    atlas
        .save(&atlas_path)
        .map_err(|e| format!("{}: {e}", atlas_path.display()))?;
    let import = out.join("atlas.png.ogt");
    if !import.exists() {
        std::fs::write(
            &import,
            "type import\nversion 1\n---\n# A style atlas drawn by hand (og style pack).\ntexture {\n  kind color\n}\n",
        )
        .map_err(|e| format!("{}: {e}", import.display()))?;
    }
    let style_path = out.join("style.ogt");
    let old = std::fs::read_to_string(&style_path).ok();
    let name = out
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("style")
        .to_string();
    // Beside the style file: ids in a style resolve relative to it first, so
    // the pack stays whole when its directory is copied or renamed.
    let text = style_text(old.as_deref(), &name, "atlas.png", &pieces, (w, h));
    std::fs::write(&style_path, text).map_err(|e| format!("{}: {e}", style_path.display()))?;
    let slices = pieces.iter().filter(|p| p.insets.is_some()).count();
    println!(
        "og style pack: {} images ({slices} nine-slices) into {} ({w}×{h}); {} {}",
        pieces.len(),
        atlas_path.display(),
        style_path.display(),
        if old.is_some() { "updated" } else { "written" }
    );
    Ok(())
}

fn read_pieces(dir: &Path) -> Result<Vec<Piece>, String> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension().is_some_and(|x| x == "png")
                && p.file_name().is_some_and(|n| n != "atlas.png")
        })
        .collect();
    files.sort();
    let mut out = Vec::new();
    for path in files {
        let file = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let image = image::open(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?
            .to_rgba8();
        if let Some(name) = file.strip_suffix(".9.png") {
            if !SLICES.contains(&name) {
                return Err(format!(
                    "{file}: a nine-patch is named for a part ({}), not '{name}'",
                    SLICES.join(", ")
                ));
            }
            let (art, insets) = nine_patch(&image).map_err(|e| format!("{file}: {e}"))?;
            out.push(Piece {
                name: name.into(),
                image: art,
                insets: Some(insets),
                at: (0, 0),
            });
        } else {
            let name = file.trim_end_matches(".png");
            if SLICES.contains(&name) {
                return Err(format!(
                    "{file}: a slice's image is a nine-patch ({name}.9.png) whose one-pixel border marks what stretches"
                ));
            }
            out.push(Piece {
                name: name.into(),
                image,
                insets: None,
                at: (0, 0),
            });
        }
    }
    Ok(out)
}

/// A marker pixel of a nine-patch's border: opaque and dark.
fn marked(p: &Rgba<u8>) -> bool {
    p[3] >= 128 && (p[0] as u32 + p[1] as u32 + p[2] as u32) < 3 * 64
}

/// The art inside a nine-patch's border, and its insets (left, top, right,
/// bottom): the columns and rows outside the stretch marks.
fn nine_patch(img: &RgbaImage) -> Result<(RgbaImage, [u32; 4]), String> {
    let (w, h) = img.dimensions();
    if w < 3 || h < 3 {
        return Err("too small for a nine-patch (a one-pixel border around the art)".into());
    }
    let span = |marks: Vec<bool>| -> Option<(u32, u32)> {
        let first = marks.iter().position(|m| *m)? as u32;
        let last = marks.iter().rposition(|m| *m)? as u32;
        Some((first, last))
    };
    let cols: Vec<bool> = (1..w - 1).map(|x| marked(img.get_pixel(x, 0))).collect();
    let rows: Vec<bool> = (1..h - 1).map(|y| marked(img.get_pixel(0, y))).collect();
    let (iw, ih) = (w - 2, h - 2);
    let (c0, c1) = span(cols).ok_or(
        "its top border marks no column that stretches (opaque dark pixels in the first row)",
    )?;
    let (r0, r1) = span(rows).ok_or(
        "its left border marks no row that stretches (opaque dark pixels in the first column)",
    )?;
    let art = image::imageops::crop_imm(img, 1, 1, iw, ih).to_image();
    Ok((art, [c0, r0, iw - 1 - c1, ih - 1 - r1]))
}

/// Places the pieces on shelves, tallest first, in an atlas as wide as the
/// next power of two over the square of their area (or the widest piece).
fn shelve(pieces: &mut [Piece]) -> (u32, u32) {
    let area: u64 = pieces
        .iter()
        .map(|p| u64::from(p.image.width() + GAP) * u64::from(p.image.height() + GAP))
        .sum();
    let widest = pieces
        .iter()
        .map(|p| p.image.width() + GAP)
        .max()
        .unwrap_or(1);
    let side = ((area as f64).sqrt().ceil() as u32).max(widest);
    let width = side.next_power_of_two();
    let mut order: Vec<usize> = (0..pieces.len()).collect();
    order.sort_by_key(|&i| {
        (
            std::cmp::Reverse(pieces[i].image.height()),
            pieces[i].name.clone(),
        )
    });
    let (mut x, mut y, mut shelf) = (GAP / 2, GAP / 2, 0);
    for i in order {
        let (w, h) = pieces[i].image.dimensions();
        if x + w + GAP / 2 > width {
            x = GAP / 2;
            y += shelf + GAP;
            shelf = 0;
        }
        pieces[i].at = (x, y);
        x += w + GAP;
        shelf = shelf.max(h);
    }
    (width, y + shelf + GAP / 2)
}

/// The atlas: each piece at its place, its edge pixels extruded one pixel
/// into the gap around it.
fn compose(pieces: &[Piece], w: u32, h: u32) -> RgbaImage {
    let mut atlas = RgbaImage::new(w, h);
    for p in pieces {
        let (px, py) = p.at;
        let (iw, ih) = p.image.dimensions();
        let e = GAP / 2;
        for y in 0..ih + 2 * e {
            for x in 0..iw + 2 * e {
                let sx = (x as i64 - e as i64).clamp(0, iw as i64 - 1) as u32;
                let sy = (y as i64 - e as i64).clamp(0, ih as i64 - 1) as u32;
                let (ax, ay) = (px + x - e, py + y - e);
                if ax < w && ay < h {
                    atlas.put_pixel(ax, ay, *p.image.get_pixel(sx, sy));
                }
            }
        }
    }
    atlas
}

fn region(p: &Piece) -> String {
    format!(
        "{} {} {} {}",
        p.at.0,
        p.at.1,
        p.image.width(),
        p.image.height()
    )
}

/// The style file: `old` with its atlas, slices and ornament regions
/// replaced, or a new style of the slices.
fn style_text(
    old: Option<&str>,
    name: &str,
    atlas_id: &str,
    pieces: &[Piece],
    size: (u32, u32),
) -> String {
    let atlas_line = format!("atlas \"{atlas_id}\" {} {}", size.0, size.1);
    // An existing slice line's words (tint or authored, stretch, repeat or
    // round) stay with its part.
    let modes = |part: &str| -> String {
        old.into_iter()
            .flat_map(str::lines)
            .map(str::split_whitespace)
            .filter_map(|mut w| {
                (w.next() == Some("slice") && w.next() == Some(part)).then(|| {
                    w.filter(|v| v.parse::<f64>().is_err())
                        .map(|v| format!(" {v}"))
                        .collect::<String>()
                })
            })
            .next()
            .unwrap_or_default()
    };
    let slice_line = |p: &Piece| {
        let [l, t, r, b] = p.insets.expect("a nine-patch");
        if p.name == "soft" {
            // The soft rectangle takes one inset (its blur).
            format!("soft {} {}", region(p), l.max(t).max(r).max(b))
        } else {
            format!(
                "slice {} {} {l} {t} {r} {b}{}",
                p.name,
                region(p),
                modes(&p.name)
            )
        }
    };
    let mut slices: Vec<&Piece> = pieces.iter().filter(|p| p.insets.is_some()).collect();
    slices.sort_by_key(|p| SLICES.iter().position(|s| *s == p.name));
    if let Some(old) = old
        && let Ok(doc) = og_core::ogt::Document::parse(old)
        && doc.version >= 2
    {
        return style_text_v2(doc, atlas_id, pieces, size);
    }
    let Some(old) = old else {
        let mut l = vec![
            "type style".to_string(),
            "version 1".into(),
            "---".into(),
            "# Packed by og style pack from the parts drawn by hand. The palette and radii start as the default style's: make them the game's, then add fonts and motion."
                .into(),
            format!("name \"{name}\""),
            "scheme dark".into(),
            atlas_line,
        ];
        let t = og_ui::Theme::dark();
        l.push("palette {".into());
        for n in og_ui::Palette::NAMES {
            let c = t.palette.get(n).expect("a role");
            let [r, g, b] = c.rgb;
            let a = if c.alpha >= 1.0 {
                String::new()
            } else {
                format!("{:02x}", (c.alpha * 255.0).round() as u8)
            };
            l.push(format!("  {n} \"#{r:02x}{g:02x}{b:02x}{a}\""));
        }
        l.push("}".into());
        let r = t.radius;
        l.push(format!(
            "radius {{\n  small {}\n  control {}\n  bar {}\n  panel {}\n}}",
            r.small, r.control, r.bar, r.panel
        ));
        l.extend(slices.iter().map(|p| slice_line(p)));
        return l.join("\n") + "\n";
    };
    let mut out: Vec<String> = Vec::new();
    let mut placed = false;
    for line in old.lines() {
        let t = line.trim_start();
        let key = t.split_whitespace().next().unwrap_or("");
        match key {
            "atlas" => out.push(atlas_line.clone()),
            "slice" | "soft" => {
                // Rewritten from the nine-patches, where the first one stood.
                if !placed {
                    out.extend(slices.iter().map(|p| slice_line(p)));
                    placed = true;
                }
            }
            "corner" | "divider" | "flourish" | "crest" | "edge" | "tile" => {
                match pieces.iter().find(|p| p.name == key && p.insets.is_none()) {
                    Some(p) => {
                        let rest: Vec<&str> = t.split_whitespace().skip(5).collect();
                        out.push(format!("{key} {} {}", region(p), rest.join(" ")));
                    }
                    None => out.push(line.to_string()),
                }
            }
            _ => out.push(line.to_string()),
        }
    }
    if !placed {
        out.extend(slices.iter().map(|p| slice_line(p)));
    }
    out.join("\n") + "\n"
}

/// A version 2 style file with its atlas, its parts' `slice` lines (in
/// their `dress` blocks, made when missing) and its ornament regions
/// replaced from the packed pieces.
fn style_text_v2(
    mut doc: og_core::ogt::Document,
    atlas_id: &str,
    pieces: &[Piece],
    size: (u32, u32),
) -> String {
    use og_core::ogt::{Block, Entry, Number, Value};
    let n = |v: u32| Value::Number(Number::from_f64(f64::from(v)).expect("finite"));
    let region = |p: &Piece| {
        vec![
            n(p.at.0),
            n(p.at.1),
            n(p.image.width()),
            n(p.image.height()),
        ]
    };
    let body = &mut doc.body;
    let mut atlas = Entry::new("atlas");
    atlas.values = vec![Value::String(atlas_id.into()), n(size.0), n(size.1)];
    match body.entries.iter_mut().find(|e| e.key == "atlas") {
        Some(e) => *e = atlas,
        None => body.entries.insert(0, atlas),
    }
    for p in pieces {
        match (p.insets, p.name.as_str()) {
            (Some([l, t, r, b]), "soft") => {
                let mut e = Entry::new("soft");
                e.values = region(p);
                e.values.push(n(l.max(t).max(r).max(b)));
                match body.entries.iter_mut().find(|e| e.key == "soft") {
                    Some(old) => *old = e,
                    None => body.push(e),
                }
            }
            (Some([l, t, r, b]), name) => {
                // `capsule` is the HUD capsule's nine-patch.
                let part =
                    og_ui::StylePart::from_name(if name == "capsule" { "hud" } else { name })
                        .expect("a nine-patch names a part");
                let sel = part.selector().to_string();
                let i = match body.entries.iter().position(|e| {
                    e.key == "dress"
                        && e.values.first().and_then(Value::as_str) == Some(sel.as_str())
                }) {
                    Some(i) => i,
                    None => {
                        body.push(
                            Entry::new("dress")
                                .with(Value::String(sel))
                                .with_block(Block::new()),
                        );
                        body.entries.len() - 1
                    }
                };
                let block = body.entries[i].block.get_or_insert_with(Block::new);
                let mut e = Entry::new("slice");
                e.values = region(p);
                e.values.extend([n(l), n(t), n(r), n(b)]);
                match block.entries.iter_mut().find(|e| e.key == "slice") {
                    Some(old) => {
                        // Its words (tint or authored, stretch, repeat or round) stay.
                        e.values
                            .extend(old.values.iter().filter(|v| v.as_word().is_some()).cloned());
                        *old = e;
                    }
                    None => block.entries.insert(0, e),
                }
            }
            (None, key @ ("corner" | "divider" | "flourish" | "crest" | "edge" | "tile")) => {
                for e in body.entries.iter_mut().filter(|e| e.key == key) {
                    let rest: Vec<Value> = e.values.iter().skip(4).cloned().collect();
                    e.values = region(p);
                    e.values.extend(rest);
                }
            }
            _ => {}
        }
    }
    doc.to_text().expect("a style file writes back")
}
