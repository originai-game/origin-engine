<meta name="component" content="RpgLootFeed">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgLootFeed.frame RpgLootFeed.entry">
<meta name="prop:items" content="list" data-doc="What the player just looted, newest first, keyed rows of { id, name, icon, glyph, rarity, label, quantity, currency, fading }: icon a texture (0: the glyph, an icon name, in its place); rarity common, uncommon, rare, epic or legendary (the edge's colour, on data-rarity; an epic or legendary item's name is inked too); label the rarity as the game words it (Epic) on a tag, empty for none; quantity the count gained (0 hides it), written ×3, or +1,240 when currency is true (gold); fading an old row on its way out (dimmed). A new row slides in from the screen's edge; the game trims old rows.">
<meta name="fixture:default" content='{"items":[{"id":"blade","name":"Ember Blade","icon":0,"glyph":"sword","rarity":"epic","label":"Epic","quantity":0,"currency":false,"fading":false},{"id":"gold","name":"Gold","icon":0,"glyph":"coin","rarity":"legendary","label":"","quantity":1240,"currency":true,"fading":false},{"id":"shard","name":"Rune shard","icon":0,"glyph":"gem","rarity":"common","label":"","quantity":3,"currency":false,"fading":true}]}'>
<meta name="fixture:rare" content='{"items":[{"id":"ring","name":"Tidewatch Ring","icon":0,"glyph":"gem","rarity":"rare","label":"Rare","quantity":0,"currency":false,"fading":false}]}'>
<meta name="fixture:empty" content='{"items":[]}'>
<meta name="fixture:overflow" content='{"items":[{"id":"long","name":"Greatsword of the Ninefold Gate, Forged in the Ashen Vale","icon":0,"glyph":"sword","rarity":"legendary","label":"Legendary artefact","quantity":123456,"currency":false,"fading":false}]}'>

<!--
  The loot feed low on the screen's side: a column of RpgLootEntry blocks,
  newest first, each an icon, the item's name, its rarity on a tag and the
  count gained on a dark glass with its edge tinted by rarity. The feed
  slides a new row in, fades an old one out and trims on a phone; a game
  that wants another feed lays RpgLootEntry out itself. Read only.
-->
<section id="RpgLootFeed" part="RpgLootFeed.frame" class="hud bare" data-layout="stack" role="log" aria-label={t("rpg.lootfeed.name")}>
  {#each items as p (p.id)}
  <div part="RpgLootFeed.entry" class="entry" data-layout="row" data-rarity={p.rarity} data-fading={p.fading} in:fly={{ x: -24, duration: 200 }} out:fade={{ duration: 300 }} animate:flip={{ duration: 200 }}><RpgLootEntry name={p.name} icon={p.icon} glyph={p.glyph} rarity={p.rarity} label={p.label} quantity={p.quantity} currency={p.currency} fading={p.fading} /></div>
  {/each}
</section>

<style>
:host { align-items: flex-start; row-gap: calc(5 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); }
.entry { flex: none; max-inline-size: 100%; min-inline-size: 0; }
@media (width < 600px) { .entry:nth-child(n + 2) { display: none; } }
</style>
