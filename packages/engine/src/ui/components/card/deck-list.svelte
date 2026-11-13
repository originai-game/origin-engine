<meta name="component" content="CardDeckList">
<meta name="strings" content="./card.en.ogt">
<meta name="parts" content="CardDeckList.frame CardDeckList.band CardDeckList.title CardDeckList.size CardDeckList.row CardDeckList.cost CardDeckList.name CardDeckList.copies CardDeckList.step">
<meta name="prop:title" content="string" data-doc="The deck's name as the player named it.">
<meta name="prop:entries" content="list" data-doc="The deck's cards in the game's order (by cost, then name), keyed rows of { id, name, cost, type, rarity, copies, canAdd, canRemove }: copies the game's count; canAdd and canRemove the game's answers (its copy limit, its deck limit, a card the deck must keep).">
<meta name="prop:total" content="number" data-doc="How many cards the deck holds (the game's count).">
<meta name="prop:limit" content="number" data-doc="The size the game requires or allows; 0 hides the size line. total above limit reaches the size as data-over, below as data-under.">
<meta name="prop:selected" content="string" data-doc="The entry shown in detail; empty is none.">
<meta name="prop:confirmLabel" content="string" data-doc="The main button's text (Save deck, Done); empty hides it.">
<meta name="prop:canConfirm" content="bool" data-doc="The main button is enabled (the game's verdict on the deck).">
<meta name="action:details" content="Remove one">
<meta name="fixture:default" content='{"title":"Ember Deck","total":12,"limit":30,"selected":"strike","confirmLabel":"Save deck","canConfirm":false,"entries":[{"id":"quick","name":"Quickstep","cost":"0","type":"skill","rarity":"uncommon","copies":2,"canAdd":true,"canRemove":true},{"id":"strike","name":"Ember Strike","cost":"1","type":"attack","rarity":"common","copies":3,"canAdd":false,"canRemove":true},{"id":"guard","name":"Iron Guard","cost":"1","type":"skill","rarity":"common","copies":2,"canAdd":true,"canRemove":true},{"id":"surge","name":"Arcane Surge","cost":"3","type":"power","rarity":"rare","copies":1,"canAdd":false,"canRemove":true},{"id":"oath","name":"Oath of Embers","cost":"2","type":"power","rarity":"epic","copies":1,"canAdd":false,"canRemove":false}]}'>
<meta name="fixture:empty" content='{"title":"New Deck","total":0,"limit":30,"selected":"","confirmLabel":"Save deck","canConfirm":false,"entries":[]}'>
<meta name="fixture:full" content='{"title":"Ember Deck","total":31,"limit":30,"selected":"","confirmLabel":"Save deck","canConfirm":false,"entries":[{"id":"strike","name":"Ember Strike","cost":"1","type":"attack","rarity":"common","copies":3,"canAdd":false,"canRemove":true},{"id":"lament","name":"Thousandfold Lamentation of the Drowned Cathedral","cost":"10","type":"power","rarity":"legendary","copies":1,"canAdd":false,"canRemove":true}]}'>

<!--
  The deck being built: its name, the size against the game's limit (a
  meter and the figures, data-over and data-under as well as the ink),
  one row per card with its cost, name and copies, a minus and a plus per
  row, and the main button. The game counts, sorts and rules; the list
  reports choose (a row's key), add and remove (a row's plus or minus,
  its key; X or pad Y anywhere in the focused row is its minus, so it is
  silent where the game refuses the removal) and confirm. The rows fill
  the height given and scroll inside it; the size and the button keep
  the foot.
