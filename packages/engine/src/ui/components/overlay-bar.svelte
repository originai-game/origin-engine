<meta name="component" content="OverlayBar">
<meta name="strings" content="./overlay-bar.en.ogt">
<meta name="prop:overlays" content="list" data-doc="The map overlays, a keyed collection of { id, name, icon, image, ramp, low, high }: icon an icon name shown when there is no image, image a texture handle (0: none), ramp the legend's CSS gradient stops low to high (#2b6cb0, #f6e05e, #e53e3e), low and high the words at its ends.">
<meta name="prop:selected" content="string" data-doc="The id of the overlay shown (its legend over the bar, its button lit and named); empty is none.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the bar; empty is Overlays.">
<meta name="action:choose" content="Show">
<meta name="fixture:default" content='{"selected":"heat","label":"","overlays":[{"id":"heat","name":"Heat","icon":"fire","image":0,"ramp":"#2b6cb0, #f6e05e, #e53e3e","low":"Cold","high":"Hot"},{"id":"water","name":"Water","icon":"drop","image":0,"ramp":"#d9e8f5, #2b6cb0","low":"Dry","high":"Flooded"},{"id":"crime","name":"Crime","icon":"skull","image":0,"ramp":"#38a169, #e53e3e","low":"Safe","high":"Dangerous"},{"id":"land","name":"Land value","icon":"coin","image":0,"ramp":"#4a5568, #ecc94b","low":"Low","high":"High"}]}'>

<div id="OverlayBar">
  {#each overlays as o (o.id)}
    {#if o.id == selected}
      <div id="Legend" role="status" aria-label={o.name + ": " + o.low + " – " + o.high}>
        <span id="Name">{o.name}</span>
        <div id="Ramp" style:--ramp={"linear-gradient(90deg, " + o.ramp + ")"}></div>
        <div id="Ends">
          <span class="end">{o.low}</span>
          <span class="end high">{o.high}</span>
        </div>
      </div>
    {/if}
  {/each}
  <nav id="Bar" role="toolbar" aria-label={label || t("component.overlay-bar.name")}>
    {#each overlays as o (o.id)}
      <button id="Overlay" class="overlay" aria-label={o.name} aria-pressed={o.id == selected} onclick={choose}>
        <div class="face">
          <img class="art" src={o.image} hidden={!o.image}>
          <div class="well" hidden={!!o.image}><icon class="glyph" icon={o.icon}></icon></div>
          <span class="name">{o.name}</span>
        </div>
      </button>
    {/each}
  </nav>
</div>

<style>
#OverlayBar { gap: 8px; align-items: center; }
#Legend { width: 308px; padding: 10px 14px; gap: 6px; border-radius: 12px; background: #1b1f27eb; border: 1px solid #ffffff1f; box-shadow: 0 4px 16px #00000073; }
#Name { font: var(--text-sm); font-family: var(--font-display); text-transform: uppercase; color: var(--ink); white-space: nowrap; }
#Ramp { width: 280px; height: 10px; border-radius: 5px; background: var(--ramp); border: 1px solid #ffffff33; }
#Ends { display: flex; gap: 12px; width: stretch; }
.end { width: stretch; font: var(--text-caption); color: var(--ink-muted); white-space: nowrap; }
.high { text-align: end; }
#Bar { display: flex; gap: 8px; padding: 8px; border-radius: 36px; background: var(--panel); -og-frame: panel; overflow-x: auto; scrollbar-width: none; }
.overlay { height: 56px; padding: 4px 14px 4px 4px; border-radius: 28px; background: #00000099; }
.overlay[aria-pressed=true] { background: #4c8dff2e; outline: 2px solid var(--primary); box-shadow: 0 0 10px #4c8dff73; }
.face { display: flex; align-items: center; gap: 8px; }
.art, .well { width: 48px; height: 48px; border-radius: 24px; }
.well { background: var(--control-strong); align-items: center; justify-content: center; }
.glyph { width: 26px; height: 26px; color: var(--ink-muted); }
.overlay[aria-pressed=true] .glyph { color: var(--ink); }
.name { font: var(--text-sm); font-family: var(--font-display); color: var(--ink-muted); white-space: nowrap; }
.overlay[aria-pressed=true] .name { color: var(--ink); }
</style>
