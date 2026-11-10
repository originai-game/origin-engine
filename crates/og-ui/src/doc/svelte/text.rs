use super::*;

impl Parser {
    /// Whether the element nearest above `parent` (through blocks) is a list.
    /// The selector tag of the element enclosing `parent` (itself, when it
    /// is one), past blocks and other non-elements.
    pub(super) fn parent_tag(&self, mut parent: usize) -> Option<&str> {
        while parent != NONE {
            let n = &self.doc.nodes[parent];
            match n.kind {
                NodeKind::Element(_) | NodeKind::Use(_) => return n.selector_tag.as_deref(),
                _ => parent = n.parent,
            }
        }
        None
    }

    pub(super) fn in_list(&self, mut parent: usize) -> bool {
        while parent != NONE {
            let n = &self.doc.nodes[parent];
            match n.kind {
                NodeKind::Element(_) | NodeKind::Use(_) => {
                    return matches!(n.selector_tag.as_deref(), Some("ul" | "ol"));
                }
                _ => parent = n.parent,
            }
        }
        // A component whose root is an item is placed in its user's list.
        true
    }

    /// An anonymous text element of `run`'s inline content under block
    /// `parent`; nothing for whitespace alone.
    pub(super) fn anonymous_text(
        &mut self,
        run: &mut Vec<Node>,
        parent: usize,
        translated: bool,
    ) -> Result<Option<usize>, Error> {
        let nodes = std::mem::take(run);
        if nodes
            .iter()
            .all(|n| matches!(n, Node::Text(t, _) if t.trim().is_empty()))
        {
            return Ok(None);
        }
        let at = match &nodes[0] {
            Node::Text(_, at) | Node::Expression(_, at) | Node::Element { at, .. } => *at,
            _ => 0,
        };
        let i = self.push(NodeKind::Element("label"), parent, at);
        self.text_runs(i, &nodes, None, at, translated)?;
        self.doc.nodes[parent].children.push(i);
        Ok(Some(i))
    }

    /// Text element `i`'s content as inline runs: its `text` (the runs'
    /// plain text), and the runs themselves when any is styled or a link.
    /// An SVG `<text>` whose `<tspan x>`s start lines: a group carrying
    /// the text's attributes (so they inherit), holding one `<text>` per
    /// line at its own point. A line's `y` is its tspan's, else the line
    /// before's, moved by every `dy` since (SVG's current text position).
    /// `None`: no tspan starts a line.
    pub(super) fn svg_lines(
        &mut self,
        attrs: &[Attr],
        white_space: Option<(bool, bool)>,
        children: &[Node],
        at: usize,
        parent: usize,
    ) -> Result<Option<usize>, Error> {
        const PLACE: [&str; 4] = ["x", "y", "dx", "dy"];
        let line = |n: &Node| match n {
            Node::Element { tag, attrs, .. } if tag == "tspan" => {
                attrs.iter().any(|a| PLACE.contains(&a.name.as_str()))
            }
            _ => false,
        };
        if !children.iter().any(line) {
            return Ok(None);
        }
        let attr = |name: &str, value: String, at: usize| Attr {
            name: name.into(),
            value,
            bound: false,
            at,
            end: at,
        };
        let own = |name: &str| attrs.iter().find(|a| a.name == name).cloned();
        if let Some(a) = own("dy").filter(|a| a.bound) {
            return Err((a.at, "a <text> of several lines takes a literal dy".into()));
        }
        let mut ws = Vec::new();
        if let Some((spaces, breaks)) = white_space {
            let v = match (spaces, breaks) {
                (true, true) => "pre",
                (false, true) => "pre-line",
                _ => "preserve",
            };
            ws.push(attr("white-space", v.into(), at));
        }
        // The first line: the text's own place and what comes before the
        // first tspan that starts a line.
        let mut lines: Vec<(Vec<Attr>, Vec<Node>)> =
            vec![(PLACE.iter().filter_map(|p| own(p)).collect(), Vec::new())];
        let mut y = own("y");
        let mut dys: Vec<String> = own("dy").map(|a| a.value).into_iter().collect();
        for child in children {
            let Node::Element {
                tag,
                attrs: t,
                children: inner,
                at: tat,
                end,
                close,
            } = child
            else {
                lines.last_mut().expect("a line").1.push(child.clone());
                continue;
            };
            if !line(child) {
                lines.last_mut().expect("a line").1.push(child.clone());
                continue;
            }
            let get = |n: &str| t.iter().find(|a| a.name == n).cloned();
            let Some(x) = get("x") else {
                return Err((
                    *tat,
                    "a <tspan> that moves starts a line: give it x (and y or dy); a run within a line takes no y, dx or dy".into(),
                ));
            };
            if let Some(a) = get("dy").filter(|a| a.bound) {
                return Err((a.at, "a line's dy is literal".into()));
            }
            if let Some(ty) = get("y") {
                y = Some(ty);
                dys.clear();
            }
            dys.extend(get("dy").map(|a| a.value));
            let mut place = vec![x];
            place.extend(y.clone());
            place.extend(get("dx"));
            if !dys.is_empty() {
                place.push(attr("dy", dys.join(" "), *tat));
            }
            let run = Node::Element {
                tag: tag.clone(),
                attrs: t
                    .iter()
                    .filter(|a| !PLACE.contains(&a.name.as_str()))
                    .cloned()
                    .collect(),
                children: inner.clone(),
                at: *tat,
                end: *end,
                close: *close,
            };
            lines.push((place, vec![run]));
        }
        let group: Vec<Attr> = attrs
            .iter()
            .filter(|a| !PLACE.contains(&a.name.as_str()))
            .cloned()
            .collect();
        let body: Vec<Node> = lines
            .into_iter()
            .filter(|(_, content)| {
                content
                    .iter()
                    .any(|n| !matches!(n, Node::Text(t, _) if t.trim().is_empty()))
            })
            .map(|(mut place, content)| {
                place.extend(ws.iter().cloned());
                Node::Element {
                    tag: "text".into(),
                    attrs: place,
                    children: content,
                    at,
                    end: at,
                    close: at,
                }
            })
            .collect();
        let g = self.element("g", &group, &body, at, parent)?;
        // Selected and named as the <text> it was written as.
        self.doc.nodes[g].selector_tag = Some("text".into());
        if let Some(entry) = self.selectors.iter_mut().find(|(n, ..)| *n == g) {
            entry.1 = "text".into();
        }
        Ok(Some(g))
    }

