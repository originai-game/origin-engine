//! Svelte's non-executable markup subset. Offsets always refer to authored text.

use crate::bind::Expr;

#[derive(Clone, Debug)]
pub(super) struct Attr {
    pub name: String,
    pub value: String,
    pub bound: bool,
    pub at: usize,
    pub end: usize,
}

#[derive(Clone, Debug)]
pub(super) enum Node {
    Element {
        tag: String,
        attrs: Vec<Attr>,
        children: Vec<Node>,
        at: usize,
        end: usize,
        close: usize,
    },
    Text(String, usize),
    Expression(String, usize),
    If {
        condition: String,
        yes: Vec<Node>,
        no: Vec<Node>,
        at: usize,
    },
    Each {
        list: String,
        item: String,
        key: String,
        body: Vec<Node>,
        empty: Vec<Node>,
        at: usize,
    },
    Snippet {
        name: String,
        body: Vec<Node>,
        at: usize,
    },
    Render {
        name: String,
        optional: bool,
        at: usize,
    },
    /// `{#key expr}`: its content is rebuilt when `expr`'s value changes.
    Key {
        expr: String,
        body: Vec<Node>,
        at: usize,
    },
    /// `{@const name = expr}`: a name for `expr` in the enclosing block.
    Const {
        name: String,
        expr: String,
        at: usize,
    },
    /// `{#await expr}`: a store holding a settled-promise record
    /// (`{ status: "pending" | "fulfilled" | "rejected", value, reason }`).
    Await {
        expr: String,
        pending: Vec<Node>,
        then: Option<(String, Vec<Node>)>,
        catch: Option<(String, Vec<Node>)>,
        at: usize,
    },
    /// A `<script>`'s text, read by the props-and-derived subset.
    Script(String, usize),
}

