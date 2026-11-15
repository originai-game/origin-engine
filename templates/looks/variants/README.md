# Variants

A look's `variant "<name>"` (docs/art.md §2.11): one preset drawn another
way. `<slug>/<name>.ogt` is a `type look` layer the cook lays over the preset
`<slug>` field by field, before the mood's, when a game's look extending the
preset asks for it; a field the layer turns off is written at its default.
A layer names no `extends`, `variant` or `ui`; its `name` and `mood` replace
the preset's.

Each variant has an entry under its look's `variants` in
`packages/knowledge/looks/catalogue.json` with its own text (title, checks,
intended traits, premise, palette, specs, art bible and authored art), which
the art brief and the critic read in place of the look's. The cook compiles
every file here in (`og-cook`'s `VARIANT_LAYERS`), and the validator holds
the files and the entries equal.