    /// A `<text>` holding a `<textPath>`: its attributes as the text's
    /// props, and its content as the text's. Other content: unchanged.
    pub(super) fn text_path(&mut self, i: usize, children: &[Node]) -> Result<Vec<Node>, Error> {
        let Some(Node::Element {
            attrs,
            children: inner,
            at,
            ..
        }) = children
            .iter()
            .find(|n| matches!(n, Node::Element { tag, .. } if tag == "textPath"))
        else {
            return Ok(children.to_vec());
        };
        if children.iter().any(|n| match n {
            Node::Element { tag, .. } => tag != "textPath",
            Node::Text(t, _) => !t.trim().is_empty(),
            _ => true,
        }) {
            return Err((
                *at,
                "a <textPath> is the whole content of its <text>".into(),
            ));
        }
        if let Some(n) = inner
            .iter()
            .find(|n| !matches!(n, Node::Text(..) | Node::Expression(..)))
        {
            let at = match n {
                Node::Element { at, .. } => *at,
                _ => *at,
            };
            return Err((
                at,
                "a <textPath> holds words and {expressions}: one run along its path".into(),
            ));
        }
        for a in attrs {
            let prop = match a.name.as_str() {
                "href" | "xlink:href" => "svgTextPathHref",
                "startOffset" => "svgStartOffset",
                "side" => "svgSide",
                "method" => "svgMethod",
                "spacing" => "svgSpacing",
                "path" => "svgPath",
                other => {
                    return Err((
                        a.at,
                        format!(
                            "<textPath> takes href, path, startOffset, side, method and spacing, not {other}; presentation attributes go on its <text>"
                        ),
                    ));
                }
            };
            if a.bound && matches!(prop, "svgTextPathHref" | "svgPath") {
                return Err((a.at, format!("{} names its path literally", a.name)));
            }
            let value = if a.bound {
                PropValue::Bind(expression(&a.value, a.at)?, None)
            } else {
                PropValue::Lit(Lit::Str(a.value.clone()))
            };
            self.prop(i, "svgText", prop, value, a.at, &a.value)?;
        }
        if !attrs
            .iter()
            .any(|a| matches!(a.name.as_str(), "href" | "xlink:href" | "path"))
        {
            return Err((
                *at,
                "a <textPath> names its path: href=\"#id\" or path=\"M…\"".into(),
            ));
        }
        Ok(inner.clone())
    }