-->
<section id="CardDeckList" part="CardDeckList.frame" aria-label={title}>
  <header part="CardDeckList.band"><h2 part="CardDeckList.title">{title}</h2></header>
  <div part="CardDeckList.size" hidden={!limit} data-over={total > limit} data-under={total < limit}>
    <span class="size-label">{t("card.deck.size")}</span>
    <span class="size-figure">{t("card.deck.count", { count: total, limit })}</span>
    <div class="bar" role="meter" aria-label={t("card.deck.size")} aria-valuetext={t("card.deck.count", { count: total, limit })} style:--fill={limit ? (total > limit ? 1 : total / limit) : 0}><div class="bar-fill"></div></div>
  </div>
  {#if entries.length}
    <scroll class="rows">
      <div class="list" role="list" aria-label={title}>
        {#each entries as e (e.id)}
          <div part="CardDeckList.row" role="listitem" data-type={e.type} data-rarity={e.rarity} data-selected={e.id == selected} aria-current={e.id == selected}>
            <button class="pick" variant="quiet" aria-label={e.name} onclick={choose}><div class="pick-face">
              <div part="CardDeckList.cost" class="cost" aria-label={t("card.face.cost", { cost: e.cost })}><span class="cost-figure">{e.cost}</span></div>
              <span part="CardDeckList.name" class="name">{e.name}</span>
              <span part="CardDeckList.copies" class="copies">{t("card.deck.copies", { count: e.copies })}</span>
            </div></button>
            <button part="CardDeckList.step" class="minus" variant="quiet" disabled={!e.canRemove} aria-label={t("card.deck.remove", { name: e.name })} onclick={remove} ondetails={remove}><span class="step-mark">{t("card.deck.less")}</span></button>
            <button part="CardDeckList.step" class="plus" variant="quiet" disabled={!e.canAdd} aria-label={t("card.deck.add", { name: e.name })} onclick={add}><span class="step-mark">{t("card.deck.more")}</span></button>
          </div>
        {/each}
      </div>
    </scroll>
  {:else}
    <div class="empty" role="status"><p class="empty-text">{t("card.deck.empty")}</p></div>
  {/if}
  <footer class="foot" hidden={!confirmLabel}>
    <button id="Confirm" class="primary" disabled={!canConfirm} onclick={confirm}>{confirmLabel}</button>
  </footer>
</section>

<style>
[part="CardDeckList.frame"][hidden], [part="CardDeckList.frame"] [hidden] { display: none; }
[part="CardDeckList.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-sm); }
[part="CardDeckList.band"] { display: flex; width: stretch; align-items: center; justify-content: center; flex: none; }
[part="CardDeckList.title"] { margin: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
[part="CardDeckList.size"] { display: grid; grid-template-columns: 1fr auto; width: stretch; flex: none; align-items: baseline; gap: var(--space-xs); padding: 0 var(--space-xs); }
.size-figure { font-variant-numeric: tabular-nums; white-space: nowrap; }
.bar { grid-column: 1 / -1; width: stretch; height: var(--space-sm); }
.bar-fill { width: calc(var(--fill, 0) * 100%); height: stretch; }
.rows { width: stretch; flex: 1; min-height: 0; }
.list { display: flex; flex-direction: column; width: stretch; gap: var(--space-xxs); }
[part="CardDeckList.row"] { display: flex; width: stretch; align-items: center; gap: var(--space-xxs); }
.pick { flex: 1; min-width: 0; min-height: var(--size-touch); padding: 0 var(--space-sm); justify-content: flex-start; }
.pick-face { display: flex; width: stretch; align-items: center; gap: var(--space-sm); }
[part="CardDeckList.cost"] { display: flex; flex: none; width: var(--size-sm); height: var(--size-sm); align-items: center; justify-content: center; font-variant-numeric: tabular-nums; }
[part="CardDeckList.name"] { flex: 1; min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
[part="CardDeckList.copies"] { flex: none; font-variant-numeric: tabular-nums; white-space: nowrap; }
[part="CardDeckList.step"] { flex: none; width: var(--size-touch); min-height: var(--size-touch); padding: 0; align-items: center; justify-content: center; }
.step-mark { text-align: center; }
.empty { display: flex; width: stretch; flex: 1; align-items: center; justify-content: center; }
.empty-text { margin: 0; }
.foot { display: flex; width: stretch; flex: none; }
#Confirm { width: stretch; min-height: var(--size-touch); }
</style>
