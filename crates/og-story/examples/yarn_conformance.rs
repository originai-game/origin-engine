//! Runs the pinned upstream fixtures without rewriting their expectations.
//! Also runs under WASI: preopen the repository as `.`.
use og_core::Rng;
use og_story::{Dialogue, Event, Library, Story, Val, Wait};
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn inventory(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(root).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            inventory(&path, files)?;
        } else if path.extension().is_some_and(|e| e == "yarn") {
            files.push(path);
        }
    }
    Ok(())
}

// Upstream permits implicit line IDs. Production does not. Only this fixture
// adapter adds them; it leaves the checked-in sources and test plans unchanged.
fn explicit_ids(source: &str) -> String {
    let mut out = format!("#og-profile:{}\n#og-name:corpus\n", og_story::yarn::PROFILE);
    let mut body = false;
    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "===" {
            body = false;
        }
        let content = line.split_once("//").map_or(line, |(s, _)| s);
        if body
            && !content.trim().is_empty()
            && !trimmed.starts_with("<<")
            && !content.contains("#line:")
        {
            // Keep comments after the inserted hashtag, not inside the ID.
            let (text, comment) = line.split_once("//").unwrap_or((line, ""));
            out.push_str(text.trim_end());
            out.push_str(&format!(" #line:fixture_{index}"));
            if !comment.is_empty() {
                out.push_str(" //");
                out.push_str(comment);
            }
        } else {
            out.push_str(line);
        }
        out.push('\n');
        if trimmed == "---" {
            body = true;
        }
    }
    out
}

#[derive(Debug, PartialEq)]
enum Output {
    Line(String),
    Option(String, bool),
    Command(String),
    Stop,
}

fn text(d: &Dialogue, key: &str, args: &[(String, Val)]) -> Result<String, String> {
    let source = d
        .strings
        .get(key)
        .ok_or_else(|| format!("missing text {key}"))?;
    let args = args
        .iter()
        .map(|(name, value)| {
            (
                name.clone(),
                og_i18n::Arg::Str(og_story::yarn::functions::string(value)),
            )
        })
        .collect();
    let text = og_i18n::Message::parse(source)
        .map_err(|e| e.to_string())?
        .format("en", &args);
    og_story::yarn::markup::parse(&text, "en").map(|markup| markup.text)
}

fn collect(s: &mut Story, lib: &Library, id: u32) -> Result<VecDeque<Output>, String> {
    let d = &lib.dialogues["corpus"];
    let mut result = VecDeque::new();
    for event in std::mem::take(&mut s.events) {
        match event {
            Event::Line { seq, .. } => {
                let shown = s.convs[&id].history.iter().find(|l| l.seq == seq).unwrap();
                let body = text(d, &shown.key, &shown.args)?;
                result.push_back(Output::Line(body));
            }
            Event::Emit { name, arg, .. } if name == "fixture_assert" => {
                if arg != "true" {
                    return Err(format!("upstream assertion failed: {arg}"));
                }
            }
            Event::Emit { name, arg, .. } => result.push_back(Output::Command(if arg.is_empty() {
                name
            } else {
                format!("{name} {arg}")
            })),
            Event::Ended { .. } => result.push_back(Output::Stop),
            event => return Err(format!("unexpected event {event:?}")),
        }
    }
    let conv = &s.convs[&id];
    if conv.wait == Wait::Choices {
        for choice in &d.node(&conv.node).unwrap().choices {
            let (visible, enabled, _) = s.choice_state(lib, d, id, choice);
            if visible {
                result.push_back(Output::Option(
                    text(d, &choice.text.key, &s.choice_args(id, choice))?,
                    enabled,
                ));
            }
        }
    }
    if !s.warnings.is_empty() {
        return Err(format!("runtime warnings: {:?}", s.warnings));
    }
    Ok(result)
}

