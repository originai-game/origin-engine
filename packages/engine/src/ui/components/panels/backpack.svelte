<meta name="component" content="RpgBackpack">
<meta name="strings" content="./backpack.en.ogt">
<meta name="parts" content="RpgBackpack.frame RpgBackpack.band RpgBackpack.kicker RpgBackpack.title RpgBackpack.purse RpgBackpack.tab RpgBackpack.count RpgBackpack.capacity RpgBackpack.cell RpgBackpack.badge RpgBackpack.state RpgBackpack.bracket RpgBackpack.dot RpgBackpack.caption RpgBackpack.figure RpgBackpack.track RpgBackpack.fill">
<meta name="prop:title" content="string" data-doc="The bag's heading (Backpack, Stash, a chest's name).">
<meta name="prop:kicker" content="string" data-doc="A small line over the title naming the screen (Inventory); empty hides it.">
<meta name="prop:tabs" content="list" data-doc="The categories, a keyed collection of { id, label, count }: count, how many items the category holds, is shown beside its label above 0.">
<meta name="prop:tab" content="string" data-doc="The id of the category shown.">
<meta name="prop:items" content="list" data-doc="The items shown, in the game's order: each { id, name, icon, count, rarity, equipped, fresh, locked, broken }. rarity is common, uncommon, rare, epic, legendary or mythic; count above 1 is shown; fresh marks an unseen item, locked one the game will not let go, broken one it will not equip.">
<meta name="prop:selected" content="string" data-doc="The selected item's id; empty is none.">
<meta name="prop:capacity" content="string" data-doc="How full the bag is, as the game words it (30 / 36 slots); empty hides it.">
<meta name="prop:weight" content="string" data-doc="The load as the game words it (92.4 / 120); empty hides it.">
<meta name="prop:load" content="number" data-doc="The load from 0 to 1 for its bar; above 1 is overburdened.">
<meta name="prop:gold" content="string" data-doc="The purse as the game formats it; empty hides it.">
<meta name="action:tabPrev" content="Previous category">
<meta name="action:tabNext" content="Next category">
<meta name="fixture:default" content='{"title":"Backpack","kicker":"Inventory","tab":"all","tabs":[{"id":"all","label":"All","count":6},{"id":"gear","label":"Weapons","count":2},{"id":"armour","label":"Armour","count":1},{"id":"use","label":"Consumables","count":2},{"id":"mat","label":"Materials","count":1}],"items":[{"id":"blade","name":"Emberwake Blade","icon":"sword","count":1,"rarity":"legendary","equipped":true,"fresh":false,"locked":false,"broken":false},{"id":"potion","name":"Healing Draught","icon":"potion","count":5,"rarity":"common","equipped":false,"fresh":true,"locked":false,"broken":false},{"id":"shield","name":"Warden Shield","icon":"shield","count":1,"rarity":"epic","equipped":false,"fresh":false,"locked":true,"broken":false},{"id":"crown","name":"Cracked Circlet","icon":"crown","count":1,"rarity":"rare","equipped":false,"fresh":false,"locked":false,"broken":true},{"id":"gem","name":"Storm Crystal","icon":"gem","count":12,"rarity":"uncommon","equipped":false,"fresh":false,"locked":false,"broken":false},{"id":"key","name":"Vault Key","icon":"key","count":1,"rarity":"mythic","equipped":false,"fresh":false,"locked":false,"broken":false}],"selected":"blade","capacity":"6 / 36 slots","weight":"92.4 / 120","load":0.77,"gold":"4,327"}'>
<meta name="fixture:empty" content='{"title":"Backpack","tab":"all","tabs":[{"id":"all","label":"All","count":0}],"items":[],"selected":"","weight":"0 / 120","load":0,"gold":"0"}'>
<meta name="fixture:over" content='{"title":"Backpack","tab":"all","tabs":[{"id":"all","label":"All","count":2}],"items":[{"id":"anvil","name":"Field Anvil","icon":"hammer","count":1,"rarity":"common","equipped":false,"fresh":false,"locked":false,"broken":false},{"id":"ore","name":"Iron Ore","icon":"gem","count":40,"rarity":"common","equipped":false,"fresh":false,"locked":false,"broken":false}],"selected":"","weight":"131.0 / 120","load":1.09,"gold":"12"}'>
<meta name="fixture:overflow" content='{"title":"The Bottomless Satchel of the Wandering Quartermaster","kicker":"Inventory","tab":"mat","tabs":[{"id":"all","label":"All","count":1240},{"id":"gear","label":"Weapons","count":312},{"id":"armour","label":"Armour","count":208},{"id":"acc","label":"Accessories","count":96},{"id":"use","label":"Consumables","count":420},{"id":"mat","label":"Crafting materials","count":204}],"items":[{"id":"a","name":"Ore","icon":"gem","count":99999,"rarity":"common","equipped":false,"fresh":true,"locked":true,"broken":false},{"id":"b","name":"Blade","icon":"sword","count":1,"rarity":"mythic","equipped":true,"fresh":true,"locked":true,"broken":true}],"selected":"b","capacity":"1,240 / 1,500 slots","weight":"1,234.5 / 2,000","load":0.62,"gold":"1,234,567,890"}'>

