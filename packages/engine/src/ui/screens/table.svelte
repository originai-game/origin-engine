<meta name="component" content="table">
<meta name="prop:columns" content="list" data-doc="The headings, a keyed collection of { id, label }.">
<meta name="prop:cells" content="list" data-doc="The body's cells row by row, a keyed collection of { id, row, text }: three to a row (eject it to change the columns). A press sends choose with the cell's id; its row field names the row.">
<meta name="prop:label" content="string" data-doc="The table's accessible name.">
<meta name="action:choose" content="Open row">
<meta name="fixture:default" content="{&quot;label&quot;:&quot;Standings&quot;,&quot;columns&quot;:[{&quot;id&quot;:&quot;n&quot;,&quot;label&quot;:&quot;Team&quot;},{&quot;id&quot;:&quot;w&quot;,&quot;label&quot;:&quot;Won&quot;},{&quot;id&quot;:&quot;p&quot;,&quot;label&quot;:&quot;Points&quot;}],&quot;cells&quot;:[{&quot;id&quot;:&quot;r.n&quot;,&quot;row&quot;:&quot;r&quot;,&quot;text&quot;:&quot;Ravens&quot;},{&quot;id&quot;:&quot;r.w&quot;,&quot;row&quot;:&quot;r&quot;,&quot;text&quot;:&quot;9&quot;},{&quot;id&quot;:&quot;r.p&quot;,&quot;row&quot;:&quot;r&quot;,&quot;text&quot;:&quot;27&quot;},{&quot;id&quot;:&quot;g.n&quot;,&quot;row&quot;:&quot;g&quot;,&quot;text&quot;:&quot;Gulls&quot;},{&quot;id&quot;:&quot;g.w&quot;,&quot;row&quot;:&quot;g&quot;,&quot;text&quot;:&quot;7&quot;},{&quot;id&quot;:&quot;g.p&quot;,&quot;row&quot;:&quot;g&quot;,&quot;text&quot;:&quot;21&quot;}]}">

<section id="Table" role="table" aria-label={$label}>
  <div id="Head" role="row">
    {#each $columns as c (c.id)}<p class="heading" role="columnheader">{c.label}</p>{/each}
  </div>
  <scroll id="Body">
    <div id="Cells">
      {#each $cells as c (c.id)}<button id="Cell" class="cell" variant="quiet" role="cell" onclick={choose}>{c.text}</button>{/each}
    </div>
  </scroll>
</section>

<style>
#Table { display: flex; flex-direction: column; width: stretch; height: stretch; }
#Head, #Cells { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 8px; width: stretch; }
#Head { padding: 6px 12px; border-bottom: 1px solid var(--line); }
#Cells { padding: 0 12px; }
.heading { margin: 0; color: var(--ink-muted); }
.cell { width: stretch; justify-content: start; }
#Body { width: stretch; height: stretch; }
</style>
