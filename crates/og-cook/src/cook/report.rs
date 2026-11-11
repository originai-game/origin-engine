use super::*;

/// One output in the report.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OutputReport {
    pub id: String,
    pub file: String,
    pub ty: String,
    pub bytes: u64,
    /// Cooked in this run (false: the manifest's output was up to date).
    pub cooked: bool,
    /// The file on disk changed.
    pub written: bool,
    pub ms: f64,
    /// When its cook began, in milliseconds after the cook's start, and
    /// the cook thread that ran it (0 to the thread count less one): the
    /// trace lays each output where it ran (`og run --trace`). 0 and 0 for
    /// an output that did not cook.
    pub start_ms: f64,
    pub worker: u32,
}

/// One source asset in the report.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssetReport {
    pub source: String,
    pub kind: String,
    pub source_bytes: u64,
    pub outputs: Vec<OutputReport>,
    pub notes: Vec<String>,
    pub error: Option<String>,
    /// What its last cook took, in milliseconds (its outputs' times summed):
    /// this run's when any output cooked, else the time recorded in
    /// `cooked/.cook-times.ogt` ([`CookTimes`]; 0 when none is).
    pub cook_ms: f64,
}

/// `cooked/.cook-times.ogt`: beside the manifest, the time each asset's
/// last cook took. The manifest itself holds no times, so the same sources
/// still cook to byte-identical `cooked/` (§Determinism); this file is the
/// one that differs, is written only when an asset cooked, and is not
/// shipped (og build leaves it out).
pub const COOK_TIMES: &str = ".cook-times.ogt";

/// [`COOK_TIMES`]: per source path, its last cook's milliseconds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CookTimes {
    pub assets: BTreeMap<String, f64>,
}

impl CookTimes {
    /// Reads `cooked/.cook-times.ogt` (empty when missing or unreadable).
    pub fn read(cooked: &Path) -> CookTimes {
        std::fs::read_to_string(cooked.join(COOK_TIMES))
            .ok()
            .and_then(|t| CookTimes::parse(&t).ok())
            .unwrap_or_default()
    }

    /// Parses the text: `asset "<source>" <ms>` lines.
    pub fn parse(text: &str) -> Result<CookTimes, String> {
        let doc = Document::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != "cook-times" || doc.version != 1 {
            return Err(format!(
                "expected a 'cook-times' version 1 document, got '{}' version {}",
                doc.ty, doc.version
            ));
        }
        let mut assets = BTreeMap::new();
        for e in doc.body.get_all("asset") {
            let source = e.string(0).map_err(|x| format!("line {}: {x}", e.line))?;
            let ms = e.f32(1).map_err(|x| format!("line {}: {x}", e.line))?;
            assets.insert(source.to_string(), f64::from(ms));
        }
        Ok(CookTimes { assets })
    }

    pub fn to_text(&self) -> String {
        use og_core::ogt::{Entry, Number, Value};
        let mut doc = Document::new("cook-times", 1);
        for (source, ms) in &self.assets {
            // To the microsecond.
            let ms = ((ms * 1000.0).round() / 1000.0) as f32;
            doc.body.push(
                Entry::new("asset")
                    .with(Value::String(source.clone()))
                    .with(Value::Number(
                        Number::from_f32(ms).unwrap_or_else(|_| Number::from(0u32)),
                    )),
            );
        }
        doc.to_text().expect("cook times are strings and numbers")
    }

    /// The `n` slowest assets, slowest first (then by path).
    pub fn slowest(&self, n: usize) -> Vec<(&str, f64)> {
        let mut v: Vec<(&str, f64)> = self.assets.iter().map(|(k, v)| (k.as_str(), *v)).collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(b.0)));
        v.truncate(n);
        v
    }
}

/// What a cook did.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub root: String,
    pub cooker: String,
    pub texture_encoder: String,
    pub assets: Vec<AssetReport>,
    /// Files under `assets/` that are not cooked (licences, notes).
    pub ignored: Vec<String>,
    /// Outputs deleted because their source is gone or failed.
    pub removed: Vec<String>,
    /// Problems that do not stop the cook: a locale's missing string keys,
    /// a Tiled map's custom property the importer does not read.
    pub warnings: Vec<String>,
    /// What the art contract changed in each asset it conformed, and every
    /// asset that opted out (docs/art.md §2.3), by asset id.
    pub conform: Vec<conform::ConformEntry>,
    pub elapsed_ms: f64,
    /// Each stage of the cook in the order it ran, with its time: `index`
    /// alone when the index found nothing changed.
    #[serde(default)]
    pub stages: Vec<StageTime>,
}

/// One stage of the cook and the milliseconds it took ([`Report::stages`]).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StageTime {
    pub name: String,
    pub ms: f64,
}

/// Bytes in and out per asset kind.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct KindTotals {
    pub assets: usize,
    pub source_bytes: u64,
    pub cooked_bytes: u64,
}

