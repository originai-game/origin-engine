<meta name="component" content="RpgLoot">
<meta name="strings" content="./economy.en.ogt">
<meta name="action:back" content="Close">
<meta name="parts" content="RpgLoot.frame RpgLoot.band RpgLoot.kicker RpgLoot.title RpgLoot.items RpgLoot.row RpgLoot.socket RpgLoot.icon RpgLoot.detail RpgLoot.name RpgLoot.description RpgLoot.count RpgLoot.reason RpgLoot.take RpgLoot.actions RpgLoot.all RpgLoot.close RpgLoot.status RpgLoot.empty RpgLoot.caption RpgLoot.figure RpgLoot.well">
<meta name="prop:title" content="string" data-default="" data-doc="The source's name; empty uses Loot.">
<meta name="prop:items" content="list" data-default="[]" data-doc="Keyed { id, name, icon, count, description, available, reason, rarity }. rarity is common, uncommon, rare, epic, legendary or mythic (empty is common): the row's edge, the icon's socket and the name take its colour. The game supplies legality/capacity and removes collected rows after take or takeAll.">
<meta name="prop:canTakeAll" content="bool" data-default="false" data-doc="The game permits taking the displayed items together. Empty/loading/busy never sends takeAll.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Rewards are loading; existing items remain visible but cannot be collected.">
<meta name="prop:busy" content="bool" data-default="false" data-doc="A collection request is pending; disables take and takeAll to prevent duplicate requests.">
<meta name="prop:error" content="string" data-default="" data-doc="The game's collection failure, such as a full inventory; empty hides it.">
<meta name="fixture:default" content='{"title":"Abbey chest","items":[{"id":"ember","name":"Heart of the Ember","icon":"fire","count":1,"description":"Crafting reagent","available":true,"reason":"","rarity":"legendary"},{"id":"gold","name":"Gold","icon":"coin","count":120,"description":"Recovered from the chest.","available":true,"reason":"","rarity":"common"},{"id":"relic","name":"The Abbey Bell","icon":"gem","count":1,"description":"Unique relic","available":false,"reason":"Inventory full","rarity":"epic"}],"canTakeAll":true,"loading":false,"busy":false,"error":""}'>
<meta name="fixture:empty" content='{}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:busy" content='{"busy":true,"items":[{"id":"gold","name":"Gold","icon":"coin","count":120,"description":"Collecting…","available":true,"reason":"","rarity":"common"}],"canTakeAll":true}'>
<meta name="fixture:error" content='{"error":"Make room in your inventory and try again.","items":[{"id":"relic","name":"The Abbey Bell","icon":"gem","count":1,"description":"Unique relic","available":false,"reason":"Inventory full","rarity":"epic"}]}'>

<!--
  What a chest, a body or a reward holds: a band with the source's name and
  how many things it holds, one row per item (its icon in a socket and its
  rarity on the row's edge, the name in that colour over what it is and,
  when the game refuses it, why in words; the count on the name's line and
  Take), and Close beside Take all. The game decides what can be taken; the
  panel reports take (an item's key), takeAll and close (Close, or Escape
  and pad B anywhere in it). The rows scroll inside the height given.
