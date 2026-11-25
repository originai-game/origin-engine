//! Caption files a caption track imports (`captions "narration.srt"` or
//! `captions "narration.vtt"`): SubRip (`.srt`), WebVTT (`.vtt`), and word
//! timestamps (`.json`, as speech-to-text tools write them) grouped into lines.

/// An inline style run on a caption's plain text (Unicode scalar offsets).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptionSpan {
    /// Style part name (`emphasis`, `strong`, `color`).
    pub name: String,
    /// Start offset in Unicode scalar values.
    pub position: u32,
    /// Length in Unicode scalar values.
    pub length: u32,
    /// Optional color attribute (e.g. `#ffd700` for `<font color="...">`).
    pub color: String,
}

/// One caption: its plain text, optional speaker, inline spans, and placement
/// (`"bottom"` or `"top"`) from `start` to `end`, in seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    pub start: f32,
    pub end: f32,
    pub text: String,
    pub speaker: String,
    pub spans: Vec<CaptionSpan>,
    pub position: String,
}

impl Cue {
    pub fn plain(start: f32, end: f32, text: String) -> Self {
        Self {
            start,
            end,
            text,
            speaker: String::new(),
            spans: Vec::new(),
            position: String::new(),
        }
    }
}

/// `HH:MM:SS,mmm` (or `.mmm`) or WebVTT `MM:SS.mmm` in seconds.
fn parse_time(s: &str, allow_mmss: bool) -> Option<f32> {
    let s = s.trim();
    let (hms, ms) = s.split_once([',', '.']).unwrap_or((s, "0"));
    let parts: Vec<&str> = hms.split(':').collect();
    let n = |x: &str| x.trim().parse::<u32>().ok();
    let (h, m, sec) = match parts.as_slice() {
        [h, m, sec] => (n(h)?, n(m)?, n(sec)?),
        [m, sec] if allow_mmss => (0, n(m)?, n(sec)?),
        _ => return None,
    };
    let ms = ms.trim();
    let frac = if ms.is_empty() || !ms.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    } else {
        ms.parse::<u32>().ok()? as f64 / 10f64.powi(ms.len() as i32)
    };
    Some((f64::from(h) * 3600.0 + f64::from(m) * 60.0 + f64::from(sec) + frac) as f32)
}

/// Extracts `<font color="...">` attribute value if present.
fn parse_font_color(tag: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let idx = lower.find("color=")?;
    let rest = tag[idx + 6..].trim_start();
    if let Some(q) = rest.strip_prefix('"').and_then(|r| r.split('"').next()) {
        return Some(q.trim().to_string());
    }
    if let Some(q) = rest.strip_prefix('\'').and_then(|r| r.split('\'').next()) {
        return Some(q.trim().to_string());
    }
    Some(
        rest.split(|c: char| c.is_whitespace() || c == '>')
            .next()
            .unwrap_or("")
            .to_string(),
    )
    .filter(|s| !s.is_empty())
}

/// Parses inline SRT/WebVTT markup (`<i>`, `<b>`, `<font color="...">`,
/// `<v Speaker>`, `{\an8}`) across joined cue lines into plain text, speaker,
/// top/bottom placement, and Unicode-scalar `CaptionSpan`s.
pub fn parse_markup(raw: &str) -> (String, String, String, Vec<CaptionSpan>) {
    let mut out = String::new();
    let mut speaker = String::new();
    let mut position = String::new();
    let mut spans = Vec::new();
    // Active open tags: (kind, start_scalar, color)
    let mut stack: Vec<(String, u32, String)> = Vec::new();
    let mut scalar_pos = 0u32;
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let ch = raw[i..].chars().next().expect("valid utf8");
        if ch == '{'
            && let Some(end_rel) = raw[i..].find('}')
        {
            let tag = &raw[i + 1..i + end_rel];
            if tag.contains("\\an7") || tag.contains("\\an8") || tag.contains("\\an9") {
                position = "top".into();
            } else if tag.contains("\\an1") || tag.contains("\\an2") || tag.contains("\\an3") {
                position = "bottom".into();
            }
            i += end_rel + 1;
            continue;
        }
        if ch == '<'
            && let Some(end_rel) = raw[i..].find('>')
        {
            let inside = raw[i + 1..i + end_rel].trim();
            let lower = inside.to_ascii_lowercase();
            if lower == "i" {
                stack.push(("emphasis".into(), scalar_pos, String::new()));
            } else if lower == "/i" {
                close_span(&mut stack, "emphasis", scalar_pos, &mut spans);
            } else if lower == "b" {
                stack.push(("strong".into(), scalar_pos, String::new()));
            } else if lower == "/b" {
                close_span(&mut stack, "strong", scalar_pos, &mut spans);
            } else if lower.starts_with("font") {
                let color = parse_font_color(inside).unwrap_or_default();
                stack.push(("color".into(), scalar_pos, color));
            } else if lower == "/font" {
                close_span(&mut stack, "color", scalar_pos, &mut spans);
            } else if let Some(rest) = inside
                .strip_prefix("v ")
                .or_else(|| inside.strip_prefix("V "))
                .or_else(|| {
                    inside
                        .strip_prefix("v.")
                        .or_else(|| inside.strip_prefix("V."))
                        .and_then(|r| r.split_once(' ').map(|(_, name)| name))
                })
            {
                let name = rest.trim();
                if !name.is_empty() && speaker.is_empty() {
                    speaker = name.to_string();
                }
            }
            i += end_rel + 1;
            continue;
        }
        out.push(ch);
        scalar_pos += 1;
        i += ch.len_utf8();
    }
    // Close any unclosed tags at end of cue.
    while let Some((name, start, color)) = stack.pop() {
        if scalar_pos > start {
            spans.push(CaptionSpan {
                name,
                position: start,
                length: scalar_pos - start,
                color,
            });
        }
    }
    let trimmed_start = out.len() - out.trim_start().len();
    let leading_scalars = out[..trimmed_start].chars().count() as u32;
    let trimmed = out.trim().to_string();
    let max_len = trimmed.chars().count() as u32;
    if leading_scalars > 0 || max_len < scalar_pos {
        for s in &mut spans {
            let start = s.position.saturating_sub(leading_scalars).min(max_len);
            let end = (s.position + s.length)
                .saturating_sub(leading_scalars)
                .min(max_len);
            s.position = start;
            s.length = end.saturating_sub(start);
        }
        spans.retain(|s| s.length > 0);
    }
    spans.sort_by_key(|s| (s.position, s.length));
    (trimmed, speaker, position, spans)
}

