<meta name="component" content="RpgLootFeed">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgLootFeed.frame RpgLootFeed.entry RpgLootFeed.art RpgLootFeed.item RpgLootFeed.rarity RpgLootFeed.count">
<meta name="prop:items" content="list" data-doc="What the player just looted, newest first, keyed rows of { id, name, icon, glyph, rarity, label, quantity, currency, fading }: icon a texture (0: the glyph, an icon name, in its place); rarity common, uncommon, rare, epic or legendary (the edge's colour, on data-rarity; an epic or legendary item's name is inked too); label the rarity as the game words it (Epic) on a tag, empty for none; quantity the count gained (0 hides it), written ×3, or +1,240 when currency is true (gold); fading an old row on its way out (dimmed). A new row slides in from the screen's edge; the game trims old rows.">
<meta name="fixture:default" content='{"items":[{"id":"blade","name":"Ember Blade","icon":0,"glyph":"sword","rarity":"epic","label":"Epic","quantity":0,"currency":false,"fading":false},{"id":"gold","name":"Gold","icon":0,"glyph":"coin","rarity":"legendary","label":"","quantity":1240,"currency":true,"fading":false},{"id":"shard","name":"Rune shard","icon":0,"glyph":"gem","rarity":"common","label":"","quantity":3,"currency":false,"fading":true}]}'>
<meta name="fixture:rare" content='{"items":[{"id":"ring","name":"Tidewatch Ring","icon":0,"glyph":"gem","rarity":"rare","label":"Rare","quantity":0,"currency":false,"fading":false}]}'>
<meta name="fixture:empty" content='{"items":[]}'>
<meta name="fixture:overflow" content='{"items":[{"id":"long","name":"Greatsword of the Ninefold Gate, Forged in the Ashen Vale","icon":0,"glyph":"sword","rarity":"legendary","label":"Legendary artefact","quantity":123456,"currency":false,"fading":false}]}'>

<!--
  The loot feed low on the screen's side: each row an icon, the item's
  name, its rarity on a tag and the count gained, on a dark glass that
  fades toward the middle with its edge at the screen's side tinted by
  rarity; older rows fade. Read only.
-->
<section id="RpgLootFeed" part="RpgLootFeed.frame" class="hud bare" data-layout="stack" role="log" aria-label={t("rpg.lootfeed.name")}>
  {#each items as p (p.id)}
  <div part="RpgLootFeed.entry" class="entry" data-layout="row" data-gap="tight" data-rarity={p.rarity} data-fading={p.fading} aria-label={p.quantity > 0 ? t("rpg.lootfeed.row", { name: p.name, quantity: p.currency ? t("rpg.lootfeed.gain", { quantity: formatNumber(p.quantity) }) : t("rpg.lootfeed.count", { quantity: formatNumber(p.quantity) }) }) : p.name} in:fly={{ x: -24, duration: 200 }} out:fade={{ duration: 300 }} animate:flip={{ duration: 200 }}>
    <div class="edge"></div>
    <div part="RpgLootFeed.art" class="art" data-layout="overlay">
      {#if p.icon}<img class="picture" data-anchor="fill" src={p.icon} alt="">{:else}{#if p.glyph != ''}<og-icon class="glyph" data-anchor="fill" icon={p.glyph}></og-icon>{/if}{/if}
    </div>
    <span part="RpgLootFeed.item" class="item">{p.name}</span>
    {#if p.label != ''}<span part="RpgLootFeed.rarity" class="tag">{p.label}</span>{/if}
    {#if p.quantity > 0}<span part="RpgLootFeed.count" class="count">{p.currency ? t("rpg.lootfeed.gain", { quantity: formatNumber(p.quantity) }) : t("rpg.lootfeed.count", { quantity: formatNumber(p.quantity) })}</span>{/if}
  </div>
  {/each}
</section>

<style>
:host { align-items: flex-start; row-gap: calc(5 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); --layout-anchor-inset: 0; }
.entry { flex: none; align-items: center; max-inline-size: 100%; min-inline-size: 0; padding-inline-end: var(--space-md); background: var(--hud-feed-fill-start); }
.entry:dir(rtl) { background: var(--hud-feed-fill); }
.entry[data-fading=true] { opacity: 0.55; }
/* The edge at the screen's side, tinted by rarity. */
.edge { flex: none; align-self: stretch; inline-size: var(--hud-feed-edge-width); margin-inline-end: var(--space-xs); background: var(--hud-feed-edge); }
[data-rarity=uncommon] .edge { background: var(--hud-boon); }
[data-rarity=rare] .edge { background: var(--hud-rare); }
[data-rarity=epic] .edge { background: var(--hud-epic); }
[data-rarity=legendary] .edge { background: var(--hud-legendary); }
.art { flex: none; margin-block: calc(3 * var(--font-step-px)); inline-size: calc(20 * var(--font-step-px)); block-size: calc(20 * var(--font-step-px)); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.picture { object-fit: cover; }
.glyph { color: var(--hud-ink); }
.item { min-inline-size: 0; font-size: var(--hud-size-feed); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); }
[data-rarity=epic] .item { color: var(--hud-epic-ink); }
[data-rarity=legendary] .item { color: var(--hud-legendary); }
.tag { flex: 0 1 auto; min-inline-size: 0; padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.count { flex: none; font-size: var(--hud-size-feed); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; color: var(--hud-ink-dim); text-shadow: var(--hud-shadow); }
[data-rarity=legendary] .count { color: var(--hud-legendary); }
@media (width < 600px) {
  .entry:nth-child(n + 2) { display: none; }
  .art { inline-size: calc(18 * var(--font-step-px)); block-size: calc(18 * var(--font-step-px)); }
  .item, .count { font-size: var(--hud-size-xs); }
}
</style>
