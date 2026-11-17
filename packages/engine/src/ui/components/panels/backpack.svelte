<meta name="component" content="RpgBackpack">
<meta name="strings" content="./backpack.en.ogt">
<meta name="parts" content="RpgBackpack.frame RpgBackpack.band RpgBackpack.title RpgBackpack.tab RpgBackpack.cell RpgBackpack.badge RpgBackpack.dot RpgBackpack.caption RpgBackpack.figure RpgBackpack.track RpgBackpack.fill">
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
<meta name="fixture:default" content='{"title":"Backpack","tab":"all","tabs":[{"id":"all","label":"All"},{"id":"gear","label":"Weapons"}],"items":[{"id":"blade","name":"Emberwake Blade","icon":"sword","count":1,"rarity":"legendary","equipped":true,"fresh":false,"locked":false,"broken":false},{"id":"potion","name":"Healing Draught","icon":"potion","count":5,"rarity":"common","equipped":false,"fresh":true,"locked":false,"broken":false}],"selected":"blade","weight":"92.4 / 120","load":0.77,"gold":"4,327"}'>
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
<section id="RpgBackpack" part="RpgBackpack.frame" data-layout="shell" aria-label={title}>
  <header part="RpgBackpack.band" data-layout="row" data-gap="loose"><div class="band-end start"></div><h2 part="RpgBackpack.title">{title}</h2><div class="band-end finish"></div></header>
  <div class="tabs" data-layout="row" data-gap="tight" role="tablist" aria-label={t("rpg.backpack.tabs")} ontabNext={tabNext} ontabPrev={tabPrev}>
    <div class="tabs-scroll" data-fill>
      <div class="tabs-row" data-layout="cluster" data-gap="tight">
        {#each tabs as c (c.id)}
          <button part="RpgBackpack.tab" variant="quiet" role="tab" aria-selected={c.id == tab} onclick={choosetab}>{c.label}</button>
        {/each}
      </div>
    </div>
    <div class="tabs-fade"></div>
  </div>
  {#if items.length}
    <div class="grid-scroll" data-layout="scroll" data-fill>
      <div class="grid" data-layout="grid" role="grid" aria-label={title}>
        {#each items as i (i.id)}
          <button part="RpgBackpack.cell" data-rarity={i.rarity} data-equipped={i.equipped} data-fresh={i.fresh} data-locked={i.locked} data-broken={i.broken} role="gridcell" aria-selected={i.id == selected} aria-label={i.name} draggable={!i.locked} data-drag-type="item" data-accept="item" onclick={choose} ondrop={move}><div class="face" data-layout="cover" data-gap="none">
            <og-icon class="art" icon={i.icon}></og-icon>
            {#if i.count > 1}<span part="RpgBackpack.badge" class="count">{formatNumber(i.count)}</span>{/if}
            {#if i.equipped}<div part="RpgBackpack.badge" class="badge equipped" data-layout="cover" data-tone="success" aria-label={t("rpg.backpack.equipped")}><og-icon icon="check"></og-icon></div>{/if}
            {#if i.fresh}<div part="RpgBackpack.dot" class="badge fresh" aria-label={t("rpg.backpack.new")}></div>{/if}
            {#if i.locked}<div part="RpgBackpack.badge" class="badge locked" data-layout="cover" aria-label={t("rpg.backpack.locked")}><og-icon icon="lock"></og-icon></div>{/if}
          </div></button>
        {/each}
      </div>
    </div>
  {:else}
    <div class="empty" data-layout="cover" data-fill role="status"><og-icon class="empty-mark" icon="bag"></og-icon><p class="empty-text">{t("rpg.backpack.empty")}</p></div>
  {/if}
  {#if weight || gold}<footer class="foot" data-layout="row" data-gap="wide">
    {#if weight}<div class="weight" data-layout="row" data-fill>
      <og-icon class="foot-mark" icon="bag"></og-icon>
      <div class="weight-text" data-layout="stack" data-gap="tight" data-fill>
        <div class="weight-line" data-layout="row"><span part="RpgBackpack.caption" class="caption">{t("rpg.backpack.weight")}</span><span part="RpgBackpack.figure" class="figure" data-tone={load > 1 ? 'warn' : ''} data-over={load > 1}>{weight}</span></div>
        <div part="RpgBackpack.track" class="bar" role="meter" aria-label={t("rpg.backpack.weight")} aria-valuetext={weight} data-over={load > 1} style:--fill={load > 1 ? 1 : load}><div part="RpgBackpack.fill" class="bar-fill" data-tone={load > 1 ? 'warn' : ''}></div></div>
      </div>
    </div>{/if}
    {#if gold}<div class="purse" data-layout="row" data-gap="tight" aria-label={t("rpg.backpack.gold", { gold })}><og-icon class="foot-mark coin" icon="coin"></og-icon><span part="RpgBackpack.figure" class="figure">{gold}</span></div>{/if}
  </footer>{/if}
</section>

<style>
[part="RpgBackpack.band"] { width: stretch; justify-content: center; flex: none; }
.band-end { width: var(--size-lg); height: var(--space-lg); flex: none; }
.tabs { position: relative; width: stretch; }
.tabs-fade { display: none; position: absolute; inset-inline-end: 0; top: 0; bottom: 0; width: var(--size-lg); pointer-events: none; }
@media (width < 600px), (height < 500px) {
  .tabs-fade { display: block; }
  [part="RpgBackpack.band"] { min-height: var(--size-lg); padding-top: 0; padding-bottom: 0; }
  .foot { padding-top: var(--space-xs); }
  [part="RpgBackpack.tab"] { min-height: calc(var(--size-touch) * 0.85); }
}
.tabs-scroll { overflow-x: auto; overflow-y: hidden; }
[part="RpgBackpack.tab"] { flex: 1 0 auto; min-height: var(--size-touch); padding-inline: var(--space-sm); }
.grid { grid-template-columns: repeat(6, 1fr); width: stretch; padding: var(--space-xs); }
[part="RpgBackpack.cell"] { position: relative; width: stretch; aspect-ratio: 1; min-width: var(--size-touch); padding: var(--space-sm); align-items: center; justify-content: center; }
[part="RpgBackpack.cell"] .face { position: relative; width: stretch; height: stretch; }
.art { width: stretch; height: stretch; }
.count { position: absolute; inset-inline-end: var(--space-xxs); bottom: 0; text-align: end; }
.badge { position: absolute; }
.badge og-icon { width: var(--space-md); height: var(--space-md); }
.badge.equipped { inset-inline-start: var(--space-xxs); top: var(--space-xxs); width: var(--space-lg); height: var(--space-lg); }
.badge.fresh { --dot-fill: var(--info); inset-inline-end: var(--space-xs); top: var(--space-xs); width: var(--space-sm); height: var(--space-sm); }
.badge.locked { inset-inline-end: var(--space-xxs); top: var(--space-xxs); width: var(--space-lg); height: var(--space-lg); }
.empty { width: stretch; }
.empty-mark { width: var(--size-lg); height: var(--size-lg); }
.empty-text { margin: 0; }
.foot { padding: var(--space-sm) var(--space-xs) 0; }
.weight-line { align-items: baseline; white-space: nowrap; }
.foot-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.bar { position: relative; height: var(--space-xs); overflow: hidden; }
.bar-fill { position: absolute; inset-inline-start: 0; top: 0; bottom: 0; inline-size: calc(var(--fill, 0) * 100%); }
.purse { flex: none; }
.figure { white-space: nowrap; }
@container (width < 400px) { .grid { grid-template-columns: repeat(5, 1fr); } }
</style>
