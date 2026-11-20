<meta name="component" content="table">
<meta name="prop:columns" content="list" data-doc="The headings, a keyed collection of { id, label }.">
<meta name="prop:cells" content="list" data-doc="The body's cells row by row, a keyed collection of { id, row, text }: three to a row (eject it to change the columns). A press sends choose with the cell's id; its row field names the row.">
<meta name="prop:label" content="string" data-doc="The table's accessible name.">
<meta name="action:choose" content="Open row">
<meta name="fixture:default" content="{&quot;label&quot;:&quot;Standings&quot;,&quot;columns&quot;:[{&quot;id&quot;:&quot;n&quot;,&quot;label&quot;:&quot;Team&quot;},{&quot;id&quot;:&quot;w&quot;,&quot;label&quot;:&quot;Won&quot;},{&quot;id&quot;:&quot;p&quot;,&quot;label&quot;:&quot;Points&quot;}],&quot;cells&quot;:[{&quot;id&quot;:&quot;r.n&quot;,&quot;row&quot;:&quot;r&quot;,&quot;text&quot;:&quot;Ravens&quot;},{&quot;id&quot;:&quot;r.w&quot;,&quot;row&quot;:&quot;r&quot;,&quot;text&quot;:&quot;9&quot;},{&quot;id&quot;:&quot;r.p&quot;,&quot;row&quot;:&quot;r&quot;,&quot;text&quot;:&quot;27&quot;},{&quot;id&quot;:&quot;g.n&quot;,&quot;row&quot;:&quot;g&quot;,&quot;text&quot;:&quot;Gulls&quot;},{&quot;id&quot;:&quot;g.w&quot;,&quot;row&quot;:&quot;g&quot;,&quot;text&quot;:&quot;7&quot;},{&quot;id&quot;:&quot;g.p&quot;,&quot;row&quot;:&quot;g&quot;,&quot;text&quot;:&quot;21&quot;}]}">

<!-- A HUD plate of plain rows: the first column a name, the others figures. -->
<section id="Table" part="frame" variant="bare" role="table" aria-label={label}>
  <div id="Head" part="band" role="row">
    {#each columns as c, i (c.id)}<p class="heading" class:end={i > 0} part="heading" role="columnheader">{c.label}</p>{/each}
  </div>
  <div id="Body">
    <div id="Cells">
      {#each cells as c, i (c.id)}<button id="Cell" part="entry" class="cell" class:figure={i % 3 != 0} variant="quiet" role="cell" onclick={choose}>{c.text}</button>{/each}
    </div>
  </div>
</section>

<style>
/* The HUD plate: headings in small tracked capitals over a faint rule,
   the cells dense and plain, a faint rule under each row. */
#Table { display: flex; flex-direction: column; width: stretch; height: stretch; background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: calc(2 * var(--hud-plate-cut)) 0; corner-shape: var(--cut-shape); clip-path: border-box; color: var(--hud-ink); }
#Head, #Cells { display: grid; grid-template-columns: 2fr 1fr 1fr; width: stretch; padding-inline: var(--space-sm); }
#Head { padding-block: var(--space-sm); border-block-end: var(--data-rule-line); }
.heading { margin: 0; min-width: 0; padding-inline: var(--space-sm); font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--data-head-ink); }
.heading.end { text-align: end; }
.cell { width: stretch; min-width: 0; min-block-size: var(--touch-target); justify-content: start; text-align: start; white-space: normal; overflow-wrap: anywhere; border-block-end: var(--data-rule-line); --button-radius: 0px; --button-quiet-ink: var(--hud-ink); }
.cell { padding-inline: var(--space-sm); }
/* Every column after the first is a figure: condensed, tabular, at the end. */
.cell.figure { justify-content: end; text-align: end; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
#Body { flex: 1; min-block-size: 0; overflow-y: auto; }
</style>
