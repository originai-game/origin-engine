<meta name="component" content="CardPiles">
<meta name="strings" content="./card.en.ogt">
<meta name="parts" content="CardPiles.frame CardPiles.pile CardPiles.count CardPiles.label CardPiles.cell CardPiles.figure CardPiles.badge">
<meta name="prop:piles" content="list" data-doc="The piles in the game's order, keyed rows of { id, kind, label, icon, count, enabled }: kind is draw, discard, exhaust or the game's own word (it reaches the pile as data-kind); count the game's number; enabled false when the game will not open it.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the group; empty is Piles.">
<meta name="fixture:default" content='{"label":"","piles":[{"id":"draw","kind":"draw","label":"Draw","icon":"bag","count":14,"enabled":true},{"id":"discard","kind":"discard","label":"Discard","icon":"back","count":3,"enabled":true},{"id":"exhaust","kind":"exhaust","label":"Exhaust","icon":"fire","count":1,"enabled":true}]}'>
<meta name="fixture:empty" content='{"label":"","piles":[{"id":"draw","kind":"draw","label":"Draw","icon":"bag","count":0,"enabled":true},{"id":"discard","kind":"discard","label":"Discard","icon":"back","count":0,"enabled":true},{"id":"exhaust","kind":"exhaust","label":"Exhaust","icon":"fire","count":0,"enabled":false}]}'>
<meta name="fixture:overflow" content='{"label":"","piles":[{"id":"draw","kind":"draw","label":"Remaining in the draw pile","icon":"bag","count":1284,"enabled":true},{"id":"discard","kind":"discard","label":"Discarded this combat","icon":"back","count":312,"enabled":true},{"id":"exhaust","kind":"exhaust","label":"Exhausted for good","icon":"fire","count":96,"enabled":false}]}'>

<!--
  The draw, discard and exhaust piles as buttons: an icon, the game's
  count and a label. A pile at 0 carries data-empty; one the game will not
  open is disabled. Reports open (a pile's key). The piles sit in a row
  spread across the width they are given; a page that wants the draw pile
  in one corner and the discard in the other mounts two.
-->
<nav id="CardPiles" part="CardPiles.frame" class="bare" data-layout="row" aria-label={label || t("card.piles.name")}>
  {#each piles as p (p.id)}
    <button part="CardPiles.pile CardPiles.cell" data-kind={p.kind} data-empty={p.count == 0} disabled={!p.enabled} aria-label={t("card.piles.count", { label: p.label, count: p.count })} onclick={open}><div class="face" data-layout="stack" data-gap="hair">
      <div class="mark" data-layout="row" data-gap="none"><og-icon class="icon" icon={p.icon}></og-icon><span part="CardPiles.count CardPiles.figure CardPiles.badge" class="count">{formatNumber(p.count)}</span></div>
      <span part="CardPiles.label" class="label">{p.label}</span>
    </div></button>
  {/each}
</nav>

<style>
:host { inline-size: stretch; align-items: flex-end; justify-content: space-between; }
[part="CardPiles.pile CardPiles.cell"] { flex: 0 1 auto; min-width: var(--size-touch); min-height: var(--size-touch); padding: var(--space-xs); align-items: center; justify-content: center; }
[part="CardPiles.pile CardPiles.cell"] .face { align-items: center; min-width: 0; }
.mark { position: relative; width: var(--size-lg); height: var(--size-lg); justify-content: center; }
.icon { width: 70%; height: 70%; }
[part~="CardPiles.count"] { position: absolute; inset-inline-end: calc(var(--space-xs) * -1); bottom: calc(var(--space-xxs) * -1); min-width: var(--size-xs); text-align: center; white-space: nowrap; }
[part="CardPiles.label"] { max-width: calc(var(--size-xxl) * 1.5); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; text-align: center; }
</style>
