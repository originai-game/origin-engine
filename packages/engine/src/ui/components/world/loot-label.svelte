<meta name="component" content="LootLabel">
<meta name="strings" content="./world.en.ogt">
<meta name="parts" content="LootLabel.frame LootLabel.glow LootLabel.chip LootLabel.strip LootLabel.name LootLabel.count LootLabel.key LootLabel.tier LootLabel.beam">
<meta name="prop:entity" content="number" data-doc="The item, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="A world point it lies at without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above it the label stands.">
<meta name="prop:name" content="string" data-doc="The item's name, in its tier's ink; a long one scrolls on one line.">
<meta name="prop:count" content="number" data-doc="A stack's size, shown as ×n from 2.">
<meta name="prop:rarity" content="string" data-doc="Its tier: common (empty), uncommon, rare, epic or legendary: the strip on the chip's leading edge, the name's ink and the glow on the ground; epic and legendary raise a beam, a legendary wears a gold wash and rim and shimmers every 2.4 s.">
<meta name="prop:hovered" content="bool" data-doc="Aimed at or under the pointer (the chip takes :hover too): it lifts 1.06 times, its rim goes all round and the pick-up key shows.">
<meta name="prop:action" content="string" data-doc="The input action whose bound key shows on a hovered chip (og.input.defineAction); empty shows none.">
<meta name="prop:more" content="number" data-doc="A chip standing for this many more items the game folded into one (+n), with a dot per tier in tiers; 0 is an item's own chip.">
<meta name="prop:tiers" content="list" data-doc="The folded items' tiers, keyed rows of { id, rarity }, a dot each on a +n chip.">
<meta name="action:pick" content="Pick up">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":0,"name":"Ember Blade","count":1,"rarity":"epic","hovered":false,"action":"","more":0,"tiers":[]}'>
<meta name="fixture:common" content='{"name":"Rusted Buckle","rarity":""}'>
<meta name="fixture:stack" content='{"name":"Gold","count":214,"rarity":""}'>
<meta name="fixture:uncommon" content='{"name":"Wolfhide Bracers","rarity":"uncommon"}'>
<meta name="fixture:rare" content='{"name":"Stormglass Pendant","rarity":"rare"}'>
<meta name="fixture:legendary" content='{"name":"Crown of the Ninefold King","rarity":"legendary"}'>
<meta name="fixture:hovered" content='{"hovered":true,"action":"pick"}'>
<meta name="fixture:more" content='{"name":"","more":6,"rarity":"","tiers":[{"id":"a","rarity":"rare"},{"id":"b","rarity":"uncommon"},{"id":"c","rarity":"uncommon"},{"id":"d","rarity":""}]}'>
<meta name="fixture:overflow" content='{"name":"Blade of the Ninefold King’s Last Oath, Reforged in Ember","count":1250,"rarity":"legendary"}'>

<!--
  An item on the ground: a glow of its tier where it lies and, over it, a
  dark chip with the tier's strip on its leading edge and the name in the
  tier's ink (a stack's count after it); epic and legendary raise a beam.
  Reports pick (the chip); the game owns what is picked up.
