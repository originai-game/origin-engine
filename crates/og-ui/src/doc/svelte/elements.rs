use super::*;

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
            "option" | "optgroup" => {
                return Err((at, format!("<{tag}> belongs inside a <select>")));
            }
            "summary" => {
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
                let i = self.element_node(tag, attrs, &[], at, parent)?;
                let strs = |v: Vec<String>| PropValue::Lit(Lit::Strs(v));
                let items = options.iter().map(|o| o.label.clone()).collect();
                let values = options.iter().map(|o| o.value.clone()).collect();
                // Without `<option>`s its options are its bound `items`.
                if !options.is_empty() {
                    self.prop(i, tag, "items", strs(items), at, "")?;
                    self.prop(i, tag, "values", strs(values), at, "")?;
                }
                if options.iter().any(|o| !o.group.is_empty()) {
                    let groups = options.iter().map(|o| o.group.clone()).collect();
                    self.prop(i, tag, "groups", strs(groups), at, "")?;
                }
                if options.iter().any(|o| o.key.is_some()) {
                    let keys = options
                        .iter()
                        .map(|o| o.key.clone().unwrap_or_default())
                        .collect();
                    self.prop(i, tag, "keys", strs(keys), at, "")?;
                }
                let off: Vec<String> = (0..options.len())
                    .filter(|&k| options[k].disabled)
                    .map(|k| k.to_string())
                    .collect();
                if !off.is_empty() {
                    self.prop(i, tag, "disabledItems", strs(off), at, "")?;
                }
                // `<option selected>` is the value when nothing is bound.
                let valued = attrs
                    .iter()
                    .any(|a| matches!(a.name.as_str(), "value" | "bind:value"));
                if !valued && let Some(o) = options.iter().find(|o| o.selected) {
                    self.prop(
                        i,
                        tag,
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
                            summary = Some((text_value(inner, *at)?, *at));
                        }
                        other => rest.push(other.clone()),
                    }
                }
                let i = self.element_node(tag, attrs, &rest, at, parent)?;
                if let Some((Some((value, source)), at)) = summary {
                    self.prop(i, tag, "summary", value, at, &source)?;
                }
                return Ok(i);
            }
            _ => {}
        }
        self.element_node(tag, attrs, children, at, parent)
    }
    fn element_node(
        &mut self,
        tag: &str,
        attrs: &[Attr],
        children: &[Node],
        at: usize,
        parent: usize,
    ) -> Result<usize, Error> {
        // A label around a checkbox is the checkbox, its text the caption
        // (HTML's implicit label association).
        if tag == "label" {
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
                    "radio"
                } else {
                    "toggle"
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
                        control,
                        "side",
                        PropValue::Lit(Lit::Str("end".into())),
                        *inner_at,
                        "",
                    )?;
                }
                if let Some(((value, source), at)) = description {
                    self.prop(i, control, "description", value, at, &source)?;
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
        let html = self.html_kind(tag, parent, at)?;
        let internal = match tag {
            _ if let Some((_, kind)) =
                schema::ELEMENT_TAGS.iter().find(|(name, _)| *name == tag) =>
            {
                kind
            }
            _ if let Some((kind, _)) = html => kind,
            // SVG's text: placed in its view box (extern_element).
            "text" if super::extern_element::in_svg(self.parent_tag(parent)) => "svgText",
            "text" | "p" | "span" | "small" | "a" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                "label"
            }
            "label" if label_for.is_some() => "label",
            "div" | "section" | "header" | "footer" | "nav" | "main" | "aside" | "article"
            | "ul" | "ol" | "li" | "menu" => "box",
            "img" => "image",
            "video" => "video",
            "audio" => "audio",
            "hr" => "divider",
            "textarea" => "field",
            // An input's kind is its literal type's: a range is a slider, a
            // checkbox a toggle, every text type a field.
            "input" => match attrs
                .iter()
                .find(|a| a.name == "type" && !a.bound)
                .map(|a| a.value.as_str())
            {
                Some("range") => "slider",
                Some("checkbox") => "toggle",
                Some("radio") => "radio",
                Some("color") => "color",
                Some("date" | "time" | "datetime-local" | "month" | "week") => "datetime",
                Some("file") => "file",
                _ => "field",
            },
            other => match super::extern_element::kind(other, self.parent_tag(parent)) {
                Some(Ok(kind)) => kind,
                Some(Err(hint)) => return Err((at, hint)),
                None => match renamed(other) {
                    Some(hint) => return Err((at, hint)),
                    None => other,
                },
            },
        };
        let component = tag.starts_with(char::is_uppercase);
        let kind = if component {
            NodeKind::Use(tag.into())
        } else {
            NodeKind::Element(schema::kind(internal).ok_or((at, format!("unknown element <{tag}>; use a reflected core tag or capitalised component")))?.name)
        };
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
                let mpath = (tag == "animateMotion").then(crate::svg::Path::default);
                crate::svg::smil::Animation::parse(tag, &literal, mpath).map_err(|e| (at, e))?;
            }
        }
        for a in attrs {
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
            // An animation's attributes are read as it plays (svg::smil).
            if smil || tag == "mpath" {
                continue;
            }
            if a.name == "class" || a.name.starts_with("class:") {
                if a.bound {
                    expression(&a.value, a.at)?;
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
            if let Some(name) = a
                .name
                .strip_prefix("style:")
                .filter(|n| n.starts_with("--"))
            {
                // A bound custom property: the data a rule's var() reads.
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
            if internal == "box" && a.name == "i18n" {
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
                if internal != "slider" && internal != "field" {
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
            if tag == "input" && internal == "toggle" && a.name == "switch" && !a.bound {
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
            if tag == "input" && !matches!(internal, "field" | "datetime") && a.name == "type" {
                continue;
            }
            // A radio's group is the store it binds; its `name` adds nothing.
            if internal == "radio" && a.name == "name" {
                continue;
            }
            if a.name == "multiline" || (tag == "textarea" && a.name == "type") {
                return Err((
                    a.at,
                    "write several lines as <textarea>, one line as <input type=…>".into(),
                ));
            }
            if internal == "field"
                && a.name == "pattern"
                && !a.bound
                && crate::input_type::pattern(&a.value).is_none()
            {
                return Err((
                    a.at,
                    format!("pattern \"{}\" is not a regular expression", a.value),
                ));
            }
            if internal == "field"
                && a.name == "data-format"
                && !a.bound
                && let Err(e) = og_i18n::Message::parse(&a.value)
            {
                return Err((
                    a.at,
                    format!("data-format \"{}\" is not a message: {e}", a.value),
                ));
            }
            if internal == "field"
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
                _ if let Some((_, _, property)) = mapped => (*property).into(),
                ("field", "maxlength") => "maxChars".into(),
                ("field", "minlength") => "minChars".into(),
                ("field", "rows") if tag == "textarea" => "lines".into(),
                ("details", "name") => "exclusive".into(),
                (_, other) => super::extern_element::property(internal, other)
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
            if name == "value" && internal == "field" {
                name = "input".into();
            }
            if name == "value" && internal == "select" {
                name = "current".into();
            }
            if name == "value" && internal == "color" {
                name = "colorValue".into();
            }
            if matches!(internal, "video" | "audio") && a.name == "src" {
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
            if internal == "audio"
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
            if internal == "file" && name == "value" {
                name = "fileValue".into();
            }
            if internal == "datetime" {
                match name.as_str() {
                    "value" => name = "dateValue".into(),
                    "type" => name = "dateType".into(),
                    "min" => name = "dateMin".into(),
                    "max" => name = "dateMax".into(),
                    _ => {}
                }
            }
            if name == "value" && internal == "radio" {
                name = "choice".into();
            }
            if name == "value" && internal == "kbd" {
                name = "binding".into();
            }
            if a.name == "hidden" && !component {
                name = "visible".into();
            }
            // Selector facts only: `:lang()`, `:dir()`, `[data-…]` and
            // `:state(name)` (CSS custom states, `state:name={…}`).
            if !component
                && model.is_none()
                && schema::prop(&name).is_none()
                && (matches!(a.name.as_str(), "lang" | "dir")
                    || a.name.starts_with("data-")
                    || a.name.strip_prefix("state:").is_some_and(identifier))
            {
                if a.bound {
                    expression(&a.value, a.at)?;
                }
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
            } else if name == "stepBy" && a.value == "any" {
                // HTML's step="any": no step constraint.
                PropValue::Lit(Lit::Num(0.0))
            } else if let Some((_, spec)) = schema::prop(&name) {
                literal(spec.ty, &a.value).map_err(|why| (a.at, why))?
            } else if component {
                PropValue::Lit(Lit::Str(a.value.clone()))
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
        // An action's glyph follows the device: it reads `$ui.device`.
        if internal == "kbd" && attrs.iter().any(|a| a.name == "data-action") {
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
        // A button holding elements lays them out in a row inside it, its
        // text between them an anonymous text as a block's is.
        let content_button = internal == "button"
            && !component
            && children
                .iter()
                .any(|c| matches!(c, Node::Element { .. }) && !inline(c));
        if internal == "box" || content_button {
            // Text between a block's elements is an anonymous text
            // element, its inline markup kept (HTML's anonymous block box).
            let key = attrs.iter().find(|a| a.name == "i18n");
            let mut texts = Vec::new();
            let mut run = Vec::new();
            for child in children {
                if inline(child) {
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
                self.prop(text, "label", "key", value, a.at, &a.value)?;
            }
            return Ok(i);
        }
        if internal == "svgText"
            || internal == "label"
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
            if internal == "svgText" {
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
        let text_bearing = schema::prop("text").is_some_and(|(_, p)| p.applies(internal));
        for (index, child) in children.iter().enumerate() {
            let value = match child {
                Node::Text(value, at) => {
                    if value.trim().is_empty() && !text_bearing {
                        continue;
                    }
                    if !translated && !value.trim().is_empty() {
                        self.warn(*at);
                    }
                    let mut value = value.chars().fold(String::new(), |mut out, c| {
                        if c.is_ascii_whitespace() {
                            if !out.ends_with(' ') {
                                out.push(' ');
                            }
                        } else {
                            out.push(c);
                        }
                        out
                    });
                    if index == 0 {
                        value = value.trim_start_matches(' ').into();
                    }
                    if index + 1 == children.len() {
                        value.truncate(value.trim_end_matches(' ').len());
                    }
                    Some(Expr::Lit(value.into()))
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
        kind: &str,
        name: &str,
        value: PropValue,
        at: usize,
        source: &str,
    ) -> Result<(), Error> {
        let (spec, p) = schema::prop(name).ok_or((
            at,
            format!("unknown property {name}; use a reflected property"),
        ))?;
        if !p.applies(kind) {
            return Err((
                at,
                format!("{name} does not apply to {kind}; use its documented properties"),
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
