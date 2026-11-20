use super::*;
use crate::doc::schema::Kind;

impl Parser {
    pub(super) fn element(
        &mut self,
        tag: &str,
        attrs: &[Attr],
        children: &[Node],
        at: usize,
        parent: usize,
    ) -> Result<usize, Error> {
        // An SVG <text>: its white-space read here, its lines split into
        // texts of their own.
        let kept;
        let mut white_space = None;
        self.white_space = None;
        let attrs = if tag == "text" && super::extern_element::in_svg(self.parent_tag(parent)) {
            let mut rest = Vec::new();
            for a in attrs {
                if a.name == "white-space" || a.name == "xml:space" {
                    if a.bound {
                        return Err((a.at, format!("{} is literal", a.name)));
                    }
                    white_space =
                        super::extern_element::white_space(&a.value).map_err(|e| (a.at, e))?;
                } else {
                    rest.push(a.clone());
                }
            }
            if let Some(i) = self.svg_lines(&rest, white_space, children, at, parent)? {
                return Ok(i);
            }
            // Read by its runs (element_node, below).
            self.white_space = white_space;
            kept = rest;
            &kept[..]
        } else {
            attrs
        };
        match tag {
            "svelte:element" => return self.dynamic_element(attrs, children, at, parent),
            "svelte:window" => {
                return Err((
                    at,
                    "<svelte:window> sits at the top of the file, beside the root element".into(),
                ));
            }
            "svelte:document" | "svelte:body" | "svelte:head" => {
                return Err((
                    at,
                    format!(
                        "<{tag}> has nothing to reach in a game: its UI is one layer stack; listen with <svelte:window onresize> or an input action (<meta name=\"action:…\">)"
                    ),
                ));
            }
            "svelte:boundary" => {
                return Err((
                    at,
                    "<svelte:boundary> catches errors a document never throws while it runs: its expressions are checked when it cooks, and a missing store fails the mount with its path".into(),
                ));
            }
            "svelte:options" | "svelte:fragment" | "svelte:self" | "svelte:component" => {
                return Err((
                    at,
                    format!(
                        "<{tag}> is not Svelte 5 markup a document takes; use a capitalised component, a snippet or <svelte:element this>"
                    ),
                ));
            }
            "option" | "optgroup" if self.select_contents.is_none() => {
                return Err((at, format!("<{tag}> belongs inside a <select>")));
            }
            // A customizable select's option (`appearance: base-select`):
            // a box of its own content. Its value, label and states are
            // the select's props; they stay readable to selectors.
            "option" => {
                let (own, kept): (Vec<&Attr>, Vec<Attr>) = (
                    attrs
                        .iter()
                        .filter(|a| OPTION_ATTRS.contains(&a.name.as_str()))
                        .collect(),
                    attrs
                        .iter()
                        .filter(|a| !OPTION_ATTRS.contains(&a.name.as_str()))
                        .cloned()
                        .collect(),
                );
                let i = self.element_node(tag, &kept, children, at, parent)?;
                // Pressed, it chooses itself: the select reads its press.
                if !own.iter().any(|a| a.name == "disabled") {
                    for name in ["interactive", "focusable"] {
                        self.prop(
                            i,
                            Some(Kind::Box),
                            name,
                            PropValue::Lit(Lit::Bool(true)),
                            at,
                            "",
                        )?;
                    }
                }
                for a in own {
                    let value = super::selectors::attribute_value(a)?;
                    self.doc.nodes[i]
                        .selector_attrs
                        .push((a.name.clone(), value));
                }
                return Ok(i);
            }
            "optgroup" => {
                let kept: Vec<Attr> = attrs
                    .iter()
                    .filter(|a| !matches!(a.name.as_str(), "label" | "disabled"))
                    .cloned()
                    .collect();
                let i = self.element_node(tag, &kept, children, at, parent)?;
                for a in attrs
                    .iter()
                    .filter(|a| matches!(a.name.as_str(), "label" | "disabled"))
                {
                    let value = super::selectors::attribute_value(a)?;
                    self.doc.nodes[i]
                        .selector_attrs
                        .push((a.name.clone(), value));
                }
                return Ok(i);
            }
            // The chosen option's content, in a customizable select's
            // `<button>`: a copy of each option's, of which the build shows
            // the chosen one's.
            "selectedcontent" => {
                let Some(contents) = self.select_contents.clone() else {
                    return Err((
                        at,
                        "<selectedcontent> belongs inside a <select>'s <button>".into(),
                    ));
                };
                if !children
                    .iter()
                    .all(|c| matches!(c, Node::Text(t, _) if t.trim().is_empty()))
                {
                    return Err((
                        at,
                        "<selectedcontent> is empty: it shows the chosen option's content".into(),
                    ));
                }
                let copies: Vec<Node> = contents
                    .into_iter()
                    .map(|content| Node::Element {
                        tag: "div".into(),
                        attrs: vec![Attr {
                            name: "style".into(),
                            value: "display: contents".into(),
                            bound: false,
                            at,
                            end: at,
                        }],
                        children: without_ids(content),
                        at,
                        end: at,
                        close: at,
                    })
                    .collect();
                return self.element_node(tag, attrs, &copies, at, parent);
            }
            "summary" if self.parent_tag(parent) != Some("details") => {
                return Err((at, "<summary> is the first child of a <details>".into()));
            }
            "datalist" => {
                // HTML renders none: its options are the values a range's
                // `list` marks, or a text input's suggestions.
                let Some(id) = attrs.iter().find(|a| a.name == "id" && !a.bound) else {
                    return Err((
                        at,
                        "a <datalist> needs a literal id, which an input's list names".into(),
                    ));
                };
                if let Some(a) = attrs
                    .iter()
                    .find(|a| a.name != "id" && !(a.name == "items" && a.bound))
                {
                    return Err((
                        a.at,
                        format!(
                            "a <datalist> takes only an id and items={{$names}}, not {}",
                            a.name
                        ),
                    ));
                }
                let items = attrs
                    .iter()
                    .find(|a| a.name == "items")
                    .map(|a| expression(&a.value, a.at))
                    .transpose()?;
                if !self.ids.insert(id.value.clone()) {
                    return Err((
                        id.at,
                        format!(
                            "duplicate id '{}'; IDs must be unique within a document",
                            id.value
                        ),
                    ));
                }
                let options = select_options(children)?;
                self.datalists.push((id.value.clone(), options, items));
                return Ok(self.push(NodeKind::If(Expr::Lit(false.into())), parent, at));
            }
            "select" => {
                let options = select_options(children)?;
                // Its own children, which a customizable select builds
                // (`appearance: base-select`) and a native one does not.
                let outer = self.select_contents.replace(option_contents(children));
                let i = self.element_node(tag, attrs, children, at, parent);
                self.select_contents = outer;
                let i = i?;
                let strs = |v: Vec<String>| PropValue::Lit(Lit::Strs(v));
                let items = options.iter().map(|o| o.label.clone()).collect();
                let values = options.iter().map(|o| o.value.clone()).collect();
                // Without `<option>`s its options are its bound `items`.
                if !options.is_empty() {
                    self.prop(i, Some(Kind::Select), "items", strs(items), at, "")?;
                    self.prop(i, Some(Kind::Select), "values", strs(values), at, "")?;
                }
                if options.iter().any(|o| !o.group.is_empty()) {
                    let groups = options.iter().map(|o| o.group.clone()).collect();
                    self.prop(i, Some(Kind::Select), "groups", strs(groups), at, "")?;
                }
                if options.iter().any(|o| o.key.is_some()) {
                    let keys = options
                        .iter()
                        .map(|o| o.key.clone().unwrap_or_default())
                        .collect();
                    self.prop(i, Some(Kind::Select), "keys", strs(keys), at, "")?;
                }
                let off: Vec<String> = (0..options.len())
                    .filter(|&k| options[k].disabled)
                    .map(|k| k.to_string())
                    .collect();
                if !off.is_empty() {
                    self.prop(i, Some(Kind::Select), "disabledItems", strs(off), at, "")?;
                }
                // `<option selected>` is the value when nothing is bound.
                let valued = attrs
                    .iter()
                    .any(|a| matches!(a.name.as_str(), "value" | "bind:value"));
                if !valued && let Some(o) = options.iter().find(|o| o.selected) {
                    self.prop(
                        i,
                        Some(Kind::Select),
                        "current",
                        PropValue::Lit(Lit::Str(o.value.clone())),
                        at,
                        "",
                    )?;
                }
                return Ok(i);
            }
            "details" => {
                let mut rest = Vec::new();
                let mut summary = None;
                for child in children {
                    match child {
                        Node::Element {
                            tag: t,
                            children: inner,
                            at,
                            ..
                        } if t == "summary" => {
                            if summary.is_some()
                                || rest.iter().any(|c| matches!(c, Node::Element { .. }))
                            {
                                return Err((
                                    *at,
                                    "a <details> takes one <summary>, as its first child".into(),
                                ));
                            }
                            // Its text names the disclosure's button.
                            summary = Some((text_value(inner, *at).ok().flatten(), *at));
                            rest.insert(0, child.clone());
                        }
                        other => rest.push(other.clone()),
                    }
                }
                let i = self.element_node(tag, attrs, &rest, at, parent)?;
                if let Some((Some((value, source)), at)) = summary {
                    self.prop(i, Some(Kind::Details), "summary", value, at, &source)?;
                }
                return Ok(i);
            }
            _ => {}
        }
        self.element_node(tag, attrs, children, at, parent)
    }
    pub(super) fn element_node(
        &mut self,
        tag: &str,
        attrs: &[Attr],
        children: &[Node],
        at: usize,
        parent: usize,
    ) -> Result<usize, Error> {
        // An inline element of a text (`runs`): its attributes read as any
        // element's, its content its text's runs.
        let inline_el = std::mem::take(&mut self.inline_element);
        // A label around a checkbox is the checkbox, its text the caption
        // (HTML's implicit label association).
        if tag == "label" && !inline_el {
            // A `<small>` in it is the checkbox's description line.
            let small = |c: &Node| matches!(c, Node::Element { tag, .. } if tag == "small");
            let controls: Vec<&Node> = children
                .iter()
                .filter(|c| matches!(c, Node::Element { .. }) && !small(c))
                .collect();
            if let [
                Node::Element {
                    tag: inner,
                    attrs: inner_attrs,
                    children: inner_children,
                    at: inner_at,
                    ..
                },
            ] = controls[..]
                && inner == "input"
                && inner_children.is_empty()
                && inner_attrs.iter().any(|a| {
                    a.name == "type" && !a.bound && matches!(a.value.as_str(), "checkbox" | "radio")
                })
            {
                let control = if inner_attrs
                    .iter()
                    .any(|a| a.name == "type" && a.value == "radio")
                {
                    Kind::Radio
                } else {
                    Kind::Toggle
                };
                let caption: Vec<Node> = children
                    .iter()
                    .filter(|c| !matches!(c, Node::Element { .. }))
                    .cloned()
                    .collect();
                // HTML lays the two out in source order: a caption written
                // after the checkbox sits after it.
                let input_at = children
                    .iter()
                    .position(|c| matches!(c, Node::Element { tag, .. } if tag == "input"));
                let text_at = children.iter().position(|c| match c {
                    Node::Text(t, _) => !t.trim().is_empty(),
                    Node::Expression(..) => true,
                    _ => false,
                });
                let after = matches!((input_at, text_at), (Some(i), Some(t)) if i < t);
                let mut description = None;
                for c in children.iter().filter(|c| small(c)) {
                    let Node::Element {
                        attrs: small_attrs,
                        children: inner,
                        at: small_at,
                        ..
                    } = c
                    else {
                        continue;
                    };
                    if description.is_some() || !small_attrs.is_empty() {
                        return Err((
                            *small_at,
                            "a checkbox's <label> takes one plain <small>, its description".into(),
                        ));
                    }
                    description = text_value(inner, *small_at)?.map(|d| (d, *small_at));
                }
                // The two are one node: the label's attributes (an id, a
                // class) are the checkbox's, and one set on both is refused.
                let mut merged = inner_attrs.clone();
                for a in attrs {
                    if inner_attrs.iter().any(|b| b.name == a.name) {
                        return Err((
                            a.at,
                            format!(
                                "{} is set on both the <label> and its checkbox; they are one control, set it once",
                                a.name
                            ),
                        ));
                    }
                    merged.push(a.clone());
                }
                let i = self.element("input", &merged, &caption, *inner_at, parent)?;
                if after {
                    self.prop(
                        i,
                        Some(control),
                        "side",
                        PropValue::Lit(Lit::Str("end".into())),
                        *inner_at,
                        "",
                    )?;
                }
                if let Some(((value, source), at)) = description {
                    self.prop(i, Some(control), "description", value, at, &source)?;
                }
                return Ok(i);
            }
        }
        let label_for = match attrs.iter().find(|a| a.name == "for") {
            Some(a) if tag == "label" => {
                if a.bound || a.value.is_empty() {
                    return Err((a.at, "for names a control by its literal id".into()));
                }
                Some(a)
            }
            _ => None,
        };
        if tag == "li" && !self.in_list(parent) {
            return Err((at, "<li> belongs inside a <ul> or an <ol>".into()));
        }
        let html = match inline_el {
            true => None,
            false => self.html_kind(tag, parent, at)?,
        };
        // The element's kind: none for a component.
        let internal = match tag {
            _ if inline_el => Some(Kind::Inline),
            _ if let Some((_, kind)) =
                schema::ELEMENT_TAGS.iter().find(|(name, _)| *name == tag) =>
            {
                Some(*kind)
            }
            _ if let Some((kind, _)) = html => Some(kind),
            // SVG's text: placed in its view box (extern_element).
            "text" if super::extern_element::in_svg(self.parent_tag(parent)) => Some(Kind::SvgText),
            "text" | "p" | "span" | "small" | "a" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                Some(Kind::Label)
            }
            "label" if label_for.is_some() => Some(Kind::Label),
            // A customizable select's parts: its trigger is the select's
            // own press, so its `<button>` is a box.
            "button" if self.parent_tag(parent) == Some("select") => Some(Kind::Box),
            "option" | "optgroup" | "selectedcontent" if self.select_contents.is_some() => {
                Some(Kind::Box)
            }
            "div" | "section" | "header" | "footer" | "nav" | "main" | "aside" | "article"
            | "ul" | "ol" | "li" | "menu" | "summary" | "iframe" => Some(Kind::Box),
            "img" => Some(Kind::Image),
            "video" => Some(Kind::Video),
            "audio" => Some(Kind::Audio),
            "hr" => Some(Kind::Divider),
            "textarea" => Some(Kind::Field),
            // An input's kind is its literal type's: a range is a slider, a
            // checkbox a toggle, every text type a field.
            "input" => Some(
                match attrs
                    .iter()
                    .find(|a| a.name == "type" && !a.bound)
                    .map(|a| a.value.as_str())
                {
                    Some("range") => Kind::Slider,
                    Some("checkbox") => Kind::Toggle,
                    Some("radio") => Kind::Radio,
                    Some("color") => Kind::Color,
                    Some("date" | "time" | "datetime-local" | "month" | "week") => Kind::Datetime,
                    Some("file") => Kind::File,
                    _ => Kind::Field,
                },
            ),
            other => match super::extern_element::kind(other, self.parent_tag(parent)) {
                Some(Ok(kind)) => Some(kind),
                Some(Err(hint)) => return Err((at, hint)),
                None => match renamed(other) {
                    Some(hint) => return Err((at, hint)),
                    // A kind written by its own name; an inline element is
                    // written by its HTML tag inside a text.
                    None => match Kind::named(other) {
                        Some(Kind::Inline) => {
                            return Err((
                                at,
                                "<inline> is no tag: write <span> (or <b>, <a>, <ruby>…) inside a text".into(),
                            ));
                        }
                        kind => kind,
                    },
                },
            },
        };
        // An explicit component name cannot collide with a native tag
        // (`ui:table` is the component, `table` the HTML element).
        let component_name = tag
            .strip_prefix("ui:")
            .filter(|name| !name.is_empty())
            .or_else(|| tag.starts_with(char::is_uppercase).then_some(tag));
        let component = component_name.is_some();
        let kind = match internal {
            _ if let Some(name) = component_name => NodeKind::Use(name.into()),
            Some(kind) => NodeKind::Element(kind),
            None => {
                return Err((
                    at,
                    format!(
                        "unknown element <{tag}>; use a reflected core tag, capitalised component or <ui:name> with the component's declared name"
                    ),
                ));
            }
        };
        // An `<iframe>` shows its `src` document; HTML's fallback content
        // between its tags has no place here.
        let frame = tag == "iframe";
        if frame
            && children
                .iter()
                .any(|c| !matches!(c, Node::Text(text, _) if text.trim().is_empty()))
        {
            return Err((
                at,
                "an <iframe> shows the document its src names; write nothing between its tags"
                    .into(),
            ));
        }
        let i = self.push(kind, parent, at);
        self.doc.nodes[i].selector_tag = Some(tag.into());
        if let Some(a) = label_for {
            self.label_fors.push((i, a.value.clone(), a.at));
        }
        let named = attrs
            .iter()
            .any(|a| matches!(a.name.as_str(), "aria-label" | "name"));
        // HTML's implicit roles (landmarks, lists); a section is a region
        // only when named.
        let in_menu = tag == "button" && self.parent_tag(parent) == Some("menu");
        self.doc.nodes[i].role = match tag {
            "menu" => Some("menu"),
            "button" if in_menu => Some("menuitem"),
            "nav" => Some("navigation"),
            "main" => Some("main"),
            "header" => Some("banner"),
            "footer" => Some("contentinfo"),
            "aside" => Some("complementary"),
            "article" => Some("article"),
            "section" if named => Some("region"),
            "ul" | "ol" => Some("list"),
            "option" => Some("option"),
            "optgroup" => Some("group"),
            "li" => Some("listitem"),
            _ => html.and_then(|(_, role)| role),
        };
        for attr in attrs
            .iter()
            .filter(|a| !a.name.starts_with("on") && !directive_name(&a.name))
        {
            let value = super::selectors::attribute_value(attr)?;
            self.doc.nodes[i]
                .selector_attrs
                .push((attr.name.clone(), value));
        }
        self.selectors.push((i, tag.into(), attrs.to_vec()));
        let smil = crate::svg::smil::TAGS.contains(&tag);
        if smil || tag == "mpath" {
            if let Some(a) = attrs.iter().find(|a| a.bound) {
                return Err((
                    a.at,
                    format!(
                        "<{tag}>'s attributes are literal: it plays on the UI clock; animate bound values with og.tween or a CSS transition"
                    ),
                ));
            }
            if let Some(a) = attrs
                .iter()
                .find(|a| tag == "mpath" && a.name != "href" && a.name != "id")
            {
                return Err((a.at, format!("<mpath> takes href=\"#id\", not {}", a.name)));
            }
            if smil {
                let literal: Vec<(String, String)> = attrs
                    .iter()
                    .map(|a| (a.name.clone(), a.value.clone()))
                    .collect();
                // An `<mpath>` child names the path later: stand in for it.
                let mpath = (tag == "animateMotion").then(og_draw::kurbo::BezPath::new);
                crate::svg::smil::Animation::parse(tag, &literal, mpath).map_err(|e| (at, e))?;
            }
        }
        // A remote `<iframe>`'s `integrity`, as url()'s modifier. HTML's
        // `crossorigin` and `referrerpolicy` are refused, as url()'s
        // cross-origin() and referrer-policy() are (css::request).
        let mut request = Vec::new();
        for a in attrs {
            if frame && matches!(a.name.as_str(), "crossorigin" | "referrerpolicy") {
                return Err((
                    a.at,
                    format!(
                        "an <iframe>'s {} is not needed: the engine fetches without cookies, credentials or a referrer of its own; only integrity applies",
                        a.name
                    ),
                ));
            }
            if frame && a.name == "integrity" {
                if a.bound {
                    return Err((a.at, "an <iframe>'s integrity is a literal".into()));
                }
                request.push(("integrity".to_string(), a.value.clone(), a.at));
                continue;
            }
            // `exportparts` (CSS Shadow Parts §4.2): a control written in a
            // control document makes parts of its own inside parts of that
            // document's host too. Read where parts are matched.
            if a.name == "exportparts" {
                // A component is styled through what it declares, never
                // from outside by selector (docs/ui.md §3.2).
                if component {
                    return Err((
                        a.at,
                        format!(
                            "<{tag}> is a component instance: a component exports no part; pass it a prop or set a custom property on it"
                        ),
                    ));
                }
                if a.bound {
                    return Err((
                        a.at,
                        "exportparts is a literal: exportparts=\"fill, track: bar-track\"".into(),
                    ));
                }
                super::super::part_mappings(&a.value).map_err(|e| (a.at, e))?;
                self.exportparts.get_or_insert(a.at);
                continue;
            }
            // `pseudo` (docs/ui.md §2.4): the element of a control document
            // that is a standard pseudo-element of its host, which the
            // host's document styles as `details::details-content`. Its
            // names are apart from `part`'s. Read where parts are matched.
            if a.name == "pseudo" {
                if component {
                    return Err((
                        a.at,
                        format!(
                            "<{tag}> is a component instance: wrap it in the element that is the pseudo-element"
                        ),
                    ));
                }
                let pseudo = super::super::selectors::PseudoElement::named(&a.value)
                    .filter(|_| !a.bound)
                    .ok_or_else(|| {
                        (
                            a.at,
                            format!(
                                "pseudo is a literal standard pseudo-element name: {}, not {}",
                                super::super::selectors::PseudoElement::ALL
                                    .map(|p| p.name())
                                    .join(", "),
                                if a.bound {
                                    "a binding"
                                } else {
                                    a.value.as_str()
                                }
                            ),
                        )
                    })?;
                self.pseudo.push((pseudo, a.at));
                continue;
            }
            if a.name == "id" {
                if a.bound || a.value.is_empty() || a.value.contains(['/', '#']) {
                    return Err((
                        a.at,
                        "id is a nonempty literal without '/' or '#'; bind data, not identity"
                            .into(),
                    ));
                }
                if !self.ids.insert(a.value.clone()) {
                    return Err((
                        a.at,
                        format!(
                            "duplicate id '{}'; IDs must be unique within a document",
                            a.value
                        ),
                    ));
                }
                self.doc.nodes[i].id = a.value.clone();
                continue;
            }
            if frame && a.name == "src" {
                if a.bound {
                    return Err((
                        a.at,
                        "an <iframe>'s src is a literal path to a document of the game".into(),
                    ));
                }
                // In a remote document a relative src is beside it.
                let remote = crate::css::request::is_remote(&self.doc.source);
                if remote || crate::css::request::is_remote(&a.value) {
                    #[cfg(not(feature = "frames"))]
                    return Err((a.at, crate::doc::frames::ABSENT.into()));
                    #[cfg(feature = "frames")]
                    {
                        let url = crate::css::request::join(&self.doc.source, &a.value);
                        if !crate::css::request::secure(&url) {
                            return Err((
                                a.at,
                                "a remote <iframe> loads over https:// (http:// only from localhost)"
                                    .into(),
                            ));
                        }
                        self.doc.nodes[i].frame = Some(url);
                        continue;
                    }
                }
                if a.value.contains(':') {
                    return Err((
                        a.at,
                        "an <iframe>'s src is a document of the game (src=\"news.svelte\") or an https:// URL".into(),
                    ));
                }
                let id = super::relative_id(&self.doc.source, &a.value)
                    .filter(|id| id.ends_with(".svelte"))
                    .ok_or((
                        a.at,
                        "an <iframe>'s src names a .svelte document within the game's asset directory"
                            .into(),
                    ))?;
                self.doc.nodes[i].frame = Some(id);
                continue;
            }
            // An animation's attributes are read as it plays (svg::smil).
            if smil || tag == "mpath" {
                continue;
            }
            // Read by its text's runs: its part, a link's href, a
            // `<tspan>`'s presentation attributes.
            if inline_el
                && (a.name == "part"
                    || (tag == "a" && a.name == "href")
                    || (tag == "tspan"
                        && super::extern_element::TSPAN_ATTRS.contains(&a.name.as_str())))
            {
                continue;
            }
            if a.name == "class" || a.name.starts_with("class:") {
                let bound = match a.bound {
                    true => Some(expression(&a.value, a.at)?),
                    false => None,
                };
                // An instance's class is the component's `class` prop,
                // which its `:host(.name)` rules read.
                if component && a.name == "class" {
                    let value = match bound {
                        Some(e) => PropValue::Bind(e, None),
                        None => PropValue::Lit(Lit::Str(a.value.clone())),
                    };
                    let line = self.line(a.at);
                    self.doc.nodes[i].args.push(("class".into(), value, line));
                }
                continue;
            }
            if self.directive(i, internal, a)? || self.html_attr(i, tag, a)? {
                continue;
            }
            if let Some(event) = a.name.strip_prefix("on") {
                let when = match event {
                    "click" => When::Click,
                    "change" | "input" => When::Changed,
                    "tap" => When::Tap,
                    "doubletap" => When::DoubleTap,
                    "longpress" => When::LongPress,
                    other if let Some((_, w)) = When::EVENTS.iter().find(|(e, _)| *e == other) => {
                        w.clone()
                    }
                    action if self.doc.actions.iter().any(|(name, _)| name == action) => {
                        When::Action(action.into())
                    }
                    _ => {
                        return Err((
                            a.at,
                            format!(
                                "unknown event {event}; use onclick, onchange, ontap, ondragstart, ondrop, onpointerdown, onwheel…, or declare an input action with <meta name=\"action:{event}\" content=\"Label\">"
                            ),
                        ));
                    }
                };
                if !a.bound || !identifier(&a.value) {
                    return Err((
                        a.at,
                        "event handlers name a mounted action: onclick={next}; no executable calls"
                            .into(),
                    ));
                }
                let line = self.line(a.at);
                self.doc.nodes[i].triggers.push(Trigger {
                    when,
                    act: Act::Emit(a.value.clone()),
                    line,
                });
                continue;
            }
            // A custom property: the data a rule's var() reads. An element
            // sets its own with `style:--name`; an instance hands its root
            // one to inherit with `--name` (Svelte's style props).
            let custom = match a.name.strip_prefix("style:") {
                Some(name) if name.starts_with("--") => {
                    if component {
                        return Err((
                            a.at,
                            format!(
                                "style: is an element's directive; on a component write {name}=… (a style prop)"
                            ),
                        ));
                    }
                    Some(name)
                }
                None if a.name.starts_with("--") => {
                    if !component {
                        return Err((
                            a.at,
                            format!(
                                "{0} is a component's style prop; on an element write style:{0}=…",
                                a.name
                            ),
                        ));
                    }
                    Some(a.name.as_str())
                }
                _ => None,
            };
            if let Some(name) = custom {
                if name.len() < 3
                    || !name[2..]
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
                {
                    return Err((a.at, format!("{name} is not a custom property name")));
                }
                let value = if a.bound {
                    expression(&a.value, a.at)?
                } else {
                    Expr::Lit(serde_json::Value::String(a.value.clone()))
                };
                self.doc.nodes[i].custom.push((name.to_string(), value));
                continue;
            }
            // A block's key translates its anonymous text.
            if internal == Some(Kind::Box) && a.name == "i18n" {
                continue;
            }
            // Read as selector facts: a label's control, a list's numbering.
            if (label_for.is_some() && a.name == "for")
                || (tag == "ol" && matches!(a.name.as_str(), "start" | "reversed"))
                || (tag == "li" && a.name == "value")
            {
                if a.name != "for"
                    && a.name != "reversed"
                    && !a.bound
                    && a.value.trim().parse::<i64>().is_err()
                {
                    return Err((a.at, format!("{} takes an integer", a.name)));
                }
                continue;
            }
            // ARIA naming by reference: the named elements' text, joined
            // once the tree is read.
            if let Some(prop) = match a.name.as_str() {
                "aria-labelledby" => Some("name"),
                "aria-describedby" => Some("ariaDescription"),
                _ => None,
            } {
                if a.bound || a.value.trim().is_empty() {
                    return Err((
                        a.at,
                        format!("{} names elements by their literal ids", a.name),
                    ));
                }
                self.aria_refs.push((i, prop, a.value.clone(), a.at));
                continue;
            }
            if a.name == "role" && !a.bound && !crate::a11y::ROLES.contains(&a.value.as_str()) {
                return Err((
                    a.at,
                    format!(
                        "role \"{}\" is not an accessibility role; take one of {}",
                        a.value,
                        crate::a11y::ROLES.join(", ")
                    ),
                ));
            }
            // A range's ticks, or a text input's suggestions: the
            // `<datalist>` it names, joined once the tree is read.
            if tag == "input" && a.name == "list" {
                if internal != Some(Kind::Slider) && internal != Some(Kind::Field) {
                    return Err((
                        a.at,
                        "list names a range's ticks or a text input's suggestions".into(),
                    ));
                }
                if a.bound || a.value.is_empty() {
                    return Err((a.at, "list names a <datalist> by its literal id".into()));
                }
                self.list_fors.push((i, a.value.clone(), a.at));
                continue;
            }
            // A bare `popover` is HTML's auto.
            if a.name == "popover" && !a.bound && matches!(a.value.as_str(), "" | "true") {
                self.prop(
                    i,
                    internal,
                    "popover",
                    PropValue::Lit(Lit::Str("auto".into())),
                    a.at,
                    "auto",
                )?;
                continue;
            }
            // An input's type chose its kind; a checkbox's `switch` is how
            // it draws, and a toggle is a switch.
            if tag == "input" && internal == Some(Kind::Toggle) && a.name == "switch" && !a.bound {
                self.prop(
                    i,
                    internal,
                    "switch",
                    PropValue::Lit(Lit::Bool(true)),
                    a.at,
                    "",
                )?;
                continue;
            }
            if tag == "input"
                && !matches!(internal, Some(Kind::Field | Kind::Datetime))
                && a.name == "type"
            {
                continue;
            }
            // A radio's group is the store it binds; its `name` adds nothing.
            if internal == Some(Kind::Radio) && a.name == "name" {
                continue;
            }
            if a.name == "multiline" || (tag == "textarea" && a.name == "type") {
                return Err((
                    a.at,
                    "write several lines as <textarea>, one line as <input type=…>".into(),
                ));
            }
            if internal == Some(Kind::Field)
                && a.name == "pattern"
                && !a.bound
                && crate::input_type::pattern(&a.value).is_none()
            {
                return Err((
                    a.at,
                    format!("pattern \"{}\" is not a regular expression", a.value),
                ));
            }
            if internal == Some(Kind::Field)
                && a.name == "data-format"
                && !a.bound
                && let Err(e) = og_i18n::Message::parse(&a.value)
            {
                return Err((
                    a.at,
                    format!("data-format \"{}\" is not a message: {e}", a.value),
                ));
            }
            if internal == Some(Kind::Field)
                && let Some(html) = match a.name.as_str() {
                    "max-chars" | "maxChars" => Some("maxlength"),
                    "min-chars" | "minChars" => Some("minlength"),
                    "lines" => Some("rows on a <textarea>"),
                    _ => None,
                }
            {
                return Err((
                    a.at,
                    format!("{} is not a field attribute; write {html}", a.name),
                ));
            }
            // HTML's custom data attributes (but the engine's reflected
            // `data-og-*` ones): read by selectors and attr().
            if a.name.starts_with("data-") && schema::prop(&property(&a.name)).is_none() {
                continue;
            }
            let model = a.name.strip_prefix("bind:");
            let mapped = schema::ELEMENT_ATTRIBUTES
                .iter()
                .find(|(owner, attribute, _)| {
                    *owner == tag && *attribute == model.unwrap_or(&a.name)
                });
            let mut name = match (internal, model.unwrap_or(&a.name)) {
                (_, name) if component => name.into(),
                _ if let Some((_, _, property)) = mapped => (*property).into(),
                (Some(Kind::Field), "maxlength") => "maxChars".into(),
                (Some(Kind::Field), "minlength") => "minChars".into(),
                (Some(Kind::Field), "rows") if tag == "textarea" => "lines".into(),
                (Some(Kind::Details), "name") => "exclusive".into(),
                (_, other) => internal
                    .and_then(|kind| super::extern_element::property(kind, other))
                    .unwrap_or_else(|| property(other)),
            };
            if matches!(name.as_str(), "suggestionLabels" | "suggestionKeys") {
                return Err((
                    a.at,
                    "write label or i18n on the datalist's <option>, not on the input".into(),
                ));
            }
            // A component's props are its own names: `count` on a component
            // is its prop, not <og-counter>'s value.
            if mapped.is_none()
                && !component
                && let Some((owner, attribute, _)) = schema::ELEMENT_ATTRIBUTES
                    .iter()
                    .find(|(_, _, property)| *property == name)
            {
                return Err((
                    a.at,
                    format!(
                        "{} is not an attribute of <{tag}>; write <{owner} {attribute}=…>",
                        a.name
                    ),
                ));
            }
            if name == "value" && internal == Some(Kind::Field) {
                name = "input".into();
            }
            if name == "value" && internal == Some(Kind::Select) {
                name = "current".into();
            }
            if name == "value" && internal == Some(Kind::Color) {
                name = "colorValue".into();
            }
            if matches!(internal, Some(Kind::Video | Kind::Audio)) && a.name == "src" {
                if !a.bound {
                    return Err((
                        a.at,
                        format!(
                            "<{tag}> plays a player the game owns, not a file: spawn an og.{} with the asset in a system and bind its entity, src={{$player}}",
                            if tag == "video" {
                                "VideoPlayer"
                            } else {
                                "AudioSource"
                            }
                        ),
                    ));
                }
                name = "mediaSrc".into();
            }
            if internal == Some(Kind::Audio)
                && matches!(
                    name.as_str(),
                    "currentTime" | "duration" | "ended" | "poster"
                )
            {
                return Err((
                    a.at,
                    format!(
                        "<audio> has no {name}: an og.AudioSource's playhead is not read; use <video> for footage"
                    ),
                ));
            }
            if internal == Some(Kind::File) && name == "value" {
                name = "fileValue".into();
            }
            if internal == Some(Kind::Datetime) {
                match name.as_str() {
                    "value" => name = "dateValue".into(),
                    "type" => name = "dateType".into(),
                    "min" => name = "dateMin".into(),
                    "max" => name = "dateMax".into(),
                    _ => {}
                }
            }
            if name == "value" && internal == Some(Kind::Radio) {
                name = "choice".into();
            }
            if name == "value" && internal == Some(Kind::Kbd) {
                name = "binding".into();
            }
            if a.name == "hidden" && !component {
                name = "visible".into();
            }
            // Selector facts only: `:lang()`, `:dir()`, `[data-…]` and
            // `:state(name)` (CSS custom states, `state:name={…}`). `lang`
            // and `dir` are HTML's global attributes: an element that does
            // not reflect them itself (a field's `lang` picks its keyboard)
            // still has them for its selectors. The new core's layout fields
            // (`layout`, `gap`, `anchor`, `fill`, `reverse`, `play`) and
            // surface names (`variant`, `tier`, `tone`, `shape`, `team`, `trend`) are read from the same facts by og-ui-next;
            // on SVG `fill` is already `svgFill` here.
            let reflected = schema::prop(&name)
                .is_some_and(|(_, p)| internal.is_none_or(|kind| p.applies(kind)));
            let field = matches!(
                name.as_str(),
                "layout"
                    | "gap"
                    | "anchor"
                    | "fill"
                    | "reverse"
                    | "play"
                    | "variant"
                    | "tier"
                    | "tone"
                    | "shape"
            );
            if !component
                && model.is_none()
                && (field
                    || !reflected && matches!(a.name.as_str(), "lang" | "dir")
                    || !reflected
                        && (schema::prop(&name).is_none()
                            && (a.name.starts_with("data-")
                                || a.name.strip_prefix("state:").is_some_and(identifier))))
            {
                if a.bound {
                    expression(&a.value, a.at)?;
                }
                continue;
            }
            // An `<iframe>`'s other attributes are its document's props,
            // read here as a component's are: the only data it is given.
            if frame && !reflected && model.is_none() {
                let value = if a.bound {
                    PropValue::Bind(expression(&a.value, a.at)?, None)
                } else {
                    PropValue::Lit(Lit::Str(a.value.clone()))
                };
                let line = self.line(a.at);
                self.doc.nodes[i].args.push((name, value, line));
                continue;
            }
            let value = if model.is_some() {
                let expr = expression(&a.value, a.at)?;
                let Expr::Path(path) = expr else {
                    return Err((
                        a.at,
                        "bind: needs a writable store or row-field path".into(),
                    ));
                };
                if !a.bound {
                    return Err((a.at, "write bind:value={$store}, not a literal".into()));
                }
                PropValue::Model(path)
            } else if a.bound {
                PropValue::Bind(expression(&a.value, a.at)?, None)
            } else if component {
                // The callee owns this literal's type, not the native
                // attribute schema (for example Inbox's string selected).
                PropValue::Lit(Lit::Str(a.value.clone()))
            } else if name == "stepBy" && a.value == "any" {
                // HTML's step="any": no step constraint.
                PropValue::Lit(Lit::Num(0.0))
            } else if let Some((_, spec)) = schema::prop(&name) {
                literal(spec.ty, &a.value).map_err(|why| (a.at, why))?
            } else {
                return Err((
                    a.at,
                    format!("unknown attribute {}; use a reflected property", a.name),
                ));
            };
            let value = if a.name == "hidden" && !component {
                match value {
                    PropValue::Bind(e, _) => PropValue::Bind(Expr::Not(Box::new(e)), None),
                    PropValue::Lit(Lit::Bool(b)) => PropValue::Lit(Lit::Bool(!b)),
                    _ => return Err((a.at, "hidden takes a boolean".into())),
                }
            } else {
                value
            };
            if matches!(name.as_str(), "placeholder" | "name") && !a.bound && !a.value.is_empty() {
                self.warn(a.at);
            }
            if component {
                let line = self.line(a.at);
                self.doc.nodes[i].args.push((name, value, line));
            } else {
                self.prop(i, internal, &name, value, a.at, &a.value)?;
            }
        }
        if frame && self.doc.nodes[i].frame.is_none() {
            return Err((
                at,
                "an <iframe> needs src naming a document of the game".into(),
            ));
        }
        // A remote frame is named by its request key, as a remote url()
        // image is; its document is given data, never a writable path.
        #[cfg(feature = "frames")]
        let remote = self.doc.nodes[i]
            .frame
            .clone()
            .filter(|url| crate::css::request::is_remote(url));
        #[cfg(not(feature = "frames"))]
        let remote: Option<String> = None;
        if let Some(url) = remote {
            let modifiers: Vec<_> = request
                .iter()
                .map(|(n, v, _)| (n.clone(), v.clone()))
                .collect();
            let key = crate::css::request::UrlRequest::new(&url, &modifiers)
                .map_err(|e| (at, e))?
                .key();
            self.doc.nodes[i].frame = Some(key);
            if let Some(a) = attrs.iter().find(|a| a.name.starts_with("bind:")) {
                return Err((
                    a.at,
                    "a remote <iframe> is given data, not a binding: pass the value, and read what it sends back from og.ui.frameMessages".into(),
                ));
            }
        } else if let Some((name, _, at)) = request.first() {
            return Err((
                *at,
                format!(
                    "{name} applies to a remote <iframe> (an https:// src); the game's own document is not fetched"
                ),
            ));
        }
        if inline_el {
            return Ok(i);
        }
        // An action's glyph follows the device: it reads `$ui.device`.
        if internal == Some(Kind::Kbd) && attrs.iter().any(|a| a.name == "data-action") {
            let device = expression("$ui.device", at)?;
            self.prop(
                i,
                internal,
                "glyphDevice",
                PropValue::Bind(device, None),
                at,
                "",
            )?;
        }
        if tag == "textarea" {
            self.prop(
                i,
                internal,
                "multiline",
                PropValue::Lit(Lit::Bool(true)),
                at,
                "",
            )?;
        }
        if let Some(level) = tag
            .strip_prefix('h')
            .and_then(|l| l.parse::<u8>().ok())
            .filter(|l| (1..=6).contains(l))
        {
            self.prop(
                i,
                internal,
                "heading",
                PropValue::Lit(Lit::Bool(true)),
                at,
                "",
            )?;
            self.doc.nodes[i].level = level;
        }
        let translated = attrs.iter().any(|a| a.name == "i18n");
        // A button holding elements or structural blocks lays them out
        // inside it, with anonymous text for the words between them.
        let content_button = internal == Some(Kind::Button)
            && !component
            && children.iter().any(|c| !inline(c, false));
        if internal == Some(Kind::Box) || content_button {
            // Text between a block's elements is an anonymous text
            // element, its inline markup kept (HTML's anonymous block box).
            let key = attrs.iter().find(|a| a.name == "i18n");
            let mut texts = Vec::new();
            let mut run = Vec::new();
            for child in children {
                if inline(child, false) {
                    run.push(child.clone());
                    continue;
                }
                texts.extend(self.anonymous_text(&mut run, i, translated)?);
                let child = self.node(child, i)?;
                self.doc.nodes[i].children.push(child);
            }
            texts.extend(self.anonymous_text(&mut run, i, translated)?);
            if let Some(a) = key {
                let [text] = texts[..] else {
                    return Err((
                        a.at,
                        format!(
                            "i18n on a <{tag}> translates its one run of text; put each run in its own <span i18n>"
                        ),
                    ));
                };
                let value = if a.bound {
                    PropValue::Bind(expression(&a.value, a.at)?, None)
                } else {
                    PropValue::Lit(Lit::Str(a.value.clone()))
                };
                self.prop(text, Some(Kind::Label), "key", value, a.at, &a.value)?;
            }
            return Ok(i);
        }
        if internal == Some(Kind::SvgText)
            || internal == Some(Kind::Label)
                && !component
                && (tag == "a" || children.iter().any(|c| matches!(c, Node::Element { .. })))
        {
            if translated && children.iter().any(|c| matches!(c, Node::Element { .. })) {
                return Err((
                    at,
                    "i18n replaces the whole text, so it takes no inline elements; put the markup in the string".into(),
                ));
            }
            let own = (tag == "a").then_some(usize::MAX);
            if internal == Some(Kind::SvgText) {
                let children = self.text_path(i, children)?;
                let read = self.text_runs(i, &children, own, at, translated);
                self.white_space = None;
                read?;
                return Ok(i);
            }
            self.text_runs(i, children, own, at, translated)?;
            return Ok(i);
        }
        let mut text = Expr::Lit(serde_json::Value::String(String::new()));
        let mut has_text = false;
        let mut bound_text = false;
        let text_bearing =
            schema::prop("text").is_some_and(|(_, p)| internal.is_some_and(|kind| p.applies(kind)));
        for child in children {
            let value = match child {
                Node::Text(value, at) => {
                    if value.trim().is_empty() && !text_bearing {
                        continue;
                    }
                    if !translated && !value.trim().is_empty() {
                        self.warn(*at);
                    }
                    // Its source: its computed white-space processes it.
                    Some(Expr::Lit(value.as_str().into()))
                }
                Node::Expression(value, at) => {
                    bound_text = true;
                    Some(expression(value, *at)?)
                }
                _ => None,
            };
            if let Some(value) = value {
                has_text = true;
                text = Expr::Bin(crate::bind::Op::Add, Box::new(text), Box::new(value));
            } else {
                let child = self.node(child, i)?;
                self.doc.nodes[i].children.push(child);
            }
        }
        if has_text {
            let value = if bound_text {
                PropValue::Bind(text, None)
            } else {
                PropValue::Lit(Lit::Str(crate::bind::text(&text.eval(
                    &crate::bind::Scope {
                        vm: &serde_json::Value::Null,
                        locals: Vec::new(),
                        env: &crate::bind::NoEnv,
                    },
                ))))
            };
            let source = children
                .iter()
                .filter_map(|child| match child {
                    Node::Text(text, _) => Some(text.clone()),
                    Node::Expression(expr, _) => Some(format!("{{{expr}}}")),
                    _ => None,
                })
                .collect::<String>();
            self.prop(i, internal, "text", value, at, &source)?;
        }
        Ok(i)
    }
}

