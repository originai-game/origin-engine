//! Structural selectors on pushed screens: each element's instantiated
//! children (after `{#if}`, keyed `{#each}`, snippets and components) are
//! listed with their selector facts before they are built, so a child's
//! position, siblings and ancestors are known when its style resolves.
//! Only built while a defined document has structural rules.
use super::*;
use crate::bind::InteractionState;
use crate::doc::matching::{self, Facts, MatchWork, Tree};
use crate::doc::selectors::Selector;

static NO_STATES: BTreeSet<InteractionState> = BTreeSet::new();

#[derive(Clone, Debug)]
pub(crate) struct Entry {
    document: Arc<UiDocument>,
    node: usize,
    attributes: BTreeMap<String, Value>,
    text: String,
    children: bool,
    /// Its instantiated element children, listed while a document has
    /// `:has()` rules.
    kids: Vec<Entry>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Level {
    entries: Vec<Entry>,
    /// The entry being built.
    current: Option<usize>,
    /// For a virtual container, each row item's first entry.
    items: Vec<usize>,
}

struct Levels<'a>(&'a [Level]);

/// An entry of a level, then a path down its listed children.
type At = (usize, usize, Vec<usize>);

impl Levels<'_> {
    fn entry(&self, (level, index, path): &At) -> &Entry {
        let mut e = &self.0[*level].entries[*index];
        for &k in path {
            e = &e.kids[k];
        }
        e
    }

    /// The element's siblings.
    fn siblings(&self, at: &At) -> &[Entry] {
        match at.2.split_last() {
            None => &self.0[at.0].entries,
            Some((_, up)) => &self.entry(&(at.0, at.1, up.to_vec())).kids,
        }
    }

    fn index(at: &At) -> usize {
        at.2.last().copied().unwrap_or(at.1)
    }

    fn sibling(at: &At, k: usize) -> At {
        let mut at = at.clone();
        match at.2.last_mut() {
            Some(last) => *last = k,
            None => at.1 = k,
        }
        at
    }
}

impl Tree for Levels<'_> {
    type Id = At;

    fn facts(&self, at: &At) -> Facts<'_> {
        let e = self.entry(at);
        Facts {
            scope: &e.document.name,
            tag: e.document.nodes[e.node]
                .selector_tag
                .as_deref()
                .unwrap_or_default(),
            root: e.node == e.document.root,
            attributes: &e.attributes,
            states: &NO_STATES,
            text: &e.text,
        }
    }

    fn parent(&self, (level, index, path): &At) -> Option<At> {
        if let Some((_, up)) = path.split_last() {
            return Some((*level, *index, up.to_vec()));
        }
        let up = level.checked_sub(1)?;
        Some((up, self.0[up].current?, Vec::new()))
    }

    fn previous(&self, at: &At) -> Option<At> {
        Some(Self::sibling(at, Self::index(at).checked_sub(1)?))
    }

    fn following(&self, at: &At) -> Option<At> {
        let k = Self::index(at) + 1;
        (k < self.siblings(at).len()).then(|| Self::sibling(at, k))
    }

    fn position(&self, at: &At) -> (usize, usize) {
        (Self::index(at), self.siblings(at).len())
    }

    fn filtered(&self, at: &At, filter: &[Selector]) -> (usize, usize) {
        let index = Self::index(at);
        let mut before = 0;
        let mut total = 0;
        for k in 0..self.siblings(at).len() {
            if matching::filter_matches(&self.facts(&Self::sibling(at, k)), filter) {
                total += 1;
                before += usize::from(k < index);
            }
        }
        (before, total)
    }

    fn has_children(&self, at: &At) -> bool {
        let e = self.entry(at);
        e.children || !e.kids.is_empty()
    }

    fn children(&self, at: &At) -> Vec<At> {
        (0..self.entry(at).kids.len())
            .map(|k| {
                let mut child = at.clone();
                child.2.push(k);
                child
            })
            .collect()
    }
}

/// The selector facts a node's authored inputs evaluate to, as the retained
/// topology records them: `class` joins the class list and true `class:` names.
pub(super) fn selector_facts(
    node: &DocNode,
    eval: impl Fn(&crate::bind::Expr) -> Value,
) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    let mut classes = BTreeSet::new();
    let mut has_classes = false;
    for (name, expression) in &node.selector_attrs {
        let value = eval(expression);
        if name == "class" {
            has_classes = true;
            if let Some(value) = value.as_str() {
                classes.extend(value.split_whitespace().map(str::to_owned));
            }
            continue;
        }
        if let Some(class) = name.strip_prefix("class:") {
            has_classes = true;
            if truthy(&value) {
                classes.insert(class.to_owned());
            }
        }
        out.insert(name.clone(), value);
    }
    if has_classes {
        out.insert(
            "class".into(),
            json!(classes.into_iter().collect::<Vec<_>>().join(" ")),
        );
    }
    out
}

impl Ui {
    /// Starts an element's place among its instantiated siblings; answers
    /// whether it opened the top level (a mount root without a parent level).
    pub(super) fn enter_structure(&mut self, b: &Build<'_>, i: usize) -> bool {
        if !self.docs.structure.any {
            return false;
        }
        if std::mem::take(&mut self.docs.component_root) {
            return false;
        }
        let root = self.docs.levels.is_empty();
        if root {
            let entry = self.entry(b, &b.doc, i);
            self.docs.levels.push(Level {
                entries: vec![entry],
                ..Default::default()
            });
        }
        let level = self.docs.levels.last_mut().expect("a structure level");
        level.current = Some(level.current.map_or(0, |k| k + 1));
        root
    }