impl Report {
    /// No asset failed.
    pub fn ok(&self) -> bool {
        self.assets.iter().all(|a| a.error.is_none())
    }

    pub fn errors(&self) -> Vec<String> {
        self.assets
            .iter()
            .filter_map(|a| {
                a.error
                    .as_ref()
                    .map(|e| format!("assets/{}: {e}", a.source))
            })
            .collect()
    }

    pub fn source_bytes(&self) -> u64 {
        self.assets.iter().map(|a| a.source_bytes).sum()
    }

    pub fn cooked_bytes(&self) -> u64 {
        self.assets
            .iter()
            .flat_map(|a| &a.outputs)
            .map(|o| o.bytes)
            .sum()
    }

    /// Outputs cooked in this run.
    pub fn cooked_outputs(&self) -> Vec<&OutputReport> {
        self.assets
            .iter()
            .flat_map(|a| &a.outputs)
            .filter(|o| o.cooked)
            .collect()
    }

    pub fn by_kind(&self) -> BTreeMap<String, KindTotals> {
        let mut m: BTreeMap<String, KindTotals> = BTreeMap::new();
        for a in &self.assets {
            let t = m.entry(a.kind.clone()).or_default();
            t.assets += 1;
            t.source_bytes += a.source_bytes;
            t.cooked_bytes += a.outputs.iter().map(|o| o.bytes).sum::<u64>();
        }
        m
    }

    /// The `n` assets whose last cook took longest ([`AssetReport::cook_ms`],
    /// this run's or the recorded one), slowest first.
    pub fn slowest(&self, n: usize) -> Vec<&AssetReport> {
        let mut v: Vec<&AssetReport> = self
            .assets
            .iter()
            .filter(|a| a.error.is_none() && !a.outputs.is_empty())
            .collect();
        v.sort_by(|a, b| {
            b.cook_ms
                .total_cmp(&a.cook_ms)
                .then(a.source.cmp(&b.source))
        });
        v.truncate(n);
        v
    }

    /// The assets cooked in this run with their times, slowest first.
    pub fn cooked_assets(&self) -> Vec<&AssetReport> {
        let mut v: Vec<&AssetReport> = self
            .assets
            .iter()
            .filter(|a| a.outputs.iter().any(|o| o.cooked))
            .collect();
        v.sort_by(|a, b| {
            b.cook_ms
                .total_cmp(&a.cook_ms)
                .then(a.source.cmp(&b.source))
        });
        v
    }

    /// The slowest ten as a Markdown table (`og cook --report`).
    pub fn slowest_markdown(&self) -> String {
        let mut s = String::from(
            "| # | asset | kind | last cook ms | this run |\n|---:|---|---|---:|---|\n",
        );
        for (i, a) in self.slowest(10).iter().enumerate() {
            let now = a.outputs.iter().any(|o| o.cooked);
            s.push_str(&format!(
                "| {} | {} | {} | {:.3} | {} |\n",
                i + 1,
                a.source,
                a.kind,
                a.cook_ms,
                if now { "cooked" } else { "up to date" }
            ));
        }
        s
    }

