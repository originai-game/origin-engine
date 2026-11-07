use og_core::Rng;
use og_story::{Dialogue, Event, Library, Story, Val, Wait};
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
fn edited_lines_and_detour_stack_restore_without_repeating_commands() {
    let source = "title: Start\n---\n<<declare $gold = 0>>\n<<set $gold = $gold + 1>>\n<<reward>>\n<<detour Shop>>\nAfter #line:after\n===\ntitle: Shop\n---\nMoved #line:moved\nShop: Hello #line:hello\nShop: Goodbye #line:goodbye\n===";
    let l = library(source);
    let (mut s, id) = start(&l);
    s.advance(&l, id).unwrap();
    s.events.clear();
    let mut back = restore(&s);
    let edited = library(
        &source
            .replace("Moved #line:moved\n", "")
            .replace("Shop: Hello", "Unrelated #line:inserted\nShop: Hello")
            .replace("After #line:after", "Moved #line:moved\nAfter #line:after"),
    );
    back.resolve(&edited, id).unwrap();
    assert_eq!(back.convs[&id].history.last().unwrap().key, "hello");
    back.advance(&edited, id).unwrap();
    assert_eq!(back.convs[&id].history.last().unwrap().key, "goodbye");
    back.advance(&edited, id).unwrap();
    assert_eq!(back.convs[&id].history.last().unwrap().key, "moved");
    back.advance(&edited, id).unwrap();
    assert_eq!(back.convs[&id].history.last().unwrap().key, "after");
    assert_eq!(back.get(&edited, "gold"), Some(Val::Num(1.0)));
    assert!(!back.events.iter().any(|e| matches!(e, Event::Emit { .. })));
}

#[test]
fn settled_choice_cannot_award_twice_after_restore() {
    let l = library(
        "title: Start\n---\n<<declare $gold = 0>>\n-> Buy <<if $gold == 0>> #line:buy\n    <<set $gold = $gold + 1>>\n    <<reward>>\n    Bought #line:bought\n-> Leave #line:leave\n    <<stop>>\n===",
    );
    let (mut s, id) = start(&l);
    assert_eq!(s.convs[&id].wait, Wait::Choices);
    s.choose(&l, id, 0).unwrap();
    s.events.clear();
    let mut back = restore(&s);
    assert!(back.choose(&l, id, 0).is_err());
    assert_eq!(back.get(&l, "gold"), Some(Val::Num(1.0)));
    assert!(back.events.is_empty());
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

#[test]
fn missing_line_id_and_unimplemented_features_are_errors() {
    for (body, expected) in [
        ("Hello", "#line:"),
        ("<<enum Mode>>", "missing <<endenum>>"),
        ("-> Buy", "#line:"),
        ("Hello #line:a #line:b", "duplicate #line"),
        ("<<jump {>>", "string expression"),
        ("Hello <<if>> #line:a", "condition"),
    ] {
        let source = format!(
            "#og-profile:yarn-spinner-3.2.2\n#og-name:test\ntitle: Start\n---\n{body}\n==="
        );
        let error = Dialogue::parse(&source).unwrap_err();
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("line 5"), "{error}");
    }
}

#[test]
fn branches_strings_and_nested_options_execute_only_the_selected_body() {
    let l = library(
        r#"title: Start
---
<<declare $name = "Kai $ and # literal">>
<<declare $gold = -2>>
<<if $gold > 0>>
Wrong #line:wrong
<<elseif $gold == -2 && !false>>
Hello {$name} #line:hello
-> Continue #line:continue
    -> Inner #line:inner
        <<set $gold to 7>>
        <<jump end>>
<<else>>
Wrong again #line:wrong_again
<<endif>>
===
title: end
---
Done #line:done
===
"#,
    );
    let (mut s, id) = start(&l);
    assert_eq!(
        s.convs[&id].history.last().unwrap().args,
        [("name".into(), Val::Str("Kai $ and # literal".into()))]
    );
    s.choose(&l, id, 0).unwrap();
    let mut back = restore(&s);
    back.choose(&l, id, 0).unwrap();
    assert_eq!(back.get(&l, "gold"), Some(Val::Num(7.0)));
    assert_eq!(back.convs[&id].history.last().unwrap().key, "done");
}