    pub(super) fn text_runs(
        &mut self,
        i: usize,
        nodes: &[Node],
        link: Option<usize>,
        at: usize,
        translated: bool,
    ) -> Result<(), Error> {
        let mut runs = Vec::new();
        let mut links = Vec::new();
        let base = InlineRun {
            text: Expr::Lit("".into()),
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color: None,
            link,
            part: None,
            atom: None,
            valign: None,
        };
        self.runs(nodes, &base, &mut runs, &mut links, translated)?;
        // No space at either end, as a CSS block trims its line (an SVG
        // text keeping its white space trims its line breaks there too).
        let ends: &[char] = if self.white_space.is_some() {
            &[' ', '\n']
        } else {
            &[' ']
        };
        if let Some(InlineRun {
            text: Expr::Lit(serde_json::Value::String(t)),
            ..
        }) = runs.first_mut()
        {
            *t = t.trim_start_matches(ends).into();
        }
        if let Some(InlineRun {
            text: Expr::Lit(serde_json::Value::String(t)),
            ..
        }) = runs.last_mut()
        {
            t.truncate(t.trim_end_matches(ends).len());
        }
        runs.retain(|r| r.text != Expr::Lit("".into()));
        let bound = runs.iter().any(|r| !matches!(r.text, Expr::Lit(_)));
        let plain = runs.iter().fold(Expr::Lit("".into()), |acc, r| {
            Expr::Bin(
                crate::bind::Op::Add,
                Box::new(acc),
                Box::new(r.text.clone()),
            )
        });
        let value = if bound {
            PropValue::Bind(plain, None)
        } else {
            let text: String = runs
                .iter()
                .map(|r| match &r.text {
                    Expr::Lit(v) => crate::bind::text(v),
                    _ => String::new(),
                })
                .collect();
            PropValue::Lit(Lit::Str(text))
        };
        self.prop(i, "label", "text", value, at, "")?;
        if runs.iter().any(|r| {
            r.bold
                || r.italic
                || r.underline
                || r.strike
                || r.color.is_some()
                || r.link.is_some()
                || r.part.is_some()
                || r.atom.is_some()
                || r.valign.is_some()
        }) {
            self.doc.nodes[i].inline = runs;
            self.doc.nodes[i].links = links;
        }
        Ok(())
    }

