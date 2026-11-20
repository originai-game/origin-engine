<meta name="component" content="HudLootEntry">
<meta name="strings" content="./loot-entry.en.ogt">
<meta name="parts" content="HudLootEntry.entry HudLootEntry.edge HudLootEntry.art HudLootEntry.item HudLootEntry.rarity HudLootEntry.count">
<meta name="prop:name" content="string" data-doc="The item's name (Ember Blade).">
<meta name="prop:icon" content="number" data-doc="Its art, a texture (og.load.texture); 0 draws the glyph in its place.">
<meta name="prop:glyph" content="string" data-doc="An icon of the atlas, drawn when there is no art; empty is none.">
<meta name="prop:rarity" content="string" data-default="common" data-doc="common, uncommon, rare, epic or legendary: the edge's colour (on data-rarity); an epic or legendary item's name is inked too.">
<meta name="prop:label" content="string" data-doc="The rarity as the game words it (Epic), on a tag; empty is none.">
<meta name="prop:quantity" content="number" data-doc="The count gained, written ×3 (or +1,240 when currency is true); 0 hides it.">
<meta name="prop:currency" content="bool" data-doc="The quantity is money (gold): written as a gain.">
<meta name="prop:fading" content="bool" data-doc="An old row on its way out: dimmed.">
<meta name="fixture:default" content='{"name":"Ember Blade","icon":0,"glyph":"sword","rarity":"epic","label":"Epic","quantity":0,"currency":false,"fading":false}'>
<meta name="fixture:gold" content='{"name":"Gold","icon":0,"glyph":"coin","rarity":"legendary","label":"","quantity":1240,"currency":true,"fading":false}'>
<meta name="fixture:fading" content='{"name":"Rune shard","icon":0,"glyph":"gem","rarity":"common","label":"","quantity":3,"currency":false,"fading":true}'>
<meta name="fixture:empty" content='{"name":"","icon":0,"glyph":"","rarity":"common","label":"","quantity":0,"currency":false,"fading":false}'>
<meta name="fixture:overflow" content='{"name":"Greatsword of the Ninefold Gate, Forged in the Ashen Vale","icon":0,"glyph":"sword","rarity":"legendary","label":"Legendary artefact","quantity":123456,"currency":false,"fading":false}'>

<!--
  One row of a HUD's loot feed, a block a feed is built from: an
  icon, the item's name, its rarity on a tag and the count gained, on a
  dark glass that fades toward the middle with its edge at the screen's
  side tinted by rarity; a fading row dims. Read only.
-->
<div id="HudLootEntry" part="HudLootEntry.entry" data-layout="row" data-gap="tight" role="group" data-rarity={rarity} data-fading={fading} aria-label={quantity > 0 ? t("hud.loot-entry.row", { name: name, quantity: currency ? t("hud.loot-entry.gain", { quantity: formatNumber(quantity) }) : t("hud.loot-entry.count", { quantity: formatNumber(quantity) }) }) : name}>
  <div part="HudLootEntry.edge" class="edge"></div>
  <div part="HudLootEntry.art" class="art" data-layout="overlay">
    {#if icon}<img class="picture" data-anchor="fill" src={icon} alt="">{:else}{#if glyph}<og-icon class="glyph" data-anchor="fill" icon={glyph}></og-icon>{/if}{/if}
  </div>
  <span part="HudLootEntry.item" class="item">{name}</span>
  {#if label}<span part="HudLootEntry.rarity" class="tag">{label}</span>{/if}
  {#if quantity > 0}<span part="HudLootEntry.count" class="count">{currency ? t("hud.loot-entry.gain", { quantity: formatNumber(quantity) }) : t("hud.loot-entry.count", { quantity: formatNumber(quantity) })}</span>{/if}
</div>

<style>
:host { inline-size: fit-content; align-items: center; max-inline-size: 100%; min-inline-size: 0; padding-inline-end: var(--space-md); color: var(--hud-ink); background: var(--hud-feed-fill-start); --layout-anchor-inset: 0; }
:host(:dir(rtl)) { background: var(--hud-feed-fill); }
[data-fading=true] { opacity: 0.55; }
/* The edge at the screen's side, tinted by rarity. */
.edge { flex: none; align-self: stretch; inline-size: var(--hud-feed-edge-width); margin-inline-end: var(--space-xs); background: var(--hud-feed-edge); }
[data-rarity=uncommon] .edge { background: var(--hud-boon); }
[data-rarity=rare] .edge { background: var(--hud-rare); }
[data-rarity=epic] .edge { background: var(--hud-epic); }
[data-rarity=legendary] .edge { background: var(--hud-legendary); }
.art { flex: none; margin-block: calc(3 * var(--font-step-px)); inline-size: calc(20 * var(--font-step-px)); block-size: calc(20 * var(--font-step-px)); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.picture { object-fit: cover; }
.glyph { color: var(--hud-ink); }
.item { min-inline-size: 0; font-size: var(--hud-size-feed); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); }
[data-rarity=epic] .item { color: var(--hud-epic-ink); }
[data-rarity=legendary] .item { color: var(--hud-legendary); }
.tag { flex: 0 1 auto; min-inline-size: 0; padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.count { flex: none; font-size: var(--hud-size-feed); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; color: var(--hud-ink-dim); text-shadow: var(--hud-shadow); }
[data-rarity=legendary] .count { color: var(--hud-legendary); }
@media (width < 600px), (height < 500px) {
  .art { inline-size: calc(18 * var(--font-step-px)); block-size: calc(18 * var(--font-step-px)); }
  .item, .count { font-size: var(--hud-size-xs); }
}
</style>
