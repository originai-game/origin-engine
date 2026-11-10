<meta name="component" content="RpgBackpack">
<meta name="strings" content="./rpg.en.ogt">
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
  item's key), tab (a category's key), tabNext and tabPrev. The grid
  fills the height it is given and scrolls inside it. Each slot carries
  data-rarity, data-equipped and the like; a style pack draws them.
-->
<section id="RpgBackpack" class="rpg-backpack" aria-label={title}>
  <header class="rpg-band"><div class="band-end start"></div><h2 class="rpg-title">{title}</h2><div class="band-end finish"></div></header>
  <div class="tabs" role="tablist" aria-label={t("rpg.backpack.tabs")} ontabNext={tabNext} ontabPrev={tabPrev}>
    <scroll class="tabs-scroll">
      <div class="tabs-row">
        {#each tabs as c (c.id)}
          <button class="tab" variant="quiet" role="tab" aria-selected={c.id == tab} onclick={choosetab}>{c.label}</button>
        {/each}
      </div>
    </scroll>
  </div>
  {#if items.length}
    <scroll class="grid-scroll">
      <div class="grid" role="grid" aria-label={title}>
        {#each items as i (i.id)}
          <button class="cell" data-rarity={i.rarity} data-equipped={i.equipped} data-fresh={i.fresh} data-locked={i.locked} data-broken={i.broken} role="gridcell" aria-selected={i.id == selected} aria-label={i.name} onclick={choose}><div class="face">
            <icon class="art" icon={i.icon}></icon>
            <span class="count" hidden={!(i.count > 1)}>{formatNumber(i.count)}</span>
            <div class="badge equipped" hidden={!i.equipped} aria-label={t("rpg.backpack.equipped")}><icon icon="check"></icon></div>
            <div class="badge fresh" hidden={!i.fresh} aria-label={t("rpg.backpack.new")}></div>
            <div class="badge locked" hidden={!i.locked} aria-label={t("rpg.backpack.locked")}><icon icon="lock"></icon></div>
          </div></button>
        {/each}
      </div>
    </scroll>
  {:else}
    <div class="empty" role="status"><icon class="empty-mark" icon="bag"></icon><p class="empty-text">{t("rpg.backpack.empty")}</p></div>
  {/if}
  <footer class="foot" hidden={!weight && !gold}>
    <div class="weight" hidden={!weight}>
      <icon class="foot-mark" icon="bag"></icon>
      <div class="weight-text">
        <div class="weight-line"><span class="caption">{t("rpg.backpack.weight")}</span><span class="figure" data-over={load > 1}>{weight}</span></div>
        <div class="bar" role="meter" aria-label={t("rpg.backpack.weight")} aria-valuetext={weight} data-over={load > 1} style:--fill={load > 1 ? 1 : load}><div class="bar-fill"></div></div>
      </div>
    </div>
    <div class="purse" hidden={!gold} aria-label={t("rpg.backpack.gold", { gold })}><icon class="foot-mark coin" icon="coin"></icon><span class="figure">{gold}</span></div>
  </footer>
</section>

<style>
.rpg-backpack { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-sm); container-type: inline-size; }
.rpg-band { display: flex; width: stretch; align-items: center; justify-content: center; gap: var(--space-md); flex: none; }
.band-end { width: var(--size-lg); height: var(--space-lg); flex: none; }
.tabs { display: flex; width: stretch; align-items: center; gap: var(--space-xs); }
.tabs-scroll { width: stretch; overflow-x: auto; overflow-y: hidden; }
.tabs-row { display: flex; gap: var(--space-xxs); justify-content: space-between; width: stretch; }
.tab { flex: none; min-height: var(--size-touch); padding: 0 var(--space-xs); }
.grid-scroll { width: stretch; flex: 1; min-height: 0; }
.grid { display: grid; grid-template-columns: repeat(6, 1fr); gap: var(--space-sm); width: stretch; padding: var(--space-xs); }
.cell { position: relative; width: stretch; aspect-ratio: 1; min-width: var(--size-touch); padding: var(--space-sm); align-items: center; justify-content: center; }
.cell .face { position: relative; width: stretch; height: stretch; align-items: center; justify-content: center; }
.art { width: stretch; height: stretch; }
.count { position: absolute; right: var(--space-xxs); bottom: 0; width: fit-content; text-align: end; }
.badge { position: absolute; align-items: center; justify-content: center; }
.badge icon { width: var(--space-md); height: var(--space-md); }
.badge.equipped { left: var(--space-xxs); top: var(--space-xxs); width: var(--space-lg); height: var(--space-lg); }
.badge.fresh { right: var(--space-xs); top: var(--space-xs); width: var(--space-sm); height: var(--space-sm); }
.badge.locked { right: var(--space-xxs); top: var(--space-xxs); width: var(--space-lg); height: var(--space-lg); }
.empty { display: flex; flex-direction: column; width: stretch; flex: 1; gap: var(--space-sm); align-items: center; justify-content: center; }
.empty-mark { width: var(--size-lg); height: var(--size-lg); }
.empty-text { margin: 0; }
.foot { display: flex; width: stretch; gap: var(--space-lg); align-items: center; padding: var(--space-sm) var(--space-xs) 0; }
.weight { display: flex; width: stretch; gap: var(--space-sm); align-items: center; }
.weight-text { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); }
.weight-line { display: flex; gap: var(--space-sm); align-items: baseline; }
.foot-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.bar { width: stretch; height: var(--space-sm); }
.bar-fill { width: calc(var(--fill, 0) * 100%); height: stretch; }
.purse { display: flex; flex: none; gap: var(--space-xs); align-items: center; }
.figure { font-variant-numeric: tabular-nums; white-space: nowrap; }
@container (width < 400px) { .grid { grid-template-columns: repeat(5, 1fr); } }
</style>
