<meta name="component" content="OverlayBar">
<meta name="strings" content="./overlay-bar.en.ogt">
<meta name="prop:overlays" content="list" data-doc="The map overlays, a keyed collection of { id, name, icon, image, ramp, low, high }: icon an icon name shown when there is no image, image a texture handle (0: none), ramp the legend's CSS gradient stops low to high (#2b6cb0, #f6e05e, #e53e3e), low and high the words at its ends.">
<meta name="prop:selected" content="string" data-doc="The id of the overlay shown (its legend over the bar, its button lit and named); empty is none.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the bar; empty is Overlays.">
<meta name="action:choose" content="Show">
<meta name="fixture:default" content='{"selected":"heat","label":"","overlays":[{"id":"heat","name":"Heat","icon":"fire","image":0,"ramp":"#2b6cb0, #f6e05e, #e53e3e","low":"Cold","high":"Hot"},{"id":"water","name":"Water","icon":"drop","image":0,"ramp":"#d9e8f5, #2b6cb0","low":"Dry","high":"Flooded"},{"id":"crime","name":"Crime","icon":"skull","image":0,"ramp":"#38a169, #e53e3e","low":"Safe","high":"Dangerous"},{"id":"land","name":"Land value","icon":"coin","image":0,"ramp":"#4a5568, #ecc94b","low":"Low","high":"High"}]}'>

<div id="OverlayBar" layout="stack">
  {#each overlays as o (o.id)}
    {#if o.id == selected}
      <div id="Legend" part="frame" variant="hud" layout="stack" gap="tight" role="status" aria-label={o.name + ": " + o.low + " – " + o.high}>
        <span class="title" id="Name" part="title">{o.name}</span>
        <div class="ramp" id="Ramp" part="fill" style:--fill-color={"linear-gradient(90deg, " + o.ramp + ")"}></div>
        <div id="Ends" layout="row" gap="loose">
          <span class="end label" part="caption" fill>{o.low}</span>
          <span class="end high label" part="caption" fill>{o.high}</span>
        </div>
      </div>
    {/if}
  {/each}
  <nav class="bar" id="Bar" part="frame" variant="hud" layout="row" role="toolbar" aria-label={label || t("component.overlay-bar.name")}>
    {#each overlays as o (o.id)}
      <button id="Overlay" part="cell" class="overlay option" aria-label={o.name} aria-pressed={o.id == selected} onclick={choose}>
        <div class="face" layout="row">
          {#if o.image}<img class="art" part="picture" src={o.image}>
          {:else}<div class="well cover" part="well" layout="stack" fill gap="none"><og-icon class="glyph icon" part="icon" icon={o.icon}></og-icon></div>{/if}
          <span class="name" part="name">{o.name}</span>
        </div>
      </button>
    {/each}
  </nav>
</div>

<style>
.cover { justify-content: center; align-items: center; }
#OverlayBar { align-items: center; max-width: 100%; text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#Legend { --title-size: var(--title-small-size); width: var(--measure-sm); max-width: 100%; padding: var(--space-sm) var(--space-md); }
.title { white-space: nowrap; }
.ramp { height: var(--space-sm); --fill-gloss: none; }
.high { text-align: end; }
.bar { max-width: 100%; padding: var(--space-sm); overflow-x: auto; }
.option { flex: none; height: calc(var(--size-lg) + var(--space-xs) * 2); padding: var(--space-xs) var(--space-md) var(--space-xs) var(--space-xs); }
.art, .well { width: var(--size-lg); height: var(--size-lg); }
.icon { width: var(--size-sm); height: var(--size-sm); }
.name { white-space: nowrap; }
</style>