    fn runs(
        &mut self,
        nodes: &[Node],
        style: &InlineRun,
        out: &mut Vec<InlineRun>,
        links: &mut Vec<String>,
        translated: bool,
    ) -> Result<(), Error> {
        let run = |text: Expr| InlineRun {
            text,
            ..style.clone()
        };
        for node in nodes {
            match node {
                Node::Text(value, at) => {
                    if !translated && !value.trim().is_empty() {
                        self.warn(*at);
                    }
                    let (spaces, breaks) = self.white_space.unwrap_or((false, false));
                    let mut collapsed = String::new();
                    for c in value.chars() {
                        if c == '\n' && breaks {
                            if !spaces {
                                // pre-line: no spaces around a line break.
                                collapsed.truncate(collapsed.trim_end_matches(' ').len());
                            }
                            collapsed.push('\n');
                        } else if c.is_ascii_whitespace() {
                            if spaces || !(collapsed.ends_with(' ') || collapsed.ends_with('\n')) {
                                collapsed.push(' ');
                            }
                        } else {
                            collapsed.push(c);
                        }
                    }
                    if !collapsed.is_empty() {
                        out.push(run(Expr::Lit(collapsed.into())));
                    }
                }
                Node::Expression(value, at) => out.push(run(expression(value, *at)?)),
                Node::Element {
                    tag,
                    attrs,
                    children,
                    at,
                    ..
                } if inline(node) => {
                    let mut inner = style.clone();
                    match tag.as_str() {
                        "b" | "strong" => inner.bold = true,
                        "i" | "em" => inner.italic = true,
                        "u" => inner.underline = true,
                        "s" | "del" => inner.strike = true,
                        "sub" => inner.valign = Some(crate::VerticalAlign::Sub),
                        "sup" => inner.valign = Some(crate::VerticalAlign::Super),
                        "span" => {
                            if let Some(a) = attrs.iter().find(|a| a.name == "part") {
                                if !super::syntax::identifier(&a.value) {
                                    return Err((a.at, "part names a span: an identifier".into()));
                                }
                                inner.part = Some(a.value.clone());
                            }
                        }
                        "br" => {
                            if !children.is_empty() {
                                return Err((*at, "<br> is empty".into()));
                            }
                            out.push(run(Expr::Lit("\n".into())));
                            continue;
                        }
                        "tspan" => {
                            for a in attrs {
                                super::extern_element::tspan(&mut inner, &a.name, &a.value)
                                    .map_err(|e| (a.at, e))?;
                            }
                        }
                        "a" => {
                            if style.link.is_some() {
                                return Err((*at, "a link cannot hold another link".into()));
                            }
                            let Some(href) = attrs.iter().find(|a| a.name == "href") else {
                                return Err((*at, "an inline <a> needs an href".into()));
                            };
                            links.push(href.value.clone());
                            inner.link = Some(links.len() - 1);
                        }
                        _ => {}
                    }
                    self.runs(children, &inner, out, links, translated)?;
                }
                // An inline object among the words: a picture, an icon, an
                // action's glyph, one placeholder character in the text.
                Node::Element {
                    tag,
                    attrs,
                    children,
                    at,
                    ..
                } if matches!(tag.as_str(), "img" | "og-icon" | "kbd")
                    && children.is_empty()
                    && !translated =>
                {
                    let literal = |names: &[&str]| {
                        attrs
                            .iter()
                            .find(|a| names.contains(&a.name.as_str()))
                            .filter(|a| !a.bound && !a.value.is_empty())
                            .map(|a| a.value.clone())
                    };
                    let atom = match tag.as_str() {
                        "img" => literal(&["src"]).map(crate::rich::InlineAtom::Image),
                        "og-icon" => literal(&["name", "icon"]).map(crate::rich::InlineAtom::Icon),
                        _ => literal(&["data-action"]).map(crate::rich::InlineAtom::Action),
                    }
                    .ok_or((
                        *at,
                        format!(
                            "an inline <{tag}> names its {} literally",
                            match tag.as_str() {
                                "img" => "picture with src",
                                "og-icon" => "icon with name",
                                _ => "action with data-action",
                            }
                        ),
                    ))?;
                    if let Some(a) = attrs.iter().find(|a| {
                        !matches!(
                            a.name.as_str(),
                            "src" | "alt" | "name" | "icon" | "data-action" | "class"
                        )
                    }) {
                        return Err((a.at, format!("an inline <{tag}> takes no {}", a.name)));
                    }
                    out.push(InlineRun {
                        text: Expr::Lit("\u{FFFC}".into()),
                        atom: Some(atom),
                        ..style.clone()
                    });
                }
                Node::Element { tag, at, .. } if tag == "tspan" => {
                    return Err((
                        *at,
                        "a <tspan> takes literal font-weight, font-style, text-decoration and fill, and starts a line with x (and y, dx, dy); its other attributes are not read".into(),
                    ));
                }
                Node::Element { tag, at, .. } if tag == "textPath" => {
                    return Err((
                        *at,
                        "a <textPath> is the whole content of its <text>".into(),
                    ));
                }
                Node::Element { tag, at, .. } if crate::svg::smil::TAGS.contains(&tag.as_str()) => {
                    return Err((
                        *at,
                        format!(
                            "<{tag}> animates a <text> from a <g> around it: the group's transform and presentation attributes reach its text"
                        ),
                    ));
                }
                Node::Element { tag, at, .. } => {
                    return Err((
                        *at,
                        format!(
                            "<{tag}> cannot sit inside text: text holds words, {{expressions}} and the inline elements b, strong, i, em, u, s, del, br, a plain <span> and <a href>; put <{tag}> beside the text in a <div>"
                        ),
                    ));
                }
                Node::If { at, .. }
                | Node::Each { at, .. }
                | Node::Snippet { at, .. }
                | Node::Render { at, .. }
                | Node::Key { at, .. }
                | Node::Const { at, .. }
                | Node::Await { at, .. }
                | Node::Script(_, at) => {
                    return Err((
                        *at,
                        "a block cannot sit inside text; put it beside the text in a <div>, or use {cond ? a : b}".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}
