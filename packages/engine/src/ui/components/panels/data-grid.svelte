<meta name="component" content="DataGrid">
<meta name="strings" content="./data-grid.en.ogt">
<meta name="parts" content="DataGrid.frame DataGrid.band DataGrid.heading DataGrid.row DataGrid.value DataGrid.ghost DataGrid.error DataGrid.empty DataGrid.mark DataGrid.message DataGrid.hint DataGrid.retry DataGrid.well DataGrid.caption DataGrid.headline DataGrid.skeleton DataGrid.sheen">
<meta name="prop:columns" content="list" data-doc="The headings, a keyed collection of { id, label, numeric }: numeric columns align to the end in tabular figures.">
<meta name="prop:tracks" content="string" data-doc="The columns' widths as CSS grid tracks, one per column; empty shares the width equally between four. A numeric column's track holds its widest figure in text-scaled units (calc(96 * var(--font-step-px))), so it grows with the player's text size; a figure is never cut, and the text columns (fr) give way.">
<meta name="prop:rows" content="list" data-doc="The rows, a keyed collection of { id, cells }: cells a keyed list of { id, text, numeric }, in the columns' order. Only the rows in view are built.">
<meta name="prop:selected" content="string" data-doc="The id of the selected row; empty is none.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the grid.">
<meta name="prop:loading" content="bool" data-doc="Rows are on their way: busy placeholder rows under the header.">
<meta name="prop:error" content="string" data-doc="An error to show with Retry instead of the rows; empty is none.">
<meta name="prop:empty" content="string" data-doc="What an empty grid says; empty is Nothing here yet.">
<meta name="prop:hint" content="string" data-doc="A line under the empty message.">
<meta name="action:choose" content="Select">
<meta name="action:retry" content="Retry">
<meta name="fixture:default" content='{"label":"Inventory","tracks":"2fr 1fr calc(48 * var(--font-step-px)) calc(96 * var(--font-step-px))","selected":"b","loading":false,"error":"","empty":"","hint":"","columns":[{"id":"name","label":"Item","numeric":false},{"id":"kind","label":"Type","numeric":false},{"id":"qty","label":"Qty","numeric":true},{"id":"value","label":"Value","numeric":true}],"rows":[{"id":"a","cells":[{"id":"name","text":"Ember Blade","numeric":false},{"id":"kind","text":"Weapon","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"1,250","numeric":true}]},{"id":"b","cells":[{"id":"name","text":"Health Potion","numeric":false},{"id":"kind","text":"Consumable","numeric":false},{"id":"qty","text":"12","numeric":true},{"id":"value","text":"40","numeric":true}]},{"id":"c","cells":[{"id":"name","text":"Iron Shield","numeric":false},{"id":"kind","text":"Armour","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"380","numeric":true}]},{"id":"d","cells":[{"id":"name","text":"Silver Key","numeric":false},{"id":"kind","text":"Quest","numeric":false},{"id":"qty","text":"2","numeric":true},{"id":"value","text":"0","numeric":true}]},{"id":"e","cells":[{"id":"name","text":"Moonstone","numeric":false},{"id":"kind","text":"Gem","numeric":false},{"id":"qty","text":"3","numeric":true},{"id":"value","text":"900","numeric":true}]}]}'>

<!-- The HUD's smoked-glass plate: headings in small tracked capitals over
     a faint rule, rows dense and plain so the data reads first, figures
     condensed and tabular. The chosen row is washed in the accent from its
     leading edge. In a narrow shell each row becomes a two-line card: the
     first column and the last on one line, the rest as a caption under. -->
