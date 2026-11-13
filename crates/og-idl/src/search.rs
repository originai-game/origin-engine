//! The API reference as entries, and the ranking `og api find` uses: BM25F
//! over each entry's name, summary, member names and body, with camelCase,
//! snake_case and dot tokenizing, a light stemmer and typo tolerance (an
//! unknown query word matches the index's words within one or two edits).
//! `og api gen` builds the index from the generated pages and ships it
//! beside them ([`INDEX_FILE`]), with each entry's static embedding
//! ([`crate::embed`]); a query's BM25F and embedding rankings are fused by
//! reciprocal rank. A query scores every entry in milliseconds on one core,
//! with no service. It returns entries and scores only: the agent chooses.

use std::collections::BTreeMap;

/// The index's file name in the wiki directory.
pub const INDEX_FILE: &str = "search.tsv";

/// How many guide sections the first [`GUIDE_WINDOW`] results may hold:
/// a guide's sections share words, so without a cap one query's sections
/// crowd out the API entries they name. The others follow the window.
const GUIDE_CAP: usize = 1;
/// The results the guide cap applies to (the five a reader looks at).
const GUIDE_WINDOW: usize = 5;

/// One anchored entry of a wiki page.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The documented name, as the heading or row writes it (`og.CharacterController`, `Camera.lookAt`, `BODY_STATIC`).
    pub name: String,
    /// The anchor's kind: module, guide, fn, record, type, member, const, value or column.
    pub kind: String,
    /// The page, relative to the wiki directory and without `.md` (`physics`, `physics/record-CharacterController`).
    pub page: String,
    /// The first sentence of its doc.
    pub summary: String,
    /// The entry's text as the page writes it.
    pub body: String,
}