-->
<div id="LootLabel" part="LootLabel.frame" class="bare" data-rarity={more > 0 ? 'common' : (rarity == '' ? 'common' : rarity)}>
  {#if more == 0}<ground class="pool" entity={entity || null} at={at} size="1.4 0.5"><div part="LootLabel.glow" class="glow"></div></ground>{/if}
  <anchor id="Label" entity={entity || null} at={at} offset={"0 " + height + " 0"} declutter="shift" avoid="danger" max-shift="128" occlusion="fade" priority={rarity == 'legendary' ? 50 : (rarity == 'epic' ? 40 : (rarity == 'rare' ? 30 : (rarity == 'uncommon' ? 20 : 10)))} max-distance="30">
    <div class="column">
      <button part="LootLabel.chip" class="chip" placement-box data-hovered={hovered} data-more={more > 0} aria-label={more > 0 ? t("world.loot.moreName", { n: more }) : name} onclick={pick}><div class="face">
        <span part="LootLabel.strip" class="strip"></span>
        {#if more > 0}
          <span part="LootLabel.count" class="more">{t("world.loot.more", { n: more })}</span>
          {#each tiers as d (d.id)}<span part="LootLabel.tier" class="dot" data-rarity={!d.rarity ? 'common' : d.rarity}></span>{/each}
        {:else}
          <span part="LootLabel.name" class="name">{name}</span>
          {#if count > 1}<span part="LootLabel.count" class="count">{t("world.loot.count", { count: count })}</span>{/if}
          {#if hovered && action != ''}<kbd part="LootLabel.key" class="key" data-action={action}></kbd>{/if}
          {#if rarity == 'legendary'}<span class="sheen"></span>{/if}
        {/if}
      </div></button>
      {#if more == 0 && (rarity == 'epic' || rarity == 'legendary')}<div part="LootLabel.beam" class="beam"></div>{/if}
    </div>
  </anchor>
</div>

<style>
/* The glow where it lies, on the ground: the ground's box is the glow's,
   the world size it is laid at. */
.pool { inline-size: var(--world-glow-width); block-size: var(--world-glow-height); }
.glow { inline-size: var(--world-glow-width); block-size: var(--world-glow-height); opacity: var(--world-glow-opacity); background: var(--world-common-glow); }
[data-rarity=uncommon] .glow { background: var(--world-uncommon-glow); }
[data-rarity=rare] .glow { background: var(--world-rare-glow); }
[data-rarity=epic] .glow { background: var(--world-epic-glow); }
[data-rarity=legendary] .glow { background: var(--world-legendary-glow); }
.column { display: flex; flex-direction: column; align-items: center; }
/* The chip: dark glass, the tier's strip, the name in the tier's ink; it
   pops in. A text box: it grows with the text scale. */
.chip { flex: none; min-inline-size: 0; max-inline-size: stretch; color: var(--world-common-ink); animation: var(--world-loot-motion); }
.chip::part(base) { padding: 0; border: var(--world-loot-border); border-radius: 0; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.face { position: relative; display: flex; align-items: center; column-gap: calc(6 * var(--font-step-px)); min-inline-size: 0; padding: calc(3 * var(--font-step-px)) calc(9 * var(--font-step-px)) calc(3 * var(--font-step-px)) calc(11 * var(--font-step-px)); overflow: hidden; font-size: var(--world-size-loot); line-height: 1.2; font-weight: var(--hud-label-weight); }
.strip { position: absolute; left: 0; top: 0; bottom: 0; inline-size: var(--world-loot-strip); background: var(--world-common); }
[data-rarity=uncommon] .strip { background: var(--world-uncommon); }
[data-rarity=rare] .strip { background: var(--hud-rare); }
[data-rarity=epic] .strip { background: var(--hud-epic); }
[data-rarity=legendary] .strip { background: var(--hud-legendary); }
.name { flex: 0 1 auto; min-inline-size: 0; max-inline-size: calc(190 * var(--font-step-px)); color: var(--world-common-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
[data-rarity=uncommon] .name { color: var(--world-uncommon); }
[data-rarity=rare] .name { color: var(--hud-rare); }
[data-rarity=epic] .name { color: var(--hud-epic-ink); }
[data-rarity=legendary] .name { color: var(--hud-legendary); font-weight: var(--hud-figure-weight); }
.count, .more { flex: none; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); white-space: nowrap; }
.dot { flex: none; inline-size: var(--world-dot-size); block-size: var(--world-dot-size); rotate: 45deg; background: var(--world-common-ink); }
.dot[data-rarity=uncommon] { background: var(--world-uncommon); }
.dot[data-rarity=rare] { background: var(--hud-rare); }
.dot[data-rarity=epic] { background: var(--hud-epic); }
.dot[data-rarity=legendary] { background: var(--hud-legendary); }
.key { flex: none; min-inline-size: calc(18 * var(--font-step-px)); block-size: calc(17 * var(--font-step-px)); padding: 0 calc(4 * var(--font-step-px)); font-size: var(--world-size-level); font-weight: var(--hud-figure-weight); color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); }
/* A legendary: a gold wash, a gold rim all round, a shimmer. */
[data-rarity=legendary] .chip::part(base) { border-color: var(--hud-legendary); background: var(--world-legendary-fill); }
.sheen { position: absolute; top: 0; bottom: 0; left: 0; inline-size: 30%; background: var(--world-sheen); animation: var(--world-shimmer-motion); }
/* Hovered: it lifts, fills, its rim goes all round. */
.chip:hover, .chip[data-hovered=true] { scale: var(--world-hover-scale); }
.chip:hover::part(base), .chip[data-hovered=true]::part(base) { border-color: var(--world-common-ink); background: var(--world-loot-hover-fill); }
[data-rarity=uncommon] .chip:hover::part(base), [data-rarity=uncommon] .chip[data-hovered=true]::part(base) { border-color: var(--world-uncommon); }
[data-rarity=rare] .chip:hover::part(base), [data-rarity=rare] .chip[data-hovered=true]::part(base) { border-color: var(--hud-rare); }
[data-rarity=epic] .chip:hover::part(base), [data-rarity=epic] .chip[data-hovered=true]::part(base) { border-color: var(--hud-epic); }
[data-rarity=legendary] .chip:hover::part(base), [data-rarity=legendary] .chip[data-hovered=true]::part(base) { border-color: var(--hud-legendary); }
/* Epic and legendary: a beam from the item up to the chip. */
.beam { inline-size: var(--stroke-thick); block-size: var(--world-beam-height); opacity: var(--world-glow-opacity); background: var(--world-epic-beam); box-shadow: var(--world-epic-beam-glow); }
[data-rarity=legendary] .beam { background: var(--world-legendary-beam); box-shadow: var(--world-legendary-beam-glow); }
@media (prefers-reduced-motion: reduce) {
  .chip, .sheen { animation: none; }
  .sheen { display: none; }
}
</style>
