<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `kits` guide: Compose and install UI kits

Genre kits are being rebuilt. No genre kit is bundled yet. Use core
controls and screen and component documents for a game's structure.
The `materials` extension remains available (ui-materials §1).
`og ui list --json` reports what is actually bundled or installed.

<a id="guide-kits-1"></a>
### `kits §1` Compose a screen

Keep structure in documents, look in CSS and semantic tokens, and data in
named sources. TypeScript does not build UI trees. The retained data API
uses the `svelte/store` model with named stores and keyed collections;
its signatures are pending the core API landing. Consult the generated
API for implemented calls rather than inventing signatures.

Use core controls for buttons, fields, lists, slots, meters, and focus.
Keep domain rules in the game. A kit reports intents with stable item IDs;
the game decides whether to apply them. Store UI state in the screen's
scope and core interaction state, never module globals.

<a id="guide-kits-2"></a>
### `kits §2` Install an available kit

1. Run `og ui list` to find a bundled or installed kit. For a local kit,
   use its directory instead of a bundled name.
2. Run `og ui add <kit-or-directory>` to copy it into
   `assets/ui-kits/<name>/`. An existing copy is not overwritten silently.
3. Edit the installed documents and pass the game's named data inputs
   when mounting them. Keep the component's named parts and states intact
   when changing its layout.
4. To copy one document with its companions, run
   `og ui eject <kit>/<document>`. The copy goes into `assets/ui/<kit>/`.

<a id="guide-kits-3"></a>
### `kits §3` Change the look

Put visual changes in the style's tokens and part selectors. Do not add
literal colours, spacing, font paths, or animation durations to kit code
or documents. Structural ratios and data values are not visual tokens.
Use the core fallback when a part has no custom dressing.

Creation currently offers `ui "default"`, the engine core style without
a pack dependency. A game can author its own style with `og style new`
or install a local style with `og style add` (screens §3).

<a id="guide-kits-4"></a>
### `kits §4` Review a kit

Provide fixtures for empty, populated, overflow, disabled, selected,
focused, loading, and error states, plus the domain's exceptional states.
Test keyboard, gamepad families, touch, live input-prompt switching,
large text, reduced motion, high contrast, and right-to-left text.

Run `og ui sheet <kit>` for a kit with a sheet project. Inspect its pages
and run `og ui check`; measurements do not replace visual review.
The integration contract in docs/ui.md §6 defines the required frame
matrix and the separate art review before landing.

<a id="guide-kits-5"></a>
### `kits §5` Author a kit directory

Declare `format`, `kind: "kit"`, `name`, `version`, and `description` in
`extension.json`, with `entry` and `types` when the kit has public data
helpers, and `components` naming the library component documents its
documents compose (`["stat-bar"]`), which `og ui add` ejects with it. Declare typed inputs, emitted intents, documents, parts, and
state fixtures before implementing them. Use public engine imports only.

`registerUiExtension` returns the kit's namespace; it does not add members
to `og.ui`. TypeScript callbacks are limited to custom drawing and
components a document cannot express, with explicit dependencies.
