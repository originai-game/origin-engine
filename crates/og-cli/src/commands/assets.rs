//! Asset queries and source moves.
use og_cook::cook::assets::Index;
use serde_json::json;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(args: &[&str]) -> ExitCode {
    match query(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("og assets: {e}");
            ExitCode::from(2)
        }
    }
}

fn query(args: &[&str]) -> Result<(), String> {
    if let ["mv", rest @ ..] = args {
        return move_asset(rest);
    }
    let mut root = PathBuf::from(".");
    let (mut kind, mut group, mut text) = (None, None, None);
    let (mut unused, mut refs, mut by, mut dupes, mut json_output) =
        (false, false, false, false, false);
    let mut near = None;
    let mut args = args.iter().copied();
    while let Some(arg) = args.next() {
        match arg {
            "--project" => root = PathBuf::from(args.next().ok_or("--project needs a directory")?),
            "--kind" => kind = Some(args.next().ok_or("--kind needs an asset kind")?),
            "--group" => group = Some(args.next().ok_or("--group needs a streaming group")?),
            "--unused" => unused = true,
            "--refs" => refs = true,
            "--by" => by = true,
            "--dupes" => dupes = true,
            "--json" => json_output = true,
            "--near" => {
                let value: u32 = args
                    .next()
                    .ok_or("--near needs a Hamming distance (0..64)")?
                    .parse()
                    .map_err(|_| "--near needs a Hamming distance (0..64)")?;
                if value > 64 {
                    return Err("--near must be 0..64".into());
                }
                near = Some(value);
            }
            flag if flag.starts_with('-') => return Err(format!("unknown option {flag}")),
            value => {
                if text.replace(value).is_some() {
                    return Err("supply one search text".into());
                }
            }
        }
    }
    if near.is_some() && !dupes {
        return Err("--near requires --dupes".into());
    }
    let index = Index::read(&root)?;
    let reachable = index.reachable();
    let text = text.map(str::to_lowercase);
    let rows: Vec<_> = index
        .assets
        .iter()
        .filter(|a| {
            kind.is_none_or(|k| a.kind == k)
                && group.is_none_or(|g| a.group.as_deref() == Some(g))
                && (!unused || !reachable.contains(&a.id))
                && text.as_ref().is_none_or(|t| {
                    a.id.to_lowercase().contains(t) || a.source.to_lowercase().contains(t)
                })
        })
        .collect();
    let mut duplicates = Vec::new();
    if dupes {
        for (i, a) in rows.iter().enumerate() {
            for b in &rows[i + 1..] {
                let distance = if a.file == b.file {
                    Some(0)
                } else if a.kind == b.kind {
                    a.perceptual
                        .zip(b.perceptual)
                        .map(|(x, y)| (x ^ y).count_ones())
                } else {
                    None
                };
                if a.file == b.file || near.is_some_and(|n| distance.is_some_and(|d| d <= n)) {
                    duplicates.push(json!({"left": a.id, "right": b.id, "exact": a.file == b.file, "distance": distance}));
                }
            }
        }
    }
    if json_output {
        let assets: Vec<_> = rows
            .iter()
            .map(|a| {
                let mut value = json!(a);
                value["unused"] = json!(!reachable.contains(&a.id));
                value["perceptual"] = json!(a.perceptual.map(|h| format!("{h:016x}")));
                if !refs {
                    value.as_object_mut().expect("asset object").remove("refs");
                }
                if by {
                    value["by"] = json!(index.referenced_by(&a.id));
                }
                value
            })
            .collect();
        println!(
            "{}",
            json!({"assets": assets, "duplicates": duplicates, "computed": index.computed})
        );
    } else {
        if dupes {
            for pair in duplicates {
                println!(
                    "{}\t{}\t{}\tdistance {}",
                    pair["left"].as_str().unwrap(),
                    pair["right"].as_str().unwrap(),
                    if pair["exact"] == true {
                        "identical"
                    } else {
                        "near"
                    },
                    pair["distance"]
                );
            }
        } else {
            for a in rows {
                println!(
                    "{}\t{}\t{} bytes\t{}{}",
                    a.id,
                    a.kind,
                    a.bytes,
                    a.group.as_deref().unwrap_or("boot"),
                    if reachable.contains(&a.id) {
                        ""
                    } else {
                        "\tunused"
                    }
                );
                if refs {
                    for id in &a.refs {
                        println!("  -> {id}");
                    }
                }
                if by {
                    for id in index.referenced_by(&a.id) {
                        println!("  <- {id}");
                    }
                }
            }
        }
        for c in &index.computed {
            eprintln!(
                "{}:{}: {}; retaining {}",
                c.source,
                c.line,
                c.reason,
                if c.prefix.is_empty() {
                    "all assets".into()
                } else {
                    format!("prefix {:?}", c.prefix)
                }
            );
        }
    }
    Ok(())
}

fn move_asset(args: &[&str]) -> Result<(), String> {
    let mut root = PathBuf::from(".");
    let mut json_output = false;
    let mut paths = Vec::new();
    let mut args = args.iter().copied();
    while let Some(arg) = args.next() {
        match arg {
            "--project" => root = PathBuf::from(args.next().ok_or("--project needs a directory")?),
            "--json" => json_output = true,
            flag if flag.starts_with('-') => return Err(format!("unknown option {flag}")),
            path => paths.push(path),
        }
    }
    let [from, to] = paths.as_slice() else {
        return Err("usage: og assets mv <from> <to> [--project DIR] [--json]".into());
    };
    let report = og_cook::cook::asset_move::move_asset(&root, from, to)?;
    if json_output {
        println!(
            "{}",
            serde_json::to_string(&report).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "moved {} to {}; rewrote {} files; run og cook to refresh cooked outputs",
            report.from,
            report.to,
            report.rewritten.len()
        );
        for path in report.rewritten {
            println!("  {path}");
        }
        for warning in report.warnings {
            eprintln!("warning: {warning}");
        }
    }
    Ok(())
}
