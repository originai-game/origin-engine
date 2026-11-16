<meta name="component" content="RpgLoot">
<meta name="strings" content="./economy.en.ogt">
<meta name="action:back" content="Close">
<meta name="parts" content="RpgLoot.frame RpgLoot.title RpgLoot.items RpgLoot.row RpgLoot.icon RpgLoot.detail RpgLoot.name RpgLoot.description RpgLoot.count RpgLoot.reason RpgLoot.take RpgLoot.actions RpgLoot.all RpgLoot.close RpgLoot.status RpgLoot.empty">
<meta name="prop:title" content="string" data-default="" data-doc="The source's name; empty uses Loot.">
<meta name="prop:items" content="list" data-default="[]" data-doc="Keyed { id, name, icon, count, description, available, reason }. The game supplies legality/capacity and removes collected rows after take or takeAll.">
<meta name="prop:canTakeAll" content="bool" data-default="false" data-doc="The game permits taking the displayed items together. Empty/loading/busy never sends takeAll.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Rewards are loading; existing items remain visible but cannot be collected.">
<meta name="prop:busy" content="bool" data-default="false" data-doc="A collection request is pending; disables take and takeAll to prevent duplicate requests.">
<meta name="prop:error" content="string" data-default="" data-doc="The game's collection failure, such as a full inventory; empty hides it.">
<meta name="fixture:default" content='{"title":"Abbey chest","items":[{"id":"gold","name":"Gold","icon":"coin","count":120,"description":"Recovered from the chest.","available":true,"reason":""},{"id":"relic","name":"The Abbey Bell","icon":"gem","count":1,"description":"Unique relic","available":false,"reason":"Inventory full"}],"canTakeAll":false,"loading":false,"busy":false,"error":""}'>
<meta name="fixture:empty" content='{}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:busy" content='{"busy":true,"items":[{"id":"gold","name":"Gold","icon":"coin","count":120,"description":"Collecting…","available":true,"reason":""}],"canTakeAll":true}'>
<meta name="fixture:error" content='{"error":"Make room in your inventory and try again.","items":[{"id":"relic","name":"The Abbey Bell","icon":"gem","count":1,"description":"Unique relic","available":false,"reason":"Inventory full"}]}'>

<section id="RpgLoot" part="RpgLoot.frame" aria-label={title || t("rpg.loot.title")} aria-busy={loading || busy} onback={close}>
  <h2 part="RpgLoot.title">{title || t("rpg.loot.title")}</h2>
  {#if error}<p part="RpgLoot.status" role="alert">{error}</p>{/if}
  {#if loading || busy}<div part="RpgLoot.status" role="status"><Spinner name={busy ? t("rpg.economy.busy") : t("rpg.economy.loading")} /><span>{busy ? t("rpg.economy.busy") : t("rpg.economy.loading")}</span></div>{/if}
  {#if items.length}<ul part="RpgLoot.items">
    {#each items as item (item.id)}<li part="RpgLoot.row">
      <og-icon part="RpgLoot.icon" icon={item.icon}></og-icon><div part="RpgLoot.detail"><h3 part="RpgLoot.name">{item.name}</h3>{#if item.description}<p part="RpgLoot.description">{item.description}</p>{/if}{#if !item.available}<p part="RpgLoot.reason">{item.reason || t("rpg.economy.unavailable")}</p>{/if}</div><div part="RpgLoot.count">{t("rpg.loot.count", { count: formatNumber(item.count) })}</div><button part="RpgLoot.take" disabled={!item.available || loading || busy} aria-label={t("rpg.loot.takeItem", { name: item.name })} onclick={take}>{t("rpg.loot.take")}</button>
    </li>{/each}
  </ul>{:else if !loading}<p part="RpgLoot.empty" role="status">{t("rpg.loot.empty")}</p>{/if}
  <footer part="RpgLoot.actions"><button part="RpgLoot.close" onclick={close}>{t("rpg.economy.close")}</button><button part="RpgLoot.all" class="primary" disabled={!items.length || !canTakeAll || loading || busy} onclick={takeAll}>{t("rpg.loot.all")}</button></footer>
</section>

<style>
:host { display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; min-block-size: 0; gap: var(--space-md); container-type: inline-size; }
h2, h3, p { margin: 0; }
[part="RpgLoot.title"], [part="RpgLoot.status"] { flex: none; }
[part="RpgLoot.items"] { display: flex; flex-direction: column; flex: 1; min-block-size: 0; overflow-y: auto; gap: var(--space-sm); list-style: none; margin: 0; padding: 0; }
[part="RpgLoot.row"] { display: flex; flex: none; flex-wrap: wrap; align-items: center; gap: var(--space-sm); padding: var(--space-sm); }
[part="RpgLoot.icon"] { inline-size: var(--size-md); block-size: var(--size-md); flex: none; }
[part="RpgLoot.detail"] { display: flex; flex-direction: column; flex: 1; gap: var(--space-xs); }
[part="RpgLoot.name"] { font-size: var(--font-step-md); font-weight: var(--font-weight-semibold); }
[part="RpgLoot.description"], [part="RpgLoot.reason"], [part="RpgLoot.empty"] { color: var(--ink-muted); font: var(--text-caption); }
[part="RpgLoot.count"] { font-family: var(--figure-font); white-space: nowrap; flex: none; }
[part="RpgLoot.actions"] { display: flex; flex: none; justify-content: space-between; gap: var(--space-sm); margin-block-start: auto; }
button { min-block-size: var(--size-touch); }
[part="RpgLoot.status"] { padding: var(--space-sm); background: var(--well); border-radius: var(--radius-control); }
[part="RpgLoot.status"][role=status] { display: flex; align-items: center; gap: var(--space-sm); }
[part="RpgLoot.status"][role=alert] { color: var(--danger); }
@container (width < 400px) { [part="RpgLoot.take"] { flex-basis: 100%; } }
</style>
