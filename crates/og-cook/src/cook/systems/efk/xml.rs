//! The element tree an Effekseer project is read into: an `.efkproj`'s
//! XML, or the `EDIT` chunk of an `.efkefc`, which holds the same document
//! compressed (Effekseer's `IO/EfkEfc.cs`, MIT; `third_party/effekseer/`).
//!
//! The project XML has no attributes worth reading and no mixed content:
//! an element holds text or child elements. This reads exactly that.

/// One element: its name, its text (trimmed) and its children.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct El {
    pub name: String,
    pub text: String,
    pub children: Vec<El>,
}

impl El {
    /// The first child named `name`.
    pub fn child(&self, name: &str) -> Option<&El> {
        self.children.iter().find(|c| c.name == name)
    }

    /// The element at a `/`-separated path of child names.
    pub fn at(&self, path: &str) -> Option<&El> {
        path.split('/')
            .filter(|p| !p.is_empty())
            .try_fold(self, |e, name| e.child(name))
    }

    /// The text at `path`, when the element is there and holds some.
    pub fn text_at(&self, path: &str) -> Option<&str> {
        self.at(path)
            .map(|e| e.text.as_str())
            .filter(|t| !t.is_empty())
    }
}

/// Reads a project from an `.efkproj` (XML) or `.efkefc` (binary) file's bytes.
pub fn read_project(bytes: &[u8]) -> Result<El, String> {
    let root = if bytes.starts_with(b"EFKE") {
        read_efkefc(bytes)?
    } else {
        let text = std::str::from_utf8(bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes))
            .map_err(|_| "an .efkproj is UTF-8 XML".to_string())?;
        parse_xml(text)?
    };
    root.children
        .into_iter()
        .find(|c| c.name == "EffekseerProject")
        .ok_or_else(|| "no <EffekseerProject> element: not an Effekseer project".to_string())
}

/// Parses XML into a document element whose children are the top-level elements.
pub fn parse_xml(text: &str) -> Result<El, String> {
    let b = text.as_bytes();
    let mut i = 0;
    let mut stack: Vec<El> = vec![El::default()];
    let err = |i: usize, what: &str| {
        let line = 1 + b[..i.min(b.len())].iter().filter(|&&c| c == b'\n').count();
        format!("XML line {line}: {what}")
    };
    while i < b.len() {
        if b[i] != b'<' {
            let end = b[i..]
                .iter()
                .position(|&c| c == b'<')
                .map_or(b.len(), |p| i + p);
            let raw = &text[i..end];
            if !raw.trim().is_empty() {
                let top = stack.last_mut().expect("the document");
                top.text.push_str(&unescape(raw.trim()));
            }
            i = end;
            continue;
        }
        let rest = &text[i..];
        if rest.starts_with("<?") {
            i += rest.find("?>").ok_or_else(|| err(i, "an unclosed <?"))? + 2;
        } else if rest.starts_with("<!--") {
            i += rest
                .find("-->")
                .ok_or_else(|| err(i, "an unclosed comment"))?
                + 3;
        } else if rest.starts_with("<![CDATA[") {
            let end = rest
                .find("]]>")
                .ok_or_else(|| err(i, "an unclosed CDATA"))?;
            stack
                .last_mut()
                .expect("the document")
                .text
                .push_str(&rest[9..end]);
            i += end + 3;
        } else if rest.starts_with("<!") {
            i += rest.find('>').ok_or_else(|| err(i, "an unclosed <!"))? + 1;
        } else if let Some(close) = rest.strip_prefix("</") {
            let end = close
                .find('>')
                .ok_or_else(|| err(i, "an unclosed end tag"))?;
            let name = close[..end].trim();
            let el = stack.pop().filter(|_| !stack.is_empty());
            match el {
                Some(el) if el.name == name => {
                    stack.last_mut().expect("a parent").children.push(el)
                }
                _ => return Err(err(i, &format!("</{name}> closes nothing open"))),
            }
            i += 2 + end + 1;
        } else {
            let end = rest.find('>').ok_or_else(|| err(i, "an unclosed tag"))?;
            let inner = &rest[1..end];
            let empty = inner.ends_with('/');
            let inner = inner.trim_end_matches('/');
            let name = inner
                .split(|c: char| c.is_whitespace())
                .next()
                .unwrap_or_default();
            if name.is_empty() {
                return Err(err(i, "a tag without a name"));
            }
            let el = El {
                name: name.to_string(),
                ..El::default()
            };
            if empty {
                stack.last_mut().expect("a parent").children.push(el);
            } else {
                stack.push(el);
            }
            i += end + 1;
        }
    }
    if stack.len() != 1 {
        return Err(format!(
            "XML: <{}> is never closed",
            stack.last().map_or("", |e| e.name.as_str())
        ));
    }
    Ok(stack.pop().expect("the document"))
}

