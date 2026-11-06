//! What the browsers' WGSL compiler (Tint) refuses and naga, which the
//! gate's other shader tests validate with (shaders.rs), accepts: Tint
//! refuses a module whole, so every pipeline made from it draws nothing in
//! a browser (the water, and with it whole frames, drew black in Chrome on
//! Android: E5's water.json,
//! the former test sample The browser compile in
//! reference-gfx.ts catches these at a wave's end; these run in the quick
//! lane over every module `og_render::shader_sources` gives.

/// WGSL's uniformity analysis (the spec's §15.2) for the derivatives and
/// implicit-level samples of every fragment stage, which naga leaves out
/// (`DISABLE_UNIFORMITY_REQ_FOR_FRAGMENT_STAGE`) and the browsers' Tint
/// enforces: a module with a derivative in non-uniform control flow is
/// refused whole in Chrome, and every pipeline made from it draws nothing
/// (the terrain after ebb512c3a's cause; the water's lake glitter after its
/// per-pixel `below_view` return, black water in every browser). The
/// browser compile in reference-gfx.ts runs only at a wave's end; this runs
/// in the quick lane. It follows the spec's rules: per-pixel values are
/// the stage's inputs, mutable module-scope variables, read-write storage
/// and what is computed from them; function-scope variables are tracked
/// along the flow; an `if` or `switch` that may return, break or continue
/// on a per-pixel condition leaves the rest non-uniform; a loop reconverges
/// unless its body returns; a call that takes a derivative must be made in
/// uniform control flow, with the callee analysed for the uniformity of
/// the arguments it is given.
mod uniformity {
    use naga::{
        AddressSpace, Block, Expression as E, Function, Handle, LocalVariable, Module, SampleLevel,
        Statement as S, StorageAccess,
    };
    use std::collections::{BTreeSet, HashMap};

    #[derive(Clone, Copy)]
    struct Summary {
        /// It takes a derivative: callable only in uniform control flow.
        requires: bool,
        returns_uniform: bool,
    }

    /// How control can leave a statement (the spec's behaviours) and the
    /// uniformity of control flow at its end.
    #[derive(Clone, Copy, Default)]
    struct Flow {
        next: bool,
        ret: bool,
        brk: bool,
        cont: bool,
    }

