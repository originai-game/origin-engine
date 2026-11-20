<meta name="component" content="OverlayBar">
<meta name="strings" content="./overlay-bar.en.ogt">
<meta name="parts" content="OverlayBar.frame OverlayBar.title OverlayBar.ramp OverlayBar.label OverlayBar.bar OverlayBar.option OverlayBar.art OverlayBar.well OverlayBar.icon OverlayBar.name OverlayBar.picture OverlayBar.fill OverlayBar.caption OverlayBar.cell">
<meta name="prop:overlays" content="list" data-doc="The map overlays, a keyed collection of { id, name, icon, image, ramp, low, high }: icon an icon name shown when there is no image, image a texture handle (0: none), ramp the legend's CSS gradient stops low to high (#2b6cb0, #f6e05e, #e53e3e), low and high the words at its ends.">
<meta name="prop:selected" content="string" data-doc="The id of the overlay shown (its legend over the bar, its button lit and named); empty is none.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the bar; empty is Overlays.">
<meta name="action:choose" content="Show">
<meta name="fixture:default" content='{"selected":"heat","label":"","overlays":[{"id":"heat","name":"Heat","icon":"fire","image":0,"ramp":"#2b6cb0, #f6e05e, #e53e3e","low":"Cold","high":"Hot"},{"id":"water","name":"Water","icon":"drop","image":0,"ramp":"#d9e8f5, #2b6cb0","low":"Dry","high":"Flooded"},{"id":"crime","name":"Crime","icon":"skull","image":0,"ramp":"#38a169, #e53e3e","low":"Safe","high":"Dangerous"},{"id":"land","name":"Land value","icon":"coin","image":0,"ramp":"#4a5568, #ecc94b","low":"Low","high":"High"}]}'>

<div id="OverlayBar" data-layout="stack">
  {#each overlays as o (o.id)}
    {#if o.id == selected}
      <div id="Legend" part="OverlayBar.frame" class="hud" data-layout="stack" data-gap="tight" role="status" aria-label={o.name + ": " + o.low + " – " + o.high}>
        <span id="Name" part="OverlayBar.title">{o.name}</span>
        <div id="Ramp" part="OverlayBar.ramp OverlayBar.fill" style:--fill-color={"linear-gradient(90deg, " + o.ramp + ")"}></div>
        <div id="Ends" data-layout="row" data-gap="loose">
          <span class="end" part="OverlayBar.label OverlayBar.caption" data-fill>{o.low}</span>
          <span class="end high" part="OverlayBar.label OverlayBar.caption" data-fill>{o.high}</span>
        </div>
      </div>
    {/if}
  {/each}
  <nav id="Bar" part="OverlayBar.bar OverlayBar.frame" class="hud" data-layout="row" role="toolbar" aria-label={label || t("component.overlay-bar.name")}>
    {#each overlays as o (o.id)}
      <button id="Overlay" part="OverlayBar.option OverlayBar.cell" class="overlay" aria-label={o.name} aria-pressed={o.id == selected} onclick={choose}>
        <div class="face" data-layout="row">
          {#if o.image}<img class="art" part="OverlayBar.art OverlayBar.picture" src={o.image}>
          {:else}<div class="well" part="OverlayBar.well" data-layout="cover" data-gap="none"><og-icon class="glyph" part="OverlayBar.icon" icon={o.icon}></og-icon></div>{/if}
          <span class="name" part="OverlayBar.name">{o.name}</span>
        </div>
      </button>
    {/each}
  </nav>
</div>

<style>
:host { align-items: center; max-width: 100%; text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#Legend { --title-size: var(--title-small-size); width: var(--measure-sm); max-width: 100%; padding: var(--space-sm) var(--space-md); }
[part="OverlayBar.title"] { white-space: nowrap; }
[part~="OverlayBar.ramp"] { height: var(--space-sm); --fill-gloss: none; }
.high { text-align: end; }
[part~="OverlayBar.bar"] { max-width: 100%; padding: var(--space-sm); overflow-x: auto; }
[part~="OverlayBar.option"] { flex: none; height: calc(var(--size-lg) + var(--space-xs) * 2); padding: var(--space-xs) var(--space-md) var(--space-xs) var(--space-xs); }
[part~="OverlayBar.art"], [part="OverlayBar.well"] { width: var(--size-lg); height: var(--size-lg); }
[part="OverlayBar.icon"] { width: var(--size-sm); height: var(--size-sm); }
[part="OverlayBar.name"] { white-space: nowrap; }
</style>
