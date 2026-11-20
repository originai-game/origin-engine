use og_ui::UiDocument;
fn main() {
    for (id, p) in std::env::args().skip(1).map(|a| {
        let (i, p) = a.split_once('=').unwrap();
        (i.to_string(), p.to_string())
    }) {
        let doc = UiDocument::parse(&id, &std::fs::read_to_string(&p).unwrap()).unwrap();
        println!(
            "== {id} name={} root={} nodes={}",
            doc.name,
            doc.root,
            doc.nodes.len()
        );
        fn walk(doc: &UiDocument, i: usize, d: usize) {
            let n = &doc.nodes[i];
            let props: Vec<String> = n
                .props
                .iter()
                .map(|p| format!("{}={:?}", og_ui::doc::schema::PROPS[p.spec].name, p.value))
                .collect();
            let attrs: Vec<String> = n
                .selector_attrs
                .iter()
                .map(|(k, e)| format!("{k}={e:?}"))
                .collect();
            let args: Vec<String> = n
                .args
                .iter()
                .map(|(k, v, _)| format!("{k}={v:?}"))
                .collect();
            println!(
                "{:w$}{i} {:?} #{} props[{}] attrs[{}] args[{}]",
                "",
                n.kind,
                n.id,
                props.join(", "),
                attrs.join(", "),
                args.join(", "),
                w = d * 2
            );
            for &c in &n.children {
                walk(doc, c, d + 1);
            }
        }
        walk(&doc, doc.root, 0);
    }
}