impl Parser {
    pub(super) fn prop(
        &mut self,
        node: usize,
        kind: Option<Kind>,
        name: &str,
        value: PropValue,
        at: usize,
        source: &str,
    ) -> Result<(), Error> {
        let (spec, p) = schema::prop(name).ok_or((
            at,
            format!("unknown property {name}; use a reflected property"),
        ))?;
        // A component takes what every element does.
        let applies = match kind {
            Some(kind) => p.applies(kind),
            None => p.on == schema::On::All,
        };
        if !applies {
            let element = match kind {
                Some(kind) => kind.name(),
                None => self.doc.nodes[node]
                    .selector_tag
                    .as_deref()
                    .unwrap_or_default(),
            };
            return Err((
                at,
                format!("{name} does not apply to {element}; use its documented properties"),
            ));
        }
        if matches!(value, PropValue::Model(_)) && !p.two_way {
            return Err((at, format!("{name} is read-only; remove bind:")));
        }
        if name == "rive"
            && let PropValue::Lit(Lit::Str(id)) = &value
            && !id.split_once('#').is_some_and(|(file, board)| {
                file.ends_with(".riv") && !board.is_empty() && !board.contains('#')
            })
        {
            return Err((at, "og-rive src takes <file>.riv#<Artboard> relative to assets/, such as ui/chest.riv#Chest".into()));
        }
        if name == "dataDetents"
            && let PropValue::Lit(Lit::Str(text)) = &value
        {
            super::build::gestures::detents(text, 100.0).map_err(|error| (at, error))?;
        }
        // A modal's content is built in the top layer, a tree of its own
        // its descendants are named by; HTML chooses it per opening too
        // (showModal or show).
        if name == "modal" && !matches!(value, PropValue::Lit(_)) {
            return Err((
                at,
                "write modal as a literal, not bound: use two dialogs, or {#if} around one".into(),
            ));
        }
        let line = self.line(at);
        self.doc.nodes[node].props.push(PropSet {
            spec,
            value,
            line,
            source: source.into(),
        });
        Ok(())
    }
}

/// An `<option>`'s attributes that are its select's, not its box's.
const OPTION_ATTRS: [&str; 5] = ["value", "label", "i18n", "disabled", "selected"];

/// Each `<option>`'s content in a select's children, in order, through
/// its `<optgroup>`s.
fn option_contents(children: &[Node]) -> Vec<Vec<Node>> {
    let mut out = Vec::new();
    for c in children {
        match c {
            Node::Element { tag, children, .. } if tag == "option" => out.push(children.clone()),
            Node::Element { tag, children, .. } if tag == "optgroup" => {
                out.extend(option_contents(children));
            }
            _ => {}
        }
    }
    out
}

/// `nodes` without their elements' ids (a copy's ids would repeat them).
fn without_ids(nodes: Vec<Node>) -> Vec<Node> {
    nodes
        .into_iter()
        .map(|n| match n {
            Node::Element {
                tag,
                attrs,
                children,
                at,
                end,
                close,
            } => Node::Element {
                tag,
                attrs: attrs.into_iter().filter(|a| a.name != "id").collect(),
                children: without_ids(children),
                at,
                end,
                close,
            },
            n => n,
        })
        .collect()
}