#[test]
fn yarn_expression_semantics_and_typed_calls() {
    use og_story::expr::Scope;
    use og_story::yarn::{
        functions, parse_expression,
        types::{Enum, Type, Types},
    };
    struct Values;
    impl Scope for Values {
        fn var(&self, _: &str) -> Option<Val> {
            None
        }
        fn seen(&self, _: &str) -> bool {
            false
        }
        fn quest_status(&self, _: &str) -> &'static str {
            "none"
        }
        fn quest_stage(&self, _: &str) -> String {
            String::new()
        }
        fn quest_count(&self, _: &str, _: &str) -> f64 {
            0.0
        }
        fn invoke(&self, name: &str, args: &[Val]) -> Result<Val, String> {
            functions::pure(name, args)?.ok_or_else(|| "unregistered function".into())
        }
    }
    for (source, expected) in [
        ("(16777216 + 1) - 16777216", Val::Num(0.0)),
        ("true or true and false", Val::Bool(false)),
        ("true xor false and true", Val::Bool(true)),
        ("2 lt 3 eq true", Val::Bool(true)),
        ("round(2.5) + round(3.5)", Val::Num(6.0)),
        ("inc(-2.2) + dec(-2.2)", Val::Num(-5.0)),
        ("int(-2.75)", Val::Num(-2.0)),
        ("decimal(-2.75)", Val::Num(-0.75)),
        ("string(true)", Val::Str("True".into())),
        (r#"format("{{{0,4}}}", "x")"#, Val::Str("{   x}".into())),
        (r#"format("{0,8:F2}", 12.5)"#, Val::Str("   12.50".into())),
        (
            r#"format("{0:N1}", -12345.5)"#,
            Val::Str("-12,345.5".into()),
        ),
        (r#"format("{0:F2}", "12.5")"#, Val::Str("12.5".into())),
    ] {
        let mut expression = parse_expression(source).unwrap();
        Types::default().infer(&mut expression, None).unwrap();
        assert_eq!(expression.eval(&Values).unwrap(), expected, "{source}");
    }
    assert!(
        parse_expression("false and unknown() ")
            .unwrap()
            .eval(&Values)
            .is_err()
    );
    let mut types = Types::default();
    let mut expression = parse_expression("$implicit + 2").unwrap();
    assert_eq!(
        types.infer(&mut expression, None).unwrap(),
        Some(Type::Number)
    );
    assert_eq!(types.variables["implicit"], Type::Number);
    for source in ["min(true, 2)", "round(2, 3)", "unknown(1)", "true + false"] {
        assert!(
            types
                .infer(&mut parse_expression(source).unwrap(), None)
                .is_err(),
            "{source}"
        );
    }
    types.enums.insert(
        "State".into(),
        Enum {
            raw: Type::Number,
            members: BTreeMap::from([("Ready".into(), Val::Num(3.0))]),
        },
    );
    types.enums.insert(
        "Other".into(),
        Enum {
            raw: Type::Number,
            members: BTreeMap::from([("Ready".into(), Val::Num(7.0))]),
        },
    );
    let mut member = parse_expression(".Ready").unwrap();
    assert_eq!(types.infer(&mut member, None).unwrap(), None);
    assert!(types.require_resolved(&member).is_err());
    assert_eq!(
        types
            .infer(&mut member, Some(&Type::Enum("State".into())))
            .unwrap(),
        Some(Type::Enum("State".into()))
    );
    assert_eq!(member.eval(&Values).unwrap(), Val::Num(3.0));
}

#[test]
fn markup_uses_ui_scalar_ranges_and_preserves_unknown_properties() {
    use og_story::yarn::markup::{Property, parse};
    let value = parse(
        "Mae: 🦊 [wave speed=2 active=true kind=gentle]Hi 🌙[/wave]",
        "en",
    )
    .unwrap();
    assert_eq!(value.text, "Mae: 🦊 Hi 🌙");
    let character = &value.spans[0];
    assert_eq!(
        (&*character.name, character.position, character.length),
        ("character", 0, 5)
    );
    assert_eq!(character.properties["name"], Property::String("Mae".into()));
    let wave = &value.spans[1];
    assert_eq!((&*wave.name, wave.position, wave.length), ("wave", 7, 4));
    assert_eq!(wave.properties["speed"], Property::Number(2.0));
    assert_eq!(wave.properties["active"], Property::Bool(true));
    assert_eq!(wave.properties["kind"], Property::String("gentle".into()));
    // The upstream .NET expected range would be (8, 5), not (7, 4).
    fn scalar(text: &str, utf16: usize) -> u32 {
        let mut offset = 0;
        for (index, c) in text.chars().enumerate() {
            if offset == utf16 {
                return index as u32;
            }
            offset += c.len_utf16();
            assert!(offset <= utf16, "upstream offset splits a surrogate pair");
        }
        assert_eq!(offset, utf16);
        text.chars().count() as u32
    }
    assert_eq!(wave.position, scalar(&value.text, 8));
    assert_eq!(
        wave.length,
        scalar(&value.text, 13) - scalar(&value.text, 8)
    );
    let raw = parse(r"\[a\] [nomarkup][b]raw[/b][/nomarkup]", "en").unwrap();
    assert_eq!(raw.text, "[a] [b]raw[/b]");
    assert_eq!(raw.spans.len(), 1);
    assert_eq!((raw.spans[0].position, raw.spans[0].length), (4, 10));
    let overlapping = parse("[a]one [b]two[/a] three[/b]", "en").unwrap();
    assert_eq!(
        (overlapping.spans[0].position, overlapping.spans[0].length),
        (0, 7)
    );
    assert_eq!(
        (overlapping.spans[1].position, overlapping.spans[1].length),
        (4, 9)
    );
    assert_eq!(parse("A [custom/] B", "en").unwrap().text, "A B");
    assert_eq!(
        parse("a\u{301} [b]x[/b]", "en").unwrap().spans[0].position,
        2
    );
}

#[test]
fn markup_locale_switch_reselects_cldr_categories() {
    use og_story::yarn::markup::parse;
    let source = "[plural value=2 one=one two=two few=few many=many other=other/]";
    assert_eq!(parse(source, "en").unwrap().text, "other");
    assert_eq!(parse(source, "ar").unwrap().text, "two");
    assert_eq!(parse(source, "ru").unwrap().text, "few");
    let source = "[ordinal value=2 one=first two=second few=third other=other/]";
    assert_eq!(parse(source, "en").unwrap().text, "second");
    assert_eq!(parse(source, "fr").unwrap().text, "other");
    assert_eq!(
        parse(r#"[select value=cat cat="% cats" dog=dogs/]"#, "en")
            .unwrap()
            .text,
        "cat cats"
    );
}

#[test]
fn repeated_commands_and_detours_keep_distinct_continuations_after_edits() {
    let source = "title: Start\n---\n<<reward>>\n<<detour Visit>>\n<<reward>>\n<<detour Visit>>\nDone #line:done\n===\ntitle: Visit\n---\nVisit #line:visit\n===";
    let lib = library(source);
    let (mut story, id) = start(&lib);
    story.events.clear();
    story.advance(&lib, id).unwrap();
    assert_eq!(
        story
            .events
            .iter()
            .filter(|e| matches!(e, Event::Emit { name, .. } if name == "reward"))
            .count(),
        1
    );
    story.events.clear();
    let mut restored = restore(&story);
    let edited = library(&source.replace(
        "Done #line:done",
        "Inserted #line:inserted\nDone #line:done",
    ));
    restored.advance(&edited, id).unwrap();
    assert_eq!(restored.convs[&id].history.last().unwrap().key, "inserted");
    assert!(
        !restored
            .events
            .iter()
            .any(|e| matches!(e, Event::Emit { .. }))
    );
    restored.advance(&edited, id).unwrap();
    assert_eq!(restored.convs[&id].history.last().unwrap().key, "done");
}

#[test]
fn typed_declarations_enums_smart_reads_and_compound_assignments() {
    let lib = library(
        r#"title: Start
---
<<declare $hp = 3 as number>>
<<declare $ready = $hp >= 10 as bool>>
<<enum Mode>>
<<case Closed>>
<<case Open>>
<<endenum>>
<<declare $mode = Mode.Closed as Mode>>
Before {$ready} #line:before
<<set $hp+=2>>
<<set $hp*=4>>
<<set $hp-=2>>
<<set $hp/=2>>
<<set $hp%=7>>
<<set $mode to .Open>>
<<if $mode == Mode.Open>>
<<set $hp = max($hp, 12)>>
After {$ready} {$hp} #line:after
<<endif>>
==="#,
    );
    let (mut story, id) = start(&lib);
    assert_eq!(
        story.convs[&id].history.last().unwrap().args,
        [("ready".into(), Val::Bool(false))]
    );
    story.advance(&lib, id).unwrap();
    assert_eq!(
        story.convs[&id].history.last().unwrap().args,
        [
            ("ready".into(), Val::Bool(true)),
            ("hp".into(), Val::Num(12.0))
        ]
    );
    assert!(story.warnings.is_empty(), "{:?}", story.warnings);
    for body in [
        "<<declare $x = true as number>>",
        "<<declare $x = 1>>\n<<set $x = false>>",
        "<<declare $x = $y + 1>>\n<<declare $y = $x + 1>>",
        "<<declare $x = 1>>\n<<declare $y = $x + 1>>\n<<set $y = 3>>",
        "<<enum Mode>>\n<<case A = 1>>\n<<case B = 1>>\n<<endenum>>",
    ] {
        let source = format!(
            "#og-profile:yarn-spinner-3.2.2\n#og-name:test\ntitle: Start\n---\n{body}\n==="
        );
        assert!(Dialogue::parse(&source).is_err(), "{body}");
    }
}

#[test]
fn choices_do_not_skip_preceding_lines_and_dynamic_jump_exits_detours() {
    let lib = library(
        r#"title: Start
---
First #line:first
Second #line:second
-> Continue #line:continue
    <<detour {"Visit"}>>
Wrong return #line:wrong
===
title: Visit
---
Visit #line:visit
<<jump {"Finish"}>>
===
title: Finish
---
Finished #line:finish
==="#,
    );
    let (mut story, id) = start(&lib);
    assert_eq!(story.convs[&id].wait, Wait::Line);
    assert_eq!(story.convs[&id].history.last().unwrap().key, "first");
    story.advance(&lib, id).unwrap();
    assert_eq!(story.convs[&id].history.last().unwrap().key, "second");
    assert_eq!(story.convs[&id].wait, Wait::Choices);
    story.choose(&lib, id, 0).unwrap();
    assert_eq!(story.convs[&id].history.last().unwrap().key, "visit");
    let mut story = restore(&story);
    story.advance(&lib, id).unwrap();
    assert_eq!(story.convs[&id].history.last().unwrap().key, "finish");
    assert!(story.convs[&id].stack.is_empty());
    story.advance(&lib, id).unwrap();
    assert_eq!(story.convs[&id].wait, Wait::Ended);
    assert!(story.warnings.is_empty(), "{:?}", story.warnings);
}

#[test]
fn registered_functions_are_typed_pure_and_survive_ir_roundtrip() {
    use og_story::yarn::{
        compile_with_functions, parse_expression,
        types::{Function, Signature, Type},
    };
    let source = "#og-profile:yarn-spinner-3.2.2\n#og-name:test\ntitle: Start\n---\nResult {scaled(3, 7)} #line:result\n===";
    let function = Function {
        signature: Signature {
            parameters: vec![Type::Number, Type::Number],
            variadic: None,
            returns: Type::Number,
        },
        parameters: vec!["value".into(), "factor".into()],
        body: parse_expression("$value * $factor + 2").unwrap(),
    };
    let registry = BTreeMap::from([("scaled".into(), function.clone())]);
    let d = compile_with_functions(source, registry.clone()).unwrap();
    let d = Dialogue::decode(&d.encode().unwrap()).unwrap();
    let lib = Library {
        dialogues: BTreeMap::from([("test".into(), Arc::new(d))]),
        ..Default::default()
    };
    let (story, id) = start(&lib);
    assert_eq!(
        story.convs[&id].history.last().unwrap().args[0].1,
        Val::Num(23.0)
    );
    assert!(story.warnings.is_empty());
    assert!(
        compile_with_functions(&source.replace("scaled(3, 7)", "scaled(true, 7)"), registry)
            .is_err()
    );
    for body in [
        "$outside",
        "random()",
        "visited_count(\"Start\")",
        "scaled($value, $factor)",
    ] {
        let mut function = function.clone();
        function.body = parse_expression(body).unwrap();
        assert!(
            compile_with_functions(source, BTreeMap::from([("scaled".into(), function)])).is_err(),
            "{body}"
        );
    }
}

#[test]
fn random_arguments_are_checkpointed_and_option_reads_do_not_draw_again() {
    let lib = library(
        "title: Start\n---\n<<declare $charm = 100>>\nReady #line:ready\nWait #line:wait\n-> Draw {dice(6)} {random_range(-3, 2)} {random()} #line:draw #reason:blocked #reason-arg:n=dice(9) #check:active:charm:dice(6) #fail:Failure\n    After {random_range_float(2.25, 5.75)} #line:after\n===\ntitle: Failure\n---\nUnexpected #line:failure\n===",
    );
    let (mut story, id) = start(&lib);
    let mut before = restore(&story);
    story.advance(&lib, id).unwrap();
    before.advance(&lib, id).unwrap();
    let d = &lib.dialogues["test"];
    let choice = &d.node(&story.convs[&id].node).unwrap().choices[0];
    let args = story.choice_args(id, choice);
    let mut expected = Rng::new(19).fork("story");
    let target = expected.integer(1, 6) as f64;
    assert_eq!(
        args.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>(),
        vec![
            Val::Num(expected.integer(1, 6) as f64),
            Val::Num(expected.integer(0, 5) as f64 - 3.0),
            Val::Num(f64::from(expected.next_f64() as f32)),
        ]
    );
    expected.integer(1, 9);
    let mut restored = restore(&story);
    for _ in 0..4 {
        assert_eq!(story.choice_args(id, choice), args);
        assert_eq!(restored.choice_args(id, choice), args);
        assert_eq!(story.choice_state(&lib, d, id, choice), (true, true, 1.0));
        assert_eq!(story.rng.save(), expected.save());
    }
    let roll = expected.integer(1, 20) as u32;
    for s in [&mut story, &mut before, &mut restored] {
        s.choose(&lib, id, 0).unwrap();
        assert!(s.warnings.is_empty(), "{:?}", s.warnings);
        assert_eq!(s.convs[&id].history[1].chosen_args, args);
        let checked = s.convs[&id].last_roll.as_ref().unwrap();
        assert_eq!((checked.target, checked.roll), (target, roll));
    }
    assert_eq!(story.convs[&id].history, before.convs[&id].history);
    assert_eq!(story.convs[&id].history, restored.convs[&id].history);
    assert_eq!(restore(&story).convs[&id].history, story.convs[&id].history);
    assert_eq!(
        story.convs[&id].history.last().unwrap().args[0].1,
        Val::Num(expected.integer(0, 3) as f64 + 2.25)
    );
    assert_eq!(story.rng.save(), expected.save());
}

#[test]
fn saliency_policies_distinguish_specificity_visits_and_unconditional_fallbacks() {
    use og_story::yarn::saliency::Saliency;
    let lib = library(
        r#"title: Start
---
<<detour Untracked>>
<<detour Group>>
Counts {visited_count("Group")} {visited_count("Un" + "tracked")} #line:counts
<<detour Group>>
===
title: Untracked
---
<<noop>>
===
title: Group
subtitle: fallback
when: always
tracking: never
---
Fallback #line:fallback
===
title: Group
subtitle: conditional
when: true and true
tracking: always
---
Conditional #line:conditional
==="#,
    );
    for (policy, first, second) in [
        (Saliency::First, "fallback", "fallback"),
        (Saliency::Best, "conditional", "conditional"),
        (Saliency::BestLeastRecentlyViewed, "conditional", "fallback"),
        (
            Saliency::RandomBestLeastRecentlyViewed,
            "conditional",
            "fallback",
        ),
        (
            Saliency::ConditionalFirstWeighted,
            "conditional",
            "conditional",
        ),
    ] {
        let mut story = Story::new(Rng::new(19).fork("story"));
        story.saliency = policy;
        let id = story
            .start(&lib, "test", Default::default(), Default::default(), None)
            .unwrap();
        assert_eq!(story.convs[&id].history.last().unwrap().key, first);
        story.advance(&lib, id).unwrap();
        let shown = story.convs[&id].history.last().unwrap();
        assert_eq!(shown.key, "counts");
        assert_eq!(
            shown
                .args
                .iter()
                .map(|(_, v)| v.clone())
                .collect::<Vec<_>>(),
            [Val::Num(1.0), Val::Num(0.0)]
        );
        let mut restored = restore(&story);
        for story in [&mut story, &mut restored] {
            story.advance(&lib, id).unwrap();
            assert_eq!(story.convs[&id].history.last().unwrap().key, second);
            assert!(story.warnings.is_empty(), "{:?}", story.warnings);
        }
        assert_eq!(story.convs[&id].history, restored.convs[&id].history);
        assert_eq!(story.rng.save(), restored.rng.save());
    }
}
