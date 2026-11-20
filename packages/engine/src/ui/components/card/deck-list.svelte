<meta name="component" content="CardDeckList">
<meta name="strings" content="./card.en.ogt">
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
<section id="CardDeckList" part="frame" layout="stack" fill aria-label={title}>
  <header class="band" part="band" layout="row"><h2 class="title" part="title">{title}</h2></header>
  {#if limit}<div class="size grid" part="size" gap="tight" data-under={total < limit} class:over={total > limit}>
    <span class="size-label">{t("card.deck.size")}</span>
    <span class="size-figure">{t("card.deck.count", { count: total, limit })}</span>
    <div class="bar" role="meter" aria-label={t("card.deck.size")} aria-valuetext={t("card.deck.count", { count: total, limit })} style:--fill={limit ? (total > limit ? 1 : total / limit) : 0}><div class="bar-fill" tone={total > limit ? 'danger' : ''} class:tone-danger={(total > limit ? 'danger' : '') == 'danger'}></div></div>
  </div>{/if}
  {#if entries.length}
    <div class="rows" layout="scroll" fill>
      <div class="list" layout="stack" gap="hair" role="list" aria-label={title}>
        {#each entries as e (e.id)}
          <div class="row" part="row" layout="row" gap="hair" role="listitem" tier={e.rarity} aria-current={e.id == selected} class:type-attack={(e.type) == 'attack'} class:type-curse={(e.type) == 'curse'} class:type-power={(e.type) == 'power'} class:type-skill={(e.type) == 'skill'} class:type-status={(e.type) == 'status'} state:selected={e.id == selected}>
            <button class="pick" fill variant="quiet" aria-label={e.name} onclick={choose}><div class="pick-face" layout="row">
              <div part="cost" class="cost" layout="row" gap="none" aria-label={t("card.face.cost", { cost: e.cost })}><span class="cost-figure">{e.cost}</span></div>
              <span part="name" class="name" fill>{e.name}</span>
              <span part="copies" class="copies">{t("card.deck.copies", { count: e.copies })}</span>
            </div></button>
            <button part="step" class="minus step" variant="quiet" disabled={!e.canRemove} aria-label={t("card.deck.remove", { name: e.name })} onclick={remove} ondetails={remove}><span class="step-mark">{t("card.deck.less")}</span></button>
            <button part="step" class="plus step" variant="quiet" disabled={!e.canAdd} aria-label={t("card.deck.add", { name: e.name })} onclick={add}><span class="step-mark">{t("card.deck.more")}</span></button>
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="empty cover" layout="stack" fill role="status"><p class="empty-text">{t("card.deck.empty")}</p></div>
  {/if}
  {#if confirmLabel}<footer class="foot" layout="row">
    <button id="Confirm" class="go" variant="primary" disabled={!canConfirm} onclick={confirm}>{confirmLabel}</button>
  </footer>{/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--layout-tile), 100%), 1fr)); }
#CardDeckList { position: relative; border-radius: var(--inv-frame-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.band { width: stretch; justify-content: flex-start; flex: none; }
.title { margin: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.size { grid-template-columns: 1fr auto; width: stretch; flex: none; align-items: baseline; padding: 0 var(--space-xs); }
.size-label { font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--tcg-count-empty-ink); }
.size-figure { white-space: nowrap; font-size: var(--inv-figure-lg-size); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); color: var(--ink-bright); }
.over .size-figure { color: var(--tcg-over-ink); }
.bar { position: relative; grid-column: 1 / -1; height: var(--space-sm); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
.bar-fill { position: absolute; top: 0; bottom: 0; inset-inline-start: 0; inline-size: calc(var(--fill, 0) * 100%); background: var(--tcg-size-fill); }
.bar-fill.tone-danger { background: var(--tcg-size-over-fill); }
.list, .row { width: stretch; }
/* A row: its card type a rail on the leading edge, the shown row lit. */
.row { align-items: center; padding-inline-end: var(--space-xxs); background: var(--tcg-row-fill); box-shadow: var(--tcg-row-rail); color: var(--tcg-neutral); }
.row.type-attack { color: var(--tcg-attack); }
.row.type-skill { color: var(--tcg-skill); }
.row.type-power { color: var(--tcg-power); }
.row.type-curse, .row.type-status { color: var(--tcg-curse); }
.row:state(selected) { background: var(--tcg-row-on-fill); box-shadow: var(--tcg-row-on-rim); }
.pick { min-height: var(--size-touch); padding: 0 var(--space-sm); justify-content: flex-start; border: none; border-radius: 0; background: var(--inv-bare-fill); box-shadow: none; color: var(--ink-bright); }
.pick-face { width: stretch; align-items: center; gap: var(--space-sm); }
.cost { flex: none; inline-size: calc(var(--tcg-gem-size) * 0.85); block-size: calc(var(--tcg-gem-size) * 0.95); justify-content: center; align-items: center; clip-path: polygon(50% 0, 100% 25%, 100% 75%, 50% 100%, 0 75%, 0 25%); background: var(--tcg-gem-fill); color: var(--tcg-gem-ink); font-weight: var(--inv-figure-weight); }
.name { min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-weight: var(--inv-name-weight); }
.copies { flex: none; white-space: nowrap; font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); color: var(--ink-bright); }
.step { flex: none; width: var(--size-touch); min-height: var(--size-touch); padding: 0; align-items: center; justify-content: center; border: none; border-radius: 0; background: var(--tcg-step-fill); box-shadow: var(--tcg-step-rim); color: var(--ink-bright); }
.step:disabled { color: var(--tcg-count-empty-ink); }
.step-mark { text-align: center; }
.empty { width: stretch; }
.empty-text { margin: 0; }
.foot { width: stretch; flex: none; }
#Confirm { width: stretch; min-height: var(--size-touch); min-width: 0; }
</style>