fn checkpoint(s: &mut Story, lib: &Library, id: u32, restore: bool) -> Result<(), String> {
    let mut bytes = Vec::new();
    s.save(&mut bytes);
    let mut back = Story::load(&bytes)?;
    back.resolve(lib, id)?;
    let mut after = Vec::new();
    back.save(&mut after);
    if bytes != after {
        return Err("checkpoint changed after load/resolve/save".into());
    }
    if restore {
        *s = back;
    }
    Ok(())
}

fn quoted(step: &str) -> Result<&str, String> {
    step.split_once('`')
        .and_then(|(_, s)| s.rsplit_once('`').map(|(s, _)| s))
        .ok_or_else(|| format!("testplan expected backtick string: {step}"))
}

fn execute(d: Dialogue, plan: &str, restore: bool) -> Result<Vec<u8>, String> {
    let lib = Library {
        dialogues: BTreeMap::from([("corpus".into(), Arc::new(d))]),
        ..Default::default()
    };
    let mut story = Story::new(Rng::new(0).fork("story"));
    use og_story::yarn::saliency::Saliency;
    story.saliency = Saliency::BestLeastRecentlyViewed;
    let mut transcript = Vec::new();
    for run in plan.split("---") {
        let mut steps: Vec<_> = run
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .collect();
        if steps.last() != Some(&"stop") {
            steps.push("stop");
        }
        let mut node = "Start";
        for step in &steps {
            if let Some(value) = step.strip_prefix("node:") {
                node = value.trim();
            } else if let Some(value) = step.strip_prefix("set:") {
                let (name, value) = value.split_once('=').ok_or("invalid testplan set")?;
                let value = match value.trim() {
                    "true" => Val::Bool(true),
                    "false" => Val::Bool(false),
                    n => Val::Num(n.parse().map_err(|_| "invalid testplan number")?),
                };
                story.set(&lib, name.trim().trim_start_matches('$'), value);
            } else if let Some(strategy) = step.strip_prefix("saliency:") {
                story.saliency = match strategy.trim() {
                    "first" => Saliency::First,
                    "best" => Saliency::Best,
                    "best_least_recently_seen" => Saliency::BestLeastRecentlyViewed,
                    other => return Err(format!("unknown upstream saliency strategy: {other}")),
                };
            }
        }
        let id = story.start(&lib, "corpus", BTreeMap::new(), BTreeMap::new(), Some(node))?;
        let mut output = collect(&mut story, &lib, id)?;
        checkpoint(&mut story, &lib, id, restore)?;
        for step in steps {
            if step.starts_with("node:")
                || step.starts_with("set:")
                || step.starts_with("saliency:")
            {
                continue;
            }
            if let Some(index) = step.strip_prefix("select:") {
                if !output.is_empty() {
                    return Err(format!("unconsumed output before {step}: {output:?}"));
                }
                story.choose(
                    &lib,
                    id,
                    index
                        .trim()
                        .parse::<u32>()
                        .map_err(|_| "invalid selection")?
                        .wrapping_sub(1),
                )?;
                output = collect(&mut story, &lib, id)?;
                checkpoint(&mut story, &lib, id, restore)?;
                continue;
            }
            if output.is_empty() && story.convs[&id].wait == Wait::Line {
                story.advance(&lib, id)?;
                output = collect(&mut story, &lib, id)?;
                checkpoint(&mut story, &lib, id, restore)?;
            }
            let actual = output
                .pop_front()
                .ok_or_else(|| format!("no output for {step}"))?;
            let expected = if step.starts_with("line:") {
                if step.trim() == "line: *" {
                    if !matches!(actual, Output::Line(_)) {
                        return Err(format!("expected any line, got {actual:?}"));
                    }
                    transcript.extend(format!("{actual:?}\n").bytes());
                    continue;
                }
                Output::Line(quoted(step)?.into())
            } else if step.starts_with("option:") {
                Output::Option(quoted(step)?.into(), !step.ends_with("[disabled]"))
            } else if step.starts_with("command:") {
                Output::Command(quoted(step)?.into())
            } else if step == "stop" {
                Output::Stop
            } else {
                return Err(format!("unhandled testplan instruction {step}"));
            };
            if actual != expected {
                return Err(format!("{step}: got {actual:?}"));
            }
            // v3.2.2 TestPlan.g4 lexes trailing # text as COMMENT (longest
            // match), not a hashtag assertion. Do not adopt main's grammar.
            transcript.extend(format!("{actual:?}\n").bytes());
        }
        if !output.is_empty() || story.convs[&id].wait != Wait::Ended {
            return Err(format!("plan ended before dialogue: {output:?}"));
        }
    }
    story.save(&mut transcript);
    Ok(transcript)
}

