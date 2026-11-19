//! Content catalog files at run time (IDL `load`: `data`, `dataJson`,
//! `dataBytes`; guide loading §14): static, versioned content the game's
//! host publishes, read by catalog, version and path from the content base
//! the host set (`og run --content <dir>`, the page's `content`).
//!
//! A version is immutable. Its manifest (`<base>/<catalog>/<version>/
//! manifest.json`) lists each file's size and integrity; the file's bytes
//! are at `<base>/<catalog>/blobs/<sha384 as hex>`. The manifest is read
//! once a session; a file is read against its integrity (natively checked
//! here, on the web by the Worker's `fetch()`), then parsed as JSON when
//! asked.
//!
//! Completion is host input, as an uncooked load's: a file is put in place
//! at a tick boundary (natively the first after the call, on the web the
//! first after it arrived) and journaled with its tick and the integrity it
//! read ([`crate::world::uncooked::Kind::Data`]). A replay puts it in place
//! at its recorded tick and requires the same integrity.
//!
//! A catalog.head answer or a `catalog` notice ([`pin`]) names a version's
//! manifest integrity; a manifest read after it is checked against it.

use crate::engine::{Engine, hooks};
use crate::world::assets::{Entry, Fetch, Payload, Status, join};
use crate::world::uncooked::{Completion, Due, Kind};
use serde_json::Value;
use std::collections::BTreeMap;

/// A file read waits this long on the web before it fails `timeout`.
const TIMEOUT_MS: f64 = 30_000.0;

/// What a ready data load holds.
pub(crate) enum Loaded {
    Json(Value),
    Bytes(Vec<u8>),
}

struct Job {
    request: u32,
    entry: usize,
    catalog: String,
    version: String,
    path: String,
    json: bool,
    max_bytes: f64,
    /// The host's clock when it was asked for; 0 for a host without one.
    asked_ms: f64,
}

/// A version's manifest: each file's size and integrity, or why it could
/// not be read.
type Manifest = Result<BTreeMap<String, (u64, String)>, String>;

/// Every data load of an engine.
#[derive(Default)]
pub(crate) struct Data {
    next: u32,
    jobs: Vec<Job>,
    manifests: BTreeMap<(String, String), Manifest>,
    /// Each version's manifest integrity, as a catalog.head answer or a
    /// `catalog` notice named it (journaled host input).
    pins: BTreeMap<(String, String), String>,
}

/// A catalog.head answer's or `catalog` notice's data, `{ catalog, version,
/// manifest }`: that version's manifest is read against `manifest` from
/// now on. Anything else is ignored; a version already read keeps it.
pub(crate) fn pin(eng: &Engine, data: &Value) {
    let (Some(catalog), Some(version), Some(manifest)) = (
        data["catalog"].as_str(),
        data["version"].as_str(),
        data["manifest"].as_str(),
    ) else {
        return;
    };
    if !name_ok(catalog, 64) || !name_ok(version, 128) || digest_hex(manifest).is_none() {
        return;
    }
    eng.assets.borrow_mut().data.pins.insert(
        (catalog.to_string(), version.to_string()),
        manifest.to_string(),
    );
}

