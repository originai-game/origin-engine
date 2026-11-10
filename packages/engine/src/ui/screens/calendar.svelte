<meta name="component" content="calendar">
<meta name="prop:month" content="string" data-doc="The month shown (March 2027).">
<meta name="prop:weekdays" content="list" data-doc="The seven weekday headings, a keyed collection of { id, label }.">
<meta name="prop:days" content="list" data-doc="The cells, a keyed collection of { id, label, outside, today, marked } from the week's first day (outside: another month's).">
<meta name="action:previous" content="Previous month">
<meta name="action:next" content="Next month">
<meta name="action:pick" content="Choose day">
<meta name="fixture:default" content="{&quot;month&quot;:&quot;March 2027&quot;,&quot;weekdays&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;label&quot;:&quot;M&quot;},{&quot;id&quot;:&quot;2&quot;,&quot;label&quot;:&quot;T&quot;},{&quot;id&quot;:&quot;3&quot;,&quot;label&quot;:&quot;W&quot;},{&quot;id&quot;:&quot;4&quot;,&quot;label&quot;:&quot;T&quot;},{&quot;id&quot;:&quot;5&quot;,&quot;label&quot;:&quot;F&quot;},{&quot;id&quot;:&quot;6&quot;,&quot;label&quot;:&quot;S&quot;},{&quot;id&quot;:&quot;7&quot;,&quot;label&quot;:&quot;S&quot;}],&quot;days&quot;:[{&quot;id&quot;:&quot;d1&quot;,&quot;label&quot;:&quot;1&quot;,&quot;today&quot;:false,&quot;outside&quot;:false,&quot;marked&quot;:false},{&quot;id&quot;:&quot;d2&quot;,&quot;label&quot;:&quot;2&quot;,&quot;today&quot;:true,&quot;outside&quot;:false,&quot;marked&quot;:true}]}">

<section id="Calendar" aria-label={$month}>
  <header id="Head">
    <button id="Prev" variant="quiet" onclick={previous} aria-label={$month}><og-icon name="chevronLeft"/></button>
    <h3 id="Month">{$month}</h3>
    <button id="Next" variant="quiet" onclick={next} aria-label={$month}><og-icon name="chevronRight"/></button>
  </header>
  <grid id="Days" role="grid">
    {#each $weekdays as w (w.id)}<small class="weekday">{w.label}</small>{/each}
    {#each $days as d (d.id)}
      <button id="Day" class="day" class:outside={d.outside} class:marked={d.marked} variant={d.today ? "primary" : "quiet"} aria-current={d.today} onclick={pick}>{d.label}</button>
    {/each}
  </grid>
</section>

<style>
#Calendar { display: flex; flex-direction: column; gap: 8px; width: 300px; }
#Head { display: flex; align-items: center; }
#Month { width: stretch; margin: 0; text-align: center; }
#Days { grid-template-columns: 1fr 1fr 1fr 1fr 1fr 1fr 1fr; gap: 2px; }
.weekday { text-align: center; color: var(--ink-muted); }
.day { width: stretch; }
.day.outside { opacity: 0.45; }
.day.marked { outline: 2px solid var(--accent); outline-offset: -2px; }
</style>
