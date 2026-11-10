use super::*;

#[test]
fn retains_selector_inputs_without_any_matching_stylesheet() {
    let doc = UiDocument::parse(
        "facts.svelte",
        r#"
<meta name="component" content="facts">
<column class="root fixed">
  {#if $shown}<text id="Label" class={$classes} class:active={$selected}>Hello</text>{/if}
  <input type="range" id="Amount" step="0.25" bind:value={$amount}/>
  <button onclick={choose} class:quiet="false">Choose</button>
</column>"#,
    )
    .unwrap();
    assert_eq!(
        doc.nodes[doc.root].selector_attrs,
        vec![("class".into(), Expr::Lit("root fixed".into()))]
    );
    let label = doc.nodes.iter().find(|node| node.id == "Label").unwrap();
    assert_eq!(label.selector_tag.as_deref(), Some("text"));
    assert_eq!(
        label.selector_attrs,
        vec![
            ("id".into(), Expr::Lit("Label".into())),
            ("class".into(), Expr::Path("$classes".into())),
            ("class:active".into(), Expr::Path("$selected".into())),
        ]
    );
    let mut paths = Vec::new();
    doc.node_paths(label, &mut paths);
    assert!(paths.contains(&"$classes".into()) && paths.contains(&"$selected".into()));
    let amount = doc.nodes.iter().find(|node| node.id == "Amount").unwrap();
    assert!(
        amount
            .selector_attrs
            .contains(&("step".into(), Expr::Lit("0.25".into())))
    );
    assert!(
        amount
            .selector_attrs
            .contains(&("bind:value".into(), Expr::Path("$amount".into())))
    );
    let button = doc
        .nodes
        .iter()
        .find(|node| node.selector_tag.as_deref() == Some("button"))
        .unwrap();
    assert_eq!(
        button.selector_attrs,
        vec![("class:quiet".into(), Expr::Lit(false.into()))]
    );
    let conditional = doc
        .nodes
        .iter()
        .find(|node| matches!(node.kind, NodeKind::If(_)))
        .unwrap();
    assert!(conditional.selector_tag.is_none() && conditional.selector_attrs.is_empty());
}

#[test]
fn interpolation_keeps_word_boundaries_and_literal_rune_names() {
    let doc = UiDocument::parse(
        "words.svelte",
        r#"
<meta name="screen" content="words">
<text> {'Hello'}   {'world'} &amp; {'$state'} </text>
"#,
    )
    .unwrap();
    let prop = doc.nodes[doc.root]
        .props
        .iter()
        .find(|p| p.name() == "text")
        .unwrap();
    let PropValue::Bind(expr, _) = &prop.value else {
        panic!("interpolated text")
    };
    let scope = crate::bind::Scope {
        vm: &serde_json::Value::Null,
        locals: Vec::new(),
        env: &crate::bind::NoEnv,
    };
    assert_eq!(expr.eval(&scope), serde_json::json!("Hello world & $state"));
    assert!(
        UiDocument::parse(
            "source.svelte",
            "<meta name='screen' content='x'><text>{$stateful}</text>"
        )
        .is_ok()
    );
}

#[test]
fn parses_keyed_branches_models_snippets_and_scoped_css() {
    let doc = UiDocument::parse(
        "test.svelte",
        r#"
<meta name="screen" content="test">
<meta name="prop:items" content="list">
<meta name="prop:amount" content="number">
<meta name="prop:active" content="bool">
<meta name="action:confirm" content="Choose">
{#snippet details()}<text id="Details">{t('details')}</text>{/snippet}
<column id="Root" onconfirm={choose}>
  {#each $items as item (item.id)}
<button id="Choice" class:active={$active} onclick={choose}>{item.name}</button>
  {:else}<text id="Empty" i18n="empty">Empty</text>{/each}
  {#if $active}<input type="range" id="Value" step="0.1" bind:value={$amount}/>
  {:else if $amount > 4}<text id="High">{t('high')}</text>
  {:else}<text id="Low">{t('low')}</text>{/if}
  {@render details()}
</column>
<style>
button { opacity: 0.2; }
#Choice { opacity: 0.4; }
button.active { opacity: 1; }
@media (width < 600px), (height < 600px) { #Root { padding: 12px; } }
</style>"#,
    )
    .unwrap();
    assert!(doc.warnings.is_empty());
    let root = doc.nodes.iter().find(|n| n.id == "Root").unwrap();
    assert!(matches!(&root.triggers[0].when, When::Action(name) if name == "confirm"));
    assert!(matches!(&root.triggers[0].act, Act::Emit(name) if name == "choose"));
    assert!(doc.nodes.iter().any(|n| matches!(&n.kind, NodeKind::For { key: Expr::Path(k), empty, .. } if k == "item.id" && !empty.is_empty())));
    let value = doc.nodes.iter().find(|n| n.id == "Value").unwrap();
    assert!(
        value
            .props
            .iter()
            .any(|p| p.value == PropValue::Model("$amount".into()))
    );
    assert!(value.props.iter().any(|p| p.name() == "stepBy"));
    let choice = doc.nodes.iter().find(|n| n.id == "Choice").unwrap();
    // Its three author rules after HTML's `button { text-align: center }`.
    assert_eq!(choice.rules.len(), 4);
    assert_eq!(
        choice.rules.last().unwrap().props[0].value,
        PropValue::Lit(Lit::Num(0.4)),
        "ID specificity beats a later class rule"
    );
    assert!(doc.nodes.iter().any(|n| matches!(
        n.kind,
        NodeKind::Render {
            optional: false,
            ..
        }
    )));
}

#[test]
fn html_tables_forms_fieldsets_and_figures_take_their_semantics() {
    let doc = UiDocument::parse(
        "html.svelte",
        r#"
<meta name="component" content="html">
<div>
  <table id="T">
<caption>Scores</caption>
<thead><tr><th id="H">Name</th><th>Score</th></tr></thead>
<tbody>
  {#each $rows as row (row.id)}
  <tr><th id="R">{row.name}</th><td colspan="2" rowspan="0">{row.score}</td></tr>
  {/each}
</tbody>
  </table>
  <form onsubmit={save} onreset={clear}>
<fieldset disabled={$busy}>
  <legend>Profile <input type="checkbox" id="Inside" bind:checked={$a}></legend>
  <input id="Name" bind:value={$name} required>
  <input type="checkbox" id="Off" disabled={$locked} bind:checked={$b}>
</fieldset>
<button id="Go">Save</button>
<button id="Undo" type="reset">Undo</button>
<button id="Other" type="button">Other</button>
  </form>
  <figure id="F"><img src={$pic} alt="A"><figcaption>A map</figcaption></figure>
  <output id="O" for="Name">{$name}</output>
</div>
"#,
    )
    .unwrap();
    let node = |id: &str| doc.nodes.iter().find(|n| n.id == id).unwrap();
    let text = |n: &DocNode, prop: &str| {
        n.props
            .iter()
            .rev()
            .find(|p| p.name() == prop)
            .map(|p| p.value.clone())
    };
    assert_eq!(node("T").role, Some("table"));
    assert_eq!(
        text(node("T"), "name"),
        Some(PropValue::Lit(Lit::Str("Scores".into())))
    );
    assert_eq!(node("H").role, Some("columnheader"));
    // A header beside data cells heads its row.
    assert_eq!(node("R").role, Some("rowheader"));
    let cell = doc
        .nodes
        .iter()
        .find(|n| n.selector_tag.as_deref() == Some("td"))
        .unwrap();
    assert_eq!(
        (cell.role, text(cell, "colspan")),
        (Some("cell"), Some(PropValue::Lit(Lit::Num(2.0))))
    );
    assert_eq!(text(cell, "rowspan"), Some(PropValue::Lit(Lit::Num(0.0))));
    // A disabled fieldset's controls, but those in its legend.
    assert_eq!(text(node("Inside"), "disabled"), None);
    assert_eq!(
        text(node("Name"), "disabled"),
        Some(PropValue::Bind(Expr::Path("$busy".into()), None))
    );
    assert_eq!(
        text(node("Off"), "disabled"),
        Some(PropValue::Bind(
            Expr::Bin(
                crate::bind::Op::Or,
                Box::new(Expr::Path("$locked".into())),
                Box::new(Expr::Path("$busy".into()))
            ),
            None
        ))
    );
    let emits = |id: &str| -> Vec<String> {
        node(id)
            .triggers
            .iter()
            .filter_map(|t| match (&t.when, &t.act) {
                (When::Click, Act::Emit(a) | Act::Submit { action: a, .. }) => Some(a.clone()),
                _ => None,
            })
            .collect()
    };
    assert_eq!(emits("Go"), ["save"]);
    assert_eq!(emits("Undo"), ["clear"]);
    assert!(emits("Other").is_empty());
    assert_eq!(
        (node("F").role, text(node("F"), "name")),
        (
            Some("figure"),
            Some(PropValue::Lit(Lit::Str("A map".into())))
        )
    );
    assert_eq!(node("O").role, Some("status"));
    for (body, reason) in [
        ("<div><tr><td/></tr></div>", "belongs inside a <table>"),
        ("<table><tr><td colspan='0'/></tr></table>", "colspan takes"),
        ("<table><tr><th scope='all'/></tr></table>", "scope is"),
        (
            "<table><tr><td/></tr><caption>x</caption></table>",
            "first child",
        ),
        ("<div><template><p/></template></div>", "{#snippet"),
        (
            "<div><button type='submit'>Go</button></div>",
            "inside a <form>",
        ),
        ("<div><output for='Nope'>x</output></div>", "no element"),
    ] {
        let text = format!("<meta name='screen' content='t'>\n{body}");
        let errors = UiDocument::parse("bad.svelte", &text).unwrap_err();
        assert!(errors[0].contains(reason), "{body}: {}", errors[0]);
    }
}

#[test]
fn rejects_wrong_syntax_and_reports_authored_locations() {
    for (body, reason) in [
        ("<column><mystery/></column>", "unknown element"),
        (
            "<column><text id='A'/><text id='A'/></column>",
            "duplicate id",
        ),
        (
            "<column>{#each $items as item}<text/>{/each}</column>",
            "stable key",
        ),
        (
            "<column>{#each $items as item ($index)}<text/>{/each}</column>",
            "stable string",
        ),
        ("<text>{$amount = 2}</text>", "side-effect-free"),
        ("<text>{$state(1)}</text>", "game's mounted stores"),
        ("<button on:click={next}/>", "onclick"),
        ("<slot/>", "{@render"),
        ("<script>let count = $state(0)</script><text/>", "TS system"),
        (
            "<script>$effect(() => log())</script><text/>",
            "$effect is not allowed",
        ),
        ("<script>import X from './x'</script><text/>", "imports"),
        (
            "<script>function go() {}</script><text/>",
            "functions are not allowed",
        ),
        (
            "<script>const a = 1</script><text/>",
            "variable, which is not allowed",
        ),
        (
            "<script>let { a = b() } = $props()</script><text/>",
            "not a literal",
        ),
        ("<text>{@html $raw}</text>", "TS system"),
        ("<column>{@const a = 1}</column>", "directly inside a block"),
        (
            "<column>{#await $a + 1}{/await}</column>",
            "a promise never runs",
        ),
        ("<column>{#key $a}<text/></column>", "{/key}"),
        (
            "<svg><path d='M0 0' in:draw={{ speed: 1 }}/></svg>",
            "give duration",
        ),
        ("<div in:draw></div>", "SVG shape"),
        ("<div animate:flip></div>", "keyed {#each}"),
        ("<div transition:spin></div>", "not a built-in transition"),
        ("<div in:fade out:fade transition:fade></div>", "set twice"),
        (
            "<div transition:fly={{ easing: wobble }}></div>",
            "svelte/easing",
        ),
        ("<div style={$css}></div>", "literal CSS"),
        (
            "<div style='colour: red'></div>",
            "unsupported CSS property",
        ),
        ("<svelte:boundary><text/></svelte:boundary>", "never throws"),
        ("<svelte:window onkeydown={go}/><text/>", "input actions"),
        (
            r#"<input type="range" bind:value={$amount + 1}/>"#,
            "writable store",
        ),
        ("<text>{ { value: 1 } }</text>", "side-effect-free"),
        (
            "<text id='T'/><style>#T { animation: missing 1s linear forwards; }</style>",
            "declare @keyframes",
        ),
        (
            "<text id='T'/><style>#T { animation: fade 1s linear forwards; } @keyframes fade { 150% { opacity: 0; } }</style>",
            "percentage from 0% to 100%",
        ),
        (
            "<text id='T'/><style>#T { animation: fade 1s cubic-bezier(2,0,1,1) forwards; } @keyframes fade { from { opacity: 0; } to { opacity: 1; } }</style>",
            "CSS easing",
        ),
        (
            "<text id='T'/><style>#T:hover { animation: fade 1s; } @keyframes fade { from { transition-duration: 1s; } }</style>",
            "transition-duration cannot be animated in @keyframes",
        ),
    ] {
        let text = format!("<meta name='screen' content='test'>\n{body}");
        let errors = UiDocument::parse("bad.svelte", &text).unwrap_err();
        assert!(errors[0].starts_with("bad.svelte:2:"), "{}", errors[0]);
        assert!(errors[0].contains(reason), "{}", errors[0]);
    }
    let doc = UiDocument::parse(
        "text.svelte",
        "<meta name='screen' content='x'><text>Untranslated</text>",
    )
    .unwrap();
    assert_eq!(doc.warnings.len(), 1);
}
