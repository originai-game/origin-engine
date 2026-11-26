//! `og script`: the cook's deterministic TS executor, without a world.
use og_cook::cook::generator::{Program, Spec};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

pub fn run(args: &[&str]) -> ExitCode {
    let result = execute(args);
    match result {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("og script: {e}");
            ExitCode::FAILURE
        }
    }
}

fn execute(args: &[&str]) -> Result<serde_json::Value, String> {
    let Some(file) = args.first().filter(|s| !s.starts_with('-')) else {
        return Err("usage: og script <file.ts> [--root DIR] [--params JSON] [--seed N] [--input name=path] [--out FILE] [--json]".into());
    };
    let mut spec = Spec {
        script: (*file).into(),
        params: serde_json::json!({}),
        seed: 1,
        inputs: BTreeMap::new(),
    };
    let mut root = ".";
    let mut output = None;
    let mut it = args[1..].iter();
    while let Some(flag) = it.next() {
        if *flag == "--json" {
            continue;
        }
        let value = it.next().ok_or_else(|| format!("{flag} takes a value"))?;
        match *flag {
            "--root" => root = value,
            "--params" => {
                spec.params = serde_json::from_str(value).map_err(|e| format!("--params: {e}"))?;
                if !spec.params.is_object() {
                    return Err("--params must be a JSON object".into());
                }
            }
            "--seed" => spec.seed = value.parse().map_err(|_| "--seed is a u32")?,
            "--input" => {
                let (name, path) = value.split_once('=').ok_or("--input takes name=path")?;
                if name.is_empty() || spec.inputs.insert(name.into(), path.into()).is_some() {
                    return Err(format!("empty or duplicate input name: {name}"));
                }
            }
            "--out" => output = Some(*value),
            _ => return Err(format!("unknown option {flag}")),
        }
    }
    let program = Program::prepare(Path::new(root), "", &spec)?;
    let data = program.run()?;
    if let Some(path) = output {
        let bytes = serde_json::to_vec(&data).map_err(|e| e.to_string())?;
        og_cook::cook::cache::atomic_write(Path::new(path), &bytes)
            .map_err(|e| format!("{path}: {e}"))?;
    }
    Ok(
        serde_json::json!({"key": program.key, "output": output, "data": output.is_none().then_some(data)}),
    )
}
