<meta name="component" content="calendar">
<meta name="parts" content="calendar.frame calendar.band calendar.title calendar.heading calendar.cell">
<meta name="prop:month" content="string" data-doc="The month shown (March 2027).">
<meta name="prop:weekdays" content="list" data-doc="The seven weekday headings, a keyed collection of { id, label }.">
<meta name="prop:days" content="list" data-doc="The cells, a keyed collection of { id, label, outside, today, marked } from the week's first day (outside: another month's).">
<meta name="action:previous" content="Previous month">
<meta name="action:next" content="Next month">
<meta name="action:pick" content="Choose day">
<meta name="fixture:default" content="{&quot;month&quot;:&quot;March 2027&quot;,&quot;weekdays&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;label&quot;:&quot;M&quot;},{&quot;id&quot;:&quot;2&quot;,&quot;label&quot;:&quot;T&quot;},{&quot;id&quot;:&quot;3&quot;,&quot;label&quot;:&quot;W&quot;},{&quot;id&quot;:&quot;4&quot;,&quot;label&quot;:&quot;T&quot;},{&quot;id&quot;:&quot;5&quot;,&quot;label&quot;:&quot;F&quot;},{&quot;id&quot;:&quot;6&quot;,&quot;label&quot;:&quot;S&quot;},{&quot;id&quot;:&quot;7&quot;,&quot;label&quot;:&quot;S&quot;}],&quot;days&quot;:[{&quot;id&quot;:&quot;d1&quot;,&quot;label&quot;:&quot;1&quot;,&quot;today&quot;:false,&quot;outside&quot;:false,&quot;marked&quot;:false},{&quot;id&quot;:&quot;d2&quot;,&quot;label&quot;:&quot;2&quot;,&quot;today&quot;:true,&quot;outside&quot;:false,&quot;marked&quot;:true}]}">

<!-- A HUD plate: the month between cut arrow keys, the days dark recesses,
     today lit in the accent and a marked day carrying a gold diamond. -->
<section id="Calendar" part="calendar.frame" class="bare" aria-label={month}>
  <header id="Head" part="calendar.band">
    <button id="Prev" variant="quiet" onclick={previous} aria-label={month}><og-icon icon="chevronLeft"/></button>
    <h3 id="Month" part="calendar.title">{month}</h3>
    <button id="Next" variant="quiet" onclick={next} aria-label={month}><og-icon icon="chevronRight"/></button>
  </header>
  <div id="Days" role="grid">
    {#each weekdays as w (w.id)}<small class="weekday" part="calendar.heading">{w.label}</small>{/each}
    {#each days as d (d.id)}
      <button id="Day" part="calendar.cell" class="day" class:outside={d.outside} class:marked={d.marked} variant={d.today ? "primary" : "quiet"} aria-current={d.today} onclick={pick}>{d.label}</button>
    {/each}
  </div>
</section>

<style>
/* As wide as its container up to a measure, inset from its edges, so the
   seven columns share the header's width on a phone at any text size. */
#Calendar { display: flex; flex-direction: column; gap: var(--space-sm); box-sizing: border-box; inline-size: 100%; max-inline-size: var(--measure-md); padding: var(--space-md); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(calc(2 * var(--hud-plate-cut)) 0, 100% 0, 100% calc(100% - 2 * var(--hud-plate-cut)), calc(100% - 2 * var(--hud-plate-cut)) 100%, 0 100%, 0 calc(2 * var(--hud-plate-cut))); color: var(--hud-ink); }
#Head { display: flex; align-items: center; gap: var(--space-xs); }
#Prev, #Next { flex: none; min-inline-size: var(--touch-target); min-block-size: var(--touch-target); --button-radius: var(--hud-tag-cut); --button-corner-shape: bevel; --button-quiet-ink: var(--hud-ink); background: var(--data-well); box-shadow: var(--data-well-rim); }
#Prev::part(base), #Next::part(base) { padding-inline: 0; }
#Month { flex: 1; min-inline-size: 0; margin: 0; text-align: center; font-size: var(--hud-size-sm); font-weight: var(--data-title-weight); color: var(--hud-ink); overflow-wrap: anywhere; }
/* Columns divide the width evenly and shrink below their content. */
#Days { display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); gap: var(--space-xxs); }
.weekday { text-align: center; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--data-head-ink); padding-block-end: var(--space-xxs); }
/* A day: a dark recess with its figure condensed and tabular. */
.day { position: relative; inline-size: stretch; min-inline-size: 0; min-block-size: var(--touch-target); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); --cell-radius: 0px; color: var(--hud-ink); background: var(--data-well); box-shadow: var(--data-well-rim); }
/* A day of the next or last month is quieter in its ink and has no recess. */
.day.outside { color: var(--data-head-ink); background: none; box-shadow: none; }
/* Today is lit in the accent, as a held key is. */
.day[variant="primary"] { color: var(--hud-key-ink); background: var(--hud-key-lit-fill); box-shadow: none; }
/* A day something happens on carries a gold diamond at its foot. */
.day.marked::after { content: ""; position: absolute; inset-block-end: var(--space-xxs); inset-inline-start: calc(50% - var(--space-xxs)); inline-size: var(--space-xs); block-size: var(--space-xs); rotate: 45deg; background: var(--data-mark); box-shadow: var(--data-mark-glow); }
</style>
