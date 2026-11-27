//! Delivery dependency loading, without recursive calls on the native stack.

use super::*;

pub(super) fn poll(assets: &mut Assets, root: usize) {
    let mut work = vec![(root, false)];
    let mut active = BTreeSet::new();
    while let Some((i, finish)) = work.pop() {
        if assets.entries[i].status != Status::Pending {
            continue;
        }
        if finish {
            active.remove(&i);
            let dependencies = assets.required.get(&i).cloned().unwrap_or_default();
            if let Some(error) =
                dependencies
                    .iter()
                    .find_map(|&j| match &assets.entries[j].status {
                        Status::Failed(e) => Some(format!(
                            "{} requires {}: {e}",
                            assets.entries[i].path, assets.entries[j].path
                        )),
                        _ => None,
                    })
            {
                assets.entries[i].status = Status::Failed(error);
                assets.entries[i].payload = Payload::None;
                continue;
            }
            if dependencies
                .iter()
                .any(|&j| assets.entries[j].status == Status::Pending)
            {
                continue;
            }
            let Payload::Bundle { bundle, dir } = &assets.entries[i].payload else {
                continue;
            };
            if let Err(e) = assets.stream.add_bundle(i, bundle, dir) {
                assets.entries[i].status = Status::Failed(e);
                assets.entries[i].payload = Payload::None;
            } else {
                crate::world::needs::bundle(bundle);
                assets.entries[i].status = Status::Ready;
            }
            continue;
        }
        if !active.insert(i) {
            assets.entries[i].status = Status::Failed(format!(
                "{}: delivery dependency cycle",
                assets.entries[i].path
            ));
            continue;
        }
        if !matches!(assets.entries[i].payload, Payload::Bundle { .. }) {
            let dir = join(&assets.root, &assets.entries[i].path);
            let manifest = format!("{dir}/{MANIFEST}");
            let got = match assets.pinned.get(&i) {
                Some(pin) => read_pinned(&manifest, pin),
                None => read(&manifest),
            };
            let bundle = match got {
                Fetch::Pending => {
                    active.remove(&i);
                    continue;
                }
                Fetch::Failed(e) => {
                    assets.entries[i].status = Status::Failed(format!("{manifest}: {e}"));
                    active.remove(&i);
                    continue;
                }
                Fetch::Ready(bytes) => match String::from_utf8(bytes)
                    .map_err(|e| e.to_string())
                    .and_then(|t| Bundle::from_manifest(&t).map_err(|e| e.to_string()))
                {
                    Ok(bundle) => bundle,
                    Err(e) => {
                        assets.entries[i].status =
                            Status::Failed(format!("invalid: {manifest}: {e}"));
                        active.remove(&i);
                        continue;
                    }
                },
            };
            let occupied: BTreeMap<_, _> = assets
                .entries
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .flat_map(|(_, entry)| {
                    let ids: Vec<_> = match &entry.payload {
                        Payload::Bundle { bundle, .. } => bundle
                            .manifest()
                            .outputs()
                            .map(|o| (o.id.to_lowercase(), entry.path.as_str()))
                            .collect(),
                        _ => Vec::new(),
                    };
                    ids
                })
                .collect();
            let duplicate = bundle.manifest().outputs().find_map(|o| {
                occupied.get(&o.id.to_lowercase()).map(|path| {
                    format!(
                        "{manifest}: duplicate global asset identity {} in {path}",
                        o.id
                    )
                })
            });
            if let Some(error) = duplicate {
                assets.entries[i].status = Status::Failed(error);
                active.remove(&i);
                continue;
            }
            assets.entries[i].payload = Payload::Bundle {
                bundle: Box::new(bundle),
                dir,
            };
        }
        if !assets.required.contains_key(&i) {
            let Payload::Bundle { bundle, dir } = &assets.entries[i].payload else {
                continue;
            };
            let requirements = bundle.manifest().requires.clone();
            let dir = dir.clone();
            let mut dependencies = Vec::new();
            let mut error = None;
            for requirement in &requirements {
                let path = match dependency_dir(&dir, &requirement.file) {
                    Ok(path) => path,
                    Err(e) => {
                        error = Some(e);
                        break;
                    }
                };
                let j = assets
                    .opened(&path)
                    .unwrap_or_else(|| assets.push(&path, Status::Pending, Payload::None));
                if let Some(pin) = assets.pinned.get(&j) {
                    if pin != &requirement.integrity {
                        error = Some(format!(
                            "integrity: {path}: conflicting delivery manifest pins"
                        ));
                        break;
                    }
                } else {
                    // Already-open manifests must match this pin as well.
                    if matches!(assets.entries[j].payload, Payload::Bundle { .. }) {
                        match read_pinned(&format!("{path}/{MANIFEST}"), &requirement.integrity) {
                            Fetch::Ready(_) => {}
                            Fetch::Pending => {
                                active.remove(&i);
                                return;
                            }
                            Fetch::Failed(e) => {
                                error = Some(e);
                                break;
                            }
                        }
                    }
                    assets.pinned.insert(j, requirement.integrity.clone());
                }
                dependencies.push(j);
            }
            if let Some(error) = error {
                assets.entries[i].status = Status::Failed(error);
                assets.entries[i].payload = Payload::None;
                active.remove(&i);
                continue;
            }
            assets.required.insert(i, dependencies);
        }
        work.push((i, true));
        for &j in assets.required.get(&i).into_iter().flatten().rev() {
            if active.contains(&j) {
                assets.entries[j].status = Status::Failed("delivery dependency cycle".into());
                assets.entries[j].payload = Payload::None;
            } else if assets.entries[j].status == Status::Pending {
                work.push((j, false));
            }
        }
    }
}

