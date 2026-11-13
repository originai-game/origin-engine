<meta name="component" content="Chart">
<meta name="strings" content="./chart.en.ogt">
<meta name="parts" content="Chart.frame Chart.title Chart.subtitle Chart.tick Chart.grid Chart.bar Chart.area Chart.line Chart.label">
<meta name="prop:title" content="string" data-doc="A title over it; empty is none.">
<meta name="prop:subtitle" content="string" data-doc="A line under the title.">
<meta name="prop:max" content="number" data-doc="The value axis' top (its bottom is 0).">
<meta name="prop:bars" content="list" data-doc="Bars, one per category, left to right: each { id, label, value, color }; color #rrggbb, empty is the primary.">
<meta name="prop:line" content="string" data-doc="A line series as SVG path data in the plot's 350 by 180 px box (y down), drawn over the bars; empty is none. The game builds it from its values (M x y L x y …).">
<meta name="prop:area" content="bool" data-doc="Fill under the line.">
<meta name="prop:ticks" content="list" data-doc="Value axis gridlines: each { id, at, label }, at a fraction of max (0 to 1).">
<meta name="fixture:default" content='{"title":"Weekly kills","subtitle":"Last 7 days","max":100,"area":true,"line":"M24 112 L74 86 L122 99 L175 54 L224 68 L273 32 L322 43","ticks":[{"id":"0","at":0,"label":"0"},{"id":"50","at":0.5,"label":"50"},{"id":"100","at":1,"label":"100"}],"bars":[{"id":"mon","label":"Mon","value":32,"color":""},{"id":"tue","label":"Tue","value":51,"color":""},{"id":"wed","label":"Wed","value":44,"color":""},{"id":"thu","label":"Thu","value":70,"color":""},{"id":"fri","label":"Fri","value":62,"color":"#e5484d"},{"id":"sat","label":"Sat","value":88,"color":""},{"id":"sun","label":"Sun","value":79,"color":""}]}'>

<div id="Chart" part="Chart.frame" role="figure" aria-label={title || t("component.chart.name")}>
  <span id="Title" part="Chart.title" hidden={!title}>{title}</span>
  <span id="Subtitle" part="Chart.subtitle" hidden={!subtitle}>{subtitle}</span>
  <div id="Body">
    <div id="Axis">
      {#each ticks as tick (tick.id)}<span part="Chart.tick" class="tick" style:--at={tick.at}>{tick.label}</span>{/each}
    </div>
    <div id="Plot">
      {#each ticks as tick (tick.id)}<div part="Chart.grid" class="grid" style:--at={tick.at}></div>{/each}
      <div id="Bars">
        {#each bars as bar (bar.id)}
          <div class="column" aria-label={bar.label + " " + formatNumber(bar.value)}>
            <div part="Chart.bar" class="bar" class:tinted={!!bar.color} style:--v={bar.value / max} style:--c={bar.color}></div>
          </div>
        {/each}
      </div>
      <svg id="Line" viewBox="0 0 350 180" hidden={!line}>
        <path id="Area" part="Chart.area" d={line + " L322 180 L25 180 Z"} hidden={!area}></path>
        <path id="Stroke" part="Chart.line" d={line} fill="none"></path>
      </svg>
    </div>
  </div>
  <div id="Categories">
    {#each bars as bar (bar.id)}<span part="Chart.label" class="category">{bar.label}</span>{/each}
  </div>
</div>

<style>
#Chart { width: var(--measure-lg); gap: var(--space-xs); }
#Subtitle { font: var(--text-caption); color: var(--ink-muted); }
#Body { display: flex; gap: var(--space-xs); height: var(--size-5xl); margin-top: var(--space-sm); }
#Axis { width: var(--size-md); height: stretch; }
.tick { position: absolute; inset-inline-start: 0; bottom: calc(var(--at) * var(--size-5xl) - var(--at) * var(--size-xs)); font: var(--text-caption); color: var(--ink-muted); }
#Plot { flex: 1; height: stretch; }
.grid { position: absolute; inset-inline-start: 0; bottom: calc(var(--at) * var(--size-5xl)); width: stretch; height: var(--stroke-hairline); background: var(--line); }
#Bars { display: flex; height: stretch; width: stretch; gap: var(--space-sm); align-items: end; padding: 0 var(--space-sm); }
.column { width: stretch; height: stretch; justify-content: end; }
.bar { width: stretch; height: calc(var(--v) * var(--size-5xl)); border-radius: var(--radius-control); background: var(--primary); opacity: var(--opacity-soft); }
.bar.tinted { background: var(--c); }
#Line { position: absolute; top: 0; inset-inline-start: 0; width: stretch; height: stretch; }
#Area { fill: var(--gold); fill-opacity: var(--opacity-faint); }
#Stroke { stroke: var(--gold); stroke-width: 2; stroke-linejoin: round; }
#Categories { display: flex; gap: var(--space-sm); padding: 0 var(--space-sm) 0 calc(var(--size-md) + var(--space-xs) + var(--space-sm)); }
.category { width: stretch; text-align: center; font-size: var(--font-step-sm); }
</style>
