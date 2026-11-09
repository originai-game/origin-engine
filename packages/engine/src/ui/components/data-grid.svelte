<meta name="component" content="DataGrid">
<meta name="strings" content="./data-grid.en.ogt">
<meta name="prop:columns" content="list" data-doc="The headings, a keyed collection of { id, label, numeric }: numeric columns align to the end in tabular figures.">
<meta name="prop:tracks" content="string" data-doc="The columns' widths as CSS grid tracks, one per column (2fr 1fr 80px); empty shares the width equally between four.">
<meta name="prop:rows" content="list" data-doc="The rows, a keyed collection of { id, cells }: cells a keyed list of { id, text, numeric }, in the columns' order. Only the rows in view are built.">
<meta name="prop:selected" content="string" data-doc="The id of the selected row; empty is none.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the grid.">
<meta name="prop:loading" content="bool" data-doc="Rows are on their way: busy placeholder rows under the header.">
<meta name="prop:error" content="string" data-doc="An error to show with Retry instead of the rows; empty is none.">
<meta name="prop:empty" content="string" data-doc="What an empty grid says; empty is Nothing here yet.">
<meta name="prop:hint" content="string" data-doc="A line under the empty message.">
<meta name="action:choose" content="Select">
<meta name="action:retry" content="Retry">
<meta name="fixture:default" content='{"label":"Inventory","tracks":"2fr 1fr 64px 80px","selected":"b","loading":false,"error":"","empty":"","hint":"","columns":[{"id":"name","label":"Item","numeric":false},{"id":"kind","label":"Type","numeric":false},{"id":"qty","label":"Qty","numeric":true},{"id":"value","label":"Value","numeric":true}],"rows":[{"id":"a","cells":[{"id":"name","text":"Ember Blade","numeric":false},{"id":"kind","text":"Weapon","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"1,250","numeric":true}]},{"id":"b","cells":[{"id":"name","text":"Health Potion","numeric":false},{"id":"kind","text":"Consumable","numeric":false},{"id":"qty","text":"12","numeric":true},{"id":"value","text":"40","numeric":true}]},{"id":"c","cells":[{"id":"name","text":"Iron Shield","numeric":false},{"id":"kind","text":"Armour","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"380","numeric":true}]},{"id":"d","cells":[{"id":"name","text":"Silver Key","numeric":false},{"id":"kind","text":"Quest","numeric":false},{"id":"qty","text":"2","numeric":true},{"id":"value","text":"0","numeric":true}]},{"id":"e","cells":[{"id":"name","text":"Moonstone","numeric":false},{"id":"kind","text":"Gem","numeric":false},{"id":"qty","text":"3","numeric":true},{"id":"value","text":"900","numeric":true}]}]}'>

<section id="DataGrid" role="grid" aria-label={label} aria-busy={loading} style:--tracks={tracks || "1fr 1fr 1fr 1fr"}>
  <div id="Head" role="row">
    {#each columns as c (c.id)}<span class="heading" class:numeric={c.numeric} role="columnheader">{c.label}</span>{/each}
  </div>
  {#if loading}
    <div id="Loading" role="status" aria-label={t("component.data-grid.loading")}>
      <div class="ghost"></div><div class="ghost"></div><div class="ghost"></div><div class="ghost"></div>
    </div>
  {:else if error}
    <div id="Error" role="alert" aria-label={error}>
      <div class="mark danger"><icon icon="close"></icon></div>
      <p class="title">{error}</p>
      <button id="Retry" onclick={retry}>{t("component.data-grid.retry")}</button>
    </div>
  {:else if !rows.length}
    <div id="Empty" role="status">
      <div class="mark"><icon icon="bag"></icon></div>
      <p class="title">{empty || t("component.data-grid.empty")}</p>
      <p class="hint" hidden={!hint}>{hint}</p>
    </div>
  {:else}
    <list id="Rows" row-height="40">
      {#each rows as r (r.id)}
        <button id="Row" class="row" variant="quiet" role="row" aria-selected={r.id == selected} onclick={choose}>
          <div class="cells">
            {#each r.cells as c (c.id)}<span class="cell" class:numeric={c.numeric} role="gridcell">{c.text}</span>{/each}
          </div>
        </button>
      {/each}
    </list>
  {/if}
</section>

<style>
:root { --surface-saturation: 120%; }
@media (prefers-color-scheme: light) { :root { --surface-saturation: 200%; } }
#DataGrid { width: stretch; padding: 8px; border-radius: 10px;  backdrop-filter: blur(32px) saturate(var(--surface-saturation)); -og-material: glass thick lift; }
#Head, .cells { display: grid; grid-template-columns: var(--tracks); gap: 12px; width: stretch; }
#Head { padding: 6px 12px; border-bottom: 1px solid var(--line); }
.heading { font: var(--text-caption); color: var(--ink-muted); text-transform: uppercase; white-space: nowrap; }
.numeric { text-align: end; font-family: var(--font-mono); }
#Rows { width: stretch; height: 220px; }
.row { width: stretch; height: 40px; padding: 0 12px; border-radius: 6px; justify-content: start; }
.row[aria-selected=true] { background: #4c8dff2e; outline: 1px solid var(--primary); }
.cell { width: stretch; color: var(--ink); white-space: nowrap; text-overflow: ellipsis; }
#Empty, #Error { width: stretch; padding: 24px; gap: 8px; align-items: center; }
.mark { width: 48px; height: 48px; border-radius: 24px; align-items: center; justify-content: center; background: var(--control); }
.mark.danger { background: #e5484d29; color: var(--danger); }
.title { margin: 0; font: var(--text-base); color: var(--ink); text-align: center; width: stretch; }
.hint { margin: 0; font: var(--text-sm); color: var(--ink-muted); text-align: center; width: stretch; }
#Loading { width: stretch; gap: 6px; padding: 6px 0; }
.ghost { width: stretch; height: 28px; border-radius: 6px; background: linear-gradient(90deg, #ffffff00 30%, #ffffff1f 50%, #ffffff00 70%), var(--control-strong); }
</style>
