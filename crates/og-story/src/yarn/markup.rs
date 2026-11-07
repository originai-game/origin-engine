//! Yarn 3.2.2 markup, resolved after localization. Ranges use Unicode scalars,
//! the same unit as og-ui; UTF-16 is confined to upstream test expectations.
//! Semantics follow Yarn's MIT-licensed LineParser (third_party/yarn-spinner).
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Property {
    String(String),
    Number(f64),
    Bool(bool),
}

impl Property {
    fn text(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => if *b { "True" } else { "False" }.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub position: u32,
    pub length: u32,
    pub properties: BTreeMap<String, Property>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Markup {
    pub text: String,
    pub spans: Vec<Attribute>,
}

fn get<'a>(properties: &'a BTreeMap<String, Property>, name: &str) -> Option<&'a Property> {
    properties
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value)
}

fn replacement(
    name: &str,
    properties: &BTreeMap<String, Property>,
    locale: &str,
) -> Result<String, String> {
    let value = get(properties, "value").ok_or_else(|| format!("[{name}] needs value=..."))?;
    let key = match name {
        "select" => value.text(),
        "plural" | "ordinal" => {
            let Property::Number(n) = value else {
                return Err(format!("[{name}] needs a numeric value"));
            };
            if name == "plural" {
                og_i18n::plural::plural_category(locale, *n)
            } else {
                og_i18n::plural::ordinal_category(locale, *n)
            }
            .keyword()
            .into()
        }
        _ => unreachable!("only built-in replacement markers"),
    };
    let selected = get(properties, &key)
        .ok_or_else(|| format!("[{name}] has no replacement for {key}"))?
        .text();
    let value = value.text();
    let mut out = String::new();
    let mut previous = None;
    for c in selected.chars() {
        if c == '%' && previous != Some('\\') {
            out.push_str(&value);
        } else {
            out.push(c);
        }
        previous = Some(c);
    }
    Ok(out)
}

struct Marker {
    name: String,
    properties: BTreeMap<String, Property>,
    closing: bool,
    single: bool,
}

fn word(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut out = String::new();
    while chars
        .peek()
        .is_some_and(|c| c.is_alphanumeric() || *c == '_')
    {
        out.push(chars.next().unwrap());
    }
    out
}

fn whitespace(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while chars.peek().is_some_and(|c| c.is_whitespace()) {
        chars.next();
    }
}

