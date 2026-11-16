# fantasy-frost: a token-only variant

`frost.css` sets `:root` tokens and nothing else: steel rims instead of
brass (`--line-strong`, `--frame-rim`, `--cell-rim`), frost-blue
washes and focus (`--accent`, `--focus`, `--gold`), a blue primary and band,
blue-black grounds. It selects no part.

It proves that `templates/styles/fantasy` restyles by tokens alone: a pack
made of fantasy's files with `frost.css` beside them (it sorts after
`base.css`, so its tokens win) renders every control and component in the
new colours, and fantasy's art still resolves against fantasy's `art/`
because a `url()` held in a token resolves against the file that declares it (`crates/og-ui/tests/style_pack.rs`).

To render it, copy `templates/styles/fantasy` to a pack directory, add
`frost.css`, and run `og ui sheet` (or any group's
review sheet) on it. It is not shipped: `tests/` is skipped by the cook.