fn close_span(
    stack: &mut Vec<(String, u32, String)>,
    kind: &str,
    end: u32,
    out: &mut Vec<CaptionSpan>,
) {
    if let Some(pos) = stack.iter().rposition(|(k, _, _)| k == kind) {
        let (name, start, color) = stack.remove(pos);
        if end > start {
            out.push(CaptionSpan {
                name,
                position: start,
                length: end - start,
                color,
            });
        }
    }
}

/// Parses SubRip text: blocks of an optional index line, `start --> end`,
/// and text lines, separated by blank lines. Preserves `<i>`, `<b>`,
/// `<font color="...">`, and `{\an8}` placement hints. Errors name the line.
pub fn parse_srt(text: &str) -> Result<Vec<Cue>, String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut cues = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim().is_empty() {
            i += 1;
            continue;
        }
        let start_line = i + 1;
        // The index line is optional.
        if !lines[i].contains("-->") {
            i += 1;
        }
        let timing = lines
            .get(i)
            .filter(|l| l.contains("-->"))
            .ok_or_else(|| format!("line {start_line}: a cue needs a `start --> end` line"))?;
        let (a, b) = timing.split_once("-->").expect("checked");
        // Position hints may follow the end time.
        let b = b.split_whitespace().next().unwrap_or("");
        let (start, end) = match (parse_time(a, false), parse_time(b, false)) {
            (Some(s), Some(e)) => (s, e),
            _ => {
                return Err(format!(
                    "line {}: '{}' is not `HH:MM:SS,mmm --> HH:MM:SS,mmm`",
                    i + 1,
                    timing.trim()
                ));
            }
        };
        if end < start {
            return Err(format!("line {}: the cue ends before it starts", i + 1));
        }
        i += 1;
        let mut body = Vec::new();
        while i < lines.len() && !lines[i].trim().is_empty() {
            body.push(lines[i].trim());
            i += 1;
        }
        let joined = body.join("\n");
        let (text, speaker, position, spans) = parse_markup(&joined);
        cues.push(Cue {
            start,
            end,
            text,
            speaker,
            spans,
            position,
        });
    }
    Ok(cues)
}

