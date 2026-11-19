<meta name="component" content="RpgEffects">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgEffects.frame RpgEffects.effect RpgEffects.tile RpgEffects.art RpgEffects.timer RpgEffects.time">
<meta name="prop:effects" content="list" data-doc="The buffs and debuffs on a character in the game's order, keyed rows of { id, name, icon, glyph, remaining, label, harmful }: name for a reader; icon a texture (0: the glyph, an icon name, in its place); remaining the share of its time still to run, 0 to 1 (a dark radial sweep over the art; negative: no timer); label its time or stacks as the game words it, on a tag (0:12, ×3), empty for none; harmful a debuff (on data-harmful, never colour alone: the edge and the tag's ink both change).">
<meta name="fixture:default" content='{"effects":[{"id":"haste","name":"Haste","icon":0,"glyph":"bolt","remaining":0.7,"label":"0:12","harmful":false},{"id":"ward","name":"Rune ward","icon":0,"glyph":"shield","remaining":0.42,"label":"×3","harmful":false},{"id":"hex","name":"Hex","icon":0,"glyph":"skull","remaining":0.3,"label":"4.2","harmful":true}]}'>
<meta name="fixture:untimed" content='{"effects":[{"id":"blessed","name":"Blessed","icon":0,"glyph":"star","remaining":-1,"label":"","harmful":false}]}'>
<meta name="fixture:expiring" content='{"effects":[{"id":"haste","name":"Haste","icon":0,"glyph":"bolt","remaining":0.04,"label":"0:01","harmful":false},{"id":"burn","name":"Burning","icon":0,"glyph":"fire","remaining":1,"label":"8.0","harmful":true}]}'>
<meta name="fixture:empty" content='{"effects":[]}'>
<meta name="fixture:overflow" content='{"effects":[{"id":"e1","name":"A","icon":0,"glyph":"bolt","remaining":0.9,"label":"12:34:56","harmful":false},{"id":"e2","name":"B","icon":0,"glyph":"fire","remaining":0.1,"label":"×99 59.9","harmful":true},{"id":"e3","name":"C","icon":0,"glyph":"drop","remaining":0.5,"label":"1","harmful":true},{"id":"e4","name":"D","icon":0,"glyph":"heart","remaining":0.2,"label":"2","harmful":false},{"id":"e5","name":"E","icon":0,"glyph":"star","remaining":0.4,"label":"3","harmful":false},{"id":"e6","name":"F","icon":0,"glyph":"skull","remaining":0.6,"label":"4","harmful":true},{"id":"e7","name":"G","icon":0,"glyph":"gem","remaining":0.8,"label":"5","harmful":false},{"id":"e8","name":"H","icon":0,"glyph":"key","remaining":1,"label":"6","harmful":false},{"id":"e9","name":"I","icon":0,"glyph":"map","remaining":0.3,"label":"7","harmful":false},{"id":"e10","name":"J","icon":0,"glyph":"coin","remaining":0.7,"label":"8","harmful":true}]}'>

<!--
  The buffs and debuffs on the leader, under the vitals: each a small tile
  with two corners cut, its edge green for a boon and red for a harm, the
  time still to run as a dark radial sweep over its art and its time or
  stacks on a tag under it. The game owns every timer; read only.
-->
<ul id="RpgEffects" part="RpgEffects.frame" class="hud bare" data-layout="cluster" data-gap="tight" aria-label={t("rpg.effects.name")}>
  {#each effects as e (e.id)}
  <li part="RpgEffects.effect" class="effect" data-layout="stack" data-gap="none" data-harmful={e.harmful} aria-label={e.label ? t("rpg.effects.effect", { name: e.name, label: e.label }) : e.name}>
    <div part="RpgEffects.tile" class="tile" data-layout="overlay">
      {#if e.icon}<img part="RpgEffects.art" class="art" data-anchor="fill" src={e.icon} alt="">{:else}<div class="blank" data-anchor="fill" data-layout="center">{#if e.glyph}<og-icon class="glyph" icon={e.glyph}></og-icon>{/if}</div>{/if}
      {#if e.remaining >= 0}<div part="RpgEffects.timer" class="timer" data-anchor="fill" style:--left={e.remaining >= 1 ? 1 : e.remaining}></div>{/if}
    </div>
    {#if e.label}<span part="RpgEffects.time" class="tag">{e.label}</span>{/if}
  </li>
  {/each}
</ul>

<style>
:host { margin: 0; padding: 0; list-style: none; column-gap: calc(8 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); --cut: calc(5 * var(--font-step-px)); }
.effect { align-items: center; row-gap: calc(3 * var(--font-step-px)); filter: var(--hud-lift); }
/* The tile: the edge is its fill, the art inset inside it. */
.tile { inline-size: calc(30 * var(--font-step-px)); block-size: calc(30 * var(--font-step-px)); background: var(--hud-boon); clip-path: polygon(var(--cut) 0, 100% 0, 100% calc(100% - var(--cut)), calc(100% - var(--cut)) 100%, 0 100%, 0 var(--cut)); --layout-anchor-inset: calc(2 * var(--font-step-px)); }
[data-harmful=true] .tile { background: var(--hud-harm); }
.art, .blank, .timer { clip-path: polygon(var(--cut) 0, 100% 0, 100% calc(100% - var(--cut)), calc(100% - var(--cut)) 100%, 0 100%, 0 var(--cut)); }
.art { object-fit: cover; }
.blank { background: var(--hud-plate-fill); }
.glyph { inline-size: 60%; block-size: 60%; color: var(--hud-ink); }
/* The time run out is dark; the sweep turns clockwise from the top. */
.timer { background: var(--hud-sweep); mask-image: conic-gradient(transparent calc(var(--left) * 360deg), #000 0); }
.tag { max-inline-size: calc(64 * var(--font-step-px)); padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
[data-harmful=true] .tag { color: var(--hud-harm); }
@media (width < 600px) { :host { column-gap: calc(6 * var(--font-step-px)); } .tile { inline-size: calc(26 * var(--font-step-px)); block-size: calc(26 * var(--font-step-px)); } .tag { padding: 0 calc(3 * var(--font-step-px)); } }
</style>
