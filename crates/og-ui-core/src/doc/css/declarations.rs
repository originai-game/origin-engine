use super::*;

/// A `translate` written in px also writes its shifts as 0, so it replaces
/// an earlier percentage in a document's cascade; readers that hold no
/// shift (a dressing, a keyframe's channels) skip these.
pub fn zero_shift(entry: &Entry) -> bool {
    matches!(entry.key.as_str(), "xShift" | "yShift")
        && entry
            .values
            .first()
            .and_then(og_core::ogt::Value::as_number)
            .is_some_and(|n| n.as_f32() == 0.0)
}

pub fn declarations(text: &str, line: usize) -> Result<Vec<Entry>, String> {
    declarations_at([(line, text)])
}

pub(crate) fn declarations_at<'a>(
    lines: impl IntoIterator<Item = (usize, &'a str)>,
) -> Result<Vec<Entry>, String> {
    let mut frame = Frame::new(Entry::new("style"));
    for (line, text) in lines {
        for (name, value) in split_decls(text)? {
            for part in lower_decl(&name, &value, false)? {
                frame.part(part, line)?;
            }
        }
    }
    frame.finish()?;
    Ok(frame.entry.block.map_or_else(Vec::new, |b| b.entries))
}

struct Frame {
    entry: Entry,
    /// Sizes and grid lines, joined at the block's end.
    size: [Option<SizeParts>; 2],
    grid: [Option<usize>; 2],
    grid_parts: [[Option<Value>; 2]; 2],
}

impl Frame {
    fn new(entry: Entry) -> Frame {
        Frame {
            entry,
            size: [None, None],
            grid: [None, None],
            grid_parts: [[None, None], [None, None]],
        }
    }

    fn block(&mut self) -> &mut Block {
        self.entry.block.get_or_insert_with(Block::new)
    }

    fn push(&mut self, e: Entry) {
        self.block().push(e);
    }

    fn len(&self) -> usize {
        self.entry.block.as_ref().map_or(0, |b| b.entries.len())
    }

    fn part(&mut self, p: Part, line: usize) -> Result<(), String> {
        match p {
            Part::Entry(k, values) => {
                const SIDES: [&str; 4] =
                    ["paddingTop", "paddingRight", "paddingBottom", "paddingLeft"];
                // The shorthand resets the sides a longhand set before it.
                if k == "gap"
                    && self.entry.block.as_ref().is_some_and(|b| {
                        b.entries
                            .iter()
                            .any(|e| e.key == "rowGap" || e.key == "columnGap")
                    })
                {
                    let v = values.clone();
                    for side in ["rowGap", "columnGap"] {
                        let mut e = Entry::new(side);
                        e.values = v.clone();
                        e.line = line;
                        self.push(e);
                    }
                }
                let reset =
                    k == "padding"
                        && self.entry.block.as_ref().is_some_and(|b| {
                            b.entries.iter().any(|e| SIDES.contains(&e.key.as_str()))
                        });
                let sides = reset.then(|| match values.as_slice() {
                    [a] => [a.clone(), a.clone(), a.clone(), a.clone()],
                    [x, y] => [y.clone(), x.clone(), y.clone(), x.clone()],
                    [l, t, r, b] => [t.clone(), r.clone(), b.clone(), l.clone()],
                    _ => std::array::from_fn(|_| Value::Number(Number::parse("0").unwrap())),
                });
                let mut e = Entry::new(k);
                e.values = values;
                e.line = line;
                self.push(e);
                for (side, v) in SIDES.into_iter().zip(sides.into_iter().flatten()) {
                    let mut e = Entry::new(side);
                    e.values = vec![v];
                    e.line = line;
                    self.push(e);
                }
            }
            Part::Size(axis, bit) => {
                let at = self.len();
                let s = self.size[axis].get_or_insert((None, None, None, line));
                match bit {
                    SizeBit::Kind(k) => match &mut s.0 {
                        Some((_, kind)) => *kind = k,
                        None => {
                            s.0 = Some((at, k));
                            let mut e = Entry::new(if axis == 0 { "width" } else { "height" });
                            e.line = line;
                            self.push(e);
                        }
                    },
                    SizeBit::Range(true, v) => s.1 = Some(v),
                    SizeBit::Range(false, v) => s.2 = Some(v),
                }
            }
            Part::Grid(axis, start, span) => {
                for (i, v) in [start, span].into_iter().enumerate() {
                    let Some(v) = v else { continue };
                    if self.grid[i].is_none() {
                        let mut e = Entry::new(if i == 0 { "cell" } else { "span" });
                        e.line = line;
                        self.grid[i] = Some(self.len());
                        self.push(e);
                    }
                    self.grid_parts[i][axis] = Some(v);
                }
            }
        }
        Ok(())
    }