fn run(path: &Path) -> Result<Vec<u8>, String> {
    let source = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    use og_story::yarn::types::{Function, Signature, Type::*};
    let mut functions = BTreeMap::new();
    // LanguageTests.cs and TestBase.cs register these before compilation.
    for (name, parameters, returns, body) in [
        (
            "add_three_operands",
            vec![("a", Number), ("b", Number), ("c", Number)],
            Number,
            "$a + $b + $c",
        ),
        (
            "set_objective_complete",
            vec![("objective", String)],
            Bool,
            "true",
        ),
        (
            "is_objective_active",
            vec![("objective", String)],
            Bool,
            "true",
        ),
        (
            "get_quest_status",
            vec![("quest", String)],
            String,
            "\"InProgress\"",
        ),
        ("dummy_bool", vec![], Bool, "true"),
        ("dummy_number", vec![], Number, "1"),
        ("dummy_string", vec![], String, "\"string\""),
        ("assert", vec![("value", Bool)], Bool, "$value"),
    ] {
        functions.insert(
            name.into(),
            Function {
                signature: Signature {
                    parameters: parameters.iter().map(|(_, t)| t.clone()).collect(),
                    variadic: None,
                    returns,
                },
                parameters: parameters.iter().map(|(n, _)| (*n).into()).collect(),
                body: og_story::yarn::parse_expression(body)?,
            },
        );
    }
    // These upstream inference fixtures describe external functions. The
    // engine requires their inferred signatures to be registered before cook.
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let registration = match name {
        "Inference-FunctionsCalledWithConvertibleParameters" => Some((
            "some_function",
            vec![("value", Enum("EnumA".into()))],
            Bool,
            "true",
        )),
        "Inference-FunctionsCannotChangeType" => Some(("somefunc", vec![], Number, "0")),
        "Inference-FunctionsMustHaveSameNumberOfParams" => {
            Some(("somefunc", vec![("value", Number)], Number, "$value"))
        }
        _ => None,
    };
    if let Some((name, parameters, returns, body)) = registration {
        functions.insert(
            name.into(),
            Function {
                signature: Signature {
                    parameters: parameters.iter().map(|(_, ty)| ty.clone()).collect(),
                    variadic: None,
                    returns,
                },
                parameters: parameters.iter().map(|(name, _)| (*name).into()).collect(),
                body: og_story::yarn::parse_expression(body)?,
            },
        );
    }
    let result = og_story::yarn::compile_with_functions(&explicit_ids(&source), functions);
    let plan_path = path.with_extension("testplan");
    if plan_path.exists() {
        let mut d = result?;
        // TestBase registers an assertion callback. Observe the checked pure
        // call's result in this harness; production runtime has no test hooks.
        for node in &mut d.nodes {
            for step in &mut node.steps {
                if let og_story::Step::Call(expr) = step {
                    let inner = match &*expr {
                        og_story::Expr::Yarn(inner) => inner.as_ref(),
                        expr => expr,
                    };
                    if matches!(inner, og_story::Expr::Invoke(name, _) if name == "assert") {
                        *step = og_story::Step::Emit("fixture_assert".into(), Some(expr.clone()));
                    }
                }
            }
        }
        let d = Dialogue::decode(&d.encode()?)?;
        let plan = std::fs::read_to_string(plan_path).map_err(|e| e.to_string())?;
        let uninterrupted = execute(d.clone(), &plan, false)?;
        let restored = execute(d, &plan, true)?;
        if uninterrupted != restored {
            return Err("mid-dialogue restore changed output/state".into());
        }
        Ok(uninterrupted)
    } else {
        let expected_nodes: Option<&[&str]> = match name {
            "Basic" | "Compiler" | "RandomOptions" | "Strings" | "SkippedOptions"
            | "AnalysisTest" => Some(&["Start"]),
            "Options" => Some(&["A", "B", "C"]),
            "TaggedLines" => Some(&["Destination", "Start"]),
            "Test" if path.to_string_lossy().contains("Projects/Basic/") => {
                Some(&["AnotherTestNode", "TestNode", "ThirdNode"])
            }
            "Sally" => Some(&[
                "Declarations",
                "Sally",
                "Sally_Exit",
                "Sally_Sorry",
                "Sally_Watch",
            ]),
            "Ship" => Some(&["Ship"]),
            "lipsum1" => Some(&["LoadTest"]),
            "lipsum2" => Some(&["LoadTest2"]),
            "lipsum3" => Some(&["LoadTest3"]),
            "Headers" => Some(&[
                "ArbitraryHeaderWithValue",
                "Comments",
                "EmptyTags",
                "HeaderWith.Comments",
                "LotsOfHeaders",
                "SingleTagOnly",
                "Tags",
            ]),
            _ => None,
        };
        if let Some(expected) = expected_nodes {
            let d = result?;
            let encoded = d.encode()?;
            if Dialogue::decode(&encoded)? != d {
                return Err("compiler IR roundtrip changed fixture".into());
            }
            if d.headers.keys().map(|s| s.as_str()).collect::<Vec<_>>() != expected {
                return Err(format!(
                    "expected source nodes {expected:?}, got {:?}",
                    d.headers.keys()
                ));
            }
            if name == "Headers" {
                for (node, key, value) in [
                    ("Tags", "tags", "one two three"),
                    ("EmptyTags", "tags", ""),
                    (
                        "ArbitraryHeaderWithValue",
                        "arbitraryheader",
                        "some-arbitrary-text",
                    ),
                    ("LotsOfHeaders", "headers", ""),
                    ("LotsOfHeaders", "contains", "lots"),
                    ("HeaderWith.Comments", "also", "/"),
                    ("HeaderWith.Comments", "another", "header"),
                ] {
                    if d.headers[node].get(key).map(|s| s.as_str()) != Some(value) {
                        return Err(format!("wrong header {node}.{key}"));
                    }
                }
                if d.file_tags != ["file_header"] {
                    return Err("missing file tag".into());
                }
            }
            if name == "TaggedLines"
                && d.strings.keys().map(|s| s.as_str()).collect::<Vec<_>>()
                    != ["test1", "test2", "test3", "test4"]
            {
                return Err("explicit string IDs changed".into());
            }
            return Ok(encoded.into_bytes());
        }
        // A failure caused by an unsupported feature is not proof of the
        // expected negative diagnostic. Match the rule each fixture violates.
        let expected = match path.file_stem().and_then(|s| s.to_str()).unwrap_or("") {
            "DuplicateLineTags" => Some("duplicate #line:"),
            "Declarations-MustHaveValues" => Some("assignment needs"),
            "Enums-CannotBeComparedAcrossTypes" => {
                Some("expected Enum(\"Food\"), found Enum(\"DayOfWeek\")")
            }
            "Enums-FunctionsAcceptingStringCannotAcceptNumberEnums" => {
                Some("expected String, found Enum")
            }
            "Enums-MemberReferenceWithoutTypesMustBeResolvable" => {
                Some("unknown enum member .Four")
            }
            "Enums-MustHaveConstantRawValues" | "Enums-RawValuesCannotBeEnums" => {
                Some("raw values must be constant")
            }
            "Enums-MustHaveUniqueCases" => Some("cases need unique names"),
            "Enums-MustHaveUniqueRawValues" => Some("cases must have unique raw values"),
            "Enums-MustNotBeEmpty" => Some("enum must have cases"),
            "Enums-RawValuesMustAllBeOfSameType" => Some("raw values must have the same type"),
            "Enums-StringRawValuesMustAllBeDefined" => Some("all enum cases must have raw values"),
            "Inference-MemberReferencesMustBeUnambiguous" => {
                Some("cannot resolve enum member .One")
            }
            "Jumps-ExpressionsMustBeStrings" => Some("expected String, found Number"),
            "Nodes-MustHaveNodeBody" => Some("node needs ---"),
            "Nodes-MustHaveUniqueNames" | "InvalidNodeTitle" => {
                Some("invalid or duplicate node title")
            }
            "Operators-AdditionsRequireNumbersOrStrings" => {
                Some("addition needs two numbers or two strings")
            }
            "SetStatements-MustHaveValues" => Some("expression ends early"),
            "IncorrectIndentation-IndentedLinesFollowingOptionsMustHaveContent" => {
                Some("indented lines following options must have content")
            }
            "Inference-FunctionsCannotChangeType" => Some("expected Bool, found Number"),
            "Inference-FunctionsMustHaveSameNumberOfParams" => Some("expects 1 arguments, got 2"),
            "Inference-FunctionsAndVarsCannotBeSolelyImplicit" => {
                Some("must be registered with a typed signature before cook")
            }
            "Commands-NewlinesNotPermittedInCommands" => {
                Some("unsupported or malformed Yarn statement")
            }
            "Nodes-MustHaveUniqueNameAcrossNodesAndGroups"
            | "Nodes-NodeGroupsMustAllHaveWhenClauses" => {
                Some("all nodes sharing a title must have when headers")
            }
            "Nodes-MustHaveUniqueSubtitles" => Some("invalid or duplicate node title or subtitle"),
            "Notes-WhenHeadersMustHaveExpressions" => Some("when header needs an expression"),
            "OptionConditions-MustHaveExpressions" => Some("unsupported Yarn line condition"),
            "ShadowLines-MustBeIdenticalToSourceLines" => Some("shadow lines must be identical"),
            "ShadowLines-MustHaveValidSourceLine" => {
                Some("shadow line source #line:abc123 does not exist")
            }
            "ShadowLines-MustNotHaveExpressions" => Some("shadow lines must not have expressions"),
            "SmartVariables-MustNotContainLoops" => Some("smart variable cycle"),
            "Variables-CannotBeAssignedConflictingTypes" => Some("expected Bool, found Number"),
            "Variables-MustBeAbleToInferDefinition" => Some("cannot infer type of $myVar"),
            _ => None,
        };
        let error = result.err().unwrap_or_else(|| "accepted".into());
        match expected {
            Some(expected) if error.contains(expected) => Ok(error.into_bytes()),
            Some(expected) => Err(format!("expected diagnostic '{expected}', got: {error}")),
            None => Err(format!(
                "fixture-specific compiler assertion not implemented; compiler: {error}"
            )),
        }
    }
}

