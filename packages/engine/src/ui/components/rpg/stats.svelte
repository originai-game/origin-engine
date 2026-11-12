<meta name="component" content="RpgStats">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgStats.frame RpgStats.band RpgStats.title RpgStats.row">
<meta name="prop:title" content="string" data-doc="The panel's heading (Stats).">
<meta name="prop:groups" content="list" data-doc="The groups, keyed rows of { id, column }: column a or b; a rule parts one group from the next in a column.">
<meta name="prop:rows" content="list" data-doc="The figures, keyed rows of { id, group, icon, label, value }: group a group's id; value as the game words it.">
<meta name="fixture:default" content='{"title":"Stats","groups":[{"id":"res","column":"a"},{"id":"rst","column":"b"}],"rows":[{"id":"hp","group":"res","icon":"heart","label":"Health","value":"2,850"},{"id":"fire","group":"rst","icon":"fire","label":"Fire resistance","value":"72%"}]}'>

<!--
  A character's figures in two columns of grouped rows, each an icon, a
  label and a value. Read only; fills its height and scrolls inside it.
-->
<section id="RpgStats" part="RpgStats.frame" aria-label={title}>
  <header part="RpgStats.band"><div class="band-end start"></div><h2 part="RpgStats.title">{title}</h2><div class="band-end finish"></div></header>
  <scroll class="body">
    <div class="columns">
      <div class="column" role="table">
        {#each groups as g (g.id)}
          {#if g.column == 'a'}
            <div class="group" role="group">
              {#each rows as r (r.id)}
                {#if r.group == g.id}<div part="RpgStats.row" role="row"><og-icon class="row-mark" icon={r.icon}></og-icon><span class="row-label" role="rowheader">{r.label}</span><span class="row-value" role="cell">{r.value}</span></div>{/if}
              {/each}
            </div>
          {/if}
        {/each}
      </div>
      <div class="split"></div>
      <div class="column" role="table">
        {#each groups as g (g.id)}
          {#if g.column == 'b'}
            <div class="group" role="group">
              {#each rows as r (r.id)}
                {#if r.group == g.id}<div part="RpgStats.row" role="row"><og-icon class="row-mark" icon={r.icon}></og-icon><span class="row-label" role="rowheader">{r.label}</span><span class="row-value" role="cell">{r.value}</span></div>{/if}
              {/each}
            </div>
          {/if}
        {/each}
      </div>
    </div>
  </scroll>
</section>

<style>
[part="RpgStats.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-sm); container-type: inline-size; }
[part="RpgStats.band"] { display: flex; width: stretch; align-items: center; justify-content: center; gap: var(--space-md); flex: none; }
.band-end { width: var(--size-lg); height: var(--space-lg); flex: none; }
.body { width: stretch; flex: 1; min-height: 0; }
.columns { display: flex; width: stretch; gap: var(--space-md); padding: 0 var(--space-xs); }
.column { display: flex; flex-direction: column; width: stretch; gap: var(--space-sm); }
.split { width: var(--stroke-hairline); align-self: stretch; }
.group { display: flex; flex-direction: column; width: stretch; padding-bottom: var(--space-xs); }
[part="RpgStats.row"] { display: flex; width: stretch; align-items: center; gap: var(--space-sm); padding: var(--space-sm) 0; }
.row-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.row-label { width: stretch; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.row-value { flex: none; font-variant-numeric: tabular-nums; white-space: nowrap; }
@container (width < 420px) { .columns { flex-direction: column; gap: 0; } .split { display: none; } }
</style>