/// Every anchored entry of one wiki page.
pub fn entries(page: &str, text: &str) -> Vec<Entry> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    // The page's module overview (`# The `input` module` to its contents):
    // `og api show input`; a guide's page is its overview whole (`og api show testing`).
    if let Some(h) = lines.iter().position(|l| {
        l.starts_with("# The `") && (l.ends_with("` module") || l.contains("` guide: "))
    }) {
        let kind = if lines[h].ends_with("` module") {
            "module"
        } else {
            "guide"
        };
        let end = lines[h..]
            .iter()
            .position(|l| l.starts_with("## "))
            .map_or(lines.len(), |n| h + n);
        let summary = lines[h + 1..end]
            .iter()
            .find(|l| !l.trim().is_empty())
            .map(|l| first_sentence(l))
            .unwrap_or_default();
        out.push(Entry {
            name: between(lines[h], '`', '`').unwrap_or_default().to_string(),
            kind: kind.to_string(),
            page: page.to_string(),
            summary,
            body: lines[h..end].join("\n").trim().to_string(),
        });
    }
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some((kind, rest)) = row_anchor(line) {
            let name = between(rest, '`', '`').unwrap_or_default().to_string();
            let summary = line
                .trim_end()
                .trim_end_matches('|')
                .rsplit('|')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            out.push(Entry {
                name,
                kind,
                page: page.to_string(),
                summary: first_sentence(&summary),
                body: line.to_string(),
            });
            i += 1;
            continue;
        }
        if let Some(kind) = block_anchor(line) {
            let mut j = i + 1;
            while j < lines.len()
                && block_anchor(lines[j]).is_none()
                && !lines[j].starts_with("## ")
            {
                j += 1;
            }
            let block = &lines[i + 1..j];
            let heading = block
                .iter()
                .find(|l| l.starts_with('#'))
                .copied()
                .unwrap_or("");
            let name = between(heading, '`', '`').unwrap_or_default().to_string();
            let summary = block
                .iter()
                .skip_while(|l| !l.starts_with('#'))
                .skip(1)
                .find(|l| !l.trim().is_empty())
                .map(|l| first_sentence(l))
                .unwrap_or_default();
            out.push(Entry {
                name,
                kind,
                page: page.to_string(),
                summary,
                body: block.join("\n").trim().to_string(),
            });
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

fn block_anchor(line: &str) -> Option<String> {
    let rest = line.strip_prefix("<a id=\"")?;
    Some(rest.split('-').next()?.to_string())
}

fn row_anchor(line: &str) -> Option<(String, &str)> {
    let rest = line.strip_prefix("| <a id=\"")?;
    let kind = rest.split('-').next()?.to_string();
    let after = rest.split_once("</a>")?.1;
    Some((kind, after))
}

fn between(s: &str, open: char, close: char) -> Option<&str> {
    let start = s.find(open)? + open.len_utf8();
    let end = s[start..].find(close)? + start;
    Some(&s[start..end])
}

/// The first sentence of a doc, cut at 200 characters: an entry's summary.
pub fn first_sentence(s: &str) -> String {
    let s = s.trim();
    let end = s
        .match_indices(". ")
        .map(|(i, _)| i + 1)
        .find(|&i| i >= 20)
        .unwrap_or(s.len());
    let mut out = s[..end].to_string();
    if out.chars().count() > 200 {
        out = out.chars().take(199).collect::<String>() + "…";
    }
    out
}

/// The name without `og.`, lowercased: how a name is matched exactly.
pub fn bare(name: &str) -> String {
    name.trim_start_matches("og.").to_lowercase()
}

// ── tokens ──────────────────────────────────────────────────────────────────

/// Words no query is about.
const STOP: &[&str] = &[
    "a", "an", "and", "any", "are", "as", "at", "be", "by", "can", "do", "does", "for", "from",
    "how", "i", "if", "in", "into", "is", "it", "its", "me", "my", "of", "on", "or", "so", "that",
    "the", "their", "them", "then", "they", "this", "to", "we", "what", "when", "which", "with",
    "you", "your",
];

/// A word reduced to its stem: plurals, `-ing` and `-ed` off, as both the
/// index and the query are, so "jumps", "jumping" and "jumped" meet "jump".
fn stem(w: &str) -> String {
    let n = w.len();
    if n > 4 && w.ends_with("ies") {
        return format!("{}y", &w[..n - 3]);
    }
    if n > 4
        && (w.ends_with("sses") || w.ends_with("shes") || w.ends_with("ches") || w.ends_with("xes"))
    {
        return w[..n - 2].to_string();
    }
    if n > 3 && w.ends_with('s') && !w.ends_with("ss") && !w.ends_with("us") && !w.ends_with("is") {
        return w[..n - 1].to_string();
    }
    if n > 5 && w.ends_with("ing") {
        return w[..n - 3].to_string();
    }
    if n > 4 && w.ends_with("ed") && !w.ends_with("eed") {
        return w[..n - 2].to_string();
    }
    w.to_string()
}

/// The words of a name or text: lowercased, split at camelCase, `_`, `.`
/// and every other non-alphanumeric, stop words dropped, stemmed.
pub fn tokens(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut prev_lower = false;
    let flush = |cur: &mut String, out: &mut Vec<String>| {
        if !cur.is_empty() {
            let w = std::mem::take(cur);
            if !STOP.contains(&w.as_str()) {
                out.push(stem(&w));
            }
        }
    };
    for c in s.chars() {
        if c.is_alphanumeric() {
            if c.is_uppercase() && prev_lower {
                flush(&mut cur, &mut out);
            }
            prev_lower = c.is_lowercase() || c.is_ascii_digit();
            cur.extend(c.to_lowercase());
        } else {
            prev_lower = false;
            flush(&mut cur, &mut out);
        }
    }
    flush(&mut cur, &mut out);
    out
}

// ── the index ───────────────────────────────────────────────────────────────

/// The fields an entry is scored on, in the index's column order.
pub const FIELDS: [&str; NF] = ["name", "summary", "members", "body", "aliases"];
/// How many fields an entry is scored on.
const NF: usize = 5;
/// Each field's weight and length normalisation (BM25F's w and b).
const WEIGHT: [f64; NF] = [3.0, 1.5, 1.5, 1.0, 2.0];
const B: [f64; NF] = [0.3, 0.5, 0.5, 0.75, 0.3];
const K1: f64 = 1.2;
/// The weight of an index word a query word reaches only by a typo.
const FUZZY: f64 = 0.8;
/// What a query holding an entry's whole multi-word name adds to its score.
const PHRASE: f64 = 2.0;

/// An entry's text split into the scored fields.
fn fields(e: &Entry) -> [Vec<String>; NF] {
    let mut aliases = String::new();
    let mut members = String::new();
    let mut body = String::new();
    // The IDL source quote and the binding line carry no words a reader asks
    // with; the TypeScript signature (its parameter names) stays.
    let mut idl = false;
    for line in e.body.lines() {
        if line.starts_with("```ogt") {
            idl = true;
            continue;
        }
        if idl {
            idl = !line.starts_with("```");
            continue;
        }
        if line.starts_with("```")
            || line.starts_with("Binding symbol ")
            || line.starts_with("<sub>")
        {
            continue;
        }
        if let Some(rest) = line.strip_prefix(crate::r#gen::wiki::ALSO_KNOWN_AS) {
            aliases.push_str(rest);
            continue;
        }
        // A table row's first cell is a field, parameter, variant or member name.
        if let Some(rest) = line.strip_prefix("| ") {
            if let Some(cell) = rest.split('|').next() {
                members.push_str(cell);
                members.push(' ');
            }
            body.push_str(rest);
        } else if let Some(h) = line.strip_prefix("####") {
            members.push_str(h);
            members.push(' ');
        } else {
            body.push_str(line);
        }
        body.push(' ');
    }
    let name = format!("{} {}", e.name, e.name.rsplit('.').next().unwrap_or(""));
    [
        tokens(&name),
        tokens(&e.summary),
        tokens(&members),
        tokens(&body),
        tokens(&aliases),
    ]
}

/// One entry of the index: what `find` prints, and its fields' lengths.
#[derive(Debug, Clone, PartialEq)]
pub struct Doc {
    pub name: String,
    pub kind: String,
    pub page: String,
    pub summary: String,
    pub len: [u32; NF],
    /// The entry's embedding (crate::embed), int8 with its scale.
    pub vec: Vec<i8>,
    pub vscale: f32,
}

/// One ranked entry: the fused score and the two rankers' own.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub doc: usize,
    /// Reciprocal-rank fusion of the two rankings (an exact name adds 1).
    pub score: f64,
    /// BM25F, when the entry holds a query word.
    pub bm25: Option<f64>,
    /// The cosine of the query's and the entry's embeddings.
    pub cosine: f32,
}

