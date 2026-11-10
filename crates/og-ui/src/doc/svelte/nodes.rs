use super::*;

impl Parser {
    pub(super) fn node(&mut self, node: &Node, parent: usize) -> Result<usize, Error> {
        match node {
            Node::If {
                condition,
                yes,
                no,
                at,
            } => {
                let cond = expression(condition, *at)?;
                let i = self.push(NodeKind::If(cond.clone()), parent, *at);
                self.doc.nodes[i].children = self.children(yes, i)?;
                if no.is_empty() {
                    return Ok(i);
                }
                // A transparent branch container preserves both child ranges.
                let wrapper = self.push(NodeKind::If(Expr::Lit(true.into())), parent, *at);
                self.doc.nodes[i].parent = wrapper;
                let other = self.push(NodeKind::If(Expr::Not(Box::new(cond))), wrapper, *at);
                self.doc.nodes[other].children = self.children(no, other)?;
                self.doc.nodes[wrapper].children = vec![i, other];
                Ok(wrapper)
            }
            Node::Each {
                list,
                item,
                key,
                body,
                empty,
                at,
            } => {
                let i = self.push(
                    NodeKind::For {
                        var: item.clone(),
                        list: expression(list, *at)?,
                        key: expression(key, *at)?,
                        empty: Vec::new(),
                    },
                    parent,
                    *at,
                );
                self.doc.nodes[i].children = self.children(body, i)?;
                let empty = self.children(empty, parent)?;
                if let NodeKind::For { empty: target, .. } = &mut self.doc.nodes[i].kind {
                    *target = empty;
                }
                Ok(i)
            }
            Node::Snippet { name, body, at } => {
                if self
                    .doc
                    .nodes
                    .iter()
                    .any(|n| n.parent == parent && n.kind == NodeKind::Snippet(name.clone()))
                {
                    return Err((
                        *at,
                        format!("duplicate snippet {name}; give each snippet a unique name"),
                    ));
                }
                let i = self.push(NodeKind::Snippet(name.clone()), parent, *at);
                self.doc.nodes[i].children = self.children(body, i)?;
                Ok(i)
            }
            Node::Render { name, optional, at } => Ok(self.push(
                NodeKind::Render {
                    name: name.clone(),
                    optional: *optional,
                },
                parent,
                *at,
            )),
            Node::Key { expr, body, at } => {
                let i = self.push(NodeKind::Key(expression(expr, *at)?), parent, *at);
                self.doc.nodes[i].children = self.children(body, i)?;
                Ok(i)
            }
            Node::Const { name, expr, at } => {
                // Reached only outside a block's children (an element's).
                let value = expression(expr, *at)?;
                self.constant(NONE, name, value, *at)?;
                unreachable!("a const outside a block is refused")
            }
            Node::Await {
                expr,
                pending,
                then,
                catch,
                at,
            } => self.await_block(expr, pending, then, catch, parent, *at),
            Node::Script(_, at) => Err((
                *at,
                "a <script> sits at the top of the file, beside the root element".into(),
            )),
            Node::Text(_, at) | Node::Expression(_, at) => Err((
                *at,
                "text needs a text-bearing element such as <text> or <button>".into(),
            )),
            Node::Element {
                tag,
                attrs,
                children,
                at,
                ..
            } => self.element(tag, attrs, children, *at, parent),
        }
    }
    /// `{#await $load}`: three `{#if}`s over the settled record its store
    /// holds (`status` `pending`, `fulfilled` or `rejected`, Promise.allSettled's
    /// vocabulary), `{:then v}` naming its `value` and `{:catch e}` its
    /// `reason` as `{@const}` does. Nothing is awaited in the tick.
    fn await_block(
        &mut self,
        expr: &str,
        pending: &[Node],
        then: &Option<(String, Vec<Node>)>,
        catch: &Option<(String, Vec<Node>)>,
        parent: usize,
        at: usize,
    ) -> Result<usize, Error> {
        let Expr::Path(path) = expression(expr, at)? else {
            return Err((
                at,
                "{#await} reads a store or row field holding { status, value, reason }, such as {#await $load}; a promise never runs in the tick".into(),
            ));
        };
        let status = |s: &str| {
            Expr::Bin(
                crate::bind::Op::Eq,
                Box::new(Expr::Path(format!("{path}.status"))),
                Box::new(Expr::Lit(s.into())),
            )
        };
        let settled = Expr::Bin(
            crate::bind::Op::Or,
            Box::new(status("fulfilled")),
            Box::new(status("rejected")),
        );
        let wrapper = self.push(NodeKind::If(Expr::Lit(true.into())), parent, at);
        let mut arms = Vec::new();
        let branches = [
            (Expr::Not(Box::new(settled)), None, pending),
            (
                status("fulfilled"),
                then.as_ref().map(|(n, _)| (n, "value")),
                then.as_ref().map_or(&[][..], |(_, b)| b),
            ),
            (
                status("rejected"),
                catch.as_ref().map(|(n, _)| (n, "reason")),
                catch.as_ref().map_or(&[][..], |(_, b)| b),
            ),
        ];
        for (condition, alias, body) in branches {
            if body.is_empty() && alias.is_none() {
                continue;
            }
            let arm = self.push(NodeKind::If(condition), wrapper, at);
            if let Some((name, field)) = alias.filter(|(n, _)| !n.is_empty()) {
                let value = Expr::Path(format!("{path}.{field}"));
                self.constant(arm, name, value, at)?;
            }
            self.doc.nodes[arm].children = self.children(body, arm)?;
            arms.push(arm);
        }
        self.doc.nodes[wrapper].children = arms;
        Ok(wrapper)
    }
    /// `<svelte:element this={tag}>`: a literal tag is that element; a
    /// bound one is an `{#if}` over the text tags (when its content is
    /// inline) or the block tags, and nothing for another value.
    pub(super) fn dynamic_element(
        &mut self,
        attrs: &[Attr],
        children: &[Node],
        at: usize,
        parent: usize,
    ) -> Result<usize, Error> {
        let Some(this) = attrs.iter().find(|a| a.name == "this") else {
            return Err((at, "<svelte:element> needs this={tag}".into()));
        };
        let rest: Vec<Attr> = attrs.iter().filter(|a| a.name != "this").cloned().collect();
        if !this.bound {
            if this.value.starts_with("svelte:") {
                return Err((this.at, "this names an HTML or core tag".into()));
            }
            return self.element(&this.value, &rest, children, at, parent);
        }
        let tags: &[&str] = if children.iter().all(inline) {
            &["p", "span", "small", "h1", "h2", "h3", "h4", "h5", "h6"]
        } else {
            &[
                "div", "section", "header", "footer", "nav", "main", "aside", "article",
            ]
        };
        let mut chain: Vec<Node> = Vec::new();
        for tag in tags.iter().rev() {
            chain = vec![Node::If {
                condition: format!("({}) == '{tag}'", this.value),
                yes: vec![Node::Element {
                    tag: (*tag).into(),
                    attrs: rest.clone(),
                    children: children.to_vec(),
                    at,
                    end: at,
                    close: at,
                }],
                no: chain,
                at,
            }];
        }
        self.node(&chain[0], parent)
    }
    /// `<svelte:window>`'s attributes, on the root once it is read:
    /// `onresize={action}` when the screen's size changes, and
    /// `bind:innerWidth={w}` / `bind:innerHeight={h}` naming its size.
    pub(super) fn window(&mut self, attrs: &[Attr]) -> Result<(), Error> {
        for a in attrs {
            match a.name.as_str() {
                "onresize" => {
                    if !a.bound || !identifier(&a.value) {
                        return Err((
                            a.at,
                            "onresize names a mounted action: onresize={relayout}".into(),
                        ));
                    }
                    let size = expression("$ui.width + 'x' + $ui.height", a.at)?;
                    let line = self.line(a.at);
                    self.doc.nodes[self.doc.root].triggers.push(Trigger {
                        when: When::Change(size),
                        act: Act::Emit(a.value.clone()),
                        line,
                    });
                }
                "bind:innerWidth" | "bind:innerHeight" => {
                    if !a.bound || !identifier(&a.value) {
                        return Err((
                            a.at,
                            format!(
                                "{} names the size for the document (bind:innerWidth={{width}}); a store reads $ui.width",
                                a.name
                            ),
                        ));
                    }
                    let field = if a.name.ends_with("Width") {
                        "width"
                    } else {
                        "height"
                    };
                    if self.doc.derived.iter().any(|(n, _)| *n == a.value) {
                        return Err((a.at, format!("{} is declared twice", a.value)));
                    }
                    self.doc
                        .derived
                        .push((a.value.clone(), Expr::Path(format!("$ui.{field}"))));
                }
                name if name.starts_with("onkey") => {
                    return Err((
                        a.at,
                        format!(
                            "{name}: keys reach a game as input actions; declare <meta name=\"action:confirm\" content=\"…\"> and write onconfirm={{…}}"
                        ),
                    ));
                }
                other => {
                    return Err((
                        a.at,
                        format!(
                            "<svelte:window> takes onresize, bind:innerWidth and bind:innerHeight, not {other}"
                        ),
                    ));
                }
            }
        }
        Ok(())
    }
}