-->
<section id="RpgLoot" part="RpgLoot.frame" data-layout="shell" data-gap="loose" aria-label={title || t("rpg.loot.title")} aria-busy={loading || busy} onback={close}>
  <div class="lit"></div>
  <header part="RpgLoot.band" class="band" data-layout="stack" data-gap="hair">
    <span part="RpgLoot.kicker" class="kicker">{items.length ? t("rpg.loot.kicker", { count: formatNumber(items.length) }) : t("rpg.loot.title")}</span>
    <h2 part="RpgLoot.title">{title || t("rpg.loot.title")}</h2>
  </header>
  {#if error}<p part="RpgLoot.status" class="alert" data-tone="danger" role="alert">{error}</p>{/if}
  {#if loading || busy}<div part="RpgLoot.status RpgLoot.well" class="wait" data-layout="row" role="status"><Spinner name={busy ? t("rpg.economy.busy") : t("rpg.economy.loading")} /><span>{busy ? t("rpg.economy.busy") : t("rpg.economy.loading")}</span></div>{/if}
  {#if items.length}<ul part="RpgLoot.items" data-layout="scroll" data-fill>
    {#each items as item (item.id)}<li part="RpgLoot.row" class="row" data-rarity={item.rarity || 'common'} data-available={item.available}>
      <div part="RpgLoot.socket" class="socket" data-layout="cover"><og-icon part="RpgLoot.icon" icon={item.icon}></og-icon></div>
      <div part="RpgLoot.detail" class="detail" data-layout="stack" data-gap="hair" data-fill>
        <div class="line" data-layout="row"><h3 part="RpgLoot.name" class="name" data-fill aria-label={item.name}>{item.name}</h3><span part="RpgLoot.count RpgLoot.figure" class="count">{t("rpg.loot.count", { count: formatNumber(item.count) })}</span></div>
        {#if !item.available}<p part="RpgLoot.reason" class="reason" data-tone="danger">{item.reason || t("rpg.economy.unavailable")}</p>{:else if item.description}<p part="RpgLoot.description RpgLoot.caption" class="description">{item.description}</p>{/if}
      </div>
      <button part="RpgLoot.take" class="take" disabled={!item.available || loading || busy} aria-label={t("rpg.loot.takeItem", { name: item.name })} onclick={take}>{t("rpg.loot.take")}</button>
    </li>{/each}
  </ul>{:else if !loading}<p part="RpgLoot.empty RpgLoot.caption" class="empty" role="status">{t("rpg.loot.empty")}</p>{/if}
  <footer part="RpgLoot.actions" class="actions" data-layout="row"><button part="RpgLoot.close" class="act" aria-label={t("rpg.economy.close")} onclick={close}><kbd class="cap" data-action="back"></kbd><span>{t("rpg.economy.close")}</span></button><button part="RpgLoot.all" class="go act all" variant="primary" disabled={!items.length || !canTakeAll || loading || busy} onclick={takeAll}>{t("rpg.loot.all")}</button></footer>
</section>

<style>
/* The frame: two corners cut, a lit top edge. */
[part="RpgLoot.frame"] { position: relative; min-block-size: 0; border-radius: var(--inv-frame-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
h2, h3, p { margin: 0; }
.band { flex: none; width: stretch; }
.kicker { font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-kicker-ink); white-space: nowrap; }
[part="RpgLoot.title"] { overflow-wrap: anywhere; }
.alert { flex: none; padding: var(--space-xs) var(--space-sm); background: var(--inv-alert-fill); box-shadow: var(--inv-alert-rim); }
.wait { flex: none; padding: var(--space-sm); }
[part="RpgLoot.items"] { list-style: none; margin: 0; padding: 0; gap: var(--space-xs); }
/* A row: the item's rarity on its leading edge, its icon in a socket. */
.row { --inv-rarity: var(--common); --inv-socket-fill: var(--cell-common-fill); --inv-socket-rim: var(--cell-common-shadow); position: relative; flex: none; display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; column-gap: var(--space-sm); row-gap: var(--space-xs); padding: var(--space-xs) var(--space-sm) var(--space-xs) var(--space-md); background: var(--inv-row-fill); box-shadow: var(--inv-row-rim); border-inline-start: var(--inv-edge); border-inline-start-color: var(--inv-rarity); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.row:hover { background: var(--inv-row-hover-fill); }
.row:focus-within { background: var(--inv-row-on-fill); box-shadow: var(--inv-row-on-rim); }
.row[data-rarity=uncommon] { --inv-rarity: var(--uncommon); --inv-socket-fill: var(--cell-uncommon-fill); --inv-socket-rim: var(--cell-uncommon-shadow); }
.row[data-rarity=rare] { --inv-rarity: var(--rare); --inv-socket-fill: var(--cell-rare-fill); --inv-socket-rim: var(--cell-rare-shadow); }
.row[data-rarity=epic] { --inv-rarity: var(--epic); --inv-socket-fill: var(--cell-epic-fill); --inv-socket-rim: var(--cell-epic-shadow); }
.row[data-rarity=legendary] { --inv-rarity: var(--legendary); --inv-socket-fill: var(--cell-legendary-fill); --inv-socket-rim: var(--cell-legendary-shadow); }
.row[data-rarity=mythic] { --inv-rarity: var(--mythic); --inv-socket-fill: var(--cell-mythic-fill); --inv-socket-rim: var(--cell-mythic-shadow); }
.row[data-available=false] .socket, .row[data-available=false] .name, .row[data-available=false] .count { opacity: var(--disabled-content-opacity); }
.socket { flex: none; align-self: flex-start; inline-size: calc(48 * var(--font-step-px)); block-size: calc(48 * var(--font-step-px)); background: var(--inv-socket-fill); box-shadow: var(--inv-socket-rim); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
[part="RpgLoot.icon"] { inline-size: 78%; block-size: 78%; filter: var(--inv-slot-art-shadow); }
.detail { min-width: 0; }
/* The count sits on the name's line, whatever the lines under it wrap to. */
.line { align-items: baseline; min-width: 0; }
.name { flex: 1 1 0; min-width: 0; overflow-wrap: anywhere; line-clamp: 2; overflow: hidden; color: var(--inv-rarity); }
.row[data-rarity=common] .name { color: var(--ink-bright); }
.count { flex: none; white-space: nowrap; font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
.reason { color: var(--inv-broken-ink); }
.take { flex: none; min-block-size: var(--size-touch); }
.empty { padding: var(--space-lg); text-align: center; }
.actions { flex: none; width: stretch; gap: var(--space-sm); margin-block-start: auto; }
.act { flex: 1; min-block-size: var(--size-touch); gap: var(--space-xs); }
.all { flex: 1.4; }
.cap { flex: none; }
/* Narrow: Take goes under the row's words, at the row's
   end, so the name keeps the row's width, and the description keeps two
   lines, so more than one item shows; the socket keeps its column. */
@container (width < 520px) {
  .socket { grid-row: span 2; }
  .take { grid-column: 2 / 4; justify-self: end; }
  .description { line-clamp: 2; overflow: hidden; }
}
</style>
