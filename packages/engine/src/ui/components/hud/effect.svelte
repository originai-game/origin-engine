<meta name="component" content="HudEffect">
<meta name="strings" content="./effect.en.ogt">
<meta name="prop:name" content="string" data-doc="What a reader calls it (Haste).">
<meta name="prop:icon" content="number" data-doc="Its art, a texture (og.load.texture); 0 draws the glyph in its place.">
<meta name="prop:glyph" content="string" data-doc="An icon of the atlas, drawn when there is no art; empty is none.">
<meta name="prop:remaining" content="number" data-doc="The share of its time still to run, 0 to 1: a dark radial sweep over the art; negative is no timer.">
<meta name="prop:label" content="string" data-doc="Its time or stacks as the game words it (0:12, ×3), on a tag under the tile; empty is none.">
<meta name="prop:harmful" content="bool" data-doc="A debuff (on data-harmful, never colour alone: the edge and the tag's ink both change).">
<meta name="fixture:default" content='{"name":"Haste","icon":0,"glyph":"bolt","remaining":0.7,"label":"0:12","harmful":false}'>
<meta name="fixture:harmful" content='{"name":"Hex","icon":0,"glyph":"skull","remaining":0.3,"label":"4.2","harmful":true}'>
<meta name="fixture:untimed" content='{"name":"Blessed","icon":0,"glyph":"star","remaining":-1,"label":"","harmful":false}'>
<meta name="fixture:empty" content='{"name":"","icon":0,"glyph":"","remaining":-1,"label":"","harmful":false}'>
<meta name="fixture:overflow" content='{"name":"Searing brand of the Ninefold Gate","icon":0,"glyph":"fire","remaining":0.1,"label":"×99 59.9","harmful":true}'>

<!--
  One buff or debuff of a HUD, a block an effects row is built
  from: a small tile with two corners cut, its edge green for a boon and
  red for a harm, the time still to run as a dark radial sweep over its
  art and its time or stacks on a tag under it. The game owns every
  timer; read only.
-->
<div id="HudEffect" part="effect" layout="stack" gap="none" role="group" class:harmful={harmful} aria-label={label ? t("hud.effect.label", { name: name, label: label }) : name}>
  <div part="tile" class="tile" layout="overlay">
    {#if icon}<img part="art" class="art" anchor="fill" src={icon} alt="">{:else}<div class="blank" anchor="fill" layout="center">{#if glyph}<og-icon class="glyph" icon={glyph}></og-icon>{/if}</div>{/if}
    {#if remaining >= 0}<div part="timer" class="timer" anchor="fill" style:--left={remaining >= 1 ? 1 : remaining}></div>{/if}
  </div>
  {#if label}<span part="time" class="tag">{label}</span>{/if}
</div>

<style>
#HudEffect { inline-size: fit-content; max-inline-size: 100%; align-items: center; row-gap: calc(3 * var(--font-step-px)); color: var(--hud-ink); filter: var(--hud-lift); }
/* The tile: the edge is its fill, the art inset inside it. */
.tile { inline-size: calc(30 * var(--font-step-px)); block-size: calc(30 * var(--font-step-px)); background: var(--hud-boon); border-radius: var(--hud-effect-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; --layout-anchor-inset: calc(2 * var(--font-step-px)); }
.harmful .tile { background: var(--hud-harm); }
.art, .blank, .timer { border-radius: var(--hud-effect-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.art { object-fit: cover; }
.blank { background: var(--hud-plate-fill); }
.glyph { inline-size: 60%; block-size: 60%; color: var(--hud-ink); }
/* The time run out is dark; the sweep turns clockwise from the top. */
.timer { background: var(--hud-sweep); mask-image: conic-gradient(transparent calc(var(--left) * 360deg), #000 0); }
.tag { max-inline-size: calc(64 * var(--font-step-px)); padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.harmful .tag { color: var(--hud-effect-harm-ink); }
@media (width < 600px) { .tile { inline-size: calc(26 * var(--font-step-px)); block-size: calc(26 * var(--font-step-px)); } .tag { padding: 0 calc(3 * var(--font-step-px)); } }
</style>
