<meta name="component" content="OverlayBar">
<meta name="strings" content="./overlay-bar.en.ogt">
<meta name="parts" content="OverlayBar.frame OverlayBar.title OverlayBar.ramp OverlayBar.label OverlayBar.bar OverlayBar.option OverlayBar.art OverlayBar.well OverlayBar.icon OverlayBar.name">
<meta name="prop:overlays" content="list" data-doc="The map overlays, a keyed collection of { id, name, icon, image, ramp, low, high }: icon an icon name shown when there is no image, image a texture handle (0: none), ramp the legend's CSS gradient stops low to high (#2b6cb0, #f6e05e, #e53e3e), low and high the words at its ends.">
<meta name="prop:selected" content="string" data-doc="The id of the overlay shown (its legend over the bar, its button lit and named); empty is none.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the bar; empty is Overlays.">
<meta name="action:choose" content="Show">
<meta name="fixture:default" content='{"selected":"heat","label":"","overlays":[{"id":"heat","name":"Heat","icon":"fire","image":0,"ramp":"#2b6cb0, #f6e05e, #e53e3e","low":"Cold","high":"Hot"},{"id":"water","name":"Water","icon":"drop","image":0,"ramp":"#d9e8f5, #2b6cb0","low":"Dry","high":"Flooded"},{"id":"crime","name":"Crime","icon":"skull","image":0,"ramp":"#38a169, #e53e3e","low":"Safe","high":"Dangerous"},{"id":"land","name":"Land value","icon":"coin","image":0,"ramp":"#4a5568, #ecc94b","low":"Low","high":"High"}]}'>

<div id="OverlayBar">
  {#each overlays as o (o.id)}
    {#if o.id == selected}
      <div id="Legend" part="OverlayBar.frame" class="hud" role="status" aria-label={o.name + ": " + o.low + " – " + o.high}>
        <span id="Name" part="OverlayBar.title">{o.name}</span>
        <div id="Ramp" part="OverlayBar.ramp" style:--ramp={"linear-gradient(90deg, " + o.ramp + ")"}></div>
        <div id="Ends">
          <span class="end" part="OverlayBar.label">{o.low}</span>
          <span class="end high" part="OverlayBar.label">{o.high}</span>
        </div>
      </div>
    {/if}
  {/each}
  <nav id="Bar" part="OverlayBar.bar" role="toolbar" aria-label={label || t("component.overlay-bar.name")}>
    {#each overlays as o (o.id)}
      <button id="Overlay" part="OverlayBar.option" class="overlay" aria-label={o.name} aria-pressed={o.id == selected} onclick={choose}>
        <div class="face">
          <img class="art" part="OverlayBar.art" src={o.image} hidden={!o.image}>
          <div class="well" part="OverlayBar.well" hidden={!!o.image}><og-icon class="glyph" part="OverlayBar.icon" icon={o.icon}></og-icon></div>
          <span class="name" part="OverlayBar.name">{o.name}</span>
        </div>
      </button>
    {/each}
  </nav>
</div>

<style>
#OverlayBar { gap: var(--space-sm); align-items: center; }
[part="OverlayBar.frame"] { --title-size: var(--title-small-size); width: var(--measure-sm); gap: var(--space-xs); }
[part="OverlayBar.title"] { white-space: nowrap; }
[part="OverlayBar.ramp"] { width: stretch; height: var(--space-sm); border-radius: var(--radius-capsule); background: var(--ramp); border: var(--stroke-hairline) solid var(--line); }
#Ends { display: flex; gap: var(--space-md); width: stretch; }
[part="OverlayBar.label"] { width: stretch; font: var(--text-caption); white-space: nowrap; }
.high { text-align: end; }
[part="OverlayBar.bar"] { display: flex; gap: var(--space-sm); padding: var(--space-sm); border-radius: var(--radius-capsule); backdrop-filter: blur(var(--glass-blur-thick)) saturate(var(--glass-saturation)); -og-material: glass thick lift; overflow-x: auto; scrollbar-width: none; }
[part="OverlayBar.option"] { height: calc(var(--size-lg) + var(--space-xs) * 2); padding: var(--space-xs) var(--space-md) var(--space-xs) var(--space-xs); border-radius: var(--radius-capsule); background: var(--control); }
[part="OverlayBar.option"][aria-pressed=true] { background: var(--selection); outline: var(--stroke-regular) solid var(--primary); box-shadow: var(--shadow-slot-selected); }
.face { display: flex; align-items: center; gap: var(--space-sm); }
[part="OverlayBar.art"], [part="OverlayBar.well"] { width: var(--size-lg); height: var(--size-lg); border-radius: var(--radius-capsule); }
[part="OverlayBar.well"] { align-items: center; justify-content: center; }
[part="OverlayBar.icon"] { width: var(--size-sm); height: var(--size-sm); color: var(--ink-muted); }
[aria-pressed=true] [part="OverlayBar.icon"] { color: var(--on-selection); }
[part="OverlayBar.name"] { font: var(--text-sm); font-family: var(--font-display); color: var(--ink-muted); white-space: nowrap; }
[aria-pressed=true] [part="OverlayBar.name"] { color: var(--on-selection); }
</style>
