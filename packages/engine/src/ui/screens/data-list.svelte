<meta name="component" content="data-list">
<meta name="parts" content="data-list.frame data-list.row data-list.label data-list.figure data-list.note">
<meta name="prop:rows" content="list" data-doc="The rows, a keyed collection of { id, title, detail, value }.">
<meta name="prop:loading" content="bool" data-doc="Rows are on their way: a busy line instead of the empty one.">
<meta name="prop:empty" content="string" data-doc="What an empty list says.">
<meta name="prop:busy" content="string" data-doc="What a loading list says.">
<meta name="action:choose" content="Open">
<meta name="fixture:default" content="{&quot;loading&quot;:false,&quot;empty&quot;:&quot;Nothing here yet&quot;,&quot;busy&quot;:&quot;Loading…&quot;,&quot;rows&quot;:[{&quot;id&quot;:&quot;a&quot;,&quot;title&quot;:&quot;Copper ore&quot;,&quot;detail&quot;:&quot;Mined in the north pit&quot;,&quot;value&quot;:&quot;×12&quot;},{&quot;id&quot;:&quot;b&quot;,&quot;title&quot;:&quot;Silk&quot;,&quot;detail&quot;:&quot;Bought in Harrow&quot;,&quot;value&quot;:&quot;×3&quot;}]}">

<!-- A HUD plate of rows: a name, its detail and its figure. -->
<section id="DataList" part="data-list.frame" class="bare" aria-busy={loading}>
  <list id="Rows">
    {#each rows as r (r.id)}
      <button id="Row" part="data-list.row" class="row" onclick={choose}>
        <div class="face">
          <div class="text"><p class="title">{r.title}</p><small class="detail" part="data-list.label">{r.detail}</small></div>
          <p class="value" part="data-list.figure">{r.value}</p>
        </div>
      </button>
    {:else}
      <p id="Empty" part="data-list.note" class="note">{loading ? busy : empty}</p>
    {/each}
  </list>
</section>

<style>
/* The HUD plate: each row a name over its detail, its figure at the end
   condensed and tabular, a faint rule under it. */
#DataList { width: stretch; height: stretch; background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(calc(2 * var(--hud-plate-cut)) 0, 100% 0, 100% calc(100% - 2 * var(--hud-plate-cut)), calc(100% - 2 * var(--hud-plate-cut)) 100%, 0 100%, 0 calc(2 * var(--hud-plate-cut))); color: var(--hud-ink); --row-fill: none; --row-hover-fill: var(--data-row-hover-fill); --row-pressed-fill: var(--data-row-pressed-fill); --row-selected-fill: var(--data-row-selected-fill); --row-selected-edge: var(--data-row-selected-edge); --row-radius: 0px; }
#Rows { width: stretch; height: stretch; }
.row { width: stretch; padding: var(--space-sm) var(--space-md); min-width: 0; border-block-end: var(--data-rule-line); }
.face { display: flex; align-items: center; justify-content: start; gap: var(--space-md); width: stretch; min-width: 0; text-align: start; }
.text { display: flex; flex-direction: column; flex: 1; min-width: 0; gap: var(--space-xxs); align-items: start; }
.title, .value { margin: 0; }
.title { color: var(--hud-ink); font-weight: var(--data-name-weight); text-align: start; overflow-wrap: anywhere; }
.detail { color: var(--hud-ink-dim); text-align: start; overflow-wrap: anywhere; }
.value { flex: none; color: var(--hud-ink); font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.note { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--data-head-ink); margin: var(--space-xl) 0; text-align: center; width: stretch; }
</style>
