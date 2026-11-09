<meta name="component" content="RpgInventory">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="Inventory.item Inventory.category Inventory.frame Inventory.card">
<meta name="prop:title" content="string" data-doc="The bag's heading (Backpack, Stash, a container's name).">
<meta name="prop:filters" content="list" data-doc="The category tabs, a keyed collection of { id, label, count }: count is the game's number of items in it.">
<meta name="prop:filter" content="string" data-doc="The id of the category shown.">
<meta name="prop:items" content="list" data-doc="The items shown, in the game's order (it filters and sorts): each { id, name, icon, quantity, rarity, kind, equipped, fresh, locked, broken }. rarity is common, uncommon, rare, epic, legendary or mythic; quantity above 1 shows a count; fresh marks an unseen item, locked one the game will not let go (a quest item), broken one the game will not equip.">
<meta name="prop:selected" content="string" data-doc="The id of the selected item; empty is none. Selection follows the id through reorders and removals.">
<meta name="prop:weight" content="string" data-doc="The carried load as the game words it (42.5 / 60); empty hides the load.">
<meta name="prop:load" content="number" data-doc="The load 0 to 1 for its bar; above 1 is overburdened.">
<meta name="prop:gold" content="string" data-doc="The purse, formatted by the game (1,284); empty hides it.">
<meta name="prop:detailName" content="string" data-doc="The selected item's name over its card; empty hides the card.">
<meta name="prop:detailIcon" content="string" data-doc="The selected item's icon.">
<meta name="prop:detailRarity" content="string" data-doc="Its rarity, as in items.">
<meta name="prop:detailKind" content="string" data-doc="Its type line (One-handed sword · Rare).">
<meta name="prop:detailText" content="string" data-doc="Its description.">
<meta name="prop:detailStats" content="list" data-doc="Its numbers as the game computes them: each { id, label, value, delta, change }: delta the game's comparison with what is equipped (+4), change better, worse or same.">
<meta name="prop:detailValue" content="string" data-doc="Its value line (Worth 240); empty hides it.">
<meta name="prop:canUse" content="bool" data-doc="Use is offered (a consumable).">
<meta name="prop:canEquip" content="bool" data-doc="Equip is offered; false greys it.">
<meta name="prop:equipLabel" content="string" data-doc="The equip button's label when it is not Equip (Unequip).">
<meta name="prop:canDrop" content="bool" data-doc="Drop is offered; false (a locked item) greys it.">
<meta name="action:details" content="Equip">
<meta name="action:tabPrev" content="Previous category">
<meta name="action:tabNext" content="Next category">
<meta name="fixture:default" content='{"title":"Backpack","filter":"all","filters":[{"id":"all","label":"All","count":9},{"id":"gear","label":"Gear","count":5},{"id":"use","label":"Consumables","count":3},{"id":"quest","label":"Quest","count":1}],"items":[{"id":"blade","name":"Emberwake Blade","icon":"sword","quantity":1,"rarity":"epic","kind":"Sword","equipped":true,"fresh":false,"locked":false,"broken":false},{"id":"shield","name":"Oakheart Shield","icon":"shield","quantity":1,"rarity":"rare","kind":"Shield","equipped":false,"fresh":true,"locked":false,"broken":false},{"id":"potion","name":"Healing Draught","icon":"potion","quantity":12,"rarity":"common","kind":"Potion","equipped":false,"fresh":false,"locked":false,"broken":false},{"id":"key","name":"Ferryman’s Key","icon":"key","quantity":1,"rarity":"uncommon","kind":"Quest item","equipped":false,"fresh":false,"locked":true,"broken":false},{"id":"gem","name":"Moonstone","icon":"gem","quantity":3,"rarity":"legendary","kind":"Gem","equipped":false,"fresh":false,"locked":false,"broken":false}],"selected":"shield","weight":"42.5 / 60","load":0.71,"gold":"1,284","detailName":"Oakheart Shield","detailIcon":"shield","detailRarity":"rare","detailKind":"Shield · Rare","detailText":"Bands of bog-oak bound in iron. It drinks the first blow of every fight.","detailStats":[{"id":"armour","label":"Armour","value":"38","delta":"+6","change":"better"},{"id":"block","label":"Block","value":"22%","delta":"+4%","change":"better"},{"id":"weight","label":"Weight","value":"6.0","delta":"+1.5","change":"worse"}],"detailValue":"Worth 240","canUse":false,"canEquip":true,"equipLabel":"","canDrop":true}'>
<meta name="fixture:empty" content='{"title":"Backpack","filter":"all","filters":[{"id":"all","label":"All","count":0}],"items":[],"selected":"","weight":"0 / 60","load":0,"gold":"0","detailName":"","detailIcon":"","detailRarity":"","detailKind":"","detailText":"","detailStats":[],"detailValue":"","canUse":false,"canEquip":false,"equipLabel":"","canDrop":false}'>