<section id="DataGrid" part="DataGrid.frame" class="bare" data-layout="stack" data-gap="none" role="grid" aria-label={label} aria-busy={loading} style:--tracks={tracks || "1fr 1fr 1fr 1fr"}>
  <div id="Head" part="DataGrid.band" class="small" role="row">
    {#each columns as c (c.id)}<span part="DataGrid.heading DataGrid.caption" class="heading" class:numeric={c.numeric} role="columnheader">{c.label}</span>{/each}
  </div>
  {#if loading}
    <div id="Loading" data-layout="stack" data-gap="none" role="status" aria-label={t("component.data-grid.loading")}>
      <div class="ghost-row"><div part="DataGrid.ghost DataGrid.skeleton" class="ghost"><div part="DataGrid.sheen" data-motion="shimmer"></div></div></div><div class="ghost-row"><div part="DataGrid.ghost DataGrid.skeleton" class="ghost"><div part="DataGrid.sheen" data-motion="shimmer"></div></div></div><div class="ghost-row"><div part="DataGrid.ghost DataGrid.skeleton" class="ghost"><div part="DataGrid.sheen" data-motion="shimmer"></div></div></div><div class="ghost-row"><div part="DataGrid.ghost DataGrid.skeleton" class="ghost"><div part="DataGrid.sheen" data-motion="shimmer"></div></div></div>
    </div>
  {:else if error}
    <div id="Error" part="DataGrid.error" data-layout="stack" role="alert" aria-label={error}>
      <div part="DataGrid.mark DataGrid.well" class="mark danger" data-layout="cover"><og-icon icon="close" data-tone="danger"></og-icon></div>
      <p part="DataGrid.message DataGrid.headline" class="title">{error}</p>
      <button id="Retry" part="DataGrid.retry" variant="primary" onclick={retry}>{t("component.data-grid.retry")}</button>
    </div>
  {:else if !rows.length}
    <div id="Empty" part="DataGrid.empty" data-layout="stack" role="status">
      <div part="DataGrid.mark DataGrid.well" class="mark" data-layout="cover"><og-icon icon="bag"></og-icon></div>
      <p part="DataGrid.message DataGrid.headline" class="title">{empty || t("component.data-grid.empty")}</p>
      {#if hint}<p part="DataGrid.hint DataGrid.caption" class="hint">{hint}</p>{/if}
    </div>
  {:else}
    <list id="Rows">
      {#each rows as r (r.id)}
        <button id="Row" part="DataGrid.row" class="row" variant="quiet" role="row" aria-selected={r.id == selected} onclick={choose}>
          <div class="cells">
            {#each r.cells as c, i (c.id)}{#if i == 1}<div class="break"></div>{/if}<span part="DataGrid.value" class="cell" class:numeric={c.numeric} role="gridcell">{c.text}</span>{/each}
          </div>
        </button>
      {/each}
    </list>
  {/if}
</section>

<style>
:host { inline-size: stretch; container-type: inline-size; --row-fill: none; --row-hover-fill: var(--data-row-hover-fill); --row-pressed-fill: var(--data-row-pressed-fill); --row-selected-fill: var(--data-row-selected-fill); --row-selected-edge: var(--data-row-selected-edge); --row-radius: 0px; }
#DataGrid { background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(calc(2 * var(--hud-plate-cut)) 0, 100% 0, 100% calc(100% - 2 * var(--hud-plate-cut)), calc(100% - 2 * var(--hud-plate-cut)) 100%, 0 100%, 0 calc(2 * var(--hud-plate-cut))); color: var(--hud-ink); }
#Head, .cells { display: grid; grid-template-columns: var(--tracks); column-gap: var(--space-md); width: stretch; min-width: 0; }
#Head { padding: var(--space-sm) var(--space-md); border-block-end: var(--data-rule-line); }
.heading { white-space: nowrap; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--data-head-ink); }
.numeric { text-align: end; }
.break { display: none; }
#Rows { width: stretch; height: var(--size-6xl); }
.row { width: stretch; min-block-size: var(--row); padding: var(--space-xs) var(--space-md); justify-content: start; min-width: 0; border-block-end: var(--data-rule-line); }
.cell { width: stretch; min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; color: var(--hud-ink-dim); }
.cell:first-child { color: var(--hud-ink); font-weight: var(--data-name-weight); }
/* A figure is never cut: an ellipsis on a number is wrong data. */
.cell.numeric { text-align: end; text-overflow: clip; overflow: visible; color: var(--hud-ink); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
#Empty, #Error { width: stretch; align-items: center; padding: var(--space-xl); --layout-gap: var(--layout-gap-loose); }
.mark { width: var(--size-xl); height: var(--size-xl); --well-fill: var(--data-well); --well-shadow: var(--data-well-rim); --well-radius: var(--radius-capsule); }
.mark.danger { --well-fill: var(--dot-alert-fill); --well-shadow: inset 0 0 0 var(--stroke-thin) var(--dot-alert-rim); }
.title, .hint { margin: 0; text-align: center; width: stretch; overflow-wrap: anywhere; }
.title { color: var(--hud-ink); }
#Retry { min-block-size: var(--touch-target); max-inline-size: 100%; white-space: normal; }
#Loading { padding: 0; }
.ghost-row { width: stretch; padding: var(--space-sm) var(--space-md); border-block-end: var(--data-rule-line); }
.ghost { width: stretch; height: var(--space-md); --skeleton-radius: var(--radius-capsule); }
/* A narrow shell: each row is a two-line card, the first column and the
   last on its first line and the rest as a caption under them, so no
   column is cut to an ellipsis. The headings above read the same. */
@container (width < 480px) {
  #Head, .cells { display: flex; flex-wrap: wrap; column-gap: var(--space-md); row-gap: var(--space-xxs); align-items: baseline; }
  .heading, .cell { flex: none; width: auto; order: 3; }
  /* The name gives way to the figure, which keeps its whole width. */
  .heading:first-child, .cell:first-child { flex: 1 1 0; order: 0; min-width: 0; white-space: normal; overflow-wrap: anywhere; }
  .heading:last-child, .cell:last-child { order: 1; }
  /* A break after them puts the rest on the caption line. */
  .break { display: block; flex: 0 0 100%; order: 2; block-size: 0; }
  .heading:not(:first-child):not(:last-child) { display: none; }
  .cell:not(:first-child):not(:last-child) { font-size: var(--hud-label-size); }
  .row { padding-block: var(--space-sm); }
}
</style>