fn unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(p) = rest.find('&') {
        out.push_str(&rest[..p]);
        rest = &rest[p..];
        let Some(end) = rest.find(';') else { break };
        let ent = &rest[1..end];
        let ch = match ent {
            "lt" => Some('<'),
            "gt" => Some('>'),
            "amp" => Some('&'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => ent
                .strip_prefix("#x")
                .and_then(|h| u32::from_str_radix(h, 16).ok())
                .or_else(|| ent.strip_prefix('#').and_then(|d| d.parse().ok()))
                .and_then(char::from_u32),
        };
        match ch {
            Some(c) => {
                out.push(c);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// An `.efkefc`: `EFKE`, a version, then chunks of a four-byte tag and a
/// length; `EDIT` holds the editor's document, zlib-compressed, as a table
/// of element names, a table of text values and the element tree by index
/// (`EfkEfcXml.Decompress`).
fn read_efkefc(bytes: &[u8]) -> Result<El, String> {
    let mut pos = 8;
    let mut edit = None;
    while pos + 8 <= bytes.len() {
        let tag = &bytes[pos..pos + 4];
        let size = i32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().expect("4 bytes"));
        let start = pos + 8;
        let end = usize::try_from(size)
            .ok()
            .and_then(|s| start.checked_add(s))
            .filter(|&e| e <= bytes.len())
            .ok_or("an .efkefc chunk runs past the file's end")?;
        if tag == b"EDIT" {
            edit = Some(&bytes[start..end]);
        }
        pos = end;
    }
    let edit = edit.ok_or(
        "the .efkefc has no EDIT chunk (a runtime-only export): re-save it from the Effekseer editor",
    )?;
    let data = miniz_oxide::inflate::decompress_to_vec_zlib(edit)
        .map_err(|e| format!("the .efkefc's EDIT chunk does not inflate: {e:?}"))?;
    let mut r = Reader { b: &data, at: 0 };
    let names = r.table()?;
    let values = r.table()?;
    let mut doc = El::default();
    r.elements(&mut doc, &names, &values, 0)?;
    Ok(doc)
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let s = self
            .b
            .get(self.at..self.at + n)
            .ok_or("the .efkefc's EDIT data ends early")?;
        self.at += n;
        Ok(s)
    }

    fn i16(&mut self) -> Result<i16, String> {
        Ok(i16::from_le_bytes(
            self.take(2)?.try_into().expect("2 bytes"),
        ))
    }

    fn flag(&mut self) -> Result<bool, String> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().expect("4 bytes")) > 0)
    }

    /// `count` (i16), then per entry a UTF-8 string (u16 length) and its index (i16).
    fn table(&mut self) -> Result<Vec<String>, String> {
        let n = self.i16()?.max(0) as usize;
        let mut t = vec![String::new(); n];
        for _ in 0..n {
            let len = u16::from_le_bytes(self.take(2)?.try_into().expect("2 bytes")) as usize;
            let s = String::from_utf8_lossy(self.take(len)?).into_owned();
            let i = self.i16()?;
            if let Some(slot) = usize::try_from(i).ok().and_then(|i| t.get_mut(i)) {
                *slot = s;
            }
        }
        Ok(t)
    }

    fn elements(
        &mut self,
        parent: &mut El,
        names: &[String],
        values: &[String],
        depth: usize,
    ) -> Result<(), String> {
        if depth > 256 {
            return Err("the .efkefc's element tree is too deep".into());
        }
        let n = self.i16()?.max(0);
        for _ in 0..n {
            let name = names
                .get(self.i16()?.max(0) as usize)
                .cloned()
                .ok_or("an .efkefc element names no known key")?;
            let mut el = El {
                name,
                ..El::default()
            };
            if self.flag()? {
                el.text = values
                    .get(self.i16()?.max(0) as usize)
                    .cloned()
                    .ok_or("an .efkefc element names no known value")?
                    .trim()
                    .to_string();
            }
            if self.flag()? {
                self.elements(&mut el, names, values, depth + 1)?;
            }
            parent.children.push(el);
        }
        Ok(())
    }
}