    pub struct Checker<'m> {
        module: &'m Module,
        source: &'m str,
        summaries: HashMap<(Handle<Function>, u64), Summary>,
        pub errors: BTreeSet<String>,
    }

    struct Walker<'m> {
        f: &'m Function,
        /// Bit i: argument i is uniform.
        args: u64,
        vars: HashMap<Handle<LocalVariable>, bool>,
        values: HashMap<Handle<E>, bool>,
        requires: bool,
        returns_uniform: bool,
        name: String,
    }

    impl<'m> Checker<'m> {
        pub fn new(module: &'m Module, source: &'m str) -> Self {
            Self {
                module,
                source,
                summaries: HashMap::new(),
                errors: BTreeSet::new(),
            }
        }

        /// Every fragment stage (its inputs per pixel), and every function
        /// with uniform arguments (what fails there fails for any caller).
        pub fn check(&mut self) {
            let module = self.module;
            for ep in &module.entry_points {
                if ep.stage == naga::ShaderStage::Fragment {
                    self.analyse(&ep.function, 0, &format!("@fragment {}", ep.name));
                }
            }
            for (h, _) in module.functions.iter() {
                self.summary(h, u64::MAX);
            }
        }

        fn summary(&mut self, h: Handle<Function>, args: u64) -> Summary {
            let f = &self.module.functions[h];
            let args = args & (1u64 << f.arguments.len().min(63)).wrapping_sub(1);
            if let Some(s) = self.summaries.get(&(h, args)) {
                return *s;
            }
            let name = format!("fn {}", f.name.as_deref().unwrap_or("?"));
            let s = self.analyse(f, args, &name);
            self.summaries.insert((h, args), s);
            s
        }

        fn analyse(&mut self, f: &'m Function, args: u64, name: &str) -> Summary {
            let mut w = Walker {
                f,
                args,
                vars: HashMap::new(),
                values: HashMap::new(),
                requires: false,
                returns_uniform: true,
                name: name.to_string(),
            };
            self.block(&mut w, &f.body, true);
            Summary {
                requires: w.requires,
                returns_uniform: w.returns_uniform,
            }
        }

        fn require(&mut self, w: &mut Walker, cf: bool, span: naga::Span, what: &str) {
            w.requires = true;
            if !cf {
                let line = span.location(self.source).line_number;
                self.errors.insert(format!(
                    "line {line}: {what} in non-uniform control flow ({})",
                    w.name
                ));
            }
        }

        /// The block's flow from control flow `cf`: (how it exits, the
        /// uniformity of control flow at its end).
        fn block(&mut self, w: &mut Walker, block: &Block, mut cf: bool) -> (Flow, bool) {
            let mut flow = Flow::default();
            for (s, span) in block.span_iter() {
                let (f, end) = self.statement(w, s, *span, cf);
                cf = end;
                flow.ret |= f.ret;
                flow.brk |= f.brk;
                flow.cont |= f.cont;
                if !f.next {
                    return (flow, cf);
                }
            }
            flow.next = true;
            (flow, cf)
        }

        fn statement(&mut self, w: &mut Walker, s: &S, span: naga::Span, cf: bool) -> (Flow, bool) {
            let next = Flow {
                next: true,
                ..Flow::default()
            };
            match s {
                S::Emit(range) => {
                    for h in range.clone() {
                        let v = w.compute(self.module, h);
                        w.values.insert(h, v);
                        let span = w.f.expressions.get_span(h);
                        match &w.f.expressions[h] {
                            E::Derivative { axis, ctrl, .. } => {
                                self.require(w, cf, span, &format!("derivative {axis:?} {ctrl:?}"))
                            }
                            E::ImageSample {
                                gather: None,
                                level: SampleLevel::Auto | SampleLevel::Bias(_),
                                ..
                            } => self.require(w, cf, span, "implicit-level texture sample"),
                            _ => {}
                        }
                    }
                    (next, cf)
                }
                S::Block(b) => self.block(w, b, cf),
                S::If {
                    condition,
                    accept,
                    reject,
                } => {
                    let inner = cf && w.value(self.module, *condition);
                    let before = w.vars.clone();
                    let (fa, ca) = self.block(w, accept, inner);
                    let after_a = std::mem::replace(&mut w.vars, before);
                    let (fb, cb) = self.block(w, reject, inner);
                    w.merge(&after_a);
                    let flow = Flow {
                        next: fa.next || fb.next,
                        ret: fa.ret || fb.ret,
                        brk: fa.brk || fb.brk,
                        cont: fa.cont || fb.cont,
                    };
                    let only_next = !(flow.ret || flow.brk || flow.cont);
                    (flow, if only_next { cf } else { ca && cb })
                }
                S::Switch { selector, cases } => {
                    let inner = cf && w.value(self.module, *selector);
                    let before = w.vars.clone();
                    let mut merged: Option<HashMap<_, _>> = None;
                    let mut flow = Flow::default();
                    let mut end = true;
                    for c in cases {
                        w.vars = before.clone();
                        let (fc, cc) = self.block(w, &c.body, inner);
                        // A break leaves the switch.
                        flow.next |= fc.next || fc.brk;
                        flow.ret |= fc.ret;
                        flow.cont |= fc.cont;
                        end &= cc;
                        match &mut merged {
                            None => merged = Some(w.vars.clone()),
                            Some(m) => {
                                for (k, v) in &w.vars {
                                    let e = m.entry(*k).or_insert(true);
                                    *e &= *v;
                                }
                            }
                        }
                    }
                    w.vars = merged.unwrap_or(before);
                    let only_next = !(flow.ret || flow.cont);
                    (flow, if only_next { cf } else { end })
                }
                S::Loop {
                    body,
                    continuing,
                    break_if,
                } => {
                    let mut iter_cf = cf;
                    let mut flow = Flow::default();
                    for _ in 0..8 {
                        let start = (iter_cf, w.vars.clone());
                        let (fb, c1) = self.block(w, body, iter_cf);
                        let (fc, mut c2) = self.block(w, continuing, c1);
                        if let Some(b) = break_if {
                            c2 &= w.value(self.module, *b);
                        }
                        flow.ret |= fb.ret || fc.ret;
                        flow.brk |= fb.brk || break_if.is_some();
                        let again = fb.next || fb.cont;
                        let next_cf = if again { cf && c1 && c2 } else { iter_cf };
                        w.merge(&start.1);
                        if next_cf == start.0 && w.vars == start.1 {
                            break;
                        }
                        iter_cf = next_cf;
                    }
                    let flow = Flow {
                        next: flow.brk,
                        ret: flow.ret,
                        ..Flow::default()
                    };
                    (flow, if flow.ret { iter_cf } else { cf })
                }
                S::Break => (
                    Flow {
                        brk: true,
                        ..Flow::default()
                    },
                    cf,
                ),
                S::Continue => (
                    Flow {
                        cont: true,
                        ..Flow::default()
                    },
                    cf,
                ),
                S::Return { value } => {
                    let v = value.is_none_or(|e| w.value(self.module, e));
                    w.returns_uniform &= cf && v;
                    (
                        Flow {
                            ret: true,
                            ..Flow::default()
                        },
                        cf,
                    )
                }
                S::Store { pointer, value } => {
                    let v = cf && w.value(self.module, *value) && w.address(self.module, *pointer);
                    if let Some((var, whole)) = w.root(*pointer) {
                        let e = w.vars.entry(var).or_insert(true);
                        *e = if whole { v } else { *e && v };
                    }
                    (next, cf)
                }
                S::Call {
                    function,
                    arguments,
                    result,
                } => {
                    let mut mask = 0u64;
                    for (i, a) in arguments.iter().enumerate().take(64) {
                        if w.value(self.module, *a) {
                            mask |= 1 << i;
                        }
                    }
                    let callee = self.summary(*function, mask);
                    if callee.requires {
                        let name = self.module.functions[*function]
                            .name
                            .clone()
                            .unwrap_or_default();
                        self.require(w, cf, span, &format!("call to {name} (takes a derivative)"));
                    }
                    if let Some(r) = result {
                        w.values.insert(*r, cf && callee.returns_uniform);
                    }
                    // What a callee writes through a pointer: uniform when
                    // everything it was given is.
                    let all = arguments.iter().all(|a| w.value(self.module, *a));
                    for a in arguments {
                        if let Some((var, _)) = w.root(*a) {
                            let e = w.vars.entry(var).or_insert(true);
                            *e &= cf && all;
                        }
                    }
                    (next, cf)
                }
                S::Atomic {
                    result: Some(r), ..
                }
                | S::SubgroupBallot { result: r, .. }
                | S::SubgroupGather { result: r, .. }
                | S::SubgroupCollectiveOperation { result: r, .. } => {
                    w.values.insert(*r, false);
                    (next, cf)
                }
                S::WorkGroupUniformLoad { result, .. } => {
                    w.values.insert(*result, true);
                    (next, cf)
                }
                // `discard` demotes to a helper invocation: not a divergence.
                _ => (next, cf),
            }
        }
    }

    impl Walker<'_> {
        fn merge(&mut self, other: &HashMap<Handle<LocalVariable>, bool>) {
            for (k, v) in other {
                let e = self.vars.entry(*k).or_insert(true);
                *e &= *v;
            }
        }

        fn value(&self, module: &Module, h: Handle<E>) -> bool {
            match self.values.get(&h) {
                Some(v) => *v,
                None => self.compute(module, h),
            }
        }

        /// The local variable a pointer is into, and whether it names all of it.
        fn root(&self, p: Handle<E>) -> Option<(Handle<LocalVariable>, bool)> {
            match self.f.expressions[p] {
                E::LocalVariable(v) => Some((v, true)),
                E::Access { base, .. } | E::AccessIndex { base, .. } => {
                    self.root(base).map(|(v, _)| (v, false))
                }
                _ => None,
            }
        }

        /// Whether a pointer's indices are uniform.
        fn address(&self, module: &Module, p: Handle<E>) -> bool {
            match self.f.expressions[p] {
                E::Access { base, index } => {
                    self.address(module, base) && self.value(module, index)
                }
                E::AccessIndex { base, .. } => self.address(module, base),
                _ => true,
            }
        }

        /// Whether what a pointer points at is uniform.
        fn pointee(&self, module: &Module, p: Handle<E>) -> bool {
            match self.f.expressions[p] {
                E::LocalVariable(v) => self.vars.get(&v).copied().unwrap_or(true),
                E::GlobalVariable(g) => match module.global_variables[g].space {
                    AddressSpace::Uniform | AddressSpace::Handle | AddressSpace::Immediate => true,
                    AddressSpace::Storage { access } => !access.contains(StorageAccess::STORE),
                    _ => false,
                },
                E::Access { base, index } => {
                    self.pointee(module, base) && self.value(module, index)
                }
                E::AccessIndex { base, .. } => self.pointee(module, base),
                _ => false,
            }
        }

        fn compute(&self, module: &Module, h: Handle<E>) -> bool {
            let v = |e: Handle<E>| self.value(module, e);
            let o = |e: Option<Handle<E>>| e.is_none_or(v);
            match &self.f.expressions[h] {
                E::Literal(_)
                | E::Constant(_)
                | E::Override(_)
                | E::ZeroValue(_)
                | E::GlobalVariable(_)
                | E::LocalVariable(_)
                | E::WorkGroupUniformLoadResult { .. } => true,
                E::FunctionArgument(i) => *i < 64 && self.args & (1 << i) != 0,
                E::Load { pointer } => {
                    self.pointee(module, *pointer) && self.address(module, *pointer)
                }
                E::Compose { components, .. } => components.iter().all(|c| v(*c)),
                E::Access { base, index } => v(*base) && v(*index),
                E::AccessIndex { base, .. } => v(*base),
                E::Splat { value, .. } => v(*value),
                E::Swizzle { vector, .. } => v(*vector),
                E::ImageSample {
                    image,
                    sampler,
                    coordinate,
                    array_index,
                    offset,
                    level,
                    depth_ref,
                    ..
                } => {
                    let level = match level {
                        SampleLevel::Auto | SampleLevel::Zero => true,
                        SampleLevel::Exact(e) | SampleLevel::Bias(e) => v(*e),
                        SampleLevel::Gradient { x, y } => v(*x) && v(*y),
                    };
                    v(*image)
                        && v(*sampler)
                        && v(*coordinate)
                        && o(*array_index)
                        && o(*offset)
                        && o(*depth_ref)
                        && level
                }
                E::ImageLoad {
                    image,
                    coordinate,
                    array_index,
                    sample,
                    level,
                } => v(*image) && v(*coordinate) && o(*array_index) && o(*sample) && o(*level),
                E::ImageQuery { image, query } => {
                    v(*image)
                        && match query {
                            naga::ImageQuery::Size { level } => o(*level),
                            _ => true,
                        }
                }
                E::Unary { expr, .. } => v(*expr),
                E::Binary { left, right, .. } => v(*left) && v(*right),
                E::Select {
                    condition,
                    accept,
                    reject,
                } => v(*condition) && v(*accept) && v(*reject),
                E::Derivative { expr, .. } => v(*expr),
                E::Relational { argument, .. } => v(*argument),
                E::Math {
                    arg,
                    arg1,
                    arg2,
                    arg3,
                    ..
                } => v(*arg) && o(*arg1) && o(*arg2) && o(*arg3),
                E::As { expr, .. } => v(*expr),
                E::ArrayLength(e) => v(*e),
                // Results recorded by their statements; the rest per invocation.
                _ => false,
            }
        }
    }
}