/// The text an entry is embedded from: its name split into words, twice (a
/// name is the densest statement of purpose), its summary and the first
/// hundred words of its prose, code and tables' markup left out.
fn embed_text(e: &Entry) -> String {
    let leaf = e.name.trim_start_matches("og.");
    let mut name = String::new();
    let mut prev_lower = false;
    for c in leaf.chars() {
        if c.is_uppercase() && prev_lower {
            name.push(' ');
        }
        prev_lower = c.is_lowercase() || c.is_ascii_digit();
        name.push(if c == '_' || c == '.' { ' ' } else { c });
    }
    let mut prose = String::new();
    let mut code = false;
    for line in e.body.lines() {
        if line.starts_with("```") {
            code = !code;
            continue;
        }
        if code
            || line.starts_with("Binding symbol ")
            || line.starts_with("<sub>")
            || line.starts_with("<a id=")
        {
            continue;
        }
        prose.push_str(line);
        prose.push(' ');
    }
    let words: Vec<&str> = prose
        .split(|c: char| c.is_whitespace() || matches!(c, '`' | '|' | '#' | '*' | '[' | ']'))
        .filter(|w| !w.is_empty())
        .take(100)
        .collect();
    format!("{name}. {name}. {} {}", e.summary, words.join(" "))
}

/// Reciprocal-rank fusion's constant: a small one lets either ranker's top
/// few lead (20, measured on the evaluation set beside 10 and 40).
const RRF_K: f64 = 20.0;
/// The embedding ranking's weight in the fusion, BM25F's being 1: the
/// lexical ranking stays the stronger voice.
const EMBED_WEIGHT: f64 = 0.7;
/// How many of the embedding ranking's best enter the fusion.
const EMBED_TOP: usize = 100;