#[derive(serde::Deserialize)]
struct MarkupCase {
    input: String,
    locale: String,
    error: bool,
    text: String,
    spans: Vec<og_story::yarn::markup::Attribute>,
}

fn scalar_offset(text: &str, utf16: u32) -> Result<u32, String> {
    let mut units = 0;
    for (scalar, c) in text.chars().enumerate() {
        if units == utf16 {
            return Ok(scalar as u32);
        }
        units += c.len_utf16() as u32;
    }
    if units == utf16 {
        Ok(text.chars().count() as u32)
    } else {
        Err(format!("invalid upstream UTF-16 offset {utf16}"))
    }
}

fn markup_case(case: MarkupCase) -> Result<Vec<u8>, String> {
    let actual = og_story::yarn::markup::parse(&case.input, &case.locale);
    if case.error {
        return if actual.is_err() {
            Ok(b"diagnostic".to_vec())
        } else {
            Err(format!("expected markup diagnostic for {:?}", case.input))
        };
    }
    let mut actual = actual?;
    let mut expected = og_story::yarn::markup::Markup {
        text: case.text,
        spans: case.spans,
    };
    for span in &mut expected.spans {
        // LineParser's tree-splitting implementation leaks its join key after
        // SquishSplitAttributes. It is not a source attribute or property.
        span.properties.remove("_internalIncrementingProperty");
        let end = scalar_offset(&expected.text, span.position + span.length)?;
        span.position = scalar_offset(&expected.text, span.position)?;
        span.length = end - span.position;
    }
    for spans in [&mut actual.spans, &mut expected.spans] {
        spans.sort_by(|a, b| (a.position, &a.name, a.length).cmp(&(b.position, &b.name, b.length)));
    }
    if actual != expected {
        return Err(format!(
            "{:?}: expected {expected:?}, got {actual:?}",
            case.input
        ));
    }
    serde_json::to_vec(&actual).map_err(|e| e.to_string())
}

