# The built-in UI library

Svelte documents built into the engine: `controls/` (each HTML control's
inside), `components/` (HUD, feedback and pattern pieces, and the genre
groups) and `screens/`. og-ui provides the capabilities they are written
on; none of their look is drawn in Rust. docs/ui.md §1 and §2.4 hold the
contract, `components/README.md` and `screens/README.md` the catalogues.

## Control documents

A control document is the inside of the element its
`<meta name="control">` names, instantiated under every such element. A
game writes the element (`<progress value={$done}>`) and never names the
document; the cook bundles it when a document of the game has the
element. `og ui eject controls/<name>` copies one into
`assets/ui/controls/`, and the game's copy is then that element's control
everywhere.

Each one declares the props it reads from its host's attributes, lists
its parts in `<meta name="parts">`, and reads tokens only. A pack dresses
it through the control's hook tokens and `host::part(name)` rules, which
outrank the document's own.

A control that must remember something between frames that the game has
no stake in (its picker being open) declares it as instance state,
`<meta name="state:open" content="bool">`: each host has its own value,
which the document writes with `bind:` and a snapshot carries
(docs/ui.md §2 Components in documents).

| Host element | Document |
|---|---|
| `<progress>`: a track filled to its value | `controls/progress` |
| `<meter>`: a track whose fill wears its value's region | `controls/meter` |

### Parts

| Document | Parts | Props read from the host |
|---|---|---|
| `controls/progress` | `track`, `fill` | `value` |
| `controls/meter` | `track`, `fill` | `value`, `min`, `max`; `region`, derived by og-ui |
