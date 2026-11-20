<meta name="component" content="DamageDirection">
<meta name="strings" content="./damage-direction.en.ogt">
<meta name="prop:hits" content="list" data-doc="Recent hits, a keyed collection of { id, angle, strength, kind }: angle in radians from straight ahead, clockwise, relative to the view; strength 0 to 1 (the arc's opacity and length); kind normal, shield (the shield took it) or critical. The game removes a hit when its mark should go.">
<meta name="fixture:default" content='{"hits":[{"id":"h1","angle":0.9,"strength":0.9,"kind":"normal"},{"id":"h2","angle":-2.4,"strength":0.5,"kind":"shield"}]}'>
<meta name="fixture:empty" content='{"hits":[]}'>
<meta name="fixture:crowded" content='{"hits":[{"id":"a","angle":0,"strength":1,"kind":"critical"},{"id":"b","angle":1.57,"strength":0.7,"kind":"normal"},{"id":"c","angle":3.14,"strength":0.4,"kind":"normal"},{"id":"d","angle":-1.57,"strength":0.6,"kind":"shield"},{"id":"e","angle":-0.6,"strength":0.3,"kind":"normal"}]}'>

<div id="DamageDirection" layout="overlay" role="image" aria-label={t("shooter.damage.name")}>
  {#each hits as h (h.id)}
    <div class="hit" anchor="fill" style:--strength={h.strength} transition:fade={{ duration: 300 }}>
      <svg class="ring" viewBox="0 0 100 100"><path class="arc" part="arc" class:shield={h.kind == 'shield'} class:critical={h.kind == 'critical'} d="M 50 4 A 46 46 0 1 1 50 96 A 46 46 0 1 1 50 4" fill="none" pathLength="1" stroke-dasharray={(0.09 + h.strength * 0.07) + " " + (0.91 - h.strength * 0.07)} stroke-dashoffset={(0.045 + h.strength * 0.035) - h.angle * 0.1591549 + 2}></path></svg>
    </div>
  {/each}
</div>

<style>
/* Each hit is a stretch of a ring around the centre, centred where the
   damage came from (a dash of the circle from its top, clockwise), its
   length and weight by the hit's strength, red with a glow (the shield's blue when the
   shield took it). */
#DamageDirection { width: 100%; height: 100%; min-width: var(--size-xxl); min-height: var(--size-xxl); pointer-events: none; --layout-anchor-inset: 0; }
.hit { opacity: calc(0.5 + var(--strength) * 0.5); }
.ring { width: 100%; height: 100%; filter: var(--hud-arc-glow); }
.arc { stroke: var(--hud-arc); stroke-width: 2.5; stroke-linecap: butt; }
.arc.shield { stroke: var(--hud-ally); }
.arc.critical { stroke-width: 3.5; }
</style>
