# Hermes

The desktop shell (`crates/og-host-c/hermes/shell.cpp`) runs on Hermes
(https://github.com/facebook/hermes, tag `hermes-v260318099.0.8`, MIT,
`LICENSE` here), built with `detach-array-buffer.patch`.

The patch adds `facebook::hermes::IHermesArrayBufferDetach`, an optional JSI
interface (reached with `jsi::castInterface`, as `ISetFatalHandler` is) whose
`detachArrayBuffer` runs ECMAScript's DetachArrayBuffer on a `jsi::ArrayBuffer`.
The shell hands JavaScript ArrayBuffers over engine column memory and must
detach them before a column moves; Hermes offers that only as
`HermesInternal.detachArrayBuffer`, behind its test methods, which change
language semantics and stay off in a store build. Offered upstream; drop the
patch once Hermes has the interface.