/// Parses WebVTT (`.vtt`) text: `WEBVTT` header, optional `NOTE`/`STYLE`/
/// `REGION` blocks, cues with `MM:SS.mmm --> MM:SS.mmm` or
/// `HH:MM:SS.mmm --> HH:MM:SS.mmm`, `line:` placement settings, `<v Speaker>`,
/// `<i>`, `<b>`, and `<font color="...">`.
pub fn parse_vtt(text: &str) -> Result<Vec<Cue>, String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }
    let Some(header) = lines.get(i) else {
        return Ok(Vec::new());
    };
    if !header.trim_start().starts_with("WEBVTT") {
        return Err(format!(
            "line {}: a WebVTT file starts with `WEBVTT`",
            i + 1
        ));
    }
    // Skip header lines up to the first blank line.
    while i < lines.len() && !lines[i].trim().is_empty() {
        i += 1;
    }
    let mut cues = Vec::new();
    while i < lines.len() {
        if lines[i].trim().is_empty() {
            i += 1;
            continue;
        }
        let first = lines[i].trim();
        if first.starts_with("NOTE") || first.starts_with("STYLE") || first.starts_with("REGION") {
            while i < lines.len() && !lines[i].trim().is_empty() {
                i += 1;
            }
            continue;
        }
        let start_line = i + 1;
        if !lines[i].contains("-->") {
            i += 1;
        }
        let timing = lines
            .get(i)
            .filter(|l| l.contains("-->"))
            .ok_or_else(|| format!("line {start_line}: a WebVTT cue needs `start --> end`"))?;
        let (a, b_rest) = timing.split_once("-->").expect("checked");
        let mut b_parts = b_rest.split_whitespace();
        let b = b_parts.next().unwrap_or("");
        let mut vtt_pos = String::new();
        for setting in b_parts {
            if let Some(val) = setting.strip_prefix("line:") {
                let pct = val.trim_end_matches('%');
                if let Ok(n) = pct.parse::<f32>()
                    && (0.0..=30.0).contains(&n)
                {
                    vtt_pos = "top".into();
                } else if val == "start" {
                    vtt_pos = "top".into();
                }
            }
        }
        let (start, end) = match (parse_time(a, true), parse_time(b, true)) {
            (Some(s), Some(e)) => (s, e),
            _ => {
                return Err(format!(
                    "line {}: '{}' is not a WebVTT timestamp `MM:SS.mmm --> MM:SS.mmm`",
                    i + 1,
                    timing.trim()
                ));
            }
        };
        if end < start {
            return Err(format!("line {}: the cue ends before it starts", i + 1));
        }
        i += 1;
        let mut body = Vec::new();
        while i < lines.len() && !lines[i].trim().is_empty() {
            body.push(lines[i].trim());
            i += 1;
        }
        let joined = body.join("\n");
        let (text, speaker, markup_pos, spans) = parse_markup(&joined);
        let position = if !markup_pos.is_empty() {
            markup_pos
        } else {
            vtt_pos
        };
        cues.push(Cue {
            start,
            end,
            text,
            speaker,
            spans,
            position,
        });
    }
    Ok(cues)
}

/// Groups word timestamps into caption lines: a line ends after a word
/// ending a sentence (`.`, `!`, `?`, `。`, `！`, `？`), before a pause of
/// `gap` seconds or more, or before it would pass `max_chars` characters.
pub fn group_words(words: &[Cue], max_chars: usize, gap: f32) -> Vec<Cue> {
    let mut out: Vec<Cue> = Vec::new();
    let mut open: Option<Cue> = None;
    for w in words {
        let word = w.text.trim();
        if word.is_empty() {
            continue;
        }
        if let Some(c) = open.as_mut() {
            let cjk = word.chars().next().is_some_and(is_cjk);
            let joined = if cjk || word.starts_with([',', '.', '!', '?', ';', ':']) {
                format!("{}{word}", c.text)
            } else {
                format!("{} {word}", c.text)
            };
            if w.start - c.end >= gap || joined.chars().count() > max_chars {
                out.push(open.take().expect("open"));
            } else {
                c.text = joined;
                c.end = w.end.max(c.end);
            }
        }
        if open.is_none() {
            open = Some(Cue::plain(w.start, w.end.max(w.start), word.to_string()));
        }
        if word.ends_with(['.', '!', '?', '。', '！', '？']) {
            out.extend(open.take());
        }
    }
    out.extend(open);
    out
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x3000..=0x30ff | 0x3400..=0x9fff | 0xf900..=0xfaff | 0xff00..=0xffef)
}

/// Parses word timestamps: an array of `{word|text, start, end}`, or an
/// object with such an array under `words`, or with `segments` each
/// holding `words` (as whisper writes them), grouped by [`group_words`].
#[cfg(feature = "import")]
pub fn parse_word_json(text: &str, max_chars: usize, gap: f32) -> Result<Vec<Cue>, String> {
    use serde_json::Value;
    let v: Value = serde_json::from_str(text).map_err(|e| format!("not JSON: {e}"))?;
    let mut raw: Vec<&Value> = Vec::new();
    match &v {
        Value::Array(a) => raw.extend(a.iter()),
        Value::Object(o) => {
            if let Some(Value::Array(a)) = o.get("words") {
                raw.extend(a.iter());
            } else if let Some(Value::Array(segs)) = o.get("segments") {
                for s in segs {
                    if let Some(Value::Array(a)) = s.get("words") {
                        raw.extend(a.iter());
                    }
                }
            }
        }
        _ => {}
    }
    if raw.is_empty() {
        return Err("no words: expected [{\"word\", \"start\", \"end\"}], {\"words\": [...]} or {\"segments\": [{\"words\": [...]}]}".into());
    }
    let mut words = Vec::with_capacity(raw.len());
    for (i, w) in raw.iter().enumerate() {
        let text = w
            .get("word")
            .or_else(|| w.get("text"))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("word {i}: no `word` (or `text`) string"))?;
        let num = |k: &str| w.get(k).and_then(Value::as_f64);
        let (Some(start), Some(end)) = (num("start"), num("end")) else {
            return Err(format!(
                "word {i} ('{text}'): `start` and `end` are seconds"
            ));
        };
        words.push(Cue::plain(start as f32, end as f32, text.to_string()));
    }
    Ok(group_words(&words, max_chars, gap))
}