    /// Joins sizes and grid lines.
    fn finish(&mut self) -> Result<(), String> {
        for axis in 0..2 {
            let Some((kind, mut min, mut max, line)) = self.size[axis].take() else {
                continue;
            };
            // A px bound of a fit-content or stretch size is the size's own
            // range (as `width fit <min> <max>` writes it); any other bound
            // is its own prop, applied to whatever size the element takes.
            let px = |b: &Option<Vec<Value>>| matches!(b.as_deref(), Some([Value::Number(_)]));
            if let Some((at, mut v)) = kind {
                let ranged = matches!(v.as_slice(), [Value::Word(w)] if w == "fit" || w == "grow");
                if ranged
                    && (px(&min) || px(&max))
                    && (min.is_none() || px(&min))
                    && (max.is_none() || px(&max))
                {
                    let zero = Value::Number(Number::parse("0").unwrap());
                    v.push(min.take().map_or(zero, |m| m[0].clone()));
                    if let Some(m) = max.take() {
                        v.push(m[0].clone());
                    }
                }
                self.entry.block.as_mut().unwrap().entries[at].values = v;
            }
            let names = if axis == 0 {
                ["minWidth", "maxWidth"]
            } else {
                ["minHeight", "maxHeight"]
            };
            for (name, b) in names.into_iter().zip([min, max]) {
                if let Some(values) = b {
                    let mut e = Entry::new(name);
                    e.values = values;
                    e.line = line;
                    self.push(e);
                }
            }
        }
        for i in 0..2 {
            let Some(at) = self.grid[i].take() else {
                continue;
            };
            let zero = || Value::Number(Number::parse("0").unwrap());
            let [a, b] = std::mem::take(&mut self.grid_parts[i]);
            self.entry.block.as_mut().unwrap().entries[at].values =
                vec![a.unwrap_or_else(zero), b.unwrap_or_else(zero)];
        }
        Ok(())
    }
}

/// A line's declarations, split at `;` outside strings and parentheses, and
/// cut at a `#` comment (a `#` that starts no colour).
pub fn split_decls(t: &str) -> Result<Vec<(String, String)>, String> {
    let c: Vec<char> = t.chars().collect();
    let mut parts = Vec::new();
    let (mut cur, mut depth, mut i, mut in_str) = (String::new(), 0, 0, false);
    while i < c.len() {
        let ch = c[i];
        if in_str {
            cur.push(ch);
            if ch == '\\' && i + 1 < c.len() {
                cur.push(c[i + 1]);
                i += 1;
            } else if ch == '"' {
                in_str = false;
            }
        } else if ch == '"' {
            in_str = true;
            cur.push(ch);
        } else if ch == '#' && depth == 0 && !c.get(i + 1).is_some_and(|d| d.is_ascii_hexdigit()) {
            // A comment; inside parentheses a `#` is a reference (`url(#id)`).
            break;
        } else if ch == '(' {
            depth += 1;
            cur.push(ch);
        } else if ch == ')' {
            depth -= 1;
            cur.push(ch);
        } else if ch == ';' && depth == 0 {
            parts.push(std::mem::take(&mut cur));
        } else {
            cur.push(ch);
        }
        i += 1;
    }
    parts.push(cur);
    let mut out = Vec::new();
    for p in parts {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        let (name, value) = p
            .split_once(':')
            .ok_or_else(|| format!("'{p}' is not a declaration (name: value)"))?;
        let value = value.trim();
        if value.ends_with("!important") {
            return Err("there is no !important: an element's later declaration wins".into());
        }
        out.push((name.trim().to_string(), value.to_string()));
    }
    Ok(out)
}