fn name_ok(s: &str, max: usize) -> bool {
    !s.is_empty()
        && s.len() <= max
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// `og.load.data`: a pending entry, or why the desc is refused.
pub(crate) fn load(eng: &Engine, desc: &Value) -> Result<usize, String> {
    let entry = &desc["entry"];
    let text = |k: &str| entry[k].as_str().unwrap_or_default().to_string();
    let (catalog, version, path) = (text("catalog"), text("version"), text("path"));
    if !name_ok(&catalog, 64) {
        return Err(format!(
            "DataDesc.entry.catalog '{catalog}' is 1 to 64 of letters, digits, `.`, `_` and `-`"
        ));
    }
    if !name_ok(&version, 128) {
        return Err(format!(
            "DataDesc.entry.version '{version}' is 1 to 128 of letters, digits, `.`, `_` and `-`"
        ));
    }
    if path.is_empty()
        || path.len() > 1024
        || path.contains('\\')
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
    {
        return Err(format!(
            "DataDesc.entry.path '{path}' is a `/`-separated path in the version, with no empty, `.` or `..` part"
        ));
    }
    let json = match desc["format"].as_str() {
        None | Some("json") => true,
        Some("bytes") => false,
        Some(f) => return Err(format!("DataDesc.format is `json` or `bytes`, not `{f}`")),
    };
    let budget = crate::world::uncooked::Budget::from_desc(desc)?;
    let asked_ms = hooks().map_or(0.0, |h| h.now_ms());
    let mut assets = eng.assets.borrow_mut();
    let index = assets.entries.len();
    assets.entries.push(Entry {
        path: format!("{catalog}/{version}/{path}"),
        bundle: None,
        status: Status::Pending,
        payload: Payload::DataWaiting,
    });
    let data = &mut assets.data;
    let request = data.next;
    data.next += 1;
    data.jobs.push(Job {
        request,
        entry: index,
        catalog,
        version,
        path,
        json,
        max_bytes: budget.bytes,
        asked_ms,
    });
    Ok(index)
}

/// A host's read failure as a reason: `not_found` when the file is not
/// there, `failed` otherwise.
fn failure(what: &str, e: &str) -> String {
    let lower = e.to_ascii_lowercase();
    let code =
        if lower.contains("404") || lower.contains("not found") || lower.contains("no such file") {
            "not_found"
        } else {
            "failed"
        };
    format!("{code}: {what}: {e}")
}

/// The manifest's files, from its JSON.
fn manifest(text: &[u8], catalog: &str, version: &str) -> Manifest {
    let v: Value = serde_json::from_slice(text)
        .map_err(|e| format!("invalid: {catalog}/{version}/manifest.json: {e}"))?;
    let invalid = |why: &str| format!("invalid: {catalog}/{version}/manifest.json: {why}");
    if v["format"] != 1 {
        return Err(invalid("format is not 1"));
    }
    if v["catalog"] != catalog || v["version"] != version {
        return Err(invalid(&format!(
            "it names {}/{}",
            v["catalog"], v["version"]
        )));
    }
    let files = v["files"]
        .as_object()
        .ok_or_else(|| invalid("no files object"))?;
    let mut out = BTreeMap::new();
    for (path, f) in files {
        let size = f["size"]
            .as_u64()
            .ok_or_else(|| invalid(&format!("{path}: size is not a whole number")))?;
        let integrity = f["integrity"]
            .as_str()
            .filter(|s| digest_hex(s).is_some())
            .ok_or_else(|| invalid(&format!("{path}: integrity is not sha384-<base64>")))?;
        out.insert(path.clone(), (size, integrity.to_string()));
    }
    Ok(out)
}

/// The hex of a `sha384-<base64>` integrity's digest: the file's address.
fn digest_hex(integrity: &str) -> Option<String> {
    let b64 = integrity.strip_prefix("sha384-")?;
    let digest = og_core::base64::decode(b64)?;
    (digest.len() == 48).then(|| digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// Whether `bytes` match `integrity`: natively checked here; the web
/// host's fetch refused other bytes already.
fn checked(bytes: &[u8], integrity: &str) -> Result<(), String> {
    #[cfg(all(feature = "fetch", not(target_arch = "wasm32")))]
    {
        og_net::http::integrity(bytes, integrity)
    }
    #[cfg(not(all(feature = "fetch", not(target_arch = "wasm32"))))]
    {
        let _ = (bytes, integrity);
        Ok(())
    }
}

enum Step {
    /// Still on its way.
    Wait,
    Done(Result<(Loaded, String), String>),
}

/// Reads job `job` as far as the host has it.
fn read(eng: &Engine, job: &Job, recorded: Option<&str>) -> Step {
    let Some(h) = hooks() else {
        return Step::Done(Err("unavailable: no host reads files".into()));
    };
    let (base, later) = {
        let a = eng.assets.borrow();
        (a.content.clone(), a.content_later)
    };
    // The host names its content base once its platform settled (the Player's hello).
    if base.is_empty() && later {
        return Step::Wait;
    }
    if base.is_empty() {
        return Step::Done(Err(
            "unavailable: no content base (og run --content <dir>, og build --content <url> or project.ogt publish { content })".into(),
        ));
    }
    let key = (job.catalog.clone(), job.version.clone());
    let (known, pinned) = {
        let a = eng.assets.borrow();
        (
            a.data.manifests.get(&key).cloned(),
            a.data.pins.get(&key).cloned().unwrap_or_default(),
        )
    };
    let files = match known {
        Some(m) => m,
        None => {
            let url = join(
                &base,
                &format!("{}/{}/manifest.json", job.catalog, job.version),
            );
            let what = format!("{}/{}/manifest.json", job.catalog, job.version);
            let m = match h.read_content(&url, &pinned) {
                Fetch::Pending => return Step::Wait,
                Fetch::Failed(e) if e.contains("integrity") => {
                    Err(format!("integrity: {what}: {e}"))
                }
                Fetch::Failed(e) => Err(failure(&format!("{}/{}", job.catalog, job.version), &e)),
                Fetch::Ready(bytes) if !pinned.is_empty() => match checked(&bytes, &pinned) {
                    Ok(()) => manifest(&bytes, &job.catalog, &job.version),
                    Err(e) => Err(format!(
                        "integrity: {what}: the catalog's head names {pinned}: {e}"
                    )),
                },
                Fetch::Ready(bytes) => manifest(&bytes, &job.catalog, &job.version),
            };
            // A version never changes, so its manifest is kept; a failed read
            // is not, and a later og.load.data asks again.
            if m.is_ok() {
                eng.assets
                    .borrow_mut()
                    .data
                    .manifests
                    .insert(key, m.clone());
            }
            m
        }
    };
    let files = match files {
        Ok(f) => f,
        Err(e) => return Step::Done(Err(e)),
    };
    let name = format!("{}/{}/{}", job.catalog, job.version, job.path);
    let Some((size, integrity)) = files.get(&job.path) else {
        return Step::Done(Err(format!(
            "not_found: {name}: the version's manifest lists no such file"
        )));
    };
    if let Some(r) = recorded.filter(|r| !r.is_empty() && r != integrity) {
        return Step::Done(Err(format!(
            "integrity: {name}: the recorded run read {r}, this version lists {integrity}"
        )));
    }
    if *size as f64 > job.max_bytes {
        return Step::Done(Err(format!(
            "too_large: {name} is {size} bytes, over LoadBudget.bytes ({})",
            job.max_bytes
        )));
    }
    let hex = digest_hex(integrity).expect("checked when the manifest was read");
    let url = join(&base, &format!("{}/blobs/{hex}", job.catalog));
    let bytes = match h.read_content(&url, integrity) {
        Fetch::Pending => return Step::Wait,
        Fetch::Failed(e) if e.contains("integrity") => {
            return Step::Done(Err(format!("integrity: {name}: {e}")));
        }
        Fetch::Failed(e) => return Step::Done(Err(failure(&name, &e))),
        Fetch::Ready(bytes) => bytes,
    };
    if let Err(e) = checked(&bytes, integrity) {
        return Step::Done(Err(format!("integrity: {name}: {e}")));
    }
    if bytes.len() as u64 != *size {
        return Step::Done(Err(format!(
            "integrity: {name} is {} bytes, the manifest says {size}",
            bytes.len()
        )));
    }
    let loaded = if job.json {
        match serde_json::from_slice(&bytes) {
            Ok(v) => Loaded::Json(v),
            Err(e) => return Step::Done(Err(format!("invalid: {name} is not JSON: {e}"))),
        }
    } else {
        Loaded::Bytes(bytes)
    };
    Step::Done(Ok((loaded, integrity.clone())))
}

/// At a tick boundary, the world at tick `tick`: every data load the host
/// has read is put in place and journaled, or with a replayed journal each
/// at its recorded tick.
pub(crate) fn poll(eng: &Engine, tick: u64) {
    let jobs = std::mem::take(&mut eng.assets.borrow_mut().data.jobs);
    if jobs.is_empty() {
        return;
    }
    let now = hooks().map_or(0.0, |h| h.now_ms());
    let mut kept = Vec::new();
    for job in jobs {
        let due = eng.assets.borrow().uncooked.due(Kind::Data, job.request);
        let outcome = match due {
            Due::Never => None,
            Due::At(at, ..) if at > tick => None,
            Due::At(_, false, _) => Some(Err(format!(
                "failed: {}/{}/{}: failed in the recorded run",
                job.catalog, job.version, job.path
            ))),
            Due::Free => match read(eng, &job, None) {
                Step::Done(r) => Some(r),
                Step::Wait if job.asked_ms > 0.0 && now - job.asked_ms > TIMEOUT_MS => {
                    Some(Err(format!(
                        "timeout: {}/{}/{}: nothing arrived in 30 seconds",
                        job.catalog, job.version, job.path
                    )))
                }
                Step::Wait => None,
            },
            // A replayed load is waited for at its tick (natively a read
            // completes at once; on the web it lands when it lands).
            Due::At(_, true, recorded) => match read(eng, &job, Some(&recorded)) {
                Step::Done(r) => Some(r),
                Step::Wait => None,
            },
        };
        let Some(result) = outcome else {
            kept.push(job);
            continue;
        };
        let mut assets = eng.assets.borrow_mut();
        let (ok, integrity) = {
            let entry = &mut assets.entries[job.entry];
            match result {
                Ok((loaded, integrity)) => {
                    entry.status = Status::Ready;
                    entry.payload = Payload::Data(Box::new(loaded));
                    (true, integrity)
                }
                Err(e) => {
                    entry.status = Status::Failed(e);
                    (false, String::new())
                }
            }
        };
        assets.uncooked.journal.push(Completion {
            kind: Kind::Data,
            request: job.request,
            tick,
            ok,
            integrity,
        });
    }
    eng.assets.borrow_mut().data.jobs.extend(kept);
}

/// og_engine_set_content: the content base the host names once it knows it
/// (the Player's hello); empty for none. Loads waiting for it go on at the
/// next tick boundary.
pub(crate) fn set_content(eng: &Engine, base: &str) {
    let mut a = eng.assets.borrow_mut();
    a.content = base.trim_end_matches('/').to_string();
    a.content_later = false;
}

/// Whether entry `index` is a data load (pending or put in place).
pub(crate) fn is_data(eng: &Engine, index: usize) -> bool {
    matches!(
        eng.assets.borrow().entries.get(index).map(|e| &e.payload),
        Some(Payload::Data(_) | Payload::DataWaiting)
    )
}

/// og.load.release of a data load: its bytes freed, a pending one never
/// put in place. Its handle then answers failed, naming the release.
pub(crate) fn release(eng: &Engine, index: usize) {
    let mut a = eng.assets.borrow_mut();
    a.data.jobs.retain(|j| j.entry != index);
    let e = &mut a.entries[index];
    e.payload = Payload::None;
    e.status = Status::Failed(format!("{} was released (og.load.release)", e.path));
}

/// What a ready data load holds, or why it holds none.
pub(crate) fn with<T>(
    eng: &Engine,
    handle: f64,
    f: impl FnOnce(&str, &Loaded) -> Result<T, String>,
) -> Result<T, String> {
    let a = eng.assets.borrow();
    let i = crate::world::assets::index_of(handle).filter(|&i| i < a.entries.len());
    let Some(i) = i else {
        return Err(format!("asset {handle} is not alive"));
    };
    let e = &a.entries[i];
    match (&e.status, &e.payload) {
        (Status::Failed(r), _) => Err(r.clone()),
        (_, Payload::Data(d)) => f(&e.path, d),
        (_, Payload::DataWaiting) => Err(format!("{} is still loading", e.path)),
        _ => Err(format!("{} is not an og.load.data file", e.path)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_files_address_is_its_sha384_digest_as_hex() {
        // sha384("abc"), FIPS 180-2's example.
        let sri = "sha384-ywB1P0WjXou1oD1pmsZQBycsMqsO3tFjGotgWkP/W+2AhgcroefMI1i67KE0yCWn";
        assert_eq!(
            digest_hex(sri).unwrap(),
            "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7"
        );
        assert_eq!(
            digest_hex("sha256-ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0="),
            None
        );
        assert_eq!(digest_hex("sha384-short"), None);
    }
}