/// The BM25F index of every entry: documents, average field lengths and,
/// for each word, the entries holding it with its count per field.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SearchIndex {
    pub docs: Vec<Doc>,
    pub terms: BTreeMap<String, Vec<(u32, [u16; NF])>>,
}

impl SearchIndex {
    pub fn build(all: &[Entry]) -> SearchIndex {
        let mut ix = SearchIndex::default();
        for (d, e) in all.iter().enumerate() {
            let f = fields(e);
            let (vec, vscale) =
                crate::embed::quantize(&crate::embed::model().embed(&embed_text(e)));
            let mut counts: BTreeMap<&str, [u16; NF]> = BTreeMap::new();
            for (k, words) in f.iter().enumerate() {
                for w in words {
                    let c = counts.entry(w.as_str()).or_default();
                    c[k] = c[k].saturating_add(1);
                }
            }
            for (w, c) in counts {
                ix.terms
                    .entry(w.to_string())
                    .or_default()
                    .push((d as u32, c));
            }
            ix.docs.push(Doc {
                name: e.name.clone(),
                kind: e.kind.clone(),
                page: e.page.clone(),
                summary: e.summary.clone(),
                len: std::array::from_fn(|k| f[k].len() as u32),
                vec,
                vscale,
            });
        }
        ix
    }

    /// Each document followed by its sorted word counts. No persisted ordinal
    /// refers to another row: inserting a document leaves other rows unchanged.
    pub fn to_text(&self) -> String {
        let mut out = String::from(
            "# og api find index 4: BM25F over name, summary, members, body, aliases; embeddings (scale:base64 int8)\n",
        );
        let mut words = vec![Vec::new(); self.docs.len()];
        for (word, posts) in &self.terms {
            for (doc, counts) in posts {
                words[*doc as usize].push(format!(
                    "{word}:{}",
                    counts.map(|n| n.to_string()).join(",")
                ));
            }
        }
        for (d, words) in self.docs.iter().zip(words) {
            out.push_str(&format!(
                "doc\t{}\t{}\t{}\t{}\t{}:{}\t{}\n",
                d.name,
                d.kind,
                d.page,
                d.len.map(|n| n.to_string()).join(","),
                d.vscale,
                og_core::base64::encode(&d.vec.iter().map(|&x| x as u8).collect::<Vec<u8>>()),
                d.summary.replace(['\t', '\n'], " ")
            ));
            out.push_str("words\t");
            out.push_str(&words.join(" "));
            out.push('\n');
        }
        out
    }

    pub fn from_text(text: &str) -> Option<SearchIndex> {
        let mut ix = SearchIndex::default();
        let mut lines = text.lines();
        let version = match lines.next()?.split(':').next()? {
            "# og api find index 3" => 3,
            "# og api find index 4" => 4,
            _ => return None,
        };
        for line in lines {
            let mut cols = line.split('\t');
            match cols.next()? {
                "doc" => {
                    let name = cols.next()?.to_string();
                    let kind = cols.next()?.to_string();
                    let page = cols.next()?.to_string();
                    let mut len = [0u32; NF];
                    for (k, n) in cols.next()?.split(',').enumerate().take(NF) {
                        len[k] = n.parse().ok()?;
                    }
                    let (scale, b64) = cols.next()?.split_once(':')?;
                    let vscale = scale.parse().ok()?;
                    let vec = og_core::base64::decode(b64)?
                        .into_iter()
                        .map(|x| x as i8)
                        .collect();
                    let summary = cols.next().unwrap_or("").to_string();
                    ix.docs.push(Doc {
                        name,
                        kind,
                        page,
                        summary,
                        len,
                        vec,
                        vscale,
                    });
                }
                "words" if version == 4 => {
                    let doc = u32::try_from(ix.docs.len().checked_sub(1)?).ok()?;
                    for post in cols.next()?.split_whitespace() {
                        let (word, counts) = post.split_once(':')?;
                        let counts: Vec<u16> = counts
                            .split(',')
                            .map(str::parse)
                            .collect::<Result<_, _>>()
                            .ok()?;
                        ix.terms
                            .entry(word.to_string())
                            .or_default()
                            .push((doc, counts.try_into().ok()?));
                    }
                }
                "term" if version == 3 => {
                    let w = cols.next()?.to_string();
                    let mut posts = Vec::new();
                    for p in cols
                        .next()
                        .unwrap_or("")
                        .split(' ')
                        .filter(|p| !p.is_empty())
                    {
                        let (d, c) = p.split_once(':')?;
                        let mut counts = [0u16; NF];
                        for (k, n) in c.split(',').enumerate().take(NF) {
                            counts[k] = n.parse().ok()?;
                        }
                        posts.push((d.parse().ok()?, counts));
                    }
                    ix.terms.insert(w, posts);
                }
                _ => return None,
            }
        }
        Some(ix)
    }

