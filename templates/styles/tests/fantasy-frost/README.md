# fantasy-frost: a token-only variant

`frost.css` sets `:root` tokens and nothing else: steel rims instead of
brass (`--line-strong`, `--fantasy-brass-*`, `--frame-rim`), frost-blue
washes and focus (`--accent`, `--focus`, `--gold`), a blue primary and band,
blue-black grounds. It selects no part.

It proves that `templates/styles/fantasy` restyles by tokens alone: a pack
made of fantasy's files with `frost.css` beside them (it sorts after
`base.css`, so its tokens win) renders every core control and rpg component
in the new colours with fantasy's rules unchanged, and its art still
resolves against fantasy's `art/` because a `url()` held in a token resolves
against the file that declares it (`crates/og-ui/tests/style_pack.rs`).

To render it, copy `templates/styles/fantasy` to a pack directory, add
`frost.css`, and run `og style check` and `og ui sheet` (or the rpg sheet)
on it. It is not shipped: `tests/` is skipped by the cook.