/// [`uniformity`] over every module: no derivative or implicit-level sample
/// in non-uniform control flow, as the browsers' compilers require.
#[test]
fn every_derivative_is_taken_in_uniform_control_flow() {
    let mut failures = Vec::new();
    let mut checked = 0;
    for s in og_render::shader_sources() {
        let module = naga::front::wgsl::parse_str(&s.wgsl)
            .unwrap_or_else(|e| panic!("{}: {}", s.label, e.emit_to_string(&s.wgsl)));
        let mut c = uniformity::Checker::new(&module, &s.wgsl);
        c.check();
        checked += 1;
        failures.extend(c.errors.into_iter().map(|e| format!("{}: {e}", s.label)));
    }
    assert!(checked >= 30, "{checked} modules");
    assert!(
        failures.is_empty(),
        "refused by the browsers' WGSL compilers (Tint): {failures:#?}"
    );
}

/// WGSL's expression grammar, which naga parses with C's precedences and
/// the browsers' Tint parses as written: the operands of `&`, `|`, `^`,
/// `<<` and `>>` are unary expressions, so `a * b ^ c` needs its
/// parentheses ("mixing '*' and '^' requires parenthesis", the water's
/// lake glints, refused whole in Chrome), as do `a && b || c`, `a < b < c`
/// and `a << b + c`. A flat run of binary operators between separators,
/// inside each pair of parentheses, brackets or template brackets (found
/// as the spec's template list discovery does), is checked against it.
mod grammar {
    #[derive(Clone, Copy, PartialEq)]
    enum K {
        Ident,
        Number,
        Punct,
    }
    struct Tok<'a> {
        k: K,
        s: &'a str,
        at: usize,
    }

    const PUNCT: [&str; 22] = [
        "<<=", ">>=", "->", "&&", "||", "==", "!=", "<=", ">=", "<<", ">>", "+=", "-=", "*=", "/=",
        "%=", "&=", "|=", "^=", "++", "--", "::",
    ];
    const KEYWORDS: [&str; 22] = [
        "let",
        "var",
        "const",
        "override",
        "return",
        "if",
        "else",
        "for",
        "while",
        "switch",
        "case",
        "default",
        "loop",
        "continuing",
        "fn",
        "struct",
        "break",
        "continue",
        "discard",
        "alias",
        "enable",
        "requires",
    ];

    fn tokens(src: &str) -> Vec<Tok<'_>> {
        let b = src.as_bytes();
        let mut out = Vec::new();
        let mut i = 0;
        while i < b.len() {
            let c = b[i];
            if c.is_ascii_whitespace() {
                i += 1;
            } else if src[i..].starts_with("//") {
                i = src[i..].find('\n').map_or(b.len(), |n| i + n);
            } else if src[i..].starts_with("/*") {
                i = src[i..].find("*/").map_or(b.len(), |n| i + n + 2);
            } else if c.is_ascii_alphabetic() || c == b'_' {
                let start = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                out.push(Tok {
                    k: K::Ident,
                    s: &src[start..i],
                    at: start,
                });
            } else if c.is_ascii_digit()
                || (c == b'.' && b.get(i + 1).is_some_and(u8::is_ascii_digit))
            {
                let start = i;
                let hex = src[i..].starts_with("0x") || src[i..].starts_with("0X");
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'.' || b[i] == b'_')
                {
                    let exp = if hex { b"pP" } else { b"eE" };
                    i += 1;
                    if exp.contains(&b[i - 1]) && i < b.len() && (b[i] == b'+' || b[i] == b'-') {
                        i += 1;
                    }
                }
                out.push(Tok {
                    k: K::Number,
                    s: &src[start..i],
                    at: start,
                });
            } else {
                let n = PUNCT
                    .iter()
                    .find(|p| src[i..].starts_with(*p))
                    .map_or(1, |p| p.len());
                out.push(Tok {
                    k: K::Punct,
                    s: &src[i..i + n],
                    at: i,
                });
                i += n;
            }
        }
        out
    }

    /// The spec's template list discovery: the `<` and `>` (or half a `>>`)
    /// that bracket a template list, by token index.
    fn templates(t: &[Tok]) -> (Vec<bool>, Vec<u8>) {
        let mut open = vec![false; t.len()];
        let mut close = vec![0u8; t.len()];
        let mut pending: Vec<(usize, usize)> = Vec::new();
        let mut depth = 0usize;
        let mut i = 0;
        while i < t.len() {
            let s = t[i].s;
            if t[i].k == K::Ident && t.get(i + 1).is_some_and(|n| n.s == "<") {
                pending.push((i + 1, depth));
                i += 2;
                continue;
            }
            match s {
                ">" | ">>" => {
                    // A `>>` closes two lists when two are open here.
                    for _ in 0..s.len() {
                        if let Some((o, _)) = pending.pop_if(|p| p.1 == depth) {
                            open[o] = true;
                            close[i] += 1;
                        }
                    }
                }
                "(" | "[" => depth += 1,
                ")" | "]" => {
                    pending.retain(|p| p.1 < depth);
                    depth = depth.saturating_sub(1);
                }
                "=" | ";" | "{" | ":" => {
                    pending.clear();
                    if s != "=" {
                        depth = 0;
                    }
                }
                "&&" | "||" => pending.retain(|p| p.1 < depth),
                _ => {}
            }
            i += 1;
        }
        (open, close)
    }

    const BINARY: [&str; 18] = [
        "*", "/", "%", "+", "-", "<<", ">>", "<", ">", "<=", ">=", "==", "!=", "&&", "||", "&",
        "|", "^",
    ];

    /// Whether a flat run of binary operators parses as WGSL.
    fn valid(ops: &[&str]) -> bool {
        let bitwise = |o: &&str| matches!(*o, "&" | "|" | "^");
        if ops.iter().any(bitwise) {
            return ops.iter().all(|o| *o == ops[0]);
        }
        if ops.contains(&"&&") && ops.contains(&"||") {
            return false;
        }
        ops.split(|o| matches!(*o, "&&" | "||")).all(|rel| {
            let relational = |o: &&str| matches!(*o, "<" | ">" | "<=" | ">=" | "==" | "!=");
            rel.iter().filter(|o| relational(o)).count() <= 1
                && rel
                    .split(relational)
                    .all(|side| !side.iter().any(|o| matches!(*o, "<<" | ">>")) || side.len() == 1)
        })
    }

    /// Every run of operators WGSL refuses: (line, the operators).
    pub fn refused(src: &str) -> Vec<(usize, String)> {
        let t = tokens(src);
        let (open, close) = templates(&t);
        let line = |at: usize| src[..at].matches('\n').count() + 1;
        let mut out = Vec::new();
        // Per open group: its run's operators and where it starts.
        let mut groups: Vec<(Vec<&str>, usize)> = vec![(Vec::new(), 0)];
        let mut operand = false;
        let end = |g: &mut (Vec<&str>, usize), out: &mut Vec<(usize, String)>| {
            if !valid(&g.0) {
                out.push((line(g.1), g.0.join(" ")));
            }
            g.0.clear();
        };
        for (i, tok) in t.iter().enumerate() {
            let s = tok.s;
            if open[i] || s == "(" || s == "[" {
                groups.push((Vec::new(), tok.at));
                operand = false;
                continue;
            }
            let closes = if close[i] > 0 {
                close[i] as usize
            } else {
                usize::from(s == ")" || s == "]")
            };
            if closes > 0 {
                for _ in 0..closes {
                    if groups.len() > 1 {
                        let mut g = groups.pop().unwrap();
                        end(&mut g, &mut out);
                    }
                }
                operand = true;
                continue;
            }
            let g = groups.last_mut().unwrap();
            match tok.k {
                K::Ident if KEYWORDS.contains(&s) => {
                    end(g, &mut out);
                    g.1 = tok.at;
                    operand = false;
                }
                K::Ident | K::Number => operand = true,
                K::Punct if operand && BINARY.contains(&s) => {
                    if g.0.is_empty() {
                        g.1 = tok.at;
                    }
                    g.0.push(s);
                    operand = false;
                }
                // A member access, or a unary operator: inside the operand.
                K::Punct
                    if matches!(s, "." | "!" | "~")
                        || (!operand && matches!(s, "-" | "&" | "*")) =>
                {
                    operand = false
                }
                K::Punct => {
                    end(g, &mut out);
                    g.1 = tok.at;
                    operand = false;
                }
            }
        }
        for mut g in groups {
            end(&mut g, &mut out);
        }
        out
    }

    #[test]
    fn the_grammar_check_refuses_what_tint_refuses() {
        for bad in [
            "let a = b * c ^ d;",
            "let a = b & c | d;",
            "if (a && b || c) {}",
            "let a = b < c < d;",
            "let a = b << c + d;",
            "let a = f(b * c ^ d, e);",
            "let a = x ^ y == z;",
        ] {
            assert!(!refused(bad).is_empty(), "accepted {bad}");
        }
        for good in [
            "let a = (b * c) ^ (d * e) ^ f;",
            "let a = vec2<f32>(b * c, -d) + e[i] * f.x;",
            "var<uniform> u: array<vec4<f32>, 4>;",
            "let a = select(b, c, d < e && f > g);",
            "let a = (b << 2u) | (c >> 3u) | d;",
            "let a = b << c == d;",
            "fn f(p: ptr<function, vec2<u32>>) -> u32 { return (*p).x ^ ~(*p).y; }",
            "let a = array<vec2<f32>, 2>(vec2<f32>(1e-5, -2.0), vec2<f32>(0x1p-3, .5))[i] * 2.0 - 1.0;",
            "let h = bitcast<u32>(a) * 3u;",
            "let a = !b && c;",
            "let a = &v; *a = b - -c * d;",
        ] {
            assert!(
                refused(good).is_empty(),
                "refused {good}: {:?}",
                refused(good)
            );
        }
    }
}

/// [`grammar`] over every module: what the browsers parse as written.
#[test]
fn every_module_parses_as_wgsl_writes_its_operators() {
    let mut failures = Vec::new();
    for s in og_render::shader_sources() {
        for (line, ops) in grammar::refused(&s.wgsl) {
            failures.push(format!(
                "{}: line {line}: `{ops}` needs parentheses",
                s.label
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "refused by the browsers' WGSL parser (Tint), accepted by naga: {failures:#?}"
    );
}