fn property(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<Property, String> {
    whitespace(chars);
    if chars.peek() == Some(&'"') {
        chars.next();
        let mut value = String::new();
        while let Some(c) = chars.next() {
            match c {
                '"' => return Ok(Property::String(value)),
                '\\' if chars.peek() == Some(&'"') => {
                    value.push(chars.next().unwrap());
                }
                c => value.push(c),
            }
        }
        return Err("unclosed markup property string".into());
    }
    if chars.peek() == Some(&'{') {
        chars.next();
        let mut value = String::new();
        for c in chars.by_ref() {
            if c == '}' {
                return Ok(Property::String(value));
            }
            value.push(c);
        }
        return Err("unclosed interpolated markup property".into());
    }
    let mut value = String::new();
    while chars
        .peek()
        .is_some_and(|c| !c.is_whitespace() && *c != '/')
    {
        value.push(chars.next().unwrap());
    }
    if value.is_empty() {
        return Err("markup property needs a value".into());
    }
    Ok(match value.as_str() {
        value if value.eq_ignore_ascii_case("true") => Property::Bool(true),
        value if value.eq_ignore_ascii_case("false") => Property::Bool(false),
        _ if value.starts_with(|c: char| c.is_ascii_digit() || c == '-') => {
            if let Ok(integer) = value.parse::<i32>() {
                return Ok(Property::Number(f64::from(integer)));
            }
            let number = value
                .parse::<f32>()
                .map_err(|_| "invalid numeric markup property")?;
            if !number.is_finite() {
                return Err("non-finite markup property".into());
            }
            Property::Number(f64::from(number))
        }
        _ => Property::String(value),
    })
}

fn marker(source: &str) -> Result<Marker, String> {
    let mut chars = source.chars().peekable();
    whitespace(&mut chars);
    let closing = chars.peek() == Some(&'/');
    if closing {
        chars.next();
    }
    let name = word(&mut chars);
    if name.is_empty() && !closing {
        return Err("markup needs an attribute name".into());
    }
    let mut properties = BTreeMap::new();
    whitespace(&mut chars);
    if chars.peek() == Some(&'=') {
        chars.next();
        properties.insert(name.clone(), property(&mut chars)?);
    }
    let mut single = false;
    loop {
        whitespace(&mut chars);
        if chars.peek().is_none() {
            break;
        }
        if closing {
            return Err("closing markup cannot have properties".into());
        }
        if chars.peek() == Some(&'/') {
            chars.next();
            whitespace(&mut chars);
            if chars.next().is_some() {
                return Err("unexpected text after self-closing slash".into());
            }
            single = true;
            properties
                .entry("trimwhitespace".into())
                .or_insert(Property::Bool(true));
            break;
        }
        let key = word(&mut chars);
        whitespace(&mut chars);
        if key.is_empty() || chars.next() != Some('=') {
            return Err("markup property needs name=value".into());
        }
        let value = property(&mut chars)?;
        if get(&properties, &key).is_some() {
            return Err(format!("duplicate markup property {key}"));
        }
        properties.insert(key, value);
    }
    Ok(Marker {
        name,
        properties,
        closing,
        single,
    })
}

/// Produces display-ready plain text and attributes, without interpreting styles.
pub fn parse(input: &str, locale: &str) -> Result<Markup, String> {
    let input: String = input.nfc().collect();
    let mut prefix = None;
    if !input.trim_start().starts_with("[character") {
        let mut escaped = false;
        for (at, c) in input.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            if c == '\\' {
                escaped = true;
                continue;
            }
            if c == ':' {
                let end = at
                    + 1
                    + input[at + 1..]
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .map(char::len_utf8)
                        .sum::<usize>();
                prefix = Some((input[..at].replace("\\:", ":"), end));
                break;
            }
        }
    }
    let mut result = Markup::default();
    let mut open: Vec<usize> = Vec::new();
    let mut position = 0_u32;
    let mut at = 0;
    let mut character = prefix.as_ref().map(|(name, _)| Attribute {
        name: "character".into(),
        position: 0,
        length: 0,
        properties: BTreeMap::from([("name".into(), Property::String(name.clone()))]),
    });
    while at < input.len() {
        if let Some((_, end)) = &prefix
            && at == *end
        {
            character.as_mut().unwrap().length = position;
        }
        let c = input[at..].chars().next().unwrap();
        if c == '\\' {
            let next = input[at + 1..].chars().next();
            if let Some(c @ ('[' | ']' | ':')) = next {
                result.text.push(c);
                position += 1;
                at += 1 + c.len_utf8();
                continue;
            }
        }
        if c != '[' {
            result.text.push(c);
            position += 1;
            at += c.len_utf8();
            continue;
        }
        let start = at + 1;
        let mut quoted = false;
        let mut escaped = false;
        let end = input[start..]
            .char_indices()
            .find_map(|(i, c)| {
                if escaped {
                    escaped = false;
                    return None;
                }
                if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    quoted = !quoted;
                } else if c == ']' && !quoted {
                    return Some(start + i);
                }
                None
            })
            .ok_or("unclosed markup marker")?;
        let tag = marker(&input[start..end])?;
        at = end + 1;
        if tag.closing {
            if tag.name.is_empty() {
                for i in open.drain(..) {
                    result.spans[i].length = position - result.spans[i].position;
                }
            } else {
                let i = open
                    .iter()
                    .rposition(|i| result.spans[*i].name == tag.name)
                    .ok_or_else(|| format!("no open [{}] to close", tag.name))?;
                let span = &mut result.spans[open.remove(i)];
                span.length = position - span.position;
            }
            continue;
        }
        let is_replacement = matches!(tag.name.as_str(), "select" | "plural" | "ordinal");
        if is_replacement {
            if !tag.single {
                return Err(format!("[{}] replacement must be self-closing", tag.name));
            }
            let text = replacement(&tag.name, &tag.properties, locale)?.replace("\\:", ":");
            position += text.chars().count() as u32;
            result.text.push_str(&text);
            continue;
        }
        let trim =
            tag.single && get(&tag.properties, "trimwhitespace") == Some(&Property::Bool(true));
        let i = result.spans.len();
        result.spans.push(Attribute {
            name: tag.name.clone(),
            position,
            length: 0,
            properties: tag.properties,
        });
        if tag.single {
            if trim {
                // Upstream consumes one whitespace character, not the whole run.
                if let Some(c) = input[at..].chars().next().filter(|c| c.is_whitespace()) {
                    at += c.len_utf8();
                }
            }
        } else if tag.name == "nomarkup" {
            let close = input[at..]
                .find("[/nomarkup]")
                .ok_or("unclosed [nomarkup]")?;
            let raw = input[at..at + close].replace("\\:", ":");
            let length = raw.chars().count() as u32;
            result.text.push_str(&raw);
            result.spans[i].length = length;
            position += length;
            at += close + "[/nomarkup]".len();
        } else {
            open.push(i);
        }
    }
    if !open.is_empty() {
        return Err(format!("unclosed [{}] markup", result.spans[open[0]].name));
    }
    if let Some(mut span) = character {
        if prefix.as_ref().is_some_and(|(_, end)| *end == input.len()) {
            span.length = position;
        }
        result.spans.insert(0, span);
    }
    Ok(result)
}