    /// The instantiated children of an element, in build order.
    pub(super) fn child_level(&self, b: &mut Build<'_>, n: &DocNode) -> Option<Level> {
        if !self.docs.structure.any {
            return None;
        }
        let mut level = Level::default();
        let virtual_rows = matches!(n.kind, NodeKind::Element("list" | "cells"));
        let doc = b.doc.clone();
        self.instantiate(b, &doc, &n.children, &mut level, virtual_rows);
        Some(level)
    }

    /// Positions the next build at a virtual container's row `k`.
    pub(super) fn structure_row(&mut self, k: usize) {
        if let Some(level) = self.docs.levels.last_mut()
            && let Some(&first) = level.items.get(k)
        {
            level.current = first.checked_sub(1);
        }
    }

    /// Whether the element being built is the subject of `selector`.
    pub(super) fn structure_matches(&self, doc: &UiDocument, selector: &Selector) -> bool {
        let levels = &self.docs.levels;
        let Some(top) = levels.len().checked_sub(1) else {
            return false;
        };
        let Some(current) = levels[top].current else {
            return false;
        };
        matching::matches(
            &Levels(levels),
            Some(&doc.name),
            selector,
            &(top, current, Vec::new()),
            &mut MatchWork::default(),
        )
    }

    /// The number of the `<li>` being built ([`crate::doc::lists::ordinal`]).
    pub(super) fn structure_ordinal(&self, filter: &[Selector], item_values: bool) -> i64 {
        let levels = &self.docs.levels;
        match levels
            .len()
            .checked_sub(1)
            .and_then(|top| Some((top, levels[top].current?, Vec::new())))
        {
            Some(at) => crate::doc::lists::ordinal(&Levels(levels), &at, filter, item_values),
            None => 1,
        }
    }

    /// A `<li>`'s number when its marker shows one: `count` reads it from
    /// the tree being built (the retained topology or the sibling levels).
    pub(super) fn ordinal_of(
        &self,
        n: &DocNode,
        r: &Resolved,
        count: impl FnOnce(&[Selector], bool) -> i64,
    ) -> Option<i64> {
        let profile = &self.docs.structure;
        (profile.ordinals
            && n.selector_tag.as_deref() == Some("li")
            && r.str("markerType").is_some_and(crate::doc::lists::numbered))
        .then(|| count(&crate::doc::lists::items(), profile.item_values))
    }

    fn entry(&self, b: &Build<'_>, doc: &Arc<UiDocument>, i: usize) -> Entry {
        let n = &doc.nodes[i];
        let text = n
            .props
            .iter()
            .find(|p| PROPS[p.spec].name == "text")
            .and_then(|p| match b.value(self, PROPS[p.spec].ty, &p.value) {
                Some(V::Str(text)) => Some(text),
                _ => None,
            })
            .unwrap_or_default();
        Entry {
            document: doc.clone(),
            node: i,
            attributes: selector_facts(n, |e| b.eval(e)),
            text,
            children: false,
            kids: Vec::new(),
        }
    }

    /// Lists the elements `children` instantiate, mirroring [`Ui::build_node`].
    fn instantiate(
        &self,
        b: &mut Build<'_>,
        doc: &Arc<UiDocument>,
        children: &[usize],
        level: &mut Level,
        virtual_rows: bool,
    ) {
        for &c in children {
            let n = &doc.nodes[c];
            match &n.kind {
                NodeKind::Element(_) | NodeKind::Use(_) => {
                    let mut entry = self.entry(b, doc, c);
                    let structure = &self.docs.structure;
                    if (structure.empty || structure.has) && matches!(n.kind, NodeKind::Element(_))
                    {
                        let mut inner = Level::default();
                        self.instantiate(b, doc, &n.children, &mut inner, false);
                        entry.children = !inner.entries.is_empty();
                        if self.docs.structure.has {
                            entry.kids = inner.entries;
                        }
                    }
                    level.entries.push(entry);
                }
                NodeKind::If(e) => {
                    if truthy(&b.eval(e)) {
                        self.instantiate(b, doc, &n.children, level, false);
                    }
                }
                NodeKind::Key(_) => self.instantiate(b, doc, &n.children, level, false),
                NodeKind::Snippet(_) => {}
                NodeKind::Render { name, .. } => {
                    if let Some(snippet) = doc
                        .nodes
                        .iter()
                        .find(|s| s.kind == NodeKind::Snippet(name.clone()))
                    {
                        self.instantiate(b, doc, &snippet.children, level, false);
                    }
                }
                NodeKind::For {
                    var,
                    list,
                    key,
                    empty,
                } => {
                    let items = b.eval(list).as_array().cloned().unwrap_or_default();
                    if items.is_empty() {
                        self.instantiate(b, doc, empty, level, false);
                    }
                    for (k, item) in items.into_iter().enumerate() {
                        if virtual_rows {
                            level.items.push(level.entries.len());
                        }
                        b.locals.push((var.clone(), item, None));
                        b.locals.push(("$index".into(), json!(k), None));
                        if b.eval(key).as_str().is_some_and(|key| !key.is_empty()) {
                            self.instantiate(b, doc, &n.children, level, false);
                        }
                        b.locals.pop();
                        b.locals.pop();
                    }
                }
            }
        }
    }
}