fn main() {
    let root = Path::new("third_party/yarn-spinner/upstream/Tests");
    let mut files = Vec::new();
    inventory(root, &mut files).unwrap();
    files.sort();
    let mut passed = 0;
    let mut failed = 0;
    for file in files {
        let name = file.strip_prefix(root).unwrap().display().to_string();
        match run(&file) {
            Ok(bytes) => {
                let hash = bytes.iter().fold(0xcbf29ce484222325_u64, |h, b| {
                    (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
                });
                println!("PASS\t{name}\t{hash:016x}");
                passed += 1;
            }
            Err(error) => {
                println!("FAIL\t{name}\t{}", error.replace('\n', " "));
                failed += 1;
            }
        }
    }
    let cases =
        std::fs::read_to_string("third_party/yarn-spinner/markup-expectations.jsonl").unwrap();
    for (index, line) in cases.lines().enumerate() {
        let case: MarkupCase = serde_json::from_str(line).unwrap();
        match markup_case(case) {
            Ok(bytes) => {
                let hash = bytes.iter().fold(0xcbf29ce484222325_u64, |h, b| {
                    (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
                });
                println!("PASS\tmarkup/{index}\t{hash:016x}");
                passed += 1;
            }
            Err(error) => {
                println!("FAIL\tmarkup/{index}\t{}", error.replace('\n', " "));
                failed += 1;
            }
        }
    }
    let cases =
        std::fs::read_to_string("third_party/yarn-spinner/format-expectations.jsonl").unwrap();
    for (index, line) in cases.lines().enumerate() {
        let case: serde_json::Value = serde_json::from_str(line).unwrap();
        let value = case["value"].as_f64().unwrap();
        let spec = case["spec"].as_str().unwrap();
        let actual = og_story::yarn::functions::pure(
            "format",
            &[Val::Str(format!("{{0:{spec}}}")), Val::Num(value)],
        );
        let expected = case["expected"].as_str().map(|s| Some(Val::Str(s.into())));
        let matches = match (&actual, &expected) {
            (Ok(actual), Some(expected)) => actual == expected,
            (Err(_), None) => true,
            _ => false,
        };
        if matches {
            let bytes = format!("{actual:?}");
            let hash = bytes.bytes().fold(0xcbf29ce484222325_u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
            });
            println!("PASS\tformat/{index}\t{hash:016x}");
            passed += 1;
        } else {
            println!("FAIL\tformat/{index}\t{value}:{spec}: expected {expected:?}, got {actual:?}");
            failed += 1;
        }
    }
    println!("TOTAL\tpass={passed}\tfail={failed}\tskip=0");
    if failed != 0 {
        std::process::exit(1);
    }
}
