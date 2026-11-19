<meta name="component" content="ShooterEffects">
<meta name="strings" content="./shooter.en.ogt">
<meta name="parts" content="ShooterEffects.frame ShooterEffects.effect ShooterEffects.tile ShooterEffects.art ShooterEffects.timer ShooterEffects.time">
<meta name="prop:effects" content="list" data-doc="The effects on the player in the game's order, keyed rows of { id, name, icon, glyph, remaining, label, harmful }: name for a reader; icon a texture (0: the glyph, an icon name, in its place); remaining the share of its time still to run, 0 to 1 (a dark sweep over the art; negative: no timer); label its time or stacks as the game words it, on a tag (0:12, ×3), empty for none; harmful a debuff (a danger edge and tag; on data-harmful, never colour alone: the tag's ink and the edge both change).">
<meta name="fixture:default" content='{"effects":[{"id":"haste","name":"Haste","icon":0,"glyph":"bolt","remaining":0.7,"label":"0:12","harmful":false},{"id":"fury","name":"Fury","icon":0,"glyph":"sword","remaining":0.35,"label":"×3 4.2","harmful":false},{"id":"burn","name":"Burning","icon":0,"glyph":"fire","remaining":0.55,"label":"6","harmful":true}]}'>
<meta name="fixture:untimed" content='{"effects":[{"id":"armor","name":"Armored","icon":0,"glyph":"shield","remaining":-1,"label":"","harmful":false}]}'>
<meta name="fixture:empty" content='{"effects":[]}'>
<meta name="fixture:overflow" content='{"effects":[{"id":"e1","name":"A","icon":0,"glyph":"bolt","remaining":0.9,"label":"12:34:56","harmful":false},{"id":"e2","name":"B","icon":0,"glyph":"fire","remaining":0.1,"label":"×99 59.9","harmful":true},{"id":"e3","name":"C","icon":0,"glyph":"drop","remaining":0.5,"label":"1","harmful":true},{"id":"e4","name":"D","icon":0,"glyph":"heart","remaining":0.2,"label":"2","harmful":false},{"id":"e5","name":"E","icon":0,"glyph":"star","remaining":0.4,"label":"3","harmful":false},{"id":"e6","name":"F","icon":0,"glyph":"skull","remaining":0.6,"label":"4","harmful":true},{"id":"e7","name":"G","icon":0,"glyph":"gem","remaining":0.8,"label":"5","harmful":false},{"id":"e8","name":"H","icon":0,"glyph":"key","remaining":1,"label":"6","harmful":false}]}'>

<!--
  The effects on the player, over the vitals: each a small tile with
  two corners cut, its edge green for a boon and red for a harm, the time
  still to run as a dark sweep over its art and its time or stacks on a
  tag under it. The game owns every timer; read only.
-->
<ul id="ShooterEffects" part="ShooterEffects.frame" class="hud bare" data-layout="cluster" data-gap="tight" aria-label={t("shooter.effects.name")}>
  {#each effects as e (e.id)}
  <li part="ShooterEffects.effect" class="effect" data-layout="stack" data-gap="none" data-harmful={e.harmful} aria-label={e.label ? t("shooter.effects.effect", { name: e.name, label: e.label }) : e.name}>
    <div part="ShooterEffects.tile" class="tile" data-layout="overlay">
      {#if e.icon}<img part="ShooterEffects.art" class="art" data-anchor="fill" src={e.icon} alt="">{:else}<div class="blank" data-anchor="fill" data-layout="center">{#if e.glyph}<og-icon class="glyph" icon={e.glyph}></og-icon>{/if}</div>{/if}
      {#if e.remaining >= 0}<div part="ShooterEffects.timer" class="timer" data-anchor="fill" style:--left={e.remaining >= 1 ? 1 : e.remaining}></div>{/if}
    </div>
    {#if e.label}<span part="ShooterEffects.time" class="tag">{e.label}</span>{/if}
  </li>
  {/each}
</ul>

<style>
:host { margin: 0; padding: 0; list-style: none; max-inline-size: stretch; color: var(--hud-ink); --cut: calc(5 * var(--font-step-px)); }
.effect { align-items: center; row-gap: calc(3 * var(--font-step-px)); filter: var(--hud-lift); }
/* The tile: the edge is its fill, the art inset inside it. */
.tile { inline-size: calc(32 * var(--font-step-px)); block-size: calc(32 * var(--font-step-px)); background: var(--hud-boon); clip-path: polygon(var(--cut) 0, 100% 0, 100% calc(100% - var(--cut)), calc(100% - var(--cut)) 100%, 0 100%, 0 var(--cut)); --layout-anchor-inset: calc(2 * var(--font-step-px)); }
[data-harmful=true] .tile { background: var(--hud-harm); }
.art, .blank, .timer { clip-path: polygon(var(--cut) 0, 100% 0, 100% calc(100% - var(--cut)), calc(100% - var(--cut)) 100%, 0 100%, 0 var(--cut)); }
.art { object-fit: cover; }
.blank { background: var(--hud-plate-fill); }
.glyph { inline-size: 60%; block-size: 60%; color: var(--hud-ink); }
/* The time run out is dark; the sweep turns clockwise from the top. */
.timer { background: var(--hud-sweep); mask-image: conic-gradient(transparent calc(var(--left) * 360deg), #000 0); }
.tag { max-inline-size: calc(56 * var(--font-step-px)); padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
[data-harmful=true] .tag { color: var(--hud-foe); }
@media (width < 600px) { .tile { inline-size: calc(24 * var(--font-step-px)); block-size: calc(24 * var(--font-step-px)); } .tag { padding: 0 calc(3 * var(--font-step-px)); } }
</style>
