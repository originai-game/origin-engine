# Yarn Spinner syntax reference

The engine's Rust compiler is an original implementation, not a port of the
C# runtime. Its syntax was checked against YarnSpinnerTool/YarnSpinner's
`YarnSpinner.Compiler/Grammars/YarnSpinnerParser.g4` and the Yarn Spinner 3
flow-control documentation on 2026-11-07. Upstream is MIT licensed; the
license was checked before implementation and is preserved here.

Sources:
- https://github.com/YarnSpinnerTool/YarnSpinner
- https://docs.yarnspinner.dev/write-yarn-scripts/scripting-fundamentals/flow-control

`yarn-3.0-og-1` is an explicitly bounded engine profile, not a claim of full
Yarn Spinner 3 compatibility. Unsupported constructs fail compilation.