    /// The entries matching `query`, best first, each with its score: an
    /// exact name first, then BM25F over the query's words; a word the index
    /// lacks counts the words within one edit (two for eight letters or
    /// more) at `FUZZY` weight.
    pub fn bm25(&self, query: &str, kind: Option<&str>) -> Vec<(usize, f64)> {
        let n = self.docs.len() as f64;
        if n == 0.0 {
            return Vec::new();
        }
        // A field's average length over the entries that have it: aliases
        // and member names are absent from most entries, and averaging their
        // zeros in would make every present one look long.
        let mut avg = [0f64; NF];
        let mut have = [0f64; NF];
        for d in &self.docs {
            for k in 0..NF {
                avg[k] += f64::from(d.len[k]);
                have[k] += f64::from(u8::from(d.len[k] > 0));
            }
        }
        for k in 0..NF {
            avg[k] = (avg[k] / have[k].max(1.0)).max(1.0);
        }
        let mut words: Vec<(String, f64)> = Vec::new();
        for q in tokens(query) {
            if self.terms.contains_key(&q) {
                words.push((q, 1.0));
                continue;
            }
            let max = if q.chars().count() >= 8 {
                2
            } else if q.chars().count() >= 4 {
                1
            } else {
                0
            };
            if max == 0 {
                continue;
            }
            for t in self.terms.keys() {
                if t.len().abs_diff(q.len()) <= max && edits(&q, t, max) {
                    words.push((t.clone(), FUZZY));
                }
            }
        }
        let mut score: BTreeMap<u32, f64> = BTreeMap::new();
        for (w, weight) in &words {
            let posts = &self.terms[w];
            let df = posts.len() as f64;
            let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
            for (d, c) in posts {
                let doc = &self.docs[*d as usize];
                let mut tf = 0.0;
                for k in 0..NF {
                    if c[k] > 0 {
                        let norm = 1.0 - B[k] + B[k] * f64::from(doc.len[k]) / avg[k];
                        tf += WEIGHT[k] * f64::from(c[k]) / norm;
                    }
                }
                *score.entry(*d).or_default() += weight * idf * tf / (K1 + tf);
            }
        }
        let want = bare(query.trim());
        // A name the query spells out, in any case or spacing
        // ("character controller" for og.CharacterController).
        let query_words = tokens(query);
        let spelled = query_words.concat();
        // A query that holds a whole multi-word name in order
        // ("character controller zzzqx") leans to that name.
        let holds = |doc: &Doc| {
            let name = tokens(doc.name.rsplit('.').next().unwrap_or(""));
            name.len() >= 2
                && query_words
                    .windows(name.len())
                    .any(|w| w == name.as_slice())
        };
        let exact = |doc: &Doc| {
            let name = bare(&doc.name);
            let last = name.rsplit('.').next().unwrap_or("");
            name == want
                || last == want
                || (!spelled.is_empty()
                    && tokens(doc.name.rsplit('.').next().unwrap_or("")).concat() == spelled)
        };
        let mut out: Vec<(usize, f64)> = score
            .into_iter()
            .map(|(d, s)| (d as usize, s))
            .chain(
                self.docs
                    .iter()
                    .enumerate()
                    .filter_map(|(d, doc)| exact(doc).then_some((d, 0.0))),
            )
            .fold(BTreeMap::<usize, f64>::new(), |mut m, (d, s)| {
                *m.entry(d).or_default() += s;
                m
            })
            .into_iter()
            .filter(|(d, _)| kind.is_none_or(|k| self.docs[*d].kind == k))
            .map(|(d, s)| {
                let doc = &self.docs[d];
                (
                    d,
                    s + if exact(doc) {
                        100.0
                    } else if holds(doc) {
                        PHRASE
                    } else {
                        0.0
                    },
                )
            })
            .collect();
        out.sort_by(|a, b| {
            b.1.total_cmp(&a.1)
                .then_with(|| self.docs[a.0].name.cmp(&self.docs[b.0].name))
        });
        out
    }

