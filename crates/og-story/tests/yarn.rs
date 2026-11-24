use og_core::Rng;
use og_story::{Dialogue, Library, Story};
use std::collections::BTreeMap;
use std::sync::Arc;

fn library(body: &str) -> Library {
    let source = format!("#og-profile:yarn-spinner-3.2.2\n#og-name:test\n{body}");
    let dialogue = Dialogue::parse(&source).unwrap();
    let dialogue = Dialogue::decode(&dialogue.encode().unwrap()).unwrap();
    Library {
        dialogues: BTreeMap::from([("test".into(), Arc::new(dialogue))]),
        ..Default::default()
    }
}

fn start(l: &Library) -> (Story, u32) {
    let mut s = Story::new(Rng::new(19).fork("story"));
    let id = s
        .start(l, "test", BTreeMap::new(), BTreeMap::new(), None)
        .unwrap();
    (s, id)
}

fn restore(s: &Story) -> Story {
    let mut bytes = Vec::new();
    s.save(&mut bytes);
    Story::load(&bytes).unwrap()
}

#[test]
fn weighted_line_groups_restore_the_story_rng() {
    let l = library(
        "title: Start\n---\nReady #line:ready\n=> A <<if true>> #line:a #weight:1\n=> B <<if true>> #line:b #weight:7\n=> Fallback #line:fallback #weight:100000\n=> Never <<if false>> #line:never #weight:100000\n<<jump Start>>\n===",
    );
    let (mut s, id) = start(&l);
    let mut back = restore(&s);
    for _ in 0..20 {
        s.advance(&l, id).unwrap();
        back.advance(&l, id).unwrap();
        assert_eq!(s.convs[&id].history.last(), back.convs[&id].history.last());
        assert!(!matches!(
            s.convs[&id].history.last().unwrap().key.as_str(),
            "fallback" | "never"
        ));
        assert_eq!(s.rng.save(), back.rng.save());
    }
}
