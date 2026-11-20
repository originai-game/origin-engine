<meta name="component" content="HudPortrait">
<meta name="prop:portrait" content="number" data-doc="A head-and-shoulders texture (og.load.texture); 0 draws the person glyph.">
<meta name="prop:dimmed" content="bool" data-doc="Out of the fight: the art greyed.">
<meta name="fixture:default" content='{"portrait":0,"dimmed":false}'>
<meta name="fixture:dimmed" content='{"portrait":0,"dimmed":true}'>

<!--
  A portrait of a HUD, a block a plate is built from: the art (or
  the person glyph) inset in an octagon with a lit edge. Its side is
  --hud-portrait-size; a plate sets it on the place it gives the portrait.
  Read only.
-->
<div id="HudPortrait" part="rim" layout="overlay" class:dimmed={dimmed}>
  {#if portrait}<img id="Face" part="face" anchor="fill" src={portrait} alt="">{:else}<div id="Blank" anchor="fill" layout="center"><og-icon id="Glyph" icon="person"></og-icon></div>{/if}
</div>

<style>
#HudPortrait { flex: none; inline-size: var(--hud-portrait-size); block-size: var(--hud-portrait-size); clip-path: var(--hud-octagon); background: var(--hud-edge); --layout-anchor-inset: calc(2 * var(--font-step-px)); }
#Face, #Blank { clip-path: var(--hud-octagon); background: var(--hud-plate-fill); }
#Face { object-fit: cover; }
.dimmed #Face { filter: var(--hud-unready); }
#Glyph { inline-size: 50%; block-size: 50%; color: var(--hud-ink-dim); }
</style>