/// Replace the open manifest set together: a cook can move an ID between packs.
/// Validate identities before replacing either owner and re-link changed pins.
pub(super) fn reload(assets: &mut Assets) -> Result<(), String> {
    let dependencies: BTreeSet<_> = assets.required.values().flatten().copied().collect();
    let mut fresh = Vec::new();
    for (i, entry) in assets.entries.iter().enumerate() {
        let Payload::Bundle { dir, .. } = &entry.payload else {
            continue;
        };
        let path = format!("{dir}/{MANIFEST}");
        let got = match assets.pinned.get(&i).filter(|_| !dependencies.contains(&i)) {
            Some(pin) => read_pinned(&path, pin),
            None => read(&path),
        };
        let bytes = match got {
            Fetch::Pending => return Ok(()),
            Fetch::Failed(e) => return Err(format!("{path}: {e}")),
            Fetch::Ready(bytes) => bytes,
        };
        let text = String::from_utf8(bytes).map_err(|e| format!("{path}: {e}"))?;
        let bundle = Bundle::from_manifest(&text).map_err(|e| e.to_string())?;
        fresh.push((i, bundle, dir.clone()));
    }
    let mut identities = BTreeMap::new();
    for (_, bundle, dir) in &fresh {
        for output in bundle.manifest().outputs() {
            if let Some(previous) = identities.insert(output.id.to_lowercase(), dir) {
                return Err(format!(
                    "duplicate global asset identity {} in {previous} and {dir}",
                    output.id
                ));
            }
        }
    }
    assets.pinned.retain(|i, _| !dependencies.contains(i));
    let indices: Vec<_> = fresh.iter().map(|(i, ..)| *i).collect();
    for (i, bundle, dir) in fresh {
        assets.required.remove(&i);
        assets.entries[i].payload = Payload::Bundle {
            bundle: Box::new(bundle),
            dir,
        };
        assets.entries[i].status = Status::Pending;
    }
    for i in indices {
        poll(assets, i);
        if let Status::Failed(e) = &assets.entries[i].status {
            return Err(e.clone());
        }
    }
    Ok(())
}

/// Resolve relative manifest paths for files and URLs, without filesystem I/O.
fn dependency_dir(dir: &str, file: &str) -> Result<String, String> {
    let Some(relative) = file.strip_suffix(MANIFEST) else {
        return Err(format!("invalid delivery manifest path {file:?}"));
    };
    if file.starts_with('/') || file.contains([':', '\\', '?', '#']) || !relative.ends_with('/') {
        return Err(format!("invalid delivery manifest path {file:?}"));
    }
    let (prefix, path) = match dir.split_once("://") {
        Some((scheme, rest)) => match rest.split_once('/') {
            Some((host, path)) => (format!("{scheme}://{host}/"), path),
            None => (format!("{scheme}://{rest}/"), ""),
        },
        None if dir.starts_with('/') => ("/".to_owned(), dir.trim_start_matches('/')),
        None => (String::new(), dir),
    };
    let joined = format!("{path}/{relative}");
    let mut parts = Vec::new();
    for part in joined.split('/') {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|p| *p != "..") => {
                parts.pop();
            }
            ".." if !prefix.is_empty() => {
                return Err(format!("delivery path escapes root: {file}"));
            }
            part => parts.push(part),
        }
    }
    Ok(format!("{prefix}{}", parts.join("/")))
}
