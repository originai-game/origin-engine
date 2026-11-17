<meta name="component" content="DataGrid">
<meta name="strings" content="./data-grid.en.ogt">
<meta name="parts" content="DataGrid.frame DataGrid.band DataGrid.heading DataGrid.row DataGrid.value DataGrid.ghost DataGrid.error DataGrid.empty DataGrid.mark DataGrid.message DataGrid.hint DataGrid.retry DataGrid.well DataGrid.caption DataGrid.headline DataGrid.skeleton DataGrid.sheen">
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

<section id="DataGrid" part="DataGrid.frame" data-layout="stack" data-gap="none" role="grid" aria-label={label} aria-busy={loading} style:--tracks={tracks || "1fr 1fr 1fr 1fr"}>
  <div id="Head" part="DataGrid.band" data-layout="grid" data-gap="loose" class="small" role="row">
    {#each columns as c (c.id)}<span part="DataGrid.heading DataGrid.caption" class="heading" class:numeric={c.numeric} role="columnheader">{c.label}</span>{/each}
  </div>
  {#if loading}
    <div id="Loading" data-layout="stack" data-gap="tight" role="status" aria-label={t("component.data-grid.loading")}>
      <div part="DataGrid.ghost DataGrid.skeleton" class="ghost"><div part="DataGrid.sheen" data-motion="shimmer"></div></div><div part="DataGrid.ghost DataGrid.skeleton" class="ghost"><div part="DataGrid.sheen" data-motion="shimmer"></div></div><div part="DataGrid.ghost DataGrid.skeleton" class="ghost"><div part="DataGrid.sheen" data-motion="shimmer"></div></div><div part="DataGrid.ghost DataGrid.skeleton" class="ghost"><div part="DataGrid.sheen" data-motion="shimmer"></div></div>
    </div>
  {:else if error}
    <div id="Error" part="DataGrid.error" data-layout="cover" role="alert" aria-label={error}>
      <div part="DataGrid.mark DataGrid.well" class="mark danger" data-layout="cover" data-tone="danger"><og-icon icon="close"></og-icon></div>
      <p part="DataGrid.message DataGrid.headline" class="title">{error}</p>
      <button id="Retry" part="DataGrid.retry" onclick={retry}>{t("component.data-grid.retry")}</button>
    </div>
  {:else if !rows.length}
    <div id="Empty" part="DataGrid.empty" data-layout="cover" role="status">
      <div part="DataGrid.mark DataGrid.well" class="mark" data-layout="cover"><og-icon icon="bag"></og-icon></div>
      <p part="DataGrid.message DataGrid.headline" class="title">{empty || t("component.data-grid.empty")}</p>
      {#if hint}<p part="DataGrid.hint DataGrid.caption" class="hint">{hint}</p>{/if}
    </div>
  {:else}
    <list id="Rows">
      {#each rows as r (r.id)}
        <button id="Row" part="DataGrid.row" class="row" variant="quiet" role="row" aria-selected={r.id == selected} onclick={choose}>
          <div class="cells" data-layout="grid" data-gap="loose">
            {#each r.cells as c (c.id)}<span part="DataGrid.value" class="cell" class:numeric={c.numeric} role="gridcell">{c.text}</span>{/each}
          </div>
        </button>
      {/each}
    </list>
  {/if}
</section>

<style>
:host { inline-size: stretch; }
#Head, .cells { grid-template-columns: var(--tracks); width: stretch; min-width: 0; }
#Head { padding-inline: var(--space-md); }
.heading { white-space: nowrap; }
.numeric { text-align: end; }
#Rows { width: stretch; height: var(--size-6xl); }
.row { width: stretch; height: var(--row); padding: 0 var(--space-md); justify-content: start; min-width: 0; }
.cell { width: stretch; text-align: start; white-space: nowrap; text-overflow: ellipsis; }
.cell.numeric { text-align: end; }
#Empty, #Error { width: stretch; block-size: auto; padding: var(--space-xl); }
.mark { width: var(--size-lg); height: var(--size-lg); }
.title, .hint { margin: 0; text-align: center; width: stretch; }
#Loading { padding: var(--space-xs) 0; }
.ghost { width: stretch; height: var(--control-sm); }
</style>