<!--
  A bag of items: a band with a kicker, the title and the purse, category
  tabs between the action's key caps, how full the bag is, a grid of item
  tiles and a footer with the carried weight. The game owns what a
  category holds, the order and every number; the bag reports choose (an
  item's key), choosetab (a category's key), tabNext and tabPrev. A tile
  drags onto another (a locked one stays put) and the bag reports move
  with { dragged, target }, the two items' keys; the dragged tile springs
  home, and the game reorders, swaps or stacks as it sees fit. The grid
  fills the height it is given and scrolls inside it. A tile shows its
  rarity as its rim, a light rising from its foot and a bar along it; its
  stack, Equipped, New, Locked and Broken sit in its corners, each named
  in words for a screen reader, Broken also on the tile. The selected
  tile wears corner brackets.
-->
<section id="RpgBackpack" part="RpgBackpack.frame" data-layout="shell" aria-label={title}>
  <div class="lit"></div>
  <header part="RpgBackpack.band" data-layout="cluster" data-gap="loose">
    <div class="heading" data-layout="stack" data-gap="hair" data-fill>
      {#if kicker}<span part="RpgBackpack.kicker" class="kicker">{kicker}</span>{/if}
      <h2 part="RpgBackpack.title">{title}</h2>
    </div>
    {#if gold}<div part="RpgBackpack.purse" class="purse" data-layout="row" data-gap="tight" aria-label={t("rpg.backpack.gold", { gold })}><span class="coin"></span><span part="RpgBackpack.figure" class="figure">{gold}</span></div>{/if}
  </header>
  {#if tabs.length}<div class="tabs" data-layout="row" data-gap="tight" role="tablist" aria-label={t("rpg.backpack.tabs")} ontabNext={tabNext} ontabPrev={tabPrev}>
    <kbd class="cap" data-action="tabPrev"></kbd>
    <div class="tabs-scroll" data-fill>
      <div class="tabs-row" data-layout="row" data-gap="none">
        {#each tabs as c (c.id)}
          <button part="RpgBackpack.tab" variant="quiet" role="tab" aria-selected={c.id == tab} onclick={choosetab}><span class="tab-label">{c.label}</span>{#if c.count > 0}<span part="RpgBackpack.count" class="count">{formatNumber(c.count)}</span>{/if}<span class="line"></span></button>
        {/each}
      </div>
    </div>
    <kbd class="cap" data-action="tabNext"></kbd>
  </div>{/if}
  {#if capacity}<p part="RpgBackpack.capacity RpgBackpack.caption" class="capacity">{capacity}</p>{/if}
  {#if items.length}
    <div class="grid-scroll" data-layout="scroll" data-fill>
      <div class="grid" data-layout="grid" role="grid" aria-label={title}>
        {#each items as i (i.id)}
          <button part="RpgBackpack.cell" data-rarity={i.rarity} data-equipped={i.equipped} data-fresh={i.fresh} data-locked={i.locked} data-broken={i.broken} role="gridcell" aria-selected={i.id == selected} aria-label={i.name} draggable={!i.locked} data-drag-type="item" data-accept="item" onclick={choose} ondrop={move}>
            <div class="tile" data-layout="cover" data-gap="none"><og-icon class="art" icon={i.icon}></og-icon><div class="bar"></div>{#if i.broken}<span part="RpgBackpack.state" class="broken">{t("rpg.backpack.broken")}</span>{/if}</div>
            {#if i.count > 1}<span part="RpgBackpack.badge" class="tag stack">{formatNumber(i.count)}</span>{/if}
            {#if i.equipped}<span part="RpgBackpack.state" class="tag equipped" aria-label={t("rpg.backpack.equipped")}>{t("rpg.backpack.equippedMark")}</span>{/if}
            {#if i.locked}<div class="tag locked" data-layout="cover" aria-label={t("rpg.backpack.locked")}><og-icon icon="lock"></og-icon></div>{/if}
            {#if i.fresh}<div part="RpgBackpack.dot" class="fresh" data-locked={i.locked} aria-label={t("rpg.backpack.new")}></div>{/if}
            {#if i.id == selected}<div part="RpgBackpack.bracket" class="bracket"></div>{/if}
          </button>
        {/each}
      </div>
    </div>
  {:else}
    <div class="empty" data-layout="cover" data-fill role="status"><og-icon class="empty-mark" icon="bag"></og-icon><p part="RpgBackpack.caption" class="empty-text">{t("rpg.backpack.empty")}</p></div>
  {/if}
  {#if weight}<footer class="foot" data-layout="stack" data-gap="tight">
    <div class="weight-line" data-layout="row"><span part="RpgBackpack.caption" class="caption" data-fill>{t("rpg.backpack.weight")}</span><span part="RpgBackpack.figure" class="figure" data-tone={load > 1 ? 'warn' : ''} data-over={load > 1}>{weight}</span></div>
    <div part="RpgBackpack.track" class="load" role="meter" aria-label={t("rpg.backpack.weight")} aria-valuetext={weight} data-over={load > 1} style:--fill={load > 1 ? 1 : load}><div part="RpgBackpack.fill" class="load-fill" data-tone={load > 1 ? 'warn' : ''}></div></div>
    {#if load > 1}<p part="RpgBackpack.caption" class="over" data-tone="warn">{t("rpg.backpack.over")}</p>{/if}
  </footer>{/if}
</section>

<style>
/* The frame: two corners cut, a lit top edge. */
[part="RpgBackpack.frame"] { position: relative; clip-path: polygon(var(--inv-frame-cut) 0, 100% 0, 100% calc(100% - var(--inv-frame-cut)), calc(100% - var(--inv-frame-cut)) 100%, 0 100%, 0 var(--inv-frame-cut)); }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
[part="RpgBackpack.band"] { width: stretch; flex: none; align-items: flex-end; }
.heading { min-width: 0; flex-basis: calc(120 * var(--font-step-px)); }
.kicker { font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-kicker-ink); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
[part="RpgBackpack.title"] { margin: 0; min-width: 0; overflow-wrap: anywhere; }
.purse { flex: none; align-items: center; padding: var(--space-xxs) var(--space-sm) var(--space-xxs) var(--space-xxs); border-radius: var(--inv-chip-radius); background: var(--inv-chip-fill); box-shadow: var(--inv-chip-rim); }
.coin { flex: none; inline-size: calc(22 * var(--font-step-px)); block-size: calc(22 * var(--font-step-px)); border-radius: var(--inv-chip-radius); background: var(--inv-coin); }
.figure { white-space: nowrap; font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
/* Tabs: an underline strip between the key caps; the strip scrolls and
   fades out before the cap, so no label sits under or past it. */
.tabs { width: stretch; flex: none; align-items: center; border-block-end: var(--inv-rule); }
.cap { flex: none; }
.tabs-scroll { min-width: 0; overflow-x: auto; overflow-y: hidden; scrollbar-width: none; mask-image: var(--inv-tab-fade); }
.tabs-row { padding-inline-end: var(--size-lg); }
[part="RpgBackpack.tab"] { position: relative; flex: none; min-height: var(--size-touch); padding: 0 var(--space-sm); gap: var(--space-xs); border-radius: 0; background: var(--inv-bare-fill); box-shadow: none; color: var(--inv-tab-ink); }
[part="RpgBackpack.tab"][aria-selected=true] { color: var(--inv-tab-on-ink); }
.line { position: absolute; inset-inline: var(--space-xs); bottom: 0; block-size: var(--stroke-thick); background: var(--inv-tab-line); box-shadow: var(--inv-tab-glow); opacity: 0; }
[aria-selected=true] > .line { opacity: 1; }
.tab-label { white-space: nowrap; }
.count { padding: 0 var(--space-xs); border-radius: var(--inv-chip-radius); font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); background: var(--inv-tab-count-fill); color: var(--inv-kicker-ink); }
[aria-selected=true] .count { background: var(--inv-tab-count-on-fill); color: var(--inv-accent-ink); }
.capacity { margin: 0; flex: none; padding-inline: var(--space-xs); }
/* The grid: each tile cut at two corners inside a cell that holds its
   corner tags and, when selected, its brackets. */
.grid { --layout-tile: calc(72 * var(--font-step-px)); grid-template-columns: repeat(auto-fill, minmax(var(--layout-tile), 1fr)); width: stretch; padding: var(--space-xs); gap: var(--space-xxs); }
[part="RpgBackpack.cell"] { --inv-socket-fill: var(--cell-common-fill); --inv-socket-rim: var(--cell-common-shadow); --inv-rarity: var(--common); position: relative; width: stretch; aspect-ratio: 1; min-width: var(--size-touch); padding: var(--space-xxs); border: none; border-radius: 0; background: var(--inv-bare-fill); box-shadow: none; }
[part="RpgBackpack.cell"]:hover, [part="RpgBackpack.cell"]:active { background: var(--inv-bare-fill); box-shadow: none; }
[part="RpgBackpack.cell"][data-rarity=uncommon] { --inv-socket-fill: var(--cell-uncommon-fill); --inv-socket-rim: var(--cell-uncommon-shadow); --inv-rarity: var(--uncommon); }
[part="RpgBackpack.cell"][data-rarity=rare] { --inv-socket-fill: var(--cell-rare-fill); --inv-socket-rim: var(--cell-rare-shadow); --inv-rarity: var(--rare); }
[part="RpgBackpack.cell"][data-rarity=epic] { --inv-socket-fill: var(--cell-epic-fill); --inv-socket-rim: var(--cell-epic-shadow); --inv-rarity: var(--epic); }
[part="RpgBackpack.cell"][data-rarity=legendary] { --inv-socket-fill: var(--cell-legendary-fill); --inv-socket-rim: var(--cell-legendary-shadow); --inv-rarity: var(--legendary); }
[part="RpgBackpack.cell"][data-rarity=mythic] { --inv-socket-fill: var(--cell-mythic-fill); --inv-socket-rim: var(--cell-mythic-shadow); --inv-rarity: var(--mythic); }
.tile { position: relative; width: stretch; height: stretch; background: var(--inv-socket-fill); box-shadow: var(--inv-socket-rim); clip-path: polygon(var(--inv-slot-cut) 0, 100% 0, 100% calc(100% - var(--inv-slot-cut)), calc(100% - var(--inv-slot-cut)) 100%, 0 100%, 0 var(--inv-slot-cut)); }
[part="RpgBackpack.cell"] { --inv-base-rim: var(--inv-socket-rim); }
[part="RpgBackpack.cell"]:hover .tile { --inv-socket-rim: var(--inv-slot-hover-rim), var(--inv-base-rim); }
[part="RpgBackpack.cell"][aria-selected=true] .tile { --inv-socket-rim: var(--inv-slot-selected-rim), var(--inv-base-rim); }
.art { width: 68%; height: 68%; filter: var(--inv-slot-art-shadow); }
[data-broken=true] .art { filter: var(--inv-slot-broken-art); }
.bar { position: absolute; inset-inline: var(--inv-slot-cut); bottom: 0; block-size: var(--stroke-thick); color: var(--inv-rarity); background: currentColor; box-shadow: var(--inv-slot-bar-glow); }
.tag { position: absolute; padding: 0 var(--space-xxs); font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); line-height: 1.35; color: var(--ink-bright); background: var(--inv-tag-fill); box-shadow: var(--inv-tag-rim); clip-path: polygon(var(--inv-tag-cut) 0, 100% 0, 100% calc(100% - var(--inv-tag-cut)), calc(100% - var(--inv-tag-cut)) 100%, 0 100%, 0 var(--inv-tag-cut)); white-space: nowrap; }
.stack { inset-inline-end: var(--space-xs); bottom: var(--space-sm); }
.equipped { inset-inline-start: var(--space-xs); top: var(--space-xs); padding: 0 var(--space-xs); color: var(--inv-equipped-ink); background: var(--inv-equipped-fill); box-shadow: none; }
.locked { inset-inline-end: var(--space-xs); top: var(--space-xs); inline-size: calc(20 * var(--font-step-px)); block-size: calc(20 * var(--font-step-px)); padding: 0; }
.locked og-icon { inline-size: 64%; block-size: 64%; }
.fresh { position: absolute; inset-inline-end: var(--space-sm); top: var(--space-sm); inline-size: var(--space-xs); block-size: var(--space-xs); border-radius: var(--inv-chip-radius); background: var(--inv-accent); box-shadow: var(--inv-fresh-glow); }
.fresh[data-locked=true] { inset-inline-end: calc(var(--space-xs) + 26 * var(--font-step-px)); }
/* Broken is said in words on the tile at every size, over its foot. */
.broken { position: absolute; inset-inline: 0; bottom: var(--space-xs); text-align: center; font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); color: var(--inv-broken-ink); white-space: nowrap; }
[data-broken=true] .stack { bottom: calc(var(--space-sm) + var(--inv-tag-size) * 1.6); }
.bracket { position: absolute; inset: 0; background: var(--inv-bracket); filter: var(--inv-bracket-glow); pointer-events: none; }
.empty { width: stretch; }
.empty-mark { width: var(--size-lg); height: var(--size-lg); }
.empty-text { margin: 0; }
/* The footer: the load as a leaning bar. */
.foot { flex: none; width: stretch; padding: var(--space-sm) var(--space-xs) 0; border-block-start: var(--inv-rule); }
.weight-line { align-items: baseline; }
.caption { min-width: 0; letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); }
.load { position: relative; margin-inline: var(--space-xs); block-size: calc(10 * var(--font-step-px)); transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
.load[data-over=true] { box-shadow: var(--inv-load-over-rim); }
.load-fill { position: absolute; inset-inline-start: 0; top: 0; bottom: 0; inline-size: calc(var(--fill, 0) * 100%); background: var(--inv-load-fill); }
.load-fill[data-tone=warn] { background: var(--inv-load-over-fill); }
.over { margin: 0; }
@media (width < 600px), (height < 500px) {
  [part="RpgBackpack.band"] { padding-block: var(--space-xs); }
  .grid { grid-template-columns: repeat(5, minmax(0, 1fr)); }
  .foot { padding-top: var(--space-xs); }
}
</style>
