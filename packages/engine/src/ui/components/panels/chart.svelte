<meta name="component" content="Chart">
<meta name="strings" content="./chart.en.ogt">
<meta name="parts" content="Chart.frame Chart.title Chart.subtitle Chart.tick Chart.grid Chart.bar Chart.area Chart.line Chart.label Chart.plate Chart.caption Chart.fill Chart.series Chart.gridline">
<meta name="prop:title" content="string" data-doc="A title over it; empty is none.">
<meta name="prop:subtitle" content="string" data-doc="A line under the title.">
<meta name="prop:max" content="number" data-default="1" data-doc="The value axis' top (its bottom is 0).">
<meta name="prop:bars" content="list" data-doc="Bars, one per category, left to right: each { id, label, value, color }; color #rrggbb, empty is the primary.">
<meta name="prop:line" content="string" data-doc="A line series as SVG path data in the plot's 350 by 180 px box (y down), drawn over the bars; empty is none. The game builds it from its values (M x y L x y …).">
<meta name="prop:area" content="bool" data-doc="Fill under the line.">
<meta name="prop:ticks" content="list" data-doc="Value axis gridlines: each { id, at, label }, at a fraction of max (0 to 1).">
<meta name="fixture:default" content='{"title":"Weekly kills","subtitle":"Last 7 days","max":100,"area":true,"line":"M24 112 L74 86 L122 99 L175 54 L224 68 L273 32 L322 43","ticks":[{"id":"0","at":0,"label":"0"},{"id":"50","at":0.5,"label":"50"},{"id":"100","at":1,"label":"100"}],"bars":[{"id":"mon","label":"Mon","value":32,"color":""},{"id":"tue","label":"Tue","value":51,"color":""},{"id":"wed","label":"Wed","value":44,"color":""},{"id":"thu","label":"Thu","value":70,"color":""},{"id":"fri","label":"Fri","value":62,"color":"#e5484d"},{"id":"sat","label":"Sat","value":88,"color":""},{"id":"sun","label":"Sun","value":79,"color":""}]}'>

<!-- A HUD plate: a small accent line over the title, the bars leaning as
     the HUD's meters do and lit along their top, the line series in gold
     over them, and the axis in faint tabular figures. -->
<div id="Chart" part="Chart.frame" class="bare" data-layout="stack" data-gap="tight" role="figure" aria-label={title || t("component.chart.name")}>
  {#if title}<span id="Title" part="Chart.title">{title}</span>{/if}
  {#if subtitle}<span id="Subtitle" part="Chart.subtitle Chart.caption" class="subtitle">{subtitle}</span>{/if}
  <div id="Body" data-layout="row" data-gap="tight">
    <div id="Axis">
      {#each ticks as tick (tick.id)}<span part="Chart.tick Chart.caption" class="tick" style:--at={tick.at}>{tick.label}</span>{/each}
    </div>
    <div id="Plot" data-fill>
      {#if !bars.length && !line}<p class="empty" part="Chart.label Chart.caption" role="status">{t("component.chart.empty")}</p>{/if}
      {#each ticks as tick (tick.id)}<div part="Chart.grid Chart.plate Chart.gridline" class="grid" data-og-overlap="*" style:--at={tick.at}></div>{/each}
      <div id="Bars" data-layout="row">
        {#each bars as bar (bar.id)}
          <div class="column" data-layout="stack" data-gap="none" data-fill aria-label={bar.label + " " + formatNumber(bar.value)}>
            <div part="Chart.bar Chart.fill" class="bar" class:tinted={!!bar.color} style:--v={bar.value / max} style:--c={bar.color}></div>
          </div>
        {/each}
      </div>
      {#if line}<svg id="Line" viewBox="0 0 350 180">
        {#if area}<path id="Area" part="Chart.area" d={line + " L322 180 L25 180 Z"}></path>{/if}
        <path id="Stroke" part="Chart.line Chart.series" d={line} fill="none"></path>
      </svg>{/if}
    </div>
  </div>
  <div id="Categories">
    <!-- Each name stands on one line under the middle of its bar. With more
         bars than names fit under, every other bar is named, and every
         third in a narrow chart. -->
    <div id="Ticks" class:dense={bars.length > 8} style:--n={bars.length}>
      {#each bars as bar, i (bar.id)}<span part="Chart.label Chart.caption" class="category" style:--i={i}>{bar.label}</span>{/each}
    </div>
  </div>
</div>

<style>
:host { inline-size: var(--measure-lg); max-inline-size: 100%; container-type: inline-size; --series-color: var(--hud-gold); --gridline-ink: var(--data-rule); }
#Chart { padding: var(--space-md) var(--space-lg) var(--space-md); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(calc(2 * var(--hud-plate-cut)) 0, 100% 0, 100% calc(100% - 2 * var(--hud-plate-cut)), calc(100% - 2 * var(--hud-plate-cut)) 100%, 0 100%, 0 calc(2 * var(--hud-plate-cut))); color: var(--hud-ink); }
#Title { font-size: var(--hud-size-sm); font-weight: var(--data-title-weight); color: var(--hud-ink); overflow-wrap: anywhere; }
.subtitle { order: -1; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent); overflow-wrap: anywhere; }
#Body { height: var(--size-5xl); margin-top: var(--space-sm); align-items: stretch; }
#Axis { position: relative; width: var(--size-md); flex: none; }
.tick { position: absolute; inset-inline-start: 0; bottom: calc(var(--at) * var(--size-5xl) - var(--at) * var(--size-xs)); font-size: var(--hud-label-size); font-variant-numeric: var(--hud-figure-numeric); color: var(--data-head-ink); }
#Plot { position: relative; border-inline-start: var(--data-guide-line); }
.empty { margin: 0; padding: var(--space-md); color: var(--hud-ink-dim); }
.grid { position: absolute; inset-inline-start: 0; inset-inline-end: 0; bottom: calc(var(--at) * var(--size-5xl)); height: var(--stroke-hairline); background: var(--data-rule); }
#Bars { block-size: 100%; align-items: end; padding: 0 var(--space-md) 0 var(--space-sm); column-gap: var(--space-xs); }
.column { height: stretch; justify-content: end; }
.bar { width: stretch; height: calc(var(--v) * var(--size-5xl)); transform: skewX(var(--data-bar-skew)); transform-origin: 50% 100%; background: var(--data-bar-fill); box-shadow: var(--data-bar-rim); }
.bar.tinted { --data-bar-fill: var(--data-bar-gloss), var(--c); }
#Line { position: absolute; inset: 0; inline-size: 100%; block-size: 100%; }
#Categories { padding: 0 var(--space-md) 0 calc(var(--size-md) + var(--space-xs) + var(--space-sm)); }
.category { position: absolute; top: 0; inset-inline-start: calc((var(--i) + 0.5) * (100% + var(--space-xs)) / var(--n) - var(--space-xs) / 2); translate: -50% 0; white-space: nowrap; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); font-stretch: var(--hud-figure-stretch); color: var(--data-head-ink); }
#Ticks { position: relative; inline-size: 100%; block-size: calc(var(--hud-label-size) * 1.5); }
.dense .category:nth-child(even) { display: none; }
@container (width < 520px) {
  .dense .category:nth-child(even) { display: block; }
  .dense .category:not(:nth-child(3n + 1)) { display: none; }
}
</style>
