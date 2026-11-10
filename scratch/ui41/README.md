# #41 step 3 measuring scripts — NOT FOR LANDING

These are the scratch scripts behind the og-ui field-to-stage plan:
https://ampcode.com/artifacts/og-ui-stage-states-field-to-stage-table-and-commit-plan-41-s-8b755f83-129c-4dbd-918e-43ba39fb82f4

Run them from this directory, against a checkout's og-ui (`OG_UI`, default `crates/og-ui` relative to the cwd).

    export OG_UI=/path/to/checkout/crates/og-ui
    L=/path/to/checkout
    # 1. access sites: each struct's fields are renamed, `cargo check -p og-ui --lib --tests` runs, the file is restored.
    #    Take the line numbers of the `struct` lines from the current source.
    python3 collect.py $L Ui    crates/og-ui/src/ui.rs          <line of `pub struct Ui`>    ui.json
    python3 collect.py $L State crates/og-ui/src/ui.rs          <line of `struct State`>     state.json
    python3 collect.py $L Node  crates/og-ui/src/layout/node.rs <line of `struct Node`>      node.json
    # 2. fns and impls with spans
    python3 index.py $OG_UI index.json
    # 3. sites -> enclosing fn, read/write
    python3 join.py
    # 4. tables
    python3 calls.py                      # stage-to-stage call matrix; writes methods.json
    python3 blocks.py > blocks.txt        # impl Ui blocks -> target; writes blocks.json
    (cat head.md; python3 gen.py ui; cat mid1.md; python3 gen.py state; cat mid2.md; python3 gen.py node; cat mid3.md; python3 gen.py blocks; cat tail.md) > plan.md

`stage.py` maps files, fns and `end_frame` regions (by line, at 66e91368) to stages; update `END_FRAME` and the `FN` overrides after the source moves. `owners.py` is the approved owner table. `collect.py` edits the checkout in place while it runs and restores it; check `git status` afterwards. The read/write split is heuristic; the site counts are exact. The `*.md` pieces are the prose of the published plan at revision 1.