<!--
  The rpg kit's bag (README.md): category tabs, a grid of item slots, the
  load and the purse, and the selected item's card with its comparison
  and actions. The game owns every rule: what a category holds, the order,
  the numbers and their comparison, and whether an action is allowed; the
  bag reports choose (the item's key), category, use, equip and drop.
  The grid wraps to the width it is given; the card sits beside it on a
  wide screen and under it on a narrow one.
-->
<section id="RpgInventory" part="Inventory.frame" aria-label={title}>
  <header id="Head">
    <h2 id="Title">{title}</h2>
    {#if gold}
    <div id="Purse" aria-label={t("rpg.inventory.gold", { gold })}><icon class="glyph" icon="coin"></icon><span class="number">{gold}</span></div>
    {/if}
  </header>
  <div id="Categories" role="tablist" aria-label={t("rpg.inventory.categories")} ontabNext={categoryNext} ontabPrev={categoryPrev}>
    <kbd id="PrevKey" data-action="tabPrev"></kbd>
    {#each filters as f (f.id)}
      <button id="Category" part="Inventory.category" class="category" variant="quiet" role="tab" aria-selected={f.id == filter} onclick={category}>
        <span class="label">{f.label}</span><span class="count">{formatNumber(f.count)}</span>
      </button>
    {/each}
    <kbd id="NextKey" data-action="tabNext"></kbd>
  </div>
  <div id="Body">
    <div id="Bag">
      {#if !items.length}
        <div id="Empty" role="status"><icon class="mark" icon="bag"></icon><p class="quiet">{t("rpg.inventory.empty")}</p></div>
      {:else}
        <scroll id="Scroll">
          <div id="Grid" role="grid" aria-label={title}>
            {#each items as i (i.id)}
              <button id="Item" part="Inventory.item" class="item" class:uncommon={i.rarity == "uncommon"} class:rare={i.rarity == "rare"} class:epic={i.rarity == "epic"} class:legendary={i.rarity == "legendary"} class:mythic={i.rarity == "mythic"} class:broken={i.broken} role="gridcell" aria-selected={i.id == selected} aria-label={i.name} aria-description={i.kind} onclick={choose}>
                <icon class="art" icon={i.icon}></icon>
                <div class="ring"></div>
                <div class="rarity" hidden={i.rarity == "common"}></div>
                <span class="quantity" hidden={!(i.quantity > 1)}>{formatNumber(i.quantity)}</span>
                <div class="flag equipped" hidden={!i.equipped} aria-label={t("rpg.inventory.equipped")}><icon icon="check"></icon></div>
                <div class="flag fresh" hidden={!i.fresh} aria-label={t("rpg.inventory.new")}></div>
                <div class="flag locked" hidden={!i.locked} aria-label={t("rpg.inventory.locked")}><icon icon="lock"></icon></div>
              </button>
            {/each}
          </div>
        </scroll>
      {/if}
      {#if weight}
      <div id="Load">
        <span class="caption">{t("rpg.inventory.load")}</span>
        <meter id="LoadBar" value={load} min="0" max="1" high="0.85" optimum="0" aria-label={t("rpg.inventory.load")} aria-valuetext={weight}></meter>
        <span class="number" class:over={load > 1}>{weight}</span>
      </div>
      {/if}
    </div>
    <aside id="Card" part="Inventory.card" hidden={!detailName} aria-label={detailName}>
      <div id="CardHead">
        <div id="CardArt" part="Inventory.item" class="item large" class:uncommon={detailRarity == "uncommon"} class:rare={detailRarity == "rare"} class:epic={detailRarity == "epic"} class:legendary={detailRarity == "legendary"} class:mythic={detailRarity == "mythic"}><icon class="art" icon={detailIcon}></icon></div>
        <div id="CardTitle">
          <h3 id="Name" class:uncommon={detailRarity == "uncommon"} class:rare={detailRarity == "rare"} class:epic={detailRarity == "epic"} class:legendary={detailRarity == "legendary"} class:mythic={detailRarity == "mythic"}>{detailName}</h3>
          <p class="quiet">{detailKind}</p>
        </div>
      </div>
      <div id="Stats" role="list">
        {#each detailStats as s (s.id)}
          <div class="stat" role="listitem">
            <span class="what">{s.label}</span>
            <span class="number value">{s.value}</span>
            {#if s.delta}
              <div class="delta number" class:better={s.change == "better"} class:worse={s.change == "worse"}>
                <icon class="trend up" icon="play" hidden={s.change != "better"}></icon><icon class="trend down" icon="play" hidden={s.change != "worse"}></icon><span>{s.delta}</span>
              </div>
            {/if}
          </div>
        {/each}
      </div>
      <p id="Lore">{detailText}</p>
      <p id="Value" class="quiet" hidden={!detailValue}>{detailValue}</p>
      <div id="Actions">
        <button id="Use" variant="secondary" hidden={!canUse} onclick={use}>{t("rpg.inventory.use")}</button>
        <button id="Equip" variant="primary" disabled={!canEquip} onclick={equip} ondetails={equip}><kbd id="EquipKey" data-action="details"></kbd><span id="EquipLabel">{equipLabel || t("rpg.inventory.equip")}</span></button>
        <button id="Drop" variant="ghost" disabled={!canDrop} onclick={drop}>{t("rpg.inventory.drop")}</button>
      </div>
    </aside>
  </div>
</section>

<style>
#RpgInventory { width: stretch; gap: var(--space-md); padding: var(--space-lg); background: var(--panel); border-radius: var(--radius-lg); container-type: inline-size; }
#Head { display: flex; width: stretch; align-items: center; justify-content: space-between; gap: var(--space-md); }
#Title { margin: 0; font: var(--text-xl); font-family: fantasy; color: var(--ink); }
#Purse { display: flex; gap: var(--space-xs); align-items: center; color: var(--gold); }
.glyph { width: var(--size-xs); height: var(--size-xs); color: var(--gold); }
.number { font-family: monospace; font-variant-numeric: tabular-nums; white-space: nowrap; }
#Categories { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-xs); width: stretch; border-bottom: var(--stroke-hairline) solid var(--line); padding-bottom: var(--space-xs); }
.category { min-height: var(--size-touch); padding: 0 var(--space-md); gap: var(--space-xs); }
.category[aria-selected=true] { color: var(--highlight); box-shadow: inset 0 calc(-1 * var(--stroke-thick)) 0 0 var(--highlight); }
.category .count { font: var(--text-caption); color: var(--ink-faint); }
#Body { display: flex; width: stretch; gap: var(--space-lg); align-items: start; }
#Bag { width: stretch; gap: var(--space-md); }
#Scroll { width: stretch; max-height: calc(6 * var(--size-xl) + 5 * var(--space-xs)); }
#Grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(var(--size-xl), 1fr)); gap: var(--space-xs); width: stretch; }
.item { position: relative; aspect-ratio: 1; width: stretch; min-width: var(--size-touch); padding: var(--space-sm); align-items: center; justify-content: center; background: var(--well); border-radius: var(--radius-sm); }
.item.broken .art { opacity: 0.45; }
.ring { display: none; position: absolute; left: 0; top: 0; width: stretch; height: stretch; border: 2px solid var(--highlight); border-radius: var(--radius-sm); box-shadow: inset 0 0 10px 0 var(--highlight); }
.item[aria-selected=true] .ring { display: block; }
.rarity { position: absolute; left: 22%; right: 22%; bottom: var(--space-xxs); height: 3px; border-radius: var(--radius-pill); background: var(--ink-faint); }
.uncommon .rarity { background: var(--uncommon); }
.rare .rarity { background: var(--rare); }
.epic .rarity { background: var(--epic); }
.legendary .rarity { background: var(--legendary); }
.mythic .rarity { background: var(--mythic); }
.art { width: stretch; height: stretch; max-width: var(--size-lg); max-height: var(--size-lg); color: var(--ink); }
.quantity { position: absolute; right: var(--space-xs); bottom: var(--space-xxs); font: var(--text-caption); font-family: monospace; color: var(--ink); text-shadow: 0 1px 2px var(--shadow); }
.flag { position: absolute; width: var(--space-lg); height: var(--space-lg); align-items: center; justify-content: center; border-radius: var(--radius-pill); }
.flag icon { width: var(--space-md); height: var(--space-md); }
.flag.equipped { left: var(--space-xxs); top: var(--space-xxs); background: var(--highlight); color: var(--ink-inverse); }
.flag.fresh { right: var(--space-xs); top: var(--space-xs); width: var(--space-sm); height: var(--space-sm); background: var(--accent); }
.flag.locked { right: var(--space-xxs); top: var(--space-xxs); color: var(--ink-muted); }
#Empty { width: stretch; padding: var(--space-xl); gap: var(--space-sm); align-items: center; }
.mark { width: var(--size-lg); height: var(--size-lg); color: var(--ink-faint); }
.quiet { margin: 0; color: var(--ink-muted); font: var(--text-sm); }
.caption { font: var(--text-caption); color: var(--ink-muted); text-transform: uppercase; }
#Load { display: flex; width: stretch; gap: var(--space-sm); align-items: center; }
#LoadBar { width: stretch; height: var(--space-sm); }
.over { color: var(--danger); }
#Card { width: calc(18 * var(--space-lg)); flex-shrink: 0; gap: var(--space-md); padding: var(--space-lg); background: var(--panel-raised); border-radius: var(--radius-md); }
#CardHead { display: flex; gap: var(--space-md); align-items: center; width: stretch; }
.item.large { width: var(--size-xl); height: var(--size-xl); flex-shrink: 0; }
#CardTitle { width: stretch; gap: var(--space-xxs); }
#Name { margin: 0; font: var(--text-lg); font-family: fantasy; color: var(--ink); }
#Name.uncommon { color: var(--uncommon); }
#Name.rare { color: var(--rare); }
#Name.epic { color: var(--epic); }
#Name.legendary { color: var(--legendary); }
#Name.mythic { color: var(--mythic); }
#Stats { margin: 0; width: stretch; gap: var(--space-xxs); border-block: var(--stroke-hairline) solid var(--line); padding-block: var(--space-sm); }
.stat { display: grid; grid-template-columns: 1fr auto auto; gap: var(--space-sm); width: stretch; align-items: baseline; }
.stat .what { color: var(--ink-muted); }
.stat .value { color: var(--ink); text-align: end; }
.delta { display: flex; gap: var(--space-xxs); align-items: center; min-width: calc(4 * var(--space-md)); justify-content: end; color: var(--ink-muted); }
.delta.better { color: var(--success); }
.delta.worse { color: var(--danger); }
.trend { width: var(--space-md); height: var(--space-md); }
.trend.up { rotate: -90deg; }
.trend.down { rotate: 90deg; }
#Lore { margin: 0; width: stretch; color: var(--ink-muted); font: var(--text-reading); font-style: italic; }
#Actions { display: flex; flex-wrap: wrap; gap: var(--space-sm); width: stretch; }
#Actions button { flex: 1 1 auto; min-height: var(--size-touch); }
@container (width < 640px) {
  #Body { flex-direction: column; }
  #Card { width: stretch; }
}
@media (pointer: coarse) { kbd { display: none; } }
</style>