pub(super) type Error = (usize, String);
pub(super) fn location(text: &str, at: usize) -> (usize, usize) {
    let prefix = &text[..at];
    (
        prefix.bytes().filter(|&b| b == b'\n').count() + 1,
        prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1,
    )
}
pub(super) fn diagnostic(source: &str, text: &str, error: Error) -> String {
    let (line, col) = location(text, error.0);
    format!("{source}:{line}:{col}: {}", error.1)
}
pub(super) fn parse(text: &str) -> Result<Vec<Node>, Error> {
    let mut parser = Parser { text, at: 0 };
    let (nodes, end) = parser.nodes()?;
    if !end.is_empty() {
        return Err((
            parser.at,
            format!("unexpected {end}; remove the unmatched closing block"),
        ));
    }
    Ok(nodes)
}
struct Parser<'a> {
    text: &'a str,
    at: usize,
}
impl Parser<'_> {
    fn rest(&self) -> &str {
        &self.text[self.at..]
    }
    fn fail<T>(&self, why: impl Into<String>) -> Result<T, Error> {
        Err((self.at, why.into()))
    }
    fn take(&mut self, s: &str) -> bool {
        if self.rest().starts_with(s) {
            self.at += s.len();
            true
        } else {
            false
        }
    }
    fn space(&mut self) {
        while self.rest().starts_with(char::is_whitespace) {
            self.at += self.rest().chars().next().unwrap().len_utf8();
        }
    }
    fn name(&mut self) -> Result<String, Error> {
        let start = self.at;
        while let Some(c) = self.rest().chars().next() {
            // `|` joins a directive's modifiers (`transition:fade|global`).
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':' | '.' | '$' | '|') {
                self.at += 1;
            } else {
                break;
            }
        }
        if start == self.at {
            return self.fail("expected an element or attribute name");
        }
        Ok(self.text[start..self.at].into())
    }
    fn expression(&mut self) -> Result<String, Error> {
        if !self.take("{") {
            return self.fail("write an expression in {braces}");
        }
        let start = self.at;
        let mut depth = 0;
        let mut quote = None;
        while let Some(c) = self.rest().chars().next() {
            if quote.is_none() && depth == 0 && c == '}' {
                let text = self.text[start..self.at].trim().to_string();
                self.at += 1;
                return Ok(text);
            }
            self.at += c.len_utf8();
            if let Some(q) = quote {
                if c == '\\' {
                    if let Some(c) = self.rest().chars().next() {
                        self.at += c.len_utf8();
                    }
                } else if c == q {
                    quote = None;
                }
            } else {
                match c {
                    '\'' | '"' => quote = Some(c),
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
            }
        }
        self.fail("unclosed expression; add '}'")
    }
    fn nodes(&mut self) -> Result<(Vec<Node>, String), Error> {
        let mut nodes = Vec::new();
        while !self.rest().is_empty() {
            let at = self.at;
            if self.take("<!--") {
                let Some(end) = self.rest().find("-->") else {
                    return self.fail("unclosed comment; add -->");
                };
                self.at += end + 3;
            } else if self.take("</") {
                let tag = self.name()?;
                self.space();
                if !self.take(">") {
                    return self.fail("close the end tag with '>'");
                }
                return Ok((nodes, format!("</{tag}>")));
            } else if self.rest().starts_with('<') {
                nodes.push(self.element()?);
            } else if self.rest().starts_with('{') {
                let expr = self.expression()?;
                if expr.starts_with(':') || expr.starts_with('/') {
                    return Ok((nodes, expr));
                }
                if let Some(condition) = expr.strip_prefix("#if ") {
                    nodes.push(self.if_block(condition.to_string(), at)?);
                } else if let Some(each) = expr.strip_prefix("#each ") {
                    let Some((list, tail)) = each.rsplit_once(" as ") else {
                        return self.fail("write {#each $items as item (item.id)}");
                    };
                    let Some((item, key)) = tail.split_once('(') else {
                        return self.fail(
                            "each needs a stable key; write {#each $items as item (item.id)}",
                        );
                    };
                    let Some(key) = key.strip_suffix(')') else {
                        return self.fail("close the each key with ')'");
                    };
                    let item = item.trim();
                    if !identifier(item) {
                        return self
                            .fail("each takes one item alias, not an index; key by item.id");
                    }
                    if key.trim() == item || key.contains("$index") {
                        return self
                            .fail("key each rows by a stable string field, such as item.id");
                    }
                    let (body, end) = self.nodes()?;
                    let (empty, end) = if end == ":else" {
                        self.nodes()?
                    } else {
                        (Vec::new(), end)
                    };
                    if end != "/each" {
                        return self.fail("close the each block with {/each}");
                    }
                    nodes.push(Node::Each {
                        list: list.trim().into(),
                        item: item.into(),
                        key: key.trim().into(),
                        body,
                        empty,
                        at,
                    });
                } else if let Some(signature) = expr.strip_prefix("#snippet ") {
                    let Some(name) = signature.trim().strip_suffix("()") else {
                        return self.fail("a content snippet is {#snippet name()} … {/snippet}");
                    };
                    if !identifier(name) {
                        return self.fail("give the snippet a simple name");
                    }
                    let (body, end) = self.nodes()?;
                    if end != "/snippet" {
                        return self.fail("close the snippet with {/snippet}");
                    }
                    nodes.push(Node::Snippet {
                        name: name.into(),
                        body,
                        at,
                    });
                } else if let Some(call) = expr.strip_prefix("@render ") {
                    let call = call.trim();
                    let optional = call.ends_with("?.()");
                    let Some(name) = call.strip_suffix(if optional { "?.()" } else { "()" }) else {
                        return self.fail(
                            "render named content with {@render name()} or {@render name?.()}",
                        );
                    };
                    if !identifier(name) {
                        return self
                            .fail("render a named snippet; executable calls are not supported");
                    }
                    nodes.push(Node::Render {
                        name: name.into(),
                        optional,
                        at,
                    });
                } else if let Some(key) = expr.strip_prefix("#key ") {
                    let (body, end) = self.nodes()?;
                    if end != "/key" {
                        return self.fail("close the key block with {/key}");
                    }
                    nodes.push(Node::Key {
                        expr: key.trim().into(),
                        body,
                        at,
                    });
                } else if let Some(decl) = expr.strip_prefix("@const ") {
                    let Some((name, value)) = decl
                        .split_once('=')
                        .filter(|(_, v)| !v.starts_with('=') && !v.trim().is_empty())
                    else {
                        return Err((at, "write {@const name = expression}".into()));
                    };
                    let name = name.trim();
                    if !identifier(name) {
                        return Err((
                            at,
                            "{@const} names one value; destructuring is not supported".into(),
                        ));
                    }
                    nodes.push(Node::Const {
                        name: name.into(),
                        expr: value.trim().into(),
                        at,
                    });
                } else if let Some(head) = expr.strip_prefix("#await ") {
                    nodes.push(self.await_block(head, at)?);
                } else if expr.starts_with("@html") {
                    return Err((
                        at,
                        "{@html} parses markup while the game runs, which a document never does; write the elements, render content with {@render name()}, or prepare rich text in a TS system and pass it through a store".into(),
                    ));
                } else if expr.starts_with(['@', '#']) {
                    return self.fail("unsupported Svelte block; use if, keyed each, key, await, snippet, render or const");
                } else {
                    nodes.push(Node::Expression(expr, at));
                }
            } else {
                let len = self.rest().find(['<', '{']).unwrap_or(self.rest().len());
                let text = decode(&self.rest()[..len]).map_err(|why| (at, why))?;
                self.at += len;
                nodes.push(Node::Text(text, at));
            }
        }
        Ok((nodes, String::new()))
    }
    /// `{#await $load}…{:then value}…{:catch error}…{/await}`, and the
    /// short forms `{#await $load then value}` and `{#await $load catch e}`.
    fn await_block(&mut self, head: &str, at: usize) -> Result<Node, Error> {
        let alias = |s: &str| -> Result<String, Error> {
            let s = s.trim();
            if s.is_empty() || identifier(s) {
                Ok(s.into())
            } else {
                Err((
                    at,
                    format!("{s} is not a name; {{:then value}} binds one name"),
                ))
            }
        };
        let (expr, mut first) = if let Some((e, v)) = head.split_once(" then ") {
            (e, Some(("then", alias(v)?)))
        } else if let Some((e, v)) = head.split_once(" catch ") {
            (e, Some(("catch", alias(v)?)))
        } else {
            (head, None)
        };
        let expr = expr.trim();
        if expr.is_empty() {
            return Err((
                at,
                "write {#await $store} with the store holding the result".into(),
            ));
        }
        let mut pending = Vec::new();
        let mut then = None;
        let mut catch = None;
        let (mut body, mut end) = self.nodes()?;
        loop {
            match first.take() {
                Some(("then", name)) if then.is_none() && catch.is_none() => {
                    then = Some((name, body))
                }
                Some(("catch", name)) if catch.is_none() => catch = Some((name, body)),
                Some(_) => {
                    return self.fail("an await block takes one {:then} before one {:catch}");
                }
                None => pending = body,
            }
            if end == "/await" {
                break;
            }
            if let Some(v) = end.strip_prefix(":then") {
                first = Some(("then", alias(v)?));
            } else if let Some(v) = end.strip_prefix(":catch") {
                first = Some(("catch", alias(v)?));
            } else {
                return self.fail("close the await block with {/await}");
            }
            (body, end) = self.nodes()?;
        }
        Ok(Node::Await {
            expr: expr.into(),
            pending,
            then,
            catch,
            at,
        })
    }
    fn if_block(&mut self, condition: String, at: usize) -> Result<Node, Error> {
        let (yes, end) = self.nodes()?;
        let no = if let Some(condition) = end.strip_prefix(":else if ") {
            vec![self.if_block(condition.to_string(), self.at)?]
        } else if end == ":else" {
            let (no, end) = self.nodes()?;
            if end != "/if" {
                return self.fail("close the if block with {/if}");
            }
            no
        } else if end == "/if" {
            Vec::new()
        } else {
            return self.fail("close the if block with {/if}");
        };
        Ok(Node::If {
            condition,
            yes,
            no,
            at,
        })
    }
    fn element(&mut self) -> Result<Node, Error> {
        let at = self.at;
        self.take("<");
        let tag = self.name()?;
        match tag.as_str() {
            "script" => {
                // Only the props-and-derived subset is read (svelte::script).
                self.space();
                if self.rest().starts_with("lang") {
                    let _ = self.name()?;
                    self.space();
                    if self.take("=") {
                        self.space();
                        let lang = ["\"ts\"", "'ts'", "\"typescript\""]
                            .into_iter()
                            .find(|l| self.rest().starts_with(l));
                        let Some(lang) = lang else {
                            return self.fail("a <script> is TypeScript: lang=\"ts\" or none");
                        };
                        self.at += lang.len();
                    }
                    self.space();
                }
                if !self.take(">") {
                    return self.fail("a <script> takes no attributes but lang=\"ts\"; module scripts are not supported");
                }
                let start = self.at;
                let Some(len) = self.rest().find("</script>") else {
                    return self.fail("close the script with </script>");
                };
                self.at += len + "</script>".len();
                return Ok(Node::Script(self.text[start..start + len].into(), start));
            }
            "slot" => {
                return self.fail("Svelte 5 uses snippets; replace <slot> with {@render name?.()}");
            }
            _ => {}
        }
        let mut attrs: Vec<Attr> = Vec::new();
        let closed;
        loop {
            self.space();
            if self.take("/>") {
                closed = true;
                break;
            }
            if self.take(">") {
                // HTML's void elements among the core tags.
                closed = matches!(tag.as_str(), "meta" | "img" | "input" | "hr");
                break;
            }
            let at = self.at;
            let (name, value, bound) = if self.rest().starts_with('{') {
                let name = self.expression()?;
                if !identifier(&name) {
                    return self.fail("attribute shorthand is {name}; use attr={expression}");
                }
                (name.clone(), name, true)
            } else {
                let name = self.name()?;
                if name.starts_with("on:") {
                    return self.fail(format!(
                        "Svelte 5 events use {}={{action}} instead of {name}",
                        name.replace("on:", "on")
                    ));
                }
                self.space();
                if self.take("=") {
                    self.space();
                    if self.rest().starts_with('{') {
                        (name, self.expression()?, true)
                    } else {
                        let Some(q @ ('\'' | '"')) = self.rest().chars().next() else {
                            return self.fail("quote a literal attribute or use attr={expression}");
                        };
                        self.at += 1;
                        let Some(end) = self.rest().find(q) else {
                            return self.fail("close the attribute's quote");
                        };
                        let value = decode(&self.rest()[..end]).map_err(|why| (at, why))?;
                        self.at += end + 1;
                        (name, value, false)
                    }
                } else {
                    (name, "true".into(), false)
                }
            };
            if attrs.iter().any(|a| a.name == name) {
                return self.fail(format!("duplicate attribute {name}; keep one value"));
            }
            attrs.push(Attr {
                name,
                value,
                bound,
                at,
                end: self.text[..self.at].trim_end().len(),
            });
        }
        let end = self.at;
        let children = if closed {
            Vec::new()
        } else if tag == "style" {
            let Some(len) = self.rest().find("</style>") else {
                return self.fail("close the style block with </style>");
            };
            let text = self.rest()[..len].to_string();
            self.at += len + "</style>".len();
            vec![Node::Text(text, end)]
        } else {
            let (children, close) = self.nodes()?;
            if close != format!("</{tag}>") {
                return self.fail(format!("unexpected {close}; close <{tag}> with </{tag}>"));
            }
            children
        };
        Ok(Node::Element {
            tag,
            attrs,
            children,
            at,
            end,
            close: if closed {
                end
            } else {
                self.text[..self.at].rfind("</").unwrap()
            },
        })
    }
}
pub(super) fn identifier(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
fn decode(text: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let Some(end) = rest
            .find(';')
            .filter(|&i| !rest[1..i].contains(char::is_whitespace))
        else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let n = &rest[1..end];
        let c = match n {
            "amp" => '&',
            "lt" => '<',
            "gt" => '>',
            "quot" => '"',
            "apos" => '\'',
            "nbsp" => '\u{a0}',
            n if n.starts_with('#') => {
                let num = if let Some(hex) = n.strip_prefix("#x").or_else(|| n.strip_prefix("#X")) {
                    u32::from_str_radix(hex, 16).ok()
                } else {
                    n[1..].parse().ok()
                };
                num.and_then(char::from_u32)
                    .ok_or_else(|| format!("invalid entity &{n}; use a Unicode character"))?
            }
            _ => {
                return Err(format!(
                    "unknown entity &{n}; use a Unicode character or numeric entity"
                ));
            }
        };
        out.push(c);
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

pub(super) fn expression(source: &str, at: usize) -> Result<Expr, Error> {
    let expr = Expr::parse(source).map_err(|why| (
        at,
        format!("{why}; use a side-effect-free expression or t(key, {{ named: value }}); sources come from the game's mounted stores ($name)"),
    ))?;
    let mut paths = Vec::new();
    expr.paths(&mut paths);
    if paths.iter().any(|path| {
        ["$state", "$derived", "$effect", "$props", "$bindable"]
            .contains(&path.split('.').next().unwrap_or_default())
    }) {
        return Err((
            at,
            "runes are not allowed; sources come from the game's mounted stores ($name)".into(),
        ));
    }
    Ok(expr)
}
