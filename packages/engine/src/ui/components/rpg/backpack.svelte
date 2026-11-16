<meta name="component" content="RpgBackpack">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgBackpack.frame RpgBackpack.band RpgBackpack.title RpgBackpack.tab RpgBackpack.cell">
<meta name="prop:title" content="string" data-doc="The bag's heading (Backpack, Stash, a chest's name).">
<meta name="prop:tabs" content="list" data-doc="The categories, a keyed collection of { id, label }.">
<meta name="prop:tab" content="string" data-doc="The id of the category shown.">
<meta name="prop:items" content="list" data-doc="The items shown, in the game's order: each { id, name, icon, count, rarity, equipped, fresh, locked, broken }. rarity is common, uncommon, rare, epic, legendary or mythic; count above 1 is shown; fresh marks an unseen item, locked one the game will not let go, broken one it will not equip.">
<meta name="prop:selected" content="string" data-doc="The selected item's id; empty is none.">
<meta name="prop:weight" content="string" data-doc="The load as the game words it (92.4 / 120); empty hides it.">
<meta name="prop:load" content="number" data-doc="The load from 0 to 1 for its bar; above 1 is overburdened.">
<meta name="prop:gold" content="string" data-doc="The purse as the game formats it; empty hides it.">
<meta name="action:tabPrev" content="Previous category">
<meta name="action:tabNext" content="Next category">
<meta name="fixture:default" content='{"title":"Backpack","tab":"all","tabs":[{"id":"all","label":"All"},{"id":"gear","label":"Weapons"}],"items":[{"id":"blade","name":"Emberwake Blade","icon":"item-blade","count":1,"rarity":"legendary","equipped":true,"fresh":false,"locked":false,"broken":false},{"id":"potion","name":"Healing Draught","icon":"item-potion","count":5,"rarity":"common","equipped":false,"fresh":true,"locked":false,"broken":false}],"selected":"blade","weight":"92.4 / 120","load":0.77,"gold":"4,327"}'>
<meta name="fixture:empty" content='{"title":"Backpack","tab":"all","tabs":[{"id":"all","label":"All"}],"items":[],"selected":"","weight":"0 / 120","load":0,"gold":"0"}'>

<!--
  A bag of items: a titled band, category tabs, a grid of slots and a
  footer with the carried weight and the purse. The game owns what a
  category holds, the order and every number; the bag reports choose (an
  item's key), tab (a category's key), tabNext and tabPrev. A slot drags
  onto another (a locked one stays put) and the bag reports move with
  { dragged, target }, the two items' keys; the dragged slot springs
  home, and the game reorders, swaps or stacks as it sees fit. The grid
  fills the height it is given and scrolls inside it. Each slot carries
  data-rarity, data-equipped and the like; a style pack draws them.