    /// The entries for `query`, best first: an exact name first, then the
    /// BM25F ranking and the embedding ranking fused by reciprocal rank, so
    /// a query worded by purpose ("the screen trembles") finds what no word
    /// of it names.
    pub fn search(&self, query: &str, kind: Option<&str>) -> Vec<Hit> {
        let lexical = self.bm25(query, kind);
        // An entry whose whole multi-word name the query holds in order
        // leads the fusion behind an exact name, as it leads BM25F.
        let query_words = tokens(query);
        let holds = |d: usize| {
            let name = tokens(self.docs[d].name.rsplit('.').next().unwrap_or(""));
            name.len() >= 2
                && query_words
                    .windows(name.len())
                    .any(|w| w == name.as_slice())
        };
        let model = crate::embed::model();
        let q = model.embed(query);
        let cosine = |d: usize| -> f32 {
            let doc = &self.docs[d];
            doc.vec
                .iter()
                .zip(&q)
                .map(|(&a, &b)| f32::from(a) * b)
                .sum::<f32>()
                * doc.vscale
        };
        let mut semantic: Vec<(usize, f32)> = if q.iter().any(|&x| x != 0.0) {
            (0..self.docs.len())
                .filter(|&d| kind.is_none_or(|k| self.docs[d].kind == k))
                .map(|d| (d, cosine(d)))
                .collect()
        } else {
            Vec::new()
        };
        semantic.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        semantic.truncate(EMBED_TOP);
        let mut hits: BTreeMap<usize, Hit> = BTreeMap::new();
        for (rank, &(d, s)) in lexical.iter().enumerate() {
            hits.insert(
                d,
                Hit {
                    doc: d,
                    score: 1.0 / (RRF_K + rank as f64 + 1.0)
                        + if s >= 100.0 {
                            1.0
                        } else if holds(d) {
                            0.5
                        } else {
                            0.0
                        },
                    bm25: Some(s),
                    cosine: cosine(d),
                },
            );
        }
        for (rank, &(d, c)) in semantic.iter().enumerate() {
            let h = hits.entry(d).or_insert(Hit {
                doc: d,
                score: 0.0,
                bm25: None,
                cosine: c,
            });
            h.score += EMBED_WEIGHT / (RRF_K + rank as f64 + 1.0);
        }
        let mut out: Vec<Hit> = hits.into_values().collect();
        out.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| self.docs[a.doc].name.cmp(&self.docs[b.doc].name))
        });
        // Past the cap, a guide section (not one named exactly) waits until
        // the window is full; the order is otherwise the score's.
        let mut kept = Vec::with_capacity(out.len());
        let mut later = Vec::new();
        let mut guides = 0;
        for h in out {
            let guide = self.docs[h.doc].kind == "guide" && h.score < 1.0;
            if guide && guides >= GUIDE_CAP && kept.len() < GUIDE_WINDOW {
                later.push(h);
                continue;
            }
            guides += usize::from(guide);
            kept.push(h);
            if kept.len() == GUIDE_WINDOW {
                kept.append(&mut later);
            }
        }
        kept.append(&mut later);
        kept
    }
}

/// Whether `a` and `b` are within `max` edits (insert, delete, substitute,
/// or swap two neighbours).
fn edits(a: &str, b: &str, max: usize) -> bool {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=n {
        let mut best = usize::MAX;
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut v = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                v = v.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = v;
            best = best.min(v);
        }
        if best > max {
            return false;
        }
    }
    d[n][m] <= max
}
