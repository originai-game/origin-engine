# Yarn Spinner syntax reference

The engine uses its own Rust compiler, IR, and runtime. Language, markup,
standard-library and saliency semantics follow YarnSpinnerTool/YarnSpinner
v3.2.2. Upstream is MIT licensed; the license was checked before porting
semantics or test inputs and is preserved here.

Sources:
- https://github.com/YarnSpinnerTool/YarnSpinner
- https://docs.yarnspinner.dev/write-yarn-scripts/scripting-fundamentals/flow-control

The declared profile is `yarn-spinner-3.2.2`. Engine extensions require explicit
line ids and keep quests, bark scheduling, and deterministic state engine-owned.

## Full-language conformance work

The unchanged `.yarn` sources and `.testplan` expectations in `upstream/Tests`
come from YarnSpinnerTool/YarnSpinner **v3.2.2**, commit
`5b3a4ff2d24e4f727e3f90fee5d8ce637474c305` (retrieved 2026-11-07):
83 `.yarn` files and 33 `.testplan` files. They use the MIT license above.
A fixture without a test plan is not automatically a negative test:
the upstream C# test that consumes it defines its expected result.

`crates/og-story/examples/yarn_conformance.rs` runs all 83 sources. All 33 test
plans check output both uninterrupted and with restores at suspensions. Compiler
fixtures check source nodes, metadata, string ids, IR roundtrips or their expected
negative diagnostic. The fixture adapter adds required line ids in memory only.

`markup-expectations.jsonl` has 99 cases evaluated by the released C# LineParser:
unique string inputs from MarkupTests' InlineData and RangeComparisons, plus an
emoji before a typed span. These run through the default parser, without the
test class's custom HTML marker processors. `format-expectations.jsonl` has 630
invariant .NET Single formatting cases, including standard and custom patterns.
The original generators are in `reference/`; expectations never come from the
Rust implementation. Only upstream's leaked `_internalIncrementingProperty`
tree-join bookkeeping is removed by the comparison, not authored properties.

The engine's text ranges use Unicode scalar indices (`og-ui/src/text.rs`
`VisualGlyph::index`, `og-ui/src/edit.rs` `wrap`), not UTF-16 code units.
Yarn markup spans must use those same scalar indices. Upstream UTF-16 test
expectations must be converted in the test harness, never exposed as a second
engine range unit.

## Reproduce the comparison

From the repository root, with the repository toolchain and Node installed:

```sh
cargo run -p og-story --example yarn_conformance > native.tsv
cargo build -p og-story --example yarn_conformance --target wasm32-wasip1 --profile wasm
node scripts/yarn-conformance.mjs > wasm.tsv
diff -u native.tsv wasm.tsv
CARGO_TARGET_WASM32_WASIP1_RUNNER="node $PWD/scripts/yarn-conformance.mjs" \
  cargo test -p og-story -p og-i18n --target wasm32-wasip1 --profile wasm -- --test-threads=1
```

Each case prints PASS or FAIL and a content/state hash. Totals include failures
and skips. WASI runs the same Rust story runtime as the WebGPU host without
requiring rendering. Test the public runtime's locale/span integration with
`cargo test -p og-runtime --test tests yarn::`.

To regenerate the extra expectations, clone upstream at the pinned tag and use
the .NET 10 SDK (the release's own test project targets net10.0):

```sh
dotnet run --project third_party/yarn-spinner/reference/markup \
  -p:YarnRoot=/absolute/path/to/YarnSpinner | grep '^{' > markup-expectations.jsonl
dotnet run --project third_party/yarn-spinner/reference/format \
  | grep '^{' > format-expectations.jsonl
```

## Development main is not this profile

Compared with v3.2.2, upstream main at
`85a0c18d96be5046703f35e90727fe1cb9fce819` removes `call_statement` and
`COMMAND_CALL`, changes test expected output and `format` argument conversion,
adds self-detour tests, async/variadic function work and project-definition
tests. These are not adopted as a new language release here:
[pinned comparison](https://github.com/YarnSpinnerTool/YarnSpinner/compare/v3.2.2...85a0c18d96be5046703f35e90727fe1cb9fce819).

Legacy Yarn 1 `[[label|destination]]` options are absent from the
[released parser](https://github.com/YarnSpinnerTool/YarnSpinner/blob/v3.2.2/YarnSpinner.Compiler/Grammars/YarnSpinnerParser.g4#L40-L57).
Use the released `shortcut_option_statement` (`->`) and `jump_statement`.
`<<call>>` is retained despite its removal on main. No other released-language
syntax is intentionally excluded.