-->
<section id="RpgBackpack" part="RpgBackpack.frame" aria-label={title}>
  <header part="RpgBackpack.band"><div class="band-end start"></div><h2 part="RpgBackpack.title">{title}</h2><div class="band-end finish"></div></header>
  <div class="tabs" role="tablist" aria-label={t("rpg.backpack.tabs")} ontabNext={tabNext} ontabPrev={tabPrev}>
    <div class="tabs-scroll">
      <div class="tabs-row">
        {#each tabs as c (c.id)}
          <button part="RpgBackpack.tab" variant="quiet" role="tab" aria-selected={c.id == tab} onclick={choosetab}>{c.label}</button>
        {/each}
      </div>
    </div>
    <div class="tabs-fade"></div>
  </div>
  {#if items.length}
    <div class="grid-scroll">
      <div class="grid" role="grid" aria-label={title}>
        {#each items as i (i.id)}
          <button part="RpgBackpack.cell" data-rarity={i.rarity} data-equipped={i.equipped} data-fresh={i.fresh} data-locked={i.locked} data-broken={i.broken} role="gridcell" aria-selected={i.id == selected} aria-label={i.name} draggable={!i.locked} data-drag-type="item" data-accept="item" onclick={choose} ondrop={move}><div class="face">
            <og-icon class="art" icon={i.icon}></og-icon>
            {#if i.count > 1}<span class="count">{formatNumber(i.count)}</span>{/if}
            {#if i.equipped}<div class="badge equipped" aria-label={t("rpg.backpack.equipped")}><og-icon icon="check"></og-icon></div>{/if}
            {#if i.fresh}<div class="badge fresh" aria-label={t("rpg.backpack.new")}></div>{/if}
            {#if i.locked}<div class="badge locked" aria-label={t("rpg.backpack.locked")}><og-icon icon="lock"></og-icon></div>{/if}
          </div></button>
        {/each}
      </div>
    </div>
  {:else}
    <div class="empty" role="status"><og-icon class="empty-mark" icon="bag"></og-icon><p class="empty-text">{t("rpg.backpack.empty")}</p></div>
  {/if}
  {#if weight || gold}<footer class="foot">
    {#if weight}<div class="weight">
      <og-icon class="foot-mark" icon="bag"></og-icon>
      <div class="weight-text">
        <div class="weight-line"><span class="caption">{t("rpg.backpack.weight")}</span><span class="figure" data-over={load > 1}>{weight}</span></div>
        <div class="bar" role="meter" aria-label={t("rpg.backpack.weight")} aria-valuetext={weight} data-over={load > 1} style:--fill={load > 1 ? 1 : load}><div class="bar-fill"></div></div>
      </div>
    </div>{/if}
    {#if gold}<div class="purse" aria-label={t("rpg.backpack.gold", { gold })}><og-icon class="foot-mark coin" icon="coin"></og-icon><span class="figure">{gold}</span></div>{/if}
  </footer>{/if}
</section>

<style>
:host { display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; gap: var(--space-sm); container-type: inline-size; }
[part="RpgBackpack.band"] { display: flex; width: stretch; align-items: center; justify-content: center; gap: var(--space-md); flex: none; }
.band-end { width: var(--size-lg); height: var(--space-lg); flex: none; }
.tabs { position: relative; display: flex; width: stretch; align-items: center; gap: var(--space-xs); }
.tabs-fade { display: none; position: absolute; inset-inline-end: 0; top: 0; bottom: 0; width: var(--size-lg); pointer-events: none; }
@media (width < 600px), (height < 500px) {
  .tabs-fade { display: block; }
  [part="RpgBackpack.band"] { min-height: var(--size-lg); padding-top: 0; padding-bottom: 0; }
  .grid { gap: var(--space-xs); }
  .foot { gap: var(--space-sm); padding-top: var(--space-xs); }
  [part="RpgBackpack.tab"] { min-height: calc(var(--size-touch) * 0.85); }
}
.tabs-scroll { flex: 1; overflow-x: auto; overflow-y: hidden; min-inline-size: 0; }
.tabs-row { display: flex; gap: var(--space-xs); min-inline-size: 100%; inline-size: max-content; }
[part="RpgBackpack.tab"] { flex: 1 0 auto; min-height: var(--size-touch); padding-inline: var(--space-sm); }
[part="RpgBackpack.tab"][aria-selected=true] { background: var(--selection); box-shadow: inset 0 calc(var(--stroke-thick) * -1) 0 var(--primary); }
.grid-scroll { flex: 1; min-block-size: 0; overflow-y: auto; }
.grid { display: grid; grid-template-columns: repeat(6, 1fr); gap: var(--space-sm); width: stretch; padding: var(--space-xs); }
[part="RpgBackpack.cell"] { position: relative; width: stretch; aspect-ratio: 1; min-width: var(--size-touch); padding: var(--space-sm); align-items: center; justify-content: center; }
[part="RpgBackpack.cell"] .face { display: flex; flex-direction: column; position: relative; width: stretch; height: stretch; align-items: center; justify-content: center; }
.art { width: stretch; height: stretch; }
.count { position: absolute; inset-inline-end: var(--space-xxs); bottom: 0; padding-inline: var(--space-xs); background: var(--scene-plate); color: var(--scene-ink); border-radius: var(--radius-control); font-family: var(--figure-font); text-align: end; }
.badge { display: flex; flex-direction: column; position: absolute; align-items: center; justify-content: center; }
.badge og-icon { width: var(--space-md); height: var(--space-md); }
.badge.equipped { inset-inline-start: var(--space-xxs); top: var(--space-xxs); width: var(--space-lg); height: var(--space-lg); background: var(--scene-plate); color: var(--success); border-radius: var(--radius-control); }
.badge.fresh { inset-inline-end: var(--space-xs); top: var(--space-xs); width: var(--space-sm); height: var(--space-sm); background: var(--info); border-radius: var(--radius-capsule); }
.badge.locked { inset-inline-end: var(--space-xxs); top: var(--space-xxs); width: var(--space-lg); height: var(--space-lg); background: var(--scene-plate); color: var(--scene-ink); border-radius: var(--radius-control); }
.empty { display: flex; flex-direction: column; width: stretch; flex: 1; gap: var(--space-sm); align-items: center; justify-content: center; }
.empty-mark { width: var(--size-lg); height: var(--size-lg); }
.empty-text { margin: 0; }
.foot { display: flex; gap: var(--space-lg); align-items: center; padding: var(--space-sm) var(--space-xs) 0; border-block-start: var(--stroke-hairline) solid var(--line); }
.weight { display: flex; flex: 1; min-width: 0; gap: var(--space-sm); align-items: center; }
.weight-text { display: flex; flex-direction: column; flex: 1; min-width: 0; gap: var(--space-xs); }
.weight-line { display: flex; gap: var(--space-sm); align-items: baseline; white-space: nowrap; }
.foot-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.bar { position: relative; height: var(--space-xs); background: var(--well); border-radius: var(--meter-radius); overflow: hidden; }
.bar-fill { position: absolute; inset-inline-start: 0; top: 0; bottom: 0; inline-size: calc(var(--fill, 0) * 100%); background: var(--primary); }
.bar[data-over=true] .bar-fill { background: var(--warn); }
.purse { display: flex; flex: none; gap: var(--space-xs); align-items: center; }
.figure { font-variant-numeric: tabular-nums; white-space: nowrap; }
@container (width < 400px) { .grid { grid-template-columns: repeat(5, 1fr); } }
</style>
