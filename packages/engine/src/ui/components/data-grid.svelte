<meta name="component" content="DataGrid">
<meta name="strings" content="./data-grid.en.ogt">
<meta name="prop:cells" content="list" data-doc="The cells, a keyed collection of { id, label, icon, count }: icon an icon name, count a stack size (0: none).">
<meta name="prop:selected" content="string" data-doc="The id of the selected cell; empty is none.">
<meta name="prop:size" content="number" data-doc="A cell's width and height, CSS px; 0 is 64.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the grid.">
<meta name="prop:loading" content="bool" data-doc="Cells are on their way: busy placeholders instead of the cells.">
<meta name="prop:error" content="string" data-doc="An error to show with Retry instead of the cells; empty is none.">
<meta name="prop:empty" content="string" data-doc="What an empty grid says; empty is Nothing here yet.">
<meta name="prop:hint" content="string" data-doc="A line under the empty message.">
<meta name="action:choose" content="Select">
<meta name="action:retry" content="Retry">
<meta name="fixture:default" content='{"selected":"b","size":64,"label":"Bag","loading":false,"error":"","empty":"","hint":"","cells":[{"id":"a","label":"Sword","icon":"sword","count":0},{"id":"b","label":"Potion","icon":"potion","count":3},{"id":"c","label":"Shield","icon":"shield","count":0},{"id":"d","label":"Coins","icon":"coin","count":120},{"id":"e","label":"Key","icon":"key","count":1},{"id":"f","label":"Gem","icon":"gem","count":2},{"id":"g","label":"Map","icon":"map","count":0},{"id":"h","label":"Bomb","icon":"bomb","count":5}]}'>

<section id="DataGrid" aria-label={label} aria-busy={loading} style:--size={size || 64}>
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
  {:else if !cells.length}
    <div id="Empty" role="status">
      <div class="mark"><icon icon="bag"></icon></div>
      <p class="title">{empty || t("component.data-grid.empty")}</p>
      <p class="hint" hidden={!hint}>{hint}</p>
    </div>
  {:else}
    <cells id="Cells" role="grid" aria-label={label} cell-width={size || 64} cell-height={size || 64}>
      {#each cells as c (c.id)}
        <button id="Cell" class="cell" aria-label={c.label} aria-pressed={c.id == selected} onclick={choose}>
          <icon class="art" icon={c.icon}></icon>
          <span class="count" hidden={!c.count}>{formatNumber(c.count)}</span>
        </button>
      {/each}
    </cells>
  {/if}
</section>

<style>
#DataGrid { width: stretch; padding: 8px; border-radius: 10px; background: var(--panel); -og-frame: panel; }
#Cells { width: stretch; gap: 8px; }
.cell { width: stretch; height: stretch; padding: 0; border-radius: 8px; align-items: center; justify-content: center; }
.cell[aria-pressed=true] { outline: 2px solid var(--primary); }
.art { width: calc(var(--size) * 0.55px); height: calc(var(--size) * 0.55px); }
.count { position: absolute; bottom: 4px; inset-inline-end: 6px; font: var(--text-caption); font-family: var(--font-mono); color: var(--ink); }
#Empty, #Error { width: stretch; padding: 24px; gap: 8px; align-items: center; }
.mark { width: 48px; height: 48px; border-radius: 24px; align-items: center; justify-content: center; background: var(--control); }
.mark.danger { background: #e5484d29; color: var(--danger); }
.title { margin: 0; font: var(--text-base); color: var(--ink); text-align: center; width: stretch; }
.hint { margin: 0; font: var(--text-sm); color: var(--ink-muted); text-align: center; width: stretch; }
#Loading { display: flex; flex-wrap: wrap; gap: 8px; width: stretch; }
.ghost { width: calc(var(--size) * 1px); height: calc(var(--size) * 1px); border-radius: 8px; background: linear-gradient(90deg, #ffffff00 30%, #ffffff1f 50%, #ffffff00 70%), var(--control-strong); }
</style>