    /// A Markdown summary: per class and per asset sizes, what was cooked.
    pub fn to_markdown(&self) -> String {
        let pct = |a: u64, b: u64| {
            if b == 0 {
                "-".to_string()
            } else {
                format!("{:.1} %", a as f64 * 100.0 / b as f64)
            }
        };
        let mut s = format!(
            "# Cook report\n\n`{}` with {}; textures by {}. {} asset(s), {} output(s) cooked this run, {:.0} ms.\n\n",
            self.root,
            self.cooker,
            self.texture_encoder,
            self.assets.len(),
            self.cooked_outputs().len(),
            self.elapsed_ms
        );
        s.push_str("| class | assets | source bytes | cooked bytes | cooked / source |\n|---|---:|---:|---:|---:|\n");
        for (k, t) in self.by_kind() {
            s.push_str(&format!(
                "| {k} | {} | {} | {} | {} |\n",
                t.assets,
                t.source_bytes,
                t.cooked_bytes,
                pct(t.cooked_bytes, t.source_bytes)
            ));
        }
        s.push_str(&format!(
            "| **total** | {} | {} | {} | **{}** |\n\n",
            self.assets.len(),
            self.source_bytes(),
            self.cooked_bytes(),
            pct(self.cooked_bytes(), self.source_bytes())
        ));
        if !self.stages.is_empty() {
            s.push_str("## Stages\n\n| stage | ms |\n|---|---:|\n");
            for st in &self.stages {
                s.push_str(&format!("| {} | {:.3} |\n", st.name, st.ms));
            }
            s.push('\n');
        }
        s.push_str(
            "## The slowest ten\n\nEach asset's last cook (`cooked/.cook-times.ogt`), its outputs' times summed.\n\n",
        );
        s.push_str(&self.slowest_markdown());
        s.push('\n');
        s.push_str("| asset | output | type | bytes | cooked this run | ms |\n|---|---|---|---:|---|---:|\n");
        for a in &self.assets {
            for o in &a.outputs {
                s.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {:.0} |\n",
                    a.source,
                    o.file,
                    o.ty,
                    o.bytes,
                    if o.cooked { "yes" } else { "up to date" },
                    o.ms
                ));
            }
        }
        s.push('\n');
        for a in &self.assets {
            for n in &a.notes {
                s.push_str(&format!("- `{}`: {n}\n", a.source));
            }
            if let Some(e) = &a.error {
                s.push_str(&format!("- `{}`: **error**: {e}\n", a.source));
            }
        }
        for i in &self.ignored {
            s.push_str(&format!("- `{i}`: not an asset, ignored\n"));
        }
        if !self.conform.is_empty() {
            s.push_str("\n## The art contract\n\nWhat conforming to the project's look changed, and what opted out (docs/art.md §2.3).\n\n| asset | | what |\n|---|---|---|\n");
            for e in &self.conform {
                s.push_str(&format!(
                    "| {} | {} | {} |\n",
                    e.source,
                    e.action.name(),
                    e.what
                ));
            }
        }
        for r in &self.removed {
            s.push_str(&format!("- `cooked/{r}`: removed (stale)\n"));
        }
        s
    }

    /// The report as JSON.
    pub fn to_json(&self) -> String {
        use serde_json::{Value, json};
        let assets: Vec<Value> = self
            .assets
            .iter()
            .map(|a| {
                json!({
                    "source": a.source, "kind": a.kind, "source_bytes": a.source_bytes,
                    "error": a.error, "notes": a.notes, "cook_ms": a.cook_ms,
                    "outputs": a.outputs.iter().map(|o| json!({
                        "id": o.id, "file": o.file, "type": o.ty, "bytes": o.bytes,
                        "cooked": o.cooked, "written": o.written, "ms": o.ms
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        let kinds: serde_json::Map<String, Value> = self
            .by_kind()
            .into_iter()
            .map(|(k, t)| {
                (k, json!({"assets": t.assets, "source_bytes": t.source_bytes, "cooked_bytes": t.cooked_bytes}))
            })
            .collect();
        serde_json::to_string_pretty(&json!({
            "root": self.root, "cooker": self.cooker, "texture_encoder": self.texture_encoder,
            "elapsed_ms": self.elapsed_ms, "source_bytes": self.source_bytes(), "cooked_bytes": self.cooked_bytes(),
            "by_kind": kinds, "assets": assets, "ignored": self.ignored, "removed": self.removed,
            "warnings": self.warnings,
            "stages": self.stages.iter().map(|s| json!({"name": s.name, "ms": s.ms})).collect::<Vec<_>>(),
            "conform": self.conform.iter().map(|e| json!({
                "source": e.source, "action": e.action.name(), "what": e.what
            })).collect::<Vec<_>>(),
        }))
        .expect("the report serializes")
    }
}

/// `cooked/.conform.ogt`: per conformed output (id and stamp), what the 3D
/// pass said of it when it last cooked, so an up-to-date output is still
/// reported. Written only when it holds something.
pub(super) const CONFORM_LOG: &str = ".conform.ogt";

pub(super) type ConformLog = BTreeMap<(String, u64), Vec<String>>;

pub(super) fn read_conform_log(cooked: &Path) -> ConformLog {
    let mut out = ConformLog::new();
    let Some(doc) = std::fs::read_to_string(cooked.join(CONFORM_LOG))
        .ok()
        .and_then(|t| Document::parse(&t).ok())
        .filter(|d| d.ty == "conform-log" && d.version == 1)
    else {
        return out;
    };
    for e in doc.body.get_all("output") {
        let strings: Vec<&str> = e.values.iter().filter_map(|v| v.as_str()).collect();
        if let [id, stamp, what @ ..] = strings.as_slice()
            && let Ok(stamp) = u64::from_str_radix(stamp, 16)
        {
            out.insert(
                (id.to_string(), stamp),
                what.iter().map(|w| w.to_string()).collect(),
            );
        }
    }
    out
}

pub(super) fn conform_log_text(log: &ConformLog) -> String {
    use og_core::ogt::{Entry, Value};
    let mut doc = Document::new("conform-log", 1);
    for ((id, stamp), what) in log {
        let mut e = Entry::new("output")
            .with(Value::String(id.clone()))
            .with(Value::String(format!("{stamp:016x}")));
        for w in what {
            e = e.with(Value::String(w.clone()));
        }
        doc.body.push(e);
    }
    doc.to_text().expect("the log is strings")
}

/// The 3D pass's line in one of an output's notes (`conform: ...`, a
/// texture's `<id>: conform: ...`), without its prefix.
pub(super) fn conform_note<'a>(id: &str, note: &'a str) -> Option<&'a str> {
    let n = note
        .strip_prefix(id)
        .and_then(|r| r.strip_prefix(": "))
        .unwrap_or(note);
    n.strip_prefix("conform: ")
}
