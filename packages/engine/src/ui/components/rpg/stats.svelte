<meta name="component" content="RpgStats">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:title" content="string" data-doc="The panel's heading (Stats).">
<meta name="prop:groups" content="list" data-doc="The groups, keyed rows of { id, column }: column a or b; a rule parts one group from the next in a column.">
<meta name="prop:rows" content="list" data-doc="The figures, keyed rows of { id, group, icon, label, value }: group a group's id; value as the game words it.">
<meta name="fixture:default" content='{"title":"Stats","groups":[{"id":"res","column":"a"},{"id":"rst","column":"b"}],"rows":[{"id":"hp","group":"res","icon":"heart","label":"Health","value":"2,850"},{"id":"fire","group":"rst","icon":"fire","label":"Fire resistance","value":"72%"}]}'>

<!--
  A character's figures in two columns of grouped rows, each an icon, a
  label and a value. Read only; fills its height and scrolls inside it.
-->
<section class="frame" id="RpgStats" part="frame" layout="stack" fill aria-label={title}>
  <header class="band" part="band" layout="row" gap="loose"><div class="band-end start"></div><h2 class="title" part="title">{title}</h2><div class="band-end finish"></div></header>
  <div class="body" layout="scroll">
    <div class="columns" layout="wrap" gap="loose">
      <div class="column" layout="stack" fill role="table">
        {#each groups as g (g.id)}
          {#if g.column == 'a'}
            <div class="group" layout="stack" gap="none" role="group">
              {#each rows as r (r.id)}
                {#if r.group == g.id}<div class="row" part="row" layout="row" role="row"><og-icon class="row-mark" part="caption" icon={r.icon}></og-icon><span class="row-label" fill role="rowheader">{r.label}</span><span class="row-value" part="figure" role="cell">{r.value}</span></div>{/if}
              {/each}
            </div>
          {/if}
        {/each}
      </div>
      <hr class="split" vertical>
      <div class="column" layout="stack" fill role="table">
        {#each groups as g (g.id)}
          {#if g.column == 'b'}
            <div class="group" layout="stack" gap="none" role="group">
              {#each rows as r (r.id)}
                {#if r.group == g.id}<div class="row" part="row" layout="row" role="row"><og-icon class="row-mark" part="caption" icon={r.icon}></og-icon><span class="row-label" fill role="rowheader">{r.label}</span><span class="row-value" part="figure" role="cell">{r.value}</span></div>{/if}
              {/each}
            </div>
          {/if}
        {/each}
      </div>
    </div>
  </div>
</section>

<style>
/* A character's figures on a smoked cut card: ruled rows, each a small
   mark, its label and its value in condensed tabular figures. */
#RpgStats { color: var(--hud-ink); }
.frame { box-sizing: border-box; padding: var(--space-lg); border: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-card-rim); border-radius: var(--party-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.band { width: stretch; justify-content: center; flex: none; min-height: 0; padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; }
.title { margin: 0; min-inline-size: 0; font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: none; white-space: normal; overflow-wrap: break-word; text-align: center; }
.band-end { width: var(--size-lg); height: var(--space-lg); flex: none; }
.body { -og-scroll-fade: var(--space-xl); }
.columns { width: stretch; padding: 0 var(--space-xs); }
.column { align-self: stretch; min-width: 0; }
.split { width: var(--stroke-hairline); align-self: stretch; margin: 0; }
.group { width: stretch; padding-bottom: var(--space-sm); }
.row { width: stretch; min-height: var(--size-md); padding: var(--space-xs) 0; border-radius: 0; color: var(--hud-ink); background: none; box-shadow: var(--party-rule); }
.row-mark { width: var(--size-xs); height: var(--size-xs); flex: none; color: var(--hud-label-ink); }
.row-label { min-width: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-label-ink); overflow-wrap: break-word; }
.row-value { flex: none; white-space: nowrap; font-size: var(--hud-size-readout); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); text-shadow: none; }
@media (width < 600px) { .frame { padding: var(--space-sm); } }
@container (width < 420px) { .split { display: none; } }
</style>
