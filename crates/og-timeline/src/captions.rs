//! Caption files a caption track imports (`captions "narration.srt"`):
//! SubRip (`.srt`) cues, and word timestamps (`.json`, as speech-to-text
//! tools write them) grouped into lines.

/// One caption: its text from `start` to `end`, in seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    pub start: f32,
    pub end: f32,
    pub text: String,
}

/// `HH:MM:SS,mmm` (or `.mmm`) in seconds.
fn srt_time(s: &str) -> Option<f32> {
    let s = s.trim();
    let (hms, ms) = s.split_once([',', '.']).unwrap_or((s, "0"));
    let mut parts = hms.split(':');
    let (h, m, sec) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    let n = |x: &str| x.trim().parse::<u32>().ok();
    let ms = ms.trim();
    let frac = if ms.is_empty() || !ms.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    } else {
        ms.parse::<u32>().ok()? as f64 / 10f64.powi(ms.len() as i32)
    };
    Some((f64::from(n(h)?) * 3600.0 + f64::from(n(m)?) * 60.0 + f64::from(n(sec)?) + frac) as f32)
}

/// Strips SubRip's inline tags (`<i>`, `{\an8}`) from a line.
fn plain(line: &str) -> String {
    let mut out = String::new();
    let mut depth = (0, 0);
    for c in line.chars() {
        match c {
            '<' => depth.0 += 1,
            '>' if depth.0 > 0 => depth.0 -= 1,
            '{' => depth.1 += 1,
            '}' if depth.1 > 0 => depth.1 -= 1,
            _ if depth == (0, 0) => out.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}

/// Parses SubRip text: blocks of an optional index line, `start --> end`,
/// and text lines, separated by blank lines. Errors name the line.
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
        let (start, end) = match (srt_time(a), srt_time(b)) {
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
            let p = plain(lines[i]);
            if !p.is_empty() {
                body.push(p);
            }
            i += 1;
        }
        cues.push(Cue {
            start,
            end,
            text: body.join("\n"),
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
            open = Some(Cue {
                start: w.start,
                end: w.end.max(w.start),
                text: word.to_string(),
            });
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
        words.push(Cue {
            start: start as f32,
            end: end as f32,
            text: text.to_string(),
        });
    }
    Ok(group_words(&words, max_chars, gap))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srt_cues_with_and_without_indexes_tags_and_hints() {
        let cues = parse_srt(
            "\u{feff}1\n00:00:01,000 --> 00:00:03,500\nThe yard is <i>quiet</i>.\n\n\n00:00:04.250 --> 00:00:06,000 X1:10\n{\\an8}Too quiet.\nSecond line\n",
        )
        .unwrap();
        assert_eq!(
            cues,
            vec![
                Cue {
                    start: 1.0,
                    end: 3.5,
                    text: "The yard is quiet.".into()
                },
                Cue {
                    start: 4.25,
                    end: 6.0,
                    text: "Too quiet.\nSecond line".into()
                },
            ]
        );
        let e = parse_srt("1\n00:00:01 --> soon\nhi\n").unwrap_err();
        assert!(e.starts_with("line 2:"), "{e}");
    }

    #[test]
    fn words_group_at_sentences_pauses_and_width() {
        let w = |t: &str, s: f32, e: f32| Cue {
            start: s,
            end: e,
            text: t.into(),
        };
        let lines = group_words(
            &[
                w("Hello", 0.0, 0.4),
                w("there.", 0.5, 0.9),
                w("A", 1.0, 1.1),
                w("long", 1.1, 1.3),
                w("pause", 1.3, 1.6),
                w("then", 2.6, 2.8),
                w("more", 2.8, 3.0),
            ],
            42,
            0.6,
        );
        let texts: Vec<_> = lines
            .iter()
            .map(|c| (c.text.as_str(), c.start, c.end))
            .collect();
        assert_eq!(
            texts,
            vec![
                ("Hello there.", 0.0, 0.9),
                ("A long pause", 1.0, 1.6),
                ("then more", 2.6, 3.0)
            ]
        );
        let narrow = group_words(&[w("abcdef", 0.0, 1.0), w("ghij", 1.0, 2.0)], 8, 1.0);
        assert_eq!(narrow.len(), 2);
    }

    #[cfg(feature = "import")]
    #[test]
    fn whisper_segments_and_plain_arrays_parse() {
        let a = parse_word_json(r#"{"segments":[{"words":[{"word":" Go","start":0,"end":0.3},{"word":" now!","start":0.3,"end":0.6}]}]}"#, 42, 0.6).unwrap();
        assert_eq!(
            a,
            vec![Cue {
                start: 0.0,
                end: 0.6,
                text: "Go now!".into()
            }]
        );
        let b = parse_word_json(r#"[{"text":"Hi","start":1,"end":1.2}]"#, 42, 0.6).unwrap();
        assert_eq!(b[0].start, 1.0);
        assert!(parse_word_json(r#"{"x":1}"#, 42, 0.6).is_err());
    }
}
