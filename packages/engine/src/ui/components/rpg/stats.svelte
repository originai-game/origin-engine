<meta name="component" content="RpgStats">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgStats.frame RpgStats.band RpgStats.title RpgStats.row RpgStats.figure RpgStats.caption">
<meta name="prop:title" content="string" data-doc="The panel's heading (Stats).">
<meta name="prop:groups" content="list" data-doc="The groups, keyed rows of { id, column }: column a or b; a rule parts one group from the next in a column.">
<meta name="prop:rows" content="list" data-doc="The figures, keyed rows of { id, group, icon, label, value }: group a group's id; value as the game words it.">
<meta name="fixture:default" content='{"title":"Stats","groups":[{"id":"res","column":"a"},{"id":"rst","column":"b"}],"rows":[{"id":"hp","group":"res","icon":"heart","label":"Health","value":"2,850"},{"id":"fire","group":"rst","icon":"fire","label":"Fire resistance","value":"72%"}]}'>

<!--
  A character's figures in two columns of grouped rows, each an icon, a
  label and a value. Read only; fills its height and scrolls inside it.
-->
<section id="RpgStats" part="RpgStats.frame" data-layout="shell" aria-label={title}>
  <header part="RpgStats.band" data-layout="row" data-gap="loose"><div class="band-end start"></div><h2 part="RpgStats.title">{title}</h2><div class="band-end finish"></div></header>
  <div class="body" data-layout="scroll">
    <div class="columns" data-layout="switcher" data-gap="loose">
      <div class="column" data-layout="stack" data-fill role="table">
        {#each groups as g (g.id)}
          {#if g.column == 'a'}
            <div class="group" data-layout="stack" data-gap="none" role="group">
              {#each rows as r (r.id)}
                {#if r.group == g.id}<div part="RpgStats.row" data-layout="row" role="row"><og-icon class="row-mark" part="RpgStats.caption" icon={r.icon}></og-icon><span class="row-label" data-fill role="rowheader">{r.label}</span><span class="row-value" part="RpgStats.figure" role="cell">{r.value}</span></div>{/if}
              {/each}
            </div>
          {/if}
        {/each}
      </div>
      <hr class="split" vertical>
      <div class="column" data-layout="stack" data-fill role="table">
        {#each groups as g (g.id)}
          {#if g.column == 'b'}
            <div class="group" data-layout="stack" data-gap="none" role="group">
              {#each rows as r (r.id)}
                {#if r.group == g.id}<div part="RpgStats.row" data-layout="row" role="row"><og-icon class="row-mark" part="RpgStats.caption" icon={r.icon}></og-icon><span class="row-label" data-fill role="rowheader">{r.label}</span><span class="row-value" part="RpgStats.figure" role="cell">{r.value}</span></div>{/if}
              {/each}
            </div>
          {/if}
        {/each}
      </div>
    </div>
  </div>
</section>

<style>
[part="RpgStats.band"] { width: stretch; justify-content: center; flex: none; }
.band-end { width: var(--size-lg); height: var(--space-lg); flex: none; }
.columns { width: stretch; padding: 0 var(--space-xs); }
.column { align-self: stretch; }
.split { width: var(--stroke-hairline); align-self: stretch; margin: 0; }
.group { width: stretch; padding-bottom: var(--space-xs); }
[part="RpgStats.row"] { width: stretch; padding: var(--space-sm) 0; }
.row-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.row-label { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.row-value { flex: none; white-space: nowrap; }
@container (width < 420px) { .split { display: none; } }
</style>
