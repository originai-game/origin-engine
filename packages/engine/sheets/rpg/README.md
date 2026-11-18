# RPG review sheet

Run from the repository root, after building `og` and the native addon:

```sh
OG_ENGINE_PACKAGE="$PWD/packages/engine" OG_STYLES_DIR="$PWD/templates/styles" \
  target/debug/og ui preview packages/engine/sheets/rpg \
  --screen 'crafting=style:default;page:crafting' \
  --screen 'trade=style:default;page:trade' \
  --screen 'loot=style:default;page:loot' \
  --screen 'history=style:default;page:dialogue' \
  --breakpoints desktop,phonePortrait --locales game --scales 100
```

Each preview needs its `page` command. Append `;scenario:missing`, `busy`,
`loading`, `empty`, `over`, `overflow`, `sell`, or `playing` to inspect those
states. On the inventory page `empty`, `over` (overburdened) and `overflow`
(long names, huge counts, every corner mark on one tile) swap the backpack.
`sell` is the merchant's other side; `playing` is an active history replay.
`overflow` supplies long loot descriptions and a long history. The sample
game owns all prices, quantity limits, requirements and legality; components
only render the data and return requests with stable row keys.

## New panels

- `RpgCrafting`: keyed `recipes`, `selected`, selected `ingredients`, batch
  `quantity`, `canIncrease`/`canDecrease`, `cost`, `loading`/`busy`/`error`.
  `choose`, `increase`, `decrease` and `craft` carry the recipe key. A
  restricted recipe remains inspectable; only crafting is disabled.
- `RpgTrade`: `mode` (`buy`/`sell`), formatted `wallet`, keyed `goods`,
  `selected`, `loading`/`busy`/`error`. `choose`, `buy` and `sell` carry the
  good's key. `switchMode` asks the game to replace goods and prices. Goods
  may represent batches; the component never computes a price or balance.
- `RpgLoot`: keyed `items`, `canTakeAll`, `loading`/`busy`/`error`. `take`
  carries the item key; `takeAll` and `close` are panel intents. The game
  removes collected rows and decides capacity. Close also responds to Back.
  The default chest demonstrates an all-or-nothing batch: the blocked relic
  disables Take all, but Gold can still be taken individually. For a
  take-what-you-can policy, the game sets `canTakeAll` when at least one row
  is available and collects only permitted rows in response to `takeAll`.
  The component does not infer either policy from the item list.
- `RpgConversationBacklog`: chronological keyed `lines`, `playing`,
  `hasOlder`, `loading`. `replay` and `stop` carry a line key; `older` and
  `close` are panel intents. Audio playback and pagination remain game-owned.

Each document lists its parts, defaults and state fixtures in metadata.
Buttons inherit the default style's hover, press, focus and disabled states;
selection and replay state are exposed through attributes for style packs.

The `dialogue` page also exercises the optional conversation controls:
default enables History/Skip/Auto; `playing` marks Auto pressed, `missing`
disables all three, and `empty` shows narration with Continue. These
buttons emit requests; the game owns history, skip permission and auto
advance. The journal's compact conversation retains the default hidden controls.

`bun test packages/engine/sheets/rpg/tests/components.test.ts` mounts these
components through game documents and exercises keyed actions after row
reordering, unavailable/pending suppression, history replay, and base slot
and avatar states.
